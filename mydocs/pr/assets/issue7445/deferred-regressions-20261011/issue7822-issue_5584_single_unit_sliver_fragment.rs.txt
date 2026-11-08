//! [#5584 원인 ②] 표 조각이 한 유닛(sliver)만 담고 쪽을 끊어 rhwp 5쪽 vs 한글
//! 4쪽이 되던 회귀 가드.
//!
//! 3232693(취업취약계층 취업지원서비스 수급 요건, 별표 서식) 실측 — 수정 전:
//! 1쪽 조각이 r=7 에서 capacity cut(budget 153.9, 소비 141.3)에 막혀 다음 유닛
//! (21.3px)을 12.7px 부족으로 밀어냈고, 그 유닛 바로 뒤가 저장 프레임 리셋이라
//! 2쪽이 **한 유닛만** 담은 채 끊겨 쪽이 하나 늘었다. 한글은 저장 사다리대로
//! 1쪽에 리셋 경계까지 담는다(4쪽). 수정: 조각 중간 행에서도 capacity cut 이
//! 저장 프레임 끝 직전(한 유닛 규모 ≤24px)에 멈추면 프레임 끝까지 당긴다 —
//! 기존 opening_source_frame 계약의 중간 행 판.

use rhwp::document_core::DocumentCore;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5584/3232693_employment_support_criteria.hwpx";

#[test]
fn issue_5584_no_single_unit_sliver_fragment() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5584 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5584 fixture");

    assert_eq!(
        core.page_count(),
        4,
        "3232693 은 한글 기준 4쪽이다 (수정 전 rhwp 는 sliver 조각으로 5쪽)"
    );
}
