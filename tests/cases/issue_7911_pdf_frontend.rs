//! 관계형·고정형 합성 Parser/IR·CLI 계약: 내용 보존·오류·선택 쪽수. 시각 좌표 golden이 아니다.
//! 주석 갱신: 2026-11-01T00:29:10+09:00 (Asia/Seoul).
//! 직전 파일 변경: 80968497c2c178eeecb3b2e1d04d086ad27a0dc7 / 2026-10-21T14:57:32+09:00.
//! #8415 인증 제어군: 독립 평문 PDF 1쪽, 2026-11-01 96dpi 보정 전 2px 실루엣.
//! Native/fresh WASM 각 100.00%, 전쪽 최저 각 100.00%; 본문·이미지 직접 판독 완료.
//! 근거: mydocs/working/assets/issue8415/password-authentication/ 및 task_m100_8413_implementation.md.
//! 이 수치는 #8415 새 암호 제어군에 한정하며 기존 합성 계약을 실제 문서 시각 검증으로 승격하지 않는다.
#![cfg(feature = "pdf-import")]

use rhwp::model::{control::Control, provenance::SourceFormat};
use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
use std::collections::BTreeSet;

fn pdf(pages: &[(&str, &str)]) -> Vec<u8> {
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        String::new(),
    ];
    let mut kids = Vec::new();
    for (text, font) in pages {
        let page = objects.len() + 1;
        kids.push(format!("{page} 0 R"));
        let content = format!("BT /F1 12 Tf 30 140 Td ({text}) Tj ET");
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 {} 0 R >> >> /Contents {} 0 R >>", page + 2, page + 1));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        ));
        objects.push(format!(
            "<< /Type /Font /Subtype /Type1 /BaseFont /{font} >>"
        ));
    }
    objects[1] = format!(
        "<< /Type /Pages /Count {} /Kids [{}] >>",
        pages.len(),
        kids.join(" ")
    );
    encode_pdf(&objects)
}

fn encode_pdf(objects: &[String]) -> Vec<u8> {
    let mut output = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(output.len());
        output.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, object).as_bytes());
    }
    let xref = output.len();
    output.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        output.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    output.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    output
}

#[test]
fn embedded_images_keep_their_bytes_and_references_in_both_save_formats() {
    use rhwp::{model::bin_data::BinDataType, parser, DocumentCore};
    // Two unfiltered RGB XObjects with distinct, independently specified pixels.
    // This checks the file-format/data contract, not a layout or pixel-position golden.
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Count 2 /Kids [3 0 R 6 0 R] >>".into(),
    ];
    for pixel in ["RGB", "CMY"] {
        let page = objects.len() + 1;
        let content = "q 20 0 0 30 10 100 cm /Im1 Do Q";
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /XObject << /Im1 {} 0 R >> >> /Contents {} 0 R >>", page + 2, page + 1));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
        objects.push(format!("<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length 3 >>\nstream\n{pixel}\nendstream"));
    }
    let core = DocumentCore::from_bytes(&encode_pdf(&objects)).unwrap();
    let expected = [b"RGB", b"CMY"];
    for (format, bytes) in [
        ("HWP", core.export_hwp_native().unwrap()),
        ("HWP adapter", core.export_hwp_with_adapter().unwrap()),
        ("HWPX", core.export_hwpx_native().unwrap()),
    ] {
        let saved = parser::parse_document(&bytes).unwrap();
        assert_eq!(saved.sections.len(), expected.len(), "{format}");
        assert_eq!(saved.bin_data_content.len(), expected.len(), "{format}");
        for (section, pixel) in saved.sections.iter().zip(expected) {
            let pictures: Vec<_> = section
                .paragraphs
                .iter()
                .flat_map(|p| &p.controls)
                .filter_map(|c| match c {
                    Control::Picture(pic) => Some(pic),
                    _ => None,
                })
                .collect();
            assert_eq!(
                pictures.len(),
                1,
                "{format}: picture belongs to its source page"
            );
            let reference = pictures[0].image_attr.bin_data_id;
            let descriptor = &saved.doc_info.bin_data_list[usize::from(reference) - 1];
            assert_eq!(descriptor.data_type, BinDataType::Embedding, "{format}");
            let content = saved
                .bin_data_content
                .iter()
                .find(|c| c.id == descriptor.storage_id)
                .unwrap();
            let decoded = image::load_from_memory(&content.data.load())
                .unwrap()
                .into_rgb8();
            assert_eq!(decoded.dimensions(), (1, 1));
            assert_eq!(
                decoded.as_raw(),
                pixel,
                "{format}: original pixel data survives save"
            );
            let original = &core.document().bin_data_content[usize::from(reference) - 1];
            assert_eq!(
                content.data.load(),
                original.data.load(),
                "{format}: encoded PNG is preserved"
            );
        }
    }
}

