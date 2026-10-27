//! 글자겹침 컨트롤 뒤 각주는 원문 글자 축의 제자리에 그린다.

use rhwp::model::control::{CharOverlap, Control};
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

fn source(text: &str, overlap_at: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
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
    document.sections[0].paragraphs[0] = Paragraph {
        text: text.into(),
        char_offsets,
        char_count: offset + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        controls: vec![Control::CharOverlap(CharOverlap {
            chars: vec!['가', '나'],
            border_type: 1,
            inner_char_size: 100,
            char_shape_ids: vec![0, 0],
            ..Default::default()
        })],
        has_para_text: true,
        ..Default::default()
    };
    document.sections[0].raw_stream = None;
    doc.set_document(document);
    doc
}

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    match &node.node_type {
        RenderNodeType::TextRun(run) if !run.text.is_empty() && run.para_index == Some(0) => {
            out.push(node)
        }
        RenderNodeType::FootnoteMarker(marker) if marker.para_index == 0 => out.push(node),
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

#[test]
fn a_note_after_the_character_following_an_overlap_is_drawn_after_that_character() {
    let mut doc = source("앞뒤끝", 1);
    // 원문 글자 2(`끝`) 앞, 곧 `뒤` 다음 자리에 각주를 넣는다.
    let note_text_pos = 2;
    let mut document = doc.document().clone();
    let p = &mut document.sections[0].paragraphs[0];
    assert_eq!(p.control_text_positions(), [1]);
    doc.set_document(document);
    doc.insert_footnote_native(0, 0, note_text_pos).unwrap();
    let positions = doc.document().sections[0].paragraphs[0].control_text_positions();
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let marker = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::FootnoteMarker(_)))
        .expect("각주 번호");
    let layout: serde_json::Value =
        serde_json::from_str(&doc.get_page_text_layout_native(0).unwrap()).unwrap();
    let before = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|run| {
            let text = run["text"].as_str()?;
            let at = text.chars().position(|ch| ch == '뒤')?;
            Some(run["x"].as_f64()? + run["charX"][at + 1].as_f64()?)
        })
        .expect("`뒤` 글자");
    assert!(
        // 공개 글 배치 좌표는 소수 첫째 자리로 반올림한다.
        marker.bbox.x >= before - 0.1,
        "각주 {positions:?}: 번호 x={} < `뒤` 끝 x={before}",
        marker.bbox.x
    );
}

fn right_aligned_nodes(doc: &mut HwpDocument) -> Vec<RenderNode> {
    doc.apply_para_format_native(0, 0, r#"{"alignment":"right"}"#)
        .unwrap();
    let tree = doc.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    nodes.into_iter().cloned().collect()
}

#[test]
fn a_note_at_the_overlap_spot_follows_control_order_and_keeps_the_one_slot_box() {
    // 겹침만 있는 문단의 한 칸 상자 폭이 기준이다.
    let mut alone = source("앞뒤끝", 1);
    let alone = right_aligned_nodes(&mut alone);
    let slot_width = alone
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.char_overlap.is_some()))
        .unwrap()
        .bbox
        .width;
    // 겹침 없이 같은 자리에 각주만 둔 대조군의 오른쪽 끝(단 오른쪽 끝)이다.
    let mut plain = source("앞뒤끝", 1);
    let mut model = plain.document().clone();
    let p = &mut model.sections[0].paragraphs[0];
    p.controls.clear();
    p.char_offsets = vec![0, 1, 2];
    p.char_count = 4;
    plain.set_document(model);
    plain.insert_footnote_native(0, 0, 1).unwrap();
    let plain = right_aligned_nodes(&mut plain);
    let right = plain.last().unwrap().bbox.x + plain.last().unwrap().bbox.width;

    for note_first in [false, true] {
        let mut doc = source("앞뒤끝", 1);
        doc.insert_footnote_native(0, 0, 1).unwrap();
        let mut model = doc.document().clone();
        let controls = &mut model.sections[0].paragraphs[0].controls;
        assert_eq!(controls.len(), 2);
        // 같은 원문 자리의 두 컨트롤은 저장 순서대로 그린다.
        let overlap_first = matches!(controls[0], Control::CharOverlap(_));
        if overlap_first == note_first {
            controls.swap(0, 1);
        }
        doc.set_document(model);
        let nodes = right_aligned_nodes(&mut doc);
        let order: Vec<_> = nodes
            .iter()
            .map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.char_overlap.is_some() => "[겹침]",
                RenderNodeType::TextRun(run) => run.text.as_str(),
                _ => "[각주]",
            })
            .collect();
        let expected = if note_first {
            ["앞", "[각주]", "[겹침]", "뒤끝"]
        } else {
            ["앞", "[겹침]", "[각주]", "뒤끝"]
        };
        assert_eq!(order, expected, "note_first={note_first}");
        let overlap = nodes
            .iter()
            .find(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.char_overlap.is_some()))
            .unwrap();
        assert!(
            (overlap.bbox.width - slot_width).abs() < 0.01,
            "note_first={note_first}: 겹침 폭 {} != 한 칸 {slot_width}",
            overlap.bbox.width
        );
        // 각주 폭도 정렬 폭에 들어가 줄이 단 오른쪽 끝에 맞는다.
        let last = nodes.last().unwrap();
        assert!(
            (last.bbox.x + last.bbox.width - right).abs() < 0.06,
            "note_first={note_first}: 오른쪽 끝 {} != {right}",
            last.bbox.x + last.bbox.width
        );
    }
}
