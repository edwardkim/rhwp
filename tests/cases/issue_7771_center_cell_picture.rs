//! 관계형·고정형 회귀 — #7771의 셀 이미지와 쪽 중간 표 소속.
//! 주석 작성 2026-11-23 20:46 KST.
//! 2026-11-23 독립 한컴 2020 Print, 96 DPI, 2px raw 실루엣.
//! 전20쪽 Native/fresh WASM 최저 99.05%/99.04%, 8쪽 99.74%/99.74%.
//! 19쪽은 두 경로 모두 99.87%다.
//! 근거는 `mydocs/pr/assets/issue7771/`의 Native·WASM 전쪽 TSV와 비교 PNG다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &str = "tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재.hwp";

fn body_column(root: &RenderNode) -> &RenderNode {
    root.children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문")
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .expect("본문 단")
}

fn table(column: &RenderNode, owner: usize) -> &RenderNode {
    column
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
            RenderNodeType::Table(table) if table.para_index == Some(owner))
        })
        .expect("원본 문단의 표")
}

#[test]
fn cell_picture_uses_its_outer_margin_and_midpage_table_keeps_heading() {
    let document =
        DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("검증 원본")).expect("문서 열기");
    assert_eq!(document.page_count(), 20, "독립 Print의 쪽수");

    let output8 = document.build_page_render_tree(7).expect("8쪽 이미지");
    let comparison = table(body_column(&output8.root), 100);
    let right = comparison
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(cell) if cell.row == 1 && cell.col == 1)
        })
        .expect("오른쪽 칸");
    let mut pictures = right
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Group(_)))
        .flat_map(|node| &node.children)
        .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)));
    let image = pictures.next().expect("오른쪽 칸의 묶음 그림");
    assert!(
        pictures.next().is_none(),
        "오른쪽 칸의 그림은 한 번만 방출된다"
    );
    // 독립 96 DPI Print에서 오른쪽 칸의 화면 캡처가 시작하는 x.
    assert!((image.bbox.x - 221.0).abs() <= 2.0);
    assert!(image.bbox.y >= right.bbox.y);
    assert!(image.bbox.y + image.bbox.height <= right.bbox.y + right.bbox.height);

    let output19 = document.build_page_render_tree(18).expect("19쪽 중간 표");
    let column = body_column(&output19.root);
    let middle = table(column, 204);
    let heading = column
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(203))
        })
        .expect("표 위 제목");
    assert!(heading.children.iter().any(|node| matches!(&node.node_type,
        RenderNodeType::TextRun(run) if run.text.contains("청약철회 방해 행위 관련"))));
    assert!(heading.bbox.y + heading.bbox.height < middle.bbox.y);
    assert!(middle.bbox.y + middle.bbox.height <= column.bbox.y + column.bbox.height);
}
