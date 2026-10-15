//! One native document owner on a dedicated worker thread.
//!
//! Successful submissions are FIFO, including edits and save acknowledgements.
//! Only renders are coalesced: a newer accepted render supersedes older renders,
//! and receipt suppresses frames from a document/revision/font generation that
//! has since changed. Every accepted request retains one completion credit until
//! receipt, so an unread completion cannot make the worker block or lose edits.
//!
//! Limits cover outstanding request count and retained input/output byte buffers,
//! not the expanded document, editor snapshots, serializer reports or Skia working
//! memory. Hosts must also bound runtime count and their own received buffers.
//! Cancellation succeeds only before the worker claims a request. `close`/Drop
//! discard queued work and completions without joining: an active synchronous
//! parse/serialization/Skia call finishes on its owner thread before its document
//! and memory are released. No running native call is forcibly interrupted.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

use super::accessibility::{NativeAccessibilityRequest, NativeAccessibilitySnapshot};
use super::clipboard::{NativeClipboardPayload, MAX_CLIPBOARD_BYTES};
use super::commands::{NativeCommandOutcome, NativeEditCommand, NativeImeContext};
use super::editor::{NativeEditOutcome, NativeEditor};
use super::save::NativeSaveCaretPolicy;
use super::save::{NativeSaveFormat, NativeSaveStatus, NativeSaveToken, PreparedSave};
use super::NativePageSize;
use super::{
    NativeDocumentSession, NativeFrame, NativeHostError, MAX_DOCUMENT_BYTES, MAX_FRAME_PIXELS,
};
use crate::document_core::native_interaction::{
    NativeCaretRect, NativeHitTest, NativeSelectionRect, NativeTextPosition, NativeTextSelection,
};
use crate::renderer::raster_pixels::RasterRegion;
use crate::serializer::ContentLossReport;

const MAX_OUTSTANDING: usize = 16;
// Reserve text, structural addresses/keys and optional geometry, including Vec
// and String allocator slack. Actual retained allocations are checked below.
const ACCESSIBILITY_RESERVATION_BYTES: usize = 8 * 1024 * 1024;
const RUNNING: u8 = 0;
const CLOSING: u8 = 1;
const CLOSED: u8 = 2;
const FAILED: u8 = 3;
const QUEUED: u8 = 0;
const CLAIMED: u8 = 1;
const CANCELLED: u8 = 2;
const COMPLETED: u8 = 3;
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Process-unique, never reused. A reconstructed ID grants no authority unless
/// it matches a live request/document in this runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NativeRequestId(u64);

