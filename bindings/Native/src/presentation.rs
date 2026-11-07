//! Process-wide CPU presentation handles. These are the explicit cross-thread
//! exception to the legacy session/worker/buffer owner-thread contract.

use std::os::raw::c_char;
use std::sync::OnceLock;

use rhwp_core::native_host::presentation::{
    NativePagePlacement, NativePresentationConfig, NativePresentationId,
    NativePresentationRegistry, NativePresentationStamp, NativePresentationViewport,
};
use rhwp_core::native_host::runtime::NativeRuntimeRegion;
use rhwp_core::native_host::NativeHostError;
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
static REGISTRY: OnceLock<NativePresentationRegistry> = OnceLock::new();

pub(crate) fn registry() -> &'static NativePresentationRegistry {
    REGISTRY.get_or_init(|| {
        NativePresentationRegistry::new(NativePresentationConfig::default())
            .expect("default presentation limits are valid")
    })
}

pub(crate) fn capabilities() -> Value {
    let config = NativePresentationConfig::default();
    json!({
        "abiVersion": 1,
        "workerRenderSnapshots": true,
        "crossThreadHandles": true,
        "fullStampChecks": true,
        "cpuFrames": cfg!(feature = "native-skia"),
        "cpuBatchFrames": cfg!(feature = "native-skia"),
        "maxCpuBatchPages":32,"maxCpuBatchDimension":4096,
        "gpuFrames": false,
        "maxSources": config.max_sources,
        "maxSnapshots": config.max_snapshots,
        "maxSnapshotEstimatedBytes": config.max_snapshot_bytes,
        "maxFrames": config.max_frames,
        "maxFrameBytes": config.max_frame_bytes,
        "maxLeases": config.max_leases,
        "maxInFlightReplays": config.max_in_flight_replays,
    })
}

#[derive(Deserialize)]
#[serde(
    tag = "op",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    SnapshotInfo {
        snapshot: NativePresentationId,
    },
    SnapshotRelease {
        snapshot: NativePresentationId,
    },
    SnapshotCheck {
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
    },
    RenderCpu {
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
        region: NativeRuntimeRegion,
        scale: f64,
    },
    RenderCpuBatch {
        stamp: NativePresentationStamp,
        viewport: NativePresentationViewport,
        placements: Vec<NativePagePlacement>,
        clear_color: u32,
    },
    FrameInfo {
        frame: NativePresentationId,
    },
    FrameAcquire {
        frame: NativePresentationId,
        stamp: NativePresentationStamp,
    },
    FrameRelease {
        frame: NativePresentationId,
    },
    LeaseInfo {
        lease: NativePresentationId,
    },
    LeaseRelease {
        lease: NativePresentationId,
    },
    Stats {},
}

/// Cross-thread immutable snapshot/CPU frame operations. Rendering is explicit,
/// synchronous work for a caller-owned render thread, never an implicit action
/// in frame acquisition or lease-data callbacks.
///
/// # Safety
/// A nonempty request must point to request_len readable bytes for this call.
/// Free the returned JSON with rhwp_string_free on any thread. Keep the library
/// loaded until all calls, worker threads, handles, and consumer leases end.
#[no_mangle]
pub unsafe extern "C" fn rhwp_presentation_request(
    request: *const u8,
    request_len: usize,
) -> *mut c_char {
    crate::session::response(|| {
        if request_len == 0 || request_len > MAX_REQUEST_BYTES || request.is_null() {
            return Err(NativeHostError::InvalidArgument(
                "invalid presentation request span".into(),
            ));
        }
        let bytes = unsafe { std::slice::from_raw_parts(request, request_len) };
        let request: Request = serde_json::from_slice(bytes).map_err(|_| {
            NativeHostError::InvalidArgument("invalid presentation request JSON".into())
        })?;
        let registry = registry();
        Ok(match request {
            Request::SnapshotInfo { snapshot } => {
                json!({"ok":true,"info":registry.snapshot_info(snapshot)?})
            }
            Request::SnapshotRelease { snapshot } => {
                registry.release_snapshot(snapshot)?;
                json!({"ok":true})
            }
            Request::SnapshotCheck { snapshot, stamp } => {
                json!({"ok":true,"current":registry.snapshot_check(snapshot, stamp)?})
            }
            Request::RenderCpu {
                snapshot,
                stamp,
                region,
                scale,
            } => {
                let info = registry.render_cpu(snapshot, stamp, region.into(), scale)?;
                json!({"ok":true,"frame":info.frame,"info":info})
            }
            Request::RenderCpuBatch {
                stamp,
                viewport,
                placements,
                clear_color,
            } => {
                let info = registry.render_cpu_batch(stamp, viewport, &placements, clear_color)?;
                json!({"ok":true,"frame":info.frame,"info":info})
            }
            Request::FrameInfo { frame } => json!({"ok":true,"info":registry.frame_info(frame)?}),
            Request::FrameAcquire { frame, stamp } => {
                let info = registry.acquire_frame(frame, stamp)?;
                json!({"ok":true,"lease":info.lease,"info":info})
            }
            Request::FrameRelease { frame } => {
                registry.release_frame(frame)?;
                json!({"ok":true})
            }
            Request::LeaseInfo { lease } => json!({"ok":true,"info":registry.lease_info(lease)?}),
            Request::LeaseRelease { lease } => {
                registry.release_lease(lease)?;
                json!({"ok":true})
            }
            Request::Stats {} => json!({"ok":true,"stats":registry.stats()?}),
        })
    })
}

/// Return immutable CPU pixels held by one live consumer lease. This does not
/// acquire a lease or wait for rendering. Source close/staleness does not revoke
/// an existing lease's memory. Invalid/released handles return NULL and length 0.
///
/// # Safety
/// byte_length must be a valid writable size_t pointer. The caller must not
/// release the lease concurrently with pointer use, mutate the returned bytes,
/// or unload the library while a lease/call/worker can still use its code/data.
#[no_mangle]
pub unsafe extern "C" fn rhwp_presentation_lease_data(
    lease: u64,
    byte_length: *mut usize,
) -> *const u8 {
    if byte_length.is_null() {
        return std::ptr::null();
    }
    unsafe { *byte_length = 0 };
    let result =
        std::panic::catch_unwind(|| registry().lease_data(NativePresentationId::from_raw(lease)));
    match result {
        Ok(Ok((data, len))) => {
            unsafe { *byte_length = len };
            data
        }
        _ => std::ptr::null(),
    }
}
