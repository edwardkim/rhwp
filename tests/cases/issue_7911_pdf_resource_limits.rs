//! Synthetic resource contracts, not PDF/Hancom layout baselines.
#![cfg(feature = "pdf-import")]

#[test]
fn product_collection_budget_is_shared_by_operand_arrays_not_page_count() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    // A valid TJ array contains a visible string and many zero adjustments.
    // Its array entries are real parser work even though it emits one span.
    // No page geometry or implementation-produced layout baseline is pinned.
    let count = 110_000;
    let mut content = b"BT /F1 12 Tf 20 150 Td [(VISIBLE) ".to_vec();
    content.extend(b"0 ".repeat(count));
    content.extend(b"] TJ ET");
    let source = object_file(&objects(&content, "", ""), "");
    let raw =
        PdfDocument::from_bytes_with_limits(source.clone(), PdfImportLimits::default()).unwrap();
    assert!(matches!(
        raw.extract_text(0),
        Err(Error::ImportResourceLimit("collection entries"))
    ));
    let imported = parse_pdf_with_policy(&source, &Default::default(), PdfImportPolicy::default())
        .unwrap_or_else(|e| panic!("shared collection budget: {e:?}"));
    let used = imported.metadata.resource_usage.collection_entries;
    assert!(used > count);
    assert_eq!(imported.metadata.resource_policy.backend.entries, 400_000);
    assert_eq!(PdfImportLimits::default().entries, 100_000);
    // Text may belong to a body paragraph or a positional TextBox paragraph;
    // collection accounting must preserve it without pinning that placement.
    fn paragraph_text(value: &serde_json::Value, output: &mut String) {
        match value {
            serde_json::Value::Object(map) => {
                if let Some(text) = map.get("text").and_then(serde_json::Value::as_str) {
                    output.push_str(text);
                }
                for child in map.values() {
                    paragraph_text(child, output);
                }
            }
            serde_json::Value::Array(array) => {
                for child in array {
                    paragraph_text(child, output);
                }
            }
            _ => {}
        }
    }
    let mut text = String::new();
    for section in &imported.document.sections {
        for paragraph in &section.paragraphs {
            paragraph_text(&serde_json::to_value(paragraph).unwrap(), &mut text);
        }
    }
    assert_eq!(text, "VISIBLE");
    for cap in [used - 1, used] {
        let mut policy = PdfImportPolicy::default();
        policy.backend.entries = cap;
        let result = parse_pdf_with_policy(&source, &Default::default(), policy);
        if cap == used {
            assert!(result.is_ok());
        } else {
            let failure = result
                .err()
                .expect("lower host cap must reject")
                .diagnostics
                .unwrap()
                .resource_usage
                .unwrap()
                .failure
                .unwrap();
            assert_eq!(failure.reason, "collection entries");
            assert_eq!(failure.limit, Some(cap as u64));
        }
    }
}

