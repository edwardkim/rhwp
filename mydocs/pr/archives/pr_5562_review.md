# PR #5562 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5562>
- 작성자: `planet6897`
- 관련 이슈: #5557
- base / 원 head: `devel` / `706e2e34e38afa9f68ad42fe47e037c219c6ba26`
- 누적 적용: `efe22660d` (`review/planet6897-20260819`)
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWP3 표 헤더의 기본 셀 여백 35 hunit×4를 한글의 기본 여백 `510/510/141/141`로만
사상하고, 사용자 지정 여백은 기존 원값 변환을 유지한다. `issue_5557_hwp3_cell_margin`은
두 경로를 분리해 검사한다.

## 판정

기본값과 명시값의 경계를 테스트로 고정했으므로 누적 수용을 권고한다. 표 높이와 페이지네이션에
간접 영향이 있어 공통 CI가 최종 조건이다.
