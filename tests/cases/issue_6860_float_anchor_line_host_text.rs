//! [Issue #6860] 자리차지 표의 **앵커 줄**을 첫 줄로 봐서, 호스트 문단의 글자 두 줄이
//! 마지막 표 조각 뒤 본문 밖으로 밀리던 결함의 가드.
//!
//! ## 한글 자신의 저장 기하가 정답지다
//!
//! 원본 `3067979_[붙임 2] 도로안전시설 … 조명시설편.hwpx` 문단 1523 의
//! `hp:linesegarray` 는 호스트 두 줄을 표보다 **위**에 적어 두었다.
//!
//! ```text
//!   문단 1522 (직전)   vertpos = 29132
//!   문단 1523 줄0     vertpos = 30732   vertsize = 1000   textpos = 0
//!   문단 1523 줄1     vertpos = 32332   vertsize = 1000   textpos = 56
//!   ── 표 밴드 (33332 … 74427) ──
//!   문단 1524 (다음)   vertpos = 74427
//! ```
//!
//! ## `v_off` 의 기준 줄이 틀렸다
//!
//! `stored_host_lines_precede_float` 는 표 세로 오프셋(`vertOffset=1256`)을 문단 **첫
//! 줄** 기준으로 쟀다. 한글은 개체의 **제어 문자가 실린 줄**(앵커 줄) 기준으로 잰다.
//! 이 문단의 표 제어 문자는 텍스트 맨 끝(`… 다음 표와 같다.` 뒤)이라 앵커는 줄1 이다.
//!
//! ```text
//!   앵커 줄 기준(정답)  표 상단 32332 + 1256 = 33588  ≥  줄1 끝 33332   → 두 줄 다 표 위 ✓
//!   첫 줄 기준(결함)    표 상단 30732 + 1256 = 31988  <   줄1 시작 32332 → 줄1 을 표 아래로 오판 ✗
//! ```
//!
//! 31988 은 줄0 이 끝난(31732) 뒤 줄1 이 시작(32332)하기도 전인 빈 자리다 — 저장 기하가
//! 스스로 그 자리에 표가 없음을 증언한다. 게이트가 줄1 에서 실패하면
//! `defer_visible_rowbreak_host_text` 지연이 켜지고 호스트 두 줄이 뒤로 밀린다.
//!
//! ## 단일변수 검정 (수정 전 바이너리)
//!
//! 표의 `vertOffset` 만 `1256` → `3000` 으로 바꾸고 같은 바이너리로 다시 그렸다.
//!
//! ```text
//!   [1256]  2쪽 y=1064.2 / 1085.5 / 1106.9   (표 뒤, 본문 1084.7 밖, 3줄)
//!   [3000]  1쪽 y= 517.3 /  538.7            (표 위, 본문 안, 2줄)  ← 저장 기하와 일치
//! ```
//!
//! ## 정본
//!
//! `pdf/3067979-road-lighting-photometric-appendix-2020.pdf` (engine 2020 — 저장 제품이
//! `hancom-office-2020` 11.0.0.2129 이라 규약 §3.5.1 의 2020 버킷). 1쪽 y=179.8·201.2
//! 두 줄 뒤 표(y≈247.1), 2쪽은 표 이어짐만, 총 2쪽이다.
//!
//! 절단본이라 절대 y 는 정본과 다르다 — rhwp 는 저장 `vertpos` 를 따르고 한글은 절단된
//! 문서를 새로 조판한다. 잠그는 것은 **차례와 쪽 귀속**이다.
//!
//! ## 통제군
//!
//! `#5584`(00072) 는 호스트가 **한 줄**이라 앵커 줄 == 첫 줄이고, 수정 전후로 계약이
//! 같아야 한다. 그 fixture 를 함께 건다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6860/3067979-road-lighting-photometric-appendix.hwpx";
/// `#5584` 호스트 한 줄 통제군 — 앵커 줄 == 첫 줄이라 계약이 그대로여야 한다.
const CONTROL_SAMPLE: &str = "samples/issue5584/float_host_title_above_table.hwpx";

