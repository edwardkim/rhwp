//! 관계형·고정형 검사. 칸 조각의 포함 관계와 독립 인쇄에서 확인한 쪽별 행 소속을 검사한다.
//! 2026-11-25 21:00 KST 갱신. 직전 변경은 2026-11-24 15:30:01 +0900, 02e2796772.
//! 96 DPI, 한컴 2020 HWPX·2024 재저장 HWP 직접 인쇄 대비 2px raw. 원본과 아래 여백
//! 10mm 대조본은 각각 2쪽, Native·fresh WASM 전쪽 최저 100.00%였다. 두 인쇄본에서
//! 2쪽의 첫 중첩 행은 각각 27·26이다. 긴 용지 두 대조군도 각각 2쪽이며 두 경로 전쪽
//! 최저 100.00%다. 한컴 2020 인쇄에서 이월된 중첩 행은 각각 2개·0개다. 쪽별 결과는
//! `mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/`에 보존한다.
//!
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
    tail_cell: Option<BoundingBox>,
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
        RenderNodeType::TableCell(cell) if depth == 1 && cell.row == 2 => {
            found.tail_cell.get_or_insert_with(|| node.bbox.clone());
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

fn check(name: &str, continued_row: usize) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}");
    let first = page(&core, 0);
    let last = page(&core, 1);
    let expected: Vec<_> = (0..36).map(|row| format!("NESTED {row:02}")).collect();
    assert_eq!(
        first.nested_rows,
        expected[..continued_row],
        "{name}: 첫 조각"
    );
    assert_eq!(
        last.nested_rows,
        expected[continued_row..],
        "{name}: 다음 조각"
    );
    assert!(
        first.tail_line.is_none() && last.end_line.is_none(),
        "{name}"
    );
    let end = first.end_line.expect("첫 쪽에 남은 본문 END");
    let first_cell = first.row1_cell.expect("첫 쪽의 가운데 칸");
    assert!(
        end.y >= first_cell.y && end.y + end.height < first_cell.y + first_cell.height,
        "{name}: 떠 있는 표와 같은 쪽 본문 흐름에 END가 남는다"
    );
    let last_cell = last.row1_cell.expect("이어받은 가운데 칸");
    let tail = last.tail_line.expect("마지막 중첩 행 뒤의 TAIL ROW");
    assert!(tail.y >= last_cell.y + last_cell.height - 0.5, "{name}");
    for (cell, table) in [
        (first_cell, first.nested_table.expect("첫 조각의 중첩 표")),
        (last_cell, last.nested_table.expect("다음 조각의 중첩 표")),
    ] {
        assert!(
            table.y >= cell.y && table.y + table.height <= cell.y + cell.height + 0.5,
            "{name}: 중첩 행은 이를 소유한 바깥 칸 조각 안에 있다"
        );
    }
}

#[test]
fn overlay_nested_table_longer_than_cell_continues_on_next_page_hwpx() {
    check("overlay_nested_grow_split.hwpx", 27);
}

#[test]
fn overlay_nested_table_longer_than_cell_continues_on_next_page_hwp5() {
    check("overlay_nested_grow_split.hwp", 27);
}

#[test]
fn overlay_nested_table_hancom_resave_preserves_all_rows_and_trailing_content() {
    check("overlay_nested_grow_split-hancom.hwp", 27);
}

#[test]
fn overlay_nested_table_bottom_margin_moves_one_row_to_continuation() {
    check("grow-bottom-margin10mm.hwpx", 26);
}

#[test]
fn overlay_nested_table_last_row_content_fits_before_nominal_row_end() {
    check("grow-last-row-content-fit.hwpx", 34);
}

#[test]
fn overlay_nested_table_tail_alone_continues_after_the_last_owned_row() {
    let core = load("grow-tail-only-continuation.hwpx");
    assert_eq!(core.page_count(), 2);
    let first = page(&core, 0);
    let second = page(&core, 1);
    let expected: Vec<_> = (0..36).map(|row| format!("NESTED {row:02}")).collect();
    assert_eq!(first.nested_rows, expected);
    assert!(first.end_line.is_some() && first.tail_line.is_none());
    assert!(second.nested_rows.is_empty() && second.end_line.is_none());
    let tail = second.tail_line.expect("뒤 행의 글줄이 다음 쪽에 있다");
    let cell = second
        .tail_cell
        .expect("뒤 행의 글줄과 같은 쪽에 소유 칸이 있다");
    assert!(tail.y >= cell.y && tail.y + tail.height <= cell.y + cell.height);
}