impl NativeRequestId {
    pub fn as_u64(self) -> u64 {
        self.0
    }
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeRuntimeTarget {
    pub document_id: NativeRequestId,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeRuntimeConfig {
    /// Includes queued, executing and completed-but-unread requests (1..=16).
    pub max_outstanding: usize,
    /// Aggregate buffer reservations (1..=128 MiB). A render reserves the existing
    /// 64 MiB frame ceiling; a save reserves 128 MiB, copy/cut 1 MiB,
    /// and accessibility 8 MiB before acceptance. Snapshot allocations are
    /// measured after the bounded query, not transient layout memory or RSS.
    pub max_pending_bytes: usize,
}

impl Default for NativeRuntimeConfig {
    fn default() -> Self {
        Self {
            max_outstanding: 4,
            max_pending_bytes: MAX_DOCUMENT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRuntimeRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Typed transport commands; no engine or renderer crosses the thread boundary.
/// New/Open replace an existing document only when `replace` names its exact
/// current revision. Passing that target explicitly discards its unsaved state.
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NativeRuntimeCommand {
    New {
        replace: Option<NativeRuntimeTarget>,
    },
    Open {
        replace: Option<NativeRuntimeTarget>,
        bytes: Vec<u8>,
        password: Option<Vec<u8>>,
    },
    Info {},
    ExecuteCommand {
        target: NativeRuntimeTarget,
        command: NativeEditCommand,
    },
    ImeContext {
        target: NativeRuntimeTarget,
    },
    CopySelection {
        target: NativeRuntimeTarget,
    },
    CutSelection {
        target: NativeRuntimeTarget,
    },
    PasteText {
        target: NativeRuntimeTarget,
        text: String,
    },
    PasteHtml {
        target: NativeRuntimeTarget,
        html: String,
    },
    /// Read-only snapshot of this exact document revision. Selection comes from
    /// the current editor at execution time; a viewer has no selection. No
    /// command changes the editor selection to satisfy an accessibility query.
    Accessibility {
        target: NativeRuntimeTarget,
        request: NativeAccessibilityRequest,
    },
    PageSize {
        target: NativeRuntimeTarget,
        page: u32,
    },
    CaretRect {
        target: NativeRuntimeTarget,
        position: NativeTextPosition,
    },
    HitTest {
        target: NativeRuntimeTarget,
        page: u32,
        x: f64,
        y: f64,
    },
    SelectionRects {
        target: NativeRuntimeTarget,
        selection: NativeTextSelection,
    },
    CloseDocument {
        target: NativeRuntimeTarget,
    },
    RenderRegion {
        target: NativeRuntimeTarget,
        font_generation: u64,
        page: u32,
        region: NativeRuntimeRegion,
        scale: f64,
    },
    BeginEdit {
        target: NativeRuntimeTarget,
        selection: NativeTextSelection,
    },
    SetSelection {
        target: NativeRuntimeTarget,
        selection: NativeTextSelection,
    },
    Replace {
        target: NativeRuntimeTarget,
        text: String,
    },
    BeginComposition {
        target: NativeRuntimeTarget,
    },
    UpdateComposition {
        target: NativeRuntimeTarget,
        composition_id: u64,
        text: String,
    },
    CommitComposition {
        target: NativeRuntimeTarget,
        composition_id: u64,
    },
    CancelComposition {
        target: NativeRuntimeTarget,
        composition_id: u64,
    },
    Undo {
        target: NativeRuntimeTarget,
    },
    Redo {
        target: NativeRuntimeTarget,
    },
    PrepareSave {
        target: NativeRuntimeTarget,
        format: NativeSaveFormat,
    },
    AcknowledgeSave {
        document_id: NativeRequestId,
        token: NativeSaveToken,
    },
    CancelSave {
        document_id: NativeRequestId,
        token: NativeSaveToken,
    },
}

// Do not print document bytes, passwords or editor text through submit errors.
impl fmt::Debug for NativeRuntimeCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeRuntimeCommand { payload redacted }")
    }
}

impl NativeRuntimeCommand {
    fn is_render(&self) -> bool {
        matches!(self, Self::RenderRegion { .. })
    }

    fn byte_reservation(&self) -> Option<usize> {
        let selection_bytes = |selection: &NativeTextSelection| {
            selection
                .anchor
                .cell_path
                .capacity()
                .checked_add(selection.focus.cell_path.capacity())?
                .checked_mul(std::mem::size_of::<
                    crate::document_core::native_interaction::NativeCellPathEntry,
                >())
        };
        match self {
            Self::Open {
                bytes, password, ..
            } => bytes
                .capacity()
                .checked_add(password.as_ref().map_or(0, Vec::capacity)),
            Self::Replace { text, .. }
            | Self::UpdateComposition { text, .. }
            | Self::PasteText { text, .. } => Some(text.capacity()),
            Self::PasteHtml { html, .. } => Some(html.capacity()),
            Self::CopySelection { .. } | Self::CutSelection { .. } => Some(MAX_CLIPBOARD_BYTES),
            Self::BeginEdit { selection, .. } | Self::SetSelection { selection, .. } => {
                selection_bytes(selection)
            }
            Self::RenderRegion { .. } => usize::try_from(MAX_FRAME_PIXELS).ok()?.checked_mul(4),
            Self::CaretRect { position, .. } => position
                .cell_path
                .capacity()
                .checked_mul(std::mem::size_of::<
                    crate::document_core::native_interaction::NativeCellPathEntry,
                >()),
            Self::SelectionRects { selection, .. } => {
                selection_bytes(selection).map(|input| input.max(1024 * 1024))
            }
            Self::PrepareSave { .. } => Some(MAX_DOCUMENT_BYTES),
            Self::ImeContext { .. } => Some(64 * 1024),
            Self::Accessibility { .. } => Some(ACCESSIBILITY_RESERVATION_BYTES),
            _ => Some(0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRuntimeInfo {
    pub document_id: NativeRequestId,
    pub revision: u64,
    pub font_generation: u64,
    pub page_count: u32,
    pub dirty: bool,
    pub editing: bool,
    pub selection: Option<NativeTextSelection>,
    pub composition_id: Option<u64>,
    pub can_undo: bool,
    pub can_redo: bool,
}

impl NativeRuntimeInfo {
    pub fn target(&self) -> NativeRuntimeTarget {
        NativeRuntimeTarget {
            document_id: self.document_id,
            revision: self.revision,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeRuntimeSkipReason {
    SupersededRender,
    StaleFrame,
}

/// Owned bytes/report from one canonical prepare, with allocator slack removed.
/// This token still needs explicit acknowledgement after successful host I/O.
#[derive(Debug)]
pub struct NativeRuntimePreparedSave {
    pub bytes: Box<[u8]>,
    pub content_loss: ContentLossReport,
    pub revision: u64,
    pub token: NativeSaveToken,
    pub caret_policy: NativeSaveCaretPolicy,
    pub caret_saved: bool,
    pub caret_preserved_from_source: bool,
}
impl NativeRuntimePreparedSave {
    fn from_prepared(prepared: PreparedSave) -> Self {
        let caret_policy = prepared.caret_policy();
        let caret_saved = prepared.caret_saved();
        let caret_preserved_from_source = prepared.caret_preserved_from_source();
        let (bytes, content_loss, revision, token) = prepared.into_parts();
        Self {
            bytes: bytes.into_boxed_slice(),
            content_loss,
            revision,
            token,
            caret_policy,
            caret_saved,
            caret_preserved_from_source,
        }
    }
}

pub enum NativeRuntimeOutput {
    Info(NativeRuntimeInfo),
    CommandExecuted(NativeCommandOutcome),
    ImeContext(NativeImeContext),
    Clipboard {
        payload: NativeClipboardPayload,
        outcome: Option<NativeEditOutcome>,
    },
    Accessibility {
        target: NativeRuntimeTarget,
        snapshot: NativeAccessibilitySnapshot,
    },
    DocumentClosed,
    PageSize {
        target: NativeRuntimeTarget,
        page: u32,
        size: NativePageSize,
    },
    CaretRect {
        target: NativeRuntimeTarget,
        caret: Option<NativeCaretRect>,
    },
    HitTest {
        target: NativeRuntimeTarget,
        hit: NativeHitTest,
    },
    SelectionRects {
        target: NativeRuntimeTarget,
        rects: Vec<NativeSelectionRect>,
    },
    Edited(NativeEditOutcome),
    Frame {
        document_id: NativeRequestId,
        frame: NativeFrame,
    },
    PreparedSave {
        document_id: NativeRequestId,
        prepared: NativeRuntimePreparedSave,
    },
    SaveStatus {
        document_id: NativeRequestId,
        status: NativeSaveStatus,
    },
    Cancelled,
    Skipped(NativeRuntimeSkipReason),
}

impl fmt::Debug for NativeRuntimeOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info(_) => "Info",
            Self::CommandExecuted(_) => "CommandExecuted",
            Self::ImeContext(_) => "ImeContext",
            Self::Clipboard { .. } => "Clipboard",
            Self::Accessibility { .. } => "Accessibility",
            Self::DocumentClosed => "DocumentClosed",
            Self::PageSize { .. } => "PageSize",
            Self::CaretRect { .. } => "CaretRect",
            Self::HitTest { .. } => "HitTest",
            Self::SelectionRects { .. } => "SelectionRects",
            Self::Edited(_) => "Edited",
            Self::Frame { .. } => "Frame",
            Self::PreparedSave { .. } => "PreparedSave",
            Self::SaveStatus { .. } => "SaveStatus",
            Self::Cancelled => "Cancelled",
            Self::Skipped(_) => "Skipped",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeRuntimeFailure {
    pub code: &'static str,
    pub message: String,
}

impl From<NativeHostError> for NativeRuntimeFailure {
    fn from(error: NativeHostError) -> Self {
        let mut message = error.to_string();
        let mut end = message.len().min(4096);
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        Self {
            code: error.code(),
            message,
        }
    }
}

#[derive(Debug)]
pub struct NativeRuntimeCompletion {
    pub request_id: NativeRequestId,
    pub result: Result<NativeRuntimeOutput, NativeRuntimeFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeRuntimeError {
    InvalidConfig,
    QueueFull,
    ByteCapacity,
    Closed,
    WorkerFailed,
    RequestIdExhausted,
    WorkerStart(String),
}

impl NativeRuntimeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfig => "invalidConfig",
            Self::QueueFull => "queueFull",
            Self::ByteCapacity => "byteCapacity",
            Self::Closed => "closed",
            Self::WorkerFailed => "workerFailed",
            Self::RequestIdExhausted => "requestIdExhausted",
            Self::WorkerStart(_) => "workerStart",
        }
    }
}
impl fmt::Display for NativeRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerStart(message) => write!(f, "worker start failed: {message}"),
            _ => f.write_str(self.code()),
        }
    }
}
impl std::error::Error for NativeRuntimeError {}

/// Failed submissions return ownership so a host can retry without losing input.
#[derive(Debug)]
pub struct NativeRuntimeSubmitError {
    pub error: NativeRuntimeError,
    pub command: Box<NativeRuntimeCommand>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeRuntimeCancelOutcome {
    Cancelled,
    TooLate,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Stamp {
    target: NativeRuntimeTarget,
    font_generation: u64,
}
struct Shared {
    lifecycle: AtomicU8,
    latest_render: AtomicU64,
    current: Mutex<Option<Stamp>>,
}
impl Shared {
    fn current(&self) -> Option<Stamp> {
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    fn set_current(&self, stamp: Option<Stamp>) {
        *self
            .current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = stamp;
    }
    fn superseded(&self, id: NativeRequestId) -> bool {
        // `try_send` publishes before the host updates latest_render. A smaller
        // value therefore means this accepted request's publication is pending,
        // not that the worker should skip it. IDs/order are monotonic.
        self.latest_render.load(Ordering::Acquire) > id.0
    }
}
struct Request {
    id: NativeRequestId,
    command: NativeRuntimeCommand,
    phase: Arc<AtomicU8>,
    reservation: usize,
}
struct InFlight {
    render: bool,
    phase: Arc<AtomicU8>,
    reservation: usize,
}

/// A movable host handle; the !Send engine is constructed only inside its worker.
/// Polling and submission never wait for work/completion queue capacity.
pub struct NativeRuntime {
    config: NativeRuntimeConfig,
    sender: Option<SyncSender<Request>>,
    completions: Option<Receiver<NativeRuntimeCompletion>>,
    in_flight: HashMap<NativeRequestId, InFlight>,
    reserved_bytes: usize,
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
}

impl NativeRuntime {
    pub fn spawn(config: NativeRuntimeConfig) -> Result<Self, NativeRuntimeError> {
        if !(1..=MAX_OUTSTANDING).contains(&config.max_outstanding)
            || config.max_pending_bytes == 0
            || config.max_pending_bytes > MAX_DOCUMENT_BYTES
        {
            return Err(NativeRuntimeError::InvalidConfig);
        }
        let (sender, requests) = mpsc::sync_channel(config.max_outstanding);
        let (results, completions) = mpsc::sync_channel(config.max_outstanding);
        let shared = Arc::new(Shared {
            lifecycle: AtomicU8::new(RUNNING),
            latest_render: AtomicU64::new(0),
            current: Mutex::new(None),
        });
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("rhwp-native-runtime".into())
            .spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    worker_loop(requests, results, &worker_shared)
                }));
                worker_shared.set_current(None);
                worker_shared.lifecycle.store(
                    if matches!(result, Ok(true)) {
                        CLOSED
                    } else {
                        FAILED
                    },
                    Ordering::Release,
                );
            })
            .map_err(|error| NativeRuntimeError::WorkerStart(error.to_string()))?;
        Ok(Self {
            config,
            sender: Some(sender),
            completions: Some(completions),
            in_flight: HashMap::new(),
            reserved_bytes: 0,
            shared,
            worker: Some(worker),
        })
    }

    pub fn try_submit(
        &mut self,
        command: NativeRuntimeCommand,
    ) -> Result<NativeRequestId, NativeRuntimeSubmitError> {
        let reject = |error, command| NativeRuntimeSubmitError {
            error,
            command: Box::new(command),
        };
        match self.shared.lifecycle.load(Ordering::Acquire) {
            RUNNING => (),
            FAILED => return Err(reject(NativeRuntimeError::WorkerFailed, command)),
            _ => return Err(reject(NativeRuntimeError::Closed, command)),
        }
        if self.in_flight.len() >= self.config.max_outstanding {
            return Err(reject(NativeRuntimeError::QueueFull, command));
        }
        let Some(reservation) = command.byte_reservation() else {
            return Err(reject(NativeRuntimeError::ByteCapacity, command));
        };
        let Some(total) = self
            .reserved_bytes
            .checked_add(reservation)
            .filter(|total| *total <= self.config.max_pending_bytes)
        else {
            return Err(reject(NativeRuntimeError::ByteCapacity, command));
        };
        let Ok(id) = NEXT_REQUEST_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map(NativeRequestId)
        else {
            return Err(reject(NativeRuntimeError::RequestIdExhausted, command));
        };
        let phase = Arc::new(AtomicU8::new(QUEUED));
        let render = command.is_render();
        let request = Request {
            id,
            command,
            phase: Arc::clone(&phase),
            reservation,
        };
        let Some(sender) = &self.sender else {
            return Err(reject(NativeRuntimeError::Closed, request.command));
        };
        match sender.try_send(request) {
            Ok(()) => {
                self.in_flight.insert(
                    id,
                    InFlight {
                        render,
                        phase,
                        reservation,
                    },
                );
                self.reserved_bytes = total;
                if render {
                    self.shared.latest_render.store(id.0, Ordering::Release);
                }
                Ok(id)
            }
            Err(TrySendError::Full(request)) => {
                Err(reject(NativeRuntimeError::QueueFull, request.command))
            }
            Err(TrySendError::Disconnected(request)) => Err(reject(
                if matches!(
                    self.shared.lifecycle.load(Ordering::Acquire),
                    RUNNING | FAILED
                ) {
                    NativeRuntimeError::WorkerFailed
                } else {
                    NativeRuntimeError::Closed
                },
                request.command,
            )),
        }
    }

    pub fn cancel(&self, id: NativeRequestId) -> NativeRuntimeCancelOutcome {
        let Some(request) = self.in_flight.get(&id) else {
            return NativeRuntimeCancelOutcome::Unknown;
        };
        match request
            .phase
            .compare_exchange(QUEUED, CANCELLED, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) | Err(CANCELLED) => NativeRuntimeCancelOutcome::Cancelled,
            Err(_) => NativeRuntimeCancelOutcome::TooLate,
        }
    }

    pub fn try_recv(&mut self) -> Result<Option<NativeRuntimeCompletion>, NativeRuntimeError> {
        let Some(receiver) = &self.completions else {
            return Err(NativeRuntimeError::Closed);
        };
        match receiver.try_recv() {
            Ok(mut completion) => {
                if let Some(request) = self.in_flight.remove(&completion.request_id) {
                    self.reserved_bytes -= request.reservation;
                    if request.render
                        && self.shared.superseded(completion.request_id)
                        && !matches!(&completion.result, Ok(NativeRuntimeOutput::Cancelled))
                    {
                        completion.result = Ok(NativeRuntimeOutput::Skipped(
                            NativeRuntimeSkipReason::SupersededRender,
                        ));
                    }
                }
                if let Ok(NativeRuntimeOutput::Frame { document_id, frame }) = &completion.result {
                    let reason = if self.shared.superseded(completion.request_id) {
                        Some(NativeRuntimeSkipReason::SupersededRender)
                    } else if self.shared.current()
                        != Some(Stamp {
                            target: NativeRuntimeTarget {
                                document_id: *document_id,
                                revision: frame.revision,
                            },
                            font_generation: frame.font_generation,
                        })
                    {
                        Some(NativeRuntimeSkipReason::StaleFrame)
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        completion.result = Ok(NativeRuntimeOutput::Skipped(reason));
                    }
                }
                Ok(Some(completion))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                self.in_flight.clear();
                self.reserved_bytes = 0;
                Err(
                    if matches!(
                        self.shared.lifecycle.load(Ordering::Acquire),
                        RUNNING | FAILED
                    ) {
                        NativeRuntimeError::WorkerFailed
                    } else {
                        NativeRuntimeError::Closed
                    },
                )
            }
        }
    }

    /// Cancel queued work and discard unread outputs. Does not persist edits,
    /// acknowledge saves, join, or interrupt an executing engine call.
    pub fn close(&mut self) {
        let _ = self.shared.lifecycle.compare_exchange(
            RUNNING,
            CLOSING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        self.sender.take();
        self.completions.take();
        self.in_flight.clear();
        self.reserved_bytes = 0;
    }
    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    /// Reap only an already-finished thread; never wait for an executing command.
    pub fn try_join(&mut self) -> Result<bool, NativeRuntimeError> {
        if !self.is_finished() {
            return Ok(false);
        }
        if let Some(worker) = self.worker.take() {
            if worker.join().is_err() {
                return Err(NativeRuntimeError::WorkerFailed);
            }
        }
        if matches!(
            self.shared.lifecycle.load(Ordering::Acquire),
            RUNNING | FAILED
        ) {
            Err(NativeRuntimeError::WorkerFailed)
        } else {
            Ok(true)
        }
    }
}
impl Drop for NativeRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

enum DocumentOwner {
    Viewer(Box<NativeDocumentSession>),
    Editor(Box<NativeEditor>),
}
struct OwnedDocument {
    id: NativeRequestId,
    owner: DocumentOwner,
}
impl OwnedDocument {
    fn session(&self) -> &NativeDocumentSession {
        match &self.owner {
            DocumentOwner::Viewer(session) => session,
            DocumentOwner::Editor(editor) => editor.session(),
        }
    }
    fn editor(&mut self) -> Result<&mut NativeEditor, NativeHostError> {
        match &mut self.owner {
            DocumentOwner::Editor(editor) => Ok(editor),
            _ => Err(NativeHostError::Unsupported("beginEdit is required".into())),
        }
    }
    fn check(&self, target: NativeRuntimeTarget) -> Result<(), NativeHostError> {
        self.check_id(target.document_id)?;
        self.session().check_revision(target.revision)
    }
    fn check_id(&self, id: NativeRequestId) -> Result<(), NativeHostError> {
        if id == self.id {
            Ok(())
        } else {
            Err(NativeHostError::InvalidArgument(
                "stale or foreign document ID".into(),
            ))
        }
    }
    fn stamp(&self) -> Stamp {
        Stamp {
            target: NativeRuntimeTarget {
                document_id: self.id,
                revision: self.session().revision(),
            },
            font_generation: self.session().font_generation(),
        }
    }
    fn info(&self) -> NativeRuntimeInfo {
        let editor = match &self.owner {
            DocumentOwner::Editor(editor) => Some(editor.as_ref()),
            _ => None,
        };
        NativeRuntimeInfo {
            document_id: self.id,
            revision: self.session().revision(),
            font_generation: self.session().font_generation(),
            page_count: self.session().document().page_count(),
            dirty: self.session().is_dirty(),
            editing: editor.is_some(),
            selection: editor.map(|e| e.selection().clone()),
            composition_id: editor.and_then(NativeEditor::composition_id),
            can_undo: editor.is_some_and(NativeEditor::can_undo),
            can_redo: editor.is_some_and(NativeEditor::can_redo),
        }
    }
    fn cancel_save(&mut self, token: NativeSaveToken) -> Result<NativeSaveStatus, NativeHostError> {
        match &mut self.owner {
            DocumentOwner::Viewer(session) => session.cancel_save(token),
            DocumentOwner::Editor(editor) => editor.cancel_save(token),
        }
    }
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}
fn document(owner: &mut Option<OwnedDocument>) -> Result<&mut OwnedDocument, NativeHostError> {
    owner.as_mut().ok_or_else(|| invalid("no document is open"))
}
fn replacement(
    owner: &Option<OwnedDocument>,
    target: Option<NativeRuntimeTarget>,
) -> Result<(), NativeHostError> {
    match (owner, target) {
        (None, None) => Ok(()),
        (Some(owner), Some(target)) => owner.check(target),
        _ => Err(invalid(
            "replacement must name the current document revision",
        )),
    }
}

fn compact_clipboard_payload(payload: NativeClipboardPayload) -> NativeClipboardPayload {
    // NativeEditor checks the combined UTF-8 length before cut mutates the
    // document. Remove allocator slack so the unread output fits its reservation.
    NativeClipboardPayload {
        plain_text: payload.plain_text.into_boxed_str().into_string(),
        html: payload.html.map(|html| html.into_boxed_str().into_string()),
    }
}

fn execute(
    owner: &mut Option<OwnedDocument>,
    id: NativeRequestId,
    command: NativeRuntimeCommand,
) -> Result<NativeRuntimeOutput, NativeHostError> {
    use NativeRuntimeCommand as C;
    use NativeRuntimeOutput as O;
    match command {
        C::New { replace } => {
            replacement(owner, replace)?;
            let session = NativeDocumentSession::new_blank()?;
            *owner = Some(OwnedDocument {
                id,
                owner: DocumentOwner::Viewer(Box::new(session)),
            });
            Ok(O::Info(document(owner)?.info()))
        }
        C::Open {
            replace,
            bytes,
            password,
        } => {
            replacement(owner, replace)?;
            let session = NativeDocumentSession::open_bytes(&bytes, password.as_deref())?;
            *owner = Some(OwnedDocument {
                id,
                owner: DocumentOwner::Viewer(Box::new(session)),
            });
            Ok(O::Info(document(owner)?.info()))
        }
        C::Info {} => Ok(O::Info(document(owner)?.info())),
        C::ExecuteCommand { target, command } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .execute_command(target.revision, command)
                .map(O::CommandExecuted)
        }
        C::ImeContext { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .ime_context(target.revision)
                .map(O::ImeContext)
        }
        C::CopySelection { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let payload = doc.editor()?.copy_selection(target.revision)?;
            Ok(O::Clipboard {
                payload: compact_clipboard_payload(payload),
                outcome: None,
            })
        }
        C::CutSelection { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let (payload, outcome) = doc.editor()?.cut_selection(target.revision)?;
            Ok(O::Clipboard {
                payload: compact_clipboard_payload(payload),
                outcome: Some(outcome),
            })
        }
        C::PasteText { target, text } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .paste_text(target.revision, &text)
                .map(O::Edited)
        }
        C::PasteHtml { target, html } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .paste_html(target.revision, &html)
                .map(O::Edited)
        }
        C::PageSize { target, page } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.session()
                .page_size(page)
                .map(|size| O::PageSize { target, page, size })
        }
        C::CaretRect { target, position } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::CaretRect {
                target,
                caret: Some(doc.session().document().native_caret_rect(&position)?),
            })
        }
        C::HitTest { target, page, x, y } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::HitTest {
                target,
                hit: doc.session().document().native_hit_test(page, x, y)?,
            })
        }
        C::SelectionRects { target, selection } => {
            let doc = document(owner)?;
            doc.check(target)?;
            Ok(O::SelectionRects {
                target,
                rects: doc
                    .session()
                    .document()
                    .native_selection_rects(&selection)?,
            })
        }
        C::CloseDocument { target } => {
            document(owner)?.check(target)?;
            *owner = None;
            Ok(O::DocumentClosed)
        }
        C::Accessibility { target, request } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let selection = match &doc.owner {
                DocumentOwner::Editor(editor) => Some(editor.selection()),
                DocumentOwner::Viewer(_) => None,
            };
            Ok(O::Accessibility {
                target,
                snapshot: doc.session().accessibility_snapshot(&request, selection)?,
            })
        }
        C::RenderRegion {
            target,
            font_generation,
            page,
            region,
            scale,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            if doc.session().font_generation() != font_generation {
                return Err(invalid("stale font generation"));
            }
            let frame = doc.session().render_region(
                page,
                RasterRegion {
                    x: region.x,
                    y: region.y,
                    width: region.width,
                    height: region.height,
                },
                scale,
            )?;
            Ok(O::Frame {
                document_id: doc.id,
                frame,
            })
        }
        C::BeginEdit { target, selection } => {
            document(owner)?.check(target)?;
            if matches!(&document(owner)?.owner, DocumentOwner::Editor(_)) {
                return Err(invalid("editor already active"));
            }
            let doc = owner.take().expect("validated document");
            let DocumentOwner::Viewer(session) = doc.owner else {
                unreachable!("validated viewer");
            };
            match NativeEditor::try_new(*session, selection) {
                Ok(editor) => {
                    *owner = Some(OwnedDocument {
                        id: doc.id,
                        owner: DocumentOwner::Editor(Box::new(editor)),
                    });
                    Ok(O::Info(document(owner)?.info()))
                }
                Err(failed) => {
                    let (session, error) = *failed;
                    *owner = Some(OwnedDocument {
                        id: doc.id,
                        owner: DocumentOwner::Viewer(Box::new(session)),
                    });
                    Err(error)
                }
            }
        }
        C::SetSelection { target, selection } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .set_selection(target.revision, selection)
                .map(O::Edited)
        }
        C::Replace { target, text } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .replace_selection(target.revision, &text)
                .map(O::Edited)
        }
        C::BeginComposition { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .begin_composition(target.revision)
                .map(O::Edited)
        }
        C::UpdateComposition {
            target,
            composition_id,
            text,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .update_composition(target.revision, composition_id, &text)
                .map(O::Edited)
        }
        C::CommitComposition {
            target,
            composition_id,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .commit_composition(target.revision, composition_id)
                .map(O::Edited)
        }
        C::CancelComposition {
            target,
            composition_id,
        } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?
                .cancel_composition(target.revision, composition_id)
                .map(O::Edited)
        }
        C::Undo { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?.undo(target.revision).map(O::Edited)
        }
        C::Redo { target } => {
            let doc = document(owner)?;
            doc.check(target)?;
            doc.editor()?.redo(target.revision).map(O::Edited)
        }
        C::PrepareSave { target, format } => {
            let doc = document(owner)?;
            doc.check(target)?;
            let prepared = match &mut doc.owner {
                DocumentOwner::Viewer(session) => session.prepare_save(format),
                DocumentOwner::Editor(editor) => editor.prepare_save(format),
            }?;
            if prepared.bytes().len() > MAX_DOCUMENT_BYTES {
                let _ = doc.cancel_save(prepared.token());
                return Err(NativeHostError::LimitExceeded(
                    "runtime save exceeds its reserved byte limit".into(),
                ));
            }
            Ok(O::PreparedSave {
                document_id: doc.id,
                prepared: NativeRuntimePreparedSave::from_prepared(prepared),
            })
        }
        C::AcknowledgeSave { document_id, token } => {
            let doc = document(owner)?;
            doc.check_id(document_id)?;
            let status = match &mut doc.owner {
                DocumentOwner::Viewer(session) => session.acknowledge_save(token),
                DocumentOwner::Editor(editor) => editor.acknowledge_save(token),
            }?;
            Ok(O::SaveStatus {
                document_id,
                status,
            })
        }
        C::CancelSave { document_id, token } => {
            let doc = document(owner)?;
            doc.check_id(document_id)?;
            doc.cancel_save(token).map(|status| O::SaveStatus {
                document_id,
                status,
            })
        }
    }
}

