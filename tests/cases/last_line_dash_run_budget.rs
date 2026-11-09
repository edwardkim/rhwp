//! 검사 유형: 관계형. 마지막 줄의 각 대시는 자기 글자 모양의 자연 전진까지만 복원한다.
//! 주석 갱신: 2026-11-09T11:27:00+09:00. 공개 빈 문서의 합성 계약이며 실물 좌표·설치 글꼴에 의존하지 않는다.
//! 원 계약 #5830·PR #5858/#5889의 자연 폭 상한과 좁은 여백 한도를 유지한다.
//! 서로 다른 크기·굵기·저자 자간을 가진 런에 첫 런의 복원량을 재사용하면 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::{
        paragraph::{CharShapeRef, Paragraph},
        style::Alignment,
    },
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};

fn collect(node: &RenderNode, out: &mut Vec<TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains("---") {
            out.push(run.clone());
        }
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn runs(shapes: &[(i32, bool, i8)], alignment: Alignment) -> Vec<TextRunNode> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let mut paragraph = Paragraph::new_empty();
    doc.doc_info.para_shapes[0].alignment = alignment;
    for &(size, bold, spacing) in shapes {
        let mut shape = doc.doc_info.char_shapes[0].clone();
        shape.raw_data = None;
        shape.base_size = size;
        shape.bold = bold;
        shape.spacings = [spacing; 7];
        shape.ratios = [100; 7];
        paragraph.char_shapes.push(CharShapeRef {
            start_pos: paragraph.text.len() as u32,
            char_shape_id: doc.doc_info.char_shapes.len() as u32,
        });
        doc.doc_info.char_shapes.push(shape);
        paragraph.text.push_str("------------");
    }
    paragraph.char_count = paragraph.text.len() as u32;
    paragraph.char_offsets = (0..paragraph.char_count).collect();
    paragraph.invalidate_layout_inputs();
    doc.sections[0].paragraphs = vec![paragraph];
    let page = &mut doc.sections[0].section_def.page_def;
    page.width = 200_000;
    page.margin_left = 1_000;
    page.margin_right = 1_000;
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut result = vec![];
    collect(&tree.root, &mut result);
    assert_eq!(result.len(), shapes.len(), "원문 런의 순서·누락·중복");
    result
}

fn assert_natural(shapes: &[(i32, bool, i8)]) {
    for run in runs(shapes, Alignment::Justify) {
        let mut natural = run.style.clone();
        natural.extra_dash_advance = 0.0;
        natural.extra_char_spacing = 0.0;
        natural.extra_word_spacing = 0.0;
        let expected = EmbeddedTextMeasurer.compute_char_positions("-", &natural)[1];
        let positions = run
            .layout_positions
            .clone()
            .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style));
        for pair in positions.windows(2).take(run.text.len() - 1) {
            let actual = pair[1] - pair[0];
            assert!(
                (actual - expected).abs() <= 1.0 / 75.0,
                "각 런의 자연 전진을 한 번만 소비한다: size={} bold={} tracking={} actual={actual} natural={expected}",
                run.style.font_size, run.style.bold, run.style.letter_spacing,
            );
        }
    }
}

#[test]
fn last_line_restores_each_font_size_independently() {
    assert_natural(&[(2_000, false, 0), (900, false, 0)]);
    assert_natural(&[(900, false, 0), (2_000, false, 0)]);
}

#[test]
fn last_line_does_not_restore_authored_tracking_twice() {
    assert_natural(&[(1_200, false, 15), (1_200, false, 0)]);
}

#[test]
fn last_line_does_not_apply_bold_tracking_to_a_regular_successor() {
    assert_natural(&[(900, true, 0), (900, false, 0)]);
}

#[test]
fn left_aligned_leaders_keep_the_existing_compressed_advances() {
    let run = &runs(&[(1_200, false, 0)], Alignment::Left)[0];
    assert_eq!(run.style.extra_dash_advance, 0.0);
    let positions = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
    let single = EmbeddedTextMeasurer.compute_char_positions("-", &run.style)[1];
    assert!(
        positions[1] < single,
        "왼쪽 정렬의 짧은 리더를 확장하지 않는다"
    );
}
