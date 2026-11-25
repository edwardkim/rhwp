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
//! 2쪽. 가운데 칸이 2쪽 위에서 이어지고 중첩 표의 남은 행(`NESTED 27`~`NESTED 35`)이 그
//! 조각에 그려진다. `TAIL ROW` 행은 그 아래에 놓인다.
//!
//! 바깥 표도 글 앞으로라 본문 흐름 자리를 차지하지 않는다. 한컴 2020 인쇄
//! (`_hancom_pk/overlay_nested_grow_split-2020.pdf`)는 표가 2쪽으로 나뉘어도 뒤 문단
//! `END` 를 1쪽 본문 흐름 자리(저장 vpos 8000 HU, 위끝 약 201px)에 표와 겹쳐 그린다.
//!
//! 수정 전 rhwp 는 칸 유닛이 빈 줄 하나뿐이라 칸 전체가 1쪽 조각에 든 것으로 보고, 2쪽의
//! 이어지는 칸 조각을 통째로 빠뜨렸다. `TAIL ROW` 가 2쪽 맨 위로 올라갔고, 중첩 표의 남은
//! 행은 어느 쪽에도 그려지지 않았다.
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
    end_line: Option<BoundingBox>,
    nested_table: Option<BoundingBox>,
    nested_rows: Vec<String>,
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
    if matches!(node.node_type, RenderNodeType::Table(_)) && depth == 2 {
        found.nested_table.get_or_insert_with(|| node.bbox.clone());
    }
    match &node.node_type {
        RenderNodeType::TableCell(cell)
            if depth == 1 && cell.row == 1 && found.row1_cell.is_none() =>
        {
            found.row1_cell = Some(node.bbox.clone());
        }
        RenderNodeType::TextLine(_) if run_text(node).contains("TAIL") => {
            found.tail_line.get_or_insert_with(|| node.bbox.clone());
        }
        RenderNodeType::TextLine(_) if run_text(node).trim() == "END" => {
            found.end_line.get_or_insert_with(|| node.bbox.clone());
        }
        RenderNodeType::TextLine(_) if run_text(node).contains("NESTED") => {
            found.nested_rows.push(run_text(node).trim().to_string());
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, depth, found);
    }
}

fn page(core: &DocumentCore, index: u32) -> Found {
    let tree = core.build_page_render_tree(index).unwrap();
    let mut found = Found::default();
    collect(&tree.root, 0, &mut found);
    found
}

fn check(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 쪽 수");
    let first = page(&core, 0);
    let found = page(&core, 1);
    // 중첩 표 남은 행이 2쪽에 이어 그려지고, 1쪽에 그린 행을 다시 그리지 않는다.
    assert!(
        found.nested_rows.iter().any(|row| row == "NESTED 35"),
        "{name}: 2쪽 중첩 행 {:?}",
        found.nested_rows
    );
    assert!(
        first.nested_rows.iter().any(|row| row == "NESTED 00"),
        "{name}: 1쪽 중첩 행 {:?}",
        first.nested_rows
    );
    let repeated: Vec<_> = found
        .nested_rows
        .iter()
        .filter(|row| first.nested_rows.contains(row))
        .collect();
    assert!(
        repeated.is_empty(),
        "{name}: 두 쪽에 겹친 중첩 행 {repeated:?}"
    );
    // 뒤 문단 END 는 표 끝을 기다리지 않고 1쪽 본문 흐름 자리에 남는다.
    assert!(found.end_line.is_none(), "{name}: END 가 2쪽으로 밀림");
    let end = first
        .end_line
        .unwrap_or_else(|| panic!("{name}: 1쪽에 END 줄이 있다"));
    assert!(
        (end.y - 201.2).abs() < 8.0,
        "{name}: END 위끝 {:.1}px (한컴 약 201px)",
        end.y
    );
    // 1쪽 칸 조각은 쪽 끝 가까이 채우고, 가운데 정렬 칸 안에서 잘린 중첩 표 상자를 위·아래
    // 같은 여백으로 놓는다. 한컴: 칸 위끝 142.5 → 중첩 표 위끝 150.5(8px), 칸 아래끝 1021.5.
    let first_cell = first.row1_cell.expect("1쪽 가운데 칸");
    let first_nested = first.nested_table.expect("1쪽 중첩 표");
    let top_gap = first_nested.y - first_cell.y;
    assert!(
        (6.5..=10.0).contains(&top_gap),
        "{name}: 1쪽 중첩 표 위 여백 {top_gap:.1}px (한컴 약 8px)"
    );
    let first_cell_bottom = first_cell.y + first_cell.height;
    assert!(
        (1016.0..=1025.0).contains(&first_cell_bottom),
        "{name}: 1쪽 칸 조각 아래끝 {first_cell_bottom:.1}px (한컴 약 1021.5px)"
    );
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