#[test]
fn structure_caps_preserve_entry_consumption_and_cannot_be_raised() {
    let request = Arc::new(
        ImportBudget::new(PdfImportLimits {
            entries: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    assert!(pdf_oxide::parser::parse_object_with_budget(b"[0]", Some(&request)).is_ok());
    request.restrict_structure_limits(40_000, 2).unwrap();
    request.restrict_structure_limits(80_000, 4).unwrap();
    assert!(pdf_oxide::parser::parse_object_with_budget(b"[1]", Some(&request)).is_ok());
    assert!(pdf_oxide::parser::parse_object_with_budget(b"[2]", Some(&request)).is_err());
    let failure = request.snapshot().failure.unwrap();
    assert_eq!(failure.reason, "collection entries");
    assert_eq!(
        (failure.used, failure.requested, failure.limit),
        (Some(2), Some(1), Some(2))
    );
    assert!(request.restrict_structure_limits(80_000, 4).is_err());
}

#[test]
fn grayscale_mask_streams_keep_alpha_polarity_and_base_colours() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    // Reader compatibility, independently exercised with PDFium: ordinary
    // DeviceGray Mask streams have alpha polarity, unlike ImageMask stencils.
    // These assertions protect decoded resources, not page geometry/goldens.
    for (bpc, samples, decode, extra, expected) in [
        (1, vec![0x50], "", "", vec![0, 255, 0, 255]),
        (2, vec![0x1b], "", "", vec![0, 85, 170, 255]),
        (4, vec![0x05, 0xaf], "", "", vec![0, 85, 170, 255]),
        (8, vec![0, 64, 128, 255], "", "", vec![0, 64, 128, 255]),
        (
            8,
            vec![0, 64, 128, 255],
            "/Decode [1 0]",
            "/ImageMask false",
            vec![255, 191, 127, 0],
        ),
        // PDFium's Matte unblend belongs only to its SMask branch. Do not
        // change base RGB because a writer also attached Matte to Mask.
        (
            8,
            vec![0, 85, 170, 255],
            "",
            "/Matte [1 1 1]",
            vec![0, 85, 170, 255],
        ),
    ] {
        let packed = flate(&samples);
        let mut mask = format!("<< /Type /XObject /Subtype /Image /Width 4 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent {bpc} {decode} {extra} /Filter /FlateDecode /Length {} >>\nstream\n", packed.len()).into_bytes();
        mask.extend(packed);
        mask.extend(b"\nendstream");
        let source = indexed_image_pdf(
            &[20, 40, 60].repeat(4),
            "/Width 4 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Mask 7 0 R",
            &mask,
        );
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgba8();
        assert_eq!(
            rgba.pixels().map(|p| p.0[3]).collect::<Vec<_>>(),
            expected,
            "bpc={bpc} {decode} {extra}"
        );
        assert!(rgba.pixels().all(|p| p.0[..3] == [20, 40, 60]));
    }
}

#[test]
fn grayscale_mask_compatibility_keeps_invalid_samples_and_budget_failures() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    for entries in [
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray",
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB",
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /ImageMask true",
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /ImageMask 1",
        "/Width 1 /Height 1 /BitsPerComponent 3 /ColorSpace /DeviceGray",
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask 7 0 R",
    ] {
        let mask = format!("<< /Subtype /Image {entries} /Length 1 >>\nstream\nX\nendstream");
        let source = indexed_image_pdf(
            &[20, 40, 60],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Mask 7 0 R",
            mask.as_bytes(),
        );
        let err = parse_pdf_with_options(&source, &PdfImportOptions::default())
            .err()
            .expect("invalid Mask must not be silently dropped");
        assert!(
            matches!(err.code, "PDF_PARSE_FAILED" | "PDF_IMAGE_UNSUPPORTED"),
            "{entries}: {err:?}"
        );
    }
    let mut mask = b"<< /Subtype /Image /Width 64 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Length 64 >>\nstream\n".to_vec();
    mask.extend([128; 64]);
    mask.extend(b"\nendstream");
    let source = indexed_image_pdf(
        &[20, 40, 60].repeat(64),
        "/Width 64 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Mask 7 0 R",
        &mask,
    );
    let doc = PdfDocument::from_bytes_with_limits(
        source.clone(),
        PdfImportLimits {
            codec_allocation_bytes: 255,
            ..Default::default()
        },
    )
    .unwrap();
    let handles = doc.page_image_handles(0).unwrap();
    assert!(matches!(
        handles[0].decode(),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(handles[0].decode().is_err(), "failed request stays failed");
    let next = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    let rgba = image::load_from_memory(&next.document.bin_data_content[0].data.load())
        .unwrap()
        .to_rgba8();
    assert!(
        rgba.pixels().all(|p| p.0 == [20, 40, 60, 128]),
        "new request must recover with actual alpha intact"
    );
}

use pdf_oxide::decoders::{decode_stream_with_budget, DecodeParams};
use pdf_oxide::error::Error;
use pdf_oxide::import_limits::{ImportBudget, PdfImportLimits};
use pdf_oxide::PdfDocument;
use std::io::Write;
use std::sync::Arc;

fn budget<T: TryInto<u64>>(stream: usize, total: T) -> ImportBudget
where
    T::Error: std::fmt::Debug,
{
    ImportBudget::new(PdfImportLimits {
        stream_bytes: stream,
        decoded_bytes: total.try_into().unwrap(),
        ..PdfImportLimits::default()
    })
    .unwrap()
}

#[test]
fn failed_reservations_keep_first_amount_and_successful_counters() {
    let request = budget(1024, 3);
    request.charge_decoded(2).unwrap();
    request.charge_decoded(2).unwrap_err();
    let observed = request.snapshot();
    assert_eq!(observed.decoded_bytes, 2);
    let failure = observed.failure.unwrap();
    assert_eq!(failure.reason, "document decoded bytes");
    assert_eq!(
        (failure.used, failure.requested, failure.limit),
        (Some(2), Some(2), Some(3))
    );
    request.reject("later failure");
    assert!(matches!(
        request.check(),
        Err(Error::ImportResourceLimit("document decoded bytes"))
    ));
    let first = request.snapshot().failure.unwrap();
    assert_eq!(
        (first.used, first.requested, first.limit),
        (Some(2), Some(2), Some(3))
    );
    let next = budget(1024, 3);
    next.charge_decoded(3).unwrap();
    assert!(next.snapshot().failure.is_none());
    assert_eq!(next.snapshot().decoded_bytes, 3);
}

fn decode(data: &[u8], filters: &[&str], budget: &ImportBudget) -> Result<Vec<u8>, Error> {
    decode_stream_with_budget(
        data,
        &filters.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
        None,
        budget,
    )
}

fn flate(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn object_file(objects: &[Vec<u8>], trailer: &str) -> Vec<u8> {
    let mut bytes = b"%PDF-1.5\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        writeln!(bytes, "{} 0 obj", i + 1).unwrap();
        bytes.extend(object);
        bytes.extend(b"\nendobj\n");
    }
    let xref = bytes.len();
    write!(
        bytes,
        "xref\n0 {}\n0000000000 65535 f \n",
        objects.len() + 1
    )
    .unwrap();
    for offset in offsets {
        writeln!(bytes, "{offset:010} 00000 n ").unwrap();
    }
    write!(
        bytes,
        "trailer\n<< /Size {} /Root 1 0 R {trailer} >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    )
    .unwrap();
    bytes
}

fn objects(content: &[u8], filter: &str, catalog_extra: &str) -> Vec<Vec<u8>> {
    let mut stream = format!("<< /Length {} {filter} >>\nstream\n", content.len()).into_bytes();
    stream.extend(content);
    stream.extend(b"\nendstream");
    vec![
        format!("<< /Type /Catalog /Pages 2 0 R {catalog_extra} >>").into_bytes(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".to_vec(),
        stream,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ]
}

#[test]
fn individual_output_boundaries_are_exact_for_supported_filters() {
    let cases = [
        (b"4142".to_vec(), "ASCIIHexDecode", b"AB".to_vec()),
        (vec![1, b'A', b'B', 128], "RunLengthDecode", b"AB".to_vec()),
        (vec![128, 16, 72, 80, 16], "LZWDecode", b"AB".to_vec()),
        (flate(b"AB"), "FlateDecode", b"AB".to_vec()),
        (b"z~>".to_vec(), "ASCII85Decode", vec![0; 4]),
    ];
    for (encoded, filter, expected) in cases {
        for cap in [expected.len() - 1, expected.len(), expected.len() + 1] {
            let budget = budget(cap, 1024 * 1024);
            let result = decode(&encoded, &[filter], &budget);
            if cap < expected.len() {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit(_))),
                    "{filter}: {result:?}"
                );
                assert!(budget.check().is_err());
            } else {
                assert_eq!(result.unwrap(), expected, "{filter}");
                assert_eq!(
                    budget.decoded_bytes(),
                    budget.codec_allocated_bytes() + expected.len() as u64
                );
            }
        }
    }
}

#[test]
fn brotli_output_is_bounded_before_vec_extension() {
    // Independently encoded by Node zlib.brotliCompressSync(Buffer.alloc(128, 65)).
    let compressed = [27, 127, 0, 248, 37, 130, 34, 177, 64, 32, 3];
    for cap in [127, 128, 129] {
        let budget = budget(cap, 512 * 1024);
        let result = decode(&compressed, &["BrotliDecode"], &budget);
        if cap == 127 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(budget.decoded_bytes(), budget.codec_allocated_bytes());
        } else {
            assert_eq!(result.unwrap(), vec![65; 128]);
            assert_eq!(budget.decoded_bytes(), budget.codec_allocated_bytes() + 128);
        }
    }
}

#[test]
fn cumulative_stream_budget_is_sticky_and_request_local() {
    let shared = budget(8, 7);
    assert_eq!(
        decode(b"41424344", &["ASCIIHexDecode"], &shared).unwrap(),
        b"ABCD"
    );
    assert!(matches!(
        decode(b"41424344", &["ASCIIHexDecode"], &shared),
        Err(Error::ImportResourceLimit(_))
    ));
    assert_eq!(shared.decoded_bytes(), 7);
    assert!(matches!(
        shared.reject("later failure"),
        Error::ImportResourceLimit("document decoded bytes")
    ));
    assert!(matches!(
        shared.check(),
        Err(Error::ImportResourceLimit("document decoded bytes"))
    ));
    assert!(decode(b"41", &["ASCIIHexDecode"], &shared).is_err());
    assert_eq!(
        decode(b"41", &["ASCIIHexDecode"], &budget(8, 8)).unwrap(),
        b"A"
    );
}

#[test]
fn intermediate_filter_outputs_consume_the_same_budget() {
    let shared = budget(8, 5);
    assert!(matches!(
        decode(b"34313432", &["ASCIIHexDecode", "ASCIIHexDecode"], &shared),
        Err(Error::ImportResourceLimit(_))
    ));
    assert_eq!(shared.decoded_bytes(), 5);
    assert_eq!(
        decode(
            b"34313432",
            &["ASCIIHexDecode", "ASCIIHexDecode"],
            &budget(8, 6)
        )
        .unwrap(),
        b"AB"
    );
}

#[test]
fn predictor_layout_overflow_and_zero_rows_are_rejected() {
    for columns in [0, usize::MAX] {
        let shared = budget(32, 128);
        let params = DecodeParams {
            predictor: 12,
            columns,
            colors: 3,
            bits_per_component: 8,
        };
        assert!(matches!(
            decode_stream_with_budget(&[0, 1, 2], &[], Some(&params), &shared),
            Err(Error::ImportResourceLimit(_))
        ));
    }
    let params = DecodeParams {
        predictor: 12,
        columns: 2,
        colors: 1,
        bits_per_component: 8,
    };
    assert_eq!(
        decode_stream_with_budget(&[0, 1, 2], &[], Some(&params), &budget(3, 16)).unwrap(),
        [1, 2]
    );
}

#[test]
fn zero_limits_do_not_disable_protection() {
    assert!(matches!(
        ImportBudget::new(PdfImportLimits {
            stream_bytes: 0,
            ..PdfImportLimits::default()
        }),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn lexical_depth_is_checked_before_the_recursive_parser_frame() {
    let policy = PdfImportLimits {
        nesting: 2,
        ..PdfImportLimits::default()
    };
    let accepted = Arc::new(ImportBudget::new(policy).unwrap());
    assert!(pdf_oxide::parser::parse_object_with_budget(b"[[0]]", Some(&accepted)).is_ok());
    let rejected = Arc::new(ImportBudget::new(policy).unwrap());
    assert!(pdf_oxide::parser::parse_object_with_budget(b"[[[0]]]", Some(&rejected)).is_err());
    assert!(rejected.check().is_err());
}

#[test]
fn parser_collection_and_object_counts_share_request_accounting() {
    for policy in [
        PdfImportLimits {
            entries: 1,
            ..PdfImportLimits::default()
        },
        PdfImportLimits {
            objects: 2,
            ..PdfImportLimits::default()
        },
    ] {
        let shared = Arc::new(ImportBudget::new(policy).unwrap());
        assert!(pdf_oxide::parser::parse_object_with_budget(b"[0 1]", Some(&shared)).is_err());
        assert!(shared.check().is_err());
    }
}

#[test]
fn cloned_streams_retain_the_original_document_budget() {
    let shared = Arc::new(budget(16, 11));
    let (_, object) = pdf_oxide::parser::parse_object_with_budget(
        b"<< /Length 4 >>\nstream\nABCD\nendstream",
        Some(&shared),
    )
    .unwrap();
    let cloned = object.clone();
    let pdf_oxide::object::Object::Stream { data, .. } = &cloned else {
        panic!("expected stream")
    };
    assert!(Arc::ptr_eq(data.budget().unwrap(), &shared));
    assert_eq!(object.decode_stream_data().unwrap(), b"ABCD");
    assert!(matches!(
        cloned.decode_stream_data(),
        Err(Error::ImportResourceLimit(_))
    ));
    assert_eq!(shared.decoded_bytes(), 8);
}

#[test]
fn declared_object_stream_count_is_checked_before_capacity() {
    let shared = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
    let (_, object) = pdf_oxide::parser::parse_object_with_budget(
        b"<< /Type /ObjStm /N 100001 /First 0 /Length 1 >>\nstream\n0\nendstream",
        Some(&shared),
    )
    .unwrap();
    assert!(matches!(
        pdf_oxide::objstm::parse_object_stream(&object),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn traditional_xref_counts_use_the_constructor_budget() {
    let source = object_file(&objects(b"", "", ""), "");
    let limits = PdfImportLimits {
        entries: 2,
        ..PdfImportLimits::default()
    };
    assert!(matches!(
        PdfDocument::from_bytes_with_limits(source, limits),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn xref_stream_decode_uses_the_budget_before_document_creation() {
    let compressed = flate(&[0; 64]);
    let mut source = b"%PDF-1.5\n".to_vec();
    let offset = source.len();
    write!(source, "1 0 obj\n<< /Type /XRef /Size 1 /W [1 4 2] /Root 2 0 R /Length {} /Filter /FlateDecode >>\nstream\n", compressed.len()).unwrap();
    source.extend(compressed);
    write!(source, "\nendstream\nendobj\nstartxref\n{offset}\n%%EOF\n").unwrap();
    let limits = PdfImportLimits {
        stream_bytes: 32,
        ..PdfImportLimits::default()
    };
    assert!(matches!(
        PdfDocument::from_bytes_with_limits(source, limits),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn decode_limit_does_not_become_empty_paths_success() {
    let source = object_file(
        &objects(&flate(&[b'q'; 128]), "/Filter /FlateDecode", ""),
        "",
    );
    let doc = PdfDocument::from_bytes_with_limits(
        source,
        PdfImportLimits {
            stream_bytes: 64,
            ..PdfImportLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        doc.extract_paths(0),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(doc.check_import_budget().is_err());
}

#[test]
fn page_path_and_image_consumers_share_a_bounded_parse() {
    let with_text = object_file(
        &objects(b"BT /F1 12 Tf 10 20 Td (A) Tj ET 0 0 m 1 1 l S", "", ""),
        "",
    );
    let all_consumers = PdfDocument::from_bytes_with_limits(
        with_text,
        PdfImportLimits {
            operators: 8,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(all_consumers.extract_spans(0).unwrap()[0].text, "A");
    assert_eq!(all_consumers.extract_paths(0).unwrap().len(), 1);
    assert!(all_consumers.page_image_handles(0).unwrap().is_empty());
    assert!(all_consumers.check_import_budget().is_ok());
    let source = object_file(&objects(b"q 1 0 0 1 0 0 cm 0 0 m 1 1 l S Q", "", ""), "");
    let doc = PdfDocument::from_bytes_with_limits(
        source,
        PdfImportLimits {
            operators: 6,
            decoded_bytes: 1024 * 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let paths = doc.extract_paths(0).unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].operations.len(), 2);
    // A separate image consumer sees the same parsed stream; re-parsing it
    // would exceed six operators. No XObject is fabricated from path paints.
    assert!(doc.page_image_handles(0).unwrap().is_empty());
    // Cache hits still copy owned payload under the same cumulative budget.
    let mut failed = false;
    for _ in 0..10000 {
        if doc.page_image_handles(0).is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed, "cache hits must not bypass cumulative accounting");
    assert!(doc.check_import_budget().is_err());
    assert!(
        doc.extract_paths(0).is_err(),
        "failed requests cannot return cached operators"
    );
}

#[test]
fn content_operator_and_graphics_stack_limits_are_request_local() {
    let shared = Arc::new(
        ImportBudget::new(PdfImportLimits {
            operators: 2,
            graphics_stack: 1,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    let mut seen = 0;
    pdf_oxide::content::parser::execute_content_with_budget(b"q Q", &shared, |_| {
        seen += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(seen, 2);
    assert!(matches!(
        pdf_oxide::content::parser::execute_content_with_budget(b"q", &shared, |_| Ok(())),
        Err(Error::ImportResourceLimit(_))
    ));
    let next = Arc::new(
        ImportBudget::new(PdfImportLimits {
            graphics_stack: 1,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    assert!(matches!(
        pdf_oxide::content::parser::execute_content_with_budget(b"q q", &next, |_| Ok(())),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn content_operands_use_the_operator_domain_without_bypassing_limits() {
    let shared = Arc::new(
        ImportBudget::new(PdfImportLimits {
            objects: 1,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    let mut seen = 0;
    pdf_oxide::content::parser::execute_content_with_budget(
        b"1 0 0 1 10 20 cm 1 2 m 3 4 l S",
        &shared,
        |_| {
            seen += 1;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(seen, 4);
    assert!(pdf_oxide::parser::parse_object_with_budget(b"0", Some(&shared)).is_ok());
    assert!(pdf_oxide::parser::parse_object_with_budget(b"1", Some(&shared)).is_err());
    assert!(matches!(
        shared.check(),
        Err(Error::ImportResourceLimit("parsed object count"))
    ));

    for count in [64, 65] {
        let next = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
        let content = format!("{}unknown", "0 ".repeat(count));
        let result = pdf_oxide::content::parser::execute_content_with_budget(
            content.as_bytes(),
            &next,
            |_| Ok(()),
        );
        if count == 64 {
            result.unwrap();
            next.check().unwrap();
        } else {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(next.check().is_err());
        }
    }
}

#[test]
fn nested_content_operands_cannot_bypass_lexical_limits() {
    let shared = Arc::new(
        ImportBudget::new(PdfImportLimits {
            nesting: 2,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    assert!(matches!(
        pdf_oxide::content::parser::execute_content_with_budget(b"[[[0]]] TJ", &shared, |_| Ok(())),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn product_import_preserves_fatal_code_then_accepts_a_new_document() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let nested = format!("/Unused {}0{}", "[".repeat(32), "]".repeat(32));
    let source = object_file(&objects(b"", "", &nested), "");
    let error = parse_pdf_with_options(&source, &PdfImportOptions::default())
        .err()
        .unwrap();
    assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
    let normal = object_file(&objects(b"BT /F1 12 Tf 20 100 Td (OK) Tj ET", "", ""), "");
    let imported = parse_pdf_with_options(&normal, &PdfImportOptions::default()).unwrap();
    assert_eq!(imported.document.sections.len(), 1);
    assert!(!imported.metadata.parser_allocation_bounds_proven);
}

#[test]
fn jpeg_header_mismatch_is_rejected_before_png_and_normal_pixels_are_unchanged() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
        .encode(
            &[255, 0, 0, 0, 255, 0],
            2,
            1,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    let wrong = PdfImage::new(
        1,
        1,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Jpeg(jpeg.clone()),
    );
    assert!(wrong.to_png_bytes_with_budget(&budget(4096, 8192)).is_err());
    let normal = PdfImage::new(2, 1, ColorSpace::DeviceRGB, 8, ImageData::Jpeg(jpeg));
    // Compare decoded pixels, not the compressor's container representation.
    assert_eq!(
        image::load_from_memory(
            &normal
                .to_png_bytes_with_budget(&budget(4096, 2 * 1024 * 1024))
                .unwrap()
        )
        .unwrap()
        .to_rgb8(),
        image::load_from_memory(&normal.to_png_bytes().unwrap())
            .unwrap()
            .to_rgb8()
    );
}

#[test]
fn huge_image_declarations_are_rejected_without_allocating_pixels() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    let image = PdfImage::new(
        u32::MAX,
        u32::MAX,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels: vec![],
            format: PixelFormat::RGB,
        },
    );
    assert!(matches!(
        image.to_png_bytes_with_budget(&budget(64, 128)),
        Err(Error::ImportResourceLimit(_))
    ));
}

#[test]
fn cmap_expansion_boundaries_apply_to_all_mapping_forms() {
    use pdf_oxide::fonts::cmap::parse_tounicode_cmap_with_budget;
    for (source, expected) in [
        (
            "beginbfrange\n<00> <02> <0041>\nendbfrange",
            ["A", "B", "C"],
        ),
        (
            "beginbfrange\n<00> <02> [<0041> <0042> <0043>]\nendbfrange",
            ["A", "B", "C"],
        ),
        (
            "beginnotdefrange\n<00> <02> <0041>\nendnotdefrange",
            ["A", "A", "A"],
        ),
    ] {
        for cap in [2, 3, 4] {
            let budget = ImportBudget::new(PdfImportLimits {
                cmap_entries: cap,
                ..PdfImportLimits::default()
            })
            .unwrap();
            let result = parse_tounicode_cmap_with_budget(source.as_bytes(), &budget);
            if cap == 2 {
                assert!(matches!(
                    result,
                    Err(Error::ImportResourceLimit("CMap entries"))
                ));
            } else {
                let cmap = result.unwrap();
                for (i, value) in expected.iter().enumerate() {
                    assert_eq!(cmap.get(&(i as u32)).as_deref(), Some(*value));
                }
            }
        }
    }
}

#[test]
fn cmap_declared_huge_ranges_and_reverse_ranges_do_not_wrap() {
    use pdf_oxide::fonts::cmap::parse_tounicode_cmap_with_budget;
    for source in [
        "beginbfrange\n<00000000> <FFFFFFFF> <0041>\nendbfrange",
        "beginbfrange\n<00000000> <FFFFFFFF> [<0041>]\nendbfrange",
        "beginnotdefrange\n<00000000> <FFFFFFFF> <0041>\nendnotdefrange",
    ] {
        let budget = ImportBudget::new(PdfImportLimits {
            cmap_entries: 16,
            ..PdfImportLimits::default()
        })
        .unwrap();
        assert!(matches!(
            parse_tounicode_cmap_with_budget(source.as_bytes(), &budget),
            Err(Error::ImportResourceLimit("CMap entries"))
        ));
    }
    let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let cmap =
        parse_tounicode_cmap_with_budget(b"beginbfrange\n<02> <00> [<0041>]\nendbfrange", &budget)
            .unwrap();
    assert!(cmap.is_empty());
    budget.check().unwrap();

    let budget = ImportBudget::new(PdfImportLimits {
        cmap_entries: 10_002,
        ..PdfImportLimits::default()
    })
    .unwrap();
    let cmap = parse_tounicode_cmap_with_budget(
        b"beginbfrange\n<0000> <2711> <0020>\nendbfrange",
        &budget,
    )
    .unwrap();
    let last = char::from_u32(0x20 + 10_001).unwrap().to_string();
    assert_eq!(cmap.get(&10_001).as_deref(), Some(last.as_str()));
}

#[test]
fn cmap_source_and_line_limits_precede_mapping_expansion() {
    use pdf_oxide::fonts::cmap::parse_tounicode_cmap_with_budget;
    let source = b"beginbfchar\n<00> <0041>\nendbfchar";
    for cap in [source.len() - 1, source.len(), source.len() + 1] {
        let budget = ImportBudget::new(PdfImportLimits {
            cmap_bytes: cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        let result = parse_tounicode_cmap_with_budget(source, &budget);
        if cap < source.len() {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("CMap source bytes"))
            ));
        } else {
            assert_eq!(result.unwrap().get(&0).as_deref(), Some("A"));
        }
    }
    let budget = ImportBudget::new(PdfImportLimits {
        string_bytes: 16,
        ..PdfImportLimits::default()
    })
    .unwrap();
    assert!(matches!(
        parse_tounicode_cmap_with_budget(b"xxxxxxxxxxxxxxxxx", &budget),
        Err(Error::ImportResourceLimit("CMap line bytes"))
    ));
    let source = b"beginbfchar beginbfchar beginbfchar";
    let budget = ImportBudget::new(PdfImportLimits {
        cmap_scan_bytes: source.len() * 5,
        ..PdfImportLimits::default()
    })
    .unwrap();
    assert!(matches!(
        parse_tounicode_cmap_with_budget(source, &budget),
        Err(Error::ImportResourceLimit("CMap scan bytes"))
    ));
}

#[test]
fn cmap_overwrites_and_repeated_parses_consume_request_work() {
    use pdf_oxide::fonts::cmap::parse_tounicode_cmap_with_budget;
    let source = b"beginbfchar\n<00> <0041>\n<00> <0042>\nendbfchar";
    let policy = PdfImportLimits {
        cmap_entries: 3,
        ..PdfImportLimits::default()
    };
    let budget = ImportBudget::new(policy).unwrap();
    assert_eq!(
        parse_tounicode_cmap_with_budget(source, &budget)
            .unwrap()
            .get(&0)
            .as_deref(),
        Some("B")
    );
    assert!(parse_tounicode_cmap_with_budget(source, &budget).is_err());
    assert!(parse_tounicode_cmap_with_budget(source, &ImportBudget::new(policy).unwrap()).is_ok());
}

#[test]
fn limited_lazy_cmap_bypasses_global_cache_and_shares_clones() {
    use pdf_oxide::fonts::cmap::LazyCMap;
    let source = b"beginbfchar\n<11> <0051>\n<12> <0052>\n<13> <0053>\nendbfchar";
    let legacy = LazyCMap::new(source.to_vec());
    assert_eq!(legacy.get().unwrap().get(&0x11).as_deref(), Some("Q"));
    let low = Arc::new(
        ImportBudget::new(PdfImportLimits {
            cmap_entries: 2,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    let limited = LazyCMap::new_with_budget(source.to_vec(), Arc::clone(&low)).unwrap();
    assert!(limited.get().is_none());
    assert!(low.check().is_err());

    let fresh = Arc::new(
        ImportBudget::new(PdfImportLimits {
            cmap_entries: 3,
            ..PdfImportLimits::default()
        })
        .unwrap(),
    );
    let lazy = LazyCMap::new_with_budget(source.to_vec(), Arc::clone(&fresh)).unwrap();
    let cloned = lazy.clone();
    assert_eq!(lazy.raw_data().as_ptr(), cloned.raw_data().as_ptr());
    assert!(Arc::ptr_eq(&lazy.get().unwrap(), &cloned.get().unwrap()));
    fresh.reject("test request exhausted");
    assert!(cloned.get().is_none());
}

#[test]
fn malformed_non_ascii_cmap_hex_cannot_panic_at_utf8_boundaries() {
    use pdf_oxide::fonts::cmap::parse_tounicode_cmap_with_budget;
    for source in [
        b"beginbfchar\n<01> <AAA\x80AAAAA>\nendbfchar".as_slice(),
        b"beginbfrange\n<01> <01> [<AAA\x80AAAAA>]\nendbfrange".as_slice(),
    ] {
        let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert!(parse_tounicode_cmap_with_budget(source, &budget)
            .unwrap()
            .is_empty());
        assert!(pdf_oxide::fonts::cmap::parse_tounicode_cmap(source)
            .unwrap()
            .is_empty());
    }
}

#[test]
fn cid_metric_expansion_is_bounded_before_range_or_array_materialization() {
    use pdf_oxide::fonts::font_dict::FontInfo;
    for metrics in [
        "/W [0 2 500]",
        "/W [0 [500 500 500]]",
        "/W2 [0 2 -1000 500 880]",
        "/W2 [0 [-1000 500 880 -1000 500 880 -1000 500 880]]",
    ] {
        let font = format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont /Probe /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /Probe /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> {metrics} >>] >>"
        );
        let (_, font) = pdf_oxide::parser::parse_object(font.as_bytes()).unwrap();
        for cap in [2, 3, 4] {
            let doc = PdfDocument::from_bytes_with_limits(
                object_file(&objects(b"", "", ""), ""),
                PdfImportLimits {
                    font_entries: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            let result = FontInfo::from_dict(&font, &doc);
            if cap == 2 {
                assert!(matches!(
                    doc.check_import_budget(),
                    Err(Error::ImportResourceLimit("font metric entries"))
                ));
                assert!(result.is_err());
            } else {
                assert!(result.is_ok(), "{metrics}: {result:?}");
                doc.check_import_budget().unwrap();
            }
        }
    }
}

#[test]
fn product_font_budget_accepts_multiple_cid_faces_and_preserves_explicit_limits() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    // Three independent CID faces with 40,000 explicitly declared widths each.
    // This is a resource-only PDF, with no text/paint or layout expectations.
    // A nonempty, nonpainting text block exercises font loading; an empty
    // content stream is correctly skipped before its fonts are needed.
    let mut parts = objects(b"BT /F0 12 Tf ET", "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F0 5 0 R /F1 6 0 R /F2 7 0 R >> >> /Contents 4 0 R >>".to_vec();
    for i in 0..3 {
        let font = format!("<< /Type /Font /Subtype /Type0 /BaseFont /MetricFace{i} /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /MetricFace{i} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /DW 1000 /W [0 39999 {}] >>] >>", 500 + i * 100).into_bytes();
        if i == 0 {
            parts[4] = font;
        } else {
            parts.push(font);
        }
    }
    let input = object_file(&parts, "");
    let opened = parse_pdf_with_policy(&input, &Default::default(), PdfImportPolicy::default())
        .unwrap_or_else(|error| panic!("ordinary multi-face font resources: {error:?}"));
    assert_eq!(opened.metadata.pages.len(), 1);
    assert!(opened.metadata.resource_usage.failure.is_none());

    for cap in [100_000, 119_999, 120_000] {
        let mut policy = PdfImportPolicy::default();
        policy.backend.font_entries = cap;
        let result = parse_pdf_with_policy(&input, &Default::default(), policy);
        if cap == 120_000 {
            assert!(result.is_ok());
        } else {
            let error = result.err().expect("explicit lower cap must reject");
            assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
            let failure = error
                .diagnostics
                .unwrap()
                .resource_usage
                .unwrap()
                .failure
                .unwrap();
            assert_eq!(failure.reason, "font metric entries");
            assert_eq!(
                (failure.used, failure.requested, failure.limit),
                (Some(80_000), Some(40_000), Some(cap as u64))
            );
        }
    }
}

fn repeated_stencil_parts(count: usize) -> Vec<Vec<u8>> {
    let mut content = String::from("0.2 0.4 0.6 rg\n");
    for i in 0..count {
        content.push_str(&format!(
            "q 2 0 0 2 {} {} cm /Im Do Q\n",
            20 + (i % 64) * 2,
            20 + (i / 64) * 2
        ));
    }
    content.push_str("BT /F1 12 Tf 20 150 Td (VISIBLE) Tj ET");
    let mut parts = objects(content.as_bytes(), "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    parts.push(b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ImageMask true /BitsPerComponent 1 /Length 1 >>\nstream\n\x00\nendstream".to_vec());
    parts
}

fn repeated_stencil_file(count: usize) -> Vec<u8> {
    object_file(&repeated_stencil_parts(count), "")
}

#[test]
fn repeated_stencils_keep_actual_work_and_explicit_request_caps() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    let count = 64 * 48;
    let input = repeated_stencil_file(count);
    let opened = parse_pdf_with_policy(&input, &Default::default(), PdfImportPolicy::default())
        .unwrap_or_else(|e| panic!("ordinary repeated texture: {e:?}"));
    // Two text interpretations plus path and image-handle interpretation.
    // Combining decoded pictures must never erase the original traversal work.
    let visits = count * 4;
    assert_eq!(opened.metadata.resource_usage.xobject_visits, visits);
    assert_eq!(opened.document.bin_data_content.len(), 1);
    for cap in [visits - 1, visits] {
        let mut policy = PdfImportPolicy::default();
        policy.backend.xobject_visits = cap;
        let result = parse_pdf_with_policy(&input, &Default::default(), policy);
        if cap == visits {
            assert!(result.is_ok());
        } else {
            let error = result.err().expect("explicit visit cap must reject");
            assert_eq!(
                error
                    .diagnostics
                    .unwrap()
                    .resource_usage
                    .unwrap()
                    .failure
                    .unwrap()
                    .reason,
                "XObject visits"
            );
        }
    }
    let mut policy = PdfImportPolicy::default();
    policy.max_document_image_placements = count - 1;
    let error = parse_pdf_with_policy(&input, &Default::default(), policy)
        .err()
        .expect("original paints, not one projected picture, consume placements");
    assert_eq!(
        error.diagnostics.unwrap().frontend_limit.unwrap().limit,
        Some((count - 1) as u64)
    );
    // Raw decoder defaults and smaller trusted-host reference limits remain.
    assert_eq!(PdfImportLimits::default().xobject_visits, 10_000);
    let mut policy = PdfImportPolicy::default();
    policy.backend.reference_visits = 8;
    assert!(parse_pdf_with_policy(&input, &Default::default(), policy).is_err());
    assert!(parse_pdf_with_policy(&input, &Default::default(), PdfImportPolicy::default()).is_ok());
}

#[test]
fn stencil_projection_preserves_samples_and_stops_at_text_or_colour() {
    use rhwp::parser::pdf::parse_pdf_with_options;
    let pair = "q 2 0 0 2 10 10 cm /Im Do Q q 2 0 0 2 12 10 cm /Im Do Q ";
    for (between, expected) in [
        ("BT /F1 12 Tf 20 150 Td (VISIBLE) Tj ET ", 2),
        ("1 0 0 rg ", 2),
        ("/Zero gs ", 1),
        ("", 1),
    ] {
        let mut parts = objects(
            format!("0.2 0.4 0.6 rg {pair}{between}{pair}").as_bytes(),
            "",
            "",
        );
        parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> /XObject << /Im 6 0 R >> /ExtGState << /Zero << /ca 0 >> >> >> /Contents 4 0 R >>".to_vec();
        parts.push(b"<< /Type /XObject /Subtype /Image /Width 2 /Height 2 /ImageMask true /BitsPerComponent 1 /Length 2 >>\nstream\n\x40\x80\nendstream".to_vec());
        let result = parse_pdf_with_options(&object_file(&parts, ""), &Default::default()).unwrap();
        assert_eq!(
            result.document.bin_data_content.len(),
            expected,
            "barrier {between}"
        );
        for resource in &result.document.bin_data_content {
            let image = image::load_from_memory(&resource.data.load())
                .unwrap()
                .to_rgba8();
            assert_eq!(image.dimensions(), (4, 2));
            assert_eq!(
                image.pixels().map(|p| p[3]).collect::<Vec<_>>(),
                [255, 0, 255, 0, 0, 255, 0, 255]
            );
        }
    }
}

#[test]
fn rejected_stencil_projection_uses_one_linear_fallback_run() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    let count = 512;
    // An ordinary packed 1-bit gray image is not a current-colour stencil.
    // Each paint needs path/handle/decode object visits, plus bounded page/font
    // setup. Rechecking every failed suffix adds a fourth per-paint object load.
    let mut parts = repeated_stencil_parts(count);
    parts[5] = String::from_utf8(parts[5].clone())
        .unwrap()
        .replace("/ImageMask true", "/ColorSpace /DeviceGray")
        .into_bytes();
    let mut policy = PdfImportPolicy::default();
    policy.backend.reference_visits = count * 3 + 64;
    let result = parse_pdf_with_policy(&object_file(&parts, ""), &Default::default(), policy)
        .unwrap_or_else(|e| panic!("linear ordinary-image fallback: {e:?}"));
    assert_eq!(result.document.bin_data_content.len(), count);
}

fn repeated_same_stencil_file(count: usize) -> Vec<u8> {
    let mut parts = repeated_stencil_parts(0);
    let mut content = b"/Im Do\n".repeat(count);
    content.extend(b"BT /F1 12 Tf 20 150 Td (VISIBLE) Tj ET");
    parts[3] = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    parts[3].extend(content);
    parts[3].extend(b"\nendstream");
    object_file(&parts, "")
}

#[test]
fn product_selected_visit_policy_reaches_actual_interpreters() {
    use rhwp::parser::pdf::parse_pdf_with_options;
    // Only 70,005 lexically parsed operators, but four interpretations of each
    // Do exceed the one-page traversal ceiling. Shared CTM/cache hits are work.
    let error = parse_pdf_with_options(&repeated_same_stencil_file(70_000), &Default::default())
        .err()
        .expect("selected traversal cap must be enforced, not metadata only");
    let failure = error
        .diagnostics
        .unwrap()
        .resource_usage
        .unwrap()
        .failure
        .unwrap();
    assert_eq!(failure.reason, "XObject visits");
    assert_eq!(
        (failure.used, failure.requested, failure.limit),
        (Some(250_000), Some(1), Some(250_000))
    );
}

#[test]
fn narrowed_traversal_caps_preserve_consumption_and_sticky_failure() {
    let request = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
    let doc = PdfDocument::from_bytes_with_budget(form_tree_file(3), Arc::clone(&request)).unwrap();
    request.restrict_traversal_limits(64, 6).unwrap();
    doc.extract_spans(0).unwrap();
    assert_eq!(request.snapshot().xobject_visits, 6);
    request
        .restrict_traversal_limits(usize::MAX, usize::MAX)
        .unwrap();
    assert!(matches!(
        doc.page_image_handles(0),
        Err(Error::ImportResourceLimit("XObject visits"))
    ));
    assert_eq!(request.snapshot().xobject_visits, 6);
    assert!(request.restrict_traversal_limits(64, 64).is_err());
    let next = Arc::new(ImportBudget::new(PdfImportLimits::default()).unwrap());
    let doc = PdfDocument::from_bytes_with_budget(form_tree_file(1), Arc::clone(&next)).unwrap();
    doc.get_page(0).unwrap();
    let used = next.snapshot().reference_visits;
    assert!(used > 1);
    assert!(next.restrict_traversal_limits(used - 1, 100).is_err());
    let failure = next.snapshot().failure.unwrap();
    assert_eq!(
        (
            failure.reason,
            failure.used,
            failure.requested,
            failure.limit
        ),
        (
            "reference visits",
            Some(used as u64),
            Some(0),
            Some((used - 1) as u64)
        )
    );
}

#[test]
fn content_array_combination_includes_separator_bytes_in_the_buffer_cap() {
    let mut parts = objects(b"1234", "", "");
    parts[2] =
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents [4 0 R 6 0 R] >>".to_vec();
    parts.push(b"<< /Length 4 >>\nstream\n5678\nendstream".to_vec());
    for cap in [9, 10, 11] {
        let doc = PdfDocument::from_bytes_with_limits(
            object_file(&parts, ""),
            PdfImportLimits {
                stream_bytes: cap,
                ..PdfImportLimits::default()
            },
        )
        .unwrap();
        let result = doc.get_page_content_data(0);
        if cap == 9 {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("stream output bytes"))
            ));
            assert!(doc.get_page_content_data(0).is_err());
        } else {
            assert_eq!(result.unwrap(), b"1234\n5678\n");
        }
    }
}

#[test]
fn cached_content_copies_consume_budget_and_cannot_revive_failed_imports() {
    let source = object_file(&objects(b"1234", "", ""), "");
    let probe =
        PdfDocument::from_bytes_with_limits(source.clone(), PdfImportLimits::default()).unwrap();
    assert_eq!(probe.get_page_content_data(0).unwrap(), b"1234");
    let first_read_bytes = probe.import_budget().unwrap().decoded_bytes();
    // Parsing and metadata copies precede the cache. After warming it, each
    // returned content buffer must still consume exactly its four payload bytes.
    let before = probe.import_budget().unwrap().decoded_bytes();
    assert_eq!(probe.get_page_content_data(0).unwrap(), b"1234");
    assert_eq!(probe.import_budget().unwrap().decoded_bytes() - before, 4);
    let doc = PdfDocument::from_bytes_with_limits(
        source.clone(),
        PdfImportLimits {
            stream_bytes: 8,
            decoded_bytes: first_read_bytes + 4,
            ..PdfImportLimits::default()
        },
    )
    .unwrap();
    assert_eq!(doc.get_page_content_data(0).unwrap(), b"1234");
    assert_eq!(doc.get_page_content_data(0).unwrap(), b"1234");
    assert!(matches!(
        doc.get_page_content_data(0),
        Err(Error::ImportResourceLimit("document decoded bytes"))
    ));
    assert!(doc.get_page_content_data(0).is_err());
    let next = PdfDocument::from_bytes_with_limits(source, PdfImportLimits::default()).unwrap();
    assert_eq!(next.get_page_content_data(0).unwrap(), b"1234");
}

#[test]
fn product_import_does_not_swallow_lazy_cmap_resource_failure() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let mut parts = objects(b"BT /F1 12 Tf 20 100 Td (A) Tj ET", "", "");
    parts[4] = b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /ToUnicode 6 0 R >>".to_vec();
    let cmap = b"beginbfrange\n<00000000> <FFFFFFFF> <0041>\nendbfrange";
    let mut stream = format!("<< /Length {} >>\nstream\n", cmap.len()).into_bytes();
    stream.extend(cmap);
    stream.extend(b"\nendstream");
    parts.push(stream);
    let error = parse_pdf_with_options(&object_file(&parts, ""), &PdfImportOptions::default())
        .err()
        .unwrap();
    assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
    let normal = object_file(&objects(b"BT /F1 12 Tf 20 100 Td (OK) Tj ET", "", ""), "");
    assert!(parse_pdf_with_options(&normal, &PdfImportOptions::default()).is_ok());
}

fn page_tree_file(depth: usize, count: i64) -> Vec<u8> {
    let mut parts = vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()];
    for level in 0..depth {
        parts.push(
            format!(
                "<< /Type /Pages /Count {count} /Kids [{} 0 R] >>",
                level + 3
            )
            .into_bytes(),
        );
    }
    parts.push(b"<< /Type /Page /MediaBox [0 0 200 200] >>".to_vec());
    object_file(&parts, "")
}

fn form_tree_file(depth: usize) -> Vec<u8> {
    let mut parts = objects(b"/Fm Do", "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Fm 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    for level in 0..depth {
        let (content, xobjects) = if level + 1 == depth {
            ("BT /F1 12 Tf 20 100 Td (VISIBLE) Tj ET", String::new())
        } else {
            (
                "BT ET /Fm Do",
                format!("/XObject << /Fm {} 0 R >>", level + 7),
            )
        };
        parts.push(format!("<< /Type /XObject /Subtype /Form /BBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> {xobjects} >> /Length {} >>\nstream\n{content}\nendstream", content.len()).into_bytes());
    }
    object_file(&parts, "")
}

#[test]
fn traversal_page_depth_limit_reaches_public_page_count_and_get_page() {
    for depth in [31, 32, 33] {
        let doc = PdfDocument::from_bytes_with_limits(
            page_tree_file(depth, 1),
            PdfImportLimits::default(),
        )
        .unwrap();
        let result = doc.get_page(0);
        if depth > 32 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(matches!(
                doc.page_count(),
                Err(Error::ImportResourceLimit(_))
            ));
        } else {
            result.unwrap();
            assert_eq!(doc.page_count().unwrap(), 1);
        }
    }
}

#[test]
fn traversal_page_count_scan_cannot_return_partial_success_after_depth_limit() {
    let doc = PdfDocument::from_bytes_with_limits(
        page_tree_file(33, 8_388_608),
        PdfImportLimits::default(),
    )
    .unwrap();
    assert!(matches!(
        doc.page_count(),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(doc.get_page(0).is_err());
}

#[test]
fn traversal_failed_request_cannot_return_a_cached_page_or_object() {
    let doc = PdfDocument::from_bytes_with_limits(page_tree_file(1, 1), PdfImportLimits::default())
        .unwrap();
    let reference = pdf_oxide::object::ObjectRef::new(2, 0);
    doc.get_page(0).unwrap();
    doc.load_object(reference).unwrap();
    doc.import_budget()
        .unwrap()
        .reject("test traversal failure");
    assert!(matches!(
        doc.get_page(0),
        Err(Error::ImportResourceLimit("test traversal failure"))
    ));
    assert!(matches!(
        doc.load_object(reference),
        Err(Error::ImportResourceLimit("test traversal failure"))
    ));
    let fresh =
        PdfDocument::from_bytes_with_limits(page_tree_file(1, 1), PdfImportLimits::default())
            .unwrap();
    assert_eq!(fresh.page_count().unwrap(), 1);
    fresh.get_page(0).unwrap();
}

#[test]
fn traversal_form_depth_limit_is_fatal_instead_of_silent_text_loss() {
    for depth in [15, 16, 17] {
        let doc =
            PdfDocument::from_bytes_with_limits(form_tree_file(depth), PdfImportLimits::default())
                .unwrap();
        let text = doc.extract_text(0);
        if depth > 16 {
            assert!(matches!(text, Err(Error::ImportResourceLimit(_))));
            assert!(doc.get_page(0).is_err());
        } else {
            assert!(text.unwrap().contains("VISIBLE"));
        }
    }
}

#[test]
fn traversal_product_import_propagates_tree_and_form_failures_then_recovers() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    for input in [page_tree_file(33, 1), form_tree_file(17)] {
        let error = parse_pdf_with_options(&input, &PdfImportOptions::default())
            .err()
            .unwrap();
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
        assert!(
            parse_pdf_with_options(&page_tree_file(1, 1), &PdfImportOptions::default()).is_ok()
        );
    }
}

#[test]
fn traversal_form_depth_and_work_limits_cover_each_product_extraction_mode() {
    for mode in ["text", "paths", "images"] {
        for (depth, cap) in [(2, 3), (3, 3), (4, 3)] {
            let doc = PdfDocument::from_bytes_with_limits(
                form_tree_file(depth),
                PdfImportLimits {
                    form_depth: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            let result = match mode {
                "text" => doc.extract_spans(0).map(|spans| {
                    assert!(spans.iter().any(|s| s.text.contains("VISIBLE")));
                }),
                "paths" => doc.extract_paths(0).map(|paths| assert!(paths.is_empty())),
                _ => doc
                    .page_image_handles(0)
                    .map(|images| assert!(images.is_empty())),
            };
            if depth > cap {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit("Form depth"))),
                    "{mode}: {result:?}"
                );
            } else {
                result.unwrap();
                doc.check_import_budget().unwrap();
            }
        }
        // Span postprocessing stamps character origins using a second content
        // interpretation. Both passes must consume the same request work budget.
        let visits = if mode == "text" { 6 } else { 3 };
        for cap in [visits - 1, visits, visits + 1] {
            let doc = PdfDocument::from_bytes_with_limits(
                form_tree_file(3),
                PdfImportLimits {
                    xobject_visits: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            let result = match mode {
                "text" => doc.extract_spans(0).map(|_| ()),
                "paths" => doc.extract_paths(0).map(|_| ()),
                _ => doc.page_image_handles(0).map(|_| ()),
            };
            if cap < visits {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit("XObject visits"))),
                    "{mode}: {result:?}"
                );
                assert!(doc.get_page(0).is_err());
            } else {
                result.unwrap_or_else(|error| panic!("{mode}, visit cap {cap}: {error:?}"));
            }
        }
    }
}

#[test]
fn traversal_object_cache_hits_still_consume_request_visits() {
    let doc = PdfDocument::from_bytes_with_limits(
        page_tree_file(1, 1),
        PdfImportLimits {
            reference_visits: 64,
            ..PdfImportLimits::default()
        },
    )
    .unwrap();
    let reference = pdf_oxide::object::ObjectRef::new(2, 0);
    doc.load_object(reference).unwrap();
    let mut rejected = false;
    for _ in 0..65 {
        match doc.load_object(reference) {
            Ok(_) => {}
            Err(Error::ImportResourceLimit("reference visits")) => {
                rejected = true;
                break;
            }
            Err(error) => panic!("unexpected failure: {error:?}"),
        }
    }
    assert!(rejected);
    assert!(doc.get_page(0).is_err());
}

#[test]
fn traversal_bulk_page_cache_cannot_swallow_deep_branch_failure() {
    let mut kids = (3..67).map(|id| format!("{id} 0 R")).collect::<Vec<_>>();
    kids.push("67 0 R".into());
    let mut parts = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        format!("<< /Type /Pages /Count 65 /Kids [{}] >>", kids.join(" ")).into_bytes(),
    ];
    for _ in 0..64 {
        parts.push(b"<< /Type /Page /MediaBox [0 0 200 200] >>".to_vec());
    }
    for id in 67..99 {
        parts.push(format!("<< /Type /Pages /Count 1 /Kids [{} 0 R] >>", id + 1).into_bytes());
    }
    parts.push(b"<< /Type /Page /MediaBox [0 0 200 200] >>".to_vec());
    let doc =
        PdfDocument::from_bytes_with_limits(object_file(&parts, ""), PdfImportLimits::default())
            .unwrap();
    for page in 0..64 {
        doc.get_page(page).unwrap();
    }
    assert!(matches!(
        doc.get_page(64),
        Err(Error::ImportResourceLimit("page tree depth"))
    ));
    assert!(doc.get_page(0).is_err());
}

#[test]
fn traversal_page_count_uses_actual_leaves_instead_of_declared_count() {
    let parts = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R 4 0 R] >>".to_vec(),
        b"<< /Type /Page /MediaBox [0 0 200 200] >>".to_vec(),
        b"<< /Type /Page /MediaBox [0 0 200 200] >>".to_vec(),
    ];
    let doc =
        PdfDocument::from_bytes_with_limits(object_file(&parts, ""), PdfImportLimits::default())
            .unwrap();
    assert_eq!(doc.page_count().unwrap(), 2);
    doc.get_page(1).unwrap();
}

#[test]
fn traversal_zero_policy_values_cannot_disable_protection() {
    for limits in [
        PdfImportLimits {
            reference_depth: 0,
            ..PdfImportLimits::default()
        },
        PdfImportLimits {
            reference_visits: 0,
            ..PdfImportLimits::default()
        },
        PdfImportLimits {
            form_depth: 0,
            ..PdfImportLimits::default()
        },
        PdfImportLimits {
            xobject_visits: 0,
            ..PdfImportLimits::default()
        },
    ] {
        assert!(matches!(
            ImportBudget::new(limits),
            Err(Error::ImportResourceLimit("zero import budget"))
        ));
    }
}

fn cached_text_file() -> Vec<u8> {
    object_file(
        &objects(b"BT /F1 12 Tf 20 100 Td (CACHE) Tj ET", "", ""),
        "",
    )
}

#[test]
fn cached_text_cannot_revive_a_failed_request() {
    for mode in ["spans", "chars"] {
        let doc =
            PdfDocument::from_bytes_with_limits(cached_text_file(), PdfImportLimits::default())
                .unwrap();
        if mode == "spans" {
            assert!(!doc.extract_spans(0).unwrap().is_empty());
        } else {
            assert!(!doc.extract_chars(0).unwrap().is_empty());
        }
        doc.import_budget().unwrap().reject("test cache failure");
        let result = if mode == "spans" {
            doc.extract_spans(0).map(|_| ())
        } else {
            doc.extract_chars(0).map(|_| ())
        };
        assert!(
            matches!(
                result,
                Err(Error::ImportResourceLimit("test cache failure"))
            ),
            "{mode}: {result:?}"
        );
    }
}

#[test]
fn cached_span_copies_charge_flat_and_owned_payload_before_return() {
    let doc = PdfDocument::from_bytes_with_limits(cached_text_file(), PdfImportLimits::default())
        .unwrap();
    let spans = doc.extract_spans(0).unwrap();
    let payload = std::mem::size_of_val(spans.as_slice())
        + spans
            .iter()
            .map(|span| {
                span.text.len()
                    + span.font_name.len()
                    + std::mem::size_of_val(span.char_widths.as_slice())
                    + std::mem::size_of_val(span.char_x_offsets.as_slice())
            })
            .sum::<usize>();
    let used = doc.import_budget().unwrap().decoded_bytes();
    let copied = doc.extract_spans(0).unwrap();
    assert_eq!(
        copied.iter().map(|s| &s.text).collect::<Vec<_>>(),
        spans.iter().map(|s| &s.text).collect::<Vec<_>>()
    );
    assert_eq!(
        doc.import_budget().unwrap().decoded_bytes() - used,
        payload as u64
    );

    for cap in [
        used + payload as u64 - 1,
        used + payload as u64,
        used + payload as u64 + 1,
    ] {
        let limited = PdfDocument::from_bytes_with_limits(
            cached_text_file(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..PdfImportLimits::default()
            },
        )
        .unwrap();
        limited.extract_spans(0).unwrap();
        let result = limited.extract_spans(0);
        if cap < used + payload as u64 {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("document decoded bytes"))
            ));
            assert_eq!(limited.import_budget().unwrap().decoded_bytes(), used);
            assert!(limited.extract_spans(0).is_err());
        } else {
            assert_eq!(result.unwrap().len(), spans.len());
        }
    }
}

#[test]
fn cached_character_copies_charge_owned_font_names_before_return() {
    let doc = PdfDocument::from_bytes_with_limits(cached_text_file(), PdfImportLimits::default())
        .unwrap();
    let chars = doc.extract_chars(0).unwrap();
    let payload = std::mem::size_of_val(chars.as_slice())
        + chars
            .iter()
            .map(|character| character.font_name.len())
            .sum::<usize>();
    let used = doc.import_budget().unwrap().decoded_bytes();
    assert_eq!(
        doc.extract_chars(0)
            .unwrap()
            .iter()
            .map(|c| c.char)
            .collect::<String>(),
        "CACHE"
    );
    assert_eq!(
        doc.import_budget().unwrap().decoded_bytes() - used,
        payload as u64
    );
    for cap in [
        used + payload as u64 - 1,
        used + payload as u64,
        used + payload as u64 + 1,
    ] {
        let limited = PdfDocument::from_bytes_with_limits(
            cached_text_file(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..PdfImportLimits::default()
            },
        )
        .unwrap();
        limited.extract_chars(0).unwrap();
        let result = limited.extract_chars(0);
        if cap < used + payload as u64 {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("document decoded bytes"))
            ));
            assert_eq!(limited.import_budget().unwrap().decoded_bytes(), used);
        } else {
            assert_eq!(result.unwrap().len(), chars.len());
        }
    }
}

fn raw_image_file(inline: bool) -> Vec<u8> {
    let mut parts = if inline {
        objects(b"BI /W 2 /H 1 /BPC 8 /CS /RGB ID ABCDEF EI", "", "")
    } else {
        objects(b"/Im Do", "", "")
    };
    if !inline {
        parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
        parts.push(b"<< /Type /XObject /Subtype /Image /Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Length 6 >>\nstream\nABCDEF\nendstream".to_vec());
    }
    object_file(&parts, "")
}

#[test]
fn image_raw_copy_is_bounded_and_sticky_for_inline_and_xobjects() {
    for inline in [false, true] {
        let doc =
            PdfDocument::from_bytes_with_limits(raw_image_file(inline), PdfImportLimits::default())
                .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        assert_eq!(handles.len(), 1);
        let metadata_copy = if inline {
            0
        } else {
            let probe = PdfDocument::from_bytes_with_limits(
                raw_image_file(false),
                PdfImportLimits::default(),
            )
            .unwrap();
            probe.page_image_handles(0).unwrap();
            let before = probe.import_budget().unwrap().decoded_bytes();
            probe
                .load_object(pdf_oxide::object::ObjectRef::new(6, 0))
                .unwrap();
            probe.import_budget().unwrap().decoded_bytes() - before
        };
        let copy_bytes = metadata_copy + 6;
        let used = doc.import_budget().unwrap().decoded_bytes();
        assert_eq!(handles[0].raw_compressed_bytes().unwrap(), b"ABCDEF");
        assert_eq!(
            doc.import_budget().unwrap().decoded_bytes() - used,
            copy_bytes
        );
        doc.import_budget()
            .unwrap()
            .reject("test raw image failure");
        assert!(matches!(
            handles[0].raw_compressed_bytes(),
            Err(Error::ImportResourceLimit("test raw image failure"))
        ));
        assert!(matches!(
            handles[0].decode(),
            Err(Error::ImportResourceLimit("test raw image failure"))
        ));
        for cap in [
            used + copy_bytes - 1,
            used + copy_bytes,
            used + copy_bytes + 1,
        ] {
            let limited = PdfDocument::from_bytes_with_limits(
                raw_image_file(inline),
                PdfImportLimits {
                    decoded_bytes: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            let handles = limited.page_image_handles(0).unwrap();
            let result = handles[0].raw_compressed_bytes();
            if cap < used + copy_bytes {
                assert!(matches!(
                    result,
                    Err(Error::ImportResourceLimit("document decoded bytes"))
                ));
                assert!(handles[0].raw_compressed_bytes().is_err());
            } else {
                assert_eq!(result.unwrap(), b"ABCDEF");
            }
        }
    }
}

#[test]
fn text_copy_buffer_limit_applies_before_cache_insertion_or_return() {
    let doc = PdfDocument::from_bytes_with_limits(cached_text_file(), PdfImportLimits::default())
        .unwrap();
    let spans = doc.extract_spans(0).unwrap();
    let chars = doc.extract_chars(0).unwrap();
    let span_bytes = std::mem::size_of_val(spans.as_slice())
        + spans
            .iter()
            .map(|span| {
                span.text.len()
                    + span.font_name.len()
                    + std::mem::size_of_val(span.char_widths.as_slice())
                    + std::mem::size_of_val(span.char_x_offsets.as_slice())
            })
            .sum::<usize>();
    let char_bytes = std::mem::size_of_val(chars.as_slice())
        + chars
            .iter()
            .map(|character| character.font_name.len())
            .sum::<usize>();
    for (mode, bytes) in [("spans", span_bytes), ("chars", char_bytes)] {
        for cap in [bytes - 1, bytes, bytes + 1] {
            let limited = PdfDocument::from_bytes_with_limits(
                cached_text_file(),
                PdfImportLimits {
                    stream_bytes: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            let result = if mode == "spans" {
                limited.extract_spans(0).map(|_| ())
            } else {
                limited.extract_chars(0).map(|_| ())
            };
            if cap < bytes {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit("cache copy bytes"))),
                    "{mode}: {result:?}"
                );
                assert!(limited.extract_spans(0).is_err());
                assert!(limited.extract_chars(0).is_err());
            } else {
                result.unwrap();
            }
        }
    }
}

#[test]
fn shared_form_stream_preserves_paths_and_image_handles_across_modes() {
    let mut parts = objects(b"/Fm Do q 1 0 0 1 20 0 cm /Fm Do Q", "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Fm 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    let content = b"0 0 m 10 10 l S q 2 0 0 1 0 0 cm /Im Do Q";
    let mut form = format!("<< /Type /XObject /Subtype /Form /BBox [0 0 200 200] /Resources << /XObject << /Im 7 0 R >> >> /Length {} >>\nstream\n", content.len()).into_bytes();
    form.extend(content);
    form.extend(b"\nendstream");
    parts.push(form);
    // The legacy materialized API deliberately filters images smaller than 8x8.
    let pixels = vec![65; 8 * 8 * 3];
    let mut image = format!("<< /Type /XObject /Subtype /Image /Width 8 /Height 8 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Length {} >>\nstream\n", pixels.len()).into_bytes();
    image.extend(&pixels);
    image.extend(b"\nendstream");
    parts.push(image);
    let bytes = object_file(&parts, "");
    for limited in [false, true] {
        let doc = if limited {
            PdfDocument::from_bytes_with_limits(bytes.clone(), PdfImportLimits::default()).unwrap()
        } else {
            PdfDocument::from_bytes(bytes.clone()).unwrap()
        };
        for _ in 0..3 {
            assert_eq!(doc.extract_paths(0).unwrap().len(), 2);
            let handles = doc.page_image_handles(0).unwrap();
            assert_eq!(handles.len(), 2);
            for handle in handles {
                assert_eq!(handle.raw_compressed_bytes().unwrap(), pixels);
                let decoded = handle.decode().unwrap();
                assert_eq!(decoded.width(), 8);
                assert_eq!(decoded.height(), 8);
            }
            assert_eq!(doc.extract_images(0).unwrap().len(), 2);
            assert!(doc.extract_spans(0).unwrap().is_empty());
        }
        doc.check_import_budget().unwrap();
    }
}

#[test]
fn brotli_empty_output_does_not_bypass_decoder_scratch_budget() {
    // Node zlib.brotliCompressSync(Buffer.alloc(0)): no decoded output bytes.
    let shared = budget(4096, 1);
    assert!(matches!(
        decode(&[59], &["BrotliDecode"], &shared),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(shared.check().is_err());
}

#[test]
fn brotli_small_output_also_requires_preallocated_decoder_work() {
    let compressed = [27, 127, 0, 248, 37, 130, 34, 177, 64, 32, 3];
    let shared = budget(128, 128);
    assert!(matches!(
        decode(&compressed, &["BrotliDecode"], &shared),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(shared.check().is_err());
}

#[test]
fn brotli_scratch_allocation_and_cumulative_boundaries_are_preallocation_checks() {
    // 6.0.1 state.rs:530 allocates 1080 repr(C) {u16, u8} entries;
    // decode.rs:3312/3318 each allocate three times that, even for empty output.
    let initial = 1080 * 4;
    let largest = initial * 3;
    let total = initial * 7;
    for cap in [
        initial - 1,
        initial,
        initial + 1,
        largest - 1,
        largest,
        largest + 1,
    ] {
        let shared = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        let result = decode(&[59], &["BrotliDecode"], &shared);
        if cap < largest {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("codec allocation bytes"))
            ));
            assert_eq!(
                shared.codec_allocated_bytes(),
                if cap < initial { 0 } else { initial as u64 }
            );
        } else {
            assert!(result.unwrap().is_empty());
            assert_eq!(shared.codec_allocated_bytes(), total as u64);
        }
    }
    for cap in [total - 1, total, total + 1] {
        let shared = budget(4096, cap);
        let result = decode(&[59], &["BrotliDecode"], &shared);
        if cap < total {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("document decoded bytes"))
            ));
            assert_eq!(shared.codec_allocated_bytes(), (initial * 4) as u64);
        } else {
            assert!(result.unwrap().is_empty());
            assert_eq!(shared.codec_allocated_bytes(), total as u64);
        }
    }
    assert!(ImportBudget::new(PdfImportLimits {
        codec_allocation_bytes: 0,
        ..PdfImportLimits::default()
    })
    .is_err());
}

#[test]
fn brotli_partial_scratch_allocation_failures_are_fatal_without_panics() {
    let compressed = [27, 127, 0, 248, 37, 130, 34, 177, 64, 32, 3];
    let successful = budget(128, 512 * 1024);
    assert_eq!(
        decode(&compressed, &["BrotliDecode"], &successful).unwrap(),
        vec![65; 128]
    );
    let scratch = successful.codec_allocated_bytes();
    assert!(scratch > 1080 * 4);
    for cap in (1..=scratch + 128)
        .step_by(257)
        .chain([scratch + 127, scratch + 128, scratch + 129])
    {
        let shared = budget(128, cap);
        let result = decode(&compressed, &["BrotliDecode"], &shared);
        if cap < scratch + 128 {
            assert!(
                matches!(result, Err(Error::ImportResourceLimit(_))),
                "cap {cap}: {result:?}"
            );
            assert!(decode(&[59], &["BrotliDecode"], &shared).is_err());
        } else {
            assert_eq!(result.unwrap(), vec![65; 128]);
        }
    }
}

#[test]
fn brotli_multiple_output_chunks_preserve_bytes_and_reject_truncated_prefixes() {
    // Node zlib: quality=4, lgwin=14, 12288 bytes of (index % 67 + 32).
    let compressed = [
        225, 248, 127, 1, 32, 3, 243, 76, 117, 148, 206, 184, 49, 176, 19, 119, 2, 0, 128, 16, 140,
        160, 24, 78, 144, 20, 205, 176, 28, 47, 136, 146, 172, 168, 154, 110, 152, 150, 237, 184,
        158, 31, 132, 81, 156, 164, 89, 94, 148, 85, 221, 180, 93, 63, 140, 211, 188, 172, 219,
        126, 156, 215, 253, 188, 222, 159, 239, 239, 15, 128, 16, 140, 160, 6,
    ];
    let expected: Vec<u8> = (0..12288).map(|index| (index % 67 + 32) as u8).collect();
    assert_eq!(
        pdf_oxide::decoders::decode_stream(&compressed, &["BrotliDecode".into()]).unwrap(),
        expected
    );
    for cap in [expected.len() - 1, expected.len(), expected.len() + 1] {
        let shared = budget(cap, 1024 * 1024);
        let result = decode(&compressed, &["BrotliDecode"], &shared);
        if cap < expected.len() {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
        } else {
            assert_eq!(result.unwrap(), expected);
        }
    }
    for end in 0..compressed.len() {
        let shared = budget(expected.len(), 1024 * 1024);
        assert!(
            matches!(
                decode(&compressed[..end], &["BrotliDecode"], &shared),
                Err(Error::Decode(_))
            ),
            "prefix {end}"
        );
    }
    for allocation_cap in [4320, 12959, 12960, 16384, 32768, 65536] {
        let shared = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: allocation_cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        match decode(&compressed, &["BrotliDecode"], &shared) {
            Ok(bytes) => assert_eq!(bytes, expected),
            Err(Error::ImportResourceLimit(_)) => assert!(shared.check().is_err()),
            result => panic!("allocation cap {allocation_cap}: {result:?}"),
        }
    }
    assert_eq!(
        decode(
            &compressed,
            &["BrotliDecode"],
            &budget(expected.len(), 1024 * 1024)
        )
        .unwrap(),
        expected
    );
}

fn ccitt_image(
    width: u32,
    height: u32,
    rows: Option<u32>,
    bytes: &[u8],
) -> pdf_oxide::extractors::images::PdfImage {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    PdfImage::with_ccitt_params(
        width,
        height,
        ColorSpace::DeviceGray,
        1,
        ImageData::Raw {
            pixels: bytes.to_vec(),
            format: PixelFormat::Grayscale,
        },
        pdf_oxide::decoders::CcittParams {
            k: -1,
            columns: width,
            rows,
            ..Default::default()
        },
    )
}

#[test]
fn ccitt_rows_cannot_allocate_beyond_the_image_geometry() {
    let image = ccitt_image(8, 1, Some(32768), &[0x80]);
    assert!(image.to_png_bytes_with_budget(&budget(512, 4096)).is_err());
}

#[test]
fn ccitt_private_transition_storage_obeys_the_codec_budget() {
    let shared = ImportBudget::new(PdfImportLimits {
        codec_allocation_bytes: 1,
        ..Default::default()
    })
    .unwrap();
    assert!(matches!(
        ccitt_image(8, 1, Some(1), &[0x80]).to_png_bytes_with_budget(&shared),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(shared.check().is_err());
}

#[test]
fn ccitt_compressed_length_cannot_impersonate_uncompressed_gray_pixels() {
    // T.6 vertical(0) against the imaginary white reference produces a white row.
    let png = ccitt_image(1, 1, Some(1), &[0x80])
        .to_png_bytes_with_budget(&budget(4096, 2 * 1024 * 1024))
        .unwrap();
    let gray = image::load_from_memory(&png).unwrap().to_luma8();
    assert_eq!(gray.as_raw(), &[255]);
}

#[test]
fn ccitt_limited_import_does_not_turn_undecodable_data_into_blank_success() {
    assert!(ccitt_image(8, 1, Some(1), &[0])
        .to_png_bytes_with_budget(&budget(4096, 8192))
        .is_err());
}

fn ccitt_bits(bits: &str) -> Vec<u8> {
    bits.as_bytes()
        .chunks(8)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u8, |byte, (index, bit)| {
                byte | (u8::from(*bit == b'1') << (7 - index))
            })
        })
        .collect()
}

fn ccitt_rows(k: i64, aligned: bool) -> Vec<u8> {
    // T.4: white0=00110101, black8=000101, black3=10, white2=0111.
    let black = "00110101000101";
    let notch = "0011010110011110";
    let mut bits = String::new();
    for (row, runs) in [black, black, notch].into_iter().enumerate() {
        if aligned {
            while !bits.len().is_multiple_of(8) {
                bits.push('0');
            }
        }
        if k < 0 {
            bits.push_str(if row < 2 {
                "00100110101000101"
            } else {
                "0010011010110001011110"
            });
        } else if k == 0 {
            bits.push_str(runs);
        } else {
            bits.push_str("000000000001");
            if row == 1 {
                bits.push_str("011");
            }
            // tag 0, two V(0) modes copy black row
            else {
                bits.push('1');
                bits.push_str(runs);
            }
        }
    }
    ccitt_bits(&bits)
}

#[test]
fn ccitt_normal_group3_group4_alignment_and_polarity_preserve_pixels() {
    for k in [-1, 0, 1] {
        for aligned in [false, true] {
            let data = ccitt_rows(k, aligned);
            for inverted in [false, true] {
                let params = pdf_oxide::decoders::CcittParams {
                    k,
                    columns: 8,
                    rows: Some(3),
                    encoded_byte_align: aligned,
                    black_is_1: inverted,
                    ..Default::default()
                };
                let decoded = pdf_oxide::decoders::decode_ccitt_with_budget(
                    &data,
                    &params,
                    &budget(4096, 8192),
                )
                .unwrap();
                assert_eq!(decoded.data, [255, 255, 231], "k={k}, aligned={aligned}");
                assert_eq!(decoded.rows_decoded, 3);
                assert!(!decoded.recovered_partial);
                let mut image = ccitt_image(8, 3, Some(3), &data);
                image.set_ccitt_params(params);
                let png = image
                    .to_png_bytes_with_budget(&budget(4096, 2 * 1024 * 1024))
                    .unwrap();
                assert_eq!(
                    image::load_from_memory(&png).unwrap().to_luma8(),
                    image::load_from_memory(&image.to_png_bytes().unwrap())
                        .unwrap()
                        .to_luma8()
                );
                let actual = image::load_from_memory(&png).unwrap().to_luma8();
                let expected: Vec<u8> = [255u8, 255, 231]
                    .into_iter()
                    .flat_map(|byte| {
                        (0..8).map(move |column| {
                            if ((byte >> (7 - column)) & 1 != 0) ^ inverted {
                                0
                            } else {
                                255
                            }
                        })
                    })
                    .collect();
                assert_eq!(actual.as_raw(), &expected);
            }
        }
    }
}

#[test]
fn ccitt_packed_and_private_allocation_boundaries_are_checked_before_reserve() {
    let params = pdf_oxide::decoders::CcittParams {
        k: -1,
        columns: 8,
        rows: Some(3),
        ..Default::default()
    };
    let data = ccitt_rows(-1, false);
    for cap in [data.len() - 1, data.len(), data.len() + 1] {
        let shared = ImportBudget::new(PdfImportLimits {
            input_bytes: cap,
            ..Default::default()
        })
        .unwrap();
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap < data.len() {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("CCITT input bytes"))
            ));
            assert_eq!(shared.decoded_bytes(), 0);
        } else {
            assert_eq!(result.unwrap().data, [255, 255, 231]);
        }
    }
    for cap in [2, 3, 4] {
        let shared = budget(cap, 4096);
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap == 2 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(shared.decoded_bytes(), 0);
            assert_eq!(shared.codec_allocated_bytes(), 0);
        } else {
            assert_eq!(result.unwrap().data, [255, 255, 231]);
        }
    }
    // Two width=8 u16 transition buffers, three temporary packed rows and 3 output bytes.
    let total = 2 * 8 * 2 + 3 + 3;
    for cap in [total * 2 - 1, total * 2, total * 2 + 1] {
        let shared = budget(3, cap);
        assert_eq!(
            pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared)
                .unwrap()
                .data,
            [255, 255, 231]
        );
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap < total * 2 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(matches!(
                pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared),
                Err(Error::ImportResourceLimit(_))
            ));
        } else {
            assert_eq!(result.unwrap().data, [255, 255, 231]);
        }
    }
    for cap in [total - 1, total, total + 1] {
        let shared = budget(3, cap);
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap < total {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(shared.check().is_err());
        } else {
            assert_eq!(result.unwrap().data, [255, 255, 231]);
            assert_eq!(shared.decoded_bytes(), total as u64);
        }
    }
    for cap in [15, 16, 17] {
        let shared = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..Default::default()
        })
        .unwrap();
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap == 15 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(shared.codec_allocated_bytes(), 0);
        } else {
            assert_eq!(result.unwrap().data, [255, 255, 231]);
        }
    }
}

#[test]
fn ccitt_transition_growth_cannot_bypass_the_private_allocation_budget() {
    let data = ccitt_bits(&format!(
        "{}001100110000110111",
        "001001101010000110111".repeat(20)
    ));
    let params = pdf_oxide::decoders::CcittParams {
        k: -1,
        columns: 8,
        rows: Some(1),
        ..Default::default()
    };
    for cap in [31, 32, 63, 64, 127, 128] {
        let shared = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..Default::default()
        })
        .unwrap();
        let result = pdf_oxide::decoders::decode_ccitt_with_budget(&data, &params, &shared);
        if cap < 128 {
            assert!(
                matches!(result, Err(Error::ImportResourceLimit(_))),
                "cap={cap}"
            );
            assert!(shared.check().is_err());
        } else {
            assert_eq!(result.unwrap().data, [0]);
        }
    }
}

#[test]
fn ccitt_unknown_rows_use_image_height_and_wide_columns_do_not_wrap() {
    let data = ccitt_rows(-1, false);
    for rows in [None, Some(0)] {
        let png = ccitt_image(8, 3, rows, &data)
            .to_png_bytes_with_budget(&budget(4096, 2 * 1024 * 1024))
            .unwrap();
        let gray = image::load_from_memory(&png).unwrap().to_luma8();
        assert_eq!(
            gray.as_raw().iter().filter(|&&pixel| pixel == 255).count(),
            2
        );
    }
    assert!(matches!(
        ccitt_image(65536, 1, Some(1), &[0x80])
            .to_png_bytes_with_budget(&ImportBudget::new(Default::default()).unwrap()),
        Err(Error::Unsupported(_))
    ));
    for k in [-1, 0, 1] {
        let params = pdf_oxide::decoders::CcittParams {
            k,
            columns: 8,
            rows: Some(3),
            ..Default::default()
        };
        let data = ccitt_rows(k, false);
        for end in 0..data.len() {
            let shared = budget(4096, 8192);
            assert!(
                matches!(
                    pdf_oxide::decoders::decode_ccitt_with_budget(&data[..end], &params, &shared),
                    Err(Error::Decode(_))
                ),
                "k={k}, prefix={end}"
            );
        }
    }
}

fn ccitt_pdf(data: &[u8], decode_parms: &str, filter: &str, polarity: &str) -> Vec<u8> {
    let mut parts = objects(b"/Im Do", "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    let mut image = format!("<< /Subtype /Image /Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace /DeviceGray /Filter {filter} /DecodeParms {decode_parms} {polarity} /Length {} >>\nstream\n", data.len()).into_bytes();
    image.extend_from_slice(data);
    image.extend_from_slice(b"\nendstream");
    parts.push(image);
    parts.push(b"<< /K -1 /Columns 8 /Rows 3 >>".to_vec());
    object_file(&parts, "")
}

#[test]
fn ccitt_product_parameters_are_checked_before_narrowing_and_recover_per_document() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let data = ccitt_rows(-1, false);
    for parameters in [
        "<< /K -1 /Columns 8 /Rows -1 >>",
        "<< /K -1 /Columns 8 /Rows 4294967299 >>",
        "<< /K -1 /Columns 4294967304 /Rows 3 >>",
        "<< /K -1 /Columns 8 /Rows 32768 >>",
    ] {
        let result = parse_pdf_with_options(
            &ccitt_pdf(&data, parameters, "/CCITTFaxDecode", ""),
            &PdfImportOptions::default(),
        );
        assert_eq!(
            result.err().unwrap().code,
            "PDF_PARSE_FAILED",
            "{parameters}"
        );
    }
    let normal = parse_pdf_with_options(
        &ccitt_pdf(&data, "7 0 R", "/CCITTFaxDecode", ""),
        &PdfImportOptions::default(),
    )
    .unwrap();
    assert_eq!(normal.document.bin_data_content.len(), 1);
    assert!(!normal.metadata.parser_allocation_bounds_proven);
}

#[test]
fn ccitt_filter_parameters_keep_chain_index_and_decode_polarity() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let data = ccitt_rows(-1, false);
    let encoded = data
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    for polarity in ["", "/Decode [1 0]"] {
        let direct = parse_pdf_with_options(
            &ccitt_pdf(&data, "7 0 R", "/CCITTFaxDecode", polarity),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let chained = parse_pdf_with_options(
            &ccitt_pdf(
                encoded.as_bytes(),
                "[null 7 0 R]",
                "[/ASCIIHexDecode /CCITTFaxDecode]",
                polarity,
            ),
            &PdfImportOptions::default(),
        )
        .unwrap();
        assert_eq!(direct.document.bin_data_content.len(), 1);
        assert_eq!(chained.document.bin_data_content.len(), 1);
        assert_eq!(
            direct.document.bin_data_content[0].data.load(),
            chained.document.bin_data_content[0].data.load()
        );
    }
}

fn indexed_image_pdf(data: &[u8], entries: &str, extra_object: &[u8]) -> Vec<u8> {
    let mut parts = objects(b"/Im Do", "", "");
    parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    let mut image = format!(
        "<< /Subtype /Image {entries} /Length {} >>\nstream\n",
        data.len()
    )
    .into_bytes();
    image.extend_from_slice(data);
    image.extend_from_slice(b"\nendstream");
    parts.push(image);
    parts.push(extra_object.to_vec());
    object_file(&parts, "")
}

#[test]
fn indexed_ccitt_decodes_before_palette_lookup_and_preserves_polarity() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    // ISO 32000-1 7.4.6 + 8.6.6.3: fax bits become sample indices first,
    // then /Decode maps the indices into this explicitly coloured lookup.
    for k in [-1, 0, 1] {
        for black_is_one in [false, true] {
            for invert in [false, true] {
                let data = ccitt_rows(k, false);
                let entries = format!(
                    "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 <0A141EC8D2DC>] /Filter /CCITTFaxDecode /DecodeParms 7 0 R {}",
                    if invert { "/Decode [1 0]" } else { "" }
                );
                let params = format!("<< /K {k} /Columns 8 /Rows 3 /BlackIs1 {black_is_one} >>");
                let source = indexed_image_pdf(&data, &entries, params.as_bytes());
                let backend =
                    PdfDocument::from_bytes_with_limits(source.clone(), Default::default())
                        .unwrap();
                let handles = backend.page_image_handles(0).unwrap();
                let decoded = handles[0].decode().unwrap();
                assert!(!decoded.samples_are_raw());
                assert_eq!(decoded.bits_per_component(), 8);
                assert_eq!(decoded.decode_folded_in(), invert);
                let imported =
                    parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
                assert_eq!(imported.document.bin_data_content.len(), 1);
                let bytes = imported.document.bin_data_content[0].data.load();
                let rgb = image::load_from_memory(&bytes).unwrap().to_rgb8();
                assert_eq!(rgb.dimensions(), (8, 3));
                for (x, y, pixel) in rgb.enumerate_pixels() {
                    let black = !(y == 2 && (3..5).contains(&x));
                    let index_one = (black == black_is_one) ^ invert;
                    assert_eq!(
                        pixel.0,
                        if index_one {
                            [200, 210, 220]
                        } else {
                            [10, 20, 30]
                        },
                        "K={k}, BlackIs1={black_is_one}, invert={invert}, ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn indexed_ccitt_rejects_invalid_geometry_and_truncated_streams() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let entries = "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 <000000FFFFFF>] /Filter /CCITTFaxDecode /DecodeParms 7 0 R";
    for (data, params) in [
        (ccitt_rows(-1, false), "<< /K -1 /Columns 8 /Rows 32768 >>"),
        (
            ccitt_rows(-1, false),
            "<< /K -1 /Columns 4294967304 /Rows 3 >>",
        ),
        (vec![0, 0, 0], "<< /K -1 /Columns 8 /Rows 3 >>"),
    ] {
        assert!(
            parse_pdf_with_options(
                &indexed_image_pdf(&data, entries, params.as_bytes()),
                &PdfImportOptions::default()
            )
            .is_err(),
            "{params}"
        );
    }
    assert!(parse_pdf_with_options(
        &indexed_image_pdf(
            &ccitt_rows(-1, false),
            entries,
            b"<< /K -1 /Columns 8 /Rows 3 >>"
        ),
        &PdfImportOptions::default(),
    )
    .is_ok());
}

#[test]
fn indexed_rgb_expansion_obeys_allocation_cap_before_reserve() {
    let source = indexed_image_pdf(
        &[0, 0, 0],
        "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 <0A141EC8D2DC>]",
        b"null",
    );
    // The output is 8 * 3 pixels, each with three independently specified
    // channels. A decoder cap below 72 must reject that reservation.
    for cap in [71, 72, 74, 75, 76] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                codec_allocation_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        assert_eq!(handles.len(), 1);
        let result = handles[0].decode();
        if cap < 72 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(doc.check_import_budget().is_err());
            assert!(handles[0].decode().is_err());
        } else {
            let decoded = result.unwrap();
            let png = decoded.to_png_bytes_with_budget(doc.import_budget().unwrap());
            // Decoding fits; the subsequent PNG compressor has a separate,
            // larger private allocation, which this deliberately tiny cap rejects.
            assert!(matches!(png, Err(Error::ImportResourceLimit(_))));
        }
    }
}

#[test]
fn indexed_palette_and_expansion_share_the_document_budget() {
    let source = indexed_image_pdf(
        &[0, 0, 0],
        "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 <0A141EC8D2DC>]",
        b"null",
    );
    let normal = PdfDocument::from_bytes_with_limits(source.clone(), Default::default()).unwrap();
    let handles = normal.page_image_handles(0).unwrap();
    handles[0].decode().unwrap();
    let consumed = normal.import_budget().unwrap().decoded_bytes();
    for cap in [consumed - 1, consumed, consumed + 1] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        let result = handles[0].decode();
        if cap < consumed {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
        } else {
            assert!(result.is_ok());
        }
        // A repeated decode is additional work in the same request.
        assert!(matches!(
            handles[0].decode(),
            Err(Error::ImportResourceLimit(_))
        ));
        assert!(doc.check_import_budget().is_err());
    }
}

#[test]
fn indexed_palette_validation_and_decode_mapping_preserve_sample_meaning() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    for (data, bpc, lookup, hival, decode, expected) in [
        (
            vec![0b0001_1000],
            2,
            "0A141E28",
            3,
            "[3 0]",
            vec![40, 30, 20],
        ),
        (vec![0, 128, 255], 8, "0A141E", 2, "[0 2]", vec![10, 20, 30]),
        (vec![0x08, 0xF0], 4, "0A141E", 2, "[2 0]", vec![30, 20, 10]),
    ] {
        let entries = format!(
            "/Width 3 /Height 1 /BitsPerComponent {bpc} /ColorSpace [/Indexed /DeviceGray {hival} <{lookup}>] /Decode {decode}"
        );
        let imported = parse_pdf_with_options(
            &indexed_image_pdf(&data, &entries, b"null"),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let pixels = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_luma8();
        assert_eq!(pixels.as_raw(), &expected);
    }
    for color_space in [
        "[/Indexed /DeviceRGB 256 <000000FFFFFF>]",
        "[/Indexed /DeviceRGB -1 <000000FFFFFF>]",
        "[/Indexed /DeviceRGB 1 <000000FFFF>]",
    ] {
        let entries = format!("/Width 1 /Height 1 /BitsPerComponent 1 /ColorSpace {color_space}");
        assert!(parse_pdf_with_options(
            &indexed_image_pdf(&[0], &entries, b"null"),
            &PdfImportOptions::default()
        )
        .is_err());
    }
}

#[test]
fn indexed_stream_palette_and_filter_chain_keep_bounds_and_pixels() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    let data = ccitt_rows(-1, false);
    let encoded = data.iter().map(|b| format!("{b:02X}")).collect::<String>();
    let entries = "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 <0A141EC8D2DC>] /Filter [/ASCIIHexDecode /CCITTFaxDecode] /DecodeParms [null 7 0 R]";
    let imported = parse_pdf_with_options(
        &indexed_image_pdf(
            encoded.as_bytes(),
            entries,
            b"<< /K -1 /Columns 8 /Rows 3 >>",
        ),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let pixels = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
        .unwrap()
        .to_rgb8();
    assert_eq!(pixels.get_pixel(3, 2).0, [200, 210, 220]);
    assert_eq!(pixels.get_pixel(0, 0).0, [10, 20, 30]);

    let entries =
        "/Width 8 /Height 3 /BitsPerComponent 1 /ColorSpace [/Indexed /DeviceRGB 1 7 0 R]";
    for size in [6, 8192] {
        let palette = if size == 6 {
            vec![10, 20, 30, 200, 210, 220]
        } else {
            vec![0; size]
        };
        let compressed = flate(&palette);
        let mut stream = format!(
            "<< /Length {} /Filter /FlateDecode >>\nstream\n",
            compressed.len()
        )
        .into_bytes();
        stream.extend_from_slice(&compressed);
        stream.extend_from_slice(b"\nendstream");
        let doc = PdfDocument::from_bytes_with_limits(
            indexed_image_pdf(&[0, 0, 0], entries, &stream),
            PdfImportLimits {
                stream_bytes: 4096,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        let decoded = handles[0].decode();
        if size == 6 {
            let png = decoded
                .unwrap()
                .to_png_bytes_with_budget(doc.import_budget().unwrap())
                .unwrap();
            assert!(image::load_from_memory(&png)
                .unwrap()
                .to_rgb8()
                .pixels()
                .all(|p| p.0 == [10, 20, 30]));
        } else {
            assert!(matches!(decoded, Err(Error::ImportResourceLimit(_))));
            assert!(handles[0].decode().is_err());
        }
    }
}

#[test]
fn inline_image_decode_keeps_the_request_stream_budget() {
    // Inline BI/ID/EI and a referenced Image XObject encode the same samples.
    // The former must not turn a bounded content stream into unbound image
    // bytes when it constructs the synthetic XObject used by decode().
    for inline in [false, true] {
        for length in [6, 4095, 4096, 4097] {
            let samples = vec![65; length];
            let compressed = flate(&samples);
            let source = if inline {
                let hex = compressed
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<String>();
                let content = format!("BI /W 2 /H 1 /BPC 8 /CS /RGB /F [/AHx /Fl] ID {hex}> EI");
                object_file(&objects(content.as_bytes(), "", ""), "")
            } else {
                indexed_image_pdf(
                    &compressed,
                    "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Filter /FlateDecode",
                    b"null",
                )
            };
            let doc = PdfDocument::from_bytes_with_limits(
                source,
                PdfImportLimits {
                    stream_bytes: 4096,
                    ..Default::default()
                },
            )
            .unwrap();
            let handles = doc.page_image_handles(0).unwrap();
            assert_eq!(handles.len(), 1);
            let result = handles[0].decode();
            if length > 4096 {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit(_))),
                    "inline={inline}, decoded length={length}"
                );
                assert!(doc.check_import_budget().is_err());
                assert!(handles[0].decode().is_err());
            } else {
                let image = result.unwrap();
                if length == 6 {
                    let png = image
                        .to_png_bytes_with_budget(doc.import_budget().unwrap())
                        .unwrap();
                    assert!(image::load_from_memory(&png)
                        .unwrap()
                        .to_rgb8()
                        .pixels()
                        .all(|p| p.0 == [65, 65, 65]));
                }
            }
        }
    }
}

#[test]
fn sample_conversion_reserves_with_the_request_codec_limit() {
    // ISO 32000-1 Table 89 / 8.9.5.2: each row has 1,025 gray samples.
    // All three transformations must reserve exactly that output, before
    // writing it. The LUT (256 bytes) is smaller than each tested limit.
    for (bpc, data, extra, expected) in [
        (1, vec![0; 129], "", 0),
        (8, vec![0; 1025], "/Decode [1 0]", 255),
        (16, vec![0x80; 2050], "", 128),
    ] {
        let entries = format!(
            "/Width 1025 /Height 1 /BitsPerComponent {bpc} /ColorSpace /DeviceGray {extra}"
        );
        for cap in [1024, 1025, 1026] {
            let doc = PdfDocument::from_bytes_with_limits(
                indexed_image_pdf(&data, &entries, b"null"),
                PdfImportLimits {
                    codec_allocation_bytes: cap,
                    ..Default::default()
                },
            )
            .unwrap();
            let handles = doc.page_image_handles(0).unwrap();
            let result = handles[0].decode();
            if cap < 1025 {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit(_))),
                    "bpc={bpc}, cap={cap}"
                );
                assert!(handles[0].decode().is_err());
            } else {
                let image = result.unwrap();
                let pdf_oxide::extractors::images::ImageData::Raw { pixels, .. } = image.data()
                else {
                    panic!("raw samples expected")
                };
                assert_eq!(pixels, &vec![expected; 1025]);
                assert_eq!(image.bits_per_component(), 8);
            }
        }
    }
}

#[test]
fn sample_lut_and_output_share_cumulative_request_accounting() {
    let source = indexed_image_pdf(
        &[0b1010_1000],
        "/Width 2 /Height 1 /BitsPerComponent 1 /ColorSpace /DeviceRGB",
        b"null",
    );
    for cap in [767, 768, 769] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                codec_allocation_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        let result = handles[0].decode();
        if cap == 767 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
        } else {
            let image = result.unwrap();
            let pdf_oxide::extractors::images::ImageData::Raw { pixels, .. } = image.data() else {
                panic!("raw expected")
            };
            assert_eq!(pixels, &[255, 0, 255, 0, 255, 0]);
            // Three 256-entry component tables plus six transformed samples.
            assert_eq!(
                doc.import_budget().unwrap().codec_allocated_bytes(),
                768 + 6
            );
        }
    }
    let doc =
        PdfDocument::from_bytes_with_limits(source.clone(), PdfImportLimits::default()).unwrap();
    doc.page_image_handles(0).unwrap()[0].decode().unwrap();
    let consumed = doc.import_budget().unwrap().decoded_bytes();
    for cap in [consumed - 1, consumed, consumed + 1] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        assert_eq!(handles[0].decode().is_ok(), cap >= consumed);
        assert!(matches!(
            handles[0].decode(),
            Err(Error::ImportResourceLimit(_))
        ));
    }
}

