//! [#8715] 관계형 회귀 — 저장 조판에서 행 안을 나눈 중첩 표의 첫 조각 상자는, 그 표가 글 앞/뒤
//! (`InFrontOfText`/`BehindText`)로 놓여 흐름 밖에 있을 때만 저장 개체 높이(`sz.height`)를 차지한다.
//! 글 흐름(위아래 등) 자식 표의 첫 조각과 모든 이어받는 조각은 내용 높이에서 닫힌다.
//!
//! 주석 갱신: 2026-11-09 00:50 KST (직전 파일 변경 e52235e4e3, 2026-11-08 09:52:58 +0900)
//!
//! - 입력 1: `samples/task2097/75544_pii_bunseok.hwpx`(66쪽). 59쪽에서 시작하는 1×1 바깥 표
//!   (저장 높이 56321 HU = 750.9px) 칸 안에 글 앞으로 놓인 2×1 중첩 표(저장 높이
//!   51122 HU = 681.6px)가 있고, 두 표 모두 59쪽에서 행 안이 나뉘어 60쪽으로 이어진다.
//!   독립 기준 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf` 59쪽 중첩 표 괘선
//!   358.5→1039.3px(680.8px), 바깥 표 291.4→1041.3px(750px) — 두 상자가 저장 높이와 같다.
//!   60쪽 이어받는 조각은 내용 끝에서 닫힌다.
//! - 입력 2: `samples/issue8715/nested_row_cell_cuts.hwp`(3쪽). 13×2 중첩 표 저장 높이
//!   56010 HU = 746.8px, 바깥 1×1 표 58888 HU = 785.2px. 독립 기준
//!   `pdf/issue8715_nested_row_cell_cuts-2020.pdf` 1쪽 중첩 280.3→1026.4px(746.1px),
//!   바깥 247.7→1032.0px(784.3px). 저장 높이와 내용 높이가 같아 이 규칙에는 중립이고,
//!   2쪽 이어받는 조각이 저장 높이로 늘지 않는지를 검사한다.
//! - 입력 3(반례): `samples/rowbreak-problem-pages.hwpx`(18쪽). 7쪽 5×2 표의 제26조 행 오른쪽 칸에
//!   위아래로 놓인 1×1 자식 표(저장 높이 12482 HU = 166.4px)가 행 안에서 나뉜다. 독립 기준
//!   `pdf/rowbreak-problem-pages-2024.pdf` 7쪽은 그 행을 1005.6px 에서 닫고(rhwp 1007.5px)
//!   왼쪽 칸 `제26조`를 그 행의 가운데에 둔다. 저장 높이를 모든 자식 표에 적용한 이전 head
//!   e52235e4e3 는 행을 1023.1px 까지 늘려 7쪽 Native 가 96.31 → 82.71 로 떨어졌다.
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(75544·nested
//! `--compat 2022`, rowbreak `--compat 2024`)의 `silhouette_raw_match_percent`(색상 경계 보정 전
//! 2px 실루엣), code head `7d940aca08`, 비교 base #8744 head `34384199de`.
//! - 75544(66쪽 = PDF 66쪽): 59쪽 Native 99.96 / fresh WASM 99.97(base Native 95.28),
//!   60쪽 100.00 / 100.00. 전쪽 최저 Native 98.13, fresh WASM 99.45. base 와 다른 쪽은 59쪽뿐이다.
//! - nested_row_cell_cuts(3쪽 = PDF 3쪽): 1쪽 93.59 / 93.06, 2쪽 99.14 / 99.38, 3쪽 98.41 / 98.42
//!   (base 와 같음). 전쪽 최저 Native 93.59, fresh WASM 93.06.
//! - rowbreak-problem-pages.hwpx(18쪽 = PDF 18쪽): 7쪽 Native 96.31 / fresh WASM 99.60(base 와 같음).
//!   전쪽 최저 Native 92.15, fresh WASM 93.00. 같은 원본의 HWP(`--compat 2024`, 18쪽)는 전쪽
//!   최저 Native 95.81, fresh WASM 97.62 로 base 와 같다.
//! - 증거: `mydocs/pr/assets/issue_8715/split_table_first_fragment/`(native·wasm·native_base TSV,
//!   75544 59·60쪽과 rowbreak HWPX 7쪽 review·overlay PNG).
//!
//! 검사는 글 앞 중첩 표 첫 조각 상자 높이와 저장 개체 높이의 일치(원본 속성에서 계산한 크기 관계,
//! 중첩 ±1px·바깥 ±2px), 중첩 표의 바깥 칸 포함(관계형), 이어받는 조각과 글 흐름 자식 표 첫 조각이
//! 저장 높이로 늘지 않음(원본 속성에서 계산한 크기 관계, 10px 이상 짧음)이다.
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

/// 쪽의 (rows×cols) 표 칸 안에 놓인 (1×1) 중첩 표 중 가장 아래 것의 (바깥 칸 상자, 중첩 표 상자).
fn lowest_nested_1x1(
    node: &RenderNode,
    rows: u16,
    cols: u16,
) -> Option<(BoundingBox, BoundingBox)> {
    let mut found: Option<(BoundingBox, BoundingBox)> = None;
    let mut visit = |cell: &BoundingBox, nested: &BoundingBox| {
        if found.as_ref().is_none_or(|(_, b)| nested.y > b.y) {
            found = Some((cell.clone(), nested.clone()));
        }
    };
    fn walk(
        node: &RenderNode,
        rows: u16,
        cols: u16,
        visit: &mut dyn FnMut(&BoundingBox, &BoundingBox),
    ) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.row_count == rows && table.col_count == cols {
                for cell in &node.children {
                    for child in &cell.children {
                        if let RenderNodeType::Table(t) = &child.node_type {
                            if t.row_count == 1 && t.col_count == 1 {
                                visit(&cell.bbox, &child.bbox);
                            }
                        }
                    }
                }
            }
        }
        for child in &node.children {
            walk(child, rows, cols, visit);
        }
    }
    walk(node, rows, cols, &mut visit);
    found
}

