//! [#7160] 줄에 남은 폭보다 넓은 글자처럼 취급 표가 줄바꿈되지 않아 host 글자가 표 아래로 간다.
//!
//! ## 무엇이 문제였나
//!
//! `samples/task1768/distribution_doc.hwpx` 는 배포용(ViewText)이라 저장 `LINE_SEG` 가 없다.
//! 3쪽 `pi=47` 은 `(단위 : 천원)` 글자와 폭 629.2px 의 글자처럼 취급 표를 함께 든 문단인데,
//! 줄 나눔의 유일한 소유자인 프레임 채움이 그 표를 **세 관문에서 거절**해 45자 합성 줄이
//! 그대로 남았다. 그 결과 표가 자기 줄을 얻지 못하고, 표가 먼저 방출돼 글자가 표 **아래**로
//! 흩어졌다.
//!
//! ```text
//!   수정 전   표 480.3..725.0  →  "(단위 : " 730.9 · 빈 줄 749.9 · "천원)" 755.3
//!   한/글     "( 단위 : 천원 )" 520.2 (x=628.0)  →  표 (첫 행 글자 550.6)
//!   수정 후   "(단위 : 천원)" 478.4 (x=613.3)   →  표 504.5..749.2
//! ```
//!
//! ## 기대값의 독립 근거
//!
//! 한/글 2024 정본 `pdf/distribution_doc-2024.pdf` 3쪽: 오른쪽 끝에 붙은 `( 단위 : 천원 )`
//! 줄 **다음** 줄에 표가 본문 폭 안으로 온다. 말미 공백에 줄상자를 주지 않는 것은 저장
//! `PARA_LINE_SEG` 전수(말미 공백 문단 5,786건 중 공백만인 마지막 줄 0건)가 증언한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task1768/distribution_doc.hwpx";
/// 표를 든 host 문단(구역 0).
const HOST_PARA: usize = 47;
/// 한컴 정본에서 다음 쪽 첫 줄로 함께 넘어가는 소제목.
const SECTION_HEADING_PARA: usize = 35;
/// 소제목 바로 뒤의 첫 목록 항목.
const FIRST_LIST_PARA: usize = 36;
/// 이 문단의 글자 — 정본에서 표 **위** 줄에 있다.
const HOST_TEXT: &str = "단위";
/// 본문 오른쪽 끝(px). 표가 이 밖으로 나가면 안 된다.
const BODY_RIGHT: f64 = 725.7;
/// 한컴 2024 기준 PDF 3쪽의 해당 표 왼쪽 테두리.
const REFERENCE_TABLE_LEFT: f64 = 91.0;
/// 한컴 2024 정본 PDF 3쪽 `(단위 : 천원)`의 render-box 왼쪽.
const REFERENCE_UNIT_LABEL_LEFT: f64 = 628.0;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn column_nodes(root: &RenderNode) -> Vec<&RenderNode> {
    fn walk<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
        let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
        if in_body && matches!(node.node_type, RenderNodeType::Column(_)) {
            out.extend(node.children.iter());
            return;
        }
        for child in &node.children {
            walk(child, in_body, out);
        }
    }
    let mut out = Vec::new();
    walk(root, false, &mut out);
    out
}

fn run_text(node: &RenderNode) -> String {
    let mut text = String::new();
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(node, &mut text);
    text
}

fn find_text_run_x(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.x);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_text_run_x(child, needle))
}

/// 표가 있는 쪽의 (표 bbox, host 문단 줄들).
fn host_page(core: &DocumentCore) -> (u32, (f64, f64, f64), Vec<(f64, String)>) {
    for page in 0..core.page_count() as u32 {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let nodes = column_nodes(&tree.root);
        let Some(table) = nodes.iter().find_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(HOST_PARA) => {
                Some((n.bbox.y, n.bbox.x, n.bbox.width))
            }
            _ => None,
        }) else {
            continue;
        };
        let lines: Vec<(f64, String)> = nodes
            .iter()
            .filter(|n| {
                matches!(&n.node_type, RenderNodeType::TextLine(line)
                    if line.para_index == Some(HOST_PARA))
            })
            .map(|n| (n.bbox.y, run_text(n)))
            .collect();
        return (page, table, lines);
    }
    panic!("표 pi={HOST_PARA} 가 있는 쪽이 없다");
}

/// host 글자는 표 **위** 줄에 있다 — 표가 줄에 안 들어가면 표가 다음 줄로 간다.
#[test]
fn oversized_tac_table_moves_to_its_own_line_below_the_host_text() {
    let core = open();
    let (page, (table_y, table_x, table_w), lines) = host_page(&core);
    let text_line = lines
        .iter()
        .find(|(_, text)| text.contains(HOST_TEXT))
        .unwrap_or_else(|| {
            panic!(
                "{}쪽에 `{HOST_TEXT}` 줄이 없다 — 줄 목록 {:?}",
                page + 1,
                lines
            )
        });
    assert!(
        text_line.0 < table_y,
        "{}쪽 `{HOST_TEXT}` 줄이 {:.1}px 로 표({table_y:.1}px) **아래**에 있다 — 정본은 표 위 줄에 둔다",
        page + 1,
        text_line.0
    );
    assert!(
        table_x + table_w <= BODY_RIGHT + 1.0,
        "{}쪽 표 오른쪽 끝이 {:.1}px 로 본문 {BODY_RIGHT}px 를 넘는다",
        page + 1,
        table_x + table_w
    );
}

