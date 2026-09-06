//! [Issue #6793] 표지 TAC 표 다음 문단 `< 차 례 >` 의 **쪽 귀속**이 틀렸다.
//!
//! `1611000-201000141` 실측:
//!
//! ```text
//!   pi=0  ls[0] vpos=0     lh=61600 th=1000     ← 앵커 줄
//!         ls[1] vpos=1600  lh=61600 gap=36960   ← 표 줄 + 꼬리
//!         저장 꼬리 끝 = (1600 + 61600 + 36960) / 75 = 1335.5px   > 예산 876.9
//!   pi=1  ls[0] vpos=0                          ← 새 쪽 상단
//! ```
//!
//! 흐름 계상이 **842.7 에서 멈춰** `pi=1` 이 1쪽 꼬리에 붙었고, 렌더가 그 꼬리
//! 간격(492.8px)을 더해 `< 차 례 >` 를 표 바닥 965.1 에서 494.6px 아래인 1459.7 에
//! 그렸다 — 용지 밖 **363.9px**.
//!
//! ⚠⚠ **초판은 이것을 렌더에서 고쳤다** — 꼬리 간격을 흐름에 안 태우는 방식.
//! 글은 보이게 됐지만 `< 차 례 >` 가 **1쪽 표지 맨 아래**에 남아 쪽 귀속은 여전히
//! 틀렸고(한/글은 2쪽 첫 줄), 시험이 그 틀린 배치를 성공 조건으로 고정했다
//! (PR #6794 지적). 렌더 변경을 되돌리고 **조판(typeset)의 쪽 귀속**을 고쳤다.
//!
//! ⭐ 판정은 저장 사다리 둘이 함께 준다 — **크기 문턱이 없다.**
//!   1. 이 문단의 첫 **비합성** 저장 줄이 `vpos == 0`.
//!   2. 앞 문단의 마지막 비합성 저장 줄이 `vpos + lh + gap` 으로 **이 쪽 예산을
//!      넘는다** — 그 꼬리가 쪽-끝 채움이라는 뜻이다.
//!
//! ⚠ 2 가 없으면 안 된다. `vpos == 0` 은 새 쪽 상단인 동시에 **"앵커 없음" 센티널**
//! 이기도 하다 — `#6753` 이 남긴 함정이다(조각 시작 `vpos == 0` 을 무조건 쪽 경계로
//! 읽은 선행 시도가 242쪽을 243쪽으로 늘려 기각됐다). 저장 사다리가 권위인
//! **네이티브 HWP5** 조판에 한정한다.
//!
//! 초판의 `col_area.height * 0.25` 경험적 문턱은 **사라졌다** — 조판이 쪽을 닫으면
//! 렌더는 손댈 것이 없다.
//!
//! 결과: `< 차 례 >` 가 **2쪽 첫 줄**(y=132.3 = 본문 상단), 1쪽 용지 밖 1 → **0**,
//! 쪽수 12 유지.
//!
//! ⚠ 남는 text-overlap 16건은 **수정 전부터** 있던 2.07px 미세 겹침으로 이 축과
//! 무관하다.
//!
//! ⚠ fixture 는 `lastSavedWith.version = 6.7.8.1045`, `product = null` 이라 저장소
//! 정책상 기준 엔진은 **2020** 이다. 초판 manifest 의 `9.1.1.4072`·`2024` 는 틀렸다
//! (PR #6794 지적) — 정정했다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6793/1611000-201000141-small-air-transport-study.hwp";
const TOC_TEXT: &str = "차";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6793 정식 HWP fixture 읽기")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

/// `pi` 로 식별한 그 문단의 글줄 상자들(본문 직계·표 안 모두).
fn runs_of_para(node: &RenderNode, pi: usize, out: &mut Vec<(String, f64, f64)>) {
    if let RenderNodeType::TextRun(tr) = &node.node_type {
        if tr.para_index == Some(pi) && !tr.text.trim().is_empty() {
            out.push((tr.text.clone(), node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    for child in &node.children {
        runs_of_para(child, pi, out);
    }
}

fn page_runs(core: &DocumentCore, page: u32, pi: usize) -> Vec<(String, f64, f64)> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    runs_of_para(&tree.root, pi, &mut out);
    out
}

/// `< 차 례 >`(`pi=1`)는 **2쪽**에 있고 1쪽에는 없다.
///
/// 수정 전: 1쪽 y=1459.7(용지 밖 363.9px). 초판 수정: 1쪽 y=966.9(보이지만 틀린 쪽).
#[test]
fn the_toc_heading_belongs_to_the_second_page() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 12, "쪽수 핀 — 한/글 2024 도 12쪽");

    let first = page_runs(&core, 0, 1);
    assert!(
        first.is_empty(),
        "1쪽에 `< 차 례 >`(pi=1)가 있으면 안 된다 — #6793 쪽 귀속 회귀 (찾음 {first:?})"
    );

    let second = page_runs(&core, 1, 1);
    assert!(
        second.iter().any(|(t, ..)| t.contains(TOC_TEXT)),
        "2쪽에 `< 차 례 >`(pi=1)가 있어야 한다 — 찾은 런 {second:?}"
    );
}

/// 그 줄은 2쪽 **본문 흐름 상단**에서 시작한다(저장 `vpos == 0`).
#[test]
fn the_toc_heading_starts_at_the_top_of_the_second_page() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let tree = core.build_page_render_tree(1).expect("2쪽 render tree");
    let body_top = find_body(&tree.root).expect("Body 노드").bbox.y;
    let runs = page_runs(&core, 1, 1);
    let top = runs.iter().map(|(_, y, _)| *y).fold(f64::MAX, f64::min);

    // 저장 상단 허용오차 — 한 줄(26.7px) 안.
    assert!(
        (top - body_top).abs() <= 27.0,
        "`< 차 례 >`는 2쪽 본문 상단에서 시작해야 한다 — #6793 회귀          (줄 상단 {top:.1}, 본문 상단 {body_top:.1})"
    );
}

/// 1쪽의 모든 글자가 용지 안에 있다.
///
/// 수정 전 `< 차 례 >`가 용지 밖 363.9px 였다.
#[test]
fn no_text_leaves_the_paper_on_the_cover_page() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;

    fn worst(node: &RenderNode, bottom: f64, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::TextRun(_)) {
            *out = out.max(node.bbox.y + node.bbox.height - bottom);
        }
        for child in &node.children {
            worst(child, bottom, out);
        }
    }
    let mut over = 0.0f64;
    worst(&tree.root, paper_bottom, &mut over);

    assert!(
        over <= 0.5,
        "1쪽 글자가 용지를 넘으면 안 된다 — #6793 회귀          (초과 {over:.1}px, 용지 하한 {paper_bottom:.1}; 수정 전 +363.9px)"
    );
}