/// Snapshot nodes contain variable text, keys and nested structural addresses;
/// counting only the outer node vector would leave queued completions uncharged.
fn accessibility_retained_bytes(snapshot: &NativeAccessibilitySnapshot) -> usize {
    let position_bytes = |position: &NativeTextPosition| {
        position
            .cell_path
            .capacity()
            .saturating_mul(std::mem::size_of::<
                crate::document_core::native_interaction::NativeCellPathEntry,
            >())
    };
    let mut bytes = std::mem::size_of::<NativeAccessibilitySnapshot>()
        .saturating_add(
            snapshot
                .nodes
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    super::accessibility::NativeAccessibilityNode,
                >()),
        )
        .saturating_add(
            snapshot
                .limitations
                .capacity()
                .saturating_mul(std::mem::size_of::<
                    super::accessibility::NativeAccessibilityLimitation,
                >()),
        );
    for node in &snapshot.nodes {
        bytes = bytes
            .saturating_add(node.key.capacity())
            .saturating_add(node.text.capacity())
            .saturating_add(position_bytes(&node.start))
            .saturating_add(position_bytes(&node.end));
    }
    if let Some(selection) = &snapshot.selection {
        bytes = bytes
            .saturating_add(position_bytes(&selection.anchor))
            .saturating_add(position_bytes(&selection.focus));
    }
    if let Some(geometry) = &snapshot.selection_geometry {
        bytes = bytes.saturating_add(
            geometry
                .rectangles
                .capacity()
                .saturating_mul(std::mem::size_of::<NativeSelectionRect>()),
        );
    }
    bytes
}

