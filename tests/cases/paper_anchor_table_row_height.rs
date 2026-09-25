//! 칸 세로 안 여백이 **저장값이 아닌** 표의 행 높이 — 합성 입력 두 갈래.
//!
//! 두 입력 모두 표 안 여백(`hp:inMargin`)이 네 축 0(= 미지정)이고 칸은 `hasMargin="0"` 이다.
//! 그래서 칸에 저장된 `cellMargin` 은 한글이 그대로 쓰는 값이 아니다(#2195 대체 규칙).
//!
//! 1. `paper-table-fallback-padding.hwpx` — **종이 기준으로 떠 있는** 3행 표. 칸마다 저장 줄
//!    하나(1000 HU)가 선언 행 높이 1800 HU 안에 들고, 저장 칸 여백은 위·아래 850 HU 다. 수정 전
//!    rhwp 는 이 대체 여백을 줄 높이에 더해(1000 + 850×2 = 2700 HU) 행마다 선언보다 12px 키웠다.
//!    떠 있는 개체는 선언 크기를 지킨다 — 행은 선언 높이 그대로여야 한다.
//! 2. `residual-cell-padding-pair.hwpx` — 문단 기준으로 놓인 본문 흐름 표 한 칸(종이·쪽 기준이 아니라
//!    1번 규칙은 걸리지 않는다). 저장 칸 여백이 위 20424 · 아래 1287 HU 인 **잔재 쌍**이다. 위 축은 위생 한도(2500 HU)에 걸려 이미 버려지지만,
//!    수정 전 rhwp 는 아래 축만 살려 선언 1740 HU 행을 1287 HU 만큼 키웠다. 한 축이 잔재면 그 쌍은
//!    같은 잔재이므로 다른 축도 쓰지 않는다 — 행은 선언 높이 그대로여야 한다.
//!
//! 입력은 공개 빈 문서 `samples/hwpx/ref/ref_empty.hwpx` 에서
//! `scripts/generate_paper_anchor_layout_fixtures.py` 로 만든다(사용자 서식 없음).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FALLBACK_PADDING_SAMPLE: &str =
    "samples/paper-anchor-layout/paper-table-fallback-padding.hwpx";
const RESIDUAL_PAIR_SAMPLE: &str = "samples/paper-anchor-layout/residual-cell-padding-pair.hwpx";

/// 생성기가 쓴 선언 행 높이(HWPUNIT). 96dpi 에서 1px = 75 HWPUNIT.
const FALLBACK_PADDING_ROW_HU: f64 = 1800.0;
const RESIDUAL_PAIR_ROW_HU: f64 = 1740.0;
const TOLERANCE_PX: f64 = 0.5;

fn first_page(sample: &str) -> RenderNode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let doc = DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open");
    assert_eq!(doc.page_count(), 1, "{sample}: 합성 입력은 한 쪽이다");
    doc.build_page_render_tree(0).expect("1쪽 render tree").root
}

fn collect_cells<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_cells(child, out);
    }
}

/// 모든 칸이 선언 행 높이로 그려지고, 칸 안 저장 줄이 그 칸 안에 든다.
fn assert_rows_keep_declared_height(sample: &str, expected_rows: usize, declared_hu: f64) {
    let root = first_page(sample);
    let mut cells = Vec::new();
    collect_cells(&root, &mut cells);
    assert_eq!(cells.len(), expected_rows, "{sample}: 칸 수");

    let declared_px = declared_hu / 75.0;
    for cell in cells {
        let RenderNodeType::TableCell(address) = &cell.node_type else {
            unreachable!("위에서 TableCell 만 모았다");
        };
        let line = cell
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::TextLine(_)))
            .unwrap_or_else(|| panic!("{sample}: 칸 ({},{}) 줄", address.row, address.col));
        // 전제 — 저장 줄이 선언 높이 안에 든다(줄이 넘치면 행이 커지는 것이 맞다).
        assert!(
            line.bbox.height <= declared_px,
            "{sample}: 전제 — 칸 ({},{}) 줄 높이 {:.1}px 가 선언 행 {:.1}px 안에 들어야 한다",
            address.row,
            address.col,
            line.bbox.height,
            declared_px,
        );
        assert!(
            (cell.bbox.height - declared_px).abs() <= TOLERANCE_PX,
            "{sample}: 칸 ({},{}) 행 높이는 선언 {:.1}px 여야 한다 — 실제 {:.1}px \
             (저장 칸 여백이 대체값·잔재인데 행을 키웠다)",
            address.row,
            address.col,
            declared_px,
            cell.bbox.height,
        );
    }
}

#[test]
fn paper_anchored_table_rows_ignore_fallback_cell_padding_within_declared_height() {
    assert_rows_keep_declared_height(FALLBACK_PADDING_SAMPLE, 3, FALLBACK_PADDING_ROW_HU);
}

#[test]
fn residual_cell_vertical_padding_pair_does_not_grow_the_row() {
    assert_rows_keep_declared_height(RESIDUAL_PAIR_SAMPLE, 1, RESIDUAL_PAIR_ROW_HU);
}
