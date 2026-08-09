---
kind: pr_review
status: accepted-awaiting-review-only-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4102 검토 - current-base 병합 뒤 review-only fast-pass

## 절차와 대상

~~~text
base route: collaborator self-merge
modifiers: intake_and_review, local_validation, review_only_fast_pass
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
~~~

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4102](https://github.com/edwardkim/rhwp/pull/4102) / @jangster77 |
| 관련 이슈 | [#4101](https://github.com/edwardkim/rhwp/issues/4101) |
| code head | `b38d715f76de12389133c12aeb24542f89f78e46` |
| base | `devel` / 문서 작성 시점 `e5e2ecc7c` |
| 규모 | 6개 파일, +266/-42 |
| 시각 검증 | 비대상. renderer, HWP/HWPX fixture, Studio UI 변경이 없다. |

## 문제와 보정

[#4073](https://github.com/edwardkim/rhwp/pull/4073)는 green code head `f10691230` 뒤에 current base
`d76d4e98`을 source에 병합한 `49d2511e9`, 그리고 review 기록 `a911a1110`을 추가했다. 기존 preflight는
commit 파일 목록을 먼저 보므로 `49d2511e9`를 새 code candidate로 오인했고, 그 SHA의 과거 workflow가 없어
전체 CI를 다시 실행했다.

PR #4102는 다음의 제한된 bridge만 재사용한다.

1. trailing single-parent review-only commit이 하나 이상 있어야 한다.
2. bridge는 정확히 두 부모이고 그중 하나만 current PR base SHA여야 한다.
3. 다른 부모와 current base의 `git merge-tree --write-tree` 결과가 실제 bridge merge tree와 같아야 한다.
4. 동일 PR·동일 source repository의 green CI/CodeQL candidate가 있어야 한다.

tree 검증은 `refs/pull/<번호>/head`를 읽기 전용으로 checkout할 뿐 source를 실행하지 않는다. tree 충돌·불일치,
입력 조회 실패, 다중 bridge, 일반 merge, candidate 실패·부재는 모두 `fast_pass=false`로 full CI를 유지한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| workflow fast-pass 계약 | `python3 scripts/tests/test_review_only_fast_pass_workflows.py`: 4 passed |
| CI 영향 workflow 계약 | `python3 scripts/tests/test_ci_impact_workflow.py`: 18 passed |
| CI 영향 분류기 | `node scripts/tests/ci-impact-classifier.test.cjs`: 27 passed |
| workflow 구문 | `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml` 통과 |
| 공백 오류 | `git diff --check` 통과 |
| #4073 실물 tree | `git merge-tree --write-tree f10691230 d76d4e98` 결과가 `49d2511e9`의 tree `e76548e3`와 일치 |

## GitHub Actions와 수용 판단

`b38d715f7`의 전체 [CI](https://github.com/edwardkim/rhwp/actions/runs/31087721202)와
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31087720657)은 성공했다. CI preflight와 CodeQL
preflight에서 bridge 검증 step은 일반 code PR이므로 각각 skip됐고, 이후 full lane의 Lint, Frontend package,
Native Skia, archive 3개, slow·일반 shard 3개, Build & Test aggregate, CodeQL 세 언어 분석이 모두 성공했다.

**수용.** 이 trailing docs-only commit을 push한 뒤 최신 head가 같은 candidate `b38d715f7`를 재사용해
preflight·aggregate를 성공시키고 heavy worker를 skip하는지, 그리고 `mergeable=MERGEABLE`·`mergeStateStatus=CLEAN`을
다시 확인한 뒤 작업지시자 승인에 따라 merge한다.
