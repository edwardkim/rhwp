//! 길이가 명시된 요청과 스레드 소유 handle을 사용하는 native host ABI.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};

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
    sessions: HashMap<u64, NativeDocumentSession>,
    buffers: HashMap<u64, Vec<u8>>,
    buffer_bytes: usize,
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
    Info {},
    FontState,
    SetFonts {
        paths: Vec<std::path::PathBuf>,
        environment: Option<rhwp_core::renderer::font_environment::FontEnvironment>,
    },
    IsFrameCurrent {
        revision: u64,
        font_generation: u64,
    },
    ClearCaches,
    CacheStats,
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
        "threadAffine":true,"maxSessions":MAX_SESSIONS,"maxDocumentBytes":MAX_DOCUMENT_BYTES,
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
            host.sessions.insert(id, session);
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
            let session = host.sessions.get_mut(&handle).ok_or_else(invalid_handle)?;
            match request {
                Request::FontState => Ok(json!({"ok":true,"state":session.font_state(),"fontGeneration":session.font_generation()})),
                Request::SetFonts { paths, environment } => {
                    let changed = session.set_font_sources(&paths, environment)?;
                    Ok(json!({"ok":true,"changed":changed,"state":session.font_state(),"fontGeneration":session.font_generation()}))
                }
                Request::IsFrameCurrent { revision, font_generation } => Ok(json!({"ok":true,"current":session.is_frame_current(revision, font_generation)})),
                Request::ClearCaches => { session.clear_caches(); Ok(json!({"ok":true})) }
                Request::CacheStats => {
                    #[cfg(feature = "native-skia")]
                    { Ok(json!({"ok":true,"stats":session.cache_stats()})) }
                    #[cfg(not(feature = "native-skia"))]
                    { Err(NativeHostError::FeatureDisabled) }
                }
                Request::Info {} => Ok(json!({"ok":true,"pageCount":session.document().page_count(),"revision":session.revision()})),
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
