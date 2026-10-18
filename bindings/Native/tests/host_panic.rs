//! Native ABI panic recovery must retain worker shutdown handles.
#[path = "../src/session.rs"]
mod native_session;

fn take_native_response(pointer: *mut std::os::raw::c_char) -> serde_json::Value {
    assert!(!pointer.is_null());
    let value = unsafe { std::ffi::CString::from_raw(pointer) };
    serde_json::from_slice(value.as_bytes()).expect("native response JSON")
}

fn host_request(value: serde_json::Value) -> serde_json::Value {
    let request = serde_json::to_vec(&value).unwrap();
    take_native_response(unsafe {
        native_session::rhwp_native_host_request(
            request.as_ptr(),
            request.len(),
            std::ptr::null(),
            0,
        )
    })
}

#[test]
fn panic_recovery_preserves_worker_shutdown_handles() {
    let created = host_request(serde_json::json!({"op":"workerCreate"}));
    assert_eq!(created["ok"], true);
    let worker = created["worker"].as_u64().unwrap();
    let opened = take_native_response(unsafe {
        native_session::rhwp_session_open(std::ptr::null(), 0, std::ptr::null(), 0)
    });
    assert_eq!(opened["ok"], true);
    let session = opened["handle"].as_u64().unwrap();

    let recovered = take_native_response(native_session::response(|| {
        panic!("simulate a synchronous document operation panic")
    }));
    assert_eq!(recovered["code"], "panic");
    assert_eq!(
        take_native_response(native_session::rhwp_session_close(session))["ok"],
        false,
        "partially mutated document sessions must be invalidated"
    );

    let submitted = host_request(serde_json::json!({
        "op":"workerSubmit", "worker":worker, "command":{"op":"new"}
    }));
    assert_eq!(submitted["code"], "closed");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let status = host_request(serde_json::json!({"op":"workerStatus", "worker":worker}));
        assert_eq!(status["ok"], true, "shutdown handle must survive recovery");
        if status["isFinished"] == true {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "worker did not finish"
        );
        std::thread::yield_now();
    }
    let released = host_request(serde_json::json!({"op":"workerRelease", "worker":worker}));
    assert_eq!(released["ok"], true);
}
