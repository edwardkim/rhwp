//! Typed Metal pointers with owner-thread handles. P89 snapshots remain the
//! only cross-thread input; this is not a cross-thread legacy worker API.

use std::ffi::{c_char, c_void, CString};

use rhwp_core::native_host::NativeHostError;
use serde_json::{json, Value};

fn response(f: impl FnOnce() -> Result<Value, NativeHostError>) -> *mut c_char {
    let value = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            let stale = matches!(&error, NativeHostError::InvalidArgument(message) if message == "stale presentation render stamp");
            // Stale checks occur before any target use or after a successful
            // producer GPU wait. Never exposed targets can then be discarded.
            json!({"ok":false,"code":if stale {"staleFrame"} else {error.code()},"retryable":stale,
                "unpublishedTargetSafeToDiscard":stale,"error":error.to_string()})
        }
        Err(_) => {
            #[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
            enabled::abandon_all();
            json!({"ok":false,"code":"panic","error":"Metal presenter panicked; owner handles abandoned"})
        }
    };
    CString::new(value.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

/// Feature probe is always exported. Production availability is conditional;
/// no CPU readback or consumer-completion guarantee is implied by this probe.
#[no_mangle]
pub extern "C" fn rhwp_metal_capabilities() -> *mut c_char {
    response(|| {
        Ok(json!({
            "ok":true,"abiVersion":1,
            "available":cfg!(all(feature="native-skia-metal",target_os="macos")),
            "backend":"metalGanesh","typedPointers":true,"ownerThreadHandles":true,
            "pixelFormat":"bgra8UnormPremultiplied","origin":"topLeft","colorSpace":"srgb",
            "producerGpuCompletion":"synchronousOwnerThread","consumerGpuCompletion":false,
            "hardwareAccelerationVerified":false,
            "maxPresentersPerThread":2,"maxTargetBytes":16*1024*1024,"maxPages":8,
            "wholeFrameReadbacks":0,
        }))
    })
}

#[cfg(not(all(feature = "native-skia-metal", target_os = "macos")))]
fn disabled() -> Result<Value, NativeHostError> {
    Ok(
        json!({"ok":false,"code":"featureDisabled","error":"native-skia-metal on macOS is required"}),
    )
}

/// # Safety
/// Enabled builds require real id<MTLDevice>/id<MTLCommandQueue> pointers from
/// the same device, borrowed during creation. Skia takes its own references.
/// Config is readable for config_len bytes. Call only on a dedicated producer
/// OS thread; its handles must be rendered/released on that same thread.
#[no_mangle]
pub unsafe extern "C" fn rhwp_metal_presenter_create(
    device: *const c_void,
    command_queue: *const c_void,
    device_generation: u64,
    config: *const u8,
    config_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
        {
            unsafe { enabled::create(device, command_queue, device_generation, config, config_len) }
        }
        #[cfg(not(all(feature = "native-skia-metal", target_os = "macos")))]
        {
            let _ = (device, command_queue, device_generation, config, config_len);
            disabled()
        }
    })
}

/// # Safety
/// texture is a real, retained id<MTLTexture> matching the presenter's device:
/// BGRA8Unorm, 2D, single-sample, render-target and shader-read usage, top-left
/// premultiplied pixels. JSON dimensions/generation accurately describe it.
/// The target is exclusively writable until successful return. Successful
/// producer completion does NOT permit reuse while Flutter may sample it.
/// Request bytes stay readable for request_len throughout this synchronous call.
#[no_mangle]
pub unsafe extern "C" fn rhwp_metal_presenter_render(
    presenter: u64,
    texture: *const c_void,
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
        {
            unsafe { enabled::render(presenter, texture, request, request_len) }
        }
        #[cfg(not(all(feature = "native-skia-metal", target_os = "macos")))]
        {
            let _ = (presenter, texture, request, request_len);
            disabled()
        }
    })
}

/// # Safety
/// Owner-thread info/release/abandon only; no pointer values are accepted in
/// JSON. A nonempty request must point to request_len readable bytes.
#[no_mangle]
pub unsafe extern "C" fn rhwp_metal_presenter_request(
    presenter: u64,
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
        {
            unsafe { enabled::request(presenter, request, request_len) }
        }
        #[cfg(not(all(feature = "native-skia-metal", target_os = "macos")))]
        {
            let _ = (presenter, request, request_len);
            disabled()
        }
    })
}

/// Bounded cross-thread source/stamp validation, without parsing/layout/replay.
/// Unlike snapshotInfo.current this checks configured view/device generations.
///
/// # Safety
/// Enabled builds require request_len readable bytes at request. Keep all
/// referenced snapshot handles alive during the call. No presenter is touched.
#[no_mangle]
pub unsafe extern "C" fn rhwp_metal_frame_check(
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
        {
            unsafe { enabled::check_frame(request, request_len) }
        }
        #[cfg(not(all(feature = "native-skia-metal", target_os = "macos")))]
        {
            let _ = (request, request_len);
            disabled()
        }
    })
}

