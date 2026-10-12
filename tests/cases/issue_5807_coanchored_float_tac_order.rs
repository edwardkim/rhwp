//! [Issue #5807] 같은 빈 문단에 co-anchored 된 자리차지 표(4행)와 TAC 표(2행)의
//! 위·아래 순서가 뒤집혀, 2쪽에서 표가 쪽 밖으로 354.6px 넘쳤다 (1880690 서식).
//!
//! 근인: 빈 host 의 float 표 v_off 오름차순 정렬(#986/#1088)이 TAC 표에 정렬 키
//! 0 을 주어, 양수 v_off(937) float 앞으로 끌어왔다. v_off 는 자리차지 개체의
//! 문단 상대 위치이고 TAC 의 흐름 위치와 다른 축이다. 저장 배열 순서·float
//! offset·TAC 저장 줄 vpos(12924)·한글 배치 모두 float 먼저다.
//!
//! 수정: TAC 표와 양수 v_off float 표가 혼재한 host 는 #1639(음수 혼재)/#2287
//! (vpos 리셋)과 같이 정렬을 끄고 배열(저장) 순서를 보존한다.
//!
//! 픽스처는 원본 HWP5 의 구역0 문단 0..4 절단 + BinData 스텁 축소본(9KB).
//!
//! 활성 렌더링 검사는 사용자 지시에 따라 #7821 (#7445 서브이슈)로 이관했다.
//! 샘플·PDF와 이 Rust 파일은 보존한다. 검사 원문은
//! `mydocs/pr/assets/issue7445/deferred-regressions-20261011/issue7821-issue_5807_coanchored_float_tac_order.rs.txt`에 있다.
//! 해당 이슈의 전쪽 Native/fresh WASM 검증 뒤 관계형 검사로 복원한다.