fn texts(document: &rhwp::model::document::Document) -> Vec<String> {
    document
        .sections
        .iter()
        .map(|s| {
            s.paragraphs
                .iter()
                .flat_map(|p| &p.controls)
                .filter_map(|c| match c {
                    Control::Shape(s) => s.drawing().and_then(|d| d.text_box.as_ref()),
                    _ => None,
                })
                .flat_map(|b| &b.paragraphs)
                .map(|p| p.text.as_str())
                .collect()
        })
        .collect()
}

#[test]
fn hwp_save_materializes_wire_contract_without_mutating_pdf_or_hwpx() {
    use rhwp::DocumentCore;
    let core =
        DocumentCore::from_bytes(&pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")])).unwrap();
    let hwpx_before = core.export_hwpx_native().unwrap();
    let snapshot = core.prepare_hwp_export_snapshot();
    for (format, bytes) in [
        ("native", core.export_hwp_native().unwrap()),
        ("adapter", core.export_hwp_with_adapter().unwrap()),
        ("snapshot", snapshot.serialize().unwrap()),
    ] {
        let saved = rhwp::parser::parse_document(&bytes).unwrap();
        // HWP5 specification table 3: major 5 denotes the emitted CFB format.
        assert_eq!(saved.header.version.major, 5, "{format}: HWP5 version");
        assert_eq!(saved.doc_properties.section_count, 2, "{format}");
        assert_eq!(texts(&saved), ["FIRST", "SECOND"], "{format}");
        for section in &saved.sections {
            // The independent Hancom HWP control and the existing HWP adapter
            // require three PAGE_BORDER_FILL records per section (#3676/#5933).
            assert_eq!(
                section.section_def.extra_page_border_fills.len(),
                2,
                "{format}"
            );
            assert!(matches!(
                section.paragraphs[0].controls[0],
                Control::SectionDef(_)
            ));
        }
        assert!(
            !saved.extra_streams.iter().any(|(path, _)| {
                path == rhwp::model::document::HWPX_ORIGIN_STREAM_PATH
                    || path == rhwp::model::document::HWP3_ORIGIN_STREAM_PATH
            }),
            "PDF conversion must not invent another source format"
        );
    }
    assert_eq!(core.document().provenance.format, SourceFormat::Pdf);
    assert_eq!(core.export_hwpx_native().unwrap(), hwpx_before);
}

#[test]
fn public_wrapper_exposes_import_snapshot_without_inventing_saved_pdf_provenance() {
    let source = pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")]);
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&source).unwrap();
    let metadata: serde_json::Value =
        serde_json::from_str(&document.get_pdf_import_diagnostics()).unwrap();
    assert_eq!(metadata["sourcePageCount"], 2);
    assert_eq!(metadata["pages"][0]["sourcePage"], 0);
    assert_eq!(metadata["pages"][1]["outputSection"], 1);
    assert_eq!(metadata["parserAllocationBoundsProven"], false);
    assert!(!metadata["paragraphPolicy"].as_str().unwrap().is_empty());
    let reopened =
        rhwp::wasm_api::HwpDocument::from_bytes(&document.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(reopened.get_pdf_import_diagnostics(), "null");
}

#[test]
fn all_pages_share_valid_style_pools_and_unique_instance_ids() {
    let source = pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")]);
    let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    let document = &imported.document;
    assert_eq!(texts(document), ["FIRST", "SECOND"]);
    assert_eq!(document.provenance.format, SourceFormat::Pdf);
    assert!(!document.layout_profile().native_hwp5_layout());
    assert!(!document.layout_profile().hwpx_container());
    assert_eq!(document.doc_properties.section_count, 2);
    let mut ids = BTreeSet::new();
    let mut fonts = Vec::new();
    for (page, section) in document.sections.iter().enumerate() {
        for control in &section.paragraphs[0].controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            assert!(ids.insert(shape.common().instance_id));
            let tb = shape.drawing().unwrap().text_box.as_ref().unwrap();
            assert!(tb.text_margin_explicit);
            for paragraph in &tb.paragraphs {
                assert!(usize::from(paragraph.para_shape_id) < document.doc_info.para_shapes.len());
                for run in &paragraph.char_shapes {
                    let style = &document.doc_info.char_shapes[run.char_shape_id as usize];
                    let font = &document.doc_info.font_faces[0][style.font_ids[0] as usize].name;
                    assert_eq!(font, if page == 0 { "Helvetica" } else { "Courier" });
                    // Width/style runs may divide one source line; every run
                    // must still resolve to the declared font on its own page.
                    if fonts.last() != Some(font) {
                        fonts.push(font.clone());
                    }
                }
            }
        }
    }
    assert_eq!(fonts, ["Helvetica", "Courier"]);
    assert_eq!(
        imported
            .metadata
            .pages
            .iter()
            .map(|p| p.source_page)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(!imported.metadata.parser_allocation_bounds_proven);
}

#[test]
fn selection_is_explicit_and_keeps_source_page_mapping() {
    let source = pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")]);
    let imported = parse_pdf_with_options(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![1]),
            ..PdfImportOptions::default()
        },
    )
    .unwrap();
    assert_eq!(texts(&imported.document), ["SECOND"]);
    assert_eq!(imported.metadata.source_page_count, 2);
    assert_eq!(imported.metadata.pages[0].source_page, 1);
    assert_eq!(imported.metadata.pages[0].output_section, 0);
    for pages in [vec![], vec![2], vec![0, 0], vec![1, 0]] {
        let error = parse_pdf_with_options(
            &source,
            &PdfImportOptions {
                source_pages: Some(pages),
                ..PdfImportOptions::default()
            },
        )
        .err()
        .unwrap();
        assert_eq!(error.code, "PDF_PAGE_SELECTION");
    }
}

