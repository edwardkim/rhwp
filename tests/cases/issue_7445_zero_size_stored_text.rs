//! 원본의 명시적 0 크기는 텍스트를 보존하되 공백 전진을 만들지 않는다.
//! issue1948 원본 전1쪽 Native/fresh WASM과 독립 Print 대조 후 추가한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
        TextStyle,
    },
    DocumentCore,
};

#[test]
fn resolved_zero_size_keeps_spaces_zero_and_unresolved_defaults_nonzero() {
    let measurer = EmbeddedTextMeasurer;
    let mut style = TextStyle {
        font_family: "HY신명조,Haansoft Batang".into(),
        font_size: 0.0,
        resolved_font_size: true,
        layout_half_space: true,
        ..Default::default()
    };
    let positions = measurer.compute_char_positions("가 나\u{00a0}다", &style);
    assert_eq!(positions.len(), 6, "원문 스칼라별 소유를 유지한다");
    assert!(
        positions.iter().all(|x| *x == 0.0),
        "명시적0 크기의 글자와 빈칸 전진"
    );
    style.resolved_font_size = false;
    assert!(
        *measurer
            .compute_char_positions("가 나", &style)
            .last()
            .unwrap()
            > 0.0,
        "미해소 기본 스타일은 정상 폴백"
    );
    style.resolved_font_size = true;
    style.font_size = 16.0;
    style.superscript = true;
    let positions = measurer.compute_char_positions("가 나", &style);
    assert!(
        (positions[2] - positions[1] - 8.0).abs() < 0.03,
        "첨자 빈칸도 원본 본문 em의 반각을 소비한다"
    );
}

fn paragraph_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.section_index == Some(0) && run.para_index == Some(10) && run.cell_context.is_none())
    {
        out.push(node);
    }
    for child in &node.children {
        paragraph_runs(child, out);
    }
}

#[test]
fn stored_zero_size_notice_preserves_source_without_advancing_the_visible_body() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/hwpx/issue1948_cross_para_fieldend.hwpx"),
    )
    .expect("원본 교차 문단 fieldEnd fixture");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 쪽수");
    let source = &core.document().sections[0].paragraphs[10];
    assert!(source
        .char_shapes
        .iter()
        .any(|r| core.document().doc_info.char_shapes[r.char_shape_id as usize].base_size == 0));
    let tree = core.build_page_render_tree(0).unwrap();
    let mut runs = Vec::new();
    paragraph_runs(&tree.root, &mut runs);
    // 같은 원본 전1쪽 Native98.73123/WASM98.73890%와 독립 Print를
    // 직접 판독한 뒤, 숨은 폭 때문에 압축됐던 실제 본문의 첫 숫자를 보호한다.
    let number = runs
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TextRun(run) if run.char_start == Some(6) && run.text.starts_with(" 2. "))
        })
        .expect("원문 붙임2의 숫자 소유");
    let RenderNodeType::TextRun(run) = &number.node_type else {
        unreachable!()
    };
    let positions = run
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style));
    for (actual, expected) in [
        (number.bbox.x + positions[1], 131.1289333333),
        (number.bbox.y + run.baseline, 679.417492),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "완료1쪽 독립 Print 숫자 원점±2CSSpx: {actual} vs {expected}"
        );
    }
    let mut hidden = 0;
    for node in &runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        if run.style.resolved_font_size && run.style.font_size == 0.0 {
            hidden += 1;
            assert!(run.text.contains(' '), "원문에는 숨은 빈칸도 있다");
            let start = run.char_start.expect("원문 문자 소유");
            let owned: String = source
                .text
                .chars()
                .skip(start)
                .take(run.text.chars().count())
                .collect();
            assert_eq!(
                run.text, owned,
                "숨은 텍스트를 삭제하거나 다른 문단으로 옮기지 않는다"
            );
            assert_eq!(
                node.bbox.width, 0.0,
                "원본0 크기 안내문은 앞 본문의 정렬 폭을 빼앗지 않는다"
            );
        }
    }
    assert_eq!(hidden, 1, "원본0 크기 안내문을 한 번 소유한다");
    assert!(
        runs.iter()
            .any(|node| matches!(&node.node_type,RenderNodeType::TextRun(run)
        if run.style.font_size > 0.0 && run.text.contains("년 상반기"))),
        "실제 본문도 같은 쪽에 남는다"
    );
}
