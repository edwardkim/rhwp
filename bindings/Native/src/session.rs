//! 길이가 명시된 요청과 스레드 소유 handle을 사용하는 native host ABI.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp_core::document_core::native_interaction::{
    NativeTextPosition, NativeTextSelection, NativeVerticalDirection,
};
use rhwp_core::native_host::accessibility::NativeAccessibilityRequest;
use rhwp_core::native_host::editor::{NativeCompositionFocusLoss, NativeEditOutcome, NativeEditor};
use rhwp_core::native_host::runtime::{
    NativeRequestId, NativeRuntime, NativeRuntimeCommand, NativeRuntimeConfig, NativeRuntimeOutput,
    NativeRuntimeTarget,
};
use rhwp_core::native_host::save::{
    NativeSaveFormat, NativeSaveStatus, NativeSaveToken, PreparedSave,
};
use rhwp_core::native_host::view::{
    NativePagePoint, NativeView, NativeViewConfig, NativeViewPoint, NativeViewRect, NativeViewState,
};
use rhwp_core::native_host::{NativeDocumentSession, NativeHostError, MAX_DOCUMENT_BYTES};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_SESSIONS: usize = 8;
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_BUFFER_BYTES: usize = 128 * 1024 * 1024;
const MAX_BUFFERS: usize = 64;
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct Host {
    sessions: HashMap<u64, ManagedSession>,
    workers: HashMap<u64, NativeRuntime>,
    buffers: HashMap<u64, Vec<u8>>,
    buffer_bytes: usize,
}

enum ManagedSession {
    View(Box<NativeDocumentSession>),
    Edit(Box<NativeEditor>),
}

impl ManagedSession {
    fn session(&self) -> &NativeDocumentSession {
        match self {
            Self::View(session) => session,
            Self::Edit(editor) => editor.session(),
        }
    }
    fn editor_mut(&mut self) -> Result<&mut NativeEditor, NativeHostError> {
        match self {
            Self::Edit(editor) => Ok(editor),
            Self::View(_) => Err(NativeHostError::InvalidArgument(
                "beginEdit is required".into(),
            )),
        }
    }
    fn set_fonts(
        &mut self,
        paths: &[std::path::PathBuf],
        environment: Option<rhwp_core::renderer::font_environment::FontEnvironment>,
    ) -> Result<bool, NativeHostError> {
        match self {
            Self::View(session) => session.set_font_sources(paths, environment),
            Self::Edit(editor) => {
                editor.set_font_sources(editor.session().revision(), paths, environment)
            }
        }
    }
    fn prepare_save(&mut self, format: NativeSaveFormat) -> Result<PreparedSave, NativeHostError> {
        match self {
            Self::View(session) => session.prepare_save(format),
            Self::Edit(editor) => editor.prepare_save(format),
        }
    }
    fn acknowledge_save(
        &mut self,
        token: NativeSaveToken,
    ) -> Result<NativeSaveStatus, NativeHostError> {
        match self {
            Self::View(session) => session.acknowledge_save(token),
            Self::Edit(editor) => editor.acknowledge_save(token),
        }
    }
    fn cancel_save(&mut self, token: NativeSaveToken) -> Result<NativeSaveStatus, NativeHostError> {
        match self {
            Self::View(session) => session.cancel_save(token),
            Self::Edit(editor) => editor.cancel_save(token),
        }
    }
    fn editing_info(&self) -> (bool, bool, Option<u64>) {
        match self {
            Self::View(_) => (false, false, None),
            Self::Edit(editor) => (
                editor.can_undo(),
                editor.can_redo(),
                editor.composition_id(),
            ),
        }
    }
}

fn edit_result(outcome: NativeEditOutcome) -> Value {
    json!({"ok":true,"outcome":outcome})
}