#[test]
fn limited_sample_geometry_and_decode_do_not_silently_fabricate_pixels() {
    for (data, bpc, extra) in [
        (vec![], 1, ""),
        (vec![0], 16, ""),
        (vec![0], 8, "/Decode [1 0]"),
        (vec![0], 257, ""),
        (vec![0], -255, ""),
        (vec![0], 1, "/Decode [0]"),
        (vec![0], 1, "/Decode [/bad 1]"),
    ] {
        let entries =
            format!("/Width 2 /Height 1 /BitsPerComponent {bpc} /ColorSpace /DeviceGray {extra}");
        let doc = PdfDocument::from_bytes_with_limits(
            indexed_image_pdf(&data, &entries, b"null"),
            PdfImportLimits::default(),
        )
        .unwrap();
        assert!(
            doc.page_image_handles(0).unwrap()[0].decode().is_err(),
            "bpc={bpc}, extra={extra}"
        );
    }
    // Non-byte-aligned rows are individually padded, and indirect Decode is
    // still the same mapping. Padding bits never become next-row samples.
    let source = indexed_image_pdf(
        &[0x18, 0xE4],
        "/Width 3 /Height 2 /BitsPerComponent 2 /ColorSpace /DeviceGray /Decode 7 0 R",
        b"[1 0]",
    );
    let doc = PdfDocument::from_bytes_with_limits(source, PdfImportLimits::default()).unwrap();
    let image = doc.page_image_handles(0).unwrap()[0].decode().unwrap();
    let pdf_oxide::extractors::images::ImageData::Raw { pixels, .. } = image.data() else {
        panic!("raw expected")
    };
    assert_eq!(pixels, &[255, 170, 85, 0, 85, 170]);
    assert!(image.decode_folded_in());

    let source = indexed_image_pdf(
        &[0x08, 0xF0, 0xF8, 0x00],
        "/Width 3 /Height 2 /BitsPerComponent 4 /ColorSpace /DeviceGray",
        b"null",
    );
    for limited in [false, true] {
        let doc = if limited {
            PdfDocument::from_bytes_with_limits(source.clone(), PdfImportLimits::default())
        } else {
            PdfDocument::from_bytes(source.clone())
        }
        .unwrap();
        let image = doc.page_image_handles(0).unwrap()[0].decode().unwrap();
        let pdf_oxide::extractors::images::ImageData::Raw { pixels, .. } = image.data() else {
            panic!("raw expected")
        };
        assert_eq!(pixels, &[0, 136, 255, 255, 136, 0]);
    }
}