#[test]
fn source_indices_are_independent_of_the_selected_page_ceiling() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    let source = pdf(&vec![("PAGE", "Helvetica"); 162]);
    let all = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    assert_eq!(all.metadata.pages.len(), 162);
    let policy = PdfImportPolicy {
        max_selected_pages: 2,
        ..PdfImportPolicy::default()
    };
    let selected = parse_pdf_with_policy(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![128, 161]),
            ..PdfImportOptions::default()
        },
        policy,
    )
    .unwrap();
    assert_eq!(selected.metadata.source_page_count, 162);
    assert_eq!(
        selected
            .metadata
            .pages
            .iter()
            .map(|p| p.source_page)
            .collect::<Vec<_>>(),
        [128, 161]
    );
    assert_eq!(selected.document.sections.len(), 2);
    let error = parse_pdf_with_policy(&source, &PdfImportOptions::default(), policy)
        .err()
        .unwrap();
    assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
    assert_eq!(
        error.diagnostics.unwrap().frontend_limit.unwrap().reason,
        "selected pages"
    );
    for pages in [vec![162], vec![161, 128], vec![128, 128], vec![0, 128, 161]] {
        assert_eq!(
            parse_pdf_with_policy(
                &source,
                &PdfImportOptions {
                    source_pages: Some(pages),
                    ..PdfImportOptions::default()
                },
                policy
            )
            .err()
            .unwrap()
            .code,
            "PDF_PAGE_SELECTION"
        );
    }
}

