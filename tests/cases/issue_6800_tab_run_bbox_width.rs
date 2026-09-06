//! [Issue #6800] 오른쪽 탭이 든 `TextRun` 의 bbox 폭이 **탭 스톱까지** 잡혀, 같은 줄의
//! 다음 런을 통째로 덮은 것처럼 계수됐다.
//!
//! `1192000-202100017` 1쪽 표 칸 실측:
//!
//! ```text
//!   수정 전  TextRun x=221.8 w=496.0  "  - <TAB>"               ← 탭 스톱까지
//!            TextRun x=249.1 w=474.3  "내수면 어업인의 경쟁력 확"
//!            → text-overlap 468.70 x 14.67px
//!
//!   형제 줄(탭 없음)
//!            TextRun x=221.8 w= 29.0  "  - "                     ← 정상
//!            TextRun x=250.8 w=467.0  "내수면어업의 지속적인 발전"
//! ```
//!
//! ⭐⭐ **근인은 오른쪽 탭의 advance 산출이다.** 오른쪽/가운데 탭의 전진량은
//! "탭 스톱까지"가 아니라 **"뒤따르는 블록이 스톱에 맞도록 필요한 만큼"** 이다.
//! `estimate_text_width` 는 전자를 주고(496.0), 실제 배치는 후자를 쓴다(27.3).
//!
//! ```text
//!   D6800 run="  - <TAB>" x_in=221.8 full_w=496.0 emitted=496.0 x_out=717.8
//!                                                  ↑ 실제 다음 런은 249.1
//! ```
//!
//! ⭐ 수정은 **재배치가 일어나는 그 자리**에서 한다 —
//! `paragraph_layout.rs` 의 `pending_right_tab_render` 분기가
//! `x = col_area.x + effective_pos - next_w` 로 다음 블록 위치를 정하는 곳이다.
//! 거기서 이미 직전 런의 `tab_leaders` 끝점을 같은 `x` 로 보정하고 있었다 —
//! **bbox 와 장식 상자도 같은 `x` 를 쓰게** 했다. 하나의 계산 결과를 공유한다.
//!
//! 이 자리에서만 하므로
//! 1. 이 재배치를 유발한 **논리적 끝 탭**의 런에만 닿는다.
//! 2. 탭 뒤에 가시문자가 있는 런은 애초에 `pending` 을 세우지 않아 대상이 아니다.
//! 3. 재배치가 없는 줄의 런은 전혀 건드리지 않는다.
//! 4. 폭을 **다음 런의 시작 x 까지만** 줄인다 — 그보다 더 줄이지 않으므로
//!    **진짜 glyph 겹침은 그대로 계수된다.**
//!
//! ⚠ **출력에는 차이가 없다** — 탭은 잉크를 안 그린다. 1쪽 SVG 는 수정 전후
//! 320,055 bytes / 같은 SHA-256 이고, 공백 제거 592자가 한/글과 완전히 일치한다.
//! 이것은 **렌더 트리 계약 결함**이다.
//!
//! ⭐ 그래도 고쳐야 하는 이유: `text_overlap_baseline` 래칫이 이 가짜 겹침을 세어
//! **진짜 글자겹침을 가린다.** 실측 — `3070000-202200004` 글자겹침 **230 → 132**.
//!
//! ⚠ fixture 는 `hancom-office-2018` 저장본이라 저장소 정책상 기준 엔진은 **2020**
//! 이다(초판은 2024 로 적었다 — PR #6801 지적). 다만 이 축은 출력 불변이라 기준 PDF
//! 대조가 판정을 주지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6800/1192000-202100017-policy-research-report.hwp";

/// 정식 fixture는 `MANIFEST.json`의 SHA-256로 고정된다. fixture 부재는 회귀 시험의
/// 성공 조건이 아니므로 읽기 실패를 즉시 드러낸다.
fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6800 정식 HWP fixture 읽기")
}

