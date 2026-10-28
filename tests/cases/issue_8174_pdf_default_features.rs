//! Independent PDF/colour inputs and visual provenance: pdf/issue8174/features/README.md.
#![cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]

use rhwp::parser::pdf::{parse_pdf_with_options, parse_pdf_with_policy, PdfImportPolicy};
use rhwp::DocumentCore;

fn input(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue8174/features")
            .join(format!("{name}.pdf")),
    )
    .unwrap()
}

fn image(core: &DocumentCore) -> image::RgbaImage {
    let resources = &core.document().bin_data_content;
    assert_eq!(
        resources.len(),
        1,
        "one source image retains page ownership"
    );
    image::load_from_memory(&resources[0].data.load())
        .unwrap()
        .to_rgba8()
}

#[test]
#[cfg(feature = "pdf-icc")]
fn icc_materialization_matches_independent_littlecms_and_survives_saving() {
    let expected = image::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue8174/features/reference/icc-lcms-expected.png"),
    )
    .unwrap()
    .to_rgba8();
    for name in ["icc-rgb", "icc-indexed", "icc-indirect-filter"] {
        let core = DocumentCore::from_bytes(&input(name)).unwrap();
        assert_eq!(image(&core), expected, "{name}: source profile is consumed");
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let saved = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(saved.page_count(), 1);
            assert_eq!(image(&saved), expected, "{name}: saved colour is preserved");
        }
    }
}

#[test]
#[cfg(feature = "pdf-jpeg2000")]
fn indexed_and_cmyk_embedded_alpha_preserve_source_samples_and_roundtrip() {
    for name in [
        "jpx-indexed-alpha",
        "jpx-indexed-premult",
        "jpx-indexed-palette-premult",
        "jpx-indexed-icc-premult",
        "jpx-cmyk-alpha",
        "jpx-cmyk-premult",
    ] {
        let raw = DocumentCore::from_bytes(&input(name)).unwrap();
        let boxed = DocumentCore::from_bytes(&input(&format!("{name}-boxed"))).unwrap();
        assert_eq!(raw.page_count(), 1);
        let pixels = image(&raw);
        assert_eq!(
            image(&boxed),
            pixels,
            "PDF colour space governs both JP2/J2K"
        );
        let alpha: Vec<_> = pixels.rows().next().unwrap().map(|p| p[3]).collect();
        let expected = if name.ends_with("premult") {
            vec![128, 255, 64, 0]
        } else {
            vec![32, 128, 192, 255]
        };
        assert_eq!(alpha, expected, "independent encoder opacity samples");
        if name == "jpx-indexed-alpha" {
            assert_eq!(pixels.get_pixel(1, 0).0, [0, 255, 0, 128]);
        }
        if name == "jpx-indexed-premult" {
            assert_eq!(
                pixels.get_pixel(0, 0).0,
                [0, 255, 0, 128],
                "palette colours, not indices, are unblended"
            );
        }
        if name == "jpx-indexed-palette-premult" {
            assert_eq!(pixels.get_pixel(0, 0).0, [255, 0, 0, 128]);
        }
        if name == "jpx-indexed-icc-premult" && rhwp::pdf_support::pdf_import_support().icc {
            let expected = image::open(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("pdf/issue8174/features/reference/icc-lcms-expected.png"),
            )
            .unwrap()
            .to_rgb8();
            for x in 0..3 {
                assert_eq!(
                    &pixels.get_pixel(x, 0).0[..3],
                    &expected.get_pixel(x, 0).0,
                    "invert source preblend before nonlinear CMM"
                );
            }
        }
        for core in [&raw, &boxed] {
            for bytes in [
                core.export_hwp_native().unwrap(),
                core.export_hwpx_native().unwrap(),
            ] {
                let saved = DocumentCore::from_bytes(&bytes).unwrap();
                assert_eq!(saved.page_count(), 1);
                assert_eq!(
                    image(&saved),
                    pixels,
                    "{name}: alpha and colour survive saving"
                );
            }
        }
    }
}

