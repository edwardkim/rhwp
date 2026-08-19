# PR #5567 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5567>
- 작성자: `planet6897`
- 관련 이슈: #4898
- base / 원 head: `devel` / `cd05c27e4f559b6fbc5eca6d32b28f143db15d8d`
- 누적 적용: `e7e377eb9` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWPX→HWP5 변환의 micro-grid 휴리스틱이 셀 자신의 `apply_inner_margin=false`를 무시하고
`width_ref` bit 0을 세우던 문제를 제거한다. raw list extra의 물질화는 유지하고, 32열 표 회귀가
aim false/true와 raw extra 보존을 함께 검사한다.

## 판정

조건을 셀의 실제 속성으로 좁히고 회귀가 핵심 세 경우를 덮으므로 누적 수용을 권고한다.
한글 HWP5 페이지네이션 영향이 있으므로 공통 CI와 원 PR이 제시한 oracle 근거의 재확인이 최종 조건이다.
