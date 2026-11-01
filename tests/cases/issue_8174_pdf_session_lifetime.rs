//! Public request lifetime, first-save and recovery contracts.
#![cfg(feature = "pdf-import")]
use rhwp::{
    parser::pdf::{PdfImportOptions, PdfImportPolicy},
    DocumentCore, PdfImportSession,
};
const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");

#[test]
fn initialization_uses_parser_budget_and_editing_owns_its_lifetime() {
    for layout in [
        rhwp::parser::pdf::PdfImportLayout::Fidelity,
        rhwp::parser::pdf::PdfImportLayout::Editable,
    ] {
        let policy = PdfImportPolicy::default();
        let options = PdfImportOptions {
            layout,
            ..Default::default()
        };
        let parsed = rhwp::parser::pdf::parse_pdf_with_policy(PDF, &options, policy).unwrap();
        let mut request = PdfImportSession::new(policy).unwrap();
        let mut core = request.open(PDF, &options).unwrap();
        let used = request.resource_usage().decoded_bytes;
        assert!(used > parsed.metadata.resource_usage.decoded_bytes);
        assert_eq!(
            core.pdf_metadata().unwrap().resource_usage.decoded_bytes,
            used
        );
        core.reflow_linesegs_on_demand();
        core.set_hangul2024_compat(true);
        assert_eq!(request.resource_usage().decoded_bytes, used);
        assert!(request.open(PDF, &options).is_err());
        let mut limited = policy;
        limited.backend.decoded_bytes = parsed.metadata.resource_usage.decoded_bytes;
        let mut refused = PdfImportSession::new(limited).unwrap();
        let error = refused
            .open(PDF, &options)
            .err()
            .expect("bounded initialization must refuse")
            .to_string();
        assert!(error.contains("PDF_RESOURCE_LIMIT:"), "{error}");
        assert!(refused.open(PDF, &options).is_err());
        let mut fresh = PdfImportSession::new(policy).unwrap();
        assert_eq!(
            fresh.open(PDF, &options).unwrap().page_count(),
            core.page_count()
        );
    }
}

#[test]
fn hwp_save_and_exact_diagnostics_share_budget_preserve_live_document_and_recover() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let ordinary = core.export_hwp_native().unwrap();
    let before = request.resource_usage().decoded_bytes;
    let output = request.export_hwp(&mut core, 1024 * 1024).unwrap();
    assert_eq!(output, ordinary);
    assert!(request.resource_usage().decoded_bytes > before);
    assert_eq!(DocumentCore::from_bytes(&output).unwrap().page_count(), 1);
    let before = request.resource_usage().decoded_bytes;
    let metadata = request.diagnostics_json(&core, 1024 * 1024).unwrap();
    assert_eq!(
        request.resource_usage().decoded_bytes,
        before + metadata.len() as u64
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
    assert_eq!(metadata["pageCount"], 1);
    assert_eq!(
        metadata["diagnostics"]["resourceUsage"]["decodedBytes"],
        before
    );
    assert!(request.export_hwp(&mut core, 512).is_err());
    let failed = request.resource_usage().decoded_bytes;
    assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed);
    assert_eq!(core.export_hwp_native().unwrap(), ordinary);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    assert_eq!(fresh.export_hwp(&mut core, 1024 * 1024).unwrap(), ordinary);
}

#[test]
fn first_save_snapshot_is_a_budgeted_public_export_without_live_ir_changes() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let ordinary = core.prepare_hwp_export_snapshot().serialize().unwrap();
    let before = request.resource_usage().decoded_bytes;
    let bounded = request.export_hwp(&mut core, 1024 * 1024).unwrap();
    assert!(request.resource_usage().decoded_bytes > before);
    assert_eq!(bounded, ordinary);
    assert_eq!(
        core.prepare_hwp_export_snapshot().serialize().unwrap(),
        ordinary
    );
}

#[test]
fn first_hwp_and_hwpx_saves_refuse_the_public_remaining_budget() {
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    baseline.open(PDF, &Default::default()).unwrap();
    // Leave room for policy metadata's different numeric spelling, but much
    // less than the measured first-save work. No private mid-request override.
    let cap = baseline.resource_usage().decoded_bytes + 1024;
    for hwpx in [false, true] {
        let mut policy = PdfImportPolicy::default();
        policy.backend.decoded_bytes = cap;
        let mut request = PdfImportSession::new(policy).unwrap();
        let mut core = request.open(PDF, &Default::default()).unwrap();
        let ordinary = if hwpx {
            core.export_hwpx_native()
        } else {
            core.export_hwp_native()
        }
        .unwrap();
        let result = if hwpx {
            request.export_hwpx(&mut core, 1024 * 1024)
        } else {
            request.export_hwp(&mut core, 1024 * 1024)
        };
        assert!(result.is_err());
        let failed = request.resource_usage();
        assert!(failed.failure.is_some());
        assert!(failed.decoded_bytes <= cap);
        assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
        assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
        assert_eq!(
            if hwpx {
                core.export_hwpx_native()
            } else {
                core.export_hwp_native()
            }
            .unwrap(),
            ordinary
        );
    }
}