impl Host {
    fn begin_edit(
        &mut self,
        handle: u64,
        expected_revision: u64,
        selection: NativeTextSelection,
    ) -> Result<Value, NativeHostError> {
        self.sessions
            .get(&handle)
            .ok_or_else(invalid_handle)?
            .session()
            .check_revision(expected_revision)?;
        let record = self.sessions.remove(&handle).ok_or_else(invalid_handle)?;
        match record {
            ManagedSession::View(session) => match NativeEditor::try_new(*session, selection) {
                Ok(editor) => {
                    let result = edit_result(NativeEditOutcome {
                        revision: editor.session().revision(),
                        selection: editor.selection().clone(),
                        document_changed: false,
                        history_changed: false,
                        composition_id: None,
                    });
                    self.sessions
                        .insert(handle, ManagedSession::Edit(Box::new(editor)));
                    Ok(result)
                }
                Err(failed) => {
                    let (session, error) = *failed;
                    self.sessions
                        .insert(handle, ManagedSession::View(Box::new(session)));
                    Err(error)
                }
            },
            ManagedSession::Edit(mut editor) => {
                let result = editor
                    .set_selection(expected_revision, selection)
                    .map(edit_result);
                self.sessions.insert(handle, ManagedSession::Edit(editor));
                result
            }
        }
    }
}

thread_local! {
    static HOST: RefCell<Host> = RefCell::new(Host::default());
}

#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    ExecuteCommand {
        command: rhwp_core::native_host::commands::NativeEditCommand,
        expected_revision: u64,
    },
    ImeContext {
        expected_revision: u64,
    },
    CopySelection {
        expected_revision: u64,
    },
    CutSelection {
        expected_revision: u64,
    },
    PasteText {
        text: String,
        expected_revision: u64,
    },
    PasteHtml {
        html: String,
        expected_revision: u64,
    },
    Info {},
    /// Omitted/null selection uses the current editor selection (none for a
    /// viewer). An explicit selection affects only this read-only snapshot.
    Accessibility {
        request: NativeAccessibilityRequest,
        selection: Option<NativeTextSelection>,
    },
    PrepareSave {
        format: NativeSaveFormat,
    },
    AcknowledgeSave {
        token: NativeSaveToken,
    },
    CancelSave {
        token: NativeSaveToken,
    },
    BeginEdit {
        selection: NativeTextSelection,
        expected_revision: u64,
    },
    SetSelection {
        selection: NativeTextSelection,
        expected_revision: u64,
    },
    ReplaceSelection {
        text: String,
        expected_revision: u64,
    },
    BeginComposition {
        expected_revision: u64,
    },
    UpdateComposition {
        composition_id: u64,
        text: String,
        expected_revision: u64,
    },
    CommitComposition {
        composition_id: u64,
        expected_revision: u64,
    },
    CancelComposition {
        composition_id: u64,
        expected_revision: u64,
    },
    FocusLost {
        policy: NativeCompositionFocusLoss,
        expected_revision: u64,
    },
    Undo {
        expected_revision: u64,
    },
    Redo {
        expected_revision: u64,
    },
    ClearHistory {
        expected_revision: u64,
    },
    HitTest {
        page: u32,
        x: f64,
        y: f64,
    },
    Caret {
        position: NativeTextPosition,
    },
    SelectionRects {
        selection: NativeTextSelection,
    },
    MoveVertical {
        position: NativeTextPosition,
        direction: NativeVerticalDirection,
        preferred_x: Option<f64>,
    },
    FontState {},
    SetFonts {
        paths: Vec<std::path::PathBuf>,
        environment: Option<rhwp_core::renderer::font_environment::FontEnvironment>,
    },
    IsFrameCurrent {
        revision: u64,
        font_generation: u64,
    },
    ClearCaches {},
    CacheStats {},
    PageSize {
        page: u32,
    },
    Render {
        page: u32,
        scale: f64,
    },
    RenderRegion {
        page: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        scale: f64,
    },
}

#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum HostRequest {
    WorkerCreate {
        #[serde(default)]
        config: NativeRuntimeConfig,
    },
    WorkerSubmit {
        worker: u64,
        command: NativeRuntimeCommand,
    },
    WorkerOpen {
        worker: u64,
        replace: Option<NativeRuntimeTarget>,
        password: Option<Vec<u8>>,
    },
    WorkerPoll {
        worker: u64,
    },
    WorkerCancel {
        worker: u64,
        request_id: NativeRequestId,
    },
    WorkerClose {
        worker: u64,
    },
    WorkerStatus {
        worker: u64,
    },
    WorkerRelease {
        worker: u64,
    },
    View {
        state: NativeViewState,
        command: ViewCommand,
    },
}