#[test]
fn input_policy_accepts_exact_limit_and_rejects_before_parsing_over_limit() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    let source = pdf(&[("BOUNDARY", "Helvetica")]);
    for extra in [0, 1] {
        let mut policy = PdfImportPolicy::default();
        policy.backend.input_bytes = source.len() + extra;
        let imported =
            parse_pdf_with_policy(&source, &PdfImportOptions::default(), policy).unwrap();
        assert_eq!(texts(&imported.document), ["BOUNDARY"]);
    }
    let mut policy = PdfImportPolicy::default();
    policy.backend.input_bytes = source.len() - 1;
    let error = parse_pdf_with_policy(&source, &PdfImportOptions::default(), policy)
        .err()
        .unwrap();
    let failure = error.diagnostics.unwrap();
    assert!(
        failure.resource_usage.is_none(),
        "reject before constructor/copy"
    );
    let limit = failure.frontend_limit.unwrap();
    assert_eq!(
        (limit.used, limit.requested, limit.limit),
        (
            Some(0),
            Some(source.len() as u64),
            Some((source.len() - 1) as u64)
        )
    );
    for invalid in [0, u16::MAX as usize + 1] {
        policy.max_selected_pages = invalid;
        assert!(parse_pdf_with_policy(&source, &PdfImportOptions::default(), policy).is_err());
    }
}

#[test]
fn invalid_or_over_budget_input_does_not_poison_next_import() {
    for source in [
        b"not pdf".to_vec(),
        b"%PDF-1.4\ninvalid".to_vec(),
        vec![0; rhwp::parser::pdf::MAX_INPUT_BYTES + 1],
    ] {
        assert!(parse_pdf_with_options(&source, &PdfImportOptions::default()).is_err());
    }
    let imported =
        parse_pdf_with_options(&pdf(&[("OK", "Helvetica")]), &PdfImportOptions::default()).unwrap();
    assert_eq!(texts(&imported.document), ["OK"]);
}

#[test]
fn successful_import_reports_consumed_policy_and_request_local_work() {
    let source = pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")]);
    let imported = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    let metadata = serde_json::to_value(&imported.metadata).unwrap();
    assert_eq!(metadata["resourcePolicy"]["schemaVersion"], 1);
    assert_eq!(
        metadata["resourcePolicy"]["backend"]["inputBytes"],
        64 * 1024 * 1024
    );
    assert_eq!(
        metadata["resourceUsage"]["failure"],
        serde_json::Value::Null
    );
    assert!(metadata["resourceUsage"]["operators"].as_u64().unwrap() > 0);
    assert!(
        metadata["resourceUsage"]["contentParseCalls"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        metadata["resourceUsage"]["pageOperatorCacheHits"]
            .as_u64()
            .unwrap()
            > 0
    );
    let next = parse_pdf_with_options(&source, &PdfImportOptions::default()).unwrap();
    assert_eq!(
        serde_json::to_value(next.metadata.resource_usage).unwrap(),
        metadata["resourceUsage"]
    );
}

#[test]
fn positional_selection_never_extracts_unselected_pages_for_header_classification() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    let first = "BT /F1 12 Tf 20 100 Td (SELECTED) Tj ET";
    let second = format!("{} BT /F1 12 Tf 20 100 Td (OTHER) Tj ET", "q Q ".repeat(30));
    let objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Count 2 /Kids [3 0 R 5 0 R] >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 7 0 R >> >> /Contents 4 0 R >>".into(),
        format!("<< /Length {} >>\nstream\n{first}\nendstream", first.len()),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>".into(),
        format!("<< /Length {} >>\nstream\n{second}\nendstream", second.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    let source = encode_pdf(&objects);
    let mut policy = PdfImportPolicy::default();
    policy.backend.operators = 8;
    let result = parse_pdf_with_policy(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![0]),
            ..PdfImportOptions::default()
        },
        policy,
    )
    .unwrap();
    assert_eq!(texts(&result.document), ["SELECTED"]);
    assert_eq!(result.metadata.resource_usage.content_parse_calls, 1);
    assert_eq!(result.metadata.resource_usage.operators, 5);
    // The operator cap is independent of the explicit reference cap.
    assert!(result.metadata.resource_usage.reference_visits > policy.backend.operators);
    let mut reference_policy = policy;
    reference_policy.backend.reference_visits = 8;
    let error = parse_pdf_with_policy(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![0]),
            ..Default::default()
        },
        reference_policy,
    )
    .err()
    .expect("the explicit reference cap still applies");
    assert_eq!(error.message, "reference visits");
    assert!(parse_pdf_with_policy(
        &source,
        &PdfImportOptions {
            source_pages: Some(vec![1]),
            ..PdfImportOptions::default()
        },
        policy
    )
    .is_err());
}

