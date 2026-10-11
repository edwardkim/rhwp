//! 한컴 도움말의 기준 크기 백분율·방향과 공개 캐럿/선택/저장 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

fn fixture(text: &str, relative_size: u8) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, text).unwrap();
    doc.apply_char_format_native(
        0,
        0,
        0,
        text.chars().count(),
        &json!({"fontSize":1500,"relativeSizes":vec![relative_size;7]}).to_string(),
    )
    .unwrap();
    doc
}

fn run_boxes(doc: &HwpDocument) -> Vec<(String, f64, f64, f64)> {
    fn collect(node: &RenderNode, result: &mut Vec<(String, f64, f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.char_start.is_some() && !run.text.is_empty() {
                result.push((run.text.clone(), node.bbox.x, node.bbox.y, node.bbox.width));
            }
        }
        for child in &node.children {
            collect(child, result);
        }
    }
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut result = Vec::new();
    collect(&tree.root, &mut result);
    result
}

fn value(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
}

#[test]
fn language_offsets_move_only_their_runs_without_changing_advance_or_line_spacing() {
    // 한글·영문·한자·일어·기호·사용자 슬롯을 기존 언어 분리로 소비한다.
    let mut doc = fixture("가A漢あ★ㆍ", 100);
    let before = run_boxes(&doc);
    assert_eq!(before.len(), 6);
    let offsets = [-100, -50, -30, 0, 30, 50, 100];
    doc.apply_char_format_native(0, 0, 0, 6, &json!({"charOffsets":offsets}).to_string())
        .unwrap();
    let after = run_boxes(&doc);
    assert_eq!(after.len(), before.len());
    for (i, (old, new)) in before.iter().zip(&after).enumerate() {
        assert_eq!(old.0, new.0);
        near(new.1, old.1);
        near(
            new.2 - old.2,
            20.0 * f64::from(offsets[[0, 1, 2, 3, 5, 6][i]]) / 100.0,
        );
        near(new.3, old.3);
    }
    let selection = value(&doc.get_selection_rects(0, 0, 0, 0, 6).unwrap());
    let rects = selection.as_array().unwrap();
    assert_eq!(rects.len(), after.len());
    for (rect, run) in rects.iter().zip(&after) {
        assert!((rect["y"].as_f64().unwrap() - run.2).abs() < 0.1);
        assert!(rect["width"].as_f64().unwrap() >= run.3 - 0.1);
    }
}

#[test]
fn baseline_offset_uses_base_size_and_moves_caret_selection_and_saved_output() {
    for relative in [80, 200] {
        let mut doc = fixture("AB", relative);
        let before = run_boxes(&doc);
        let caret = value(&doc.get_cursor_rect_native(0, 0, 1).unwrap());
        let selection = value(&doc.get_selection_rects(0, 0, 0, 0, 2).unwrap());
        doc.apply_char_format_native(
            0,
            0,
            0,
            2,
            &json!({"charOffsets":[0,50,0,0,0,0,0]}).to_string(),
        )
        .unwrap();
        let after = run_boxes(&doc);
        near(after[0].2 - before[0].2, 10.0);
        let moved = value(&doc.get_cursor_rect_native(0, 0, 1).unwrap());
        near(
            moved["y"].as_f64().unwrap() - caret["y"].as_f64().unwrap(),
            10.0,
        );
        near(moved["x"].as_f64().unwrap(), caret["x"].as_f64().unwrap());
        let selected = value(&doc.get_selection_rects(0, 0, 0, 0, 2).unwrap());
        near(
            selected[0]["y"].as_f64().unwrap() - selection[0]["y"].as_f64().unwrap(),
            10.0,
        );
        near(
            selected[0]["width"].as_f64().unwrap(),
            selection[0]["width"].as_f64().unwrap(),
        );
        for bytes in [
            doc.export_hwp_native().unwrap(),
            doc.export_hwpx_native().unwrap(),
        ] {
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            let saved = run_boxes(&reopened);
            near(saved[0].2, after[0].2);
            assert_eq!(
                value(&reopened.get_char_properties_at_native(0, 0, 0).unwrap())["charOffsets"],
                json!([0, 50, 0, 0, 0, 0, 0])
            );
            assert_eq!(reopened.document().sections[0].paragraphs[0].text, "AB");
        }
    }
}

