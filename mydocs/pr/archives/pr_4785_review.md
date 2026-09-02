---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4785 검토 - Gym 능력 커버리지 측정기

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4785](https://github.com/edwardkim/rhwp/pull/4785) · @kevin9327 |
| 원 head | `56b3d860f56fddc34398c8cec66a91bfd2b43b99` |
| 통합 기준선 | `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` |
| 누적 적용 | `56b3d860` → `5765290ba` |
| 원 CI | Lint·Build & Test·CodeQL 성공 |

## 변경과 검토

`gym/tools/coverage.py`와 Python 계약 테스트로 pack/task metadata에서 능력 축의 빈 곳을 집계한다.
Rust·renderer 동작을 바꾸지 않으며, 이후 pack을 추가할 때 coverage와 profile 계약을 함께 점검하는
기준을 제공한다.

## 검증과 판단

- `test_gym_coverage.py`를 포함한 Gym Python 계약 58건 통과, 의도된 1건 skip.
- `git diff --check upstream/devel...HEAD`: 통과.

**통합 후보 수용.** 누적 branch에서 후속 pack들과 함께 검증했다.
