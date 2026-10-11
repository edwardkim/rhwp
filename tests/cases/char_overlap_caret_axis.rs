//! 글자겹침은 payload 글자 수와 무관하게 논리 캐럿 한 칸이다.
//! 물리 treatAsChar나 HWP UTF-16 원문 자리를 바꾸지 않고 공개 입력·복사 축을 맞춘다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{AutoNumber, CharOverlap, Control};
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn overlap() -> Control {
    Control::CharOverlap(CharOverlap {
        chars: vec!['가', '나'],
        border_type: 1,
        inner_char_size: 100,
        char_shape_ids: vec![0, 0],
        ..Default::default()
    })
}

/// 저장 스트림에서는 겹침 하나가 8 UTF-16 유닛이고, 캐럿에서는 한 칸이다.
fn source(text: &str, overlap_at: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_text_native(0, 0, 0, "머리").unwrap();
    doc.split_paragraph_native(0, 0, 2, None).unwrap();
    let mut document = doc.document().clone();
    let mut offset = 0u32;
    let mut char_offsets = Vec::new();
    for (i, ch) in text.chars().enumerate() {
        if i == overlap_at {
            offset += 8;
        }
        char_offsets.push(offset);
        offset += ch.len_utf16() as u32;
    }
    if overlap_at == text.chars().count() {
        offset += 8;
    }
    document.sections[0].paragraphs[1] = Paragraph {
        text: text.into(),
        char_offsets,
        char_count: offset + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        controls: vec![overlap()],
        has_para_text: true,
        ..Default::default()
    };
    document.sections[0].raw_stream = None;
    doc.set_document(document);
    doc
}

fn mixed() -> HwpDocument {
    let mut doc = source("앞뒤끝", 1);
    doc.insert_footnote_native(0, 1, 2).unwrap();
    doc
}

fn para(doc: &HwpDocument, pi: usize) -> &Paragraph {
    &doc.document().sections[0].paragraphs[pi]
}

fn slots(p: &Paragraph) -> Vec<(&'static str, usize)> {
    p.controls
        .iter()
        .zip(p.control_text_positions())
        .filter_map(|(control, at)| match control {
            Control::CharOverlap(_) => Some(("겹침", at)),
            Control::Footnote(_) => Some(("각주", at)),
            _ => None,
        })
        .collect()
}

fn variants(make: impl Fn() -> HwpDocument) -> Vec<(&'static str, HwpDocument)> {
    let mut doc = make();
    let hwp = doc.export_hwp_native().unwrap();
    let hwpx = doc.export_hwpx_native().unwrap();
    vec![
        ("편집 중", doc),
        ("HWP", HwpDocument::from_bytes(&hwp).unwrap()),
        ("HWPX", HwpDocument::from_bytes(&hwpx).unwrap()),
    ]
}

#[test]
fn mixed_overlap_note_uses_one_axis_in_public_getters_and_both_formats() {
    for (format, doc) in variants(mixed) {
        let p = para(&doc, 1);
        assert_eq!(p.text, "앞뒤끝", "{format}");
        assert_eq!(slots(p), [("겹침", 1), ("각주", 2)], "{format}");
        assert_eq!(p.logical_control_positions(), [1, 3], "{format}");
        assert_eq!(doc.get_logical_length(0, 1).unwrap(), 5, "{format}");
        for (logical, text) in [0, 1, 1, 2, 2, 3].into_iter().enumerate() {
            assert_eq!(
                doc.logical_to_text_offset(0, 1, logical as u32).unwrap(),
                text,
                "{format} 논리 {logical}"
            );
        }
        for (text, logical) in [0, 1, 3, 5].into_iter().enumerate() {
            assert_eq!(
                doc.text_to_logical_offset(0, 1, text as u32).unwrap(),
                logical,
                "{format} 글자 {text}"
            );
        }
        let Control::CharOverlap(overlap) = &p.controls[0] else {
            panic!("겹침 원문 누락: {format}");
        };
        assert_eq!(overlap.chars, ['가', '나'], "두 glyph도 한 슬롯: {format}");
        assert_eq!(overlap.border_type, 1, "{format}");
        assert!(!p.controls[0].is_treat_as_char_object(), "{format}");
    }
}

