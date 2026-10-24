//! #7445/#4138: Frame 재조판에서도 수식의 줄간격 참여 높이를 소비한다.
//! 저장197쪽 입력의 Native/fresh WASM 전쪽 raw≥90%, 130쪽은 각각
//! 98.19711/98.16280%다. 독립 한컴2024 Print의 같은 두 제목 기준선 차이는
//! 98.772056CSSpx이며 해당 쪽의 ±2CSSpx 보호와 원본 속성 관계 검사를 함께 둔다.
//! 참여/비참여 수식과 실제 빈 후속 줄의 원본 소속을 검사한다.
//! 보정 전 frozen13 두 건 FAIL → frozen19 두 건 PASS. 증적은 stage8 C425와
//! ignored rebase-4a07d3ae-20261019/repair26-frame-leading-regression/에 있다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::{control::Control, document::Document, table::Cell},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn input() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp")).unwrap()
}
fn body(d: &Document) -> &Cell {
    let Control::Table(t) = &d.sections[0].paragraphs[0].controls[2] else {
        panic!("source table")
    };
    t.cells.iter().find(|c| c.row == 2 && c.col == 0).unwrap()
}
fn lines<'a>(n: &'a RenderNode, pi: usize, li: u32, out: &mut Vec<&'a RenderNode>) {
    if matches!(&n.node_type,RenderNodeType::TextLine(l)if l.para_index==Some(pi)&&l.line_index==Some(li))
    {
        out.push(n)
    }
    for c in &n.children {
        lines(c, pi, li, out)
    }
}
fn line(n: &RenderNode, pi: usize, li: u32) -> &RenderNode {
    let mut out = vec![];
    lines(n, pi, li, &mut out);
    assert_eq!(out.len(), 1, "unique source line {pi}/{li}");
    out[0]
}
fn render() -> rhwp::renderer::render_tree::PageRenderTree {
    let mut d = DocumentCore::from_bytes(&input()).unwrap();
    d.set_hangul2024_compat(true);
    d.build_page_render_tree(129).unwrap()
}
#[test]
fn participating_equation_advances_the_next_owned_empty_line_by_source_percent_spacing() {
    let raw = rhwp::parser::parse_hwp(&input()).unwrap();
    let source = body(&raw);
    let p = &source.paragraphs[1698];
    let Control::Equation(eq) = &p.controls[0] else {
        panic!("source equation")
    };
    assert!(eq.common.treat_as_char && eq.common.affect_line_spacing);
    let style = &raw.doc_info.para_shapes[p.para_shape_id as usize];
    assert!(matches!(
        style.line_spacing_type,
        rhwp::model::style::LineSpacingType::Percent
    ));
    assert!(style.line_spacing > 100);
    assert!(source.paragraphs[1699].text.is_empty());
    let t = render();
    let host = line(&t.root, 1698, 0);
    let next = line(&t.root, 1699, 0);
    let expected = f64::from(eq.common.height) * f64::from(style.line_spacing) / 100.0 / 75.0;
    let actual = next.bbox.y - host.bbox.y;
    assert!(
        (host.bbox.height - f64::from(eq.common.height) / 75.0).abs() <= 1.0 / 75.0,
        "the source equation owns this flow row"
    );
    assert!(actual>=expected,"participating equation uses its source height in Percent spacing: step={actual}, minimum={expected}, height={}, percent={}",eq.common.height,style.line_spacing);
    let control = &source.paragraphs[1701];
    let Control::Equation(eq) = &control.controls[0] else {
        panic!("nonparticipating equation")
    };
    assert!(!eq.common.affect_line_spacing);
    let a = line(&t.root, 1701, 0);
    let b = line(&t.root, 1701, 1);
    let expected = f64::from(eq.common.height as i32 + control.line_segs[0].line_spacing) / 75.0;
    let actual = b.bbox.y - a.bbox.y;
    assert!(
        (actual - expected).abs() <= 1.0 / 75.0,
        "nonparticipating source keeps its saved text leading: {actual} vs {expected}"
    );
}
#[test]
fn completed_page130_keeps_independent_print_heading_gap_after_participating_equation() {
    let t = render();
    let a = line(&t.root, 1697, 0);
    let b = line(&t.root, 1700, 0);
    // Independent Hancom2024 Print p130: same-face heading baselines248.0489173333
    // and346.8209733333 CSSpx. Their difference removes the common font baseline.
    let expected = 346.8209733333333 - 248.04891733333338;
    let actual = b.bbox.y - a.bbox.y;
    assert!(
        (actual - expected).abs() <= 2.0,
        "independent Print heading gap: actual={actual}, expected={expected}, tolerance2CSSpx"
    );
}