#[test]
fn bounded_recovery_ignores_payload_headers_and_uses_latest_objects() {
    for name in ["recovery-fake-header", "recovery-incremental"] {
        let result = parse_pdf_with_options(&input(name), &Default::default()).unwrap();
        assert_eq!(result.metadata.source_page_count, 1);
        let core = DocumentCore::from_bytes(&input(name)).unwrap();
        assert_eq!(core.page_count(), 1);
        let colour = if name == "recovery-incremental" {
            "#0000ff"
        } else {
            "#ff0000"
        };
        let svg = core.render_page_svg_native(0).unwrap();
        assert_eq!(
            svg.matches(&format!("fill=\"{colour}\"")).count(),
            1,
            "the source content is retained once"
        );
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let saved = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(saved.page_count(), 1);
            assert_eq!(
                saved
                    .render_page_svg_native(0)
                    .unwrap()
                    .matches(&format!("fill=\"{colour}\""))
                    .count(),
                1
            );
        }
    }
}

#[test]
#[cfg(feature = "pdf-icc")]
fn repeated_profile_references_reuse_the_cmm_and_release_request_ownership() {
    use pdf_oxide::{document::PdfDocument, import_limits::PdfImportLimits};
    let weak;
    {
        let doc = PdfDocument::from_bytes_with_limits(input("icc-rgb"), PdfImportLimits::default())
            .unwrap();
        // Inspect the source profile at the real handle's extraction boundary.
        // Handle::decode already converts to sRGB and clears that profile.
        let reference = pdf_oxide::object::ObjectRef::new(5, 0);
        let object = doc.load_object(reference).unwrap();
        let decode = || {
            pdf_oxide::extractors::extract_image_from_xobject(
                Some(&doc),
                &object,
                Some(reference),
                None,
            )
            .unwrap()
        };
        let first = decode();
        let second = decode();
        let profile = first.icc_profile().unwrap();
        assert!(std::sync::Arc::ptr_eq(
            profile,
            second.icc_profile().unwrap()
        ));
        weak = std::sync::Arc::downgrade(profile);
        let budget = doc.import_budget().unwrap();
        let before = budget.snapshot().codec_allocated_bytes;
        let a = first.to_png_bytes_with_budget(budget).unwrap();
        let compiled = budget.snapshot().codec_allocated_bytes;
        let b = second.to_png_bytes_with_budget(budget).unwrap();
        let repeated = budget.snapshot().codec_allocated_bytes;
        assert_eq!(a, b);
        assert!(
            compiled - before > repeated - compiled,
            "repeated paint must not compile the CMM again"
        );
    }
    assert!(
        weak.upgrade().is_none(),
        "profile/transform cache must not form a reference cycle"
    );
}

