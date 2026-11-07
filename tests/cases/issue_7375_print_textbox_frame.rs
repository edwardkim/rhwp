//! 고정형: 독립 Print의 글상자 외곽을 96DPI ±2px로 보호한다.
//! 주석 갱신: 2026-11-06T23:54:43+09:00.
//! 2026-11-06 raw(96DPI, PDFium, 2px): 원본·목록 선언·실사용 선언 각1쪽,
//! Native/fresh WASM 모두100.00000%; 문서별 전쪽 최저도 각각100.00000%.
//! 상자/쪽 좌표 Top 대조군도 각각 전1쪽, 양쪽100.00000%로 확인했다.
//! 부모4b03792e56 + 최소 높이 보정의 source·입력·PDF 해시는 아래 정식 증거로 고정한다.
//! 원본 줄·여백·정렬·직렬화 보존은 issue_7375_overlay_textbox_minimum에서 별도 검사한다.
//! 근거: mydocs/pr/assets/issue7375/paper-textbox-minimum-20261106/README.md.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn find(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Rectangle(_))
        && node
            .children
            .iter()
            .any(|child| matches!(child.node_type, RenderNodeType::TextBox))
    {
        return Some(node);
    }
    node.children.iter().find_map(find)
}

const BOTTOM_BOX_PT: [f64; 4] = [
    73.63999938964844,
    679.0570068359375,
    163.5919952392578,
    693.6810302734375,
];
const TOP_BOX_PT: [f64; 4] = [
    73.63999938964844,
    694.52001953125,
    163.5919952392578,
    709.1439819335938,
];

fn check(input: &str, pdf: [f64; 4]) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(root.join(input)).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let page = core.build_page_render_tree(0).unwrap();
    let shape = find(&page.root).expect("독립 Print에 대응하는 꼬리말 글상자");
    // 대응 기준 PDF 각1쪽에서 독립 추출한 사각형의 경계(pt).
    // 구현의 글자 크기·여백·height 계산을 기대값으로 재인용하지 않는다.
    let expected = [pdf[0], pdf[1], pdf[2] - pdf[0], pdf[3] - pdf[1]];
    let actual = [
        shape.bbox.x,
        shape.bbox.y,
        shape.bbox.width,
        shape.bbox.height,
    ];
    for (axis, (actual, expected)) in ["x", "y", "width", "height"]
        .into_iter()
        .zip(actual.into_iter().zip(expected))
    {
        let expected = expected * 96.0 / 72.0;
        assert!(
            (actual - expected).abs() <= 2.0,
            "{input} {axis}: 독립 Print ±2px, actual={actual}, expected={expected}"
        );
    }
}

#[test]
fn original_paper_box_matches_the_independent_print_frame() {
    check("samples/header-footer-paper-frame.hwpx", BOTTOM_BOX_PT);
}

#[test]
fn unused_list_font_box_matches_the_independent_print_frame() {
    check("pdf/issue8687/font_list_name_fallback.hwpx", BOTTOM_BOX_PT);
}

#[test]
fn used_list_font_box_matches_the_independent_print_frame() {
    check(
        "pdf/issue8687/font_list_name_used_fallback.hwpx",
        BOTTOM_BOX_PT,
    );
}

#[test]
fn box_local_vpos_matches_the_independent_top_frame() {
    check("pdf/issue7375/paper-textbox-vpos-local.hwpx", TOP_BOX_PT);
}

#[test]
fn page_vpos_matches_the_independent_top_frame() {
    check("pdf/issue7375/paper-textbox-vpos-page.hwpx", TOP_BOX_PT);
}
