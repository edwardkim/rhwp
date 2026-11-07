//! [#8715] 관계형 회귀 — 저장 조판에서 행 안을 나눈 표(최상위·중첩)의 첫 조각 상자는 그 표의
//! 저장 개체 높이(`sz.height`)를 차지하고, 이어받는·마지막 조각은 내용 높이만 차지한다.
//!
//! 주석 갱신: TBD
//!
//! - 입력 1: `samples/task2097/75544_pii_bunseok.hwpx`(66쪽). 59쪽에서 시작하는 1×1 바깥 표
//!   (저장 높이 56321 HU = 750.9px) 칸 안에 글 앞으로 놓인 2×1 중첩 표(저장 높이
//!   51122 HU = 681.6px)가 있고, 두 표 모두 59쪽에서 행 안이 나뉘어 60쪽으로 이어진다.
//! - 독립 기준 1: `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`. 59쪽 중첩 표 괘선
//!   358.5→1039.3px(680.8px), 바깥 표 291.4→1041.3px(750px) — 두 상자가 저장 높이와 같다.
//!   60쪽 이어받는 조각은 내용 끝(중첩 819.7px)에서 닫힌다.
//! - 입력 2: `samples/issue8715/nested_row_cell_cuts.hwp`(3쪽). 13×2 중첩 표 저장 높이
//!   56010 HU = 746.8px, 바깥 1×1 표 58888 HU = 785.2px. 독립 기준
//!   `pdf/issue8715_nested_row_cell_cuts-2020.pdf` 1쪽 중첩 280.3→1026.4px(746.1px),
//!   바깥 247.7→1032.0px(784.3px); 2쪽 이어받는 조각은 저장 높이와 무관한 내용 높이다.
//! - 수정 전 rhwp(#8744 head fdf67e5ec0): 75544 59쪽 중첩 상자 358.8→1023.9px(665px),
//!   바깥 상자 291.4→1017.7px(726px)로 저장 높이보다 짧고 중첩 표가 바깥 칸 밖으로 나갔다.
//!
//! 측정: TBD
//!
//! 검사는 첫 조각 상자 높이와 저장 개체 높이의 일치(원본 속성에서 계산한 크기 관계, ±1px),
//! 중첩 표의 바깥 칸 포함(관계형), 이어받는 조각이 저장 높이로 늘지 않음(관계형)이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const DPI: f64 = 96.0;

fn px(hu: u32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 문단 표 중 칸 문단에 (rows×cols) 표를 품은 바깥 표와 그 중첩 표.
fn outer_and_nested(core: &DocumentCore, rows: u16, cols: u16) -> (&Table, &Table) {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Table(outer) => outer.cells.iter().find_map(|cell| {
                cell.paragraphs.iter().find_map(|para| {
                    para.controls.iter().find_map(|control| match control {
                        Control::Table(child)
                            if child.row_count == rows && child.col_count == cols =>
                        {
                            Some((outer.as_ref(), child.as_ref()))
                        }
                        _ => None,
                    })
                })
            }),
            _ => None,
        })
        .expect("중첩 표")
}

/// (바깥 표 상자, 바깥 칸 상자, 중첩 표 상자).
fn boxes(
    node: &RenderNode,
    rows: u16,
    cols: u16,
) -> Option<(BoundingBox, BoundingBox, BoundingBox)> {
    if let RenderNodeType::Table(_) = &node.node_type {
        for cell in &node.children {
            if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
                continue;
            }
            for child in &cell.children {
                if let RenderNodeType::Table(table) = &child.node_type {
                    if table.row_count == rows && table.col_count == cols {
                        return Some((node.bbox.clone(), cell.bbox.clone(), child.bbox.clone()));
                    }
                }
            }
        }
    }
    node.children
        .iter()
        .find_map(|child| boxes(child, rows, cols))
}