#[test]
fn hwpx_output_rejection_preserves_live_ir_and_a_fresh_request_recovers() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let ordinary = core.export_hwpx_native().unwrap();
    assert_eq!(
        request.export_hwpx(&mut core, 1024 * 1024).unwrap(),
        ordinary
    );
    assert_eq!(
        core.pdf_metadata().unwrap().resource_usage.decoded_bytes,
        request.resource_usage().decoded_bytes
    );
    assert!(request.export_hwpx(&mut core, 1).is_err());
    let failed = request.resource_usage().decoded_bytes;
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed);
    assert_eq!(core.export_hwpx_native().unwrap(), ordinary);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut recovered = fresh.open(PDF, &Default::default()).unwrap();
    assert_eq!(
        fresh.export_hwpx(&mut recovered, 1024 * 1024).unwrap(),
        ordinary
    );
}

#[test]
fn diagnostics_output_refusal_and_invalid_input_close_the_request() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let before = request.resource_usage().decoded_bytes;
    assert!(request.diagnostics_json(&core, 1).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, before);
    assert!(request.diagnostics_json(&core, 1024 * 1024).is_err());
    let mut invalid = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    assert!(invalid.open(b"not a PDF", &Default::default()).is_err());
    assert!(invalid.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert!(invalid.open(PDF, &Default::default()).is_err());
}

#[test]
fn initialization_refusal_is_fatal_and_new_request_recovers() {
    let policy = PdfImportPolicy::default();
    let options = PdfImportOptions::default();
    let parsed = rhwp::parser::pdf::parse_pdf_with_policy(PDF, &options, policy).unwrap();
    let mut lowered = policy;
    lowered.backend.decoded_bytes = parsed.metadata.resource_usage.decoded_bytes;
    assert!(rhwp::parser::pdf::parse_pdf_with_policy(PDF, &options, lowered).is_ok());
    let mut request = PdfImportSession::new(lowered).unwrap();
    assert!(request.open(PDF, &options).is_err());
    assert!(request.resource_usage().failure.is_some());
    assert!(request.open(PDF, &options).is_err());
    let mut fresh = PdfImportSession::new(policy).unwrap();
    let core = fresh.open(PDF, &options).unwrap();
    let detected = DocumentCore::from_bytes(PDF).unwrap();
    assert_eq!(core.page_count(), detected.page_count());
    assert_eq!(
        core.pdf_metadata().unwrap().resource_usage.decoded_bytes,
        detected
            .pdf_metadata()
            .unwrap()
            .resource_usage
            .decoded_bytes
    );
}

#[test]
fn selected_page_retains_document_budget_and_explicit_lower_host_cap() {
    const SOURCE: &[u8] = include_bytes!("../../pdf/pr7704/6778-square-2020.pdf");
    let options = PdfImportOptions {
        source_pages: Some(vec![6]),
        include_table_inference: true,
        ..Default::default()
    };
    let mut normal = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = normal.open(SOURCE, &options).unwrap();
    let output = normal.export_hwp(&mut core, 32 * 1024 * 1024).unwrap();
    assert!(normal.resource_usage().decoded_bytes > 256 * 1024 * 1024);
    assert_eq!(DocumentCore::from_bytes(&output).unwrap().page_count(), 1);
    let mut capped = PdfImportPolicy::default();
    capped.backend.decoded_bytes = 64 * 1024 * 1024;
    let mut request = PdfImportSession::new(capped).unwrap();
    assert!(request.open(SOURCE, &options).is_err());
    assert!(request.resource_usage().failure.is_some());
    assert!(request.resource_usage().decoded_bytes <= capped.backend.decoded_bytes);
}