/// 말미 공백은 줄상자를 얻지 못한다 — host 문단의 보이는 줄은 하나뿐이다.
#[test]
fn trailing_whitespace_does_not_get_its_own_line() {
    let core = open();
    let (page, _, lines) = host_page(&core);
    assert_eq!(
        lines.len(),
        1,
        "{}쪽 host 문단 줄이 {}개다 — 말미 공백은 앞 줄이 흡수하므로 글자 줄 하나여야 한다: {:?}",
        page + 1,
        lines.len(),
        lines
    );
}

/// ViewText host의 선행 TAC 표는 기본 안쪽 여백을 inline 시작 폭에 보존한다.
/// 이 값이 빠지면 표와 `(단위 : 천원)`이 함께 약 8px 왼쪽으로 밀린다. 한컴 2024
/// 정본 PDF의 p3 괘선 왼쪽(x=91px)을 독립 기준으로 둔다.
#[test]
fn leading_viewtext_inline_table_keeps_its_visible_left_inset() {
    let core = open();
    let (page, (_, table_x, _), _) = host_page(&core);
    assert!(
        (table_x - REFERENCE_TABLE_LEFT).abs() <= 1.0,
        "{}쪽 표 왼쪽이 {table_x:.1}px 이다 — 한컴 정본 {REFERENCE_TABLE_LEFT:.1}px의 첫 줄 들여쓰기를 따라야 한다",
        page + 1
    );
}

#[test]
fn viewtext_table_unit_label_uses_the_table_right_inset() {
    let core = open();
    for page in 0..core.page_count() as u32 {
        let tree = core.build_page_render_tree(page).expect("render tree");
        for node in column_nodes(&tree.root) {
            if let Some(unit_x) = find_text_run_x(node, "(단위 : 천원)") {
                assert!(
                    (unit_x - REFERENCE_UNIT_LABEL_LEFT).abs() <= 1.0,
                    "{}쪽 단위 문구가 {unit_x:.1}px 이다 — 한컴 정본 {REFERENCE_UNIT_LABEL_LEFT:.1}px 표 우측 inset에 있어야 한다",
                    page + 1
                );
                return;
            }
        }
    }
    panic!("`(단위 : 천원)` 문구가 없다");
}

/// 반례 — 줄에 들어가는 글자처럼 취급 표는 같은 줄에 남는다(같은 문서 1쪽 제목 표).
#[test]
fn tac_table_that_fits_keeps_its_line() {
    let core = open();
    let tree = core.build_page_render_tree(0).expect("render tree");
    let nodes = column_nodes(&tree.root);
    let tables: Vec<(usize, f64, f64)> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(t) => Some((t.para_index?, n.bbox.x, n.bbox.width)),
            _ => None,
        })
        .collect();
    assert!(!tables.is_empty(), "1쪽에 표가 있어야 한다");
    for (para_index, x, width) in tables {
        assert!(
            x + width <= BODY_RIGHT + 1.0,
            "1쪽 표 pi={para_index} 오른쪽 끝 {:.1}px 가 본문 {BODY_RIGHT}px 를 넘는다",
            x + width
        );
    }
}

/// 한/글 정본은 `직무활동 범위` 제목을 p2 바닥에 잘라 두지 않고, 다음 목록과 함께
/// p3으로 보낸다. 저장 `LINE_SEG`가 없는 제목은 마지막 잉크 높이만 fit에 쓰면
/// 뒤 줄간격 전진을 잃어 layout에서 본문 하단을 넘긴다.
#[test]
fn no_lineseg_heading_moves_with_its_following_list_to_the_next_page() {
    let core = open();
    assert_eq!(
        core.page_count(),
        3,
        "원본·한/글 정본과 같은 3쪽이어야 한다"
    );
    let mut heading_page = None;
    let mut first_list_page = None;
    for page in 0..core.page_count() as u32 {
        let tree = core.build_page_render_tree(page).expect("render tree");
        for node in column_nodes(&tree.root) {
            match &node.node_type {
                RenderNodeType::TextLine(line) if line.para_index == Some(SECTION_HEADING_PARA) => {
                    heading_page = Some(page);
                }
                RenderNodeType::TextLine(line) if line.para_index == Some(FIRST_LIST_PARA) => {
                    first_list_page = Some(page);
                }
                _ => {}
            }
        }
    }
    assert_eq!(
        heading_page,
        first_list_page,
        "소제목 pi={SECTION_HEADING_PARA:?}와 첫 목록 pi={FIRST_LIST_PARA:?}가 같은 쪽이어야 한다: heading={heading_page:?}, list={first_list_page:?}"
    );
    assert_eq!(
        heading_page,
        Some(2),
        "한/글 기준 p3(0-based 2)에서 소제목과 목록이 시작해야 한다"
    );
}
