//! SOF frame sampling and the progressive coefficient/IDCT grid share a contract.
#![cfg(feature = "pdf-import")]
use pdf_oxide::{
    error::Error,
    extractors::images::{ColorSpace, ImageData, PdfImage},
    import_limits::{ImportBudget, PdfImportLimits},
};
use std::path::PathBuf;

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/issue8174_jpeg_compat")
            .join(name),
    )
    .unwrap()
}
fn image(name: &str, width: u32, height: u32) -> PdfImage {
    PdfImage::new(
        width,
        height,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Jpeg(fixture(name)),
    )
}
#[test]
fn all_sampling_planes_decode_and_baseline_progressive_pixels_agree() {
    let rows: serde_json::Value = serde_json::from_slice(&fixture("manifest.json")).unwrap();
    let rows = rows["inputs"].as_array().unwrap();
    assert_eq!(rows.len(), 52, "every original audit case is required");
    let mut images = std::collections::BTreeMap::new();
    for row in rows {
        let name = row["name"].as_str().unwrap();
        let width = row["width"].as_u64().unwrap() as u32;
        let height = row["height"].as_u64().unwrap() as u32;
        let png = image(name, width, height)
            .to_png_bytes_with_budget(&ImportBudget::new(PdfImportLimits::default()).unwrap())
            .unwrap();
        let pixels = image::load_from_memory(&png).unwrap().to_rgb8();
        assert_eq!(
            pixels.dimensions(),
            (width, height),
            "SOF dimensions: {name}"
        );
        if width == 1 && height == 1 {
            assert_eq!(
                pixels.as_raw(),
                &[0, 0, 0],
                "independent djpeg black control: {name}"
            );
        }
        images.insert(name.to_owned(), pixels);
    }
    let mut pairs = 0;
    for (name, pixels) in &images {
        if name.starts_with("progressive-") {
            // Same quantized source coefficients, a different scan encoding.
            assert_eq!(
                pixels,
                &images[&name.replacen("progressive-", "baseline-", 1)],
                "scan encoding must preserve pixels: {name}"
            );
            pairs += 1;
        }
    }
    assert_eq!(pairs, 26);
}
#[test]
fn corrected_sampling_keeps_exact_request_limits_sticky_failures_and_recovery() {
    for (width, height) in [(9, 1), (17, 17), (65, 33)] {
        let name = format!("progressive-{width}x{height}-mixed-Y1x2-Cb2x2-Cr2x2.jpg");
        let source = image(&name, width, height);
        let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let expected = source.to_png_bytes_with_budget(&normal).unwrap();
        let exact = ImportBudget::new(PdfImportLimits {
            decoded_bytes: normal.decoded_bytes(),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(source.to_png_bytes_with_budget(&exact).unwrap(), expected);
        let low = ImportBudget::new(PdfImportLimits {
            decoded_bytes: normal.decoded_bytes() - 1,
            ..Default::default()
        })
        .unwrap();
        assert!(matches!(
            source.to_png_bytes_with_budget(&low),
            Err(Error::ImportResourceLimit(_))
        ));
        assert!(low.check().is_err());
        assert!(matches!(
            source.to_png_bytes_with_budget(&low),
            Err(Error::ImportResourceLimit(_))
        ));
        let mut malformed = fixture(&name);
        let sof = malformed
            .windows(2)
            .position(|v| v == [0xff, 0xc2])
            .unwrap();
        malformed[sof + 11] = 0; // Invalid zero sampling factor remains an error.
        let invalid = PdfImage::new(
            width,
            height,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Jpeg(malformed),
        );
        assert!(invalid
            .to_png_bytes_with_budget(&ImportBudget::new(PdfImportLimits::default()).unwrap())
            .is_err());
        assert_eq!(
            source
                .to_png_bytes_with_budget(&ImportBudget::new(PdfImportLimits::default()).unwrap())
                .unwrap(),
            expected
        );
    }
}
#[test]
fn mixed_progressive_pdf_pixels_and_soft_mask_survive_both_document_saves() {
    for alpha in [false, true] {
        let filename = format!(
            "progressive-17x17-mixed-Y1x2-Cb2x2-Cr2x2{}.pdf",
            if alpha { "-alpha" } else { "" }
        );
        let bytes = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("pdf/issue8174/jpeg-compat")
                .join(filename),
        )
        .unwrap();
        let mut core = rhwp::DocumentCore::from_pdf_bytes(&bytes, &Default::default()).unwrap();
        assert_eq!(core.page_count(), 1);
        let source = core.get_bin_data(0).unwrap();
        let expected = image::load_from_memory(&source).unwrap().to_rgba8();
        assert_eq!(expected.dimensions(), (17, 17));
        assert!(expected
            .pixels()
            .all(|p| p.0[3] == if alpha { 128 } else { 255 }));
        for saved in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = rhwp::DocumentCore::from_bytes(&saved).unwrap();
            assert_eq!(reopened.page_count(), 1);
            assert_eq!(
                image::load_from_memory(&reopened.get_bin_data(0).unwrap())
                    .unwrap()
                    .to_rgba8(),
                expected
            );
            assert_eq!(
                reopened
                    .render_page_svg_native(0)
                    .unwrap()
                    .matches("<image ")
                    .count(),
                1
            );
        }
    }
}
