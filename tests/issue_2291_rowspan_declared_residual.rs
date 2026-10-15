//! Issue #2291/#2237 (PR #2309) — 저장 줄 문단의 내용 보존 검사.
//!
//! `mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp` s5 pi8 (244×10, 다층
//! rowspan: c0 rs=67 / c1 rs=28 / c2 rs=9 / c3 rs=3~4) 기준. 정답지:
//! `mydocs/pr/assets/issue7445/education-curriculum-map/existing-hancom2022-print.pdf` (한글 2022 COM, 415쪽).
//!
//! 기존 첫 조각의 절대 높이 검사는 base에서도 90% 미만인 원본의 잠정 배치를
//! 고정하므로 #7445/#7743으로 이관했다. 원본·기준 출력과 이관 근거는 위 자산의
//! README.md에 보존한다. 내용 보존 검사는 계속 실행한다.
//!
//! **저장-ls-1 폭 초과 문단의 재래핑** — r183 c8 76자 문단(저장 lineseg
//!    1개)이 재래핑 없이 1줄 렌더되면 "…실천 계획 세"에서 절단된다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root)
        .join("mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 1342000_edu_curriculum_map.hwp")
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

#[test]
fn issue_2291_stored_single_seg_paragraph_rewraps_full_text() {
    let core = core();
    let mut found = false;
    for page in 130..140u32 {
        if let Ok(tree) = core.build_page_render_tree(page) {
            if find_text(&tree.root, "실천 계획 세우기") {
                found = true;
                break;
            }
        }
    }
    assert!(
        found,
        "r183 c8 문단 꼬리('…실천 계획 세우기') 부재 — 저장-ls-1 재래핑 회귀 (절단 '…실천 계획 세')"
    );
}