#[test]
fn initial_composition_refuses_remaining_request_work_and_fresh_open_recovers() {
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let active = healthy.open(PDF, &Default::default()).unwrap();
    let expected = active.export_hwp_native().unwrap();
    let parsed = rhwp::parser::pdf::parse_pdf_with_policy(
        PDF,
        &Default::default(),
        PdfImportPolicy::default(),
    )
    .unwrap();
    let full = healthy.resource_usage().decoded_bytes;
    let start = parsed.metadata.resource_usage.decoded_bytes;
    let mut exercised = false;
    for cap in (start..full).step_by(64) {
        let mut policy = PdfImportPolicy::default();
        policy.backend.decoded_bytes = cap;
        let mut request = PdfImportSession::new(policy).unwrap();
        if let Err(error) = request.open(PDF, &Default::default()) {
            if error.to_string().contains("PDF composition:") {
                let failed = request.resource_usage();
                assert!(failed.failure.is_some());
                assert!(failed.decoded_bytes <= cap);
                assert!(request.open(PDF, &Default::default()).is_err());
                assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
                exercised = true;
                break;
            }
        }
    }
    assert!(
        exercised,
        "initial composition never refused its remaining request work"
    );
    assert_eq!(active.export_hwp_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    assert_eq!(
        fresh
            .open(PDF, &Default::default())
            .unwrap()
            .export_hwp_native()
            .unwrap(),
        expected
    );
}

#[test]
fn hwpx_context_refuses_remaining_budget_and_new_request_recovers() {
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let active = baseline.open(PDF, &Default::default()).unwrap();
    let expected_hwp = active.export_hwp_native().unwrap();
    let expected_hwpx = active.export_hwpx_native().unwrap();
    let initial = baseline.resource_usage().decoded_bytes;
    let mut observed = false;
    for extra in (64..2048).step_by(64) {
        let cap = initial + extra;
        let mut policy = PdfImportPolicy::default();
        policy.backend.decoded_bytes = cap;
        let mut request = PdfImportSession::new(policy).unwrap();
        let Ok(mut core) = request.open(PDF, &Default::default()) else {
            continue;
        };
        let Err(error) = request.export_hwpx(&mut core, 1024 * 1024) else {
            continue;
        };
        if !error.to_string().contains("HWPX context:") {
            continue;
        }
        assert!(request.resource_usage().failure.is_some());
        assert!(request.resource_usage().decoded_bytes <= cap);
        assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
        assert_eq!(core.export_hwp_native().unwrap(), expected_hwp);
        observed = true;
        break;
    }
    assert!(
        observed,
        "first-save context never refused its remaining request budget"
    );
    assert_eq!(active.export_hwp_native().unwrap(), expected_hwp);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    assert_eq!(
        fresh.export_hwpx(&mut core, 1024 * 1024).unwrap(),
        expected_hwpx
    );
}

#[test]
fn hwpx_header_buffer_refuses_before_large_raw_tail_and_fresh_request_recovers() {
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut small = baseline.open(PDF, &Default::default()).unwrap();
    baseline.export_hwpx(&mut small, 1024 * 1024).unwrap();
    let cap = baseline.resource_usage().decoded_bytes + 64 * 1024;
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = cap;
    // This cap accepts the ordinary header and compressed output.
    let mut ordinary = PdfImportSession::new(policy).unwrap();
    let mut ordinary_core = ordinary.open(PDF, &Default::default()).unwrap();
    ordinary
        .export_hwpx(&mut ordinary_core, 1024 * 1024)
        .unwrap();

    let tail = format!("<!--{}-->", "x".repeat(512 * 1024));
    let mut request = PdfImportSession::new(policy).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    core.document_mut().doc_info.hwpx_head_tail = Some(tail.clone());
    let expected = core.export_hwpx_native().unwrap();
    assert!(
        expected.len() < 128 * 1024,
        "compressed ZIP must fit the cap"
    );
    let result = request.export_hwpx(&mut core, 1024 * 1024);
    assert!(
        result.is_err(),
        "large uncompressed header bypassed the request budget"
    );
    let error = result.unwrap_err();
    assert!(error.to_string().contains("head tail splice:"), "{error}");
    let failed = request.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= cap);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    assert_eq!(
        core.document().doc_info.hwpx_head_tail.as_ref(),
        Some(&tail)
    );
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    core.document_mut().doc_info.hwpx_head_tail = Some(tail);
    assert_eq!(fresh.export_hwpx(&mut core, 1024 * 1024).unwrap(), expected);
}