/// (텍스트, x, 폭) — 같은 `TextLine` 에 속한 런들.
fn lines_with_runs(node: &RenderNode, out: &mut Vec<Vec<(String, f64, f64)>>) {
    if matches!(node.node_type, RenderNodeType::TextLine { .. }) {
        let mut runs: Vec<(String, f64, f64)> = node
            .children
            .iter()
            .filter_map(|c| match &c.node_type {
                RenderNodeType::TextRun(tr) => Some((tr.text.clone(), c.bbox.x, c.bbox.width)),
                _ => None,
            })
            .collect();
        runs.sort_by(|a, b| a.1.total_cmp(&b.1));
        if !runs.is_empty() {
            out.push(runs);
        }
    }
    for child in &node.children {
        lines_with_runs(child, out);
    }
}

/// 대상 런과 **논리적 다음 런**을 직접 집어 경계를 고정한다.
///
/// ⚠ 종전 시험은 그 쪽 전체의 최대 겹침만 봤다 — 대상 런이 사라져도 0 으로 통과할 수
/// 있었다(PR #6801 지적).
#[test]
fn the_trailing_tab_run_ends_exactly_where_the_next_run_starts() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 1, "쪽수 핀 — 본 수정은 조판 불변");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut lines = Vec::new();
    lines_with_runs(&tree.root, &mut lines);

    let target = lines
        .iter()
        .find(|runs| runs.iter().any(|(t, ..)| t == "  - \t"))
        .expect("1쪽에 `\"  - <TAB>\"` 런이 있는 줄이 있어야 한다");

    let tab_idx = target
        .iter()
        .position(|(t, ..)| t == "  - \t")
        .expect("탭 런");
    let (_, tab_x, tab_w) = target[tab_idx];
    let (next_text, next_x, _) = target
        .get(tab_idx + 1)
        .expect("탭 런 뒤에 논리적 다음 런이 있어야 한다")
        .clone();

    assert!(
        next_text.starts_with("내수면"),
        "다음 런이 예상 텍스트여야 한다: {next_text:?}"
    );
    // 양방향 — 덜 줄여도(겹침) 더 줄여도(가짜 여백) 실패한다.
    assert!(
        (tab_x + tab_w - next_x).abs() <= 0.5,
        "끝 탭 런은 다음 런 시작에서 정확히 끝나야 한다 — #6800 회귀          (탭 런 {tab_x:.1}..{:.1}, 다음 런 {next_x:.1}; 수정 전 폭 496.0 → 717.8 까지)",
        tab_x + tab_w
    );
}

/// ⚠ 음성 — **탭 뒤에 가시문자가 있는 런**은 건드리지 않는다.
///
/// `lseg-05-tab.hwp` 의 `"이 파일은 탭시작<TAB>탭끝 라"` 류는 끝 탭이 아니므로
/// 오른쪽 탭 재배치를 유발하지 않고, 폭도 종전 그대로여야 한다.
#[test]
fn a_mid_run_tab_with_text_after_it_is_untouched() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/lseg-05-tab.hwp"))
            .expect("lseg-05-tab.hwp 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut lines = Vec::new();
    lines_with_runs(&tree.root, &mut lines);

    let mut checked = 0;
    for runs in &lines {
        for (text, _, width) in runs {
            // 탭이 들었는데 **뒤에 가시문자가 있는** 런.
            let Some((_, after)) = text.rsplit_once('\t') else {
                continue;
            };
            if after.trim().is_empty() {
                continue;
            }
            checked += 1;
            assert!(
                *width > 400.0,
                "탭 뒤 가시문자가 있는 런의 폭은 종전대로여야 한다                  (텍스트 {text:?}, 폭 {width:.1})"
            );
        }
    }
    assert!(
        checked >= 3,
        "탭 뒤 가시문자 런을 3개 이상 확인해야 한다 (확인 {checked})"
    );
}

/// 같은 줄의 런끼리 가로로 크게 겹치면 안 된다 — 쪽 전체 관문.
#[test]
fn tab_run_bbox_does_not_swallow_the_next_run() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut lines = Vec::new();
    lines_with_runs(&tree.root, &mut lines);

    let mut worst = 0.0f64;
    for runs in &lines {
        for pair in runs.windows(2) {
            worst = worst.max((pair[0].1 + pair[0].2) - pair[1].1);
        }
    }

    assert!(
        worst < 20.0,
        "같은 줄의 런끼리 크게 겹치면 안 된다 — #6800 회귀          (겹침 {worst:.1}px; 수정 전 468.7px)"
    );
}
