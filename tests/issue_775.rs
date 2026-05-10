//! Issue #775: Task #703 회귀 — `samples/exam_eng.hwp` 4페이지 27번 보기 그림 (1×1 InFrontOfText 표)
//! 이 다단(2단) 단 1 (우측 컬럼) 상단(정상 y≈277.08 px)에서 단 1 중반(현재 y≈723.69 px)으로
//! 약 +446.6 px 밀리는 회귀.
//!
//! 회귀 진원지: a759a1c2 (Task #703 / PR #707 — typeset.rs 의 InFrontOfText/BehindText 가드).
//! Task #703 fix 가 단일 컬럼 케이스(calendar_year.hwp)에서는 정확하나, 다단 영역에서
//! 컬럼 분배 자체를 변경하여 회귀.
//!
//! 권위 자료: `pdf/exam_eng-2022.pdf` (한글 2022 편집기 PDF)
//!
//! 결함 메커니즘:
//! - pi=181 ci=1: 1×1 InFrontOfText 표, treat_as_char=false, vert=문단(2.4mm),
//!   size=106.0×111.9 mm. 그림(pi=181 ci=0) 위 데코레이션
//! - exam_eng.hwp 는 2단 (column_count == 2) 다단 영역
//! - Task #703 fix 적용 시 InFrontOfText 표를 cur_h 누적에서 제외 → 단 0/1 컨텐츠 분배 변경
//!   → pi=181 paragraph 가 단 1 상단(y≈277)에서 단 1 중반(y≈723)으로 밀림

use std::fs;
use std::path::Path;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/exam_eng.hwp";
const TARGET_PAGE: u32 = 3; // 0-indexed: page 4

/// para_index/control_index 일치하는 첫 Table 노드의 bbox 를 (y_top, y_bottom) 으로 반환.
fn find_table_bbox(root: &RenderNode, target_pi: usize, target_ci: usize) -> Option<(f64, f64)> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if let RenderNodeType::Table(t) = &node.node_type {
            if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
                let bbox = &node.bbox;
                return Some((bbox.y, bbox.y + bbox.height));
            }
        }
        for child in &node.children {
            stack.push(child);
        }
    }
    None
}

#[test]
fn issue_775_exam_eng_p4_pi181_table_at_column_top() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    let tree = doc.build_page_render_tree(TARGET_PAGE)
        .expect("build_page_render_tree page 4");

    // pi=181 ci=1 = 1×1 InFrontOfText 표 (27번 보기 그림 위 데코레이션)
    let (pi181_top, pi181_bottom) = find_table_bbox(&tree.root, 181, 1)
        .expect("pi=181 ci=1 (27번 보기 1×1 InFrontOfText 표) Table 노드 누락");

    eprintln!(
        "[issue_775] page=4 pi=181 ci=1 bbox=[{:.2}..{:.2}] (height={:.2})",
        pi181_top, pi181_bottom, pi181_bottom - pi181_top,
    );

    // 정상 동작: pi=181 ci=1 표가 단 1 상단 ≈ y=277.08 px (PDF 권위 정합).
    // 회귀: y=723.69 px (+446.6 px 밀림).
    // 5 px 허용 오차 (rounding / 후속 fix 누적 drift 안전 마진).
    assert!(
        (pi181_top - 277.08).abs() <= 5.0,
        "pi=181 ci=1 (27번 보기 InFrontOfText 표) 가 단 1 상단(≈y=277.08)에 위치해야 함. \
         실제 y={:.2} px (PDF 정상값과 차이={:.2} px). \
         회귀 진원지: a759a1c2 (Task #703 / PR #707) — typeset.rs:1550 InFrontOfText 가드 \
         가 다단 영역에서 컬럼 분배를 변경.",
        pi181_top, pi181_top - 277.08,
    );
}
