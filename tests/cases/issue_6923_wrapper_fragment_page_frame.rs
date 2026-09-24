//! [#6923] 본문을 감싼 1×1 표의 **비끝 조각 상자**가 내용 끝에서 멈춘다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#7095` 가 "마지막이 아닌 조각의 상자는 내용이 아니라 쪽이 정한다"를 넣었지만, 적용
//! 조건이 **쪽 상단에서 시작하는 조각**뿐이었다. 쪽 중간에서 시작하는 조각을 뺀 근거는
//! `156645214` 19쪽 반례(상자를 늘리니 내용이 정본보다 8px 내려감)인데, 그 문서의 칸은
//! `valign=Center` 라 상자를 늘리면 내용이 따라 내려간다.
//!
//! 이 문서의 감싼 칸은 `valign=Top` 이고 내용이 칸 상단 + 안여백(141HU=1.9px)에 붙는다 —
//! 상자를 늘려도 내용은 움직이지 않는다. 그런데 같은 제외 규칙에 걸려 조각 상자가 내용
//! 끝에서 끊겼다.
//!
//! # 독립 기대값 — 한/글 정본
//!
//! `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf` 1쪽의
//! 가로선(96dpi 환산)이다. 감싼 표의 바닥만 어긋나 있었다.
//!
//! ```text
//!   정본   106.0 136.3 176.9 202.5 246.1 281.1 284.2 338.0 488.3 558.9 761.4 932.7 1021.9
//!   수정 전 106.0 136.5 178.5 202.6 247.9 281.4 284.5 338.4 488.7 559.4 762.2 933.7 1003.5
//!   수정 후 …(앞은 같음)…                                                          1022.9
//! ```
//!
//! 머리 표와 중첩 표의 가로선은 정본과 0.2~1.8px 안에서 맞는다. 바닥만 **−18.4px** 였고
//! 수정 뒤 **+1.0px** 로 같은 오차 범위에 든다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp";
/// `valign=Center`인 1×1 RowBreak 표의 쪽 중간 조각 제어군.
///
/// 한/글 2020 정본은 19쪽에서 이 표의 위·아래 괘선을 96dpi 기준
/// `217.3 .. 1013.5px`에 그린다. 쪽 프레임까지 상자를 늘리면 아래 괘선이
/// 약 1023px로 내려가므로, Top 앵커 보정의 비적용 경계를 이 실물 문서로 잠근다.
const CENTER_CONTROL: &str =
    "tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재.hwp";
/// 정본 1쪽 감싼 표 바닥 가로선(96dpi 환산).
const ORACLE_WRAPPER_BOTTOM_PX: f64 = 1021.9;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn center_control_core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CENTER_CONTROL);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 Center 제어 원본"))
        .expect("Center 제어 문서 로드")
}

/// 폭이 가장 넓은 최상위 표 = 본문을 감싼 1×1 표.
fn wrapper_table_bounds(root: &RenderNode) -> (f64, f64) {
    fn walk(node: &RenderNode, out: &mut Vec<(f64, f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) {
            out.push((node.bbox.width, node.bbox.y, node.bbox.y + node.bbox.height));
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut tables = Vec::new();
    walk(root, &mut tables);
    let widest = tables
        .iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).expect("유한값"))
        .copied()
        .expect("최상위 표");
    (widest.1, widest.2)
}

/// 비끝 조각의 상자 바닥은 내용 끝이 아니라 쪽이 정한다.
#[test]
fn wrapper_fragment_box_reaches_the_page_frame() {
    let core = core();
    assert_eq!(core.page_count(), 7, "정본과 같은 7쪽");
    let page1 = core.build_page_render_tree(0).expect("1쪽 렌더 트리").root;
    let (top, bottom) = wrapper_table_bounds(&page1);
    assert!(
        (top - 338.0).abs() < 2.0,
        "감싼 표 윗변은 정본 338.0 근처여야 한다 — got {top:.1}"
    );
    assert!(
        (bottom - ORACLE_WRAPPER_BOTTOM_PX).abs() < 2.5,
        "감싼 표 바닥은 정본 {ORACLE_WRAPPER_BOTTOM_PX} 근처여야 한다(수정 전 1003.5) — got {bottom:.1}"
    );
}

