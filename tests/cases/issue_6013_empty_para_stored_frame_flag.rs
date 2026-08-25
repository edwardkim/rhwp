//! [#6013] 빈 문단의 쪽-스케일 저장 프레임 되감김 플래그가 유실돼 RowBreak
//! 조각의 마지막 줄이 저장 chunk 경계 0.8px 앞에서 다음 쪽으로 밀리던 가드.
//!
//! 30269(제도개선권고안, 22쪽 HWP5) 문단 0.136(자리차지 1×1 RowBreak 표) —
//! 저장 사다리 chunk1 은 29유닛(p[17] 빈 문단에서 vpos 67053+1200 → 500 으로
//! 되감김 = 쪽 경계)인데, 유닛 생성자의 #1488 게이트(`para_has_visible_text
//! &&`)가 빈 문단의 stored_frame_break_before 를 지워 흡수 기계
//! (absorb_tail_before_stored_frame_break)가 경계를 못 보고 capacity cut 이
//! 0.6~0.8px 차로 28유닛에서 멈췄다. 한글 2020 은 29유닛(마지막 줄 "없는
//! 경우에는 그러하지 아니한다.")을 10쪽에 수용한다. 수정 = 쪽-스케일 되감김
//! (is_stored_frame_rewind, 본문-절반 하한 통과)은 빈 문단이어도 나른다 —
//! hard_break_before 는 #1488 그대로라 빈 문단 리셋의 강제 분할은 없다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

// #6023(PR #6047)과 같은 재현물 — 같은 경로·같은 내용이라 랜딩 순서 무관 합류.
const SAMPLE: &str = "samples/issue6023/30269_reform_recommendation.hwp";

fn count_runs_containing(node: &RenderNode, needle: &str) -> usize {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) => 1,
        _ => 0,
    };
    own + node
        .children
        .iter()
        .map(|child| count_runs_containing(child, needle))
        .sum::<usize>()
}

#[test]
fn issue_6013_stored_chunk_last_line_stays_on_page_10() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6013 fixture");
    assert_eq!(core.page_count(), 22, "쪽수 고정 (#6011 DoD)");

    // 한글 2020: 저장 chunk1 의 마지막 줄은 물리 10쪽 소유.
    let p10 = core.build_page_render_tree(9).expect("render p10");
    assert_eq!(
        count_runs_containing(&p10.root, "없는 경우에는 그러하지"),
        1,
        "p10 이 저장 chunk1 마지막 줄을 소유해야 한다 (플래그 유실 시 p11 로 밀림)"
    );
    let p11 = core.build_page_render_tree(10).expect("render p11");
    assert_eq!(
        count_runs_containing(&p11.root, "없는 경우에는 그러하지"),
        0,
        "p11 이 그 줄을 중복/이월 소유하면 안 된다"
    );
}
