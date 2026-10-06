# Task M100 #4608 Stage 1 — stale run 완료 상태 bounded polling

- **Issue**: [#4608](https://github.com/edwardkim/rhwp/issues/4608)
- **기준**: `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6`
- **재현**: [PR #6116 cleanup run](https://github.com/edwardkim/rhwp/actions/runs/32923641712)

## 변경

- `force-cancel` 오류 뒤 `[0, 500, 1_000, 2_000]`ms 간격으로 최대 네 번 대상 run 상태를 확인한다.
- `completed`가 확인된 경우에만 eventual-consistency race로 정상 처리한다.
- 3.5초 window 뒤에도 active이면 원래 force-cancel 오류를 전파한다.
- 상태 API 자체가 실패해도 원래 force-cancel 오류를 보존한다.
- 기존 `502/503/504` POST 재시도와 live PR head·fork branch 식별 계약은 변경하지 않았다.

## 테스트 우선 근거

새 bounded polling 계약 테스트를 먼저 작성한 뒤 기존 workflow에서 실행했다. polling 상수와 helper가 없어
2건이 실패했고, workflow 구현 뒤 같은 묶음 7건이 모두 통과했다.

## 검증

```text
$ python3 -m unittest \
    scripts/tests/test_cancel_stale_pr_runs_workflow.py \
    scripts/tests/test_workflow_contract_wiring.py
Ran 7 tests — OK

$ python3 -m unittest \
    scripts/tests/test_ci_impact_workflow.py \
    scripts/tests/test_ci_impact_policy_workflow.py
Ran 42 tests — OK

$ node --test \
    scripts/tests/ci-impact-classifier.test.cjs \
    scripts/tests/ci-impact-policy.test.cjs
65 pass, 0 fail

$ git diff --check
exit 0
```

## 안전 경계

`pull_request_target`은 계속 PR source를 checkout하거나 실행하지 않고 GitHub API만 사용한다. trigger,
permission, concurrency group, 최신 head 보호와 stale run 식별식은 그대로다. polling은 취소 오류가 발생한
stale run의 읽기 전용 상태 조회에만 추가된다.