#[test]
fn blank_pages_are_not_silently_dropped() {
    let imported = parse_pdf_with_options(
        &pdf(&[("", "Helvetica"), ("OK", "Helvetica")]),
        &PdfImportOptions::default(),
    )
    .unwrap();
    assert_eq!(imported.document.sections.len(), 2);
    assert_eq!(texts(&imported.document), ["", "OK"]);
    assert!(imported.metadata.pages[0].source_spans.is_empty());
}

#[test]
fn automatic_parser_and_core_share_pdf_origin_and_page_mapping() {
    use rhwp::{parser, DocumentCore};
    let source = pdf(&[("FIRST", "Helvetica"), ("SECOND", "Courier")]);
    let parsed = parser::parse_document_with_metadata(&source).unwrap();
    assert!(parsed.hml_metadata.is_none());
    assert_eq!(texts(&parsed.document), ["FIRST", "SECOND"]);
    assert_eq!(parsed.document.provenance.format, SourceFormat::Pdf);
    let metadata = parsed.pdf_metadata.unwrap();
    assert_eq!(metadata.source_page_count, 2);
    assert_eq!(
        metadata
            .pages
            .iter()
            .map(|p| (p.source_page, p.output_section))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 1)]
    );
    assert!(!metadata.parser_allocation_bounds_proven);
    for core in [
        DocumentCore::from_bytes(&source).unwrap(),
        DocumentCore::from_bytes_with_password(&source, b"unused").unwrap(),
    ] {
        assert_eq!(texts(core.document()), ["FIRST", "SECOND"]);
        assert_eq!(core.document().provenance.format, SourceFormat::Pdf);
        assert!(core.hml_metadata().is_none());
        assert_eq!(core.pdf_metadata().unwrap().pages.len(), 2);
        assert_eq!(core.page_count(), 2);
        for (page, text) in [(0, "FIRST"), (1, "SECOND")] {
            let svg = core.render_page_svg_native(page).unwrap();
            // SVG may split one word into per-glyph <text> elements. Check the
            // visible text sequence, not incidental XML serialization grouping.
            let xml = roxmltree::Document::parse(&svg).unwrap();
            let rendered: String = xml
                .descendants()
                .filter(|node| {
                    node.is_text() && node.ancestors().any(|parent| parent.has_tag_name("text"))
                })
                .filter_map(|node| node.text())
                .collect();
            assert_eq!(rendered, text, "source text order on page {page}");
        }
    }
}

