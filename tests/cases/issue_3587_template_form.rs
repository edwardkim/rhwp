//! Fixed-form contracts; synthetic fixtures do not claim Hancom visual equivalence.
use rhwp::{
    document_core::{
        DocumentCore, FillTemplateRequest, ParagraphBlockLimits, ParagraphBlockPathStep as Step,
        RepeatParagraphBlockRequest, TemplateBinding, TemplateFillRequest, TemplateFillTarget,
        TemplateScope,
    },
    model::{
        control::{Control, Field, FieldType},
        paragraph::{FieldRange, Paragraph},
    },
};
use std::collections::BTreeMap;

fn para(text: &str) -> Paragraph {
    let mut p = Paragraph::default();
    p.insert_text_at(0, text);
    p
}
fn core() -> DocumentCore {
    let mut c = DocumentCore::new_empty();
    c.create_blank_document_native().unwrap();
    c.document_mut().sections[0].paragraphs =
        ["before", "A😀B", "", "tail", "after"].map(para).to_vec();
    c
}
fn scope() -> TemplateScope {
    TemplateScope {
        section_index: 0,
        start: 1,
        end: 4,
        limits: ParagraphBlockLimits::default(),
    }
}
fn binding(key: &str, root: usize, start: usize, end: usize) -> TemplateBinding {
    TemplateBinding {
        key: key.into(),
        target: TemplateFillTarget::TextRange {
            path: vec![Step::Paragraph(root)],
            start,
            end,
        },
    }
}
fn request() -> FillTemplateRequest {
    FillTemplateRequest {
        scope: scope(),
        bindings: vec![binding("head", 0, 0, 1), binding("tail", 2, 0, 4)],
        record: BTreeMap::from([
            ("head".into(), "새".into()),
            ("tail".into(), "끝\n본문".into()),
        ]),
    }
}
fn state(c: &DocumentCore) -> (String, String, String) {
    (
        format!("{:?}", c.document()),
        c.serialize_event_log(),
        format!("{:?}", c.get_clipboard_text_native()),
    )
}
fn field(p: &mut Paragraph, id: u32) {
    p.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: id,
        ctrl_id: u32::from_le_bytes(*b"klc%"),
        ctrl_data_name: Some("title".into()),
        ..Default::default()
    }));
    p.field_ranges.push(FieldRange {
        start_char_idx: 0,
        end_char_idx: p.text.chars().count(),
        control_idx: 0,
        ..Default::default()
    });
    let mut offset = 8;
    p.char_offsets = p
        .text
        .chars()
        .map(|c| {
            let o = offset;
            offset += c.len_utf16() as u32;
            o
        })
        .collect();
    p.char_count = offset + 9;
}

#[test]
fn fixed_form_changes_only_selected_roots_without_copying_or_clipboard() {
    let mut c = core();
    let clip = c.get_clipboard_text_native();
    let blank_ptr = &c.document().sections[0].paragraphs[2] as *const Paragraph;
    let result = c.fill_template_native(&request()).unwrap();
    assert_eq!(result.paragraphs, vec![1, 3]);
    assert_eq!(result.target_count, 2);
    assert_eq!(
        c.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>(),
        ["before", "새😀B", "", "끝\n본문", "after"]
    );
    assert_eq!(
        &c.document().sections[0].paragraphs[2] as *const Paragraph,
        blank_ptr
    );
    assert_eq!(c.get_clipboard_text_native(), clip);
    let events: serde_json::Value = serde_json::from_str(&c.serialize_event_log()).unwrap();
    let event = events["events"].as_array().unwrap().last().unwrap();
    assert_eq!(event["type"], "TemplateFilled");
    assert_eq!(event["paragraphs"], serde_json::json!([1, 3]));
    // Reusing an exact scalar request changes content again, but never adds paragraphs.
    c.fill_template_native(&request()).unwrap();
    assert_eq!(c.document().sections[0].paragraphs.len(), 5);
}

#[test]
fn invalid_overlap_missing_key_and_empty_request_preserve_state() {
    let mut c = core();
    let before = state(&c);
    let mut r = request();
    r.record.remove("tail");
    assert!(c.fill_template_native(&r).is_err());
    assert_eq!(state(&c), before);
    r = request();
    r.bindings[1].target = r.bindings[0].target.clone();
    assert!(c.fill_template_native(&r).is_err());
    assert_eq!(state(&c), before);
    r.bindings.clear();
    r.record.clear();
    assert_eq!(c.fill_template_native(&r).unwrap().target_count, 0);
    assert_eq!(state(&c), before);
    r.scope.end = usize::MAX;
    assert!(c.fill_template_native(&r).is_err());
    assert_eq!(state(&c), before);
}

#[test]
fn staging_growth_failure_does_not_leak_earlier_root_edits() {
    let mut c = core();
    c.document_mut().sections[0].paragraphs[3] = para(&"x".repeat(100));
    let mut r = request();
    r.scope.limits.max_structure_bytes = 800_000;
    r.bindings = (0..100)
        .map(|i| binding(&format!("k{i}"), 2, i, i + 1))
        .collect();
    r.record = (0..100)
        .map(|i| (format!("k{i}"), "x".repeat(200)))
        .collect();
    r.bindings.push(binding("first", 0, 0, 1));
    r.record.insert("first".into(), "edited".into());
    let before = state(&c);
    let e = c.fill_template_native(&r).unwrap_err().to_string();
    assert!(e.contains("fill byte budget"), "{e}");
    assert_eq!(state(&c), before);
}