/// 호스트 문단 첫 줄의 글자. 이 fixture 에서 유일하다.
const HOST_LINE_1: &str = "도로조명계산";
/// 호스트 문단 둘째 줄의 글자.
const HOST_LINE_2: &str = "국제적으로";

fn read(rel: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()))
}

fn document(rel: &str) -> HwpDocument {
    HwpDocument::from_bytes(&read(rel)).expect("문서 로드")
}

fn tree(document: &HwpDocument, page: u32) -> RenderNode {
    document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"))
        .root
}

/// `Column` 직계 표의 `(y0, y1)` — 칸 안 중첩 표는 세지 않는다.
fn column_table_tops(node: &RenderNode, in_column: bool, out: &mut Vec<(f64, f64)>) {
    if in_column {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height));
            return;
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        column_table_tops(child, in_column, out);
    }
}

/// 줄 안 `TextRun` 을 이어 붙인 글자. 글자 모양이 갈리면 한 줄이 여러 run 으로 쪼개지므로
/// **줄 단위**로 봐야 needle 이 run 경계에서 끊기지 않는다.
fn line_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        line_text(child, out);
    }
}

/// 칸 **밖** `TextLine` 중 `needle` 을 담은 것의 `(y, y + 높이)` 를 모은다.
fn body_line_bands(node: &RenderNode, needle: &str, in_cell: bool, out: &mut Vec<(f64, f64)>) {
    let in_cell = in_cell || matches!(node.node_type, RenderNodeType::TableCell(_));
    if !in_cell {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut text = String::new();
            line_text(node, &mut text);
            if text.contains(needle) {
                out.push((node.bbox.y, node.bbox.y + node.bbox.height));
            }
            return;
        }
    }
    for child in &node.children {
        body_line_bands(child, needle, in_cell, out);
    }
}

fn find_lines(document: &HwpDocument, page: u32, needle: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    body_line_bands(&tree(document, page), needle, false, &mut out);
    out
}

/// `Column` 직계 흐름 항목(표·줄)의 세로 범위와 종류.
fn column_flow_boxes(node: &RenderNode, in_column: bool, out: &mut Vec<(f64, f64, &'static str)>) {
    if in_column {
        let kind = match node.node_type {
            RenderNodeType::Table(_) => Some("표"),
            RenderNodeType::TextLine(_) => Some("줄"),
            _ => None,
        };
        if let Some(kind) = kind {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height, kind));
            return;
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        column_flow_boxes(child, in_column, out);
    }
}

fn body_bounds(node: &RenderNode) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children.iter().find_map(body_bounds)
}

/// 어느 쪽에서도 흐름 항목이 본문을 넘지 않는다.
///
/// 수정 전 결함판은 호스트 두 줄이 2쪽 본문을 14.1px·35.5px 넘어 꼬리말과 겹쳤다.
/// 개체 원점만 앵커 줄로 내리고 조판 예산을 그대로 두면 이번엔 1쪽 표 조각이
/// 10.3px 넘는다 — 두 기계가 같은 기준점을 써야 이 계약이 성립한다.
///
/// ⚠ 쪽 **수**는 걸지 않는다. 이 fixture 는 절단본이라 rhwp(저장 `vertpos` 추종)와
/// 한글(절단본 재조판)의 절대 좌표가 다르고, 실제로 rhwp 문단 블록은 저장 사다리보다
/// 31.9px 아래에 있다(줄0 저장 485.4 vs rhwp 517.3 — 이 fixture 와 무관한 선행 드리프트).
/// 그만큼 1쪽 표 예산이 줄어 꼬리 한 행이 3쪽으로 넘어가므로 rhwp 3쪽 · 정본 2쪽이다.
/// 원본 문서(104쪽)의 쪽수는 이 수정 전후로 바뀌지 않는다.
#[test]
fn no_flow_item_overflows_the_body() {
    let document = document(SAMPLE);
    for page in 0..document.page_count() {
        let root = tree(&document, page);
        let (body_top, body_bottom) = body_bounds(&root).expect("Body");
        let mut boxes = Vec::new();
        column_flow_boxes(&root, false, &mut boxes);
        for (top, bottom, kind) in boxes {
            assert!(
                top >= body_top - 0.5 && bottom <= body_bottom + 0.5,
                "쪽 {page} 의 {kind} 이 본문({body_top:.1}..{body_bottom:.1}) 밖이다 —                  {top:.1}..{bottom:.1}"
            );
        }
    }
}

