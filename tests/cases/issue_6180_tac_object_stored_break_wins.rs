//! Issue #6180: 글자처럼 취급되는 라벨 상자를 줄머리에 단 문단이 저장 나눔보다 **한 글자
//! 일찍** 접혀 줄이 하나 더 생긴다.
//!
//! `layout_inline_table_paragraph` 의 줄바꿈 판정은 저장 나눔에 도달하기 전이라도 측정 폭이
//! 오른 여백을 넘으면 접었다. rhwp 의 글자 폭 추정이 한/글과 몇 px 다르면 마지막 한 글자가
//! 제 줄로 떨어진다.
//!
//! ```text
//! 156745974 7쪽 pi=94  `- 재정지원 대·중소기업 상생협력 활동을 통해 중소 협력업체의 안전`
//!   저장 나눔  ch=35
//!   rhwp       ch=34 에서 폭 초과로 먼저 접음 → `전` 한 글자가 제 줄로 (2줄이어야 할 문단이 3줄)
//! ```
//!
//! 저장 나눔이 아직 남아 있으면 그것이 그 줄의 권위다. 다 쓴 뒤(재래핑 구간)에는 종전대로
//! 폭으로 접는다.
//!
//! 같은 쪽 `pi=93`·`pi=96` 은 #6181(인라인 TAC 표 줄의 다음 줄을 저장 `vertpos` 에 놓기)로
//! 이미 닫혔고, 이 시험은 세 문단 모두를 함께 고정한다 — 저장 줄 전진 2764HU = 36.85px,
//! 줄 수 2.
//!
//! 한/글 2022 오라클과 7쪽이 줄 단위로 일치한다(세 문단 모두 2줄, 런 상단 간격 32.8px —
//! 줄 상자 전진 36.85px 에서 첫 줄 baseline 이 상자 안에서 내려앉은 만큼 작다).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6180/156745974_tac_object_line_spacing.hwpx";
const PAGE_INDEX: u32 = 6; // 0-based — 7쪽

/// 저장 `vertpos` 델타 2764 HU = 36.85px @96dpi (= 줄 상자 lh 1864 + ls 900).
const STORED_ADVANCE_PX: f64 = 36.85;
const TOLERANCE_PX: f64 = 1.5;

/// `○ (지원내용) …` · `- 재정지원 …` · `- 기술지원 …` — 셋 다 줄머리에 1×1 TAC 표를 단다.
const PARAS: [usize; 3] = [93, 94, 96];

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 이 문단이 만든 본문 글줄들의 상단 y — 6px 안쪽은 같은 줄로 묶는다.
fn line_tops(nodes: &[&RenderNode], para_index: usize) -> Vec<f64> {
    let mut ys: Vec<f64> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.para_index == Some(para_index)
                    && run.cell_context.is_none()
                    && !run.text.trim().is_empty() =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .collect();
    ys.sort_by(f64::total_cmp);
    let mut grouped: Vec<f64> = Vec::new();
    for y in ys {
        if grouped.last().map(|&last| y - last > 6.0).unwrap_or(true) {
            grouped.push(y);
        }
    }
    grouped
}

#[test]
fn issue_6180_stored_break_wins_over_measured_width() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6180 sample");
    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p7");

    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    for para_index in PARAS {
        let tops = line_tops(&nodes, para_index);
        assert_eq!(
            tops.len(),
            2,
            "pi={para_index} 은 두 줄이어야 한다 — 실측 {}줄 {tops:?} \
             (회귀 시 pi=94 가 3줄: 저장 나눔보다 한 글자 일찍 접힘)",
            tops.len()
        );
        let advance = tops[1] - tops[0];
        assert!(
            (advance - STORED_ADVANCE_PX).abs() <= TOLERANCE_PX,
            "pi={para_index} 둘째 줄이 저장 vertpos 에 놓이지 않았다 — \
             실측 {advance:.2}px, 저장 {STORED_ADVANCE_PX:.2}px"
        );
    }
}
