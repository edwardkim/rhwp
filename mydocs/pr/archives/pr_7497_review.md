---
kind: report
status: historical
last_verified: 2026-10-02
---

# PR #7497 리뷰 — HTML 인라인 그림 붙여넣기

## 최종 판정

승인 — 보류 PR을 제외한 선별 통합 후보의 전체 Rust·필수 lint·fresh WASM API 검증을 통과했다.
통합 PR #7511의 code candidate `39f0a2792`와 최종 문서 head `f2a9341f7`의 필수 게이트를 확인했다.
작업지시자의 병합 승인으로 2026-10-02 12:36:38 KST에 devel로 병합했다.
merge SHA: `c77ed685e21de0b2edca918efcec7c2ebbbf2bd5`.

검토일: 2026-10-02. 작성자: semanticist21. 대상: devel.
기준 devel: `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`.
누적 진단 head: `c6ef30ea943308c37e5d68c8304dfdabdd7b8f74`.

누적 실행 명령·로그·제한은 [일괄 검토 기록](../pr_semanticist21_20261002_review_impl.md#누적-검증-결과)에 연결한다.
원 PR의 exact-head 녹색 CI와 누적 진단 head의 결과는 별개다. 누적 head는 7건을 포함하며
원 PR 또는 최종 수용 그룹의 전체 CI 통과로 간주하지 않는다. 메인터너 source/test 보정은 없다.

원 PR code head: `64f76e37b31bad9c0dedcb9bde67cc7a8f70eb58`.
[원 PR](https://github.com/edwardkim/rhwp/pull/7497) · [exact-head Build & Test](https://github.com/edwardkim/rhwp/actions/runs/36841040332/job/110330389788).
CI 집계 실패·진행 중 없음(확인 당시). 원 PR head는 최초 접수 이후 바뀌지 않았다.
Reviewer edwardkim 지정. 원격 GitHub 승인 이벤트는 아직 게시하지 않았다.

## 수용 권고 범위

#7496 종료 제안. 3개 기능 commit을 누적 적용했다.
본문 p/li/최상위 span의 data URI img를 글자처럼 취급하는 8-slot 컨트롤로 만들고,
char_offsets·서식 시작·문단 병합 및 저장/재열기에 보존한다. bullet을 위해 갭을 재구성하지 않는다.
셀 붙여넣기는 기존 text-only 정책을 유지하고 미사용 그림 데이터의 추가를 되돌린다.
조판 알고리즘·renderer·golden 변경은 없다. 입력 IR의 글/그림 순서 보존 변경이다.

## 검증 결과와 제한

- focused 8/8 PASS: 글 사이 그림, 반복/그림만 있는 항목, 서식·컨트롤 앵커, HWP/HWPX 재열기.
- fresh WASM 실제 HeadlessChrome 152: 글 ‘앞뒤’, 최종 SVG의 image 존재,
  Canvas 렌더 호출 및 HWP/HWPX 재열기 모두 PASS.
- e509 기준 merge simulation clean, 원 exact-head Full CI 성공.
- 중첩 span 그림과 모든 HTML 요소를 지원한다는 판정은 아니다. 원 PR의 잔여 범위를 유지한다.
- 브라우저 결과는 API smoke이며 Studio UI·독립 한컴 픽셀/좌표 일치 판정이 아니다.
  렌더러 규칙 수정이 없어 신규 조판 회귀의 PDF 점수 게이트는 이 변경의 주장과 비해당으로 분리한다.

## 통합 및 후속 처리

선별 source head `514d4933b`, 검증 checkout `f27661e63`에서 전체 Rust 10,243 PASS / 0 FAIL / 50 skip,
필수 Clippy 3종·workspace build·fmt·manifest 및 fresh WASM 실제 API 3/3를 통과했다.
[선별 후보 최종 결과](../pr_semanticist21_20261002_review_impl.md#선별-후보의-최종-rust-결과)에 source SHA와 명령을 연결한다.
원 head CI는 원 head의 근거이고 이 선별 후보의 GitHub CI로 대신하지 않는다.
승인된 선별 코드는 [통합 PR #7511](https://github.com/edwardkim/rhwp/pull/7511)로 병합했다. 원 author와 cherry-pick 출처를 유지했다.
[게시 head의 Full CI](https://github.com/edwardkim/rhwp/actions/runs/36955832835)와 CodeQL·Render Diff·Adapter·Proptest는 모두 성공했다.
관련 #7496은 devel 병합 후 자동 종료됐다. 원 PR은 통합 반영을 안내하고 중복 병합하지 않고 닫는다.
기여자의 fork branch는 보존한다. 후속 댓글에서 이 기록의 확정 commit과 merge SHA를 연결한다.