#[test]
fn limited_import_preserves_supported_masks_and_rejects_malformed_masks() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    // #7974 supports colour keys: the black source sample matches all three
    // original component ranges and is transparent; the red control remains
    // opaque (PDF 1.7 4.8.5).
    let keyed = indexed_image_pdf(
        &[0, 0, 0, 255, 0, 0],
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask [0 0 0 0 0 0]",
        b"null",
    );
    let imported = parse_pdf_with_options(&keyed, &PdfImportOptions::default()).unwrap();
    let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
        .unwrap()
        .to_rgba8();
    assert_eq!(rgba.dimensions(), (2, 1));
    assert_eq!(
        rgba.pixels().map(|p| p.0).collect::<Vec<_>>(),
        [[0, 0, 0, 0], [255, 0, 0, 255]]
    );
    {
        let (extra, mask_object) = ("/SMask 7 0 R", b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /DCTDecode /Length 1 >>\nstream\n\x80\nendstream".to_vec());
        let entries =
            format!("/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB {extra}");
        let source = indexed_image_pdf(&[0, 0, 0], &entries, &mask_object);
        let doc = PdfDocument::from_bytes_with_limits(source.clone(), PdfImportLimits::default())
            .unwrap();
        assert!(
            matches!(
                doc.page_image_handles(0).unwrap()[0].decode(),
                Err(Error::Image(_))
            ),
            "{extra}: DCT is supported, but a single 0x80 byte is not a JPEG"
        );
        let err = parse_pdf_with_options(&source, &PdfImportOptions::default())
            .err()
            .expect("mask cannot be silently discarded");
        assert_eq!(err.code, "PDF_PARSE_FAILED");
        assert_eq!(
            err.diagnostics.as_ref().unwrap().backend_kind,
            Some("image")
        );
        // Legacy image extraction still serves callers that apply masks
        // themselves; this restriction belongs to bounded handle import.
        assert!(PdfDocument::from_bytes(source)
            .unwrap()
            .page_image_handles(0)
            .unwrap()[0]
            .decode()
            .is_ok());
    }
    // Explicit 1-bit masks are now applied, rather than blanket-rejected.
    // PDF Reference 1.7 4.8.5: 0 paints under Decode [0 1], and [1 0]
    // reverses the stencil. These are decoded-resource contracts, not layout
    // baselines: verify actual PNG alpha independently of paint interpolation.
    for (width, height, mw, mh, bits, decode, compressed, expected) in [
        (
            8,
            1,
            8,
            1,
            vec![0x55],
            "",
            false,
            vec![255, 0, 255, 0, 255, 0, 255, 0],
        ),
        (
            8,
            1,
            8,
            1,
            vec![0x55],
            "/Decode [1 0]",
            false,
            vec![0, 255, 0, 255, 0, 255, 0, 255],
        ),
        (
            9,
            2,
            9,
            2,
            vec![0x00, 0x7f, 0xff, 0x80],
            "",
            false,
            [vec![255; 9], vec![0; 9]].concat(),
        ),
        (
            4,
            2,
            2,
            1,
            vec![0x40],
            "",
            false,
            vec![255, 255, 0, 0, 255, 255, 0, 0],
        ),
        (
            8,
            1,
            8,
            1,
            vec![0x55],
            "",
            true,
            vec![255, 0, 255, 0, 255, 0, 255, 0],
        ),
    ] {
        let samples = if compressed { flate(&bits) } else { bits };
        let filter = if compressed {
            "/Filter /FlateDecode"
        } else {
            ""
        };
        let mut mask = format!("<< /Subtype /Image /Width {mw} /Height {mh} /ImageMask true /BitsPerComponent 1 {decode} {filter} /Length {} >>\nstream\n", samples.len()).into_bytes();
        mask.extend(samples);
        mask.extend(b"\nendstream");
        let source = indexed_image_pdf(
            &[10u8, 20, 30].repeat(width * height),
            &format!(
                "/Width {width} /Height {height} /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask 7 0 R"
            ),
            &mask,
        );
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgba8();
        assert_eq!(rgba.pixels().map(|p| p.0[3]).collect::<Vec<_>>(), expected);
        assert!(rgba.pixels().all(|p| p.0[..3] == [10, 20, 30]));
    }
    // PDF Reference 1.7 tables 7.11/7.12: soft-mask samples are alpha,
    // independent of base colours. Matte is undone in the original space.
    for (label, base, base_entries, samples, mask_entries, expected) in [
        (
            "gray8 alpha",
            [20, 40, 60].repeat(4),
            "/Width 4 /Height 1 /ColorSpace /DeviceRGB",
            vec![0, 64, 128, 255],
            "/Width 4 /Height 1 /BitsPerComponent 8",
            vec![
                [20, 40, 60, 0],
                [20, 40, 60, 64],
                [20, 40, 60, 128],
                [20, 40, 60, 255],
            ],
        ),
        (
            "reversed alpha",
            [20, 40, 60].repeat(3),
            "/Width 3 /Height 1 /ColorSpace /DeviceRGB",
            vec![0, 128, 255],
            "/Width 3 /Height 1 /BitsPerComponent 8 /Decode [1 0]",
            vec![[20, 40, 60, 255], [20, 40, 60, 127], [20, 40, 60, 0]],
        ),
        (
            "black matte",
            vec![64, 32, 16],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![128],
            "/Width 1 /Height 1 /BitsPerComponent 8 /Matte [0 0 0]",
            vec![[128, 64, 32, 128]],
        ),
        (
            "white matte",
            vec![190, 200, 210],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![85],
            "/Width 1 /Height 1 /BitsPerComponent 8 /Matte [1 1 1]",
            vec![[60, 90, 120, 85]],
        ),
        (
            "gray matte",
            vec![64],
            "/Width 1 /Height 1 /ColorSpace /DeviceGray",
            vec![128],
            "/Width 1 /Height 1 /BitsPerComponent 8 /Matte [0]",
            vec![[128, 128, 128, 128]],
        ),
        (
            "different resolution",
            [20, 40, 60].repeat(4),
            "/Width 4 /Height 1 /ColorSpace /DeviceRGB",
            vec![0x40],
            "/Width 2 /Height 1 /BitsPerComponent 1",
            vec![
                [20, 40, 60, 0],
                [20, 40, 60, 0],
                [20, 40, 60, 255],
                [20, 40, 60, 255],
            ],
        ),
        (
            "higher resolution alpha",
            vec![20, 40, 60],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![0, 64, 128, 255],
            "/Width 4 /Height 1 /BitsPerComponent 8",
            vec![
                [20, 40, 60, 0],
                [20, 40, 60, 64],
                [20, 40, 60, 128],
                [20, 40, 60, 255],
            ],
        ),
        (
            "2-bit alpha",
            vec![20, 40, 60],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![0x1b],
            "/Width 4 /Height 1 /BitsPerComponent 2",
            vec![
                [20, 40, 60, 0],
                [20, 40, 60, 85],
                [20, 40, 60, 170],
                [20, 40, 60, 255],
            ],
        ),
        (
            "4-bit alpha",
            vec![20, 40, 60],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![0x05, 0xaf],
            "/Width 4 /Height 1 /BitsPerComponent 4",
            vec![
                [20, 40, 60, 0],
                [20, 40, 60, 85],
                [20, 40, 60, 170],
                [20, 40, 60, 255],
            ],
        ),
        (
            "zero alpha matte",
            vec![0, 0, 0],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB",
            vec![0],
            "/Width 1 /Height 1 /BitsPerComponent 8 /Matte [0 0 0]",
            vec![[0, 0, 0, 0]],
        ),
    ] {
        let packed = flate(&samples);
        let mut mask = format!("<< /Type /XObject /Subtype /Image /ColorSpace /DeviceGray {mask_entries} /Filter /FlateDecode /Length {} >>\nstream\n", packed.len()).into_bytes();
        mask.extend(packed);
        mask.extend(b"\nendstream");
        // A valid SMask overrides even an unsupported colour-key Mask.
        let source = indexed_image_pdf(
            &base,
            &format!("{base_entries} /BitsPerComponent 8 /Mask [0 0 0 0 0 0] /SMask 7 0 R"),
            &mask,
        );
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgba8();
        assert_eq!(
            rgba.pixels().map(|p| p.0).collect::<Vec<_>>(),
            expected,
            "{label}"
        );
    }
    for (label, entries, bytes) in [
        (
            "matte size mismatch",
            "/Width 2 /Height 1 /Matte [0 0 0]",
            vec![128, 128],
        ),
        (
            "matte wrong space arity",
            "/Width 1 /Height 1 /Matte [0]",
            vec![128],
        ),
        ("short samples", "/Width 2 /Height 1", vec![128]),
        (
            "stencil is not soft mask",
            "/Width 1 /Height 1 /ImageMask true",
            vec![128],
        ),
    ] {
        let mut mask = format!("<< /Subtype /Image /ColorSpace /DeviceGray /BitsPerComponent 8 {entries} /Length {} >>\nstream\n", bytes.len()).into_bytes();
        mask.extend(bytes);
        mask.extend(b"\nendstream");
        let source = indexed_image_pdf(
            &[20, 40, 60],
            "/Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 7 0 R",
            &mask,
        );
        let error = parse_pdf_with_options(&source, &PdfImportOptions::default())
            .err()
            .expect("invalid soft mask must not drop alpha");
        assert_eq!(error.code, "PDF_PARSE_FAILED", "{label}: {error:?}");
    }
    // Wrong lengths/Decode and dangling refs must fail, not return an opaque
    // base picture. Unsupported mask families remain explicit errors above.
    for (label, reference, mask) in [
        ("short samples", "7 0 R", b"<< /ImageMask true /Width 9 /Height 1 /BitsPerComponent 1 /Length 1 >>\nstream\n\x00\nendstream".as_slice()),
        ("invalid Decode", "7 0 R", b"<< /ImageMask true /Width 1 /Height 1 /BitsPerComponent 1 /Decode [0 0] /Length 1 >>\nstream\n\x00\nendstream".as_slice()),
        ("invalid sample depth", "7 0 R", b"<< /ImageMask true /Width 1 /Height 1 /BitsPerComponent false /Length 1 >>\nstream\n\x00\nendstream".as_slice()),
    ] {
        let source = indexed_image_pdf(&[10, 20, 30], &format!("/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask {reference}"), mask);
        let error = parse_pdf_with_options(&source, &PdfImportOptions::default()).err().expect("invalid mask must fail atomically");
        assert_eq!(error.code, "PDF_PARSE_FAILED", "{label}: {error:?}");
    }
    // A missing xref target deliberately forbids the unbounded object-recovery
    // scan under request limits; this is a sticky resource failure, not a
    // malformed sample error.
    let dangling = indexed_image_pdf(
        &[10, 20, 30],
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask 900 0 R",
        b"null",
    );
    let error = parse_pdf_with_options(&dangling, &PdfImportOptions::default())
        .err()
        .expect("dangling mask cannot trigger unbounded recovery");
    assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
    // The newly introduced RGBA output is reserved before allocation. RGB
    // needs 9 bytes; the composited 3-pixel RGBA result needs 12 bytes.
    let source = indexed_image_pdf(&[10u8, 20, 30].repeat(3), "/Width 3 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask 7 0 R", b"<< /ImageMask true /Width 3 /Height 1 /BitsPerComponent 1 /Length 1 >>\nstream\n\x00\nendstream");
    for cap in [11, 12] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                codec_allocation_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        let result = handles[0].decode();
        if cap == 11 {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(doc.check_import_budget().is_err());
        } else {
            assert!(result.is_ok());
        }
    }
    for inline in [false, true] {
        let source = if inline {
            object_file(
                &objects(b"BI /W 2 /H 1 /IM true /BPC 1 ID \x80 EI", "", ""),
                "",
            )
        } else {
            indexed_image_pdf(
                &[0x80],
                "/Width 2 /Height 1 /ImageMask true /BitsPerComponent 1",
                b"null",
            )
        };
        // #7974 supplies the current painting colour for both invocation
        // paths. Default Decode [0 1] leaves the set bit transparent and paints
        // the clear bit with the default black nonstroking colour.
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgba8();
        assert_eq!(rgba.dimensions(), (2, 1));
        assert_eq!(
            rgba.pixels().map(|p| p.0).collect::<Vec<_>>(),
            [[0, 0, 0, 0], [0, 0, 0, 255]],
            "inline={inline}"
        );
    }
    // Null dictionary values are absent, and a false ImageMask is ordinary
    // colour data (ISO 32000-1 7.3.9 and Table 89).
    let source = indexed_image_pdf(
        &[10, 20, 30],
        "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Mask null /SMask 7 0 R /ImageMask false",
        b"null",
    );
    let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    assert_eq!(
        image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgb8()
            .get_pixel(0, 0)
            .0,
        [10, 20, 30]
    );

    // A later malformed JPEG mask must fail the whole import, while
    // selecting just the ordinary first page remains valid. #8413 adds valid
    // DCT masks; it must not accept these raw bytes under a false codec label.
    let mut pages = objects(b"/Im Do", "", "");
    pages[1] = b"<< /Type /Pages /Count 2 /Kids [3 0 R 7 0 R] >>".to_vec();
    pages[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
    pages.push(b"<< /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Length 3 >>\nstream\nABC\nendstream".to_vec());
    pages.push(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 8 0 R >> >> /Contents 4 0 R >>".to_vec());
    pages.push(b"<< /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /SMask 9 0 R /Length 3 >>\nstream\nABC\nendstream".to_vec());
    pages.push(b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /DCTDecode /Length 1 >>\nstream\n\x80\nendstream".to_vec());
    let source = object_file(&pages, "");
    for source_pages in [None, Some(vec![1])] {
        let error = parse_pdf_with_options(
            &source,
            &PdfImportOptions {
                source_pages,
                ..PdfImportOptions::default()
            },
        )
        .err()
        .expect("whole selected import must fail");
        assert_eq!(error.code, "PDF_PARSE_FAILED");
        assert_eq!(
            error.diagnostics.as_ref().unwrap().backend_kind,
            Some("image")
        );
    }
    let normal = parse_pdf_with_options(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![0]),
            ..PdfImportOptions::default()
        },
    )
    .unwrap();
    assert_eq!(normal.metadata.pages.len(), 1);
    assert_eq!(normal.metadata.pages[0].source_page, 0);
}

