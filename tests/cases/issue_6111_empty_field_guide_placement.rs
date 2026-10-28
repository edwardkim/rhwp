//! [Issue #6111] 빈 누름틀 안내문이 저장 줄마다 중복되고, 첫 줄은 본문 우단에서
//! 시작해 쪽 밖으로 잘리며, 줄바꿈 없이 넘친다 (56345 7쪽 "o 해외사례").
//!
//! 근인 둘.
//!   1) 누름틀이 걸린 문자가 **줄 끝이자 다음 줄의 시작**이라 두 줄이 같은
//!      누름틀을 각자 그렸다. 그중 앞 줄은 배분 정렬된 줄의 마지막 문자라
//!      `char_x_map` 이 본문 우단(718.6px)을 돌려줬다.
//!   2) 안내문은 흐름에 영향이 없는 편집 전용 표시라 조판 줄바꿈을 타지 않아,
//!      49자 안내문이 한 줄로 x 93.7 → 943.7px(용지 폭 794) 까지 나갔다.
//!
//! 수정: 경계 문자의 소유권을 다음 줄에 준다(같은 파일의 TAC 계약
//! `next_line_starts_at_run_end` 와 같은 규칙) + 안내문을 본문 폭에서 접는다.
//!
//! 인쇄 경로는 #3375 가 이미 `editor_only` 로 막았다 — 이 테스트는 편집 표시의
//! 좌표 계약을 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6111/56345_regulatory_impact_analysis.hwp";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 6;
/// 빈 누름틀 안내문의 첫머리.
const GUIDE_HEAD: &str = "해외 선진국";

#[test]
fn issue_6111_empty_field_guide_is_drawn_once_inside_the_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 7 render tree");

    let mut runs: Vec<(f64, f64, f64)> = Vec::new();
    collect_guide_runs(&page.root, GUIDE_HEAD, &mut runs);
    assert_eq!(
        runs.len(),
        1,
        "안내문 첫머리가 여러 줄에 중복으로 그려졌다 ({}회): {runs:?}",
        runs.len()
    );

    // 본문 우단(≈718.1px) 안에서 시작하고 끝나야 한다 — 결함 시 시작 718.6·끝 943.7.
    let (_, x, right) = runs[0];
    assert!(
        x < 200.0,
        "안내문이 본문 우단에서 시작한다: x={x:.1} (배분 정렬 줄의 마지막 문자 좌표)"
    );
    assert!(
        right <= 719.0,
        "안내문이 본문 폭을 넘어 그려진다: 오른끝={right:.1}"
    );

    // 접힌 뒤 줄들도 본문 안이어야 한다.
    let mut all: Vec<(f64, f64, f64)> = Vec::new();
    collect_guide_runs(&page.root, "국제적 근거", &mut all);
    for (_, x, right) in all {
        assert!(
            x >= 0.0 && right <= 719.0,
            "접힌 안내문 조각이 본문 밖이다: x={x:.1}~{right:.1}"
        );
    }
}

fn collect_guide_runs(node: &RenderNode, needle: &str, out: &mut Vec<(f64, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            out.push((node.bbox.y, node.bbox.x, node.bbox.x + node.bbox.width));
        }
    }
    for child in &node.children {
        collect_guide_runs(child, needle, out);
    }
}
