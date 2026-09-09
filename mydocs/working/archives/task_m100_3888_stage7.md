# Task M100 #3888 7단계 - CI impact worker-boundary 계약 보정

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 대상 head: `c7f02c813`
- 최초 CI: run `30825840258`

## 관측

최초 CI의 `Lint (fmt, clippy, WASM check)`에서 `Validate CI impact classifier`가 실패했다. 로컬에서
같은 명령을 실행한 결과 `scripts/tests/test_ci_impact_workflow.py`가 `WORKER_MARKER`인
`# [#2393] 기본 테스트 병렬화` 주석을 찾지 못해 preflight/worker 영역을 분리하지 못했다.

이번 6단계에서 기존 archive/worker block을 reusable workflow 호출로 교체하면서 해당 marker도 함께
삭제된 것이 원인이다. CI impact shadow는 여전히 관찰 전용이고 worker 조건은 shadow output을 소비하지
않는다는 계약 자체는 바꾸지 않았다.

## 구현 계획

1. 새 네 archive worker block의 첫 주석에 기존 `WORKER_MARKER` 문자열을 복원한다.
2. Python workflow contract test, actionlint, diff 검사를 다시 실행한다.
3. 보정 코드와 이 결과를 한 커밋으로 push해 최신 CI에서 Lint gate와 뒤이은 four archive 경로를 확인한다.

## 수용 기준

- `test_ci_impact_workflow.py`가 preflight와 worker 영역을 분리하고 통과한다.
- marker 뒤 영역에 `shadow_` output 참조가 없고 기존 fast-pass/frontend 조건 검증이 유지된다.
- 최신 PR CI가 새 head로 실행된다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` | 통과: 5건 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 통과: 20건 |
| `actionlint` 1.7.12 | 통과: caller와 두 reusable workflow 포함 |
| `git diff --check` | 통과 |

보정은 workflow worker boundary 주석 한 줄과 이 단계 기록만 추가한다. four archive planner/worker 계약은
변경하지 않았다.