#[test]
fn png_color_conversion_obeys_the_same_codec_allocation_limit() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    for (space, format, pixels) in [
        (
            ColorSpace::DeviceCMYK,
            PixelFormat::CMYK,
            vec![0, 0, 0, 0, 0, 255, 255, 0],
        ),
        (
            ColorSpace::ICCBased(1),
            PixelFormat::Grayscale,
            vec![0, 255],
        ),
    ] {
        let image = PdfImage::new(2, 1, space, 8, ImageData::Raw { pixels, format });
        // Color policy is unchanged in this resource patch. Compare the
        // unmodified legacy conversion; six bytes are independently required
        // for two RGB pixels, regardless of that colour policy.
        let expected = image::load_from_memory(&image.to_png_bytes().unwrap())
            .unwrap()
            .to_rgb8();
        for cap in [5, 6, 7, 8, 1024 * 1024] {
            let budget = ImportBudget::new(PdfImportLimits {
                codec_allocation_bytes: cap,
                ..Default::default()
            })
            .unwrap();
            let result = image.to_png_bytes_with_budget(&budget);
            if cap < 1024 * 1024 {
                assert!(
                    matches!(result, Err(Error::ImportResourceLimit(_))),
                    "{space:?} cap={cap}"
                );
                assert!(budget.check().is_err());
                // RGB conversion fits at six; the compressor must still be gated.
                assert_eq!(budget.codec_allocated_bytes(), if cap >= 6 { 6 } else { 0 });
            } else {
                assert_eq!(
                    image::load_from_memory(&result.unwrap()).unwrap().to_rgb8(),
                    expected
                );
                assert!(budget.codec_allocated_bytes() > 6 + 65536);
            }
        }
    }
}

#[test]
fn png_encoder_scratch_is_reserved_before_the_output_writer() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    let image = PdfImage::new(
        2,
        1,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels: vec![1, 2, 3, 4, 5, 6],
            format: PixelFormat::RGB,
        },
    );
    let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
    image.to_png_bytes_with_budget(&normal).unwrap();
    let scratch = usize::try_from(normal.codec_allocated_bytes()).unwrap();
    // The audited compressor owns a 32KiB window and a hash table, not merely
    // the seven-byte scanline. Its actual allocation is charged before init.
    assert!(scratch >= 65536);
    for cap in [scratch - 1, scratch, scratch + 1] {
        let budget = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..Default::default()
        })
        .unwrap();
        let result = image.to_png_bytes_with_budget(&budget);
        if cap < scratch {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(budget.codec_allocated_bytes(), 0);
            assert!(image.to_png_bytes_with_budget(&budget).is_err());
        } else {
            assert_eq!(
                image::load_from_memory(&result.unwrap())
                    .unwrap()
                    .to_rgb8()
                    .as_raw(),
                &[1, 2, 3, 4, 5, 6]
            );
            assert_eq!(budget.codec_allocated_bytes(), scratch as u64);
        }
    }
}

#[test]
fn jpeg_input_copy_is_charged_before_the_image_decoder_is_created() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
        .encode(
            &[12, 34, 56, 78, 90, 123],
            2,
            1,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    let input_bytes = jpeg.len();
    let image = PdfImage::new(2, 1, ColorSpace::DeviceRGB, 8, ImageData::Jpeg(jpeg));
    let expected = image::load_from_memory(&image.to_png_bytes().unwrap())
        .unwrap()
        .to_rgb8();
    for cap in [input_bytes - 1, input_bytes, input_bytes + 1, 1024 * 1024] {
        let budget = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..Default::default()
        })
        .unwrap();
        let result = image.to_png_bytes_with_budget(&budget);
        if cap < input_bytes {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(budget.codec_allocated_bytes(), 0);
            assert!(image.to_png_bytes_with_budget(&budget).is_err());
        } else if cap < 1024 * 1024 {
            // Input fits; a private JPEG or subsequent PNG reservation must fail.
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(budget.codec_allocated_bytes() >= input_bytes as u64);
        } else {
            assert_eq!(
                image::load_from_memory(&result.unwrap()).unwrap().to_rgb8(),
                expected
            );
            assert!(budget.codec_allocated_bytes() >= input_bytes as u64 + 6);
        }
    }
}

