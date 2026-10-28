# Task M100 #3888 8단계 - reusable workflow action pin 완전성 보정

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 대상 head: `e26648fe0`
- 최초 CI: run `30826138309`

## 관측

7단계 보정 후 Lint와 Frontend gate는 통과했다. 이어서 시작한 slow, `1`·`2`, `3` builder는 모두
`Set up job`에서 실패했다. 완료된 job log는 새 reusable workflow의
`actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a`가 짧은 SHA라서 GitHub가 action을
resolve하지 못했다고 기록한다. 올바른 pin은 기존 workflow와 같은
`043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`다.

이 실패는 action 실행 전 발생했으므로 planner, archive 생성, worker 실행 결과는 아직 판단할 수 없다.

## 구현 계획

1. build/run reusable workflow의 모든 `actions/upload-artifact` pin을 40자리 SHA로 복원한다.
2. 세 reusable workflow와 caller에서 사용하는 third-party action pin의 SHA 길이를 정적으로 검사한다.
3. actionlint, CI impact test, diff 검사를 다시 실행한다.
4. 보정 코드와 이 기록을 함께 push하고, 최신 CI에서 실제 four archive builder/worker를 확인한다.

## 수용 기준

- 새 reusable workflow의 모든 `uses: owner/repo@<SHA>` pin이 40자리 SHA다.
- builder가 `Set up job`을 넘어 archive 생성 단계로 진입한다.
- 최신 PR CI가 slow, `1`, `2`, `3` archive만 생성한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| workflow action SHA 길이 검사 | 통과: caller와 두 reusable workflow의 hexadecimal action pin은 모두 40자리 |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` | 통과: 5건 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 통과: 20건 |
| `actionlint` 1.7.12 | 통과 |
| `git diff --check` | 통과 |

이 보정은 새 reusable workflow의 six `actions/upload-artifact` 참조에만 적용했다. planner와 archive
분배·worker 수·집계 계약은 변경하지 않았다.
