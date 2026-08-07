---
kind: pr_review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4173 검토 - mydocs 충돌 해소 병합의 review-only fast-pass 재사용

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4173](https://github.com/edwardkim/rhwp/pull/4173) / @jangster77 |
| 연동 이슈 | [#4172](https://github.com/edwardkim/rhwp/issues/4172) |
| 기준 `devel` | `23ff5b6f1acd67aaccdf1ecfb47a7dc1918a13ed` |
| code head | `b1ecf57e9e63a0032f25452aebbca9230ab11e23` |
| 변경 규모 | 9 files, +531 / -80 |
| 변경 영역 | CI, CodeQL, Render Diff preflight와 review-only fast-pass 문서·계약 테스트 |

이 PR은 `mydocs/` 안의 오늘할일 등만 수동 충돌 해소한 current-base merge를 review-only bridge로
재사용한다. source, test, workflow, sample, PDF, golden, baseline의 충돌 해소나 복수 current-base
merge는 기존과 같이 full CI로 fallback한다. renderer·layout·문서 변환 동작 자체는 바꾸지 않는다.

## 검토 근거

[#4136](https://github.com/edwardkim/rhwp/pull/4136)의 마지막 merge
`ecda0c15e32168dbb21223e7f2446eb9df491455`를 실제로 검사했다. source parent는
`0762ad241bb21bc720188938fa0d3451bd3d0caa`, current-base parent는
`9dbd3dc6c49c36e8d1012a19ec60dea1abd5123c`였으며, `git show --remerge-diff`의 수동 해소 경로는
`mydocs/orders/20260807.md` 하나였다.

새 검사기는 다음을 모두 확인한다.

1. 대상이 정확히 두 parent를 가진 merge이고 current base SHA가 parent 하나와 일치한다.
2. 자동 3-way merge tree가 일치하면 기존 경로를 유지한다.
3. 자동 병합 충돌이면 remerge diff의 경로가 비어 있지 않고 모두 `mydocs/` 아래인지 확인한다.
4. 검사기는 PR head가 아니라 current base tree에서 읽어 실행한다.

따라서 PR source의 검사기 변경으로 판정을 우회하지 않으며, 검사기를 읽을 수 없거나 경로가 하나라도
허용 범위를 벗어나면 fast-pass를 선택하지 않는다.

## 최종 보정

최초 code head 뒤의 review 기록 commit `8930b89b3`은 CI preflight에서 review-only fast-pass를
선택했지만, Render Diff는 code candidate 결과를 찾지 못해 Canvas 전체 경로를 실행했다. 원인은
Render Diff의 GitHub Script에서 candidate SHA를 찾은 직후 `break`한 뒤의 결과 조회 코드가 같은
반복문 안에 남아 도달할 수 없었던 제어 흐름 오류였다.

최종 code head `b1ecf57e9`은 결과 조회를 반복문 밖으로 옮겼다. 또한 계약 테스트는 후보 조회가
반복문 밖에 있는지 확인하고, 추출한 Render Diff GitHub Script를 Node 문법 검사로 실행해 같은
배치 오류를 다시 잡도록 보강했다. 이 보정은 review-only 후보 판정 범위를 넓히지 않는다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| #4136 실제 merge 검사 | `current-base-merge-resolution-mydocs-only` |
| 허용·거부 계약 | `scripts/tests/test_review_only_fast_pass_workflows.py` 포함 34 passed |
| CI workflow 계약 | cache sweep·workflow wiring 테스트 통과 |
| workflow 문법 | `actionlint`로 CI, CodeQL, Render Diff 통과 |
| 변경 정합 | `git diff --check` 통과 |
| GitHub CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/31184307222) 전체 성공 |
| GitHub 보조 workflow | [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31184306396), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31184306339) 성공 |

이 PR 자체의 current base에는 새 검사기가 아직 없으므로 최종 code head는 의도대로 full CI가 실행됐다.
이 문서와 오늘할일은 그 성공 head 뒤의 single-parent review-only commit이며, push 뒤 최신 head의
CI·CodeQL·Render Diff preflight와 aggregate가 fast-pass로 성공하는지 다시 확인한다.

## 수용 판단

**수용.** 허용 범위는 `mydocs/` 수동 해소로 한정되고, 자동 merge tree가 일치하지 않는 기존 거부 경계는
완화하지 않았다. 문서 trailing commit의 fast-pass와 최신 merge 가능 상태를 확인하면 작업지시자 지시에 따라
병합한다.