#[test]
fn unchanged_zero_and_superscript_keep_their_existing_metrics() {
    let mut doc = fixture("A", 100);
    doc.apply_char_format_native(0, 0, 0, 1, r#"{"superscript":true}"#)
        .unwrap();
    let before = run_boxes(&doc);
    doc.apply_char_format_native(0, 0, 0, 1, r#"{"charOffsets":[0,-50,0,0,0,0,0]}"#)
        .unwrap();
    let after = run_boxes(&doc);
    near(after[0].2 - before[0].2, -10.0);
    near(after[0].3, before[0].3);
    doc.apply_char_format_native(0, 0, 0, 1, r#"{"charOffsets":[0,0,0,0,0,0,0]}"#)
        .unwrap();
    assert_eq!(run_boxes(&doc), before);
}

#[test]
fn cell_offset_moves_its_caret_and_selection_without_touching_the_neighbor() {
    let mut doc = fixture("", 100);
    let created = value(&doc.create_table_native(0, 0, 0, 1, 2).unwrap());
    let parent = created["paraIdx"].as_u64().unwrap() as usize;
    let control = created["controlIdx"].as_u64().unwrap() as usize;
    for cell in 0..2 {
        doc.insert_text_in_cell_native(
            0,
            parent,
            control,
            cell,
            0,
            0,
            if cell == 0 { "AB" } else { "SAFE" },
        )
        .unwrap();
    }
    doc.apply_char_format_in_cell_native(0, parent, control, 0, 0, 0, 2, r#"{"fontSize":1500}"#)
        .unwrap();
    let before = run_boxes(&doc);
    let caret = value(
        &doc.get_cursor_rect_in_cell_native(0, parent, control, 0, 0, 1)
            .unwrap(),
    );
    let selection = value(
        &doc.get_selection_rects_in_cell(0, parent as u32, control as u32, 0, 0, 0, 0, 2)
            .unwrap(),
    );
    doc.apply_char_format_in_cell_native(
        0,
        parent,
        control,
        0,
        0,
        0,
        2,
        r#"{"charOffsets":[0,-30,0,0,0,0,0]}"#,
    )
    .unwrap();
    let after = run_boxes(&doc);
    for (old, new) in before.iter().zip(&after) {
        assert_eq!(old.0, new.0);
        near(new.2 - old.2, if old.0 == "AB" { -6.0 } else { 0.0 });
        near(new.1, old.1);
        near(new.3, old.3);
    }
    let moved = value(
        &doc.get_cursor_rect_in_cell_native(0, parent, control, 0, 0, 1)
            .unwrap(),
    );
    near(
        moved["y"].as_f64().unwrap() - caret["y"].as_f64().unwrap(),
        -6.0,
    );
    let selected = value(
        &doc.get_selection_rects_in_cell(0, parent as u32, control as u32, 0, 0, 0, 0, 2)
            .unwrap(),
    );
    near(
        selected[0]["y"].as_f64().unwrap() - selection[0]["y"].as_f64().unwrap(),
        -6.0,
    );
    assert_eq!(run_boxes(&doc), after);
    for bytes in [
        doc.export_hwp_native().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        let reopened = HwpDocument::from_bytes(&bytes).unwrap();
        assert_eq!(
            value(
                &reopened
                    .get_cell_char_properties_at_native(0, parent, control, 0, 0, 0)
                    .unwrap()
            )["charOffsets"],
            json!([0, -30, 0, 0, 0, 0, 0])
        );
        let saved = run_boxes(&reopened);
        near(
            saved.iter().find(|run| run.0 == "AB").unwrap().2,
            after.iter().find(|run| run.0 == "AB").unwrap().2,
        );
    }
}
