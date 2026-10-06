---
kind: working
status: completed
issue: 4176
last_verified: 2026-08-07
---

# Task #4176 Stage 1 - source parent 재사용 보정

## 구현

- 최신 current-base merge의 source parent를 먼저 녹색 candidate로 조회하도록 세 preflight를 보정했다.
- CI 실행 경로를 수정한 source PR은 full CI로 고정했다.
- Render Diff는 이 좁은 경로에서만 같은 PR의 prior-base identity를 수용한다.

## 검증 결과

- `python3 -m unittest scripts/tests/test_review_only_fast_pass_workflows.py`
  `scripts/tests/test_cache_sweep_workflow.py` `scripts/tests/test_workflow_contract_wiring.py`를 실행해
  36건이 통과했다.
- `actionlint`로 CI, CodeQL, Render Diff workflow를 검사했고, `git diff --check`도 통과했다.
- trusted merge-resolution checker는 #4136 `bed52d02`와 #4165 `d2621a6` 모두에서
  `current-base-merge-resolution-mydocs-only`를 반환했다.
- #4136 source head `ecda0c15`와 #4165 source head `fd792a0`의 CI·CodeQL은 모두 녹색이었다.
- #4136·#4165 PR diff에는 CI 실행 경로가 없고, #4170에는 `.github/workflows/ci.yml`과
  `scripts/ci-impact-classifier.cjs`가 있어 guard 대상임을 확인했다.
