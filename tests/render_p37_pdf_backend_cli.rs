#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/re-03-latin-only-hancom.hwp")
}

fn unique_pdf_path(label: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-render-p37-{label}-{}-{nonce}.pdf",
        std::process::id()
    ))
}

fn run_compatibility_export(output_path: &Path, explicit_backend: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rhwp"));
    command.arg("export-pdf").arg(sample_path());
    if explicit_backend {
        command.args(["--backend", "svg"]);
    }
    command.arg("--output").arg(output_path);
    command.output().expect("run compatibility PDF CLI")
}

#[test]
fn explicit_svg_backend_preserves_default_pdf_bytes_and_stdout() {
    let output_path = unique_pdf_path("compatibility");
    let default_output = run_compatibility_export(&output_path, false);
    assert!(default_output.status.success());
    let default_pdf = std::fs::read(&output_path).expect("read default PDF");

    let explicit_output = run_compatibility_export(&output_path, true);
    assert!(explicit_output.status.success());
    let explicit_pdf = std::fs::read(&output_path).expect("read explicit SVG PDF");
    let _ = std::fs::remove_file(&output_path);

    assert_eq!(default_output.stdout, explicit_output.stdout);
    assert_eq!(default_output.stderr, explicit_output.stderr);
    assert_eq!(default_pdf, explicit_pdf);
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn direct_backend_reports_missing_native_skia_feature() {
    let output_path = unique_pdf_path("missing-feature");
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct", "--output"])
        .arg(&output_path)
        .output()
        .expect("run direct PDF CLI without native-skia");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("direct PDF backend requires a build with the native-skia feature"));
    assert!(!output_path.exists());
}