#[test]
fn hwpx_attribute_encoding_refuses_expansion_before_writing_the_tag() {
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut small = healthy.open(PDF, &Default::default()).unwrap();
    healthy.export_hwpx(&mut small, 1024 * 1024).unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = healthy.resource_usage().decoded_bytes + 64 * 1024;
    let name = "&".repeat(256 * 1024);
    let mut request = PdfImportSession::new(policy).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    core.document_mut().doc_info.font_faces[0][0].name = name.clone();
    let expected = core.export_hwpx_native().unwrap();
    assert!(expected.len() < 128 * 1024);
    let result = request.export_hwpx(&mut core, 1024 * 1024);
    assert!(
        result.is_err(),
        "escaped font attribute bypassed the request budget"
    );
    let error = result.unwrap_err();
    assert!(error.to_string().contains("HWPX XML:"), "{error}");
    let failed = request.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(core.document().doc_info.font_faces[0][0].name, name);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    core.document_mut().doc_info.font_faces[0][0].name = name;
    assert_eq!(fresh.export_hwpx(&mut core, 1024 * 1024).unwrap(), expected);
}

fn exercise_hwpx_part_assembly_limit(master_page: bool) {
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut normal = baseline.open(PDF, &Default::default()).unwrap();
    baseline.export_hwpx(&mut normal, 1024 * 1024).unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = baseline.resource_usage().decoded_bytes + 64 * 1024;
    let mut request = PdfImportSession::new(policy).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let mut paragraph = core.document().sections[0].paragraphs[0].clone();
    paragraph.text = "<&".repeat(256 * 1024);
    paragraph.char_count = paragraph.text.encode_utf16().count() as u32 + 1;
    paragraph.controls.clear();
    if master_page {
        core.document_mut().sections[0]
            .section_def
            .master_pages
            .push(rhwp::model::header_footer::MasterPage {
                paragraphs: vec![paragraph],
                ..Default::default()
            });
    } else {
        core.document_mut().sections[0].paragraphs.push(paragraph);
    }
    let expected = core.export_hwpx_native().unwrap();
    assert!(
        expected.len() < 64 * 1024,
        "compressed ZIP must fit the remaining budget"
    );
    let result = request.export_hwpx(&mut core, 1024 * 1024);
    assert!(result.is_err(), "part assembly bypassed the request budget");
    assert!(result.unwrap_err().to_string().contains("HWPX XML:"));
    let failed = request.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut next = fresh.open(PDF, &Default::default()).unwrap();
    // Exercise the identical host-edited IR through a new request.
    *next.document_mut() = core.document().clone();
    assert_eq!(fresh.export_hwpx(&mut next, 1024 * 1024).unwrap(), expected);
}

#[test]
fn hwpx_section_assembly_refuses_remaining_work_and_fresh_request_recovers() {
    exercise_hwpx_part_assembly_limit(false);
}

#[test]
fn hwpx_master_page_assembly_refuses_remaining_work_and_fresh_request_recovers() {
    exercise_hwpx_part_assembly_limit(true);
}

