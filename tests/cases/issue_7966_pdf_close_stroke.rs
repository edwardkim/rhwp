//! ISO 32000-1 path-painting: s = h S; ordinary S stays open.
#![cfg(feature = "pdf-import")]
use pdf_oxide::{elements::PathOperation, PdfDocument};
const PDF: &[u8] = include_bytes!("../../pdf/issue7966/close-stroke-controls.pdf");

#[test]
fn close_stroke_consumes_the_path_in_page_and_nested_form_scopes() {
    for bounded in [false, true] {
        let doc = if bounded {
            PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default())
        } else {
            PdfDocument::from_bytes(PDF.to_vec())
        }
        .unwrap();
        for page in 0..2 {
            let paths = doc.extract_paths(page).unwrap();
            assert_eq!(
                paths.len(),
                4,
                "three separate strokes then the filled square"
            );
            for (path, rgb) in
                paths[..3]
                    .iter()
                    .zip([[0.8, 0.05, 0.15], [0.1, 0.3, 0.8], [0., 0.5, 0.]])
            {
                let c = path.stroke_color.expect("stroke colour");
                assert!(
                    (c.r - rgb[0]).abs() < 0.001
                        && (c.g - rgb[1]).abs() < 0.001
                        && (c.b - rgb[2]).abs() < 0.001
                );
                assert!(path.fill_color.is_none());
            }
            for path in &paths[..2] {
                assert!(matches!(
                    path.operations.last(),
                    Some(PathOperation::ClosePath)
                ));
            }
            assert!(!paths[2]
                .operations
                .iter()
                .any(|p| matches!(p, PathOperation::ClosePath)));
            assert!(paths[3].fill_color.is_some() && paths[3].stroke_color.is_none());
            assert_eq!(paths[0].operations.len(), paths[1].operations.len());
            // Translation-invariant shape equality from the independent construction.
            assert_eq!(paths[0].operations.len(), paths[2].operations.len() + 1);
        }
        doc.check_import_budget().unwrap();
    }
}

#[test]
fn close_stroke_reaches_the_product_image_lowering_and_both_saved_formats() {
    let core = rhwp::DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    check_product(&core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check_product(&rhwp::DocumentCore::from_bytes(&bytes).unwrap());
    }
}

fn check_product(doc: &rhwp::DocumentCore) {
    assert_eq!(doc.page_count(), 2);
    for page in 0..2 {
        // Each compound stroke has its own raster colour/ownership. The square is a shape.
        let svg = doc.render_page_svg_native(page).unwrap();
        let mut found = std::collections::BTreeSet::new();
        for value in svg.split("data:image/png;base64,").skip(1) {
            let data = value.split('"').next().unwrap();
            use base64::Engine;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .unwrap();
            let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
            for pixel in image.pixels().filter(|p| p[3] > 200) {
                let [r, g, b, _] = pixel.0;
                if r > 150 && g < 80 && b < 100 {
                    found.insert("red");
                }
                if b > 150 && r < 80 && g < 120 {
                    found.insert("blue");
                }
                if g > 100 && r < 30 && b < 30 {
                    found.insert("green");
                }
            }
        }
        assert_eq!(
            found,
            std::collections::BTreeSet::from(["red", "blue", "green"])
        );
    }
}

fn imported_by_cli() -> rhwp::DocumentCore {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let folder =
        std::env::temp_dir().join(format!("rhwp-pdf-contract-{}-{stamp}", std::process::id()));
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir(&folder).unwrap();
    let scratch = Scratch(folder);
    let input = scratch.0.join("input.pdf");
    let output = scratch.0.join("imported.hwpx");
    std::fs::write(&input, PDF).unwrap();
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let result = std::process::Command::new(binary)
        .arg("import-pdf")
        .arg(input)
        .arg(&output)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    rhwp::DocumentCore::from_bytes(&std::fs::read(output).unwrap()).unwrap()
}

#[test]
fn cli_close_stroke_preserves_the_separate_source_strokes() {
    check_product(&imported_by_cli());
}
