//! 고정형: #4138 한점 리더·한자 뒤 닫는 괄호의 독립 Print 줄 원점을 192DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-06T22:39:06+09:00.
//! 2026-11-06 raw(192DPI, PDFium, 2px): 30쪽 Native/fresh WASM 각각100.00000%,
//! 189쪽 99.98891% / 99.99010%; 전197쪽 최저94.83034% / 94.98570%.
//! producer는 부모708256f1ba에 공통 금칙 보정을 적용한 source이며 validation.json의 diff·파일 해시로 고정한다.
//! 원문 소유·내용 보존은 별도 관계 검사로 유지한다. 전쪽98% 완료를 뜻하지 않는다.
//! 근거: mydocs/pr/assets/issue4138/closing-punctuation-20261106/README.md.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn text(n: &RenderNode) -> String {
    if let RenderNodeType::TextRun(r) = &n.node_type {
        return r.display_or_text().to_owned();
    }
    n.children.iter().map(text).collect()
}
fn find<'a>(n: &'a RenderNode, owner: usize, start: &str) -> Option<&'a RenderNode> {
    if matches!(&n.node_type,RenderNodeType::TextLine(m)if m.para_index==Some(owner))
        && text(n).starts_with(start)
    {
        return Some(n);
    }
    n.children.iter().find_map(|c| find(c, owner, start))
}
fn check(page: u32, owner: usize, start: &str, pdf_x: f64) {
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core=DocumentCore::from_bytes(&std::fs::read(r.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp")).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let t = core.build_page_render_tree(page - 1).unwrap();
    let line = find(&t.root, owner, start).expect("독립 Print의 이어받은 줄 소유");
    let physical_x = line.bbox.x * 2.0;
    let expected = pdf_x * 192.0 / 72.0;
    assert!(
        (physical_x - expected).abs() <= 2.0,
        "독립 PDF의 192DPI 원점 ±2px: {physical_x} / {expected}"
    )
}
#[test]
fn one_dot_continuation_matches_the_independent_print_origin() {
    check(30, 396, "기․냉각수", 131.568_695_068_359_38)
}
#[test]
fn cjk_closing_continuation_matches_the_independent_print_origin() {
    check(189, 2445, "物)의", 103.863_708_496_093_75)
}