#[test]
fn recovery_scan_budget_failure_is_fatal_and_next_request_recovers() {
    let policy = PdfImportPolicy::default()
        .with_limits_json(r#"{"backend":{"codecWorkUnits":1}}"#)
        .unwrap();
    assert_eq!(
        parse_pdf_with_policy(&input("recovery-fake-header"), &Default::default(), policy)
            .err()
            .unwrap()
            .code,
        "PDF_RESOURCE_LIMIT"
    );
    assert!(parse_pdf_with_options(&input("recovery-fake-header"), &Default::default()).is_ok());
}

#[test]
fn recovery_keeps_trailer_metadata_and_refuses_ambiguous_or_truncated_objects() {
    use pdf_oxide::{document::PdfDocument, import_limits::PdfImportLimits};
    let original = String::from_utf8(input("recovery-incremental")).unwrap();
    let metadata = original.replace("/Title", "/Producer");
    let doc =
        PdfDocument::from_bytes_with_limits(metadata.into_bytes(), PdfImportLimits::default())
            .unwrap();
    assert_eq!(
        doc.document_producer().as_deref(),
        Some("998 0 obj << /Type /Catalog >> endobj")
    );
    // Mutate the actual stream length, without depending on its decimal width.
    let at = original.find("/Length ").unwrap() + "/Length ".len();
    let end = at + original[at..].find(' ').unwrap();
    let mut ambiguous = original.clone();
    ambiguous.replace_range(at..end, "999999");
    assert!(PdfDocument::from_bytes_with_limits(
        ambiguous.into_bytes(),
        PdfImportLimits::default()
    )
    .is_err());
    let truncated = original + "trailer\n<< /Root ";
    assert!(PdfDocument::from_bytes_with_limits(
        truncated.into_bytes(),
        PdfImportLimits::default()
    )
    .is_err());
    assert!(parse_pdf_with_options(&input("recovery-incremental"), &Default::default()).is_ok());
}

#[test]
#[cfg(feature = "pdf-icc")]
fn icc_path_work_and_clut_allocations_obey_the_request_budget() {
    for (name, limits) in [
        ("icc-path-rgb", r#"{"backend":{"codecWorkUnits":1}}"#),
        (
            "icc-path-cmyk",
            r#"{"backend":{"codecAllocationBytes":128}}"#,
        ),
    ] {
        let policy = PdfImportPolicy::default().with_limits_json(limits).unwrap();
        assert_eq!(
            parse_pdf_with_policy(&input(name), &Default::default(), policy)
                .err()
                .unwrap()
                .code,
            "PDF_RESOURCE_LIMIT",
            "CMM exhaustion must not turn into fallback colour success"
        );
        assert!(parse_pdf_with_options(&input(name), &Default::default()).is_ok());
    }
}

#[test]
#[cfg(feature = "pdf-icc")]
fn compressed_profile_expansion_keeps_the_request_stream_limit() {
    use std::io::Write;
    let profile = include_bytes!("../../vendor/rhwp-hayro-jpeg2000/assets/ProPhoto-v2-micro.icc");
    let mut padded = profile.to_vec();
    padded.resize(padded.len() + 65_536, 0);
    let length = padded.len() as u32;
    padded[..4].copy_from_slice(&length.to_be_bytes());
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&padded).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(compressed.len() < 1024 && padded.len() > 1024);
    let stream = |fields: &str, data: &[u8]| {
        let mut bytes = format!("<< {fields} /Length {} >>\nstream\n", data.len()).into_bytes();
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(b"\nendstream");
        bytes
    };
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 160 160] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream("", b"q 160 0 0 160 0 0 cm /Im Do Q"),
        stream("/Type /XObject /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace [/ICCBased 6 0 R]", &[128, 64, 32]),
        stream("/N 3 /Filter /FlateDecode", &compressed),
    ];
    let pdf = pdf_objects(&objects);
    let policy = PdfImportPolicy::default()
        .with_limits_json(r#"{"backend":{"streamBytes":1024}}"#)
        .unwrap();
    assert_eq!(
        parse_pdf_with_policy(&pdf, &Default::default(), policy)
            .err()
            .unwrap()
            .code,
        "PDF_RESOURCE_LIMIT"
    );
    assert!(parse_pdf_with_options(&input("icc-rgb"), &Default::default()).is_ok());
}

fn pdf_objects(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut pdf = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![0];
    for (i, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        pdf.extend_from_slice(object);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

// Error/compatibility boundary, not a Lab rendering golden.
#[test]
#[cfg(feature = "pdf-jpeg2000")]
fn preblend_requires_source_palette_components_while_straight_alpha_is_preserved() {
    use pdf_oxide::{
        document::PdfDocument,
        import_limits::PdfImportLimits,
        object::{Object, ObjectRef},
    };
    let doc = PdfDocument::from_bytes_with_limits(
        input("jpx-indexed-palette-premult"),
        PdfImportLimits::default(),
    )
    .unwrap();
    let Object::Stream { data, .. } = doc.load_object(ObjectRef::new(5, 0)).unwrap() else {
        panic!("image stream")
    };
    let stream = |fields: &str, bytes: &[u8]| {
        let mut out = format!("<< {fields} /Length {} >>\nstream\n", bytes.len()).into_bytes();
        out.extend_from_slice(bytes);
        out.extend_from_slice(b"\nendstream");
        out
    };
    for mode in [1, 2] {
        let objects = [
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
            b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 160 160] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
            stream("", b"q 160 0 0 160 0 0 cm /Im Do Q"),
            stream(&format!("/Type /XObject /Subtype /Image /Width 4 /Height 4 /Filter /JPXDecode /ColorSpace [/Indexed [/Lab << /WhitePoint [0.9505 1 1.089] >>] 3 <808080ff8080008080808080>] /SMaskInData {mode}"), &data),
        ];
        let result = parse_pdf_with_options(&pdf_objects(&objects), &Default::default());
        if mode == 1 {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.err().unwrap().code, "PDF_IMAGE_UNSUPPORTED");
        }
    }
}
