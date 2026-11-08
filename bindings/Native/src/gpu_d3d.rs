//! Owner-thread D3D12/Ganesh C ABI. Disabled builds ignore all input spans.
//! P89 presentation handles are the only cross-thread rendering input.
use std::ffi::{c_char, c_void, CString};

#[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
fn disabled() -> *mut c_char {
    CString::new(
        r#"{"ok":false,"code":"featureDisabled","error":"native-skia-d3d on Windows is required"}"#,
    )
    .expect("constant JSON")
    .into_raw()
}

#[no_mangle]
pub extern "C" fn rhwp_d3d_capabilities() -> *mut c_char {
    #[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
    {
        enabled::response(enabled::capabilities)
    }
    #[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
    {
        CString::new(r#"{"ok":true,"abiVersion":1,"available":false,"producerApi":"d3d12Ganesh","consumerBridge":"d3d11GpuCopyToLegacyDxgiHandle","typedPointers":true,"ownerThreadHandles":true,"consumerGpuCompletion":false,"wholeFrameReadbacks":0}"#).expect("constant JSON").into_raw()
    }
}

/// # Safety
/// Enabled builds require live, matching IDXGIAdapter1/ID3D12Device/direct
/// ID3D12CommandQueue pointers. Native clones their COM references. Config is a
/// readable bounded span. All presenter calls stay on this producer OS thread.
#[no_mangle]
pub unsafe extern "C" fn rhwp_d3d_presenter_create(
    adapter: *const c_void,
    device: *const c_void,
    queue: *const c_void,
    generation: u64,
    config: *const u8,
    length: usize,
) -> *mut c_char {
    #[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
    {
        enabled::response(|| unsafe {
            enabled::create(adapter, device, queue, generation, config, length)
        })
    }
    #[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
    {
        let _ = (adapter, device, queue, generation, config, length);
        disabled()
    }
}

/// # Safety
/// Enabled builds require a live, unpublished, matching-device shared
/// ID3D12Resource in COMMON with the fixed target contract. Keep it exclusively
/// writable and retained until checked producer completion. Render failure with
/// no safe-to-discard evidence requires quarantine; Flutter consumption never
/// permits reuse without a separate consumer-completion proof.
#[no_mangle]
pub unsafe extern "C" fn rhwp_d3d_presenter_render(
    presenter: u64,
    resource: *const c_void,
    request: *const u8,
    length: usize,
) -> *mut c_char {
    #[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
    {
        enabled::response(|| unsafe { enabled::render(presenter, resource, request, length) })
    }
    #[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
    {
        let _ = (presenter, resource, request, length);
        disabled()
    }
}

/// # Safety
/// Enabled info/release/abandon requests are bounded readable JSON. Presenter
/// handles belong to their creating OS thread; no raw pointers enter JSON.
#[no_mangle]
pub unsafe extern "C" fn rhwp_d3d_presenter_request(
    presenter: u64,
    request: *const u8,
    length: usize,
) -> *mut c_char {
    #[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
    {
        enabled::response(|| unsafe { enabled::request(presenter, request, length) })
    }
    #[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
    {
        let _ = (presenter, request, length);
        disabled()
    }
}

/// # Safety
/// Enabled builds require bounded readable {stamp,snapshots:[1..8]} JSON and
/// retained P89 snapshot IDs. Checks every source/view/device/DPR generation
/// without touching a presenter, Core, layout or GPU. Safe across OS threads.
#[no_mangle]
pub unsafe extern "C" fn rhwp_d3d_frame_check(request: *const u8, length: usize) -> *mut c_char {
    #[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
    {
        enabled::response(|| unsafe { enabled::check_frame(request, length) })
    }
    #[cfg(not(all(feature = "native-skia-d3d", target_os = "windows")))]
    {
        let _ = (request, length);
        disabled()
    }
}

#[cfg(all(feature = "native-skia-d3d", target_os = "windows"))]
mod enabled {
    use super::*;
    use rhwp_core::native_host::gpu_d3d::{
        NativeD3dPlacement, NativeD3dPresenter, NativeD3dPresenterConfig, NativeD3dTarget,
    };
    use rhwp_core::native_host::presentation::NativePresentationStamp;
    use rhwp_core::native_host::NativeHostError;
    use serde::Deserialize;
    use serde_json::{json, Value};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    pub(super) fn response(f: impl FnOnce() -> Result<Value, NativeHostError>) -> *mut c_char {
        let value = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => {
                let stale = matches!(&error, NativeHostError::InvalidArgument(message) if message == "stale presentation render stamp");
                json!({"ok":false,"code":if stale {"staleFrame"} else {error.code()},"retryable":stale,
                    "unpublishedTargetSafeToDiscard":stale,"error":error.to_string()})
            }
            Err(_) => {
                abandon_all();
                json!({"ok":false,"code":"panic","error":"D3D presenter panicked; targets require quarantine"})
            }
        };
        CString::new(value.to_string())
            .expect("JSON escapes NUL")
            .into_raw()
    }
    pub(super) fn capabilities() -> Result<Value, NativeHostError> {
        Ok(
            json!({"ok":true,"abiVersion":1,"available":true,"producerApi":"d3d12Ganesh",
            "consumerBridge":"d3d11GpuCopyToLegacyDxgiHandle","typedPointers":true,"ownerThreadHandles":true,
            "pixelFormat":"bgra8UnormPremultiplied","origin":"topLeft","colorSpace":"srgb",
            "producerGpuCompletion":"checkedQueueFence","consumerGpuCompletion":false,
            "wholeFrameReadbacks":0,"maxPresentersPerThread":2,"maxTargetBytes":16777216,"maxPages":8,
            "contextCredits":rhwp_core::native_host::gpu_d3d::context_credits()}),
        )
    }

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    thread_local! { static OWNERS:RefCell<HashMap<u64,NativeD3dPresenter>> = RefCell::new(HashMap::new()); }
    const MAX_REQUEST_BYTES: usize = 64 * 1024;

    fn invalid_handle() -> NativeHostError {
        NativeHostError::InvalidArgument("unknown or foreign-thread D3D12 presenter".into())
    }
    unsafe fn bytes<'a>(pointer: *const u8, length: usize) -> Result<&'a [u8], NativeHostError> {
        if length > MAX_REQUEST_BYTES || (length != 0 && pointer.is_null()) {
            return Err(NativeHostError::InvalidArgument(
                "invalid D3D12 request/config span".into(),
            ));
        }
        Ok(if length == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(pointer, length) }
        })
    }
    pub(super) unsafe fn create(
        adapter: *const c_void,
        device: *const c_void,
        queue: *const c_void,
        generation: u64,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let data = unsafe { bytes(pointer, length)? };
        let config: NativeD3dPresenterConfig = if data.is_empty() {
            Default::default()
        } else {
            serde_json::from_slice(data).map_err(|_| {
                NativeHostError::InvalidArgument("invalid D3D12 presenter config".into())
            })?
        };
        config.validate()?;
        OWNERS.with(|owners| {
            if owners.borrow().len() >= 2 {
                Err(NativeHostError::LimitExceeded(
                    "D3D12 presenter count quota exceeded".into(),
                ))
            } else {
                Ok(())
            }
        })?;
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| NativeHostError::LimitExceeded("D3D12 handle space exhausted".into()))?;
        let presenter =
            unsafe { NativeD3dPresenter::new(adapter, device, queue, generation, config)? };
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
        target: NativeD3dTarget,
        placements: Vec<NativeD3dPlacement>,
        clear_color: u32,
    }
    pub(super) unsafe fn render(
        id: u64,
        texture: *const c_void,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, NativeHostError> {
        let request: RenderRequest = serde_json::from_slice(unsafe { bytes(pointer, length)? })
            .map_err(|_| NativeHostError::InvalidArgument("invalid D3D12 render request".into()))?;
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
                NativeHostError::InvalidArgument("invalid D3D12 presenter request".into())
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
            .map_err(|_| NativeHostError::InvalidArgument("invalid D3D12 frame check".into()))?;
        if request.snapshots.is_empty() || request.snapshots.len() > 8 {
            return Err(NativeHostError::LimitExceeded(
                "D3D12 frame check requires 1..8 snapshots".into(),
            ));
        }
        let mut source = None;
        for snapshot in request.snapshots {
            let input = crate::presentation::registry().borrow_snapshot(snapshot, request.stamp)?;
            if input.snapshot().profile() != rhwp_core::paint::RenderProfile::Screen
                || source.is_some_and(|source| source != input.source())
            {
                return Err(NativeHostError::InvalidArgument(
                    "D3D frame requires one screen presentation source".into(),
                ));
            }
            source = Some(input.source());
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