/// 상자를 늘려도 **내용은 움직이지 않는다** — Top 앵커라 늘림이 내용 중립이다.
#[test]
fn extending_the_box_does_not_move_the_top_anchored_content() {
    let core = core();
    let page1 = core.build_page_render_tree(0).expect("1쪽 렌더 트리").root;
    let (top, _) = wrapper_table_bounds(&page1);

    fn first_line_top(node: &RenderNode, table_top: f64, out: &mut Option<f64>) {
        if matches!(node.node_type, RenderNodeType::TextLine { .. }) && node.bbox.y > table_top {
            *out = Some(out.map_or(node.bbox.y, |y: f64| y.min(node.bbox.y)));
        }
        for child in &node.children {
            first_line_top(child, table_top, out);
        }
    }
    let mut first = None;
    first_line_top(&page1, top, &mut first);
    let first = first.expect("감싼 칸 첫 줄");
    assert!(
        first - top < 6.0,
        "감싼 칸 내용은 칸 상단 + 안여백(1.9px)에 붙어야 한다 — top={top:.1} first_line={first:.1}"
    );
}

/// 쪽 중간에서 시작하는 `Center` 칸의 조각 상자 — 정본은 **쪽 프레임까지** 그린다.
///
/// [#7063 레인②] 이 대조군의 기대값 `1013.5px` 는 한/글 정본이 아니라 **당시 rhwp 값**을
/// 굳힌 것이었다. `156645214...-2020.pdf` 19쪽의 이 표는 아래 가로 괘선을 그리지 않고
/// **세로 점선 괘선**만 그리는데, 그 점선을 좌·우 양변에서 모두 뜨면 이렇다.
///
/// ```text
///   좌변 x= 77.40  조각 1014개  217.04 .. 1023.68   5px 넘는 끊김 0곳
///   우변 x=711.94  조각 1014개  217.04 .. 1023.68   5px 넘는 끊김 0곳
///   clip 상자      x=77.40 y=217.04 w=634.54 h=806.64  → 괘선과 정확히 일치
///   쪽 척도(1122.5/1121.17) 보정 후                   217.27 .. 1024.75
/// ```
///
/// 곧 정본 상자는 `217.27 .. 1024.75` 이고, 종전 기대값 `1013.5` 는 그보다 **11.25px 짧다**.
/// 앞선 `issue_2004` 핀과 같은 유형의 기록 오류다(계보 예외로 rhwp 값을 굳힌 자리).
/// 기대값을 정본 괘선으로 교체한다.
#[test]
fn midpage_center_control_does_not_claim_the_page_frame() {
    let core = center_control_core();
    assert_eq!(core.page_count(), 20, "한/글 2020 정본과 같은 20쪽");
    let page19 = core
        .build_page_render_tree(18)
        .expect("19쪽 렌더 트리")
        .root;

    fn walk(node: &RenderNode, tables: &mut Vec<(f64, f64, f64)>) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(204) && table.row_count == 1 && table.col_count == 1 {
                tables.push((node.bbox.y, node.bbox.y + node.bbox.height, node.bbox.width));
            }
        }
        for child in &node.children {
            walk(child, tables);
        }
    }

    let mut tables = Vec::new();
    walk(&page19, &mut tables);
    let (top, bottom, width) = tables
        .into_iter()
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .expect("19쪽 본문 1×1 표");
    assert!(
        (top - 217.3).abs() < 1.0,
        "Center 제어 표의 위 괘선은 PDF 217.3px 근처여야 한다 — got {top:.1}"
    );
    assert!(
        (bottom - 1024.75).abs() < 2.0,
        "Center 칸 조각 상자는 정본 괘선(PDF 1024.75px)에서 끝나야 한다 — got {bottom:.1}"
    );
}
