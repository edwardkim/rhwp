//! 관계형·고정형 검사. 빈 줄, 중첩 행과 뒤 문단의 소속 및 두 쪽의 연속성을 검사한다.
//! 2026-11-25 19:21 KST 갱신. 직전 변경은 2026-11-24 10:50:00 +0900, edc1ba957d.
//! 독립 한컴 2020 원본 및 한컴 2024 재저장 HWP의 96 DPI raw 2px 비교에서 두 쪽 모두
//! Native·fresh WASM 100.00%였다. 전쪽 TSV는
//! `mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/`에 보존한다.
//!
//! 표 칸 안의 글 앞으로(InFrontOfText) 중첩 표는 칸의 글 흐름을 밀지 않는다. 그 칸이 쪽 끝에서
//! 나뉠 때도 행 나눔 유닛에 중첩 표 높이를 더하지 않는다.
//!
//! 재현물 `samples/pk40991_overlay_nested/overlay_nested_split.{hwpx,hwp}`: 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 의 머리·구역 정의·표 XML 을 조합한
//! 합성본이다. 1열 3행 표의 가운데 칸 첫 문단에 문단 기준 글 앞으로 중첩 표(1열 24행,
//! 높이 57600 HU, 세로 오프셋 -10385 HU, flowWithText=1)를 두고, 그 뒤에 빈 문단 60개를 둔다.
//! 칸 문단에는 저장 줄이 없다. HWP 는 같은 HWPX 를 rhwp 로 변환한 것이다.
//!
//! 참고(웹한글기안기 인쇄): 2쪽. 중첩 표는 칸 위끝에 놓이고, 칸 높이는 빈 문단 줄만큼이며,
//! `TAIL ROW` 행과 `END` 문단은 2쪽에 놓인다.
//!
//! 수정 전 rhwp 는 칸이 쪽을 넘어 나뉠 때 중첩 표를 행 단위 흐름 유닛으로 쪼개 칸 높이에
//! 중첩 표 높이(768px)를 더했고, 3쪽이 되었다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/pk40991_overlay_nested")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn has_text(node: &RenderNode, text: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(text) {
            return true;
        }
    }
    node.children.iter().any(|child| has_text(child, text))
}

fn collect_row_text(node: &RenderNode, rows: &mut Vec<String>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.starts_with("NESTED ") {
            rows.push(run.text.trim().to_owned());
        }
    }
    for child in &node.children {
        collect_row_text(child, rows);
    }
}

fn check(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}");
    let first = core.build_page_render_tree(0).unwrap();
    let last = core.build_page_render_tree(1).unwrap();
    assert!(has_text(&last.root, "TAIL"), "{name}: 끝 행은 2쪽");
    assert!(has_text(&last.root, "END"), "{name}: 표 뒤 문단은 2쪽");
    assert!(
        !has_text(&first.root, "TAIL") && !has_text(&first.root, "END"),
        "{name}"
    );
    let mut first_rows = Vec::new();
    let mut last_rows = Vec::new();
    collect_row_text(&first.root, &mut first_rows);
    collect_row_text(&last.root, &mut last_rows);
    let expected: Vec<_> = (0..24).map(|row| format!("NESTED {row:02}")).collect();
    assert_eq!(
        first_rows, expected,
        "{name}: 중첩 행이 원래 순서대로 한 번씩 그려진다"
    );
    assert!(
        last_rows.is_empty(),
        "{name}: 둘째 쪽은 본문을 계속하고 중첩 행을 반복하지 않는다"
    );
}

#[test]
fn overlay_nested_table_does_not_grow_split_row_hwpx() {
    check("overlay_nested_split.hwpx");
}

#[test]
fn overlay_nested_table_does_not_grow_split_row_hwp5() {
    check("overlay_nested_split.hwp");
}

#[test]
fn overlay_nested_table_saved_anchor_line_preserves_blank_paragraphs_across_pages() {
    check("overlay_nested_split-hancom.hwp");
}
