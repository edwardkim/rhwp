//! [#6925] 감싼 칸 안의 **빈 문단이 저장 슬롯만큼 자리를 차지한다** — 접으면 뒤 문단이
//! 문단마다 12~17px 씩 위로 당겨져 한 쪽 안에서 67.3px 이 쌓인다.
//!
//! `samples/issue6924/148751598-briefing.hwp` 1쪽 본문은 전부 1행×1열
//! 감싼 표 안에 있고, 그 칸의 빈 문단은 저장 사다리가 자기 슬롯(`lh + ls`)만큼 전진한다.
//!
//! ```text
//!   p[2] 빈 lh=1000 ls=492 vpos=3120   슬롯 1492   p[3] vpos=4612 = 3120+1492  ✔
//!   p[4] 빈 lh= 600 ls=296 vpos=6848   슬롯  896   p[5] vpos=7744 = 6848+ 896  ✔
//!   p[8] 빈 lh= 800 ls=392 vpos=20412  슬롯 1192   p[9] vpos=21604 = 20412+1192 ✔ (표 host)
//! ```
//!
//! 수정 전에는 "빈 줄 높이가 다음 줄의 75% 이상" 이라는 대리 지표만 봐서 p[2]·p[4]·p[8] 이
//! 접혔다(다음 줄이 1500·1500·28421 로 크다). 접힌 만큼 뒤가 당겨져 표가 정본보다
//! 67.3px 위에 그려졌다.
//!
//! 기대값은 구현과 무관하게 **한/글 2020 PDF**(`-2020.pdf`, MCP engine 2020)에서 읽은
//! 같은 쪽 같은 줄의 y 다. 잔여 오차 ~4px 은 이슈 본문이 따로 적은 머리 표 오프셋
//! (−3.0 ~ −4.5px) 축이며 이 검사는 그 범위를 허용한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/issue6924/148751598-briefing.hwp";

/// 한/글 2020 PDF 1쪽 실측 y (px, 96dpi 환산).
const ORACLE: &[(&str, f64)] = &[
    ("ㅇ 주요 내용은", 391.3),
    ("< 금주 지식경제부", 535.2),
    ("브리핑", 580.3),
];

/// 수정 전 관측값 — 이 검사가 무엇을 막는지 남긴다.
const BEFORE_FIX: &[(&str, f64)] = &[
    ("ㅇ 주요 내용은", 344.8),
    ("< 금주 지식경제부", 487.9),
    ("브리핑", 517.5),
];

fn first_run_y(node: &RenderNode, prefix: &str, out: &mut Option<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.display_or_text();
        if text.trim_start().starts_with(prefix) && out.is_none() {
            *out = Some(node.bbox.y);
        }
    }
    for child in &node.children {
        first_run_y(child, prefix, out);
    }
}

#[test]
fn empty_cell_paragraphs_keep_their_stored_slot_so_the_page_does_not_drift() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {FIXTURE}: {e:?}"));
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree 1쪽: {e:?}"));

    let mut failures = Vec::new();
    for ((prefix, oracle_y), (_, before)) in ORACLE.iter().zip(BEFORE_FIX.iter()) {
        let mut found = None;
        first_run_y(&tree.root, prefix, &mut found);
        let y = found.unwrap_or_else(|| panic!("1쪽에서 '{prefix}' 줄을 찾지 못했다"));
        let delta = y - oracle_y;
        if delta.abs() > 6.0 {
            failures.push(format!(
                "'{prefix}' y={y:.1} (정본 {oracle_y:.1}, 차 {delta:+.1}px, 수정 전 {before:.1})"
            ));
        }
        // 정본보다 **아래**로 내려가는 것은 이 축의 반대 방향이다 — 함께 막는다.
        assert!(
            delta < 6.0,
            "'{prefix}' 가 정본보다 아래에 놓였다: y={y:.1} 정본={oracle_y:.1}"
        );
    }
    assert!(
        failures.is_empty(),
        "빈 문단의 저장 슬롯이 접혀 쪽 안에서 드리프트가 누적됐다:\n  {}",
        failures.join("\n  ")
    );
}
