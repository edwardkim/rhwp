# PR #5560 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5560>
- 작성자: `planet6897`
- 관련 이슈: #5554
- base / 원 head: `devel` / `b3d85b4f219522f104edb80fa7cc66ba38621b2d`
- 누적 적용: `04220d147` (`review/planet6897-20260819`)
- 메인터너 회귀: `cd95d22f5`
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWP3 문단 모양의 양쪽 정렬을 `attr1` bit 7(`breakNonLatinWord`)으로 유도한다.
정렬 이외는 bit 7을 비운다. 원 PR에는 source-side `#[cfg(test)]` 추가가 없으며, 메인터너가
`tests/cases/issue_5554_hwp3_break_non_latin_word.rs`에 통합 회귀를 별도 추가했다.

## 판정

테스트 위치 규약을 지킨 상태로 누적 수용을 권고한다. HWP3 조판과 HWPX 저장 양쪽에 영향을 주므로
공통 CI 및 fixture 기반 회귀 통과 전 최종 merge는 보류한다.
