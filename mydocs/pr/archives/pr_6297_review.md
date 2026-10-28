---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
pr: 6297
issue: 6243
---

# PR #6297 self-review — devel merge bridge 뒤 Render Diff 재사용

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
current head: 0c837f36e57b33ce2aa1cf62caf86b1eff0b944c (self-review 작성 전 code candidate)
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#6297](https://github.com/edwardkim/rhwp/pull/6297) |
| 관련 이슈 | [#6243](https://github.com/edwardkim/rhwp/issues/6243), OPEN, assignee `edwardkim` |
| 작성자·reviewer | `edwardkim` collaborator self-review, 외부 reviewer 미지정 |
| 대상 / head | `devel` ← `task_m100_6243` |
| code candidate | `0c837f36e57b33ce2aa1cf62caf86b1eff0b944c` |
| Full CI 기준 base | `94ff48d2b81dee5241110db9d2417dffbfb7f9ec` |
| 규모 | 5 files, +572 / -2, 7 commits |
| code candidate 상태 | Open, non-draft, `MERGEABLE / CLEAN`, Full required checks 성공 |

이 문서는 작성자 자체 검토 기록이며 GitHub approval이나 merge 권한 행사를 뜻하지 않는다. #6243은 이 PR
병합만으로 닫지 않고, 정상 release로 `main`의 trusted controller가 활성화된 뒤 후행 canary까지 관찰한다.

## 문제와 변경 범위

PR #6214의 `code candidate -> current-base merge bridge -> review-only tail` 계보에서 Render Diff는 bridge를
검증하고도 일반 candidate loop가 `allowPriorPrBase=false`로 조회해 과거 base의 녹색 Canvas identity를
`canvas-visual-diff-identity-mismatch`로 거부했다. 그 결과 안전성 문제 없이 Canvas worker가 5분 24초
중복 실행됐다.

이 PR은 candidate loop 호출 한 곳을
`renderDiffResult(candidateSha, pr, Boolean(baseMergeBridge))`로 정정한다. prior-base identity 조회는 이미
current base와 연결되는 bridge가 정확히 하나 검증된 경우에만 열린다. 조회 뒤의
`pending-base-merge-tree`, 자동 merge-tree 일치 또는 `mydocs/` 한정 수동 충돌 해소 검증은 바꾸지 않았다.

제품 Rust, renderer, WASM, Studio, fixture, golden, event, permission과 required
`Render Diff / Canvas visual diff` check 이름은 변경하지 않았다. 실행 가능한 workflow harness와
bridge 부재·복수 bridge·wrong PR·branch·repository·PR 생성 전 run·missing·pending·failed run·identity
step 실패의 fail-closed 계약, 수행계획과 운영 기록만 함께 추가했다.

## 로컬 검증과 원인 고정

- O2-1 red 계약에서 기존·음성 20건은 통과하고 #6214 긍정 계보만 기대
  `pending-base-merge-tree` 대신 `false / canvas-visual-diff-identity-mismatch`를 반환해 원인을 실행
  수준으로 고정했다.
- 한 호출을 정정한 O2-2에서 Render Diff Python 계약 21/21과 핵심 긍정·음성 계약 3/3이 통과했다.
- `python3 -m unittest scripts.tests.test_ci_impact_workflow` 31/31과
  `node --test scripts/tests/ci-impact-policy.test.cjs` 32/32가 통과했다. Node 계약은 CI·CodeQL·Render
  Diff trigger 대칭성과 required aggregate 경계도 함께 확인했다.
- `python3 -m py_compile scripts/tests/test_review_only_fast_pass_workflows.py`, `cargo fmt --all`,
  `cargo fmt --all -- --check`, 변경 문서 3개의 Markdown 상대 링크 검사와 `git diff --check`가 통과했다.
- PR 생성 직전 최신 `devel@94ff48d2b`를 merge한 `0c837f36e`에서 위 계약 84건과 형식 검사를 다시
  통과했다. Cargo 전체 회귀, Clippy, Native Skia, WASM, Studio, 시각 검증은 제품·Rust·렌더러 변경이
  없는 workflow 계약 수정이므로 로컬에서 중복 실행하지 않았다.

## code candidate Full GitHub Actions

exact code candidate `0c837f36e57b33ce2aa1cf62caf86b1eff0b944c`에서 workflow 변경 PR의 Full 실행을
확인했다.

- [CI run 33161417940](https://github.com/edwardkim/rhwp/actions/runs/33161417940): preflight, Lint,
  Frontend package, Native Skia, archive builders·네 test shard와 required `Build & Test` 성공.
- [CodeQL run 33161417936](https://github.com/edwardkim/rhwp/actions/runs/33161417936): Rust,
  JavaScript/TypeScript, Python Analyze 성공. 같은 candidate의 GHAS `CodeQL` check도 성공.
- [Render Diff run 33161417753](https://github.com/edwardkim/rhwp/actions/runs/33161417753): preflight와
  Canvas visual diff 성공. 이 Full 실행은 workflow 변경 candidate의 정상 검증이며 post-release canary가 아니다.
- [Adapter inter-diff run 33161417946](https://github.com/edwardkim/rhwp/actions/runs/33161417946)와
  [Proptest roundtrip run 33161417989](https://github.com/edwardkim/rhwp/actions/runs/33161417989) 성공.

## 최신 devel과 trailing review 경계

self-review 작성 직전 `upstream/devel`은 `b1485e0a143dc74a0ba462a1b34f8c03b0306ddd`까지 29커밋
전진했다. 이 구간은 `.github/workflows/render-diff.yml`과
`scripts/tests/test_review_only_fast_pass_workflows.py`를 바꾸지 않았다. merge-tree 대사에서도 양쪽
오늘할일의 서로 다른 섹션이 자동 보존되고 source·test·workflow 충돌은 없었다.

review-only 기록만을 위해 최신 base를 source branch에 merge하지 않는 절차를 따른다. 이 PR은 CI 실행 정책
자체를 바꾸고 기본 브랜치 `main`에는 trusted controller가 아직 활성화되지 않았으므로, 이 문서와 오늘할일의
single-parent trailing commit은 fast-pass가 아니라 Full fallback하는 것이 정상이다. 최신 trailing head의
Full required checks가 현재 base merge ref에서 성공해야 merge 후보가 된다.

## 위험과 보호 불변식

- bridge가 없거나 둘 이상이면 prior-base lookup을 열지 않는다.
- 같은 PR, head branch, source repository와 PR 생성 이후 성공한 Full Render Diff identity만 후보로 삼는다.
- identity 조회 성공은 최종 fast-pass가 아니다. 기존 merge-tree 검증을 통과하기 전에는 pending 상태를
  유지한다.
- missing·pending·failed 최신 완료 후보, wrong identity와 허용되지 않은 merge·conflict path는 Full로
  닫힌다.
- rollback은 workflow 호출 1줄과 그 계약 commit을 revert해 모든 해당 입력을 기존 Full fallback으로
  되돌리는 방식이다.

## 판정

**code candidate는 self-review trailing 제출에 적합하다.** 원인 재현, 최소 정정, 음성 matrix, 비례 로컬
검증과 exact candidate의 Full GitHub Actions에서 차단 결함을 발견하지 못했다. 최종 merge 권고는 최신
trailing head의 Full CI·CodeQL·Render Diff·required aggregate 성공, `MERGEABLE / CLEAN`, 메인테이너의
별도 merge 승인을 모두 확인한 뒤 확정한다. #6243은 post-release canary 전까지 OPEN으로 유지한다.
