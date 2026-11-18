//! 검사 유형: 관계형(합성 런별 자연 전진), 고정형(독립 Print 707쪽의 글리프 원점 ±2 CSSpx).
//! 주석 갱신: 2026-11-09T12:46:50.257955+09:00. 합성 계약과 실물 좌표 검사를 구분하며 설치 글꼴에 의존하지 않는다.
//! 실물 707쪽: 2026-11-09 frozen B4 Native/fresh WASM 각각 raw 100.00%, Print 96 DPI·compat2022·2px.
//! 독립 Print: pdf/issue8371/hwp3-sample10-2020.pdf, SHA310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918.
//! 두 경로의 같은 쪽 review·overlay를 직접 판독했다. 실물 검사는 원본의 portable 메트릭으로 실행한다.
//! 증적: mydocs/pr/assets/planet6897_20261106/last-line-dash-run-budget-20261109/validation.json.
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

#[test]
fn source_page707_keeps_the_independent_print_regular_run_and_last_dash_origins() {
    fn owned<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a TextRunNode)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.section_index == Some(0) && run.para_index == Some(24_857) {
                out.push((node, run));
            }
        }
        for child in &node.children {
            owned(child, out);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample10.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(706).unwrap();
    let mut rows = vec![];
    owned(&tree.root, &mut rows);
    assert_eq!(
        rows.len(),
        2,
        "같은 원본 문단의 굵은 대시와 일반 대시를 각각 한 번만 소유한다"
    );
    let source = &core.document().sections[0].paragraphs[24_857];
    assert_eq!(
        rows.iter()
            .map(|(_, r)| r.text.as_str())
            .collect::<String>(),
        source.text,
        "원문 순서·누락·중복을 보존한다"
    );
    assert!(rows[0].1.style.bold && !rows[1].1.style.bold);
    assert_eq!(rows[0].1.char_start, Some(0));
    assert_eq!(rows[1].1.char_start, Some(rows[0].1.text.chars().count()));
    let (regular_box, regular) = rows[1];
    // 독립 PDF T13의 첫/마지막 '-' 원점 228.8359832763672pt / 495.33709716796875pt.
    // 물리 pt를 CSS96으로 환산했으며 구현의 bbox에서 기대값을 만들지 않는다.
    assert!(
        (regular_box.bbox.x - 305.11464436848956).abs() <= 2.0,
        "일반 대시 런 원점: {}",
        regular_box.bbox.x
    );
    let positions = regular.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(&regular.text, &regular.style)
    });
    let last = regular_box.bbox.x + positions[positions.len() - 2];
    assert!(
        (last - 660.449462890625).abs() <= 2.0,
        "마지막 대시 원점: {last}"
    );
}
