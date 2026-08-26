# PR #5544 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5544>
- 작성자: `planet6897`
- 관련 이슈: #5543
- base / 원 head: `devel` / `3bd5b349291c5829be26dc62fd7ceebc3d83e963`
- 누적 적용: `42924f94e` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

`src/renderer/typeset.rs`에서 자리차지 스택의 이월 앵커 사다리 계상을 `--compat 2024`
경로에 맞추고, 실물 HWPX fixture와 `issue_5543_carried_anchor_ladder` 회귀를 추가한다.
기본값 247건 코호트 지문 diff 0이라는 원 PR의 범위 설명과 맞고, 관련 조판 계산 외 변경은 없다.

## 판정

누적 수용을 권고한다. 다만 typeset과 신규 HWPX fixture가 사용자 표시 결과를 바꾸므로, 통합 PR에는
기준 PDF 또는 동등한 시각 증적이 필요하다. 공통 검증의 미해결 항목과 최신 CI가 통과하기 전에는
최종 merge를 승인하지 않는다.