#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum ViewCommand {
    State {},
    Configure {
        config: NativeViewConfig,
    },
    Scroll {
        x: f64,
        y: f64,
    },
    ScrollBy {
        x: f64,
        y: f64,
    },
    ZoomAt {
        zoom: f64,
        x: f64,
        y: f64,
    },
    PageAt {
        x: f64,
        y: f64,
    },
    PointInViewport {
        page: u32,
        x: f64,
        y: f64,
    },
    EnsureCaret {
        page: u32,
        rect: NativeViewRect,
        margin: f64,
    },
}

/// Platform-neutral host operations. Only workerOpen consumes binary document bytes.
/// # Safety
/// Each nonempty span must be readable for its declared length during the call.
#[no_mangle]
pub unsafe extern "C" fn rhwp_native_host_request(
    request: *const u8,
    request_len: usize,
    data: *const u8,
    data_len: usize,
) -> *mut c_char {
    response(|| {
        let request = unsafe { input(request, request_len, MAX_REQUEST_BYTES)? };
        let data = unsafe { input(data, data_len, MAX_DOCUMENT_BYTES)? };
        let request: HostRequest = serde_json::from_slice(request)
            .map_err(|_| NativeHostError::InvalidArgument("invalid host request JSON".into()))?;
        if !matches!(&request, HostRequest::WorkerOpen { .. }) && !data.is_empty() {
            return Err(NativeHostError::InvalidArgument(
                "this request does not accept binary data".into(),
            ));
        }
        match request {
            HostRequest::WorkerCreate { config } => HOST.with(|host| {
                let mut host = host.borrow_mut();
                if host.workers.len() >= 4 { return Err(NativeHostError::LimitExceeded("release finished workers before creating more (max 4)".into())); }
                let runtime = match NativeRuntime::spawn(config) { Ok(runtime) => runtime, Err(error) => return Ok(json!({"ok":false,"code":error.code(),"error":error.to_string()})) };
                let worker = next_handle()?;
                host.workers.insert(worker,runtime);
                Ok(json!({"ok":true,"worker":worker}))
            }),
            HostRequest::WorkerSubmit { worker, command } => HOST.with(|host| {
                let mut host=host.borrow_mut();
                let runtime=host.workers.get_mut(&worker).ok_or_else(invalid_handle)?;
                Ok(worker_submit_result(runtime.try_submit(command)))
            }),
            HostRequest::WorkerOpen { worker,replace,password } => HOST.with(|host| {
                let mut host=host.borrow_mut();
                let runtime=host.workers.get_mut(&worker).ok_or_else(invalid_handle)?;
                Ok(worker_submit_result(runtime.try_submit(NativeRuntimeCommand::Open {replace,bytes:data.to_vec(),password})))
            }),
            HostRequest::WorkerPoll { worker } => HOST.with(|host| {
                let mut host=host.borrow_mut();
                let runtime=host.workers.get_mut(&worker).ok_or_else(invalid_handle)?;
                match runtime.try_recv() {
                    Ok(None) => Ok(json!({"ok":true,"completion":null})),
                    Ok(Some(completion)) => {
                        let mut output=match completion.result {
                            Ok(output) => worker_output(&mut host,worker,output),
                            Err(error) => json!({"ok":false,"code":error.code,"error":error.message}),
                        };
                        output["requestId"]=json!(completion.request_id);
                        Ok(json!({"ok":true,"completion":output}))
                    }
                    Err(error) => Ok(json!({"ok":false,"code":error.code(),"error":error.to_string()})),
                }
            }),
            HostRequest::WorkerCancel { worker,request_id } => HOST.with(|host| {
                let host=host.borrow();
                let runtime=host.workers.get(&worker).ok_or_else(invalid_handle)?;
                Ok(json!({"ok":true,"outcome":runtime.cancel(request_id)}))
            }),
            HostRequest::WorkerClose { worker } => HOST.with(|host| {
                let mut host=host.borrow_mut();
                let runtime=host.workers.get_mut(&worker).ok_or_else(invalid_handle)?;
                runtime.close();
                Ok(json!({"ok":true,"isFinished":runtime.is_finished()}))
            }),
            HostRequest::WorkerStatus { worker } => HOST.with(|host| {
                let host=host.borrow();
                let runtime=host.workers.get(&worker).ok_or_else(invalid_handle)?;
                Ok(json!({"ok":true,"isFinished":runtime.is_finished()}))
            }),
            HostRequest::WorkerRelease { worker } => HOST.with(|host| {
                let mut host=host.borrow_mut();
                if !host.workers.get(&worker).ok_or_else(invalid_handle)?.is_finished() {return Err(NativeHostError::InvalidArgument("close worker and wait for isFinished before releasing it or unloading the library".into()));}
                let mut runtime=host.workers.remove(&worker).ok_or_else(invalid_handle)?;
                match runtime.try_join() {
                    Ok(true)=>Ok(json!({"ok":true})),
                    Ok(false)=>unreachable!("finished worker can join"),
                    Err(error)=>Ok(json!({"ok":false,"code":error.code(),"error":error.to_string()})),
                }
            }),
            HostRequest::View { state, command } => {
                let mut view = NativeView::try_from(state)?;
                let mut query = Value::Null;
                match command {
                    ViewCommand::State {} => {}
                    ViewCommand::Configure { config } => {
                        view.set_config(config)?;
                    }
                    ViewCommand::Scroll { x, y } => {
                        view.set_scroll(NativeViewPoint { x, y })?;
                    }
                    ViewCommand::ScrollBy { x, y } => {
                        view.scroll_by(NativeViewPoint { x, y })?;
                    }
                    ViewCommand::ZoomAt { zoom, x, y } => {
                        view.zoom_at(zoom, NativeViewPoint { x, y })?;
                    }
                    ViewCommand::PageAt { x, y } => {
                        query = json!(view.viewport_to_page(NativeViewPoint { x, y })?);
                    }
                    ViewCommand::PointInViewport { page, x, y } => {
                        query = json!(view.page_to_viewport(NativePagePoint {
                            page,
                            point: NativeViewPoint { x, y }
                        })?);
                    }
                    ViewCommand::EnsureCaret { page, rect, margin } => {
                        view.ensure_caret_visible(page, rect, margin)?;
                    }
                }
                Ok(
                    json!({"ok":true,"state":view.state(),"contentSize":view.content_size(),"scrollLimit":view.scroll_limit(),"visible":view.visible_regions(),"rasterScale":view.raster_scale(),"query":query}),
                )
            }
        }
    })
}

