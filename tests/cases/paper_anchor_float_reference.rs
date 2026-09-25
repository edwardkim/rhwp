//! 종이·쪽 기준으로 놓인 개체의 기준 영역 — 합성 입력 두 갈래.
//!
//! 1. `square-table-next-line.hwpx` — 빈 앵커 문단에 걸린 **종이 기준 어울림(Square) 표**가
//!    본문 폭을 다 차지해 옆에 글이 흐를 레인이 없다(좌·우 레인 0px). 한글은 이 앵커 줄을 표
//!    아래로 옮기고 다음 문단을 그 뒤에 놓는다. 수정 전 rhwp 는 다음 문단을 앞 문단 흐름
//!    그대로 이어 **표 첫 행 위에 겹쳐** 그렸다.
//! 2. `cell-picture-page-paper.hwpx` — 표 칸 안 문단에 앵커된 그림이 위치 기준을 `종이`/`쪽` 으로
//!    가진다. 한글은 칸이 아니라 그 쪽의 용지·본문을 기준으로 오프셋을 잰다. 수정 전 rhwp 는 칸의
//!    안쪽 영역을 용지·본문 자리에 넘겨 오프셋이 **칸 시작점에서 한 번 더** 더해졌다.
//!
//! 입력은 공개 빈 문서 `samples/hwpx/ref/ref_empty.hwpx` 에서
//! `scripts/generate_paper_anchor_layout_fixtures.py` 로 만든다(사용자 서식 없음). 좌표 상수가
//! 아니라 표·본문·칸의 실제 bbox 와의 관계로 잠근다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SQUARE_TABLE_SAMPLE: &str = "samples/paper-anchor-layout/square-table-next-line.hwpx";
const CELL_PICTURE_SAMPLE: &str = "samples/paper-anchor-layout/cell-picture-page-paper.hwpx";

/// 생성기가 쓴 그림 위치(HWPUNIT). 96dpi 에서 1px = 75 HWPUNIT.
const PAPER_PICTURE_OFFSET_HU: (f64, f64) = (15000.0, 30000.0);
const PAGE_PICTURE_OFFSET_HU: (f64, f64) = (6000.0, 24000.0);
const TOLERANCE_PX: f64 = 0.5;

fn hu_to_px(hu: f64) -> f64 {
    hu / 75.0
}

fn first_page(sample: &str) -> RenderNode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let doc = DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open");
    assert_eq!(doc.page_count(), 1, "{sample}: 합성 입력은 한 쪽이다");
    doc.build_page_render_tree(0).expect("1쪽 render tree").root
}

fn bottom(bbox: &BoundingBox) -> f64 {
    bbox.y + bbox.height
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

fn line_text(node: &RenderNode) -> String {
    node.children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn text_line<'a>(root: &'a RenderNode, text: &str) -> &'a RenderNode {
    find(root, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).trim() == text
    })
    .unwrap_or_else(|| panic!("{text:?} 줄"))
}

#[test]
fn paper_anchored_square_table_without_side_lane_pushes_next_paragraph_below_it() {
    let root = first_page(SQUARE_TABLE_SAMPLE);
    let body = find(&root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");
    let table = find(&root, &|node| {
        matches!(node.node_type, RenderNodeType::Table(_))
    })
    .expect("종이 기준 어울림 표");
    let heading = text_line(&root, "Heading");
    let next = text_line(&root, "Next paragraph");

    // 전제 — 표가 본문 폭을 다 차지해(옆 레인 없음) 앞 문단보다 아래에 떠 있어야 이 계약이 선다.
    assert!(
        (table.bbox.x - body.bbox.x).abs() <= TOLERANCE_PX
            && (table.bbox.width - body.bbox.width).abs() <= TOLERANCE_PX,
        "표가 본문 폭 전체여야 한다 — 표 x={:.1} w={:.1}, 본문 x={:.1} w={:.1}",
        table.bbox.x,
        table.bbox.width,
        body.bbox.x,
        body.bbox.width,
    );
    assert!(
        table.bbox.y > bottom(&heading.bbox),
        "표는 앞 문단 아래(종이 기준 오프셋)에 떠 있어야 한다 — 표 y={:.1}, 앞 문단 바닥={:.1}",
        table.bbox.y,
        bottom(&heading.bbox),
    );

    assert!(
        next.bbox.y >= bottom(&table.bbox) - TOLERANCE_PX,
        "옆 레인이 없는 종이 기준 어울림 표 뒤 문단은 표 아래에서 시작해야 한다 — \
         다음 문단 y={:.1}, 표 y={:.1}..{:.1}",
        next.bbox.y,
        table.bbox.y,
        bottom(&table.bbox),
    );
}

#[test]
fn cell_anchored_pictures_measure_page_and_paper_offsets_from_the_page() {
    let root = first_page(CELL_PICTURE_SAMPLE);
    let body = find(&root, &|node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");

    let cell_picture = |col: u16| -> (BoundingBox, BoundingBox) {
        let cell = find(&root, &|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == col)
        })
        .unwrap_or_else(|| panic!("칸 (0,{col})"));
        let image = cell
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::Image(_)))
            .unwrap_or_else(|| panic!("칸 (0,{col}) 안 그림"));
        (cell.bbox, image.bbox)
    };

    // 종이 기준 그림 — 원점은 용지 왼쪽 위 (0, 0).
    let (paper_cell, paper_image) = cell_picture(0);
    let expected = (
        hu_to_px(PAPER_PICTURE_OFFSET_HU.0),
        hu_to_px(PAPER_PICTURE_OFFSET_HU.1),
    );
    assert!(
        paper_cell.x > 50.0 && paper_cell.y > 50.0,
        "전제: 칸 시작점이 용지 원점에서 떨어져 있어야 두 기준이 갈린다 — 칸 ({:.1}, {:.1})",
        paper_cell.x,
        paper_cell.y,
    );
    assert!(
        (paper_image.x - expected.0).abs() <= TOLERANCE_PX
            && (paper_image.y - expected.1).abs() <= TOLERANCE_PX,
        "칸 안 종이 기준 그림은 용지 원점 + 오프셋({:.1}, {:.1})에 놓여야 한다 — 실제 ({:.1}, {:.1}), \
         칸 시작점 ({:.1}, {:.1})",
        expected.0,
        expected.1,
        paper_image.x,
        paper_image.y,
        paper_cell.x,
        paper_cell.y,
    );

    // 쪽 기준 그림 — 원점은 본문 영역 왼쪽 위.
    let (page_cell, page_image) = cell_picture(1);
    let expected = (
        body.bbox.x + hu_to_px(PAGE_PICTURE_OFFSET_HU.0),
        body.bbox.y + hu_to_px(PAGE_PICTURE_OFFSET_HU.1),
    );
    assert!(
        page_cell.x - body.bbox.x > 50.0,
        "전제: 둘째 칸 시작점이 본문 왼쪽에서 떨어져 있어야 두 기준이 갈린다 — 칸 x={:.1}, 본문 x={:.1}",
        page_cell.x,
        body.bbox.x,
    );
    assert!(
        (page_image.x - expected.0).abs() <= TOLERANCE_PX
            && (page_image.y - expected.1).abs() <= TOLERANCE_PX,
        "칸 안 쪽 기준 그림은 본문 원점 + 오프셋({:.1}, {:.1})에 놓여야 한다 — 실제 ({:.1}, {:.1}), \
         칸 시작점 ({:.1}, {:.1})",
        expected.0,
        expected.1,
        page_image.x,
        page_image.y,
        page_cell.x,
        page_cell.y,
    );
}