fn exercise_hwpx_run_or_lineseg_budget(linesegs: bool) {
    let mut baseline = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut normal = baseline.open(PDF, &Default::default()).unwrap();
    baseline.export_hwpx(&mut normal, 1024 * 1024).unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = baseline.resource_usage().decoded_bytes + 64 * 1024;
    let mut request = PdfImportSession::new(policy).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let mut paragraph = rhwp::model::paragraph::Paragraph {
        text: "A".into(),
        char_count: 2,
        char_shapes: vec![rhwp::model::paragraph::CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![rhwp::model::paragraph::LineSeg {
            line_height: 1600,
            text_height: 1000,
            ..Default::default()
        }],
        ..Default::default()
    };
    let para = &mut paragraph;
    if linesegs {
        let mut row = para.line_segs[0].clone();
        row.text_start = 0;
        row.tag &= !rhwp::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        para.line_segs = vec![row; 50_000];
    } else {
        let shape = para.char_shapes[0].char_shape_id;
        para.char_shapes = vec![
            rhwp::model::paragraph::CharShapeRef {
                start_pos: 0,
                char_shape_id: shape
            };
            50_000
        ];
    }
    core.document_mut().sections[0].paragraphs.push(paragraph);
    let expected = core.export_hwpx_native().unwrap();
    let error = request.export_hwpx(&mut core, 1024 * 1024).unwrap_err();
    let phase = if linesegs {
        "HWPX lineseg:"
    } else {
        "HWPX run:"
    };
    assert!(
        error.to_string().contains(phase),
        "guard did not run inside {phase}: {error}"
    );
    let failed = request.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(request.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(request.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut next = fresh.open(PDF, &Default::default()).unwrap();
    *next.document_mut() = core.document().clone();
    assert_eq!(fresh.export_hwpx(&mut next, 1024 * 1024).unwrap(), expected);
}

#[test]
fn hwpx_run_planning_refuses_before_section_assembly() {
    exercise_hwpx_run_or_lineseg_budget(false);
}

#[test]
fn hwpx_lineseg_backing_refuses_before_section_assembly() {
    exercise_hwpx_run_or_lineseg_budget(true);
}

#[test]
fn pagination_failure_reaches_the_import_request_and_a_new_request_recovers() {
    let mut normal_request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let normal = normal_request.open(PDF, &Default::default()).unwrap();
    let expected_pages = normal.page_count();
    let expected = normal.export_hwpx_native().unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = normal_request.resource_usage().decoded_bytes - 1;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let error = match limited.open(PDF, &Default::default()) {
        Err(error) => error,
        Ok(_) => panic!("initial pagination escaped the request budget"),
    };
    // Additional initial-core reservations may exhaust this fixed historical
    // input policy before pagination. The public resource error is the contract.
    assert!(error.to_string().contains("PDF_RESOURCE_LIMIT:"), "{error}");
    let failed = limited.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(limited.open(PDF, &Default::default()).is_err());
    assert_eq!(limited.resource_usage().decoded_bytes, failed.decoded_bytes);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let core = fresh.open(PDF, &Default::default()).unwrap();
    assert_eq!(core.page_count(), expected_pages);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
}

#[test]
fn nested_table_xml_budget_failure_preserves_live_ir_and_recovers() {
    let mut normal_request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut normal = normal_request.open(PDF, &Default::default()).unwrap();
    normal_request
        .export_hwpx(&mut normal, 1024 * 1024)
        .unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = normal_request.resource_usage().decoded_bytes + 64 * 1024;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let mut core = limited.open(PDF, &Default::default()).unwrap();
    let mut nested = rhwp::model::table::Table {
        row_count: 1,
        col_count: 1,
        cells: vec![rhwp::model::table::Cell {
            width: 7200,
            height: 7200,
            paragraphs: vec![rhwp::model::paragraph::Paragraph {
                text: "<&".repeat(32_768),
                char_count: 65_537,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    nested.rebuild_grid();
    core.document_mut().sections[0].paragraphs[0]
        .controls
        .push(rhwp::model::control::Control::Table(Box::new(nested)));
    let expected = core.export_hwpx_native().unwrap();
    assert!(limited.export_hwpx(&mut core, 1024 * 1024).is_err());
    let failed = limited.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(limited.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(limited.resource_usage().decoded_bytes, failed.decoded_bytes);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut next = fresh.open(PDF, &Default::default()).unwrap();
    *next.document_mut() = core.document().clone();
    assert_eq!(fresh.export_hwpx(&mut next, 1024 * 1024).unwrap(), expected);
}

#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn exercise_measurement_request_limit(source: &[u8], decoded_limit: usize) {
    // Each cap is a recorded pre-fix Native initialization measurement plus
    // an allowance recorded with its regression. Preserve the input policy;
    // later producers may reject earlier as more reservations are connected.
    // It is an input policy, not an exact expected value for today's counter.
    let mut normal_request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let normal = normal_request.open(source, &Default::default()).unwrap();
    let expected_pages = normal.page_count();
    let expected = normal.export_hwpx_native().unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = decoded_limit as u64;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let error = match limited.open(source, &Default::default()) {
        Err(error) => error,
        Ok(_) => panic!("measurement producers escaped the remaining request budget"),
    };
    // Additional initial-core reservations may exhaust this fixed historical
    // input policy before pagination. The public resource error is the contract.
    assert!(error.to_string().contains("PDF_RESOURCE_LIMIT:"), "{error}");
    let failed = limited.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(limited.open(source, &Default::default()).is_err());
    assert_eq!(limited.resource_usage().decoded_bytes, failed.decoded_bytes);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let core = fresh.open(source, &Default::default()).unwrap();
    assert_eq!(core.page_count(), expected_pages);
    assert_eq!(core.export_hwpx_native().unwrap(), expected);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn nested_cell_measurement_shares_the_initial_request_and_recovers() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_739_954 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn multiline_cell_measurement_shares_the_initial_request_and_recovers() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        7_556_074 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn stored_cell_frame_admission_uses_the_initial_request_budget() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_820_188 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn multiline_stored_frame_uses_the_initial_request_budget() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        7_606_256 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn stored_row_metric_probes_reserve_within_the_import_request() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        7_606_720 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn body_frame_metric_probes_share_the_import_request_and_recover() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/paragraph-origins.pdf"),
        7_640_617 + 64,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn render_normalization_uses_the_initial_request_and_recovers() {
    exercise_measurement_request_limit(PDF, 1_626_516 + 64);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn nested_table_normalization_uses_the_initial_request_and_recovers() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_824_434 + 64,
    );
}

#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn attach_text_clip(core: &mut rhwp::DocumentCore) {
    use rhwp::model::text_clip::{ClipCommand, TextClipPath};
    use rhwp::model::{
        control::Control,
        paragraph::Paragraph,
        shape::{ShapeObject, TextBox},
    };
    let mut shape = ShapeObject::Rectangle(Default::default());
    shape.common_mut().instance_id = 817455;
    shape.common_mut().width = 7200;
    shape.common_mut().height = 7200;
    shape.drawing_mut().unwrap().text_box = Some(TextBox {
        paragraphs: vec![Paragraph {
            text: "clip owner".into(),
            ..Default::default()
        }],
        paint_clip: Some(vec![TextClipPath {
            commands: vec![
                ClipCommand::Move(0., 0.),
                ClipCommand::Line(7200., 0.),
                ClipCommand::Line(7200., 7200.),
                ClipCommand::Line(0., 7200.),
                ClipCommand::Close,
            ],
            even_odd: false,
        }]),
        ..Default::default()
    });
    core.document_mut().sections[0].paragraphs[0]
        .controls
        .push(Control::Shape(Box::new(shape)));
}

#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn exercise_metadata_save_limit(hwp: bool, decoded_limit: usize) {
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut original = healthy.open(PDF, &Default::default()).unwrap();
    // Trusted host edits are legal between open and the first bounded save.
    // This tests the storage lifetime, not the layout of a synthetic fixture.
    attach_text_clip(&mut original);
    let expected = if hwp {
        original.export_hwp_with_adapter_snapshot_with_report()
    } else {
        original.export_hwpx_native_with_report()
    }
    .unwrap();
    assert_eq!(expected.content_loss().len(), 1);
    let loss = &expected.content_loss().losses()[0];
    assert_eq!(
        loss.code,
        rhwp::serializer::ContentLossCode::MetadataReduced
    );
    assert_eq!(
        loss.reason,
        rhwp::serializer::ContentLossReason::UnsupportedByOutputFormat
    );
    assert!(loss.path.contains("텍스트 클립"));
    let healthy_bytes = if hwp {
        healthy.export_hwp(&mut original, 1024 * 1024)
    } else {
        healthy.export_hwpx(&mut original, 1024 * 1024)
    }
    .unwrap();
    assert_eq!(healthy_bytes, expected.bytes());
    let mut policy = PdfImportPolicy::default();
    // S54's measured Native64 complete-save work plus 64 bytes. This remains
    // an input-policy boundary; it does not pin today's exact counter value.
    policy.backend.decoded_bytes = decoded_limit as u64;
    let mut limited = PdfImportSession::new(policy).unwrap();
    let mut core = limited.open(PDF, &Default::default()).unwrap();
    attach_text_clip(&mut core);
    let saved = if hwp {
        limited.export_hwp(&mut core, 1024 * 1024)
    } else {
        limited.export_hwpx(&mut core, 1024 * 1024)
    };
    assert!(
        saved.is_err(),
        "first-save metadata escaped the request budget"
    );
    let failed = limited.resource_usage();
    assert!(failed.failure.is_some());
    assert!(failed.decoded_bytes <= policy.backend.decoded_bytes);
    assert!(limited.export_hwpx(&mut core, 1024 * 1024).is_err());
    assert_eq!(limited.resource_usage().decoded_bytes, failed.decoded_bytes);
    let after = if hwp {
        core.export_hwp_with_adapter_snapshot_with_report()
    } else {
        core.export_hwpx_native_with_report()
    }
    .unwrap();
    assert_eq!(after.bytes(), expected.bytes());
    assert_eq!(after.content_loss(), expected.content_loss());
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut next = fresh.open(PDF, &Default::default()).unwrap();
    attach_text_clip(&mut next);
    let recovered = if hwp {
        fresh.export_hwp(&mut next, 1024 * 1024)
    } else {
        fresh.export_hwpx(&mut next, 1024 * 1024)
    }
    .unwrap();
    assert_eq!(recovered, expected.bytes());
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn first_hwp_metadata_save_shares_the_request_and_recovers() {
    exercise_metadata_save_limit(true, 1_690_710 + 64);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn first_hwpx_metadata_save_shares_the_request_and_recovers() {
    exercise_metadata_save_limit(false, 2_049_973 + 64);
}

#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn exercise_paginator_process_limit(source: &[u8], decoded_limit: usize) {
    // Keep RHWP_USE_PAGINATOR isolated from concurrently running ordinary tests.
    if std::env::var_os("RHWP_8174_PAGINATOR_BUDGET_CHILD").is_none() {
        let name = std::thread::current().name().unwrap().to_owned();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env("RHWP_USE_PAGINATOR", "1")
            .env("RHWP_8174_PAGINATOR_BUDGET_CHILD", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "isolated Paginator request failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return;
    }
    assert_eq!(std::env::var("RHWP_USE_PAGINATOR").unwrap(), "1");
    // S55's measured fallback initialization plus 512 bytes. The allowance
    // exceeds the added Core column scratch, so the actual Paginator must run.
    exercise_measurement_request_limit(source, decoded_limit);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn fallback_paginator_state_uses_the_request_budget_and_recovers() {
    exercise_paginator_process_limit(PDF, 1_626_908 + 512);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn fallback_nested_table_state_uses_the_request_budget_and_recovers() {
    exercise_paginator_process_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_824_954 + 512,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_page_events_share_the_request_budget_and_recover() {
    // S57's independent event-budget policy. Healthy requests still execute
    // page events; later state reservations may reject this cap earlier.
    exercise_measurement_request_limit(PDF, 1_626_940 + 512);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_nested_context_shares_the_request_budget_and_recovers() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_824_986 + 512,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_page_state_uses_the_request_budget_and_recovers() {
    // S57's initial work plus room for context creation. No counters are pinned.
    exercise_measurement_request_limit(PDF, 1_627_804 + 512);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_nested_page_state_uses_the_request_budget_and_recovers() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_826_906 + 512,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_paragraph_metrics_use_the_request_budget_and_recover() {
    // S58's initial work plus 16 bytes. This input policy catches newly
    // accounted paragraph work without pinning today's usage or refusal site.
    exercise_measurement_request_limit(PDF, 1_633_724 + 16);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn main_typeset_table_inline_work_uses_the_request_budget_and_recovers() {
    // S59 initialization plus a small allowance, independent of the current
    // counter value and of which connected producer refuses first.
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        7_614_879 + 16,
    );
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn ordinary_paragraph_state_uses_the_initial_request_and_recovers() {
    // S60's independently recorded initialization plus a small allowance.
    // The input policy detects missing State/column backing reservations;
    // neither today's counter nor the first refusing producer is pinned.
    exercise_measurement_request_limit(PDF, 1_633_772 + 16);
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), target_pointer_width = "64"))]
fn table_format_and_state_use_the_initial_request_and_recover() {
    exercise_measurement_request_limit(
        include_bytes!("../../pdf/issue7998/cell-nested.pdf"),
        1_832_874 + 16,
    );
}

#[test]
fn pdf_initial_and_first_save_keep_page_owned_roots_without_fragment_cuts() {
    use rhwp::model::{
        control::Control,
        paragraph::Paragraph,
        shape::{HorzRelTo, TextWrap, VertRelTo},
        table::TablePageBreak,
    };
    fn check_paragraph(paragraph: &Paragraph, root: bool) {
        if root {
            assert!(paragraph.text.is_empty());
            assert!(paragraph.char_offsets.is_empty());
            assert!(paragraph.line_segs.len() <= 1);
        }
        for control in &paragraph.controls {
            match control {
                Control::Table(table) => {
                    assert_eq!(table.page_break, TablePageBreak::None);
                    if root {
                        assert!(!table.common.treat_as_char);
                        assert_eq!(table.common.horz_rel_to, HorzRelTo::Paper);
                        assert_eq!(table.common.vert_rel_to, VertRelTo::Paper);
                        assert_eq!(table.common.text_wrap, TextWrap::InFrontOfText);
                    }
                    for cell in &table.cells {
                        for paragraph in &cell.paragraphs {
                            check_paragraph(paragraph, false);
                        }
                    }
                }
                Control::Shape(shape) => {
                    if let Some(textbox) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                        for paragraph in &textbox.paragraphs {
                            check_paragraph(paragraph, false);
                        }
                    }
                }
                Control::Picture(_) => {}
                _ => panic!("PDF lowering opened an unaudited authored-control path"),
            }
        }
    }
    // A source page is an absolute canvas. Nested tables stay cell-owned;
    // authored notes/fields and top-level flow tables are not fabricated.
    // The source producer and consumer guards are audited separately from
    // these real inputs; zero debugger hits alone do not prove reachability.
    for source in [
        PDF,
        include_bytes!("../../pdf/issue5875/nested_table_text_caption-2020.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/cell-nested.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/body-columns.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/cell-picture-overlap.pdf").as_slice(),
    ] {
        let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
        let mut core = request.open(source, &Default::default()).unwrap();
        for section in &core.document().sections {
            assert_eq!(section.paragraphs.len(), 1);
            check_paragraph(&section.paragraphs[0], true);
        }
        let before = core.export_hwpx_native().unwrap();
        assert_eq!(
            request.export_hwpx(&mut core, 32 * 1024 * 1024).unwrap(),
            before
        );
        request.export_hwp(&mut core, 32 * 1024 * 1024).unwrap();
        for page in core.dump_page_items_json(None).as_array().unwrap() {
            let columns = page["columns"].as_array().unwrap();
            assert_eq!(columns.len(), 1);
            for item in columns[0]["items"].as_array().unwrap() {
                assert!(matches!(
                    item["kind"].as_str(),
                    Some("fullParagraph" | "shape")
                ));
            }
        }
    }
}

#[test]
fn transport_reservation_precedes_open_and_failed_candidate_cannot_restart() {
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = 8;
    let mut request = PdfImportSession::new(policy).unwrap();
    request.reserve_transfer(8).unwrap();
    assert!(request
        .reserve_transfer(1)
        .unwrap_err()
        .to_string()
        .contains("PDF_RESOURCE_LIMIT"));
    assert!(request.open(PDF, &Default::default()).is_err());
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    fresh.reserve_transfer(PDF.len()).unwrap();
    assert_eq!(
        fresh.open(PDF, &Default::default()).unwrap().page_count(),
        1
    );
}

#[test]
fn bounded_svg_matches_ordinary_output_and_small_limit_retires_request() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let core = request.open(PDF, &Default::default()).unwrap();
    let expected = core.render_page_svg_layer_native(0).unwrap();
    assert_eq!(
        request.render_page_svg(&core, 0, expected.len()).unwrap(),
        expected
    );
    assert!(request.render_page_svg(&core, 0, 16).is_err());
    assert!(request.render_page_svg(&core, 0, expected.len()).is_err());
    assert_eq!(core.render_page_svg_layer_native(0).unwrap(), expected);
}

#[test]
fn editor_reopen_reserves_archive_reads_core_and_detaches_editing_lifetime() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    let mut import = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut original = import
        .open(
            include_bytes!("../../pdf/issue7966/image-placement-controls.pdf"),
            &Default::default(),
        )
        .unwrap();
    let handoff = import.export_hwpx(&mut original, 1024 * 1024).unwrap();
    let live = Arc::new(AtomicBool::new(true));
    let guard = live.clone();
    let mut request =
        PdfImportSession::new_with_work_guard(PdfImportPolicy::default(), move || {
            guard.load(Ordering::Relaxed)
        })
        .unwrap();
    request.reserve_transfer(handoff.len()).unwrap();
    let mut editor = request.reopen_hwpx_export(&handoff, 1024 * 1024).unwrap();
    let used = request.resource_usage().decoded_bytes;
    assert!(used > (handoff.len() * 2) as u64);
    assert_eq!(editor.page_count(), original.page_count());
    live.store(false, Ordering::Relaxed);
    let image = original
        .get_bin_data(0)
        .expect("picture PDF supplies an asset");
    assert_eq!(editor.get_bin_data(0).unwrap(), image);
    editor.reflow_linesegs_on_demand();
    assert!(!editor.export_hwp_native().unwrap().is_empty());
    assert!(!editor.export_hwpx_native().unwrap().is_empty());
    assert_eq!(request.resource_usage().decoded_bytes, used);
    assert!(request.reopen_hwpx_export(&handoff, 1024 * 1024).is_err());
}

#[test]
fn editor_reopen_refuses_remaining_budget_owner_expiry_and_recovers() {
    let mut import = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut original = import.open(PDF, &Default::default()).unwrap();
    let handoff = import.export_hwpx(&mut original, 1024 * 1024).unwrap();
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = handoff.len() as u64;
    let mut low = PdfImportSession::new(policy).unwrap();
    assert!(low.reopen_hwpx_export(&handoff, 1024 * 1024).is_err());
    assert!(low.resource_usage().failure.is_some());
    let mut expired =
        PdfImportSession::new_with_work_guard(PdfImportPolicy::default(), || false).unwrap();
    assert!(expired.reopen_hwpx_export(&handoff, 1024 * 1024).is_err());
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    assert_eq!(
        fresh
            .reopen_hwpx_export(&handoff, 1024 * 1024)
            .unwrap()
            .page_count(),
        1
    );
}
