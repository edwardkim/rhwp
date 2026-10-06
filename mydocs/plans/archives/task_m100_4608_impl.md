# Task M100 #4608 — bounded polling 구현 계획

## 변경 파일

### `.github/workflows/cancel-stale-pr-runs.yml`

- 완료 상태 확인 간격을 `[0, 500, 1_000, 2_000]`ms로 제한한다.
- `getWorkflowRun(runId)` helper로 상태 API 호출을 한 곳에 모은다.
- `waitForCompletedRun(runId)`은 즉시 1회와 세 번의 지연 조회 중 `completed`를 만나면 해당 run을
  반환하고, 끝까지 active이면 `null`을 반환한다.
- `forceCancelWithRetry(run.id)` 오류를 받으면 bounded polling을 수행한다.
- polling 중 완료 run을 확인한 경우에만 기존 정상 경과 로그를 남긴다.
- polling이 `null`을 반환하거나 상태 API 자체가 실패하면 원래 force-cancel 오류를 전파한다.
- 기존 `502/503/504` POST 재시도, 취소 직전 live PR head 재확인과 대상 식별 조건은 그대로 둔다.

### `scripts/tests/test_cancel_stale_pr_runs_workflow.py`

- polling delay가 유한한 네 번의 조회로 고정됐는지 확인한다.
- 상태 조회가 sleep 뒤 반복되고 `completed`만 성공 반환하는 순서를 확인한다.
- helper가 한도 뒤 `null`을 반환하며 호출부가 이를 원래 오류로 전파하는지 확인한다.
- 완료 확인 뒤에만 정상 경과 로그가 나오는 기존 계약을 유지한다.

### 문서

- 단계별 완료 문서에는 실패 run, stale 대상 run, 로컬 검증 결과를 기록한다.
- 최종 보고서에는 `devel` PR 검증과 별개로 `pull_request_target`이 `main` workflow를 사용한다는 배포
  경계를 명시한다.

## 검증 명령

```bash
python3 -m unittest \
  scripts/tests/test_cancel_stale_pr_runs_workflow.py \
  scripts/tests/test_workflow_contract_wiring.py

python3 -m unittest \
  scripts/tests/test_ci_impact_workflow.py \
  scripts/tests/test_ci_impact_policy_workflow.py

node --test \
  scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs

cargo fmt --all
cargo fmt --all -- --check
git diff --check
```

제품 소스·Rust·Studio·renderer·fixture를 바꾸지 않으므로 Cargo 전체 회귀, WASM build와 시각 검증은
실행하지 않는다. 실제 권한·event 경계는 PR 최신 head의 Actions와 fork `synchronize` run에서 판정한다.