fn worker_submit_result(
    result: Result<NativeRequestId, rhwp_core::native_host::runtime::NativeRuntimeSubmitError>,
) -> Value {
    match result {
        Ok(id) => json!({"ok":true,"requestId":id}),
        Err(error) => json!({"ok":false,"code":error.error.code(),"error":error.error.to_string()}),
    }
}

fn worker_output(host: &mut Host, worker: u64, output: NativeRuntimeOutput) -> Value {
    let cancel = match &output {
        NativeRuntimeOutput::PreparedSave {
            document_id,
            prepared,
        } => Some((*document_id, prepared.token)),
        _ => None,
    };
    let result: Result<Value, NativeHostError> = (|| {
        Ok(match output {
            NativeRuntimeOutput::CommandExecuted(result) => {
                json!({"ok":true,"kind":"command","outcome":result.outcome,"preferredX":result.preferred_x})
            }
            NativeRuntimeOutput::ImeContext(context) => {
                json!({"ok":true,"kind":"imeContext","context":context})
            }
            NativeRuntimeOutput::Clipboard { payload, outcome } => {
                json!({"ok":true,"kind":"clipboard","payload":payload,"outcome":outcome})
            }
            NativeRuntimeOutput::Accessibility { target, snapshot } => {
                json!({"ok":true,"kind":"accessibility","target":target,"snapshot":snapshot})
            }
            NativeRuntimeOutput::Info(info) => {
                let mut encoded = json!(info);
                if let Some(fields) = encoded.as_object_mut() {
                    if let Some(dirty) = fields.remove("dirty") {
                        fields.insert("isDirty".into(), dirty);
                    }
                }
                json!({"ok":true,"kind":"info","info":encoded})
            }
            NativeRuntimeOutput::DocumentClosed => json!({"ok":true,"kind":"documentClosed"}),
            NativeRuntimeOutput::PageSize { target, page, size } => {
                json!({"ok":true,"kind":"pageSize","target":target,"page":page,"size":size})
            }
            NativeRuntimeOutput::CaretRect { target, caret } => {
                json!({"ok":true,"kind":"caret","target":target,"caret":caret})
            }
            NativeRuntimeOutput::HitTest { target, hit } => {
                json!({"ok":true,"kind":"hit","target":target,"hit":hit})
            }
            NativeRuntimeOutput::SelectionRects { target, rects } => {
                json!({"ok":true,"kind":"selectionRects","target":target,"rects":rects})
            }
            NativeRuntimeOutput::Edited(outcome) => {
                json!({"ok":true,"kind":"edited","outcome":outcome})
            }
            NativeRuntimeOutput::Frame { document_id, frame } => {
                let mut value = frame_result(host, frame)?;
                value["kind"] = json!("frame");
                value["documentId"] = json!(document_id);
                value
            }
            NativeRuntimeOutput::PreparedSave {
                document_id,
                prepared,
            } => {
                let byte_length = prepared.bytes.len();
                let report: Value = serde_json::from_str(&prepared.content_loss.to_json())
                    .expect("valid content-loss JSON");
                let buffer = retain_buffer(host, prepared.bytes.into_vec())?;
                json!({"ok":true,"kind":"preparedSave","documentId":document_id,"buffer":buffer,"byteLength":byte_length,
                "token":prepared.token,"revision":prepared.revision,"report":report,"caretSaved":prepared.caret_saved,
                "caretPreservedFromSource":prepared.caret_preserved_from_source,"caretPolicy":prepared.caret_policy})
            }
            NativeRuntimeOutput::SaveStatus {
                document_id,
                status,
            } => {
                json!({"ok":true,"kind":"saveStatus","documentId":document_id,"isDirty":status.dirty,"status":status})
            }
            NativeRuntimeOutput::Cancelled => json!({"ok":true,"kind":"cancelled"}),
            NativeRuntimeOutput::Skipped(reason) => {
                json!({"ok":true,"kind":"skipped","reason":reason})
            }
        })
    })();
    match result {
        Ok(value) => value,
        Err(error) => {
            let mut value = json!({"ok":false,"code":error.code(),"error":error.to_string()});
            if let Some((document_id, token)) = cancel {
                if let Some(runtime) = host.workers.get_mut(&worker) {
                    let cleanup = worker_submit_result(
                        runtime.try_submit(NativeRuntimeCommand::CancelSave { document_id, token }),
                    );
                    value["saveCleanup"] = cleanup;
                }
            }
            value
        }
    }
}