#[test]
fn field_names_are_scope_local_and_require_disambiguation_preserving_identity() {
    let mut c = core();
    field(&mut c.document_mut().sections[0].paragraphs[0], 10);
    field(&mut c.document_mut().sections[0].paragraphs[1], 11);
    field(&mut c.document_mut().sections[0].paragraphs[3], 12);
    assert_eq!(
        c.template_field_target_native(&scope(), "title", None)
            .unwrap_err()
            .code,
        "fillAmbiguous"
    );
    assert!(c
        .template_field_target_native(&scope(), "title", Some(2))
        .is_err());
    assert!(c
        .template_field_target_native(&scope(), "missing", None)
        .is_err());
    let target = c
        .template_field_target_native(&scope(), "title", Some(1))
        .unwrap();
    assert!(
        matches!(&target, TemplateFillTarget::Field { path, .. } if path == &[Step::Paragraph(2)])
    );
    c.fill_template_native(&FillTemplateRequest {
        scope: scope(),
        bindings: vec![TemplateBinding {
            key: "value".into(),
            target,
        }],
        record: BTreeMap::from([("value".into(), "채움😀".into())]),
    })
    .unwrap();
    for (pi, id, text) in [(0, 10, "before"), (1, 11, "A😀B"), (3, 12, "채움😀")] {
        let p = &c.document().sections[0].paragraphs[pi];
        assert_eq!(p.text, text);
        let Control::Field(f) = &p.controls[0] else {
            panic!("lost field")
        };
        assert_eq!(f.field_id, id);
    }
}

#[test]
fn real_cell_coordinate_fill_preserves_structure_and_saves_both_formats() {
    let mut c =
        DocumentCore::from_bytes(&std::fs::read("samples/rnote/labnote-001.hwp").unwrap()).unwrap();
    let scope = TemplateScope {
        start: 12,
        end: 13,
        ..scope()
    };
    let Control::Table(t) = &c.document().sections[0].paragraphs[12].controls[1] else {
        panic!("table")
    };
    let anchor = (t.cells[5].row, t.cells[5].col);
    let identity = t.common.instance_id;
    let original_cell_count = t.cells.len();
    let original_para_count = c.document().sections[0].paragraphs.len();
    let path = [Step::Paragraph(0), Step::Control(1)];
    assert!(c
        .template_cell_target_native(&scope, &path, (u16::MAX, 0), 0, 0..0)
        .is_err());
    assert!(c
        .template_cell_target_native(&scope, &path, anchor, usize::MAX, 0..0)
        .is_err());
    let target = c
        .template_cell_target_native(&scope, &path, anchor, 0, 0..0)
        .unwrap();
    c.fill_template_native(&FillTemplateRequest {
        scope,
        bindings: vec![TemplateBinding {
            key: "body".into(),
            target,
        }],
        record: BTreeMap::from([("body".into(), "실물😀\n본문".into())]),
    })
    .unwrap();
    assert_eq!(
        c.document().sections[0].paragraphs.len(),
        original_para_count
    );
    for bytes in [
        c.export_hwp_native().unwrap(),
        c.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        let Control::Table(t) = &reopened.document().sections[0].paragraphs[12].controls[1] else {
            panic!("table")
        };
        assert_eq!(t.cells[5].paragraphs[0].text, "실물😀\n본문");
        assert_eq!(t.common.instance_id, identity);
        assert_eq!(t.cells.len(), original_cell_count);
    }
}

#[test]
fn merged_covered_cell_is_not_silently_redirected() {
    use rhwp::model::table::{Cell, Table};
    let mut c = core();
    let t = Table {
        cells: vec![Cell {
            row: 0,
            col: 0,
            col_span: 2,
            paragraphs: vec![para(""), para("")],
            ..Default::default()
        }],
        ..Default::default()
    };
    c.document_mut().sections[0].paragraphs[1]
        .controls
        .push(Control::Table(Box::new(t)));
    let path = [Step::Paragraph(0), Step::Control(0)];
    assert_eq!(
        c.template_cell_target_native(&scope(), &path, (0, 1), 0, 0..0)
            .unwrap_err()
            .code,
        "fillCell"
    );
    let target = c
        .template_cell_target_native(&scope(), &path, (0, 0), 1, 0..0)
        .unwrap();
    // The resolved convenience target also works for repetition without a second editing engine.
    let r = TemplateFillRequest {
        block: RepeatParagraphBlockRequest {
            section_index: 0,
            source_start: 1,
            source_end: 4,
            insert_before: 4,
            count: 1,
            limits: ParagraphBlockLimits::default(),
        },
        bindings: vec![TemplateBinding {
            key: "x".into(),
            target,
        }],
        records: vec![BTreeMap::from([("x".into(), "second paragraph".into())])],
    };
    c.repeat_and_fill_paragraph_block_native(&r).unwrap();
    let Control::Table(t) = &c.document().sections[0].paragraphs[4].controls[0] else {
        panic!("table")
    };
    assert!(t.cells[0].paragraphs[0].text.is_empty());
    assert_eq!(t.cells[0].paragraphs[1].text, "second paragraph");
}
