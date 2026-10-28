//! #8183 PDF source-image contracts; see the stage 3 independent PDF/Native/WASM evidence.
#![cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]

use rhwp::parser::pdf::{parse_pdf_with_options, parse_pdf_with_policy, PdfImportPolicy};
use rhwp::DocumentCore;

fn input(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue8183")
            .join(format!("{name}.pdf")),
    )
    .unwrap()
}
fn pixels(core: &DocumentCore) -> image::RgbaImage {
    assert_eq!(
        core.document().bin_data_content.len(),
        1,
        "one painted image resource"
    );
    image::load_from_memory(&core.document().bin_data_content[0].data.load())
        .unwrap()
        .to_rgba8()
}
fn selected() -> [(&'static str, bool); 2] {
    let support = rhwp::pdf_support::pdf_import_support();
    [("jbig2", support.jbig2), ("jpx", support.jpeg2000)]
}
#[test]
fn selected_codecs_match_actual_import_and_independent_checker_samples() {
    for (name, enabled) in selected() {
        let result = parse_pdf_with_options(&input(name), &Default::default());
        if !enabled {
            let error = result.err().expect("unselected codec must reject");
            assert!(matches!(
                error.code,
                "PDF_PARSE_FAILED" | "PDF_IMAGE_UNSUPPORTED"
            ));
            continue;
        }
        let result = result.unwrap();
        assert_eq!(result.metadata.source_page_count, 1);
        assert_eq!(result.metadata.pages.len(), 1);
        assert_eq!(result.metadata.pages[0].source_page, 0);
        assert!(!result.metadata.parser_allocation_bounds_proven);
        assert_eq!(
            result.metadata.pdf_import_support,
            rhwp::pdf_support::pdf_import_support()
        );
        let core = DocumentCore::from_bytes(&input(name)).unwrap();
        let image = pixels(&core);
        assert_eq!(image.dimensions(), (16, 8));
        for y in 0..8 {
            for x in 0..16 {
                let v = if (x / 4 + y / 2) % 2 == 0 { 0 } else { 255 };
                assert_eq!(image.get_pixel(x, y).0, [v, v, v, 255]);
            }
        }
    }
}
#[test]
#[cfg(any(feature = "pdf-jbig2", feature = "pdf-jpeg2000"))]
fn selected_codecs_respect_work_limits_and_recover_for_next_request() {
    for (name, enabled) in selected() {
        if !enabled {
            continue;
        }
        let bytes = input(name);
        let used = parse_pdf_with_options(&bytes, &Default::default())
            .unwrap()
            .metadata
            .resource_usage
            .codec_work_units;
        assert!(used > 0);
        for limit in [used - 1, used, used + 1] {
            let mut policy = PdfImportPolicy::default();
            policy.backend.codec_work_units = limit;
            let result = parse_pdf_with_policy(&bytes, &Default::default(), policy);
            if limit < used {
                assert_eq!(result.err().unwrap().code, "PDF_RESOURCE_LIMIT");
            } else {
                assert!(result.is_ok());
            }
        }
        let mut policy = PdfImportPolicy::default();
        policy.backend.codec_allocation_bytes = 8;
        assert_eq!(
            parse_pdf_with_policy(&bytes, &Default::default(), policy)
                .err()
                .unwrap()
                .code,
            "PDF_RESOURCE_LIMIT"
        );
        let corrupt = input(if name == "jbig2" {
            "corrupt-JBIG2Decode"
        } else {
            "corrupt-JPXDecode"
        });
        let error = parse_pdf_with_options(&corrupt, &Default::default())
            .err()
            .unwrap();
        assert_ne!(
            error.code, "PDF_RESOURCE_LIMIT",
            "format error is not budget exhaustion"
        );
        assert!(parse_pdf_with_options(&bytes, &Default::default()).is_ok());
    }
}
#[test]
fn decoded_images_preserve_resources_pixels_and_page_ownership_on_save() {
    for (name, enabled) in [
        ("scan-jpeg-control", true),
        ("scan-jpeg-control-200", true),
        ("scan-jbig2", cfg!(feature = "pdf-jbig2")),
        ("scan-jbig2-200", cfg!(feature = "pdf-jbig2")),
        ("jbig2-generic_template0", cfg!(feature = "pdf-jbig2")),
        ("jbig2-symbol_global", cfg!(feature = "pdf-jbig2")),
        ("scan-jpx", cfg!(feature = "pdf-jpeg2000")),
        ("scan-jpx-200", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-tiles", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-palette", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-indexed", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-alpha", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-premult", cfg!(feature = "pdf-jpeg2000")),
        ("jpx-mask", cfg!(feature = "pdf-jpeg2000")),
        (
            "jbig2-mask",
            cfg!(all(feature = "pdf-jbig2", feature = "pdf-jpeg2000")),
        ),
    ] {
        if !enabled {
            continue;
        }
        let core = DocumentCore::from_bytes(&input(name)).unwrap();
        assert_eq!(core.page_count(), 1, "{name}");
        let original = pixels(&core);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let saved = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(saved.page_count(), 1, "{name}");
            assert_eq!(
                pixels(&saved),
                original,
                "{name}: decoded image survives saving"
            );
            assert!(
                saved.render_page_svg_native(0).unwrap().contains("<image"),
                "{name}: painted resource remains owned"
            );
        }
    }
}
#[test]
#[cfg(feature = "pdf-jpeg2000")]
fn jpx_palette_pdf_indexed_and_alpha_follow_source_samples() {
    for (name, first, second) in [
        ("jpx-palette", [255, 0, 0, 255], [0, 255, 0, 255]),
        ("jpx-indexed", [0, 0, 255, 255], [0, 255, 255, 255]),
        ("jpx-premult", [255, 0, 0, 128], [0, 255, 0, 64]),
    ] {
        let image = pixels(&DocumentCore::from_bytes(&input(name)).unwrap());
        assert_eq!(image.get_pixel(0, 0).0, first, "{name}");
        assert_eq!(image.get_pixel(1, 0).0, second, "{name}");
    }
    let alpha = pixels(&DocumentCore::from_bytes(&input("jpx-alpha")).unwrap());
    assert_eq!(alpha.get_pixel(1, 0).0, [0, 255, 0, 64]);
    assert_eq!(alpha.get_pixel(2, 0).0, [0, 0, 255, 128]);
}

#[test]
#[cfg(any(feature = "pdf-jbig2", feature = "pdf-jpeg2000"))]
fn lossless_scans_preserve_the_complete_independent_source_bitmap() {
    for (name, expected, enabled) in [
        (
            "scan-jbig2-200",
            "scan-source-mono-200.png",
            cfg!(feature = "pdf-jbig2"),
        ),
        (
            "scan-jpx-200",
            "scan-source-gray-200.png",
            cfg!(feature = "pdf-jpeg2000"),
        ),
    ] {
        if !enabled {
            continue;
        }
        let source = image::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/issue8183")
                .join(expected),
        )
        .unwrap()
        .to_luma8();
        let actual = pixels(&DocumentCore::from_bytes(&input(name)).unwrap());
        assert_eq!(
            image::DynamicImage::ImageRgba8(actual).to_luma8(),
            source,
            "{name}: all source rows and pixels survive decoding, including the last TIFF rows"
        );
    }
}

#[test]
#[cfg(feature = "pdf-jpeg2000")]
fn valid_repeated_jpx_paints_exhaust_work_and_next_request_recovers() {
    let error = parse_pdf_with_options(&input("work-limit-jpx"), &Default::default())
        .err()
        .unwrap();
    assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
    assert!(parse_pdf_with_options(&input("jpx"), &Default::default()).is_ok());
}