#[test]
fn selected_core_uses_common_loader_and_saved_documents_keep_selected_content() {
    use rhwp::{parser, DocumentCore};
    let source = pdf(&[
        ("FIRST", "Helvetica"),
        ("SECOND", "Courier"),
        ("THIRD", "Helvetica"),
    ]);
    let options = PdfImportOptions {
        source_pages: Some(vec![0, 2]),
        ..PdfImportOptions::default()
    };
    let parsed = parser::parse_pdf_with_metadata(&source, &options).unwrap();
    let mut core = DocumentCore::from_pdf_bytes(&source, &options).unwrap();
    assert_eq!(texts(&parsed.document), ["FIRST", "THIRD"]);
    assert_eq!(texts(core.document()), texts(&parsed.document));
    assert_eq!(core.page_count(), 2);
    let metadata = core.pdf_metadata().unwrap();
    assert_eq!(metadata.source_page_count, 3);
    assert_eq!(
        metadata
            .pages
            .iter()
            .map(|p| (p.source_page, p.output_section))
            .collect::<Vec<_>>(),
        [(0, 0), (2, 1)]
    );
    for bytes in [
        core.export_hwpx_native().unwrap(),
        core.export_hwp_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(texts(reopened.document()), ["FIRST", "THIRD"]);
        assert!(reopened.pdf_metadata().is_none());
        assert_ne!(reopened.document().provenance.format, SourceFormat::Pdf);
    }
    core.create_blank_document_native().unwrap();
    assert!(
        core.pdf_metadata().is_none(),
        "new document must not retain PDF diagnostics"
    );
}

#[test]
fn common_loader_errors_keep_pdf_codes_and_do_not_poison_later_imports() {
    use rhwp::{parser, DocumentCore};
    let source = pdf(&[("OK", "Helvetica")]);
    let broken = b"%PDF-1.4\ninvalid";
    assert!(matches!(
        parser::parse_document(broken),
        Err(parser::ParseError::PdfError(_))
    ));
    let error = DocumentCore::from_bytes(broken).err().unwrap().to_string();
    assert!(error.contains("PDF_PARSE_FAILED"), "{error}");
    for pages in [vec![], vec![1], vec![0, 0]] {
        let error = DocumentCore::from_pdf_bytes(
            &source,
            &PdfImportOptions {
                source_pages: Some(pages),
                ..PdfImportOptions::default()
            },
        )
        .err()
        .unwrap()
        .to_string();
        assert!(error.contains("PDF_PAGE_SELECTION"), "{error}");
    }
    let core = DocumentCore::from_bytes(&source).unwrap();
    assert_eq!(texts(core.document()), ["OK"]);
    assert_eq!(core.page_count(), 1);
}

#[test]
fn service_exposes_pdf_format_without_a_synthetic_hwp_version() {
    let source = pdf(&[("PDF CONTENT", "Helvetica")]);
    let opened = rhwp::service::DocumentService::new()
        .open_bytes(&source, &rhwp::service::OpenOptions::new())
        .unwrap();
    assert_eq!(opened.format(), rhwp::parser::FileFormat::Pdf);
    let info = opened.info();
    assert_eq!(info.format, "pdf");
    assert!(info.version.is_none());
    assert_eq!(texts(opened.document()), ["PDF CONTENT"]);
    assert!(opened.into_core().pdf_metadata().is_some());
}

// #8415: authentication/byte-preservation contracts, not layout coordinates.
// Public pikepdf controls and independent creation: pdf/issue8415/authentication/README.md.
fn password_control_8415(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue8415/authentication")
            .join(format!("{name}.pdf")),
    )
    .unwrap()
}