#[cfg(all(feature = "native-skia-metal", target_os = "macos"))]
mod enabled {
    use super::*;
    use rhwp_core::native_host::gpu_metal::{
        NativeMetalPlacement, NativeMetalPresenter, NativeMetalPresenterConfig, NativeMetalTarget,
    };
    use rhwp_core::native_host::presentation::NativePresentationStamp;
    use serde::Deserialize;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    thread_local! { static OWNERS:RefCell<HashMap<u64,NativeMetalPresenter>> = RefCell::new(HashMap::new()); }
    const MAX_REQUEST_BYTES: usize = 64 * 1024;

    fn invalid_handle() -> NativeHostError {
        NativeHostError::InvalidArgument("unknown or foreign-thread Metal presenter".into())
    }
    unsafe fn bytes<'a>(pointer: *const u8, length: usize) -> Result<&'a [u8], NativeHostError> {
        if length > MAX_REQUEST_BYTES || (length != 0 && pointer.is_null()) {
            return Err(NativeHostError::InvalidArgument(
                "invalid Metal request/config span".into(),
            ));
        }
        Ok(if length == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(pointer, length) }
        })
    }
    pub(super) unsafe fn create(
        device: *const c_void,
        queue: *const c_void,
        generation: u64,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let data = unsafe { bytes(pointer, length)? };
        let config: NativeMetalPresenterConfig = if data.is_empty() {
            Default::default()
        } else {
            serde_json::from_slice(data).map_err(|_| {
                NativeHostError::InvalidArgument("invalid Metal presenter config".into())
            })?
        };
        config.validate()?;
        OWNERS.with(|owners| {
            if owners.borrow().len() >= 2 {
                Err(NativeHostError::LimitExceeded(
                    "Metal presenter count quota exceeded".into(),
                ))
            } else {
                Ok(())
            }
        })?;
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| NativeHostError::LimitExceeded("Metal handle space exhausted".into()))?;
        let presenter = unsafe { NativeMetalPresenter::new(device, queue, generation, config)? };
        let info = presenter.info()?;
        OWNERS.with(|owners| {
            owners.borrow_mut().insert(id, presenter);
        });
        Ok(json!({"ok":true,"presenter":id,"info":info}))
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct RenderRequest {
        stamp: NativePresentationStamp,
        target: NativeMetalTarget,
        placements: Vec<NativeMetalPlacement>,
        clear_color: u32,
    }
    pub(super) unsafe fn render(
        id: u64,
        texture: *const c_void,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let request: RenderRequest = serde_json::from_slice(unsafe { bytes(pointer, length)? })
            .map_err(|_| NativeHostError::InvalidArgument("invalid Metal render request".into()))?;
        OWNERS.with(|owners| {
            let mut owners = owners.borrow_mut();
            let presenter = owners.get_mut(&id).ok_or_else(invalid_handle)?;
            let frame = unsafe {
                presenter.render(
                    crate::presentation::registry(),
                    request.stamp,
                    texture,
                    request.target,
                    &request.placements,
                    request.clear_color,
                )?
            };
            Ok(json!({"ok":true,"frame":frame}))
        })
    }
    #[derive(Deserialize)]
    #[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
    enum Request {
        Info {},
        Release {},
        Abandon {},
    }
    pub(super) unsafe fn request(
        id: u64,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let request: Request =
            serde_json::from_slice(unsafe { bytes(pointer, length)? }).map_err(|_| {
                NativeHostError::InvalidArgument("invalid Metal presenter request".into())
            })?;
        OWNERS.with(|owners| {
            let mut owners = owners.borrow_mut();
            let presenter = owners.get_mut(&id).ok_or_else(invalid_handle)?;
            match request {
                Request::Info {} => Ok(json!({"ok":true,"info":presenter.info()?})),
                Request::Release {} => {
                    presenter.release()?;
                    owners.remove(&id);
                    Ok(json!({"ok":true}))
                }
                Request::Abandon {} => {
                    presenter.abandon();
                    owners.remove(&id);
                    Ok(json!({"ok":true}))
                }
            }
        })
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct FrameCheck {
        stamp: NativePresentationStamp,
        snapshots: Vec<rhwp_core::native_host::presentation::NativePresentationId>,
    }
    pub(super) unsafe fn check_frame(
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let request: FrameCheck = serde_json::from_slice(unsafe { bytes(pointer, length)? })
            .map_err(|_| NativeHostError::InvalidArgument("invalid Metal frame check".into()))?;
        if request.snapshots.is_empty() || request.snapshots.len() > 8 {
            return Err(NativeHostError::LimitExceeded(
                "Metal frame check requires 1..8 snapshots".into(),
            ));
        }
        for snapshot in request.snapshots {
            crate::presentation::registry().borrow_snapshot(snapshot, request.stamp)?;
        }
        Ok(json!({"ok":true,"current":true}))
    }
    pub(super) fn abandon_all() {
        let _ = OWNERS.try_with(|owners| {
            if let Ok(mut owners) = owners.try_borrow_mut() {
                for presenter in owners.values_mut() {
                    presenter.abandon();
                }
                owners.clear();
            }
        });
    }
}
