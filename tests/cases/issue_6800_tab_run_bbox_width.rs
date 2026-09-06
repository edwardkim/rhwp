//! [Issue #6800] 탭이 든 `TextRun` 의 bbox 폭이 **줄 끝까지** 잡혀, 같은 줄의 다음
//! 런을 통째로 덮은 것처럼 계수됐다.
//!
//! `1192000-202100017` 1쪽 표 칸 실측:
//!
//! ```text
//!   수정 전  TextRun x=221.8 w=496.0  "  - \t"                  ← 줄 폭 전체
//!            TextRun x=249.1 w=474.3  "내수면 어업인의 경쟁력 확"
//!            → text-overlap 468.70 x 14.67px
//!
//!   형제 줄(탭 없음)
//!            TextRun x=221.8 w= 29.0  "  - "                     ← 정상
//!            TextRun x=250.8 w=467.0  "내수면어업의 지속적인 발전"
//! ```
//!
//! ⚠ **출력에는 차이가 없다** — 탭은 잉크를 안 그린다. 한/글 2024 PDF 와 공백 제거
//! 592자로 **수정 전후 모두 완전히 일치**한다. 이것은 **렌더 트리 계약 결함**이다.
//!
//! ⭐ 그래도 고쳐야 하는 이유: `text_overlap_baseline` 래칫이 이 가짜 겹침을 세어
//! **진짜 글자겹침을 가린다.** 실측 — `3070000-202200004` 글자겹침 **230 → 132**.
//!
//! 수정: 탭이 실제로 든 런만, **다음 런의 시작 x** 로 폭을 자른다. 줄 마지막 런은
//! 다음이 없으므로 그대로 둔다.

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

/// 한 `TextLine` 안에서 이웃한 두 `TextRun` 의 최대 가로 겹침.
fn worst_run_overlap(node: &RenderNode, out: &mut f64) {
    if matches!(node.node_type, RenderNodeType::TextLine { .. }) {
        let mut runs: Vec<(f64, f64)> = node
            .children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::TextRun(_)))
            .map(|c| (c.bbox.x, c.bbox.x + c.bbox.width))
            .collect();
        runs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for pair in runs.windows(2) {
            *out = out.max(pair[0].1 - pair[1].0);
        }
    }
    for child in &node.children {
        worst_run_overlap(child, out);
    }
}

/// 같은 줄의 런끼리 가로로 크게 겹치면 안 된다.
///
/// 수정 전 `468.7px`. 글자 사이 미세 커닝(수 px)은 허용한다.
#[test]
fn tab_run_bbox_does_not_swallow_the_next_run() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 1, "쪽수 핀 — 본 수정은 조판 불변");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut worst = 0.0f64;
    worst_run_overlap(&tree.root, &mut worst);

    assert!(
        worst < 20.0,
        "같은 줄의 런끼리 크게 겹치면 안 된다 — #6800 회귀          \
         (겹침 {worst:.1}px; 수정 전 468.7px)"
    );
}
