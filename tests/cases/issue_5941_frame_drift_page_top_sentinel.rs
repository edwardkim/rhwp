//! [#5941 · #5822 후속] 저장 frame 드리프트 허용(#5822)이 `top == 0`(쪽 시작
//! vpos 센티널)에까지 발동해 문서를 3쪽 압축하던 자기-회귀 가드.
//!
//! 3240179(효율관리기자재 별표, HWPX) — r8 통합 커밋(c627bb2f9)의 #5822 갈래
//! (`current_flow_inside_source_frame`)가 top=0.0·흐름 42~48px 형상에서 오발동,
//! 쪽-말미 크기 표(913~918px)를 현재 쪽에 강제로 앉혀 16→13쪽(한글 18)으로
//! 압축했다(#5941 bisect 로 분리 특정). top==0 은 frame 위치 증거가 아니라
//! 쪽-시작 센티널이므로 실제 frame(top>0)만 이 갈래의 대상이다. 수정 후 이
//! 문서는 한글 정답 18쪽과 정확히 일치하고, #5822 원 재현체(156634833)의
//! 39쪽 정합도 로컬 실측으로 유지 확인했다.

use rhwp::document_core::DocumentCore;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5941/3240179_efficiency_test_orgs.hwpx";

#[test]
fn issue_5941_page_top_sentinel_is_not_a_stored_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5941 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5941 fixture");

    assert_eq!(
        core.page_count(),
        18,
        "3240179 은 한글 2024 기준 18쪽이다 (top==0 오발동 시 13~15쪽으로 압축)"
    );
}