fn png_rgb_pair() -> pdf_oxide::extractors::images::PdfImage {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    PdfImage::new(
        2,
        1,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels: vec![1, 2, 3, 4, 5, 6],
            format: PixelFormat::RGB,
        },
    )
}

#[test]
fn png_output_and_cumulative_limits_include_container_and_repeated_work() {
    let image = png_rgb_pair();
    let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let encoded_bytes = image.to_png_bytes_with_budget(&normal).unwrap().len();
    let scratch = usize::try_from(normal.codec_allocated_bytes()).unwrap();
    for cap in [encoded_bytes - 1, encoded_bytes, encoded_bytes + 1] {
        let budget = budget(cap, 2 * 1024 * 1024);
        let result = image.to_png_bytes_with_budget(&budget);
        if cap < encoded_bytes {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(budget.codec_allocated_bytes(), scratch as u64);
            assert!(budget.check().is_err());
        } else {
            let png = result.unwrap();
            assert_eq!(png.len(), encoded_bytes);
            assert_eq!(
                image::load_from_memory(&png).unwrap().to_rgb8().as_raw(),
                &[1, 2, 3, 4, 5, 6]
            );
        }
    }
    // The RGB samples are already decoded and borrowed by the encoder. Only
    // actual compressor reservations and PNG output consume this request;
    // reuse the returned compressor block and charge each new encoded output.
    let one_request = scratch + encoded_bytes;
    assert_eq!(normal.decoded_bytes(), one_request as u64);
    for count in [1, 2] {
        for cap in [
            scratch + count * encoded_bytes - 1,
            scratch + count * encoded_bytes,
            scratch + count * encoded_bytes + 1,
        ] {
            let budget = budget(4096, cap);
            for _ in 1..count {
                image.to_png_bytes_with_budget(&budget).unwrap();
            }
            let result = image.to_png_bytes_with_budget(&budget);
            if cap < scratch + count * encoded_bytes {
                assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
                assert!(image.to_png_bytes_with_budget(&budget).is_err());
            } else {
                result.unwrap();
                assert_eq!(
                    budget.decoded_bytes(),
                    (scratch + count * encoded_bytes) as u64
                );
            }
        }
    }
}

#[test]
fn png_scanlines_and_blocks_preserve_every_sample_across_boundaries() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    for (width, height, channels) in [
        (16383, 1, 1),
        (16384, 1, 1),
        (5461, 1, 3),
        (5462, 1, 3),
        (7, 800, 3),
        (20000, 3, 3),
    ] {
        let mut seed = 7911u32;
        let pixels: Vec<u8> = (0..width * height * channels)
            .map(|i| {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                if width == 5462 || height > 1 {
                    (seed >> 24) as u8
                } else {
                    ((i * 37 + i / 251) % 256) as u8
                }
            })
            .collect();
        let image = PdfImage::new(
            width,
            height,
            if channels == 1 {
                ColorSpace::DeviceGray
            } else {
                ColorSpace::DeviceRGB
            },
            8,
            ImageData::Raw {
                pixels: pixels.clone(),
                format: if channels == 1 {
                    PixelFormat::Grayscale
                } else {
                    PixelFormat::RGB
                },
            },
        );
        let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let actual =
            image::load_from_memory(&image.to_png_bytes_with_budget(&budget).unwrap()).unwrap();
        assert_eq!((actual.width(), actual.height()), (width, height));
        let bytes = if channels == 1 {
            actual.to_luma8().into_raw()
        } else {
            actual.to_rgb8().into_raw()
        };
        assert_eq!(bytes, pixels);
        let tiny = ImportBudget::new(PdfImportLimits::default()).unwrap();
        png_rgb_pair().to_png_bytes_with_budget(&tiny).unwrap();
        // Streaming rows/IDAT chunks reuse exactly the same compressor memory.
        assert_eq!(budget.codec_allocated_bytes(), tiny.codec_allocated_bytes());
    }
}

#[test]
fn png_large_custom_limits_do_not_overflow_geometry_or_accept_short_pixels() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    for (width, height) in [(0, 1), (1, 0), (100_000, 100_000), (u32::MAX, u32::MAX)] {
        let image = PdfImage::new(
            width,
            height,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels: vec![],
                format: PixelFormat::RGB,
            },
        );
        let limits = ImportBudget::new(PdfImportLimits {
            stream_bytes: usize::MAX,
            decoded_bytes: usize::MAX as u64,
            codec_allocation_bytes: usize::MAX,
            ..Default::default()
        })
        .unwrap();
        assert!(image.to_png_bytes_with_budget(&limits).is_err());
        assert_eq!(limits.codec_allocated_bytes(), 0);
    }
}

#[test]
fn bounded_png_compresses_repeated_scanlines_without_changing_pixels() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    let pixels: Vec<u8> = (0..512 * 256 * 3).map(|i| (i % 256) as u8).collect();
    let image = PdfImage::new(
        512,
        256,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels: pixels.clone(),
            format: PixelFormat::RGB,
        },
    );
    let limits = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let png = image.to_png_bytes_with_budget(&limits).unwrap();
    assert_eq!(
        image::load_from_memory(&png).unwrap().to_rgb8().as_raw(),
        &pixels
    );
    // Identical scanlines are compressible. This detects the previous stored-
    // block expansion without pinning a compressor's exact byte representation.
    assert!(
        png.len() < pixels.len() / 2,
        "repeated rows expanded to {} bytes",
        png.len()
    );
}

