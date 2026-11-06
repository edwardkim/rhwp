//! Explicit owner-thread GL/FBO ABI. No Skia/context pointers cross the ABI.

use std::ffi::{c_char, c_void, CString};

use rhwp_core::native_host::NativeHostError;
use serde_json::{json, Value};

#[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
use rhwp_core::native_host::gpu_gl::{NativeGlError, NativeGlFailureKind};
#[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
type CallError = NativeGlError;
#[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
type CallError = NativeHostError;

pub type NativeGlGetProcAddress = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_void;
pub type NativeGlIsCurrent = unsafe extern "C" fn(*mut c_void, u64) -> i32;

pub(crate) fn capabilities() -> Value {
    let available = cfg!(all(feature = "native-skia-gl", target_os = "linux"));
    #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
    let limits = json!(rhwp_core::native_host::gpu_gl::NativeGlPresenterConfig::default());
    #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
    let credits = json!(rhwp_core::native_host::gpu_gl::context_credits());
    #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
    let limits = Value::Null;
    #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
    let credits = Value::Null;
    json!({
        "abiVersion":1,"available":available,"hostFboRendering":available,
        "externalTextureExport":false,"crossThreadHandles":false,"tileCache":available,"clearCaches":available,
        "maxPresentersPerThread":4,"limits":limits,"typedFailures":available,
        "recreate":available,"retire":available,"contextCredits":credits,
    })
}

fn response(f: impl FnOnce() -> Result<Value, CallError>) -> *mut c_char {
    let value = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
            {
                let info = error.info();
                json!({"ok":false,"code":info.code,"error":info.message,
                "failureKind":info.kind,"domain":info.domain,"deviceFailure":info.device_failure,
                "requiresRecreate":info.requires_recreate,"retryable":info.retryable})
            }
            #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
            {
                json!({"ok":false,"code":error.code(),"error":error.to_string()})
            }
        }
        Err(_) => {
            #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
            enabled::abandon_all();
            json!({"ok":false,"code":"panic","error":"GL presenter panicked; owner handles abandoned"})
        }
    };
    CString::new(value.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

#[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
fn disabled() -> Result<Value, CallError> {
    Ok(json!({"ok":false,"code":"featureDisabled","error":"native-skia-gl on Linux is required"}))
}

/// Create a presenter while its host context is current on this OS thread.
/// Loader is used only during this call. is_current/user_data remain borrowed
/// until explicit release/abandon (or owner-thread exit); callbacks must not
/// reenter this ABI or unwind. Every GL procedure stays valid for this owner.
///
/// # Safety
/// Callbacks/user_data must satisfy the lifetime/current-context contract above.
/// Nonempty config bytes must remain readable for config_len during this call.
#[no_mangle]
pub unsafe extern "C" fn rhwp_gl_presenter_create(
    context_id: u64,
    device_generation: u64,
    get_proc_address: Option<NativeGlGetProcAddress>,
    is_current: Option<NativeGlIsCurrent>,
    user_data: *mut c_void,
    config: *const u8,
    config_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
        {
            unsafe {
                enabled::create(
                    context_id,
                    device_generation,
                    get_proc_address,
                    is_current,
                    user_data,
                    config,
                    config_len,
                )
            }
        }
        #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
        {
            let _ = (
                context_id,
                device_generation,
                get_proc_address,
                is_current,
                user_data,
                config,
                config_len,
            );
            disabled()
        }
    })
}

/// Owner-thread info/render/clearCaches/release/abandon JSON operations. Render,
/// cache clear, and release
/// check is_current immediately before using GL. Abandon/Drop never call it.
///
/// # Safety
/// Nonempty request bytes must remain readable for request_len during this call.
/// The render target FBO must be valid and accurately described while borrowed.
/// The host owns context/current/swap and restores its own GL state afterwards.
#[no_mangle]
pub unsafe extern "C" fn rhwp_gl_presenter_request(
    presenter: u64,
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
        {
            unsafe { enabled::request(presenter, request, request_len) }
        }
        #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
        {
            let _ = (presenter, request, request_len);
            disabled()
        }
    })
}

