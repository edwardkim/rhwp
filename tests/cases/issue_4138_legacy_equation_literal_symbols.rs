//! 실제 과거 분할 저장본과 독립 한컴2024 Print의130쪽 기호 슬롯 계약.
//! 원문 △/U+2024는 보존하고 그리는 글리프만 수식 기호로 투영한다.
//! 두 Type3 기호의 독립 /Widths=1000/1000em; 페이지 절대 px 고정은 아니다.
use rhwp::renderer::{
    equation::{
        hancom_glyphs::GlyphRun,
        layout::{LayoutBox, LayoutKind},
    },
    render_tree::{EquationNode, RenderNode, RenderNodeType},
};
use rhwp::DocumentCore;
fn energy(node: &RenderNode) -> Option<&EquationNode> {
    if let RenderNodeType::Equation(e) = &node.node_type {
        if e.script.contains("0.5") && e.script.contains("V_imp") {
            return Some(e);
        }
    }
    node.children.iter().find_map(energy)
}
fn literal_run(b: &LayoutBox) -> Option<&GlyphRun> {
    match &b.kind {
        LayoutKind::HancomGlyphs(r) if r.glyphs.contains('△') => Some(r),
        LayoutKind::Row(children) => children.iter().find_map(literal_run),
        LayoutKind::FontStyle { body, .. } => literal_run(body),
        _ => None,
    }
}
#[test]
fn literal_symbols_use_independent_full_em_slots_in_final_paint() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/issue4138/tab-period-split.hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(129).unwrap();
    let eq = energy(&tree.root).expect("독립 Print130쪽의 운동 에너지 수식 소속");
    assert!(eq.script.contains("△․"), "원본 기호 스크립트 보존");
    assert!(
        core.extract_page_text_native(129).unwrap().contains("△․"),
        "텍스트 추출은 그리기용 글리프로 대체하지 않음"
    );
    let glyphs = literal_run(&eq.layout_box).expect("실제 paint가 읽는 수식 글꼴의 기호 슬롯");
    assert_eq!(glyphs.glyphs, "△⋅");
    assert_eq!(glyphs.positions.len(), 2);
    assert!(
        (glyphs.positions[1] - glyphs.positions[0] - eq.font_size).abs() < 1e-6,
        "독립 Type3의 첫 기호1em 전진폭"
    );
    assert!(
        (glyphs.width - 2.0 * eq.font_size).abs() < 1e-6,
        "독립 Type3의 두 기호2em 슬롯"
    );
    let painted = eq
        .svg_content
        .lines()
        .filter(|l| l.contains('△') || l.contains('⋅'))
        .collect::<Vec<_>>();
    assert_eq!(painted.len(), 2);
    assert!(
        painted
            .iter()
            .all(|l| l.contains("font-family=\"HyhwpEQ\"")),
        "SVG도 공통 글리프 선택을 소비"
    );
}