fn discard_prepared(
    owner: &mut Option<OwnedDocument>,
    result: &Result<NativeRuntimeOutput, NativeRuntimeFailure>,
) {
    if let Ok(NativeRuntimeOutput::PreparedSave {
        document_id,
        prepared,
    }) = result
    {
        if let Some(doc) = owner.as_mut().filter(|doc| doc.id == *document_id) {
            let _ = doc.cancel_save(prepared.token);
        }
    }
}

// Return false only for an impossible full completion queue. Disconnect/close
// is normal; neither path blocks or leaks an undisclosed save checkpoint.
fn worker_loop(
    requests: Receiver<Request>,
    results: SyncSender<NativeRuntimeCompletion>,
    shared: &Shared,
) -> bool {
    let mut owner = None; // All !Send document/renderer state is born here.
    while let Ok(request) = requests.recv() {
        if shared.lifecycle.load(Ordering::Acquire) != RUNNING {
            break;
        }
        let render = request.command.is_render();
        let mut result = if request
            .phase
            .compare_exchange(QUEUED, CLAIMED, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            Ok(NativeRuntimeOutput::Cancelled)
        } else if render && shared.superseded(request.id) {
            Ok(NativeRuntimeOutput::Skipped(
                NativeRuntimeSkipReason::SupersededRender,
            ))
        } else {
            execute(&mut owner, request.id, request.command).map_err(NativeRuntimeFailure::from)
        };
        shared.set_current(owner.as_ref().map(OwnedDocument::stamp));
        let retained = match &result {
            Ok(NativeRuntimeOutput::Frame { frame, .. }) => frame.pixels.bytes.capacity(),
            Ok(NativeRuntimeOutput::PreparedSave { prepared, .. }) => prepared.bytes.len(),
            Ok(NativeRuntimeOutput::SelectionRects { rects, .. }) => rects
                .capacity()
                .checked_mul(std::mem::size_of::<NativeSelectionRect>())
                .unwrap_or(usize::MAX),
            Ok(NativeRuntimeOutput::Accessibility { snapshot, .. }) => {
                accessibility_retained_bytes(snapshot)
            }
            _ => 0,
        };
        if retained > request.reservation {
            discard_prepared(&mut owner, &result);
            result = Err(NativeHostError::LimitExceeded(
                "runtime output exceeds its reserved byte limit".into(),
            )
            .into());
        } else if render
            && shared.superseded(request.id)
            && !matches!(&result, Ok(NativeRuntimeOutput::Cancelled))
        {
            result = Ok(NativeRuntimeOutput::Skipped(
                NativeRuntimeSkipReason::SupersededRender,
            ));
        }
        request.phase.store(COMPLETED, Ordering::Release);
        if shared.lifecycle.load(Ordering::Acquire) != RUNNING {
            discard_prepared(&mut owner, &result);
            break;
        }
        let completion = NativeRuntimeCompletion {
            request_id: request.id,
            result,
        };
        match results.try_send(completion) {
            Ok(()) => (),
            Err(TrySendError::Disconnected(completion)) => {
                discard_prepared(&mut owner, &completion.result);
                break;
            }
            Err(TrySendError::Full(completion)) => {
                discard_prepared(&mut owner, &completion.result);
                return false;
            }
        }
    }
    true
}
