#![cfg(not(target_arch = "wasm32"))]

//! [#7872] 칸의 저장 높이가 내용보다 작을 때 칸 안 **둘째** 글자처럼 취급 중첩 표의
//! 둘째 행이 통째로 사라지던 결함의 가드.
//!
//! ## 계약
//!
//! 바깥 1×1 표(`sz`/`cellSz` 900HU)의 칸에 글자처럼 취급 중첩 표 두 개(4×5 〈보기〉 상자,
//! 2×3 선택지)가 문단 하나씩에 들어 있다. 생성기가 적은 저장 줄은 둘째 문단부터
//! `vpos=0` 이고 줄 높이는 각 표의 **선언** 높이 + 바깥 여백뿐이다. 첫 표의 칸은 내용이
//! 선언보다 자라 실측 97.7px 인데, 측정이 저장 줄의 선언 흡수에 머물러 칸을 132px 로
//! 재면 페인트가 첫 표를 실측대로 그린 뒤 둘째 표에 24.8px 만 남기고 그 viewport 로
//! 행을 잘라(`inferred_viewport_split`) ④⑤ 행이 render tree 에서 빠졌다.
//!
//! 한/글 2020 Print PDF(`pdf/issue7872/nested-tac-row-drop-2020.pdf`, 1쪽)는 바깥 행을
//! 내용만큼 늘려 ①~⑤ 를 모두 그린다.
//!
//! ## fixture 출처
//!
//! 제보자 gist 의 `nested-tac-row-drop.hwpx`(sha256 `b9f285f0…`)에서 네 가지를 바꾼
//! 파생본이다. 바꾼 것 모두 결함 경로(저장 host 줄 vpos=0·선언 높이 흡수)와 무관하다.
//!
//! - 글꼴 `KoPubWorld돋움체 Medium` → `한컴돋움`: 한/글 변환 서버와 rhwp 가 같은 글꼴을
//!   공급해 글자 전진폭 차이를 없앤다(KoPubWorld 전진폭은 #7968 의 축).
//! - `hasMargin="0"` 칸의 보존 `cellMargin` 141HU → 0: HWPX 계약상 비활성 값이며 한/글은
//!   원본과 같은 쪽을 낸다(변형 오라클 동일). rhwp 는 저장 줄 빈 칸 행에 이 비활성 여백을
//!   더해 행이 15.8px 로 자라므로(한/글 12.0px) 이 축을 fixture 에서 분리한다.
//! - 본문 첫 빈 문단의 생성기 저장 줄 `1000/600`HU → 9pt·160% 의 `900/540`HU.
//! - 〈보기〉 칸에 `가.` 문단을 하나 더 두어 첫 표가 선언 높이보다 더 자라게 한다 —
//!   비활성 여백을 뺀 뒤에도 수정 전 devel 에서 ④⑤ 행 소실이 재현되게 하기 위해서다.
//!
//! 검사는 절대 좌표가 아니라 소속·순서·포함 관계다: 둘째 중첩 표가 두 행 여섯 칸을
//! 모두 갖고, ④·⑤ 글자가 그 둘째 행 칸 안에 있으며, 둘째 행이 첫 행 아래에 놓이고,
//! 중첩 표 전체가 바깥 칸 상자 안에 든다(잘린 viewport 가 아니다).
//!
//! 수정 전(devel `9415b920c`)에는 둘째 표의 칸이 셋뿐이라 첫 assert 에서 실패한다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

fn page0(input: &str) -> (DocumentCore, PageRenderTree) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#7872 fixture 읽기 {}: {error}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    (core, tree)
}

fn text_of(node: &RenderNode) -> String {
    let mut text = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        text.push_str(&text_of(child));
    }
    text
}

/// 바깥 표(본문 직속 첫 Table)의 유일한 칸.
fn outer_cell(tree: &PageRenderTree) -> &RenderNode {
    fn first_table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            return Some(node);
        }
        node.children.iter().find_map(first_table)
    }
    let outer = first_table(&tree.root).expect("바깥 1×1 표");
    let cells: Vec<&RenderNode> = outer
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), 1, "바깥 표는 칸 하나다");
    cells[0]
}

fn nested_tables(cell: &RenderNode) -> Vec<&RenderNode> {
    cell.children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .collect()
}

fn cells_by_row(table: &RenderNode) -> Vec<(u16, &RenderNode)> {
    table
        .children
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(cell) => Some((cell.row, n)),
            _ => None,
        })
        .collect()
}

