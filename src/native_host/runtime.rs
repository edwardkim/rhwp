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
//! Cancellation covers queued commands and executing tiled renders at tile
//! boundaries. `close`/Drop discard queued work and unread completions without
//! joining, and stop tiled work at the next boundary. An active synchronous
//! parse/serialization/Skia call finishes on its owner thread before its document
//! and memory are released. No running native call is forcibly interrupted.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

use super::accessibility::{NativeAccessibilityRequest, NativeAccessibilitySnapshot};
use super::clipboard::{NativeClipboardPayload, MAX_CLIPBOARD_BYTES};
use super::commands::{NativeCommandOutcome, NativeEditCommand, NativeImeContext};
use super::editor::ime_edit::{NativeImeEdit, NativeImeEditContext};
use super::editor::{NativeEditOutcome, NativeEditor};
use super::render_snapshot::{NativeRenderSnapshot, NativeRenderSnapshotLimits};
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

mod worker;
use worker::worker_loop;

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
    /// Aggregate buffer reservations (1..=128 MiB). Renders reserve their exact
    /// enclosing RGBA output size; a save reserves 128 MiB, copy/cut 1 MiB,
    /// and accessibility 8 MiB before acceptance. Render snapshot input estimates
    /// are charged after the bounded query, with its ceiling clamped to this
    /// quota. These estimates exclude transient layout/native memory and RSS.
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

