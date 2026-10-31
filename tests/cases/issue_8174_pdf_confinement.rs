//! Native process boundary contracts; fixtures contain no real credentials.
#![cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
use std::process::Command;

#[test]
fn child_environment_discards_explicit_synthetic_credentials_and_loader_hooks() {
    let mut command = Command::new("unused");
    command.env("SYNTHETIC_SERVICE_TOKEN", "fixture-only");
    command.env("LD_PRELOAD", "/fixture-only.so");
    rhwp::resource_supervisor::configure_pdf_worker_environment(&mut command);
    assert!(!command.get_envs().any(|(name, value)| {
        value.is_some() && (name == "SYNTHETIC_SERVICE_TOKEN" || name == "LD_PRELOAD")
    }));
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[test]
fn invalid_font_scope_fails_closed_preserves_output_and_next_request_recovers() {
    use std::path::PathBuf;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_root = root.join("output/pr-review/issue8174/confinement-contracts");
    std::fs::create_dir_all(&output_root).unwrap();
    let dir = output_root.join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("input.pdf");
    let target = dir.join("keep.hwpx");
    std::fs::write(
        &input,
        include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf"),
    )
    .unwrap();
    std::fs::write(&target, b"KEEP").unwrap();
    let bin =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let run = |invalid: bool| {
        let mut command = Command::new(&bin);
        command
            .arg("import-pdf")
            .arg(&input)
            .arg(&target)
            .arg("--json");
        if invalid {
            command.env("RHWP_FONT_PATH", "/");
        }
        command.output().unwrap()
    };
    let rejected = run(true);
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
    assert_eq!(std::fs::read(&target).unwrap(), b"KEEP");
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("PDF_PROCESS_SANDBOX_UNAVAILABLE"));
    let healthy = run(false);
    // Unsupported kernels must report unavailability, not silently claim a
    // sandbox. Supported CI/validation hosts additionally exercise success.
    if !healthy.status.success() {
        assert!(
            String::from_utf8_lossy(&healthy.stderr).contains("requires Landlock ABI 3"),
            "{}",
            String::from_utf8_lossy(&healthy.stderr)
        );
        assert!(healthy.stdout.is_empty());
        assert_eq!(std::fs::read(&target).unwrap(), b"KEEP");
    } else {
        let envelope: serde_json::Value = serde_json::from_slice(&healthy.stdout).unwrap();
        assert_eq!(envelope["execution"]["filesystemNetworkSandbox"], true);
        let saved = std::fs::read(&target).unwrap();
        assert!(saved.starts_with(b"PK\x03\x04"));
        assert!(rhwp::DocumentCore::from_bytes(&saved).is_ok());
    }
    std::fs::remove_dir_all(dir).unwrap();
}
