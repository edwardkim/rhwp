//! 고정형: 낱말 사이 연속 공백 뒤 이어받는 첫 글자의 독립 Print 원점을 192DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-07T02:11:45+09:00.
//! 21쪽 raw: Native 99.97728% / fresh WASM 99.97745% (192DPI, PDFium, 2px).
//! 전197쪽 최저 94.83034% / 94.98570%; 44·129·130쪽은 별도 수식 보정 중이다.
//! 독립 한컴2024 Print의 첫 '다' 원점 (95.947998046875, 431.76812744140625)pt를 사용한다.
//! 줄 소속과 작성 공백 보존은 issue_4138_interword_space_ownership의 관계형 검사로 보호한다.
//! 근거: mydocs/pr/assets/issue4138/interword-space-20261107/README.md.

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn continuation(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.para_index == Some(271) && run.text.contains("다음에 따라 계산하여야 한다."))
    {
        return Some(node);
    }
    node.children.iter().find_map(continuation)
}

#[test]
fn print_interword_continuation_origin_matches_the_independent_pdf() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(20).unwrap();
    let node = continuation(&page.root).expect("독립 Print 21쪽의 문단271 이어받기");
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    let index = run.text.chars().position(|ch| ch == '다').unwrap();
    let positions = run.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style)
    });
    let actual = [
        (node.bbox.x + positions[index]) * (192.0 / 96.0),
        (node.bbox.y + run.baseline) * (192.0 / 96.0),
    ];
    let expected = [95.947_998_046_875, 431.768_127_441_406_25];
    for (axis, (actual, expected)) in ["x", "baseline"]
        .into_iter()
        .zip(actual.into_iter().zip(expected))
    {
        let expected = expected * (192.0 / 72.0);
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 한컴 Print 원점 {axis} ±2px: actual={actual}, expected={expected}"
        );
    }
}
