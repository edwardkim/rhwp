//! 관계형·고정형 회귀 — #7772의 제어문자 그림·곡선·셀 안 후속 설명의 소속과 쪽수를 보호한다.
//! 주석 갱신 2026-11-24T07:31:13+09:00. 직전 파일 변경 2026-11-24T02:00:37+09:00
//! `8e64e08a1842b1bb938f159528746de0a8edc0f5`.
//! 2026-11-23 한컴 2024 1-up Print, 96 DPI 2px raw 실루엣 대조.
//! 20·106쪽 Native/fresh WASM 각각 100.00%, 21쪽 각각 99.99%.
//! 전178쪽 최저 Native 98.14%, fresh WASM 98.16%.
//! 전쪽 TSV와 확대 판독은 `mydocs/pr/assets/issue7772/README.md`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn figures(node: &RenderNode, owners: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.section_index == Some(2)
            && image.control_index == Some(0)
            && image.cell_index.is_none()
        {
            if let Some(para) = image
                .para_index
                .filter(|&para| matches!(para, 83 | 86 | 89))
            {
                owners.push((para, node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
    }
    for child in &node.children {
        figures(child, owners);
    }
}

#[test]
fn hwpspec_has_one_print_figure_per_control_on_the_owning_page() {
    let mut document =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpspec.hwp")).expect("사양서 원본");
    document.set_hangul2024_compat(true);
    assert_eq!(document.page_count(), 178, "독립 한컴 Print의 쪽수");

    let page20 = document.build_page_render_tree(19).expect("20쪽 본문");
    let page21 = document.build_page_render_tree(20).expect("21쪽 본문");
    let mut owners20 = Vec::new();
    let mut owners21 = Vec::new();
    figures(&page20.root, &mut owners20);
    figures(&page21.root, &mut owners21);
    let [first, second] = owners20.as_slice() else {
        panic!("20쪽에 문자와 인라인 그림 한 점씩: {owners20:?}");
    };
    assert_eq!((first.0, second.0), (83, 86));
    assert!(first.2 < second.1, "두 그림은 원문 순서를 지킨다");
    let [extended] = owners21.as_slice() else {
        panic!("21쪽에 확장 컨트롤 그림 한 점: {owners21:?}");
    };
    assert_eq!(extended.0, 89);
    let next_text = page21
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("21쪽 본문")
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .flat_map(|node| &node.children)
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(90))
        })
        .expect("확장 컨트롤 뒤 본문");
    assert!(
        extended.2 < next_text.bbox.y,
        "그림을 설명 본문과 겹치지 않게 배치한다"
    );
}

#[test]
fn hwpspec_curve_reserves_its_full_line_before_the_next_label() {
    let mut document =
        DocumentCore::from_bytes(include_bytes!("../../samples/hwpspec.hwp")).expect("사양서 원본");
    document.set_hangul2024_compat(true);
    let tree = document
        .build_page_render_tree(105)
        .expect("106쪽 곡선 설명");
    let table = tree.root.children.iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .flat_map(|body| &body.children)
        .filter(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .flat_map(|column| &column.children)
        .find(|node| matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(344)))
        .expect("곡선 설명의 본문 표");
    let cell = table
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .expect("곡선과 좌표 라벨이 있는 셀");
    let curve = cell
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Path(_)))
        .expect("셀에 속한 곡선 도형");
    let label = |paragraph, expected: &str| {
        cell.children.iter().find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(paragraph))
                && node.children.iter().any(|node| matches!(&node.node_type,
                    RenderNodeType::TextRun(text) if text.text.trim() == expected))
        }).expect("곡선 셀에 원본 소속의 좌표 라벨을 한 번 그린다")
    };
    let x2 = label(2, "(x2, y2)");
    let x3 = label(3, "(x3, y3)");
    let bottom = curve.bbox.y + curve.bbox.height;
    let RenderNodeType::Table(owner) = &table.node_type else {
        unreachable!()
    };
    let source = &document.document().sections[owner.section_index.unwrap()].paragraphs[344];
    let rhwp::model::control::Control::Table(model) =
        &source.controls[owner.control_index.unwrap()]
    else {
        panic!("곡선 라벨의 저장 표")
    };
    let stored_line = &model.cells[0].paragraphs[2].line_segs[0];
    let saved_height = f64::from(stored_line.line_height) / 75.0;
    assert!(
        (x2.bbox.height - saved_height).abs() <= 1.0 / 75.0,
        "곡선과 같은 라벨 줄이 문서가 기록한 전체 높이를 소비한다"
    );
    let center = curve.bbox.y + curve.bbox.height / 2.0;
    assert!((x2.bbox.y..=x2.bbox.y + x2.bbox.height).contains(&center));
    assert!(bottom <= x3.bbox.y && x2.bbox.y + x2.bbox.height <= x3.bbox.y);
    assert!(x3.bbox.y + x3.bbox.height <= cell.bbox.y + cell.bbox.height);
}