#[test]
fn public_caret_and_selection_use_the_rendered_overlap_and_note_boundaries() {
    fn find(node: &RenderNode, note: bool) -> Option<&RenderNode> {
        let matches = match &node.node_type {
            RenderNodeType::FootnoteMarker(marker) => note && marker.para_index == 1,
            RenderNodeType::TextRun(run) => !note && run.char_overlap.is_some(),
            _ => false,
        };
        if matches {
            Some(node)
        } else {
            node.children.iter().find_map(|child| find(child, note))
        }
    }
    for (format, doc) in variants(mixed) {
        let tree = doc.build_page_render_tree(0).unwrap();
        let overlap = &find(&tree.root, false).expect("실제 겹침 run").bbox;
        let note = &find(&tree.root, true).expect("실제 각주 마커").bbox;
        // 공개 사각형 JSON은 소수 첫째 자리로 반올림한다.
        for (offset, x) in [
            (1, overlap.x),
            (2, overlap.x + overlap.width),
            (3, note.x),
            (4, note.x + note.width),
        ] {
            let rect: serde_json::Value =
                serde_json::from_str(&doc.get_cursor_rect_native(0, 1, offset).unwrap()).unwrap();
            let actual = rect["x"].as_f64().unwrap();
            assert!(
                (actual - x).abs() <= 0.06,
                "{format} 캐럿 {offset}: {actual} != {x}"
            );
            assert_eq!(rect["pageIndex"], 0);
        }
        let rects: serde_json::Value =
            serde_json::from_str(&doc.get_selection_rects(0, 1, 1, 1, 2).unwrap()).unwrap();
        assert_eq!(rects.as_array().unwrap().len(), 1, "{format}: {rects}");
        let selected = &rects[0];
        assert!(
            (selected["x"].as_f64().unwrap() - overlap.x).abs() <= 0.06,
            "{format}: {rects}; 겹침 {overlap:?}"
        );
        assert!(
            (selected["width"].as_f64().unwrap() - overlap.width).abs() <= 0.06,
            "{format}: {rects}; 겹침 {overlap:?}"
        );
    }
    // 셀 선택도 같은 공통 경계를 쓰되, 독립적인 실제 셀 run 상자로 검증한다.
    for (format, doc) in variants(|| {
        let cell_para = para(&source("앞뒤", 1), 1).clone();
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        let created: serde_json::Value =
            serde_json::from_str(&doc.create_table_native(0, 0, 0, 1, 2).unwrap()).unwrap();
        let parent = created["paraIdx"].as_u64().unwrap() as usize;
        let control = created["controlIdx"].as_u64().unwrap() as usize;
        let mut model = doc.document().clone();
        let Control::Table(table) = &mut model.sections[0].paragraphs[parent].controls[control]
        else {
            panic!("생성한 표 누락");
        };
        table.cells[0].paragraphs = vec![cell_para];
        model.sections[0].raw_stream = None;
        doc.set_document(model);
        doc
    }) {
        let (parent, control) = doc.document().sections[0]
            .paragraphs
            .iter()
            .enumerate()
            .find_map(|(pi, p)| {
                p.controls
                    .iter()
                    .position(|c| matches!(c, Control::Table(_)))
                    .map(|ci| (pi, ci))
            })
            .expect("재열기한 표 주소");
        let tree = doc.build_page_render_tree(0).unwrap();
        let overlap = &find(&tree.root, false).expect("실제 셀 겹침 run").bbox;
        for (offset, x) in [(1, overlap.x), (2, overlap.x + overlap.width)] {
            let rect: serde_json::Value = serde_json::from_str(
                &doc.get_cursor_rect_in_cell_native(0, parent, control, 0, 0, offset)
                    .unwrap(),
            )
            .unwrap();
            assert!(
                (rect["x"].as_f64().unwrap() - x).abs() <= 0.06,
                "{format}: {rect}"
            );
        }
        let rects: serde_json::Value = serde_json::from_str(
            &doc.get_selection_rects_in_cell(0, parent as u32, control as u32, 0, 0, 1, 0, 2)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(rects.as_array().unwrap().len(), 1, "{format}: {rects}");
        assert!(
            (rects[0]["x"].as_f64().unwrap() - overlap.x).abs() <= 0.06,
            "{format}: {rects}"
        );
        assert!(
            (rects[0]["width"].as_f64().unwrap() - overlap.width).abs() <= 0.06,
            "{format}: {rects}"
        );
    }
}

#[test]
fn input_before_and_after_overlap_and_note_preserves_source_slots() {
    for logical in 1..=4u32 {
        for input in ["가", "🦦"] {
            for (format, mut doc) in variants(mixed) {
                let before = doc.save_snapshot_native();
                let inserted: serde_json::Value =
                    serde_json::from_str(&doc.insert_text_logical(0, 1, logical, input).unwrap())
                        .unwrap();
                let expected_text = if logical <= 2 {
                    format!("앞{input}뒤끝")
                } else {
                    format!("앞뒤{input}끝")
                };
                let expected_slots = match logical {
                    1 => vec![("겹침", 2), ("각주", 3)],
                    2 => vec![("겹침", 1), ("각주", 3)],
                    3 => vec![("겹침", 1), ("각주", 3)],
                    4 => vec![("겹침", 1), ("각주", 2)],
                    _ => unreachable!(),
                };
                assert_eq!(para(&doc, 1).text, expected_text, "{format} {logical}");
                assert_eq!(slots(para(&doc, 1)), expected_slots, "{format} {logical}");
                assert_eq!(inserted["logicalOffset"], logical + 1, "{format} {logical}");
                assert_eq!(doc.get_logical_length(0, 1).unwrap(), 6, "{format}");
                for (saved_format, reopened) in variants(|| {
                    let mut copy = HwpDocument::create_empty();
                    copy.set_document(doc.document().clone());
                    copy
                }) {
                    assert_eq!(para(&reopened, 1).text, expected_text, "{saved_format}");
                    assert_eq!(slots(para(&reopened, 1)), expected_slots, "{saved_format}");
                }
                let after = doc.save_snapshot_native();
                doc.restore_snapshot_native(before).unwrap();
                assert_eq!(para(&doc, 1).text, "앞뒤끝");
                assert_eq!(slots(para(&doc, 1)), [("겹침", 1), ("각주", 2)]);
                doc.restore_snapshot_native(after).unwrap();
                assert_eq!(para(&doc, 1).text, expected_text);
                assert_eq!(slots(para(&doc, 1)), expected_slots);
                assert_eq!(para(&doc, 0).text, "머리");
            }
        }
    }
}

#[test]
fn logical_copy_excludes_or_includes_the_overlap_slot_without_neighbor_text() {
    for (start, end, expected_text, expected_slots) in [
        (1, 2, "", vec![("겹침", 0)]),
        (2, 3, "뒤", vec![]),
        (3, 4, "", vec![("각주", 0)]),
        (2, 5, "뒤끝", vec![("각주", 1)]),
    ] {
        for (format, mut doc) in variants(mixed) {
            let copied: serde_json::Value =
                serde_json::from_str(&doc.copy_selection_logical(0, 1, start, 1, end).unwrap())
                    .unwrap();
            assert_eq!(copied["text"], expected_text, "{format} {start}..{end}");
            doc.split_paragraph_native(0, 1, 5, None).unwrap();
            doc.insert_text_native(0, 2, 0, "복").unwrap();
            doc.paste_internal_native(0, 2, 0).unwrap();
            assert_eq!(para(&doc, 2).text, format!("{expected_text}복"), "{format}");
            assert_eq!(
                slots(para(&doc, 2)),
                expected_slots,
                "{format} {start}..{end}"
            );
            assert_eq!(para(&doc, 1).text, "앞뒤끝", "{format}");
            assert_eq!(slots(para(&doc, 1)), [("겹침", 1), ("각주", 2)]);
        }
    }
}

#[test]
fn split_merge_and_snapshot_restore_keep_the_logical_seam_after_overlap() {
    for (format, mut doc) in variants(mixed) {
        let before = doc.save_snapshot_native();
        doc.split_paragraph_native(0, 1, 2, None).unwrap();
        assert_eq!(
            (para(&doc, 1).text.as_str(), para(&doc, 2).text.as_str()),
            ("앞", "뒤끝")
        );
        assert_eq!(slots(para(&doc, 1)), [("겹침", 1)], "{format}");
        assert_eq!(slots(para(&doc, 2)), [("각주", 1)], "{format}");
        let seam = doc.get_logical_length(0, 1).unwrap() as usize;
        assert_eq!(seam, 2, "{format}");
        doc.merge_paragraph_native(0, 2).unwrap();
        assert_eq!(para(&doc, 1).text, "앞뒤끝", "{format}");
        assert_eq!(slots(para(&doc, 1)), [("겹침", 1), ("각주", 2)]);
        doc.split_paragraph_native(0, 1, seam, None).unwrap();
        assert_eq!(
            (para(&doc, 1).text.as_str(), para(&doc, 2).text.as_str()),
            ("앞", "뒤끝")
        );
        doc.restore_snapshot_native(before).unwrap();
        assert_eq!(para(&doc, 1).text, "앞뒤끝");
        assert_eq!(para(&doc, 1).logical_control_positions(), [1, 3]);
        assert_eq!(doc.document().sections[0].paragraphs.len(), 2);
    }
}

#[test]
fn empty_overlap_and_same_anchor_slots_are_counted_without_claiming_middle_input() {
    for (format, mut doc) in variants(|| source("", 0)) {
        assert_eq!(doc.get_logical_length(0, 1).unwrap(), 1, "{format}");
        assert_eq!(para(&doc, 1).logical_control_positions(), [0], "{format}");
        assert_eq!(doc.logical_to_text_offset(0, 1, 0).unwrap(), 0);
        assert_eq!(doc.logical_to_text_offset(0, 1, 1).unwrap(), 0);
        doc.insert_text_logical(0, 1, 0, "🦦").unwrap();
        assert_eq!(
            slots(para(&doc, 1)),
            [("겹침", 1)],
            "{format} 빈 문단 앞 입력"
        );
    }
    let mut doc = source("앞뒤", 1);
    let mut model = doc.document().clone();
    let p = &mut model.sections[0].paragraphs[1];
    p.controls.push(overlap());
    p.char_offsets[1] += 8;
    p.char_count += 8;
    doc.set_document(model);
    assert_eq!(slots(para(&doc, 1)), [("겹침", 1), ("겹침", 1)]);
    assert_eq!(para(&doc, 1).logical_control_positions(), [1, 2]);
    assert_eq!(doc.get_logical_length(0, 1).unwrap(), 4);
    // 같은 글자 자리에 붙은 두 슬롯의 중간 입력 주소는 #7509의 별도 보호 범위다.
}

#[test]
fn floating_and_auto_number_keep_their_existing_caret_contract() {
    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, "앞 뒤").unwrap();
    doc.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6750,
        0,
        0,
        false,
        "Square",
        "rectangle",
        false,
        false,
        &[],
    )
    .unwrap();
    let mut model = doc.document().clone();
    let p = &mut model.sections[0].paragraphs[0];
    p.controls.push(Control::AutoNumber(AutoNumber::default()));
    p.char_offsets = vec![8, 9, 17];
    p.char_count = 19;
    doc.set_document(model);
    let p = para(&doc, 0);
    assert!(!p.controls[0].is_logical_inline());
    assert!(!p.controls[1].is_logical_inline());
    assert_eq!(p.control_text_positions(), [0, 1]);
    assert_eq!(p.logical_control_positions(), [0, 1]);
    assert_eq!(doc.get_logical_length(0, 0).unwrap(), 3);
    for at in 0..=3 {
        assert_eq!(doc.logical_to_text_offset(0, 0, at).unwrap(), at);
        assert_eq!(doc.text_to_logical_offset(0, 0, at).unwrap(), at);
    }
}
