//! Issue #6544: 2단 미주 조판에서 **위험 휴리스틱이 저장 사다리를 뚫고** 단을 일찍 끊는다.
//! 밀려난 문단만큼 오른쪽 단이 내려가 단 아래끝을 넘는다.
//!
//! `late_compact_text_tail_overflow_risk` 는 저장 증거를 보지 않는 순수 띠다 — "단의 96%
//! 를 넘었고 이 문단을 넣으면 하단 20px 안으로 들어온다"면 넘긴다. 그것이
//! `advance_for_fit` 안에서 `compact_endnote_own_vpos_span_fits_for_flow`(= 문단의 저장
//! vpos 폭이 남은 공간에 든다)를 **뚫는 예외**로 등재돼 있어, 파일이 같은 단에 두라고
//! 적어 둔 문단까지 넘겼다.
//!
//! 저장 `LINE_SEG` 가 정답지다 (한컴 2022 저장, 미주 `s0:p155:ci0`):
//!
//! ```text
//! pi=657  vpos=435213  lh=900          '(ⅰ), (ⅱ)에 의하여 함수 f(x)는'
//! pi=658  vpos=436565  lh=900   Δ1352  (수식)
//! pi=659  vpos=437917  lh=900   Δ1352  'f(5)≠0 이므로'
//! pi=660  vpos=376577  lh=2070  되감김 ← 한글의 단 경계
//! ```
//!
//! `advance == text_height + line_spacing = 900 + 452 = 1352` 이라 657·658·659 는 **연속**
//! 이고, 단 경계 신호(vpos 되감김)는 660 에서만 난다.
//!
//! 완화는 **넣어도 가용 안에 남는** 경우로만 좁혔다. 예외를 통째로 빼면 `3-11월_실전_
//! 통합_2024-…` 가 12쪽부터 무너진다(18쪽 median 0.07 → 143.6pt, off-canvas 7 → 9).
//! 그래서 `pi=658` 만 왼쪽 단으로 돌아오고 `pi=659` 는 남는다 — 조판 누계가 그 지점에서
//! 렌더보다 21.8px 과대라(EN_ACC 가 pi=655·656 을 각각 +10.9px 크게 잡는다) `pi=659` 는
//! "안 들어간다"로 나오기 때문이다. 그 과대 계상은 별개 축이다.
//!
//! 실측 (`samples/3-09월_교육_통합_2023.hwp` 13쪽):
//!
//! ```text
//!            수정 전                        수정 후
//! 단 0  pi=637..657  잔여 33.30pt        pi=637..658  잔여 19.80pt
//! 단 1  pi=658 부터  초과 +21.15pt       pi=659 부터  초과 +7.65pt
//! ```
//!
//! 같은 수정이 같은 계열 문서의 다른 쪽들을 한글과 맞춘다 — `3-09월_교육_통합_2023` 20쪽,
//! `3-10월_교육_통합_2022` 11·13·16쪽(16쪽 최대 편차 741.7 → 26.5pt).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/3-09월_교육_통합_2023.hwp";
const PAGE_INDEX: u32 = 12; // 0-based — 13쪽

/// 완화 뒤 왼쪽 단 마지막 문단. 회귀 시에는 657 에서 끊겼다.
const EXPECTED_LAST_IN_COLUMN0: usize = 658;
/// 완화 뒤 오른쪽 단 첫 문단. 저장 사다리의 참 경계는 660 이다(잔여 축).
const EXPECTED_FIRST_IN_COLUMN1: usize = 659;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// (단 하단, 그 단의 글줄 [(y, h, pi)]) 목록.
fn columns(root: &RenderNode) -> Vec<(f64, Vec<(f64, f64, usize)>)> {
    let mut out: Vec<(f64, Vec<(f64, f64, usize)>)> = Vec::new();
    fn rec(node: &RenderNode, col: Option<usize>, out: &mut Vec<(f64, Vec<(f64, f64, usize)>)>) {
        let mut col = col;
        if matches!(node.node_type, RenderNodeType::Column(_)) {
            out.push((node.bbox.y + node.bbox.height, Vec::new()));
            col = Some(out.len() - 1);
        }
        if let (Some(idx), RenderNodeType::TextLine(line)) = (col, &node.node_type) {
            if let Some(pi) = line.para_index {
                out[idx].1.push((node.bbox.y, node.bbox.height, pi));
            }
        }
        for child in &node.children {
            rec(child, col, out);
        }
    }
    rec(root, None, &mut out);
    for (_, lines) in out.iter_mut() {
        lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    out
}

#[test]
fn column_break_follows_stored_ladder_not_risk_heuristic() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6544 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p13");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let cols = columns(&tree.root);
    assert!(
        cols.len() >= 2,
        "13쪽은 2단이어야 한다 — 실측 {}단",
        cols.len()
    );

    let (col0_bottom, col0) = &cols[0];
    let (col1_bottom, col1) = &cols[1];
    assert!(!col0.is_empty() && !col1.is_empty(), "빈 단이 있다");

    // ① 저장 사다리가 지목하는 지점에서 끊는다.
    let last0 = col0.last().expect("단 0 마지막 줄").2;
    let first1 = col1.first().expect("단 1 첫 줄").2;
    assert_eq!(
        (last0, first1),
        (EXPECTED_LAST_IN_COLUMN0, EXPECTED_FIRST_IN_COLUMN1),
        "단 경계가 저장 되감김 지점과 다르다 — 실측 단0 마지막 pi={last0}, 단1 첫 pi={first1}. \
         회귀 시 (657, 658) 로 한 문단 더 일찍 끊긴다"
    );

    // ② 왼쪽 단을 크게 비우지 않는다 (회귀 시 33.30pt).
    let col0_end = col0.last().map(|(y, h, _)| y + h).expect("단 0 끝");
    let col0_left_pt = (col0_bottom - col0_end) * 0.75;
    assert!(
        col0_left_pt < 26.0,
        "왼쪽 단을 {col0_left_pt:.2}pt 비운 채 끊었다 (회귀 시 33.30pt, 수정 후 19.80pt)"
    );

    // ③ 그 파생인 오른쪽 단 넘침도 없다 (회귀 시 −21.15pt).
    let col1_end = col1.last().map(|(y, h, _)| y + h).expect("단 1 끝");
    let col1_over_pt = (col1_end - col1_bottom) * 0.75;
    assert!(
        col1_over_pt < 14.0,
        "오른쪽 단이 단 아래끝을 {col1_over_pt:.2}pt 넘었다 (회귀 시 21.15pt, 수정 후 7.65pt)"
    );
}