/// Recreate a TLS owner on an actual new current context and advanced device
/// generation. Old callback/user_data remain borrowed until success/abandon.
///
/// # Safety
/// The new context identity and callback/procedure lifetimes must be accurate.
/// Nonempty config bytes must remain readable for the call.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn rhwp_gl_presenter_recreate(
    presenter: u64,
    context_id: u64,
    device_generation: u64,
    get_proc_address: Option<NativeGlGetProcAddress>,
    is_current: Option<NativeGlIsCurrent>,
    user_data: *mut c_void,
    config: *const u8,
    config_len: usize,
) -> *mut c_char {
    response(|| {
        #[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
        {
            unsafe {
                enabled::recreate(
                    presenter,
                    context_id,
                    device_generation,
                    get_proc_address,
                    is_current,
                    user_data,
                    config,
                    config_len,
                )
            }
        }
        #[cfg(not(all(feature = "native-skia-gl", target_os = "linux")))]
        {
            let _ = (
                presenter,
                context_id,
                device_generation,
                get_proc_address,
                is_current,
                user_data,
                config,
                config_len,
            );
            disabled()
        }
    })
}

#[cfg(all(feature = "native-skia-gl", target_os = "linux"))]
mod enabled {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    use rhwp_core::native_host::gpu_gl::{
        NativeGlPlacement, NativeGlPresenter, NativeGlPresenterConfig, NativeGlTarget,
    };
    use rhwp_core::native_host::presentation::NativePresentationStamp;
    use serde::Deserialize;

    use super::*;

    const MAX_REQUEST_BYTES: usize = 1024 * 1024;
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    thread_local! { static OWNERS: RefCell<HashMap<u64, Owner>> = RefCell::new(HashMap::new()); }

    struct Owner {
        presenter: NativeGlPresenter,
        context_id: u64,
        is_current: NativeGlIsCurrent,
        user_data: *mut c_void,
    }

    impl Owner {
        fn current(&self) -> Result<(), CallError> {
            // SAFETY: host maintains callback/user_data through release/abandon.
            if unsafe { (self.is_current)(self.user_data, self.context_id) } != 1 {
                return Err(NativeGlError::new(
                    NativeGlFailureKind::ContextNotCurrent,
                    "matching host GL context is not current",
                ));
            }
            Ok(())
        }
    }

    fn invalid_handle() -> CallError {
        NativeGlError::new(
            NativeGlFailureKind::WrongOwner,
            "unknown or foreign-thread GL presenter",
        )
    }