impl From<NativeRuntimeRegion> for RasterRegion {
    fn from(region: NativeRuntimeRegion) -> Self {
        Self {
            x: region.x,
            y: region.y,
            width: region.width,
            height: region.height,
        }
    }
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
    OpenRecovery {
        replace: Option<NativeRuntimeTarget>,
        bytes: Vec<u8>,
    },
    Info {},
    ExecuteCommand {
        target: NativeRuntimeTarget,
        command: NativeEditCommand,
    },
    ImeContext {
        target: NativeRuntimeTarget,
    },
    ImeEditContext {
        target: NativeRuntimeTarget,
    },
    ApplyImeEdit {
        target: NativeRuntimeTarget,
        expected: Box<NativeImeEditContext>,
        edit: NativeImeEdit,
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
    /// Export only owned immutable inputs. The worker clamps the input ceiling
    /// to its byte quota and charges the actual estimate before completion.
    ExportRenderSnapshot {
        target: NativeRuntimeTarget,
        font_generation: u64,
        page: u32,
        #[serde(with = "snapshot_profile")]
        profile: crate::paint::RenderProfile,
        #[serde(default)]
        limits: NativeRenderSnapshotLimits,
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
    /// Fixed-grid tiles for interactive viewports; not the exact full-page crop API.
    RenderTiledRegion {
        target: NativeRuntimeTarget,
        font_generation: u64,
        page: u32,
        region: NativeRuntimeRegion,
        scale: f64,
    },
    /// Combined full-page and tile raster retention, separate from renderer resources.
    SetRasterCacheLimit {
        target: NativeRuntimeTarget,
        max_bytes: usize,
    },
    RasterCacheStats {
        target: NativeRuntimeTarget,
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
        matches!(
            self,
            Self::RenderRegion { .. } | Self::RenderTiledRegion { .. }
        )
    }

    fn byte_reservation(&self) -> Result<usize, NativeRuntimeError> {
        if let Self::ExportRenderSnapshot { limits, .. } = self {
            limits
                .validate()
                .map_err(|_| NativeRuntimeError::InvalidArgument)?;
            return Ok(0);
        }
        if let Self::RenderRegion { region, scale, .. }
        | Self::RenderTiledRegion { region, scale, .. } = self
        {
            NativeDocumentSession::check_scale(*scale)
                .map_err(|_| NativeRuntimeError::InvalidArgument)?;
            let options = crate::renderer::layer_renderer::RasterRenderOptions::default();
            return RasterRegion::from(*region)
                .pixel_window(*scale, options.max_dimension, MAX_FRAME_PIXELS)
                .map(|window| window.output_bytes)
                .map_err(|_| NativeRuntimeError::InvalidArgument);
        }
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
        let bytes = match self {
            Self::Open {
                bytes, password, ..
            } => bytes
                .capacity()
                .checked_add(password.as_ref().map_or(0, Vec::capacity)),
            Self::OpenRecovery { bytes, .. } => Some(bytes.capacity()),
            Self::Replace { text, .. }
            | Self::UpdateComposition { text, .. }
            | Self::PasteText { text, .. } => Some(text.capacity()),
            Self::PasteHtml { html, .. } => Some(html.capacity()),
            Self::CopySelection { .. } | Self::CutSelection { .. } => Some(MAX_CLIPBOARD_BYTES),
            Self::BeginEdit { selection, .. } | Self::SetSelection { selection, .. } => {
                selection_bytes(selection)
            }
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
            Self::ImeContext { .. } | Self::ImeEditContext { .. } => Some(64 * 1024),
            Self::ApplyImeEdit { expected, edit, .. } => {
                let replacement_bytes = match edit {
                    NativeImeEdit::Continue { replacement, .. } => {
                        Some(replacement.text.capacity())
                    }
                    NativeImeEdit::Commit {
                        replacement,
                        preedit,
                        ..
                    } => replacement.text.capacity().checked_add(
                        preedit
                            .as_ref()
                            .map_or(0, |preview| preview.replacement.text.capacity()),
                    ),
                };
                expected
                    .context
                    .text
                    .capacity()
                    .checked_add(expected.context.selected_text.capacity())
                    .and_then(|bytes| bytes.checked_add(selection_bytes(&expected.selection)?))
                    .and_then(|bytes| {
                        bytes.checked_add(
                            expected.context.position.cell_path.capacity().checked_mul(
                                std::mem::size_of::<
                                    crate::document_core::native_interaction::NativeCellPathEntry,
                                >(),
                            )?,
                        )
                    })
                    .and_then(|bytes| bytes.checked_add(replacement_bytes?))
            }
            Self::Accessibility { .. } => Some(ACCESSIBILITY_RESERVATION_BYTES),
            _ => Some(0),
        };
        bytes.ok_or(NativeRuntimeError::ByteCapacity)
    }
}

mod snapshot_profile {
    use crate::paint::RenderProfile;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        profile: &RenderProfile,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(super::super::render_snapshot::profile_name(*profile))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<RenderProfile, D::Error> {
        let value = String::deserialize(deserializer)?;
        RenderProfile::parse(&value)
            .ok_or_else(|| serde::de::Error::custom("invalid render snapshot profile"))
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
    ImeEditContext(Box<NativeImeEditContext>),
    Clipboard {
        payload: NativeClipboardPayload,
        outcome: Option<NativeEditOutcome>,
    },
    Accessibility {
        target: NativeRuntimeTarget,
        snapshot: NativeAccessibilitySnapshot,
    },
    RenderSnapshot {
        document_id: NativeRequestId,
        snapshot: Arc<NativeRenderSnapshot>,
    },
    DocumentClosed,
    RasterCacheLimit {
        target: NativeRuntimeTarget,
        max_bytes: usize,
    },
    #[cfg(feature = "native-skia")]
    RasterCacheStats {
        target: NativeRuntimeTarget,
        stats: super::region_cache::NativeRegionCacheStats,
    },
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
            Self::ImeEditContext(_) => "ImeEditContext",
            Self::Clipboard { .. } => "Clipboard",
            Self::Accessibility { .. } => "Accessibility",
            Self::RenderSnapshot { .. } => "RenderSnapshot",
            Self::DocumentClosed => "DocumentClosed",
            Self::RasterCacheLimit { .. } => "RasterCacheLimit",
            #[cfg(feature = "native-skia")]
            Self::RasterCacheStats { .. } => "RasterCacheStats",
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
    InvalidArgument,
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
            Self::InvalidArgument => "invalidArgument",
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
    budget: Arc<ByteBudget>,
}

struct ByteBudget {
    max_bytes: usize,
    used: AtomicUsize,
}

impl ByteBudget {
    fn try_add(&self, amount: usize) -> bool {
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(amount)
                    .filter(|total| *total <= self.max_bytes)
            })
            .is_ok()
    }

    fn reserve(self: &Arc<Self>, bytes: usize) -> Option<Arc<ByteReservation>> {
        self.try_add(bytes).then(|| {
            Arc::new(ByteReservation {
                budget: Arc::clone(self),
                bytes: Mutex::new(Some(bytes)),
            })
        })
    }
}

/// A close/receipt can release a reservation while a worker is preparing its
/// output. The same short lock guards grow and release, so late output cannot
/// recreate a released credit or subtract its bytes twice.
struct ByteReservation {
    budget: Arc<ByteBudget>,
    bytes: Mutex<Option<usize>>,
}

impl ByteReservation {
    fn grow_to(&self, retained: usize) -> bool {
        let mut bytes = self.bytes.lock().unwrap_or_else(|error| error.into_inner());
        let Some(current) = *bytes else {
            return false;
        };
        if retained <= current {
            return true;
        }
        if !self.budget.try_add(retained - current) {
            return false;
        }
        *bytes = Some(retained);
        true
    }