#[test]
fn password_pdf_authentication_keeps_plaintext_and_images_in_both_save_formats() {
    use rhwp::{parser, DocumentCore};
    let plain = DocumentCore::from_bytes(&password_control_8415("password-control")).unwrap();
    let expected_text = texts(plain.document());
    let expected_image = plain.document().bin_data_content[0].data.load().to_vec();
    for revision in 2..=6 {
        let input = password_control_8415(&format!("password-r{revision}"));
        let missing = parse_pdf_with_options(&input, &Default::default())
            .err()
            .unwrap();
        assert_eq!(missing.code, "PDF_PASSWORD_REQUIRED", "R{revision}");
        let wrong = parse_pdf_with_options(
            &input,
            &PdfImportOptions {
                password: Some("wrong".into()),
                ..Default::default()
            },
        )
        .err()
        .unwrap();
        assert_eq!(wrong.code, "PDF_PASSWORD_INVALID", "R{revision}");
        for password in [b"synthetic-open".as_slice(), b"synthetic-owner".as_slice()] {
            let core = DocumentCore::from_bytes_with_password(&input, password).unwrap();
            assert_eq!(
                texts(core.document()),
                expected_text,
                "R{revision}: authenticated content"
            );
            assert_eq!(
                core.document().bin_data_content[0].data.load().to_vec(),
                expected_image
            );
            for bytes in [
                core.export_hwp_native().unwrap(),
                core.export_hwpx_native().unwrap(),
            ] {
                let reopened = parser::parse_document(&bytes).unwrap();
                assert_eq!(
                    texts(&reopened),
                    expected_text,
                    "R{revision}: saved plaintext"
                );
                assert_eq!(
                    reopened.bin_data_content[0].data.load().to_vec(),
                    expected_image
                );
            }
        }
    }
}

#[test]
fn password_pdf_unicode_empty_permissions_and_resource_rejection_are_request_local() {
    use rhwp::parser::pdf::{parse_pdf_with_policy, PdfImportPolicy};
    use rhwp::DocumentCore;
    let expected = texts(
        DocumentCore::from_bytes(&password_control_8415("password-control"))
            .unwrap()
            .document(),
    );
    for (name, password) in [
        ("password-unicode", "합성-암호-8415".to_owned()),
        ("password-long-unicode", "가".repeat(43)),
        ("password-metadata-clear", "synthetic-open".to_owned()),
        ("password-padded-fields", "synthetic-open".to_owned()),
        ("password-padded-fields", "synthetic-owner".to_owned()),
        ("password-empty", String::new()),
    ] {
        let core = DocumentCore::from_bytes_with_password(
            &password_control_8415(name),
            password.as_bytes(),
        )
        .unwrap();
        assert_eq!(texts(core.document()), expected, "{name}");
    }
    let input = password_control_8415("password-r6");
    let options = PdfImportOptions {
        password: Some("synthetic-open".into()),
        ..Default::default()
    };
    assert!(!format!("{options:?}").contains("synthetic-open"));
    // Keep xref offsets intact while independently corrupting the encryption
    // contract. These are admission tests, not alternate valid algorithms.
    for (before, after, code) in [
        (
            "/Filter /Standard",
            "/Filter /PubSecXX",
            "PDF_ENCRYPTION_UNSUPPORTED",
        ),
        ("/R 6", "/R 9", "PDF_ENCRYPTION_UNSUPPORTED"),
        ("/CFM /AESV3", "/CFM /AESV9", "PDF_ENCRYPTION_UNSUPPORTED"),
        ("/U <", "/Z <", "PDF_PARSE_FAILED"),
    ] {
        assert_eq!(before.len(), after.len());
        let start = input
            .windows(before.len())
            .position(|w| w == before.as_bytes())
            .unwrap();
        let mut invalid = input.clone();
        invalid[start..start + before.len()].copy_from_slice(after.as_bytes());
        assert_eq!(
            parse_pdf_with_options(&invalid, &options)
                .err()
                .unwrap()
                .code,
            code
        );
    }
    let policy = PdfImportPolicy::default()
        .with_limits_json(r#"{"backend":{"decodedBytes":1048576}}"#)
        .unwrap();
    assert_eq!(
        parse_pdf_with_policy(&input, &options, policy)
            .err()
            .unwrap()
            .code,
        "PDF_RESOURCE_LIMIT"
    );
    assert_eq!(
        texts(&parse_pdf_with_options(&input, &options).unwrap().document),
        expected
    );
    // The original empty-user-password PDF has restrictive copy/edit flags.
    assert_eq!(
        texts(
            &parse_pdf_with_options(
                &password_control_8415("password-empty"),
                &Default::default()
            )
            .unwrap()
            .document
        ),
        expected
    );
}
