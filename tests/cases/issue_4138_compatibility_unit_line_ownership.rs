//! 관계형: CJK 호환 단위 뒤 줄 경계와 같은 문단의 원문 보존을 검사한다.
//! 주석 갱신: 2026-11-07T00:18:17+09:00.
//! 2026-11-07 raw(192DPI, PDFium, 2px): 120쪽 Native/fresh WASM 각100.00000%.
//! 전197쪽 최저94.83034% / 94.98570%; 21·44·129·130쪽은 양쪽98% 미만으로 보정 중이다.
//! 부모fc038e4d54에 공통 줄 끝 후보를 보완한 source이며 절대 좌표는 별도 고정형 검사다.
//! 근거: mydocs/pr/assets/issue4138/compatibility-unit-20261107/README.md.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn text(n: &RenderNode) -> String {
    if let RenderNodeType::TextRun(t) = &n.node_type {
        return t.display_or_text().to_owned();
    }
    n.children.iter().map(text).collect()
}
fn lines(n: &RenderNode, out: &mut Vec<String>) {
    if matches!(&n.node_type,RenderNodeType::TextLine(m)if m.para_index==Some(1575)) {
        out.push(text(n));
    }
    for c in &n.children {
        lines(c, out)
    }
}
#[test]
fn compatibility_unit_keeps_its_print_line_ownership_and_source() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input=root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
        panic!("원본 표")
    };
    let source = table
        .cells
        .iter()
        .find(|c| c.row == 2 && c.col == 0)
        .unwrap()
        .paragraphs[1575]
        .text
        .clone();
    let t = core.build_page_render_tree(119).unwrap();
    let mut actual = Vec::new();
    lines(&t.root, &mut actual);
    assert!(
        actual
            .windows(2)
            .any(|w| w[0].trim_end().ends_with("0.5㎥") && w[1].trim_start().starts_with("/kg")),
        "독립 Print의 앞줄 종료·다음 줄 시작 소속"
    );
    let normal = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert_eq!(
        normal(&actual.concat()),
        normal(&source),
        "해당 문단 원문의 누락·중복 없음"
    );
}
