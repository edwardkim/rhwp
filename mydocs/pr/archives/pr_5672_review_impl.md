---
kind: pr-review-implementation
status: merged
pr: 5672
issue: 5671
merged_at: 2026-08-19T17:54:04Z
---

# PR #5672 메인터너 보정 기록

## 계보

- contributor source head: `d81d69b`
- maintainer correction head: `47b2fab503c3ce6d74528c09dbfefd0710c00afe`
- merged squash commit: `f9616a95fdffb917e9a5a74d4cdf4f4ad774b32e`

## 보정 내용

- `caption-tables`의 결과를 caption이 있는 Table Control로 한정하고, 구조 요약과 caption 문단 수를
  직렬화한다.
- `ctrl-kinds`의 kind 이름을 Control enum 전체에 대해 명시하고 문서 전체에서 count를 집계한다.
- `page-starts-on`에서 model의 page start 값을 JSON 계약 값으로 직접 직렬화한다.
- 50개 query command의 help, JSON envelope, volume-probe 경로를 integration contract로 고정한다.

## 검증 및 반영 방식

- 보정은 contributor source branch에 direct push하여 원 PR #5672의 최신 head로 검증했다.
- 최신 CI와 전체 release-test 회귀가 성공한 뒤 해당 PR을 squash merge했다.
- 원 코드 PR을 다시 변경하지 않도록 이 문서와 review 기록은 merge 뒤 별도 review-only PR로 남긴다.

