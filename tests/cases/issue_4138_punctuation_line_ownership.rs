//! 관계형: #4138 한점 리더·한자 뒤 닫는 괄호의 줄 소속과 원문 보존을 검사한다.
//! 주석 갱신: 2026-11-06T22:39:06+09:00.
//! 2026-11-06 raw(192DPI, PDFium, 2px): 30쪽 Native/fresh WASM 각각100.00000%,
//! 189쪽 99.98891% / 99.99010%; 전197쪽 최저94.83034% / 94.98570%.
//! producer는 부모708256f1ba에 공통 금칙 보정을 적용한 source이며 validation.json의 diff·파일 해시로 고정한다.
//! 절대 좌표는 별도 고정형 검사로 유지한다. 전쪽98% 완료를 뜻하지 않는다.
//! 근거: mydocs/pr/assets/issue4138/closing-punctuation-20261106/README.md.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn text(n: &RenderNode) -> String {
    if let RenderNodeType::TextRun(r) = &n.node_type {
        return r.display_or_text().to_owned();
    }
    n.children.iter().map(text).collect()
}
fn lines(n: &RenderNode, owner: usize, out: &mut Vec<String>) {
    if matches!(&n.node_type,RenderNodeType::TextLine(m)if m.para_index==Some(owner)) {
        out.push(text(n))
    }
    for c in &n.children {
        lines(c, owner, out)
    }
}
fn check(page: u32, owner: usize, last: &str, next: &str) {
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core=DocumentCore::from_bytes(&std::fs::read(r.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp")).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
        panic!("원문 표")
    };
    let source = table
        .cells
        .iter()
        .find(|c| c.row == 2 && c.col == 0)
        .unwrap()
        .paragraphs[owner]
        .text
        .clone();
    let t = core.build_page_render_tree(page - 1).unwrap();
    let mut all = vec![];
    lines(&t.root, owner, &mut all);
    assert!(
        all.windows(2)
            .any(|w| w[0].trim_end().ends_with(last) && w[1].trim_start().starts_with(next)),
        "독립 Print의 앞줄 종료·다음 줄 시작 소속"
    );
    let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    assert_eq!(
        normalize(&all.concat()),
        normalize(&source),
        "같은 페이지의 원문 문단에 누락·중복이 없어야 함"
    )
}
#[test]
fn one_dot_keeps_the_previous_character_and_all_source_text() {
    check(30, 396, "흡", "기․냉각수")
}
#[test]
fn cjk_closer_keeps_the_previous_character_and_all_source_text() {
    check(189, 2445, "(滴下", "物)의")
}
