---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6475
author: jeong-sik
---

# PR #6475 review - 셀 줄나눔 계측기 왜곡 제거

## 검토 기준

- 원 PR head: `8ee739a03a285a586e2d762660e60c04e17c58aa`
- 통합 적용 commit: `c0aedcd32`
- 기준 base: `upstream/devel@19b89d967b1505cd4bdcdbba7d1f1413f32a1505`
- 작성 시점 원 PR은 Open/non-draft, `MERGEABLE/CLEAN`이고 최신 head의 Build & Test와 CodeQL을 포함한
필수 check가 성공했다. 최종 통합 PR 직전에 다시 확인한다.

## 검토와 검증

- multiline cell dump에서 닫는 quote까지 text를 복원하고, 빈 content key cell을 normal pairing에서
  제외해 `emptyCells`로 분류한다. baseline은 이 분류 변화를 반영한다.
- `node --test scripts/tests/cell-lineseg-agreement.test.mjs`를 실행해 `21 passed`를 확인했다.
- 변경 범위는 measurement script, baseline, Node test뿐이며 renderer/runtime HWP fixture를 바꾸지 않으므로
visual sweep 대상은 아니다.

## 판단

**수용 권고.** multiline parsing과 empty-key 처리의 두 회귀 경계가 독립 test로 고정돼 있다. 통합 branch의
최종 head Full CI와 mergeability 통과가 merge 전 조건이다.