fn next_handle() -> Result<u64, NativeHostError> {
    NEXT_HANDLE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| NativeHostError::LimitExceeded("handle space exhausted".into()))
}

fn response(f: impl FnOnce() -> Result<Value, NativeHostError>) -> *mut c_char {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    let value = match result {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"ok":false,"code":error.code(),"error":error.to_string()}),
        Err(_) => {
            // 이후 호출이 부분 mutation 상태를 소비하지 않도록 이 스레드 세션을 폐기한다.
            HOST.with(|host| *host.borrow_mut() = Host::default());
            json!({"ok":false,"code":"panic","error":"native host panicked; thread handles invalidated"})
        }
    };
    CString::new(value.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

/// # Safety
/// 비어 있지 않은 입력은 호출 동안 읽을 수 있는 `len` 바이트를 가리켜야 한다.
unsafe fn input<'a>(ptr: *const u8, len: usize, limit: usize) -> Result<&'a [u8], NativeHostError> {
    if len > limit {
        return Err(NativeHostError::LimitExceeded(
            "input exceeds byte limit".into(),
        ));
    }
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(NativeHostError::InvalidArgument("null input".into()));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn invalid_handle() -> NativeHostError {
    NativeHostError::InvalidArgument("unknown, closed, or wrong-thread handle".into())
}

fn retain_buffer(host: &mut Host, bytes: Vec<u8>) -> Result<u64, NativeHostError> {
    let total = host
        .buffer_bytes
        .checked_add(bytes.len())
        .ok_or_else(|| NativeHostError::LimitExceeded("buffer size overflow".into()))?;
    if total > MAX_BUFFER_BYTES || host.buffers.len() >= MAX_BUFFERS {
        return Err(NativeHostError::LimitExceeded(
            "release buffers before requesting more output".into(),
        ));
    }
    let id = next_handle()?;
    host.buffer_bytes = total;
    host.buffers.insert(id, bytes);
    Ok(id)
}

fn frame_result(
    host: &mut Host,
    frame: rhwp_core::native_host::NativeFrame,
) -> Result<Value, NativeHostError> {
    let pixels = frame.pixels;
    let metadata = (
        pixels.width,
        pixels.height,
        pixels.stride,
        pixels.bytes.len(),
    );
    let buffer = retain_buffer(host, pixels.bytes)?;
    Ok(
        json!({"ok":true,"buffer":buffer,"width":metadata.0,"height":metadata.1,
        "stride":metadata.2,"byteLength":metadata.3,"revision":frame.revision,
        "fontGeneration":frame.font_generation,"originX":frame.origin_x,"originY":frame.origin_y,
        "pixelFormat":"rgba8Premultiplied","colorSpace":"srgb"}),
    )
}

#[no_mangle]
pub extern "C" fn rhwp_session_capabilities() -> *mut c_char {
    response(|| {
        Ok(
            json!({"ok":true,"abiVersion":1,"nativeSkia":cfg!(feature="native-skia"),
        "worker":true,"maxWorkers":4,"hostRequests":true,"viewState":true,"accessibilitySnapshots":true,"threadAffine":true,"maxSessions":MAX_SESSIONS,"maxDocumentBytes":MAX_DOCUMENT_BYTES,
        "maxRequestBytes":MAX_REQUEST_BYTES,"maxBufferBytes":MAX_BUFFER_BYTES,
        "pixelFormat":"rgba8Premultiplied","colorSpace":"srgb"}),
        )
    })
}

/// 문서 길이가 0이면 내장 템플릿으로 새 문서를 만든다.
/// # Safety
/// data/password는 각각 지정 길이만큼 읽을 수 있는 영역이어야 한다.
#[no_mangle]
pub unsafe extern "C" fn rhwp_session_open(
    data: *const u8,
    len: usize,
    password: *const u8,
    password_len: usize,
) -> *mut c_char {
    response(|| {
        let bytes = unsafe { input(data, len, MAX_DOCUMENT_BYTES)? };
        let password = unsafe { input(password, password_len, 4096)? };
        HOST.with(|host| {
            let mut host = host.borrow_mut();
            if host.sessions.len() >= MAX_SESSIONS {
                return Err(NativeHostError::LimitExceeded(
                    "too many open sessions".into(),
                ));
            }
            if bytes.is_empty() && !password.is_empty() {
                return Err(NativeHostError::InvalidArgument(
                    "new document does not accept password".into(),
                ));
            }
            let session = if bytes.is_empty() {
                NativeDocumentSession::new_blank()?
            } else {
                NativeDocumentSession::open_bytes(
                    bytes,
                    (!password.is_empty()).then_some(password),
                )?
            };
            let id = next_handle()?;
            let page_count = session.document().page_count();
            host.sessions
                .insert(id, ManagedSession::View(Box::new(session)));
            Ok(json!({"ok":true,"handle":id,"pageCount":page_count,"revision":0}))
        })
    })
}

/// # Safety
/// request는 호출 중 읽을 수 있는 request_len 바이트의 UTF-8 JSON이어야 한다.
#[no_mangle]
pub unsafe extern "C" fn rhwp_session_request(
    handle: u64,
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    response(|| {
        let bytes = unsafe { input(request, request_len, MAX_REQUEST_BYTES)? };
        let request: Request = serde_json::from_slice(bytes)
            .map_err(|_| NativeHostError::InvalidArgument("invalid native request JSON".into()))?;
        HOST.with(|host| {
            let mut host = host.borrow_mut();
            if let Request::BeginEdit { selection, expected_revision } = request {
                return host.begin_edit(handle, expected_revision, selection);
            }
            let record = host.sessions.get_mut(&handle).ok_or_else(invalid_handle)?;
            let session = record.session();
            match request {
                Request::Accessibility { request, selection } => {
                    let selection = selection.as_ref().or_else(|| match &*record {
                        ManagedSession::Edit(editor) => Some(editor.selection()),
                        ManagedSession::View(_) => None,
                    });
                    Ok(json!({"ok":true,"snapshot":session.accessibility_snapshot(&request, selection)?}))
                }
                Request::ExecuteCommand {command,expected_revision}=>{let result=record.editor_mut()?.execute_command(expected_revision,command)?;Ok(json!({"ok":true,"outcome":result.outcome,"preferredX":result.preferred_x}))},
                Request::ImeContext {expected_revision}=>Ok(json!({"ok":true,"context":record.editor_mut()?.ime_context(expected_revision)?})),
                Request::CopySelection {expected_revision}=>Ok(json!({"ok":true,"kind":"clipboard","payload":record.editor_mut()?.copy_selection(expected_revision)?,"outcome":null})),
                Request::CutSelection {expected_revision}=>{
                    let (payload,outcome)=record.editor_mut()?.cut_selection(expected_revision)?;
                    Ok(json!({"ok":true,"kind":"clipboard","payload":payload,"outcome":outcome}))
                },
                Request::PasteText {text,expected_revision}=>Ok(edit_result(record.editor_mut()?.paste_text(expected_revision,&text)?)),
                Request::PasteHtml {html,expected_revision}=>Ok(edit_result(record.editor_mut()?.paste_html(expected_revision,&html)?)),
                Request::PrepareSave { format } => {
                    let prepared = record.prepare_save(format)?;
                    let caret_saved = prepared.caret_saved();
                    let caret_preserved = prepared.caret_preserved_from_source();
                    let caret_policy = prepared.caret_policy();
                    let (bytes,report,revision,token) = prepared.into_parts();
                    let report: Value = serde_json::from_str(&report.to_json()).expect("content-loss report is valid JSON");
                    let byte_length = bytes.len();
                    let buffer = match retain_buffer(&mut host, bytes) {
                        Ok(buffer) => buffer,
                        Err(error) => {
                            host.sessions.get_mut(&handle).ok_or_else(invalid_handle)?.cancel_save(token)?;
                            return Err(error);
                        }
                    };
                    Ok(json!({"ok":true,"buffer":buffer,"byteLength":byte_length,"token":token,"revision":revision,"report":report,"caretSaved":caret_saved,"caretPreservedFromSource":caret_preserved,"caretPolicy":caret_policy}))
                }
                Request::AcknowledgeSave { token } => {
                    let status = record.acknowledge_save(token)?;
                    Ok(json!({"ok":true,"isDirty":status.dirty,"status":status}))
                }
                Request::CancelSave { token } => {
                    let status = record.cancel_save(token)?;
                    Ok(json!({"ok":true,"isDirty":status.dirty,"status":status}))
                }
                Request::BeginEdit { .. } => unreachable!("handled before borrowing session"),
                Request::SetSelection { selection, expected_revision } => Ok(edit_result(record.editor_mut()?.set_selection(expected_revision,selection)?)),
                Request::ReplaceSelection { text, expected_revision } => Ok(edit_result(record.editor_mut()?.replace_selection(expected_revision,&text)?)),
                Request::BeginComposition { expected_revision } => Ok(edit_result(record.editor_mut()?.begin_composition(expected_revision)?)),
                Request::UpdateComposition { composition_id, text, expected_revision } => Ok(edit_result(record.editor_mut()?.update_composition(expected_revision,composition_id,&text)?)),
                Request::CommitComposition { composition_id, expected_revision } => Ok(edit_result(record.editor_mut()?.commit_composition(expected_revision,composition_id)?)),
                Request::CancelComposition { composition_id, expected_revision } => Ok(edit_result(record.editor_mut()?.cancel_composition(expected_revision,composition_id)?)),
                Request::FocusLost { policy, expected_revision } => Ok(edit_result(record.editor_mut()?.focus_lost(expected_revision,policy)?)),
                Request::Undo { expected_revision } => Ok(edit_result(record.editor_mut()?.undo(expected_revision)?)),
                Request::Redo { expected_revision } => Ok(edit_result(record.editor_mut()?.redo(expected_revision)?)),
                Request::ClearHistory { expected_revision } => Ok(edit_result(record.editor_mut()?.clear_history(expected_revision)?)),
                Request::HitTest { page, x, y } => Ok(json!({"ok":true,"hit":session.document().native_hit_test(page,x,y)?})),
                Request::Caret { position } => Ok(json!({"ok":true,"caret":session.document().native_caret_rect(&position)?})),
                Request::SelectionRects { selection } => Ok(json!({"ok":true,"rects":session.document().native_selection_rects(&selection)?})),
                Request::MoveVertical { position, direction, preferred_x } => Ok(json!({"ok":true,"movement":session.document().native_move_vertical(&position,direction,preferred_x)?})),
                Request::FontState {} => Ok(json!({"ok":true,"state":session.font_state(),"fontGeneration":session.font_generation()})),
                Request::SetFonts { paths, environment } => {
                    let changed = record.set_fonts(&paths, environment)?;
                    let session = record.session();
                    Ok(json!({"ok":true,"changed":changed,"state":session.font_state(),"fontGeneration":session.font_generation()}))
                }
                Request::IsFrameCurrent { revision, font_generation } => Ok(json!({"ok":true,"current":session.is_frame_current(revision, font_generation)})),
                Request::ClearCaches {} => { session.clear_caches(); Ok(json!({"ok":true})) }
                Request::CacheStats {} => {
                    #[cfg(feature = "native-skia")]
                    { Ok(json!({"ok":true,"stats":session.cache_stats()})) }
                    #[cfg(not(feature = "native-skia"))]
                    { Err(NativeHostError::FeatureDisabled) }
                }
                Request::Info {} => {
                    let (can_undo,can_redo,composition_id) = record.editing_info();
                    Ok(json!({"ok":true,"pageCount":session.document().page_count(),"revision":session.revision(),"fontGeneration":session.font_generation(),"canUndo":can_undo,"canRedo":can_redo,"compositionId":composition_id,"isDirty":session.is_dirty()}))
                },
                Request::PageSize { page } => Ok(json!({"ok":true,"page":page,"size":session.page_size(page)?})),
                Request::Render { page, scale } => {
                    let frame = session.render_page(page, scale)?;
                    frame_result(&mut host, frame)
                }
                Request::RenderRegion { page, x, y, width, height, scale } => {
                    let frame = session.render_region(page,
                        rhwp_core::renderer::raster_pixels::RasterRegion { x, y, width, height }, scale)?;
                    frame_result(&mut host, frame)
                }
            }
        })
    })
}

#[no_mangle]
pub extern "C" fn rhwp_session_close(handle: u64) -> *mut c_char {
    response(|| {
        HOST.with(|host| {
            host.borrow_mut()
                .sessions
                .remove(&handle)
                .ok_or_else(invalid_handle)?;
            Ok(json!({"ok":true}))
        })
    })
}

/// # Safety
/// destination은 capacity 바이트만큼 쓸 수 있고 보관 버퍼와 겹치지 않아야 한다.
#[no_mangle]
pub unsafe extern "C" fn rhwp_session_buffer_copy(
    buffer: u64,
    destination: *mut u8,
    capacity: usize,
) -> *mut c_char {
    response(|| {
        HOST.with(|host| {
            let host = host.borrow();
            let bytes = host.buffers.get(&buffer).ok_or_else(invalid_handle)?;
            if capacity < bytes.len() || destination.is_null() {
                return Err(NativeHostError::InvalidArgument(
                    "output buffer is null or too small".into(),
                ));
            }
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
            }
            Ok(json!({"ok":true,"byteLength":bytes.len()}))
        })
    })
}

#[no_mangle]
pub extern "C" fn rhwp_session_buffer_release(buffer: u64) -> *mut c_char {
    response(|| {
        HOST.with(|host| {
            let mut host = host.borrow_mut();
            let bytes = host.buffers.remove(&buffer).ok_or_else(invalid_handle)?;
            host.buffer_bytes -= bytes.len();
            Ok(json!({"ok":true}))
        })
    })
}
