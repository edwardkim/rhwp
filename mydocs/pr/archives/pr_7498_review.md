---
kind: report
status: historical
last_verified: 2026-10-02
---

# PR #7498 리뷰 — ‘채우기 없음’ 속성 조회

## 최종 판정

승인 — 선별 통합 후보의 전체 Rust·필수 lint·fresh WASM Query/재열기 검증을 통과했다.
통합 PR #7511의 code candidate `39f0a2792`와 최종 문서 head `f2a9341f7`의 필수 게이트를 확인했다.
작업지시자의 병합 승인으로 2026-10-02 12:36:38 KST에 devel로 병합했다.
merge SHA: `c77ed685e21de0b2edca918efcec7c2ebbbf2bd5`.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](../pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `98133ad9c57c695fce2dcf8d684e801c8d431438`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7498) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36812415019/job/110227773337).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 수용 권고 범위와 소비 경로

#7495 종료 제안. 기능 commit `98133ad9`를 누적 적용했다.
`fill_json_values` 공통 Query 결과를 글자·문단·표 셀·쪽 채우기 조회가 사용한다.
단색·무늬 없음·배경 sentinel 0xFFFFFFFF를 none으로 답하고 실제 흰색 0x00FFFFFF는 solid로 유지한다.
무늬 있는 채우기는 none으로 지우지 않는다. IR·serializer·paint는 변경하지 않는다.
조판 변경 규칙/페이지네이션/픽셀 기준값 수정은 비해당이다.

## 검증 결과

- focused 4/4 PASS. 실제 HWP/HWPX no-fill, 무늬 있는 셀, 진짜 흰색 셀과 API 지정 흰색을 대조했다.
- fresh WASM 실제 브라우저에서 no-fill과 white의 글자·문단 Query 및 HWP/HWPX 재열기 PASS.
- e509 기준 merge simulation clean. 원 exact-head Full CI 성공.
- 전체 GUI 메뉴 조작 검증은 하지 않았으며 API 계약의 통과와 구별한다.

## 통합 및 후속 처리

선별 source head `514d4933b`, 검증 checkout `f27661e63`에서 전체 Rust 10,243 PASS / 0 FAIL / 50 skip,
필수 Clippy 3종·workspace build·fmt·manifest 및 fresh WASM 실제 API 3/3를 통과했다.
[선별 후보 최종 결과](../pr_semanticist21_20261002_review_impl.md#선별-후보의-최종-rust-결과)에 source SHA와 명령을 연결한다.
원 head CI는 선별 후보의 GitHub CI를 대신하지 않는다. 선별 코드는 [통합 PR #7511](https://github.com/edwardkim/rhwp/pull/7511)로 게시했다.
[게시 head의 Full CI](https://github.com/edwardkim/rhwp/actions/runs/36955832835)와 CodeQL·Render Diff·Adapter·Proptest는 모두 성공했다.
source/test 보정 없이 통합 PR로 병합했으며 관련 #7495는 devel 병합 후 자동 종료됐다.
원 PR은 통합 반영을 안내하고 중복 병합하지 않고 닫는다. 기여자의 fork branch는 보존한다.
후속 댓글에서 이 기록의 확정 commit과 merge SHA를 연결한다.
