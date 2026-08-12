---
kind: pr-review
status: pending-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4587 리뷰 - CodeQL neutral 요약의 review-only 재사용

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4587](https://github.com/edwardkim/rhwp/pull/4587) |
| 관련 이슈 | [#4585](https://github.com/edwardkim/rhwp/issues/4585) |
| base | `devel` / `d30e5d4afaaed42ed664f248dd17eed33cca3ae0` |
| code candidate | `725568649ee231dc87dd98f3906735878296e166` |
| trailing docs candidate | 이 문서와 오늘할일 commit |
| 변경 범위 | `.github/workflows/codeql.yml`, workflow 계약 테스트, 외부 PR 통합 검토 절차 |

## 변경 판단

review-only 문서 commit 뒤의 CodeQL preflight는 원 code candidate의 language Analyze job이 모두
허용 결론이어도, GitHub Advanced Security의 집계 `CodeQL` check가 `neutral`이면 재사용을 거부했다.
실제 [PR #4525](https://github.com/edwardkim/rhwp/pull/4525)의 trailing docs head에서
`security-check-not-green:CodeQL:neutral` 때문에 Rust CodeQL이 다시 실행됐다.

이 PR은 language-level Analyze 결과와 집계 summary의 역할을 분리한다. candidate SHA, PR identity,
실행 시각, 세 language Analyze check는 기존처럼 모두 확인한다. 그 뒤 GHAS summary는 `success` 또는
`neutral`만 허용한다. `skipped`, 누락, 대기, 실패, 취소 결론은 full CodeQL로 fail-closed 한다.

또한 외부 PR 통합 검토의 기본 작업공간을 최신 `devel` 위의 가시성 review branch로 명시했다. 사용자가
열어 둔 VS Code 작업공간에서 contributor commit을 cherry-pick하고 메인터너 보정을 같은 그래프로 볼 수
있으며, 별도 worktree는 주 작업공간이 dirty이거나 작업지시자가 격리를 요구한 경우에만 사용한다.

## 완료한 검증

- `python3 -m unittest scripts.tests.test_codeql_workflow scripts.tests.test_review_only_fast_pass_workflows`:
  23건 통과. 실행형 preflight harness에서 `neutral` summary 재사용과 `skipped` summary 거부를 확인했다.
- `python3 scripts/check_markdown_links.py mydocs/manual/pr_review/collaborator_external_pr.md`:
  내부 Markdown 상대 링크 이상 없음.
- `git diff --check`: 통과.
- code candidate `725568649`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/31483194140):
  Build & Test, Native Skia, lint, 모든 default-feature shard 성공.
- 같은 candidate의 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31483193889): 세 language
  Analyze job 성공. GHAS `CodeQL` summary는 예상대로 `neutral`이며, 이 PR의 허용 대상이다.

renderer·layout·sample·WASM 구현은 바꾸지 않았으므로 Render Diff와 별도 시각 검증은 적용 대상이 아니다.

## 최종 권고

**수용.** 이 trailing docs-only head의 CI·CodeQL preflight가 `725568649`를 재사용하고 heavy worker와
Analyze job을 skip하면 merge 가능하다. summary의 `neutral`만으로 재사용하지 않고, 개별 Analyze job과
candidate identity를 계속 검증하므로 security failure를 성공으로 바꾸지 않는다.