    fn covers(&self, retained: usize) -> bool {
        self.bytes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some_and(|bytes| retained <= bytes)
    }

    fn release(&self) {
        if let Some(bytes) = self
            .bytes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        {
            self.budget.used.fetch_sub(bytes, Ordering::AcqRel);
        }
    }
}

impl Drop for ByteReservation {
    fn drop(&mut self) {
        self.release();
    }
}

/// Weak observation of a runtime's live document stamp. This retains neither
/// the runtime, its queues, nor the document's native objects.
#[derive(Clone)]
pub struct NativeRuntimeObserver {
    shared: Weak<Shared>,
}

impl NativeRuntimeObserver {
    pub fn is_current(&self, target: NativeRuntimeTarget, font_generation: u64) -> bool {
        self.shared.upgrade().is_some_and(|shared| {
            shared.lifecycle.load(Ordering::Acquire) == RUNNING
                && shared.current()
                    == Some(Stamp {
                        target,
                        font_generation,
                    })
        })
    }
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
    reservation: Arc<ByteReservation>,
}
struct InFlight {
    render: bool,
    tiled: bool,
    phase: Arc<AtomicU8>,
    reservation: Arc<ByteReservation>,
}

/// A movable host handle; the !Send engine is constructed only inside its worker.
/// Polling and submission never wait for work/completion queue capacity.
pub struct NativeRuntime {
    config: NativeRuntimeConfig,
    sender: Option<SyncSender<Request>>,
    completions: Option<Receiver<NativeRuntimeCompletion>>,
    in_flight: HashMap<NativeRequestId, InFlight>,
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
            budget: Arc::new(ByteBudget {
                max_bytes: config.max_pending_bytes,
                used: AtomicUsize::new(0),
            }),
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
        let reservation = match command.byte_reservation() {
            Ok(reservation) => reservation,
            Err(error) => return Err(reject(error, command)),
        };
        let Some(reservation) = self.shared.budget.reserve(reservation) else {
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
        let tiled = matches!(command, NativeRuntimeCommand::RenderTiledRegion { .. });
        let request = Request {
            id,
            command,
            phase: Arc::clone(&phase),
            reservation: Arc::clone(&reservation),
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
                        tiled,
                        phase,
                        reservation,
                    },
                );
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
            Err(CLAIMED) if request.tiled => match request.phase.compare_exchange(
                CLAIMED,
                CANCELLED,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) | Err(CANCELLED) => NativeRuntimeCancelOutcome::Cancelled,
                Err(_) => NativeRuntimeCancelOutcome::TooLate,
            },
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
                    request.reservation.release();
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
                if let Ok(NativeRuntimeOutput::RenderSnapshot {
                    document_id,
                    snapshot,
                }) = &completion.result
                {
                    if self.shared.current()
                        != Some(Stamp {
                            target: NativeRuntimeTarget {
                                document_id: *document_id,
                                revision: snapshot.revision(),
                            },
                            font_generation: snapshot.font_generation(),
                        })
                    {
                        completion.result = Ok(NativeRuntimeOutput::Skipped(
                            NativeRuntimeSkipReason::StaleFrame,
                        ));
                    }
                }
                Ok(Some(completion))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                self.release_reservations();
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

    /// Cancel queued work, stop tiled work at its next tile boundary and discard
    /// unread outputs. Does not persist edits, acknowledge saves, join, or
    /// interrupt an executing native call.
    pub fn close(&mut self) {
        let _ = self.shared.lifecycle.compare_exchange(
            RUNNING,
            CLOSING,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        self.sender.take();
        self.completions.take();
        self.release_reservations();
    }
    fn release_reservations(&mut self) {
        for request in self.in_flight.values() {
            request.reservation.release();
        }
        self.in_flight.clear();
    }

    pub fn observer(&self) -> NativeRuntimeObserver {
        NativeRuntimeObserver {
            shared: Arc::downgrade(&self.shared),
        }
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
