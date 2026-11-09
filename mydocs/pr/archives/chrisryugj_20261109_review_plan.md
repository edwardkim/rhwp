# chrisryugj HWP3 PR 2건 검토 계획 — 2026-11-09

기준 `upstream/devel` `b768bee1c4c0023087687f79a0a95f12108743e9`. 기존 contributor의 Ready PR [#8789](https://github.com/edwardkim/rhwp/pull/8789)·[#8791](https://github.com/edwardkim/rhwp/pull/8791)을 오래된 순서로 검토한다. 두 원 PR에 reviewer edwardkim을 지정하고 REST API로 재확인했다. closing issue는 없다. 상세 검토·검증의 로컬 범위이며 push·공개 댓글·새 PR·merge는 별도 승인 범위다.

## 경로와 원 head

- maintainer_general + intake_and_review + local_validation + multi_pr_update_branch + visual_fixture_evidence + post_merge. 기존 기여 기록 #8224·#8233을 확인했으며 첫 기여자 경로는 비해당이다.
- #8789 `3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9`: johab 공통 매핑 8종, 새 contract test 2건.
- #8791 `383f66acca8c966da9a198fd046c3da0ed94c3b6`: 공통 HWP3 표 격자 생성·이웃 탐색, 새 contract test 2건.
- 두 원 head의 CI는 성공, 현재 base와 단독 merge-tree는 모두 exit 0이다. 원 head는 별도 ref로 보존하고 누적 검토 branch의 merge commit으로 이력을 보존한다.

## 완료할 검토

1. #5860·#6380의 원 매핑 보정, #6057 반각 PUA의 실제 증거, #1692·#7642·#7814의 원 통합과 현재 source·독립 출력을 추적한다. 최신 devel의 문제 재현과 이미 해결된 범위를 분리한다.
2. 실제 HWP3 27-byte 셀 구조·좌표 및 40HU/2048 탐색 경계의 적용 근거·반례를 확인한다. 문자 코드 → IR 문자 → 공통 PUA 표시와 셀 geometry → IR row/col/span → 최종 배치 소비를 추적한다. 합성 parser contract와 한컴 Print 시각 증거를 구분한다.
3. 원본 SO-SUEOP, sample16, table-cell-order와 기존 한컴 변환본·원본의 올바른 Print PDF를 재사용한다. 같은 입력·글꼴·profile·DPI에서 base/후보 Native/fresh WASM 전쪽 TSV와 변경 영역의 직접 확대 판독을 수행한다. 양쪽 90%·쪽수·하락 기준을 적용하며 측정 전에 완료 판정을 하지 않는다.
4. 작은 구조·시각 대조를 먼저 수행하고 해당 focused·lint·전체/Native 적용 게이트를 완료한다. exact 원 head의 녹색 CI를 재사용할 때는 4.3.0 조건·생략 범위를 명시한다. maintainer source/test 보정이 필요하면 보정 범위를 먼저 제시하고 새 head에서 검증한다.
5. PR별 최종 판정·남은 범위·댓글 계획과 필요한 최소 TSV·대표 PNG·Markdown 기록을 보존한다. 생성 JSON·로그·원시 raster는 output의 진단 자료로 두고 커밋하지 않는다. 수용 가능한 후보의 실제 원격 처리안을 준비한 뒤 별도 승인을 요청한다.

제출된 두 Rust 테스트의 첫머리에는 현행 규칙의 Native/fresh WASM 실제 시각 수치·증거가 없으며, 기존 테스트 통과만으로 이를 보완한 것으로 기록하지 않는다.

## 원격 확정

2026-11-09 KST, 통합 [PR #8851](https://github.com/edwardkim/rhwp/pull/8851)의 head `9fa72c60365f6b1f3f168223e7ac56d2fa4b6402`에서 CI·self-review를 완료하고 merge `50b6098853afdb45625e5d505dc3b2070998fc46`으로 통합했다. 원 PR #8789·#8791은 모두 자동 MERGED로 확인했다. [통합 최종 검토](pr_8851_review.md)와 [원격 확정·후속 게이트](../assets/chrisryugj-20261109/remote-completion.md)를 따른다. 원 fork branch는 삭제하지 않는다.
