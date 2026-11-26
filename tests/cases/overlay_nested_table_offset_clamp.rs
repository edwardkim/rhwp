//! 표 칸 안 문단 기준 글 앞으로(InFrontOfText) 중첩 표가 '쪽 영역 안으로 제한'(flowWithText=1)
//! 이면, 음수 세로 오프셋으로 호스트 문단 위로 올라가지 않는다. 칸 안 어울림 개체의 오프셋
//! 규칙과 같다.
//!
//! 재현물(`samples/pk40991_overlay_nested/`, 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 조합 합성본, HWP 는 같은 HWPX 를
//! rhwp 로 변환):
//! - `overlay_nested_split.{hwpx,hwp}`: 칸 첫 문단에 세로 오프셋 -10385 HU 의 24행 중첩 표,
//!   뒤에 빈 문단 60개. 칸이 1쪽에서 2쪽으로 나뉜다.
//! - `overlay_nested_anchor_offset.{hwpx,hwp}`: 빈 문단 30개 사이 열한째 문단(글 `ANCHOR`)에
//!   세로 오프셋 -6000 HU 의 6행 중첩 표.
//!
//! 참고(웹한글기안기 인쇄, `pdf/pk40991_overlay_nested/`): 첫 문단의 표는 칸 위끝에서 1쪽에
//! 그려지고, 열한째 문단의 표는 `ANCHOR` 줄 위끝에 놓인다.
//!
//! 수정 전 rhwp 는 오프셋을 그대로 써서 첫 문단의 표를 칸 위(쪽 위끝 밖, y<0)에 놓았고,
//! 칸이 나뉘면 조각 clip 에 잘려 표가 통째로 보이지 않았다. 열한째 문단의 표는 80px 위에 놓였다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/pk40991_overlay_nested")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

#[derive(Default)]
struct Found {
    row1_cell: Option<BoundingBox>,
    nested_table: Option<BoundingBox>,
    nested_text: bool,
    anchor_line: Option<BoundingBox>,
}

fn run_text(node: &RenderNode) -> String {
    let mut text = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        text.push_str(&run_text(child));
    }
    text
}

fn collect(node: &RenderNode, depth: usize, found: &mut Found) {
    let depth = match node.node_type {
        RenderNodeType::Table(_) => depth + 1,
        _ => depth,
    };
    match &node.node_type {
        RenderNodeType::Table(_) if depth == 2 && found.nested_table.is_none() => {
            found.nested_table = Some(node.bbox.clone());
        }
        RenderNodeType::TableCell(cell)
            if depth == 1 && cell.row == 1 && found.row1_cell.is_none() =>
        {
            found.row1_cell = Some(node.bbox.clone());
        }
        RenderNodeType::TextLine(_) if run_text(node).contains("ANCHOR") => {
            found.anchor_line.get_or_insert_with(|| node.bbox.clone());
        }
        RenderNodeType::TextRun(run) if run.text.contains("NESTED 00") => {
            found.nested_text = true;
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, depth, found);
    }
}

fn first_page(name: &str) -> Found {
    let core = load(name);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut found = Found::default();
    collect(&tree.root, 0, &mut found);
    found
}

fn check_split(name: &str) {
    let found = first_page(name);
    let cell = found.row1_cell.expect("1쪽 가운데 행 칸");
    let nested = found
        .nested_table
        .unwrap_or_else(|| panic!("{name}: 1쪽에 중첩 표가 그려진다"));
    assert!(
        found.nested_text,
        "{name}: 중첩 표 첫 행 글자가 1쪽에 보인다"
    );
    // 칸 위끝(안 여백·표 바깥 여백 몇 px)에서 시작한다. 칸 위로 올라가지 않는다.
    assert!(
        nested.y >= cell.y - 0.5 && nested.y <= cell.y + 12.0,
        "{name}: 중첩 표 위끝 {:.1}px, 칸 위끝 {:.1}px",
        nested.y,
        cell.y
    );
}

fn check_anchor(name: &str) {
    let found = first_page(name);
    let nested = found.nested_table.expect("중첩 표");
    let anchor = found.anchor_line.expect("ANCHOR 줄");
    // 표 바깥 여백(283 HU = 3.8px) 안에서 앵커 줄 위끝과 맞는다. 음수 오프셋(80px)만큼 올라가지 않는다.
    assert!(
        (nested.y - anchor.y).abs() <= 6.0,
        "{name}: 중첩 표 위끝 {:.1}px, ANCHOR 줄 위끝 {:.1}px",
        nested.y,
        anchor.y
    );
}

#[test]
fn flow_limited_overlay_table_stays_in_split_cell_hwpx() {
    check_split("overlay_nested_split.hwpx");
}

#[test]
fn flow_limited_overlay_table_stays_in_split_cell_hwp5() {
    check_split("overlay_nested_split.hwp");
}

#[test]
fn flow_limited_overlay_table_starts_at_anchor_line_hwpx() {
    check_anchor("overlay_nested_anchor_offset.hwpx");
}

#[test]
fn flow_limited_overlay_table_starts_at_anchor_line_hwp5() {
    check_anchor("overlay_nested_anchor_offset.hwp");
}
