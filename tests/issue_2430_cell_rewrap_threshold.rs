//! [#2430] PR #2309(커밋 6d910836) 의 셀 저장-ls1 문단 폭초과 재래핑 허용이
//! 한글과 정합이던 분할 표 문서를 과다분할로 회귀시킨 건에 대한 회귀 가드.
//!
//! 근인: `recompose_stored_single_line_if_overflowing` 의 발동 임계가 실폭 >
//! 내폭 ×1.05 로 너무 느슨해, 측정/렌더 패딩 발산(#2237)으로 살짝(1.05~1.35×)
//! 초과한 정합 셀까지 재래핑해 줄수를 부풀리고 쪽당 표 행 적재를 떨어뜨렸다.
//! 임계를 ×1.8 로 좁혀(#2525 body 판과 동일) 거짓 재래핑을 제거한다. #2291
//! 원 타깃(76자 1-lineseg = ~7.6× 초과)은 임계 위라 계속 재래핑(절단 방지
//! 유지 — issue_2287/issue_2291 테스트가 별도 고정).
//!
//! 대표: `1382000_중간보고자료_2022_가정폭력실태조사` — 한글 39쪽, 임계
//! ×1.05 에서 40쪽(+1) 과다분할, ×1.8 에서 39쪽 정합. (10k 서베이 순변화
//! +4/회귀 0 으로 무회귀 확인.)

use rhwp::document_core::DocumentCore;

#[test]
fn issue_2430_cell_rewrap_threshold_no_oversplit() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2430/1382000_domestic_violence_survey.hwp");
    let bytes = std::fs::read(&path).expect("read fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let pages = core.page_count();
    // 한글 정답 39쪽. 셀 재래핑 임계가 ×1.05 로 되돌아가면 40쪽(+1) 과다분할.
    assert_eq!(
        pages, 39,
        "1382000 은 한글 기준 39쪽이어야 함 (셀 재래핑 임계 완화 회귀 시 40쪽). #2430"
    );
}