/// 반례: 글 흐름(위아래) 자식 표는 저장 높이가 내용보다 커도 첫 조각이 내용 높이에서 닫힌다.
/// 한컴 PDF(`pdf/rowbreak-problem-pages-2024.pdf`) 7쪽 제26조 행은 저장 높이로 늘지 않고,
/// 왼쪽 칸 `제26조`는 그 행의 가운데에 있다. 저장 높이를 적용하면 행이 약 21px 늘어난다.
#[test]
fn flow_child_first_fragment_keeps_content_height() {
    let core = load("samples/rowbreak-problem-pages.hwpx");
    let outer = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Table(t) if t.row_count == 5 && t.col_count == 2 => Some(t.as_ref()),
            _ => None,
        })
        .expect("5×2 바깥 표");
    let (_, child) = outer
        .cells
        .iter()
        .filter_map(|cell| {
            cell.paragraphs.iter().find_map(|para| {
                para.controls.iter().find_map(|control| match control {
                    Control::Table(t) if t.row_count == 1 && t.col_count == 1 => {
                        Some((cell, t.as_ref()))
                    }
                    _ => None,
                })
            })
        })
        .max_by_key(|(cell, _)| cell.row)
        .expect("칸 안 1×1 중첩 표");
    // 전제: 자식 표는 글 흐름(위아래)에 놓이고, 저장 높이가 7쪽 첫 조각보다 크다.
    assert!(!child.common.treat_as_char);
    assert!(matches!(
        child.common.text_wrap,
        rhwp::model::shape::TextWrap::TopAndBottom
    ));
    let child_frame = px(child.common.height);

    let tree = core.build_page_render_tree(6).expect("7쪽 렌더 트리");
    let (cell_box, nested_box) = lowest_nested_1x1(&tree.root, 5, 2).expect("7쪽 중첩 표");
    assert!(
        nested_box.height + 10.0 < child_frame,
        "7쪽 위아래 자식 첫 조각 {:.1}px 는 저장 높이 {child_frame:.1}px 로 늘지 않는다",
        nested_box.height
    );
    assert!(
        bottom(&nested_box) <= bottom(&cell_box) + 0.5,
        "7쪽 중첩 표({:.1})는 바깥 칸({:.1}) 안",
        bottom(&nested_box),
        bottom(&cell_box)
    );
}
