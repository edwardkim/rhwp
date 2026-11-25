//! 저장 줄 없는 칸 문단의 글 앞으로(InFrontOfText) 중첩 표는 글줄을 밀지 않지만, 칸 내용의
//! 끝은 그 표의 바깥 상자 끝까지 간다. 세로 가운데 정렬 칸은 흐름 줄 합과 표 끝 가운데 큰
//! 값을 내용 높이로 쓴다.
//!
//! 재현물 `samples/pk40991_overlay_nested/overlay_nested_center.{hwpx,hwp}`: 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 의 머리·구역 정의·표 XML 을 조합한
//! 합성본이다. 1열 3행 표의 가운데 칸(세로 가운데 정렬) 첫 문단에 문단 기준 글 앞으로 중첩 표
//! (1열 8행, 높이 10472 HU, 세로 오프셋 +75 HU, flowWithText=1)를 두고, 뒤에 빈 문단 하나를
//! 둔다. 가운데 칸의 선언 높이는 16000 HU 로 표보다 크다. 칸 문단에는 저장 줄이 없다.
//! HWP 는 같은 HWPX 를 rhwp 로 변환한 것이다. 생성: `make_pk40991_overlay_nested_fixture.py
//! --empty 1 --nested-rows 8 --row-h 1309 --vert 75 --outer-break NONE --mid-cell-h 16000`.
//!
//! 참고(웹한글기안기 인쇄, `pdf/pk40991_overlay_nested/overlay_nested_center-webhwp-print.pdf`):
//! 중첩 표가 칸 안에서 위아래 여백이 거의 같게 가운데에 놓이고, 8행이 모두 칸 안에 보인다.
//!
//! 수정 전 rhwp 는 정렬 높이를 글줄 두 개로 잡아 표를 정렬 여유의 절반만큼 아래로 내렸고,
//! 표 아래쪽 행이 칸 아래 경계에서 잘렸다.
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
        _ => {}
    }
    for child in &node.children {
        collect(child, depth, found);
    }
}

fn check(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "{name}: 쪽 수");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut found = Found::default();
    collect(&tree.root, 0, &mut found);
    let cell = found.row1_cell.expect("가운데 행 칸");
    let nested = found.nested_table.expect("중첩 표");
    // 표 바깥 상자(위아래 바깥 여백 포함)의 위·아래 여백이 거의 같다(표 세로 오프셋 1px 정도).
    let above = nested.y - cell.y;
    let below = cell.y + cell.height - (nested.y + nested.height);
    assert!(
        below >= 0.0 && (above - below).abs() <= 8.0,
        "{name}: 중첩 표 위 여백 {above:.1}px, 아래 여백 {below:.1}px"
    );
}

#[test]
fn overlay_nested_table_reach_sets_center_alignment_height_hwpx() {
    check("overlay_nested_center.hwpx");
}

#[test]
fn overlay_nested_table_reach_sets_center_alignment_height_hwp5() {
    check("overlay_nested_center.hwp");
}
