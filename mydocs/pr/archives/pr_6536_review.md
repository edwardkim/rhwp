# PR #6536 검토 - page-anchored occupied block stored vpos

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 기준: `upstream/devel@336c4526e`
- 원 PR head: `8e4269db82cae5a45115f332c2fb80a467a45f32`
- 원 적용 commit: `b8041f23c`
- 메인터너 보정: `0ff2e25b6`, `7cf17c1ce`
- 상태: 메인터너 보정 후 통합 병합 완료
- 통합 PR / merge: #6541 / `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`

## 최초 차단 결함

원 patch는 빈 페이지를 제거했지만 `2.` 본문을 `연번` 표 뒤로 보냈다. 첫 보정
`0ff2e25b6`은 문서 순서를 복원했으나 표의 top만 검사해, 표 bottom과 `끝.` 문단의 bbox가 겹치는
결함을 남겼다. 따라서 기존의 “보정 완료” 판정은 철회했다.

## 최종 보정

`7cf17c1ce`는 양수 offset 빈 host 문단에서 다음 세 계약을 함께 지킨다.

1. 생성 본문은 host의 저장 line advance 뒤에서 시작한다.
2. 표의 저장 anchor는 유지한다.
3. 후속 문단은 실제 `lanes.max_bottom()`과 표 바깥쪽 아래 여백을 모두 소비한 뒤 시작한다.

회귀 테스트는 `body_bottom <= table_top`과 `table_bottom <= ending_top`을 고정하며, Hancom 2020
PDF의 대표 위치 `body 393.5`, `table 456.2`, `ending 646.9`를 각각 ±2 px 범위로 확인한다.

## 직접 시각 판정

- 입력: `samples/issue6535/36404612_page_anchored_footer_block.hwpx`, physical page 1
- Hancom 2020 PDF SHA-256:
  `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`
- [최종 Q2 비교 패널](../assets/pr_6541_issue6535_p1_q2_review.png):
  `2.` 본문 → `연번` 표 → `끝.` 순서와 비겹침을 양쪽에서 확인
- 패널 SHA-256:
  `e55a9ad21caf159f8c40d8061af776af30135056c4a82cdf044d91d0d9a4ada2`
- pixel diff `9.35%`, text missing `0/0`. 상단 그림 placeholder 차이는 이번 표 흐름 범위 밖의 기존 잔여다.

## 누적 검증과 결론

focused test, 전체 nextest 8,914건, 네 종류 global layout ratchet, Native Skia, Docker WASM이 모두
통과했다. #6535의 일곱 신고 문서 전체 해결을 뜻하지 않으므로 이슈는 자동 close하지 않는다.
이번 fixture의 양수 offset·빈 host·flow-with-text 표 계약에 한해 #6541 candidate에 수용한다.

## Merge 후 contributor PR comment 계획

- 원 head `8e4269db8` → 원 적용 `b8041f23c` → 메인터너 보정 `0ff2e25b6`,
  `7cf17c1ce` → 통합 merge `e9d2f8b25` 계보를 남긴다.
- Hancom 2020 p1의 `2.` 본문 → `연번` 표 → `끝.` 순서·비겹침과 최종 비교 패널을 알린다.
- #6535는 신고 7건 전체 해결이 아니므로 계속 open임을 명시한다.
- 계보 comment를 게시한 뒤 원 PR #6536을 중복 병합하지 않고 close한다.