fn assert_second_nested_rows(input: &str) -> PageRenderTree {
    let (core, tree) = page0(input);
    assert_eq!(core.page_count(), 1, "한/글 Print PDF 와 같은 1쪽");

    let outer = outer_cell(&tree);
    let nested = nested_tables(outer);
    assert_eq!(nested.len(), 2, "칸 안 중첩 표 두 개(〈보기〉·선택지)");
    let (first, second) = (nested[0], nested[1]);

    // 둘째 표: 두 행 × 세 칸이 모두 있고 ④·⑤ 는 둘째 행 칸의 글자다.
    let cells = cells_by_row(second);
    let row0: Vec<&RenderNode> = cells
        .iter()
        .filter(|(r, _)| *r == 0)
        .map(|(_, n)| *n)
        .collect();
    let row1: Vec<&RenderNode> = cells
        .iter()
        .filter(|(r, _)| *r == 1)
        .map(|(_, n)| *n)
        .collect();
    assert_eq!(
        (row0.len(), row1.len()),
        (3, 3),
        "둘째 중첩 표의 칸 수 — 수정 전에는 첫 행 셋뿐이었다"
    );
    let row1_text: String = row1.iter().map(|n| text_of(n)).collect();
    assert!(
        row1_text.contains('④') && row1_text.contains('⑤'),
        "둘째 행 글자가 ④·⑤ 를 담아야 한다: {row1_text:?}"
    );
    let row0_text: String = row0.iter().map(|n| text_of(n)).collect();
    assert!(
        row0_text.contains('①') && row0_text.contains('②') && row0_text.contains('③'),
        "첫 행 글자 ①②③ 보존: {row0_text:?}"
    );

    // 순서: 둘째 행은 첫 행 아래, 둘째 표는 첫 표 아래.
    let row0_bottom = row0
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(0.0, f64::max);
    let row1_top = row1.iter().map(|n| n.bbox.y).fold(f64::INFINITY, f64::min);
    assert!(
        row1_top + 0.5 >= row0_bottom,
        "둘째 행({row1_top})은 첫 행 바닥({row0_bottom}) 아래에 놓인다"
    );
    assert!(
        second.bbox.y + 0.5 >= first.bbox.y + first.bbox.height,
        "둘째 표({})는 첫 표 바닥({}) 아래에 놓인다",
        second.bbox.y,
        first.bbox.y + first.bbox.height
    );

    // 포함: 두 중첩 표의 모든 칸이 바깥 칸 상자 안에 든다 — 잘린 viewport 가 아니다.
    let outer_bottom = outer.bbox.y + outer.bbox.height;
    for (table_name, table) in [("첫 표", first), ("둘째 표", second)] {
        for (row, cell) in cells_by_row(table) {
            let bottom = cell.bbox.y + cell.bbox.height;
            assert!(
                cell.bbox.y + 0.5 >= outer.bbox.y && bottom <= outer_bottom + 0.5,
                "{table_name} 행 {row} 칸 [{:.1}, {bottom:.1}] 이 바깥 칸 [{:.1}, {outer_bottom:.1}] 안에 든다",
                cell.bbox.y,
                outer.bbox.y
            );
        }
    }
    tree
}

#[test]
fn second_nested_tac_table_keeps_its_second_row_when_outer_cell_is_short() {
    assert_second_nested_rows("samples/issue7872/nested-tac-row-drop.hwpx");
}

/// 공개 gist 원본은 글꼴·비활성 여백·저장 줄·본문을 바꾸지 않았다.
/// b9f285f0ce01773c9c64240392477a374e5c67bef00eabe0ace36af66ccae0ba.
/// 독립 Print: pdf/issue7872/nested-tac-row-drop-original-2020.pdf (1쪽),
/// bb8f105fb46aa5dafd81e717f7f1ea2a506defaaf7e23248024ff34812df5993.
/// 후보 f7ddf477e의96dpi print Native99.89402/WASM99.91495%와 직접 판독 후
/// 마지막 행의 실제 글자 원점/기준선을±2px로 보호한다. 관계 검사도 동일하게 실행한다.
#[test]
fn original_gist_preserves_second_row_ownership_and_independent_print_positions() {
    let tree =
        assert_second_nested_rows("tests/fixtures/issue7872/nested-tac-row-drop-original.hwpx");
    let second = nested_tables(outer_cell(&tree))[1];
    fn find_run(node: &RenderNode, marker: char) -> Vec<&RenderNode> {
        let mut out = Vec::new();
        if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(marker)) {
            out.push(node);
        }
        for child in &node.children {
            out.extend(find_run(child, marker));
        }
        out
    }
    for (marker, independent_x) in [('④', 58.88), ('⑤', 166.4)] {
        let runs = find_run(second, marker);
        assert_eq!(runs.len(), 1, "원본 선택지의 누락/중복 없음");
        let node = runs[0];
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert!((node.bbox.x - independent_x).abs() <= 2.0);
        assert!(
            (node.bbox.y + run.baseline - 255.2).abs() <= 2.0,
            "독립 Print 마지막 행 기준선255.2px"
        );
    }
}