    unsafe fn bytes<'a>(pointer: *const u8, length: usize) -> Result<&'a [u8], NativeHostError> {
        if length > MAX_REQUEST_BYTES || (length != 0 && pointer.is_null()) {
            return Err(NativeHostError::InvalidArgument(
                "invalid GL request/config span".into(),
            ));
        }
        Ok(if length == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(pointer, length) }
        })
    }

    pub(super) unsafe fn create(
        context_id: u64,
        device_generation: u64,
        loader: Option<NativeGlGetProcAddress>,
        current: Option<NativeGlIsCurrent>,
        user_data: *mut c_void,
        config: *const u8,
        config_len: usize,
    ) -> Result<Value, CallError> {
        let loader = loader.ok_or_else(|| {
            NativeHostError::InvalidArgument("GL loader callback is required".into())
        })?;
        let current = current.ok_or_else(|| {
            NativeHostError::InvalidArgument("GL current-context callback is required".into())
        })?;
        let config = unsafe { bytes(config, config_len)? };
        let config: NativeGlPresenterConfig = if config.is_empty() {
            Default::default()
        } else {
            serde_json::from_slice(config).map_err(|_| {
                NativeHostError::InvalidArgument("invalid GL presenter config".into())
            })?
        };
        config.validate()?;
        OWNERS.with(|owners| {
            if owners.borrow().len() >= 4 {
                Err(NativeHostError::LimitExceeded(
                    "GL presenter count quota exceeded".into(),
                ))
            } else {
                Ok(())
            }
        })?;
        if context_id == 0
            || device_generation == 0
            || unsafe { current(user_data, context_id) } != 1
        {
            return Err(NativeGlError::new(
                NativeGlFailureKind::ContextNotCurrent,
                "matching host GL context is not current",
            ));
        }
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| NativeHostError::LimitExceeded("GL handle space exhausted".into()))?;
        let presenter = unsafe {
            NativeGlPresenter::new(context_id, device_generation, config, |name| {
                loader(user_data, name.as_ptr())
            })?
        };
        let info = presenter.info()?;
        OWNERS.with(|owners| {
            owners.borrow_mut().insert(
                id,
                Owner {
                    presenter,
                    context_id,
                    is_current: current,
                    user_data,
                },
            );
        });
        Ok(json!({"ok":true,"presenter":id,"info":info}))
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
        Render {
            stamp: NativePresentationStamp,
            target: NativeGlTarget,
            placements: Vec<NativeGlPlacement>,
            clear_color: u32,
        },
        Release {
            context_id: u64,
            device_generation: u64,
        },
        ClearCaches {
            context_id: u64,
            device_generation: u64,
        },
        Retire {
            context_id: u64,
            device_generation: u64,
        },
        Abandon {},
    }

    pub(super) unsafe fn request(
        id: u64,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, CallError> {
        let request: Request =
            serde_json::from_slice(unsafe { bytes(pointer, length)? }).map_err(|_| {
                NativeHostError::InvalidArgument("invalid GL presenter request JSON".into())
            })?;
        OWNERS.with(|owners| {
            let mut owners = owners.borrow_mut();
            let owner = owners.get_mut(&id).ok_or_else(invalid_handle)?;
            match request {
                Request::Info {} => Ok(json!({"ok":true,"info":owner.presenter.info()?})),
                Request::Render {
                    stamp,
                    target,
                    placements,
                    clear_color,
                } => {
                    owner
                        .presenter
                        .check_context(target.context_id, target.device_generation)?;
                    owner.current()?;
                    let frame = unsafe {
                        owner.presenter.render(
                            crate::presentation::registry(),
                            stamp,
                            target,
                            &placements,
                            clear_color,
                        )?
                    };
                    Ok(json!({"ok":true,"frame":frame,"info":owner.presenter.info()?}))
                }
                Request::ClearCaches {
                    context_id,
                    device_generation,
                } => {
                    owner
                        .presenter
                        .check_context(context_id, device_generation)?;
                    owner.current()?;
                    unsafe {
                        owner
                            .presenter
                            .clear_caches(context_id, device_generation)?;
                    }
                    Ok(json!({"ok":true,"info":owner.presenter.info()?}))
                }
                Request::Retire {
                    context_id,
                    device_generation,
                } => {
                    owner
                        .presenter
                        .check_context(context_id, device_generation)?;
                    owner.current()?;
                    unsafe {
                        owner.presenter.release(context_id, device_generation)?;
                    }
                    Ok(json!({"ok":true,"info":owner.presenter.info()?}))
                }
                Request::Release {
                    context_id,
                    device_generation,
                } => {
                    owner
                        .presenter
                        .check_context(context_id, device_generation)?;
                    owner.current()?;
                    unsafe {
                        owner.presenter.release(context_id, device_generation)?;
                    }
                    owners.remove(&id);
                    Ok(json!({"ok":true}))
                }
                Request::Abandon {} => {
                    owner.presenter.abandon()?;
                    owners.remove(&id);
                    Ok(json!({"ok":true}))
                }
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) unsafe fn recreate(
        id: u64,
        context_id: u64,
        generation: u64,
        loader: Option<NativeGlGetProcAddress>,
        current: Option<NativeGlIsCurrent>,
        user_data: *mut c_void,
        pointer: *const u8,
        length: usize,
    ) -> Result<Value, CallError> {
        let loader = loader.ok_or_else(|| {
            NativeHostError::InvalidArgument("GL loader callback is required".into())
        })?;
        let current = current.ok_or_else(|| {
            NativeHostError::InvalidArgument("GL current-context callback is required".into())
        })?;
        let config_bytes = unsafe { bytes(pointer, length)? };
        OWNERS.with(|owners| {
            let mut owners = owners.borrow_mut();
            let owner = owners.get_mut(&id).ok_or_else(invalid_handle)?;
            let config: NativeGlPresenterConfig = if config_bytes.is_empty() {
                owner.presenter.info()?.config
            } else {
                serde_json::from_slice(config_bytes).map_err(|_| {
                    NativeHostError::InvalidArgument("invalid GL presenter config".into())
                })?
            };
            config.validate()?;
            if unsafe { current(user_data, context_id) } != 1 {
                return Err(NativeGlError::new(
                    NativeGlFailureKind::ContextNotCurrent,
                    "new matching host GL context is not current",
                ));
            }
            unsafe {
                owner
                    .presenter
                    .recreate(context_id, generation, config, |name| {
                        loader(user_data, name.as_ptr())
                    })?;
            }
            owner.context_id = context_id;
            owner.is_current = current;
            owner.user_data = user_data;
            Ok(json!({"ok":true,"presenter":id,"info":owner.presenter.info()?}))
        })
    }

    pub(super) fn abandon_all() {
        let _ = OWNERS.try_with(|owners| {
            if let Ok(mut owners) = owners.try_borrow_mut() {
                for owner in owners.values_mut() {
                    let _ = owner.presenter.abandon();
                }
                owners.clear();
            }
        });
    }
}