/// 호스트 두 줄은 표가 **시작하는 쪽**에, 표보다 **위**에 있어야 한다.
#[test]
fn host_lines_render_above_the_float_table() {
    let document = document(SAMPLE);

    let mut tables = Vec::new();
    column_table_tops(&tree(&document, 0), false, &mut tables);
    assert_eq!(
        tables.len(),
        1,
        "1쪽에는 자리차지 표 하나가 시작해야 한다 — got {tables:?}"
    );
    let table_top = tables[0].0;

    for needle in [HOST_LINE_1, HOST_LINE_2] {
        let lines = find_lines(&document, 0, needle);
        assert_eq!(
            lines.len(),
            1,
            "호스트 줄 {needle:?} 이 1쪽에 정확히 한 번 있어야 한다 — \
             회귀 시 0개(2쪽으로 밀림). got {lines:?}"
        );
        let (top, bottom) = lines[0];
        assert!(
            bottom <= table_top + 0.5,
            "호스트 줄 {needle:?} 은 표 위에서 끝나야 한다 — \
             줄 {top:.1}..{bottom:.1}, 표 상단 {table_top:.1}"
        );
    }
}

/// 마지막 표 조각 뒤(2쪽)에는 호스트 글자가 남지 않는다 — 지연 계약이 켜지면 여기로 온다.
#[test]
fn host_lines_are_not_deferred_behind_the_last_fragment() {
    let document = document(SAMPLE);
    for needle in [HOST_LINE_1, HOST_LINE_2] {
        let lines = find_lines(&document, 1, needle);
        assert!(
            lines.is_empty(),
            "2쪽(마지막 표 조각)에 호스트 줄 {needle:?} 이 남았다 — \
             회귀 시 y=1064.2 / 1085.5 로 그려진다. got {lines:?}"
        );
    }
}

/// 호스트 줄은 본문 안에 있어야 한다 — 결함판은 본문을 14.1px·35.5px 넘겨 꼬리말과 겹쳤다.
#[test]
fn host_lines_stay_inside_the_body() {
    let document = document(SAMPLE);
    let root = tree(&document, 0);
    let (body_top, body_bottom) = body_bounds(&root).expect("1쪽 Body");

    for needle in [HOST_LINE_1, HOST_LINE_2] {
        let lines = find_lines(&document, 0, needle);
        let (top, bottom) = lines
            .first()
            .copied()
            .unwrap_or_else(|| panic!("호스트 줄 {needle:?} 이 1쪽에 없다"));
        assert!(
            top >= body_top - 0.5 && bottom <= body_bottom + 0.5,
            "호스트 줄 {needle:?} 이 본문({body_top:.1}..{body_bottom:.1}) 밖이다 — \
             {top:.1}..{bottom:.1}"
        );
    }
}

/// `#5584` 통제군 — 호스트가 한 줄이면 앵커 줄 == 첫 줄이라 계약이 그대로다.
#[test]
fn single_line_host_control_still_renders_above_the_table() {
    let document = document(CONTROL_SAMPLE);

    let mut tables = Vec::new();
    column_table_tops(&tree(&document, 0), false, &mut tables);
    assert!(
        !tables.is_empty(),
        "#5584 통제군 1쪽에 자리차지 표가 있어야 한다"
    );
    let table_top = tables.iter().map(|t| t.0).fold(f64::INFINITY, f64::min);

    let lines = find_lines(&document, 0, "취업취약계층");
    assert_eq!(
        lines.len(),
        1,
        "#5584 제목이 1쪽에 있어야 한다 — got {lines:?}"
    );
    assert!(
        lines[0].1 <= table_top + 0.5,
        "#5584 제목이 표 위에서 끝나야 한다 — 줄 {:.1}..{:.1}, 표 상단 {table_top:.1}",
        lines[0].0,
        lines[0].1
    );
}