#[test]
fn flate_private_allocation_is_guarded_even_for_empty_output() {
    // RFC 1950 empty stream, independently generated with Python zlib.compress(b"").
    let empty = [0x78, 0x9c, 3, 0, 0, 0, 0, 1];
    for cap in [1, 32767] {
        let shared = ImportBudget::new(PdfImportLimits {
            codec_allocation_bytes: cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        assert!(matches!(
            decode(&empty, &["FlateDecode"], &shared),
            Err(Error::ImportResourceLimit(_))
        ));
        assert_eq!(shared.codec_allocated_bytes(), 0);
        assert_eq!(shared.decoded_bytes(), 0);
        assert!(decode(&empty, &["FlateDecode"], &shared).is_err());
    }
    assert!(decode(&empty, &["FlateDecode"], &budget(1, 1024 * 1024))
        .unwrap()
        .is_empty());
}

#[test]
fn flate_requires_complete_stream_and_valid_checksum() {
    let expected = b"bounded Flate stream";
    let encoded = flate(expected);
    // A complete payload without its RFC 1950 checksum is not a complete stream.
    for missing in [1, 4, 5] {
        let result = decode(
            &encoded[..encoded.len() - missing],
            &["FlateDecode"],
            &budget(1024, 1024 * 1024),
        );
        assert!(
            result.is_err(),
            "accepted {missing} missing trailer/payload bytes: {result:?}"
        );
    }
    let mut damaged = encoded;
    *damaged.last_mut().unwrap() ^= 1;
    assert!(decode(&damaged, &["FlateDecode"], &budget(1024, 1024 * 1024)).is_err());
    assert_eq!(
        decode(
            &flate(expected),
            &["FlateDecode"],
            &budget(1024, 1024 * 1024)
        )
        .unwrap(),
        expected
    );
}

#[test]
fn flate_scratch_and_output_have_independent_exact_limits() {
    let bytes = flate(b"AB");
    let normal = budget(2, 1024 * 1024);
    assert_eq!(decode(&bytes, &["FlateDecode"], &normal).unwrap(), b"AB");
    let scratch = usize::try_from(normal.codec_allocated_bytes()).unwrap();
    // The pinned decoder requests the 32 KiB Deflate window plus state and
    // alignment. Exact size is ABI-dependent; do not pin a native struct size.
    assert!(scratch > 32768);
    assert_eq!(normal.decoded_bytes(), (scratch + 2) as u64);
    for cap in [scratch - 1, scratch, scratch + 1] {
        let shared = ImportBudget::new(PdfImportLimits {
            stream_bytes: 2,
            codec_allocation_bytes: cap,
            ..PdfImportLimits::default()
        })
        .unwrap();
        let result = decode(&bytes, &["FlateDecode"], &shared);
        if cap < scratch {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert_eq!(shared.codec_allocated_bytes(), 0);
            assert!(decode(&bytes, &["FlateDecode"], &shared).is_err());
        } else {
            assert_eq!(result.unwrap(), b"AB");
        }
    }
    for count in [1, 2, 3] {
        // The request owns one reusable codec block; each successful decode
        // still allocates and charges its distinct two-byte output.
        let required = scratch + count * 2;
        for cap in [required - 1, required, required + 1] {
            let shared = budget(2, cap);
            for _ in 1..count {
                assert_eq!(decode(&bytes, &["FlateDecode"], &shared).unwrap(), b"AB");
            }
            let result = decode(&bytes, &["FlateDecode"], &shared);
            if cap < required {
                assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
                assert!(decode(&bytes, &["FlateDecode"], &shared).is_err());
            } else {
                assert_eq!(result.unwrap(), b"AB");
                assert_eq!(shared.codec_allocated_bytes(), scratch as u64);
                assert_eq!(shared.decoded_bytes(), required as u64);
            }
        }
    }
}

#[test]
fn flate_raw_fallback_charges_both_attempts_and_never_retries_resource_failure() {
    // Independently encoded raw RFC 1951 stored block: BFINAL=1, LEN=2, NLEN.
    let raw = [1, 2, 0, 253, 255, b'A', b'B'];
    let normal = budget(2, 1024 * 1024);
    assert_eq!(
        decode(&flate(b"AB"), &["FlateDecode"], &normal).unwrap(),
        b"AB"
    );
    let scratch = usize::try_from(normal.codec_allocated_bytes()).unwrap();
    let total = scratch + 2;
    for cap in [scratch - 1, scratch, total - 1, total, total + 1] {
        let shared = budget(2, cap);
        let result = decode(&raw, &["FlateDecode"], &shared);
        if cap < total {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(decode(&raw, &["FlateDecode"], &shared).is_err());
            if cap < scratch {
                assert_eq!(shared.codec_allocated_bytes(), 0);
            }
            if cap == scratch {
                assert_eq!(shared.codec_allocated_bytes(), scratch as u64);
            }
        } else {
            assert_eq!(result.unwrap(), b"AB");
            assert_eq!(shared.codec_allocated_bytes(), scratch as u64);
            assert_eq!(shared.decoded_bytes(), total as u64);
        }
    }
    assert!(decode(
        &raw[..raw.len() - 1],
        &["FlateDecode"],
        &budget(2, 1024 * 1024)
    )
    .is_err());
    let mut trailing = raw.to_vec();
    trailing.extend_from_slice(b"\nunused PDF bytes");
    assert_eq!(
        decode(&trailing, &["FlateDecode"], &budget(2, 1024 * 1024)).unwrap(),
        b"AB"
    );
}

#[test]
fn flate_multichunk_and_filter_chain_preserve_bytes_and_account_scratch() {
    let pixels: Vec<u8> = (0..80000).map(|i| (i * 31 + i / 256) as u8).collect();
    let encoded = flate(&pixels);
    for cap in [pixels.len() - 1, pixels.len(), pixels.len() + 1] {
        let shared = budget(cap, 2 * 1024 * 1024);
        let result = decode(&encoded, &["FlateDecode"], &shared);
        if cap < pixels.len() {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            assert!(shared.check().is_err());
        } else {
            assert_eq!(result.unwrap(), pixels);
            assert_eq!(
                shared.decoded_bytes(),
                shared.codec_allocated_bytes() + pixels.len() as u64
            );
        }
    }
    let shared = budget(4, 1024 * 1024);
    assert_eq!(
        decode(&flate(b"4142"), &["FlateDecode", "ASCIIHexDecode"], &shared).unwrap(),
        b"AB"
    );
    assert_eq!(
        shared.decoded_bytes(),
        shared.codec_allocated_bytes() + 4 + 2
    );
}

#[test]
fn flate_private_limit_reaches_document_content_xref_and_image_consumers() {
    let limits = PdfImportLimits {
        // Smaller than the 32 KiB Deflate window plus its state, but larger
        // than every small content/image/palette buffer in these documents.
        codec_allocation_bytes: 32768,
        ..Default::default()
    };
    let compressed = flate(b"0 0 m 10 10 l S");
    let source = object_file(&objects(&compressed, "/Filter /FlateDecode", ""), "");
    let doc = PdfDocument::from_bytes_with_limits(source.clone(), limits).unwrap();
    assert!(matches!(
        doc.extract_paths(0),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(doc.check_import_budget().is_err());
    assert!(doc.extract_paths(0).is_err());
    let normal = PdfDocument::from_bytes_with_limits(source, Default::default()).unwrap();
    assert!(!normal.extract_paths(0).unwrap().is_empty());

    let xref = flate(&[0; 64]);
    let mut source = b"%PDF-1.5\n".to_vec();
    let offset = source.len();
    write!(source, "1 0 obj\n<< /Type /XRef /Size 1 /W [1 4 2] /Root 2 0 R /Length {} /Filter /FlateDecode >>\nstream\n", xref.len()).unwrap();
    source.extend(xref);
    write!(source, "\nendstream\nendobj\nstartxref\n{offset}\n%%EOF\n").unwrap();
    assert!(matches!(
        PdfDocument::from_bytes_with_limits(source, limits),
        Err(Error::ImportResourceLimit(_))
    ));

    for inline in [false, true] {
        let compressed = flate(&[10, 20, 30, 40, 50, 60]);
        let source = if inline {
            let hex = compressed
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>();
            let content = format!("BI /W 2 /H 1 /BPC 8 /CS /RGB /F [/AHx /Fl] ID {hex}> EI");
            object_file(&objects(content.as_bytes(), "", ""), "")
        } else {
            indexed_image_pdf(
                &compressed,
                "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Filter /FlateDecode",
                b"null",
            )
        };
        let doc = PdfDocument::from_bytes_with_limits(source.clone(), limits).unwrap();
        let handles = doc.page_image_handles(0).unwrap();
        assert_eq!(handles.len(), 1);
        assert!(matches!(
            handles[0].decode(),
            Err(Error::ImportResourceLimit(_))
        ));
        assert_eq!(doc.import_budget().unwrap().codec_allocated_bytes(), 0);
        assert!(handles[0].decode().is_err());
        let normal = PdfDocument::from_bytes_with_limits(source, Default::default()).unwrap();
        let image = normal.page_image_handles(0).unwrap()[0].decode().unwrap();
        let png = image
            .to_png_bytes_with_budget(normal.import_budget().unwrap())
            .unwrap();
        assert_eq!(
            image::load_from_memory(&png).unwrap().to_rgb8().into_raw(),
            [10, 20, 30, 40, 50, 60]
        );
    }
}

fn jpeg_allocation_pair() -> (
    pdf_oxide::extractors::images::PdfImage,
    pdf_oxide::extractors::images::PdfImage,
    usize,
) {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let jpeg = include_bytes!("../fixtures/issue7911_jpeg/RGB-0-0.jpg");
    let payload = [b"Exif\0\0".as_slice(), &vec![0; 32768]].concat();
    let mut tagged = jpeg[..2].to_vec();
    for _ in 0..4 {
        tagged.extend_from_slice(&[0xff, 0xe1]);
        tagged.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        tagged.extend_from_slice(&payload);
    }
    tagged.extend_from_slice(&jpeg[2..]);
    let extra_input = tagged.len() - jpeg.len();
    (
        PdfImage::new(
            19,
            17,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Jpeg(jpeg.to_vec()),
        ),
        PdfImage::new(19, 17, ColorSpace::DeviceRGB, 8, ImageData::Jpeg(tagged)),
        extra_input,
    )
}

#[test]
fn replaced_jpeg_metadata_reservations_remain_in_the_request_budget() {
    let (plain, tagged, extra_input) = jpeg_allocation_pair();
    let a = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let b = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let plain_png = plain.to_png_bytes_with_budget(&a).unwrap();
    let tagged_png = tagged.to_png_bytes_with_budget(&b).unwrap();
    assert_eq!(plain_png, tagged_png);
    // Four replaced APP1 EXIF payloads each allocate 32KiB. This lower bound
    // comes from the encoded marker lengths, independent of decoder internals.
    assert!(
        b.codec_allocated_bytes() - a.codec_allocated_bytes() >= extra_input as u64 + 4 * 32768
    );
}

#[test]
fn jpeg_metadata_cumulative_failure_is_sticky_and_request_local() {
    let (plain, tagged, extra_input) = jpeg_allocation_pair();
    let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let expected = plain.to_png_bytes_with_budget(&normal).unwrap();
    let limited = ImportBudget::new(PdfImportLimits {
        decoded_bytes: normal.decoded_bytes() + extra_input as u64 + 4096,
        ..Default::default()
    })
    .unwrap();
    assert!(matches!(
        tagged.to_png_bytes_with_budget(&limited),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(matches!(
        plain.to_png_bytes_with_budget(&limited),
        Err(Error::ImportResourceLimit(_))
    ));
    let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
    assert_eq!(tagged.to_png_bytes_with_budget(&fresh).unwrap(), expected);
}

#[test]
fn bounded_jpeg_baseline_progressive_subsampling_and_cmyk_preserve_pixels() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let cases: &[(&[u8], ColorSpace)] = &[
        (
            include_bytes!("../fixtures/issue7911_jpeg/CMYK-0-0.jpg"),
            ColorSpace::DeviceCMYK,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/CMYK-1-0.jpg"),
            ColorSpace::DeviceCMYK,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/L-0-0.jpg"),
            ColorSpace::DeviceGray,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/L-1-0.jpg"),
            ColorSpace::DeviceGray,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-0-0.jpg"),
            ColorSpace::DeviceRGB,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-0-1.jpg"),
            ColorSpace::DeviceRGB,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-0-2.jpg"),
            ColorSpace::DeviceRGB,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-1-0.jpg"),
            ColorSpace::DeviceRGB,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-1-1.jpg"),
            ColorSpace::DeviceRGB,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-1-2.jpg"),
            ColorSpace::DeviceRGB,
        ),
    ];
    for (bytes, color) in cases {
        let image = PdfImage::new(19, 17, color.clone(), 8, ImageData::Jpeg(bytes.to_vec()));
        let expected = image::load_from_memory(&image.to_png_bytes().unwrap())
            .unwrap()
            .to_rgb8();
        let budget = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let actual = image::load_from_memory(&image.to_png_bytes_with_budget(&budget).unwrap())
            .unwrap()
            .to_rgb8();
        assert_eq!(actual, expected, "{color:?}");
    }
}

fn cmyk_allocation_pair() -> (
    pdf_oxide::extractors::images::PdfImage,
    pdf_oxide::extractors::images::PdfImage,
) {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let source = include_bytes!("../fixtures/issue7911_jpeg/CMYK-0-0.jpg");
    let payload = [b"Exif\0\0".as_slice(), &vec![0; 32768]].concat();
    let mut tagged = source[..2].to_vec();
    for _ in 0..4 {
        tagged.extend_from_slice(&[0xff, 0xe1]);
        tagged.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        tagged.extend_from_slice(&payload);
    }
    tagged.extend_from_slice(&source[2..]);
    (
        PdfImage::new(
            19,
            17,
            ColorSpace::DeviceCMYK,
            8,
            ImageData::Jpeg(source.to_vec()),
        ),
        PdfImage::new(19, 17, ColorSpace::DeviceCMYK, 8, ImageData::Jpeg(tagged)),
    )
}

#[test]
fn cmyk_jpeg_replaced_metadata_is_charged_before_reservation() {
    let (plain, tagged) = cmyk_allocation_pair();
    let a = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let b = ImportBudget::new(PdfImportLimits::default()).unwrap();
    assert_eq!(
        plain.to_png_bytes_with_budget(&a).unwrap(),
        tagged.to_png_bytes_with_budget(&b).unwrap()
    );
    // The encoded four EXIF payloads each contain 32KiB. Replaced/free metadata
    // remains request work; this lower bound does not pin private struct sizes.
    assert!(b.codec_allocated_bytes() - a.codec_allocated_bytes() >= 4 * 32768);
    let limited = ImportBudget::new(PdfImportLimits {
        codec_allocation_bytes: 32768,
        ..Default::default()
    })
    .unwrap();
    assert!(matches!(
        tagged.to_png_bytes_with_budget(&limited),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(limited.check().is_err());
}

#[test]
fn cmyk_jpeg_budget_failure_is_sticky_and_request_local() {
    let (plain, tagged) = cmyk_allocation_pair();
    let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
    let expected = plain.to_png_bytes_with_budget(&normal).unwrap();
    let limited = ImportBudget::new(PdfImportLimits {
        decoded_bytes: normal.decoded_bytes() + 4096,
        ..Default::default()
    })
    .unwrap();
    assert!(matches!(
        tagged.to_png_bytes_with_budget(&limited),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(matches!(
        plain.to_png_bytes_with_budget(&limited),
        Err(Error::ImportResourceLimit(_))
    ));
    let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
    assert_eq!(tagged.to_png_bytes_with_budget(&fresh).unwrap(), expected);
}

#[test]
fn cmyk_pdf_xobject_propagates_private_decoder_limits() {
    use pdf_oxide::extractors::images::ImageData;
    let (_, tagged) = cmyk_allocation_pair();
    let ImageData::Jpeg(bytes) = tagged.data() else {
        unreachable!()
    };
    let source = indexed_image_pdf(
        bytes,
        "/Width 19 /Height 17 /BitsPerComponent 8 /ColorSpace /DeviceCMYK /Filter /DCTDecode",
        b"null",
    );
    let doc = PdfDocument::from_bytes_with_limits(
        source.clone(),
        PdfImportLimits {
            codec_allocation_bytes: 32768,
            ..Default::default()
        },
    )
    .unwrap();
    let im = doc.page_image_handles(0).unwrap()[0].decode().unwrap();
    assert!(matches!(
        im.to_png_bytes_with_budget(doc.import_budget().unwrap()),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(doc.check_import_budget().is_err());
    assert!(doc.page_image_handles(0).is_err());
    let fresh = PdfDocument::from_bytes_with_limits(source, Default::default()).unwrap();
    let im = fresh.page_image_handles(0).unwrap()[0].decode().unwrap();
    assert!(!im
        .to_png_bytes_with_budget(fresh.import_budget().unwrap())
        .unwrap()
        .is_empty());
}

#[test]
fn cmyk_large_workers_and_lossless_preserve_legacy_samples() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage};
    let cases: &[(&[u8], u32, u32)] = &[
        (
            include_bytes!("../fixtures/issue7911_jpeg/YCCK-large.jpg"),
            257,
            129,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/CMYK-large-progressive.jpg"),
            257,
            129,
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/CMYK-lossless-8.jpg"),
            19,
            17,
        ),
    ];
    for &(bytes, width, height) in cases {
        let im = PdfImage::new(
            width,
            height,
            ColorSpace::DeviceCMYK,
            8,
            ImageData::Jpeg(bytes.to_vec()),
        );
        let normal = ImportBudget::new(PdfImportLimits::default()).unwrap();
        let actual = im.to_png_bytes_with_budget(&normal).unwrap();
        let expected = im.to_png_bytes().unwrap();
        assert_eq!(
            image::load_from_memory(&actual).unwrap().to_rgb8(),
            image::load_from_memory(&expected).unwrap().to_rgb8()
        );
        let repeated = ImportBudget::new(PdfImportLimits {
            decoded_bytes: normal.decoded_bytes(),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(im.to_png_bytes_with_budget(&repeated).unwrap(), actual);
        assert!(matches!(
            im.to_png_bytes_with_budget(&repeated),
            Err(Error::ImportResourceLimit(_))
        ));
    }
}

fn copied_object_file(stream: bool) -> Vec<u8> {
    let mut value = format!("<< /Nested [({}) /Name [1 2 3]]", "a".repeat(4096)).into_bytes();
    if stream {
        value.extend(b" /Length 16384 >>\nstream\n");
        value.extend(vec![b'X'; 16384]);
        value.extend(b"\nendstream");
    } else {
        value.extend(b" >>");
    }
    object_file(&[b"<< /Type /Catalog >>".to_vec(), value], "")
}

#[test]
fn object_cache_owned_payload_is_accounted_on_warm_read() {
    let doc =
        PdfDocument::from_bytes_with_limits(copied_object_file(false), PdfImportLimits::default())
            .unwrap();
    let reference = pdf_oxide::object::ObjectRef::new(2, 0);
    let first = doc.load_object(reference).unwrap();
    let before = doc.import_budget().unwrap().decoded_bytes();
    let second = doc.load_object(reference).unwrap();
    assert_eq!(first, second);
    assert!(
        doc.import_budget().unwrap().decoded_bytes() - before >= 4096,
        "the owned 4096-byte string must be counted on every cache read"
    );
}

#[test]
fn object_cache_copy_exhaustion_is_sticky_and_request_local() {
    let input = copied_object_file(false);
    let reference = pdf_oxide::object::ObjectRef::new(2, 0);
    let doc =
        PdfDocument::from_bytes_with_limits(input.clone(), PdfImportLimits::default()).unwrap();
    let expected = doc.load_object(reference).unwrap();
    let before = doc.import_budget().unwrap().decoded_bytes();
    doc.load_object(reference).unwrap();
    let after = doc.import_budget().unwrap().decoded_bytes();
    assert!(after >= before + 4096);
    for cap in [after - 1, after, after + 1] {
        let limited = PdfDocument::from_bytes_with_limits(
            input.clone(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..PdfImportLimits::default()
            },
        )
        .unwrap();
        assert_eq!(limited.load_object(reference).unwrap(), expected);
        let result = limited.load_object(reference);
        if cap < after {
            assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            // Rejection precedes the whole copy, not a partially copied tree.
            assert_eq!(limited.import_budget().unwrap().decoded_bytes(), before);
            assert!(limited.load_object(reference).is_err());
        } else {
            assert_eq!(result.unwrap(), expected);
            assert_eq!(limited.import_budget().unwrap().decoded_bytes(), after);
        }
    }
    let fresh = PdfDocument::from_bytes_with_limits(input, PdfImportLimits::default()).unwrap();
    assert_eq!(fresh.load_object(reference).unwrap(), expected);
}

#[test]
fn object_cache_stream_payload_stays_shared_with_originating_budget() {
    use pdf_oxide::object::{Object, ObjectRef};
    let doc =
        PdfDocument::from_bytes_with_limits(copied_object_file(true), PdfImportLimits::default())
            .unwrap();
    let first = doc.load_object(ObjectRef::new(2, 0)).unwrap();
    let before = doc.import_budget().unwrap().decoded_bytes();
    let second = doc.load_object(ObjectRef::new(2, 0)).unwrap();
    let charged = doc.import_budget().unwrap().decoded_bytes() - before;
    assert!((4096..16384).contains(&charged));
    let (Object::Stream { data: a, .. }, Object::Stream { data: b, .. }) = (first, second) else {
        panic!("expected streams")
    };
    assert_eq!(a.as_ptr(), b.as_ptr());
    assert!(Arc::ptr_eq(a.budget().unwrap(), b.budget().unwrap()));
    assert!(Arc::ptr_eq(
        a.budget().unwrap(),
        doc.import_budget().unwrap()
    ));
}

fn color_resource_copy_file(inline: bool, padding: usize) -> Vec<u8> {
    let content = if inline {
        b"BI /W 1 /H 1 /BPC 8 /CS /CS0 ID \0 EI".as_slice()
    } else {
        b"/Im Do".as_slice()
    };
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend(content);
    stream.extend(b"\nendstream");
    let resources = format!(
        "<< /ColorSpace << /CS0 [/Indexed /DeviceRGB 1 <ff000000ff00>] /Unused ({}) >> /XObject << /Im 5 0 R >> >>",
        "p".repeat(padding)
    );
    object_file(&[
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 10 10] /Resources {resources} /Contents 4 0 R >>").into_bytes(),
        stream,
        b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /CS0 /Length 1 >>\nstream\n\0\nendstream".to_vec()
    ], "")
}

#[test]
fn color_resource_owned_payload_is_accounted_for_inline_and_xobject_handles() {
    for inline in [false, true] {
        let mut deltas = Vec::new();
        let mut pixels = Vec::new();
        for padding in [0, 4096] {
            let doc = PdfDocument::from_bytes_with_limits(
                color_resource_copy_file(inline, padding),
                PdfImportLimits::default(),
            )
            .unwrap();
            assert_eq!(doc.page_image_handles(0).unwrap().len(), 1);
            let before = doc.import_budget().unwrap().decoded_bytes();
            let handles = doc.page_image_handles(0).unwrap();
            deltas.push(doc.import_budget().unwrap().decoded_bytes() - before);
            let image = handles[0].decode().unwrap();
            pixels.push(
                image
                    .to_png_bytes_with_budget(doc.import_budget().unwrap())
                    .unwrap(),
            );
        }
        assert!(deltas[1] >= deltas[0] + 4096, "inline={inline}: {deltas:?}");
        assert_eq!(
            pixels[0], pixels[1],
            "unused resources must not change pixels"
        );
    }
}

#[test]
fn color_resource_limit_cannot_return_partial_handle_success() {
    for inline in [false, true] {
        let input = color_resource_copy_file(inline, 4096);
        let full =
            PdfDocument::from_bytes_with_limits(input.clone(), PdfImportLimits::default()).unwrap();
        full.page_image_handles(0).unwrap();
        let before = full.import_budget().unwrap().decoded_bytes();
        full.page_image_handles(0).unwrap();
        let after = full.import_budget().unwrap().decoded_bytes();
        assert!(after > before);
        for cap in [after - 1, after, after + 1] {
            let doc = PdfDocument::from_bytes_with_limits(
                input.clone(),
                PdfImportLimits {
                    decoded_bytes: cap,
                    ..PdfImportLimits::default()
                },
            )
            .unwrap();
            assert_eq!(doc.page_image_handles(0).unwrap().len(), 1);
            let result = doc.page_image_handles(0);
            if cap < after {
                assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
                assert!(doc.page_image_handles(0).is_err());
                assert!(doc.check_import_budget().is_err());
            } else {
                assert_eq!(result.unwrap().len(), 1);
            }
        }
    }
}

#[test]
fn compressed_object_cache_preserves_values_and_accounts_return_copies() {
    use pdf_oxide::object::ObjectRef;
    let mut input = b"%PDF-1.5\n".to_vec();
    let catalog_offset = input.len();
    input.extend(b"1 0 obj\n<< /Type /Catalog >>\nendobj\n");
    let object_stream_offset = input.len();
    let body = format!("4 0 << /Nested [({}) /Name] >>", "c".repeat(4096));
    write!(input, "2 0 obj\n<< /Type /ObjStm /N 1 /First 4 /Length {} >>\nstream\n{body}\nendstream\nendobj\n", body.len()).unwrap();
    let xref_offset = input.len();
    let mut entries = Vec::new();
    for (kind, offset, generation) in [
        (0u8, 0usize, 65535u16),
        (1, catalog_offset, 0),
        (1, object_stream_offset, 0),
        (1, xref_offset, 0),
        (2, 2, 0),
    ] {
        entries.push(kind);
        entries.extend((offset as u32).to_be_bytes());
        entries.extend(generation.to_be_bytes());
    }
    write!(
        input,
        "3 0 obj\n<< /Type /XRef /Size 5 /W [1 4 2] /Root 1 0 R /Length {} >>\nstream\n",
        entries.len()
    )
    .unwrap();
    input.extend(entries);
    write!(
        input,
        "\nendstream\nendobj\nstartxref\n{xref_offset}\n%%EOF\n"
    )
    .unwrap();
    let doc = PdfDocument::from_bytes_with_limits(input, PdfImportLimits::default()).unwrap();
    let first = doc.load_object(ObjectRef::new(4, 0)).unwrap();
    let nested = first.as_dict().unwrap()["Nested"].as_array().unwrap();
    assert_eq!(nested[0].as_string().unwrap(), vec![b'c'; 4096]);
    let before = doc.import_budget().unwrap().decoded_bytes();
    assert_eq!(doc.load_object(ObjectRef::new(4, 0)).unwrap(), first);
    assert!(doc.import_budget().unwrap().decoded_bytes() - before >= 4096);
}
// TrueType bytes below follow OpenType cmap formats 0/4/6/12; these are
// resource contracts, not rendered font or Hancom layout baselines.
fn font_cmap_sfnt(subtable: &[u8]) -> Vec<u8> {
    let mut data = vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    data.extend(b"cmap");
    data.extend(0u32.to_be_bytes());
    data.extend(28u32.to_be_bytes());
    data.extend(((12 + subtable.len()) as u32).to_be_bytes());
    data.extend([0, 0, 0, 1, 0, 3, 0, 10]);
    data.extend(12u32.to_be_bytes());
    data.extend(subtable);
    data
}

fn font_cmap_format12(groups: &[(u32, u32, u32)]) -> Vec<u8> {
    let mut data = vec![0, 12, 0, 0];
    data.extend(((16 + groups.len() * 12) as u32).to_be_bytes());
    data.extend(0u32.to_be_bytes());
    data.extend((groups.len() as u32).to_be_bytes());
    for (start, end, gid) in groups {
        for value in [start, end, gid] {
            data.extend(value.to_be_bytes());
        }
    }
    font_cmap_sfnt(&data)
}

fn limited_truetype_font(
    data: Vec<u8>,
    limits: PdfImportLimits,
) -> (PdfDocument, pdf_oxide::fonts::FontInfo) {
    let doc = PdfDocument::from_bytes_with_limits(object_file(&objects(b"", "", ""), ""), limits)
        .unwrap();
    let (_, dict) = pdf_oxide::parser::parse_object(
        b"<< /Type /Font /Subtype /TrueType /BaseFont /AAAAAA+BudgetProbe >>",
    )
    .unwrap();
    let mut font = pdf_oxide::fonts::FontInfo::from_dict(&dict, &doc).unwrap();
    font.embedded_font_data = Some(Arc::new(data));
    font.is_truetype_font = true;
    (doc, font)
}

fn metric_cache_file(name: &str) -> Vec<u8> {
    let mut parts = objects(b"BT /F1 12 Tf 20 100 Td <00010002> Tj ET", "", "");
    parts[4] = format!("<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{name} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /W [0 2 500] >>] >>").into_bytes();
    object_file(&parts, "")
}

fn load_probe_font(doc: &PdfDocument) -> Result<Arc<pdf_oxide::fonts::FontInfo>, Error> {
    let (_, resources) = pdf_oxide::parser::parse_object(b"<< /Font << /F1 5 0 R >> >>").unwrap();
    let mut extractor = pdf_oxide::extractors::TextExtractor::new();
    doc.load_fonts_public(&resources, &mut extractor)?;
    Ok(extractor
        .get_font_set()
        .into_iter()
        .find(|(name, _)| name == "F1")
        .unwrap()
        .1)
}

#[test]
fn font_global_cache_cannot_bypass_a_later_requests_metric_limit() {
    let input = metric_cache_file("BudgetWarmLegacy7911");
    let legacy = PdfDocument::from_bytes(input.clone()).unwrap();
    assert_eq!(
        load_probe_font(&legacy)
            .unwrap()
            .cid_widths
            .as_ref()
            .unwrap()
            .len(),
        3
    );
    let limited = PdfDocument::from_bytes_with_limits(
        input.clone(),
        PdfImportLimits {
            font_entries: 2,
            ..Default::default()
        },
    )
    .unwrap();
    let result = load_probe_font(&limited);
    assert!(matches!(
        result,
        Err(Error::ImportResourceLimit("font metric entries"))
    ));
    assert!(load_probe_font(&limited).is_err());
    let fresh = PdfDocument::from_bytes_with_limits(
        input,
        PdfImportLimits {
            font_entries: 3,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        load_probe_font(&fresh)
            .unwrap()
            .cid_widths
            .as_ref()
            .unwrap()
            .len(),
        3
    );
    fresh.check_import_budget().unwrap();
}

#[test]
fn limited_fonts_share_only_within_the_originating_document() {
    let input = metric_cache_file("BudgetLocalOnly7911");
    let first = PdfDocument::from_bytes_with_limits(input.clone(), Default::default()).unwrap();
    let second = PdfDocument::from_bytes_with_limits(input.clone(), Default::default()).unwrap();
    let a = load_probe_font(&first).unwrap();
    assert!(Arc::ptr_eq(&a, &load_probe_font(&first).unwrap()));
    let b = load_probe_font(&second).unwrap();
    assert!(!Arc::ptr_eq(&a, &b));
    let legacy = PdfDocument::from_bytes(input.clone()).unwrap();
    let c = load_probe_font(&legacy).unwrap();
    assert!(!Arc::ptr_eq(&a, &c));
    assert!(!Arc::ptr_eq(&b, &c));
    let legacy2 = PdfDocument::from_bytes(input).unwrap();
    assert!(Arc::ptr_eq(&c, &load_probe_font(&legacy2).unwrap()));
    first
        .import_budget()
        .unwrap()
        .reject("font isolation probe");
    assert!(load_probe_font(&first).is_err());
    second.check_import_budget().unwrap();
    assert!(load_probe_font(&second).is_ok());
}

#[test]
fn truetype_cmap_expansion_obeys_exact_request_limits() {
    let data = font_cmap_format12(&[(65, 67, 1)]);
    for cap in [2, 3, 4] {
        let (doc, font) = limited_truetype_font(
            data.clone(),
            PdfImportLimits {
                font_entries: cap,
                ..Default::default()
            },
        );
        let cmap = font.truetype_cmap();
        if cap == 2 {
            assert!(cmap.is_none());
            assert!(doc.check_import_budget().is_err());
        } else {
            let cmap = cmap.unwrap();
            assert_eq!(cmap.len(), 3);
            for (gid, ch) in [(1, 'A'), (2, 'B'), (3, 'C')] {
                assert_eq!(cmap.get_unicode(gid), Some(ch));
            }
            let before = doc.import_budget().unwrap().decoded_bytes();
            assert!(font.truetype_cmap().is_some());
            assert_eq!(doc.import_budget().unwrap().decoded_bytes(), before);
        }
    }
}

#[test]
fn truetype_cmap_repeated_ranges_charge_work_not_distinct_glyphs() {
    let data = font_cmap_format12(&[(65, 67, 1), (65, 67, 1)]);
    let (doc, font) = limited_truetype_font(
        data.clone(),
        PdfImportLimits {
            font_entries: 5,
            ..Default::default()
        },
    );
    assert!(font.truetype_cmap().is_none());
    assert!(doc.check_import_budget().is_err());
    let (doc, font) = limited_truetype_font(
        data,
        PdfImportLimits {
            font_entries: 6,
            ..Default::default()
        },
    );
    assert_eq!(font.truetype_cmap().unwrap().len(), 3);
    doc.check_import_budget().unwrap();
}

#[test]
fn truetype_lazy_success_cannot_outlive_a_fatal_request() {
    let (doc, font) = limited_truetype_font(font_cmap_format12(&[(65, 65, 1)]), Default::default());
    assert_eq!(font.truetype_cmap().unwrap().get_unicode(1), Some('A'));
    doc.import_budget().unwrap().reject("lazy font probe");
    assert!(font.truetype_cmap().is_none());
    assert!(font.clone().truetype_cmap().is_none());
    let (fresh, font) =
        limited_truetype_font(font_cmap_format12(&[(65, 65, 1)]), Default::default());
    assert!(font.truetype_cmap().is_some());
    fresh.check_import_budget().unwrap();
}

#[test]
fn truetype_maps_consume_cumulative_allocation_budget_before_expansion() {
    let data = font_cmap_format12(&[(65, 67, 1)]);
    let (doc, font) = limited_truetype_font(data.clone(), Default::default());
    let before = doc.import_budget().unwrap().decoded_bytes();
    assert!(font.truetype_cmap().is_some());
    let after = doc.import_budget().unwrap().decoded_bytes();
    assert!(after >= before + 3 * std::mem::size_of::<(u16, char)>() as u64);
    for cap in [after - 1, after, after + 1] {
        let (doc, font) = limited_truetype_font(
            data.clone(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..Default::default()
            },
        );
        if cap < after {
            assert!(font.truetype_cmap().is_none());
            assert!(doc.check_import_budget().is_err());
        } else {
            assert_eq!(font.truetype_cmap().unwrap().len(), 3);
            doc.check_import_budget().unwrap();
        }
    }
}

#[test]
fn truetype_format12_rejects_glyph_overflow_without_wrapping() {
    let (_, font) =
        limited_truetype_font(font_cmap_format12(&[(65, 67, 65535)]), Default::default());
    assert!(font.truetype_cmap().is_none());
}

fn embedded_truetype_file(data: &[u8], cid: bool) -> Vec<u8> {
    let mut parts = objects(b"BT /F1 12 Tf 20 100 Td (ABC) Tj ET", "", "");
    let descriptor = "/FontDescriptor 7 0 R";
    parts[4] = if cid {
        format!("<< /Type /Font /Subtype /Type0 /BaseFont /AAAAAA+ProgramProbe /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /AAAAAA+ProgramProbe /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> {descriptor} >>] >>").into_bytes()
    } else {
        format!("<< /Type /Font /Subtype /TrueType /BaseFont /AAAAAA+ProgramProbe {descriptor} >>")
            .into_bytes()
    };
    let mut stream = format!("<< /Length {} >>\nstream\n", data.len()).into_bytes();
    stream.extend(data);
    stream.extend(b"\nendstream");
    parts.push(stream);
    parts.push(
        b"<< /Type /FontDescriptor /FontName /AAAAAA+ProgramProbe /Flags 32 /FontFile2 6 0 R >>"
            .to_vec(),
    );
    object_file(&parts, "")
}

#[test]
fn embedded_truetype_limits_propagate_through_simple_and_cid_font_loading() {
    for cid in [false, true] {
        let input = embedded_truetype_file(&font_cmap_format12(&[(65, 67, 1)]), cid);
        for cap in [2, 3, 4] {
            let doc = PdfDocument::from_bytes_with_limits(
                input.clone(),
                PdfImportLimits {
                    font_entries: cap,
                    ..Default::default()
                },
            )
            .unwrap();
            let result = load_probe_font(&doc);
            if cap == 2 {
                assert!(
                    matches!(
                        result,
                        Err(Error::ImportResourceLimit("font metric entries"))
                    ),
                    "cid={cid}"
                );
                assert!(doc.extract_text(0).is_err());
            } else {
                assert_eq!(
                    result.unwrap().truetype_cmap().unwrap().get_unicode(3),
                    Some('C')
                );
                doc.check_import_budget().unwrap();
            }
        }
    }
}

fn font_cmap_small_formats() -> Vec<(Vec<u8>, usize)> {
    let mut zero = vec![0, 0, 1, 6, 0, 0];
    zero.resize(262, 0);
    zero[6 + 65] = 1;
    zero[6 + 66] = 2;
    zero[6 + 67] = 3;
    let six: Vec<u8> = [6u16, 16, 0, 65, 3, 1, 2, 3]
        .iter()
        .flat_map(|n| n.to_be_bytes())
        .collect();
    // Format 4: A..C via delta, then the required 0xffff sentinel segment.
    let four: Vec<u8> = [
        4u16, 32, 0, 4, 4, 1, 0, 67, 65535, 0, 65, 65535, 65472, 1, 0, 0,
    ]
    .iter()
    .flat_map(|n| n.to_be_bytes())
    .collect();
    vec![
        (font_cmap_sfnt(&zero), 256),
        (font_cmap_sfnt(&four), 3),
        (font_cmap_sfnt(&six), 3),
    ]
}

#[test]
fn truetype_supported_formats_keep_mapping_and_obey_work_limits() {
    for (data, work) in font_cmap_small_formats() {
        let legacy = pdf_oxide::fonts::TrueTypeCMap::from_font_data(&data).unwrap();
        for cap in [work - 1, work, work + 1] {
            let (doc, font) = limited_truetype_font(
                data.clone(),
                PdfImportLimits {
                    font_entries: cap,
                    ..Default::default()
                },
            );
            if cap < work {
                assert!(font.truetype_cmap().is_none());
                assert!(doc.check_import_budget().is_err());
            } else {
                let map = font.truetype_cmap().unwrap();
                assert_eq!(map.len(), legacy.len());
                for (gid, ch) in [(1, 'A'), (2, 'B'), (3, 'C')] {
                    assert_eq!(map.get_unicode(gid), Some(ch));
                    assert_eq!(map.get_unicode(gid), legacy.get_unicode(gid));
                }
                doc.check_import_budget().unwrap();
            }
        }
    }
}

#[test]
fn truetype_source_and_scan_caps_precede_mapping_allocation() {
    let data = font_cmap_format12(&[(65, 67, 1)]);
    let cmap_bytes = data.len() - 28;
    for source_limit in [true, false] {
        for cap in [cmap_bytes - 1, cmap_bytes, cmap_bytes + 1] {
            let mut limits = PdfImportLimits::default();
            if source_limit {
                limits.cmap_bytes = cap;
            } else {
                limits.cmap_scan_bytes = cap;
            }
            let (doc, font) = limited_truetype_font(data.clone(), limits);
            if cap < cmap_bytes {
                assert!(font.truetype_cmap().is_none());
                assert!(doc.check_import_budget().is_err());
            } else {
                assert_eq!(font.truetype_cmap().unwrap().len(), 3);
                doc.check_import_budget().unwrap();
            }
        }
    }
}

#[test]
fn truetype_format4_cannot_read_glyphs_from_the_next_table() {
    // The first idRangeOffset points to a glyph just AFTER this subtable.
    let four: Vec<u8> = [4u16, 32, 0, 4, 4, 1, 0, 65, 65535, 0, 65, 65535, 0, 1, 4, 0]
        .iter()
        .flat_map(|n| n.to_be_bytes())
        .collect();
    let mut data = font_cmap_sfnt(&four);
    data.extend(7u16.to_be_bytes());
    let (_, font) = limited_truetype_font(data, Default::default());
    let map = font.truetype_cmap();
    assert!(map.is_none_or(|map| map.get_unicode(7).is_none()));
}

#[test]
fn truetype_declared_ranges_are_bounded_without_large_materialization() {
    let (doc, font) = limited_truetype_font(
        font_cmap_format12(&[(0, 65535, 0)]),
        PdfImportLimits {
            font_entries: 16,
            ..Default::default()
        },
    );
    assert!(font.truetype_cmap().is_none());
    assert!(doc.check_import_budget().is_err());
    for data in [
        font_cmap_format12(&[(0, u32::MAX, 0)]),
        font_cmap_format12(&[(67, 65, 1)]),
        font_cmap_format12(&[(65, 67, u32::MAX)]),
    ] {
        let (_, font) = limited_truetype_font(data, Default::default());
        assert!(font.truetype_cmap().is_none());
    }
    let mut truncated = font_cmap_format12(&[(65, 67, 1)]);
    truncated[52..56].copy_from_slice(&u32::MAX.to_be_bytes());
    let (_, font) = limited_truetype_font(truncated, Default::default());
    assert!(font.truetype_cmap().is_none());
}

fn font_cmap_symbol_sfnt() -> Vec<u8> {
    let four: Vec<u8> = [
        4u16, 32, 0, 4, 4, 1, 0, 0xf043, 0xffff, 0, 0xf041, 0xffff, 0x0fc0, 1, 0, 0,
    ]
    .iter()
    .flat_map(|n| n.to_be_bytes())
    .collect();
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 0];
    cmap.extend(12u32.to_be_bytes());
    cmap.extend(four);
    let mut head = vec![0; 54];
    head[18..20].copy_from_slice(&2048u16.to_be_bytes());
    let tables = [
        (b"cmap", cmap),
        (b"head", head),
        (b"hhea", vec![0; 36]),
        (b"maxp", vec![0, 0, 0x50, 0, 0, 4]),
    ];
    let mut data = vec![0, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0];
    let mut payload = Vec::new();
    for (tag, table) in tables {
        data.extend(tag);
        data.extend(0u32.to_be_bytes());
        data.extend(((76 + payload.len()) as u32).to_be_bytes());
        data.extend((table.len() as u32).to_be_bytes());
        payload.extend(table);
    }
    data.extend(payload);
    data
}

#[test]
fn truetype_symbol_mapping_lookup_is_bounded_and_preserves_pua_resolution() {
    let data = font_cmap_symbol_sfnt();
    let legacy = pdf_oxide::fonts::TrueTypeCMap::from_font_data(&data).unwrap();
    assert_eq!(legacy.code_to_gid(65), Some(1));
    for cap in [258, 259, 260] {
        let (doc, font) = limited_truetype_font(
            data.clone(),
            PdfImportLimits {
                font_entries: cap,
                ..Default::default()
            },
        );
        if cap == 258 {
            assert!(font.truetype_cmap().is_none());
            assert!(doc.check_import_budget().is_err());
        } else {
            let map = font.truetype_cmap().unwrap();
            assert_eq!(map.code_to_gid(65), Some(1));
            assert_eq!(map.get_unicode(1), Some('\u{f041}'));
            doc.check_import_budget().unwrap();
        }
    }
}

#[test]
fn truetype_format4_scratch_and_maps_obey_cumulative_byte_boundary() {
    let data = font_cmap_small_formats().remove(1).0;
    let (doc, font) = limited_truetype_font(data.clone(), Default::default());
    let before = doc.import_budget().unwrap().decoded_bytes();
    assert!(font.truetype_cmap().is_some());
    let after = doc.import_budget().unwrap().decoded_bytes();
    // Four 2-entry u16/i16 arrays precede the three map-entry reservations.
    assert!(after >= before + 16 + 3 * std::mem::size_of::<(u16, char)>() as u64);
    for cap in [after - 1, after, after + 1] {
        let (doc, font) = limited_truetype_font(
            data.clone(),
            PdfImportLimits {
                decoded_bytes: cap,
                ..Default::default()
            },
        );
        if cap < after {
            assert!(font.truetype_cmap().is_none());
            assert!(doc.check_import_budget().is_err());
        } else {
            assert!(font.truetype_cmap().is_some());
            doc.check_import_budget().unwrap();
        }
    }
}

#[test]
fn png_codec_scratch_reuse_is_request_scoped_and_preserves_changing_pixels() {
    use pdf_oxide::extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat};
    let first = budget(4096, 2 * 1024 * 1024);
    let second = budget(4096, 2 * 1024 * 1024);
    for request in [&first, &second] {
        let mut scratch = 0;
        for n in 0..100u8 {
            let input = vec![n, 255 - n, n / 2, 255 - n, n, n / 3];
            let image = PdfImage::new(
                2,
                1,
                ColorSpace::DeviceRGB,
                8,
                ImageData::Raw {
                    pixels: input.clone(),
                    format: PixelFormat::RGB,
                },
            );
            let png = image.to_png_bytes_with_budget(request).unwrap();
            assert_eq!(
                image::load_from_memory(&png).unwrap().to_rgb8().as_raw(),
                &input
            );
            if n == 0 {
                scratch = request.codec_allocated_bytes();
                assert!(scratch > 0);
            }
            assert_eq!(
                request.codec_allocated_bytes(),
                scratch,
                "another PNG must reuse its returned codec block"
            );
        }
    }
    assert_eq!(
        first.codec_allocated_bytes(),
        second.codec_allocated_bytes(),
        "each request owns its own initial allocation"
    );
}

#[test]
fn selected_work_restrictions_preserve_consumption_and_never_raise_limits() {
    let request = budget(100, 100);
    request.charge_decoded(40).unwrap();
    request.restrict_work_limits(60, 10).unwrap();
    request.restrict_work_limits(100, 20).unwrap();
    request.charge_decoded(20).unwrap();
    assert!(request.charge_decoded(1).is_err());
    let failure = request.snapshot().failure.unwrap();
    assert_eq!(
        (failure.used, failure.requested, failure.limit),
        (Some(60), Some(1), Some(60))
    );
    assert!(request.restrict_work_limits(100, 100).is_err());
    let next = budget(100, 100);
    next.charge_decoded(40).unwrap();
    assert!(next.restrict_work_limits(39, 10).is_err());
    assert_eq!(next.snapshot().failure.unwrap().used, Some(40));
    assert!(budget(100, 100).restrict_work_limits(0, 1).is_err());
}

#[test]
fn indirect_stream_length_preserves_binary_tail_bytes() {
    for bytes in [vec![1, 13], vec![1, 10], vec![1, 13, 10], vec![1, 32]] {
        let mut parts = objects(&bytes, "", "");
        let mut content = b"<< /Length 6 0 R >>\nstream\n".to_vec();
        content.extend_from_slice(&bytes);
        content.extend_from_slice(b"\nendstream");
        parts[3] = content;
        parts.push(bytes.len().to_string().into_bytes());
        let doc = PdfDocument::from_bytes_with_limits(
            object_file(&parts, ""),
            PdfImportLimits::default(),
        )
        .unwrap();
        let object = doc
            .load_object(pdf_oxide::object::ObjectRef::new(4, 0))
            .unwrap();
        let pdf_oxide::object::Object::Stream { data, .. } = object else {
            panic!("stream required")
        };
        assert_eq!(&data[..], bytes);
    }
}

#[test]
fn indirect_predictors_and_filter_specific_parameters_preserve_samples() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    for first_predictor in [false, true] {
        let pixels = [10, 20, 30];
        let (data, filter, params, columns) = if first_predictor {
            let inner = flate(&pixels);
            let mut predicted = vec![0];
            predicted.extend_from_slice(&inner);
            (
                flate(&predicted),
                "/Filter [/FlateDecode /FlateDecode]",
                "/DecodeParms [7 0 R null]",
                inner.len(),
            )
        } else {
            (
                flate(&[0, 10, 20, 30]),
                "/Filter /FlateDecode",
                "/DecodeParms 7 0 R",
                3,
            )
        };
        let mut parts = objects(b"/Im Do", "", "");
        parts[2] = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im 6 0 R >> >> /Contents 4 0 R >>".to_vec();
        let mut image = format!("<< /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 {filter} {params} /Length {} >>\nstream\n", data.len()).into_bytes();
        image.extend_from_slice(&data);
        image.extend_from_slice(b"\nendstream");
        parts.push(image);
        parts.push(b"<< /Predictor 15 /Colors 1 /Columns 8 0 R /BitsPerComponent 8 >>".to_vec());
        parts.push(columns.to_string().into_bytes());
        let source = object_file(&parts, "");
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgb = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgb8();
        assert_eq!(rgb.get_pixel(0, 0).0, pixels);
        // A cycle cannot silently discard predictor semantics or escape the request.
        parts[6] = b"7 0 R".to_vec();
        assert_eq!(
            parse_pdf_with_options(&object_file(&parts, ""), &PdfImportOptions::default())
                .err()
                .unwrap()
                .code,
            "PDF_RESOURCE_LIMIT"
        );
    }
}

#[test]
fn interpolated_soft_mask_resamples_alpha_without_changing_rgb() {
    use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
    for (flag, decode, expected) in [
        ("false", "", [0, 0, 255, 255]),
        ("true", "", [0, 64, 191, 255]),
        ("true", "/Decode [1 0]", [255, 191, 64, 0]),
    ] {
        let mask=format!("<< /Subtype /Image /Width 2 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Interpolate {flag} {decode} /Length 2 >>\nstream\n");
        let mut mask = mask.into_bytes();
        mask.extend_from_slice(&[0, 255]);
        mask.extend_from_slice(b"\nendstream");
        let source = indexed_image_pdf(
            &[20, 40, 60].repeat(4),
            "/Width 4 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /SMask 7 0 R",
            &mask,
        );
        let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
        let rgba = image::load_from_memory(&imported.document.bin_data_content[0].data.load())
            .unwrap()
            .to_rgba8();
        assert_eq!(rgba.pixels().map(|p| p.0[3]).collect::<Vec<_>>(), expected);
        assert!(rgba.pixels().all(|p| p.0[..3] == [20, 40, 60]));
        let doc = PdfDocument::from_bytes_with_limits(
            source,
            PdfImportLimits {
                codec_allocation_bytes: 15,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(matches!(
            doc.page_image_handles(0).unwrap()[0].decode(),
            Err(Error::ImportResourceLimit(_))
        ));
    }
}

// Resource-policy contracts: compressed JPEG storage is not an allocated
// eight-byte-per-pixel plane. Actual codec and PNG buffers retain their caps.
fn jpeg_extraction_cap_bytes() -> Vec<u8> {
    let rgb = vec![73u8; 64 * 64 * 3];
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 90)
        .encode(&rgb, 64, 64, image::ExtendedColorType::Rgb8)
        .unwrap();
    jpeg
}

fn jpeg_extraction_cap_source() -> Vec<u8> {
    indexed_image_pdf(
        &jpeg_extraction_cap_bytes(),
        "/Width 64 /Height 64 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Filter /DCTDecode",
        b"null",
    )
}

fn jpeg_cap_png(source: Vec<u8>, limits: PdfImportLimits) -> Result<Vec<u8>, Error> {
    let doc = PdfDocument::from_bytes_with_limits(source, limits)?;
    let image = doc.page_image_handles(0)?[0].decode()?;
    image.to_png_bytes_with_budget(doc.import_budget().unwrap())
}

#[test]
fn jpeg_extraction_uses_actual_rgb_gray_and_cmyk_buffer_reservations() {
    for (jpeg, space) in [
        (
            include_bytes!("../fixtures/issue7911_jpeg/RGB-0-0.jpg").as_slice(),
            "DeviceRGB",
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/L-0-0.jpg").as_slice(),
            "DeviceGray",
        ),
        (
            include_bytes!("../fixtures/issue7911_jpeg/CMYK-0-0.jpg").as_slice(),
            "DeviceCMYK",
        ),
    ] {
        let source = indexed_image_pdf(
            jpeg,
            &format!(
                "/Width 19 /Height 17 /BitsPerComponent 8 /ColorSpace /{space} /Filter /DCTDecode"
            ),
            b"null",
        );
        let expected = jpeg_cap_png(source.clone(), Default::default()).unwrap();
        // 19*17*4 = 1292 fits; 19*17*8 = 2584 does not. The compressed
        // fixture also fits, independently of its decoded geometry.
        assert_eq!(
            jpeg_cap_png(
                source,
                PdfImportLimits {
                    stream_bytes: 2400,
                    ..Default::default()
                }
            )
            .unwrap(),
            expected,
            "{space}: request limits must preserve decoded samples"
        );
    }
}

#[test]
fn jpeg_extraction_pixel_buffer_cap_remains_exact_and_request_local() {
    let source = jpeg_extraction_cap_source();
    let expected = jpeg_cap_png(source.clone(), Default::default()).unwrap();
    for cap in [64 * 64 * 4 - 1, 64 * 64 * 4, 64 * 64 * 4 + 1] {
        let doc = PdfDocument::from_bytes_with_limits(
            source.clone(),
            PdfImportLimits {
                stream_bytes: cap,
                ..Default::default()
            },
        )
        .unwrap();
        let image = doc.page_image_handles(0).unwrap()[0].decode().unwrap();
        let result = image.to_png_bytes_with_budget(doc.import_budget().unwrap());
        if cap < 64 * 64 * 4 {
            assert!(matches!(
                result,
                Err(Error::ImportResourceLimit("image pixel buffer"))
            ));
            assert!(doc.check_import_budget().is_err());
            assert!(doc.page_image_handles(0).is_err());
        } else {
            assert_eq!(result.unwrap(), expected);
        }
    }
    assert_eq!(jpeg_cap_png(source, Default::default()).unwrap(), expected);
}

#[test]
fn jpeg_extraction_keeps_private_codec_caps_after_compressed_storage() {
    let source = jpeg_extraction_cap_source();
    let doc = PdfDocument::from_bytes_with_limits(
        source.clone(),
        PdfImportLimits {
            stream_bytes: 64 * 64 * 4,
            codec_allocation_bytes: 8192,
            ..Default::default()
        },
    )
    .unwrap();
    let image = doc.page_image_handles(0).unwrap()[0].decode().unwrap();
    assert!(matches!(
        image.to_png_bytes_with_budget(doc.import_budget().unwrap()),
        Err(Error::ImportResourceLimit(_))
    ));
    assert!(doc.check_import_budget().is_err());
    assert!(doc.page_image_handles(0).is_err());
    assert!(!jpeg_cap_png(source, Default::default()).unwrap().is_empty());
}

#[test]
fn raw_extraction_keeps_its_existing_geometry_guard() {
    let source = indexed_image_pdf(
        &vec![73; 64 * 64 * 3],
        "/Width 64 /Height 64 /BitsPerComponent 8 /ColorSpace /DeviceRGB",
        b"null",
    );
    let doc = PdfDocument::from_bytes_with_limits(
        source,
        PdfImportLimits {
            stream_bytes: 20_000,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(
        doc.page_image_handles(0).unwrap()[0].decode(),
        Err(Error::ImportResourceLimit("image extraction buffer"))
    ));
    assert!(doc.check_import_budget().is_err());
}

#[test]
fn jpeg_extraction_filter_chains_and_inline_share_actual_buffer_reservations() {
    let hex = jpeg_extraction_cap_bytes()
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<String>()
        + ">";
    let expected = jpeg_cap_png(jpeg_extraction_cap_source(), Default::default()).unwrap();
    for inline in [false, true] {
        let source = if inline {
            let content = format!("BI /W 64 /H 64 /BPC 8 /CS /RGB /F [/AHx /DCT] ID {hex} EI");
            object_file(&objects(content.as_bytes(), "", ""), "")
        } else {
            indexed_image_pdf(hex.as_bytes(),
                "/Width 64 /Height 64 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Filter [/ASCIIHexDecode /DCTDecode]", b"null")
        };
        assert_eq!(
            jpeg_cap_png(
                source,
                PdfImportLimits {
                    stream_bytes: 64 * 64 * 4,
                    ..Default::default()
                }
            )
            .unwrap(),
            expected
        );
    }
}

// Decoded-resource contracts, not rendering goldens.
#[test]
fn stream_masks_share_bounded_lossless_filter_chains() {
    fn ascii85(bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for chunk in bytes.chunks(4) {
            let mut padded = [0u8; 4];
            padded[..chunk.len()].copy_from_slice(chunk);
            let mut value = u32::from_be_bytes(padded);
            let mut digits = [0u8; 5];
            for digit in digits.iter_mut().rev() {
                *digit = (value % 85) as u8 + b'!';
                value /= 85;
            }
            out.extend_from_slice(&digits[..chunk.len() + 1]);
        }
        out.extend_from_slice(b"~>");
        out
    }
    fn source(data: &[u8], filter: &str, soft: bool) -> Vec<u8> {
        let mask_type = if soft {
            "/ColorSpace /DeviceGray /BitsPerComponent 8"
        } else {
            "/ImageMask true /BitsPerComponent 1"
        };
        let mut mask = format!("<< /Type /XObject /Subtype /Image /Width 4 /Height 1 {mask_type} {filter} /Length {} >>\nstream\n",data.len()).into_bytes();
        mask.extend_from_slice(data);
        mask.extend_from_slice(b"\nendstream");
        let key = if soft { "SMask" } else { "Mask" };
        indexed_image_pdf(
            &[20, 40, 60].repeat(4),
            &format!("/Width 4 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /{key} 7 0 R"),
            &mask,
        )
    }
    fn pixels(source: Vec<u8>, limits: PdfImportLimits) -> Result<Vec<u8>, Error> {
        let doc = PdfDocument::from_bytes_with_limits(source, limits)?;
        let image = doc.page_image_handles(0)?[0].decode()?;
        let pdf_oxide::extractors::images::ImageData::Raw { pixels, .. } = image.data() else {
            panic!("mask must yield raw RGBA")
        };
        Ok(pixels.clone())
    }
    for soft in [true, false] {
        let raw: &[u8] = if soft { &[0, 64, 128, 255] } else { &[0x50] };
        let compressed = flate(raw);
        let mut hex = Vec::new();
        for byte in &compressed {
            hex.extend_from_slice(format!("{byte:02X}").as_bytes());
        }
        hex.push(b'>');
        let mut runs = vec![(raw.len() - 1) as u8];
        runs.extend_from_slice(raw);
        runs.push(128);
        // Independent PDF LZW 9-bit codes: clear, the raw samples, end.
        let codes: Vec<u16> = std::iter::once(256)
            .chain(raw.iter().map(|b| u16::from(*b)))
            .chain(std::iter::once(257))
            .collect();
        let mut lzw = vec![0u8; (codes.len() * 9).div_ceil(8)];
        for (i, code) in codes.iter().enumerate() {
            for bit in 0..9 {
                if code & (1 << (8 - bit)) != 0 {
                    let at = i * 9 + bit;
                    lzw[at / 8] |= 1 << (7 - at % 8);
                }
            }
        }
        let expected: Vec<u8> = if soft {
            [0, 64, 128, 255]
                .into_iter()
                .flat_map(|a| [20, 40, 60, a])
                .collect()
        } else {
            [255, 0, 255, 0]
                .into_iter()
                .flat_map(|a| [20, 40, 60, a])
                .collect()
        };
        for (filter, data) in [
            ("", raw.to_vec()),
            ("/Filter /FlateDecode", compressed.clone()),
            ("/Filter /ASCII85Decode", ascii85(raw)),
            (
                "/Filter [/ASCII85Decode /FlateDecode]",
                ascii85(&compressed),
            ),
            ("/Filter [/ASCIIHexDecode /FlateDecode]", hex),
            ("/Filter /RunLengthDecode", runs),
            ("/Filter /LZWDecode", lzw),
        ] {
            assert_eq!(
                pixels(source(&data, filter, soft), PdfImportLimits::default()).unwrap(),
                expected,
                "soft={soft} {filter}"
            );
        }
        let encoded = ascii85(&compressed);
        let valid = source(&encoded, "/Filter [/ASCII85Decode /FlateDecode]", soft);
        let doc =
            PdfDocument::from_bytes_with_limits(valid.clone(), PdfImportLimits::default()).unwrap();
        doc.page_image_handles(0).unwrap()[0].decode().unwrap();
        let consumed = doc.import_budget().unwrap().decoded_bytes();
        for cap in [consumed - 1, consumed, consumed + 1] {
            let result = pixels(
                valid.clone(),
                PdfImportLimits {
                    decoded_bytes: cap,
                    ..Default::default()
                },
            );
            if cap < consumed {
                assert!(matches!(result, Err(Error::ImportResourceLimit(_))));
            } else {
                assert_eq!(result.unwrap(), expected);
            }
        }
        let limited = PdfDocument::from_bytes_with_limits(
            valid,
            PdfImportLimits {
                filters: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(matches!(
            limited.page_image_handles(0).unwrap()[0].decode(),
            Err(Error::ImportResourceLimit(_))
        ));
        assert!(
            limited.check_import_budget().is_err(),
            "filter budget failure stays sticky"
        );
        for codec in ["DCTDecode", "JPXDecode", "JBIG2Decode", "CCITTFaxDecode"] {
            let result = pixels(
                source(raw, &format!("/Filter /{codec}"), soft),
                PdfImportLimits::default(),
            );
            // These bytes are uncompressed samples with a deliberately false
            // filter label. Enabled codecs must reject the malformed stream;
            // disabled codecs (and the other mask formats) reject support.
            let enabled = match codec {
                "DCTDecode" => true,
                "JPXDecode" => cfg!(feature = "pdf-jpeg2000"),
                "JBIG2Decode" => cfg!(feature = "pdf-jbig2"),
                _ => false,
            };
            assert!(
                matches!(&result, Err(Error::Unsupported(_)))
                    || (enabled
                        && matches!(
                            &result,
                            Err(Error::Decode(_) | Error::Image(_) | Error::UnsupportedFilter(_))
                        )),
                "{codec}: compressed codec bytes cannot be interpreted as mask samples: {result:?}"
            );
        }
        assert!(pixels(
            source(b"!~>", "/Filter /ASCII85Decode", soft),
            PdfImportLimits::default()
        )
        .is_err());
        assert!(
            pixels(
                source(
                    &ascii85(&flate(&[])),
                    "/Filter [/ASCII85Decode /FlateDecode]",
                    soft
                ),
                PdfImportLimits::default()
            )
            .is_err(),
            "decoded length must match packed rows"
        );
        assert!(
            pixels(
                source(
                    &encoded,
                    "/Filter [/ASCII85Decode /FlateDecode] /DecodeParms << /Predictor 12 >>",
                    soft
                ),
                PdfImportLimits::default()
            )
            .is_err(),
            "predictor support is unchanged"
        );
    }
}
