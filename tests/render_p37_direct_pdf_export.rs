#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::error::HwpError;
use rhwp::renderer::pdf::DirectPdfExportOptions;
use rhwp::wasm_api::HwpDocument;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/re-03-latin-only-hancom.hwp")
}

fn sample_doc() -> HwpDocument {
    let path = sample_path();
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("load direct PDF fixture")
}

fn assert_complete_pdf(bytes: &[u8]) {
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(0);
    assert!(bytes.starts_with(b"%PDF-"), "missing PDF header");
    assert!(bytes[..end].ends_with(b"%%EOF"), "missing PDF trailer");
}

fn unique_pdf_path() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-render-p37-{}-{nonce}.pdf",
        std::process::id()
    ))
}

#[test]
fn document_core_direct_pdf_exports_page_selection_and_document() {
    let doc = sample_doc();

    let page = doc
        .render_page_pdf_direct_native(0)
        .expect("direct page PDF");
    let mut options = DirectPdfExportOptions::default();
    options.raster_dpi = 96.0;
    options.title = Some("rhwp direct PDF integration".to_string());
    let selected = doc
        .render_pages_pdf_direct_native_with_options(&[0], &options)
        .expect("direct selected-page PDF");
    let document = doc
        .render_document_pdf_direct_native()
        .expect("direct document PDF");

    assert_complete_pdf(&page);
    assert_complete_pdf(&selected);
    assert_complete_pdf(&document);
    assert!(selected
        .windows(b"rhwp direct PDF integration".len())
        .any(|window| window == b"rhwp direct PDF integration"));
}

#[test]
fn document_core_direct_pdf_preserves_selection_errors() {
    let doc = sample_doc();

    let empty = doc
        .render_pages_pdf_direct_native(&[])
        .expect_err("empty page selection must fail");
    assert!(
        matches!(empty, HwpError::RenderError(ref message) if message.contains("at least one page")),
        "unexpected error: {empty:?}"
    );

    let out_of_range = doc
        .render_pages_pdf_direct_native(&[0, 99])
        .expect_err("out-of-range page must fail");
    assert!(
        matches!(out_of_range, HwpError::PageOutOfRange(99)),
        "unexpected error: {out_of_range:?}"
    );
}

#[test]
fn export_pdf_cli_selects_direct_backend_explicitly() {
    let output_path = unique_pdf_path();
    let output = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct"])
        .args(["--raster-dpi", "96", "--output"])
        .arg(&output_path)
        .output()
        .expect("run direct PDF CLI");

    assert!(
        output.status.success(),
        "direct PDF CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("PDF backend: direct"),
        "CLI did not report direct backend"
    );
    let pdf = std::fs::read(&output_path).expect("read direct CLI PDF");
    let _ = std::fs::remove_file(&output_path);
    assert_complete_pdf(&pdf);

    let explicit_print_path = unique_pdf_path();
    let explicit_print = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct", "--profile", "print"])
        .args(["--raster-dpi", "96", "--output"])
        .arg(&explicit_print_path)
        .output()
        .expect("run explicit print direct PDF CLI");
    assert!(explicit_print.status.success());
    let explicit_print_pdf =
        std::fs::read(&explicit_print_path).expect("read explicit print direct PDF");
    let _ = std::fs::remove_file(&explicit_print_path);
    assert_eq!(pdf, explicit_print_pdf);
}

#[test]
fn export_pdf_cli_rejects_backend_specific_option_mismatches() {
    let direct_output_path = unique_pdf_path();
    let direct_with_svg_option = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct", "--fallback-serif", "serif"])
        .arg("--output")
        .arg(&direct_output_path)
        .output()
        .expect("run direct PDF CLI with compatibility option");
    assert_eq!(direct_with_svg_option.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&direct_with_svg_option.stderr)
        .contains("SVG 호환 옵션을 지원하지 않습니다"));
    assert!(!direct_output_path.exists());

    let compatibility_output_path = unique_pdf_path();
    let svg_with_direct_option = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "svg", "--raster-dpi", "96"])
        .arg("--output")
        .arg(&compatibility_output_path)
        .output()
        .expect("run compatibility PDF CLI with direct option");
    assert_eq!(svg_with_direct_option.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&svg_with_direct_option.stderr)
        .contains("direct PDF backend에서만"));
    assert!(!compatibility_output_path.exists());
}
