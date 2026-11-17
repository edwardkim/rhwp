//! 저장 줄 없는 칸 문단의 글 앞으로(InFrontOfText) 중첩 표가 칸 선언 높이보다 훨씬 길면, 칸은
//! 표 끝까지 이어진다. 그 칸이 쪽에서 나뉘면 남은 몫이 다음 쪽 조각으로 간다.
//!
//! 재현물 `samples/pk40991_overlay_nested/overlay_nested_grow_split.{hwpx,hwp}`: 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 의 머리·구역 정의·표 XML 을 조합한
//! 합성본이다. 첫 문단 뒤 자기 문단에 놓인 글 앞으로 1열 3행 표의 가운데 칸(선언 높이
//! 72000 HU)에 문단 하나만 두고, 그 문단에 문단 기준 글 앞으로 중첩 표(1열 36행, 높이 86400 HU,
//! 세로 오프셋 0, flowWithText=1)를 둔다. 칸 문단에는 저장 줄이 없다. HWP 는 같은 HWPX 를
//! rhwp 로 변환한 것이다. 생성: `make_pk40991_overlay_nested_fixture.py --empty 0
//! --nested-rows 36 --vert 0 --mid-cell-h 72000 --outer-wrap IN_FRONT_OF_TEXT --outer-own-para 3`.
//!
//! 참고(웹한글기안기 인쇄, `pdf/pk40991_overlay_nested/overlay_nested_grow_split-webhwp-print.pdf`):
//! 2쪽. 가운데 칸이 2쪽 위에서 이어지고(중첩 표 9행 몫), `TAIL ROW` 행은 그 아래에 놓인다.
//!
//! 수정 전 rhwp 는 칸 유닛이 빈 줄 하나뿐이라 칸 전체가 1쪽 조각에 든 것으로 보고, 2쪽의
//! 이어지는 칸 조각을 통째로 빠뜨렸다. `TAIL ROW` 가 2쪽 맨 위로 올라갔다.
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
    tail_line: Option<BoundingBox>,
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
        RenderNodeType::TableCell(cell)
            if depth == 1 && cell.row == 1 && found.row1_cell.is_none() =>
        {
            found.row1_cell = Some(node.bbox.clone());
        }
        RenderNodeType::TextLine(_) if run_text(node).contains("TAIL") => {
            found.tail_line.get_or_insert_with(|| node.bbox.clone());
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, depth, found);
    }
}

fn check(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 쪽 수");
    let tree = core.build_page_render_tree(1).unwrap();
    let mut found = Found::default();
    collect(&tree.root, 0, &mut found);
    let cell = found
        .row1_cell
        .unwrap_or_else(|| panic!("{name}: 2쪽에 가운데 칸이 이어진다"));
    let tail = found.tail_line.expect("TAIL ROW 줄");
    // 중첩 표 남은 몫(9행 안팎, 약 200px)이 2쪽 칸 조각 높이로 남는다.
    assert!(
        cell.height > 150.0,
        "{name}: 2쪽 가운데 칸 조각 높이 {:.1}px",
        cell.height
    );
    assert!(
        tail.y >= cell.y + cell.height - 0.5,
        "{name}: TAIL ROW 위끝 {:.1}px, 칸 조각 아래끝 {:.1}px",
        tail.y,
        cell.y + cell.height
    );
}

#[test]
fn overlay_nested_table_longer_than_cell_continues_on_next_page_hwpx() {
    check("overlay_nested_grow_split.hwpx");
}

#[test]
fn overlay_nested_table_longer_than_cell_continues_on_next_page_hwp5() {
    check("overlay_nested_grow_split.hwp");
}