fn page_boxes(
    core: &DocumentCore,
    page: u32,
    rows: u16,
    cols: u16,
) -> (BoundingBox, BoundingBox, BoundingBox) {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 렌더 트리: {e:?}", page + 1));
    boxes(&tree.root, rows, cols).unwrap_or_else(|| panic!("{}쪽 중첩 표", page + 1))
}

fn bottom(b: &BoundingBox) -> f64 {
    b.y + b.height
}

#[test]
fn first_fragment_boxes_take_stored_object_height() {
    let core = load("samples/task2097/75544_pii_bunseok.hwpx");
    let (outer, child) = outer_and_nested(&core, 2, 1);
    // 전제: 두 표 모두 `CELL` 분할이고, 중첩 표는 글 앞으로 놓였다.
    assert!(!outer.common.treat_as_char && !child.common.treat_as_char);
    let (outer_frame, child_frame) = (px(outer.common.height), px(child.common.height));

    let (outer_box, cell_box, nested_box) = page_boxes(&core, 58, 2, 1);
    // 바깥 상자는 중첩 표를 품고 닫힌다(정본 바깥−중첩 2.0px, rhwp 3.2px). 저장 높이와의
    // 차이는 그 닫힘 여백 차이 안이다.
    assert!(
        (outer_box.height - outer_frame).abs() <= 2.0,
        "59쪽 바깥 첫 조각 {:.1}px = 저장 높이 {outer_frame:.1}px",
        outer_box.height
    );
    assert!(
        (nested_box.height - child_frame).abs() <= 1.0,
        "59쪽 중첩 첫 조각 {:.1}px = 저장 높이 {child_frame:.1}px",
        nested_box.height
    );
    assert!(
        bottom(&nested_box) <= bottom(&cell_box) + 0.5,
        "중첩 표({:.1})는 바깥 칸({:.1}) 안",
        bottom(&nested_box),
        bottom(&cell_box)
    );

    // 이어받는 60쪽 조각도 바깥 칸 안에 있다(내용 높이 조각).
    let (_, cell_box, nested_box) = page_boxes(&core, 59, 2, 1);
    assert!(
        bottom(&nested_box) <= bottom(&cell_box) + 0.5,
        "60쪽 중첩 표({:.1})는 바깥 칸({:.1}) 안",
        bottom(&nested_box),
        bottom(&cell_box)
    );
}

#[test]
fn continuation_fragment_keeps_content_height() {
    let core = load("samples/issue8715/nested_row_cell_cuts.hwp");
    let (outer, child) = outer_and_nested(&core, 13, 2);
    let (outer_frame, child_frame) = (px(outer.common.height), px(child.common.height));

    // 1쪽 첫 조각은 저장 높이.
    let (outer_box, cell_box, nested_box) = page_boxes(&core, 0, 13, 2);
    assert!(
        (outer_box.height - outer_frame).abs() <= 1.0,
        "1쪽 바깥 첫 조각 {:.1}px = 저장 높이 {outer_frame:.1}px",
        outer_box.height
    );
    assert!(
        (nested_box.height - child_frame).abs() <= 1.0,
        "1쪽 중첩 첫 조각 {:.1}px = 저장 높이 {child_frame:.1}px",
        nested_box.height
    );
    assert!(bottom(&nested_box) <= bottom(&cell_box) + 0.5);

    // 2쪽 이어받는 조각: 중첩 표 아래와 바깥 칸 아래 사이에 칸 여백보다 큰 빈 띠가 남는다.
    let (_, cell_box, nested_box) = page_boxes(&core, 1, 13, 2);
    let pad = f64::from(outer.cells[0].padding.bottom) * DPI / 7200.0;
    assert!(
        bottom(&nested_box) <= bottom(&cell_box) + 0.5,
        "2쪽 중첩 표는 바깥 칸 안"
    );
    assert!(
        bottom(&nested_box) < bottom(&cell_box) - pad - 3.0,
        "2쪽 이어받는 조각은 칸 끝까지 늘지 않는다: 중첩 {:.1}, 칸 {:.1}",
        bottom(&nested_box),
        bottom(&cell_box)
    );
}
