//! Issue #2287 / PR #2290 P1 — 47×9 RowBreak rowspan 블록의 연속 조각 내용
//! 보존 + frame-tail overflow 부재 통합 회귀.
//!
//! `samples/task2287/1342000_edu_curriculum_map.hwp` (교육부 범교과 연결 맵)
//! s1 pi0 47×9 표: rowspan 블록(rows 2..4, 셀[2] 85문단+중첩표 123유닛)이
//! p25~p31 로 분할된다. 문서 총 페이지 수만으로는 잡히지 않는 회귀(리뷰
//! 지적: 내용이 p26 에서 p30 으로 이동 + page frame 아래 tail overflow)를
//! render tree 로 직접 고정한다:
//!
//! 회귀 시그니처 (수정 전):
//! - 첫 조각 split_total 이 per-row 합산으로 블록 전체(2354.6px)로 과대
//!   → p25 typeset used 2396.8px(예산 513.6) 오버플로
//! - orphan-rewind 이월 유닛이 연속 조각 시작 직후 hard-break 에 막혀
//!   17.3px sliver 페이지(p26 공백화)
//! - 걸침-전용 행/선언 max 로 rowspan 셀 bbox 가 원본(2354.6px) 유지
//!   → valign 이 콘텐츠를 페이지 밖(y≈1259~2808)으로 밀어 tail overflow

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/task2287/1342000_edu_curriculum_map.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 1342000_edu_curriculum_map.hwp")
}

fn text_stats(node: &RenderNode, ymax: &mut f64, runs: &mut usize) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            *ymax = (node.bbox.y + node.bbox.height).max(*ymax);
            *runs += 1;
        }
    }
    for c in &node.children {
        text_stats(c, ymax, runs);
    }
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

/// p25~p31 (0-based 24..=30) — 각 조각의 텍스트가 page frame 안에 있고
/// (tail overflow 부재), 조각마다 실질 내용이 존재해야 한다 (sliver 부재).
#[test]
fn issue_2287_edu_fragments_stay_in_page_frame() {
    let core = core();
    // A4 가로(용지 회전) 섹션: body 하단 ≈ 741px (마진 포함 여유 +40).
    const PAGE_TEXT_YMAX_LIMIT: f64 = 790.0;
    const MIN_RUNS_PER_FRAGMENT: usize = 30;
    for page in 24..=30u32 {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree p{}: {e:?}", page + 1));
        let (mut ymax, mut runs) = (0.0f64, 0usize);
        text_stats(&tree.root, &mut ymax, &mut runs);
        assert!(
            ymax <= PAGE_TEXT_YMAX_LIMIT,
            "p{} 텍스트 ymax {ymax:.0}px — page frame 밖 tail overflow 회귀 \
             (수정 전 p25=1585/p29=2808)",
            page + 1
        );
        assert!(
            runs >= MIN_RUNS_PER_FRAGMENT,
            "p{} 텍스트 run {runs}개 — 조각 공백화(sliver) 회귀 (수정 전 p26 소수)",
            page + 1
        );
    }
}

/// p26 (0-based 25) — 리뷰가 지적한 "p26 내용 공백화"의 직접 회귀:
/// 학교안전교육 조문 내용이 p26 에 존재해야 한다 (수정 전 p30 으로 이동).
#[test]
fn issue_2287_edu_p26_keeps_content() {
    let core = core();
    let tree = core.build_page_render_tree(25).expect("render tree p26");
    assert!(
        find_text(&tree.root, "조(학생 안전교육)"),
        "p26에 학생 안전교육 조문 부재 — 연속 조각 내용 이동 회귀"
    );
    assert!(
        find_text(&tree.root, "학교안전교육"),
        "p26에 학교안전교육 본문 부재 — 연속 조각 내용 이동 회귀"
    );
}
