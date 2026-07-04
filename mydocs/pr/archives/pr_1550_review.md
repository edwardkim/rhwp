# PR #1550 처리 기록 — PR 리뷰 문서 전용 커밋 CI fast-pass 도입

- PR: https://github.com/edwardkim/rhwp/pull/1550
- 제목: `외부 contributor PR 리뷰 문서 전용 커밋의 CI fast-pass 추가`
- 작성자: `postmelee` (Taegyu Lee)
- 관련 이슈: #1547
- base/head: `edwardkim/rhwp:devel` <- `postmelee/rhwp:local/task1547`
- PR head SHA: `e492d0b8c4d3097c642413995c20b92a0d05a75d`
- merge commit: `d807870ead48e8b444765dd0de0ec155a58c45ee`
- 처리일: 2026-06-26

## 1. 처리 경로

작업지시자 지시에 따라 기존 PR 리뷰 문서 선작성 규칙은 이 PR 처리에 적용하지 않고,
PR #1550이 도입한 `mydocs/manual/pr_review_workflow.md` 9.3.1/9.4 규칙을 먼저 merge한 뒤
사후 처리 기록을 남긴다.

PR #1550의 새 규칙은 collaborator-mediated 외부 PR에서 review 문서 전용 후속 커밋이
`mydocs/pr/**`와 `mydocs/orders/*.md`만 변경하고, 직전 코드 검증 대상 SHA의 relevant check가
green일 때 heavy CI 결과를 재사용할 수 있음을 문서화한다.

이 기록은 #1550 merge 이후 작성한 사후 보관용 기록이며, merge 완료 사실과 issue 상태의 원천 기록은
GitHub PR/Issue metadata로 둔다.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `.github/workflows/ci.yml` | `CI preflight` 추가, review 문서 전용 후속 커밋에서 `Build & Test` fast-pass 판정 |
| `.github/workflows/codeql.yml` | `CodeQL preflight` 추가, CodeQL matrix check 3종 green 재사용 판정 |
| `.github/workflows/render-diff.yml` | `Render Diff preflight` 추가, `Canvas visual diff` green 재사용 판정 |
| `mydocs/manual/pr_review_workflow.md` | collaborator-mediated 외부 PR 경로에 9.3.1 fast-pass 규칙 추가 |
| `mydocs/orders/20260626.md` | #1547 작업 완료 상태 기록 |
| 작업 문서 | #1547 계획서, 단계별 보고서, 최종 보고서 추가 |

## 3. 검증 결과

PR #1550은 workflow 파일 변경이 포함되어 fast-pass 대상이 아니었고, 최신 PR head 기준 full CI가 실행됐다.

| 체크 | 결과 |
|---|---|
| `CI preflight` | pass |
| `Build & Test` | pass |
| `CodeQL preflight` | pass |
| `Analyze (javascript-typescript)` | pass |
| `Analyze (python)` | pass |
| `Analyze (rust)` | pass |
| `CodeQL` | pass |
| `Render Diff preflight` | pass |
| `Canvas visual diff` | pass |
| `WASM Build` | skipped |

로컬 후속 확인:

| 명령 | 결과 |
|---|---|
| `git fetch upstream` | `upstream/devel`이 merge commit `d807870e`로 갱신됨 |
| `git merge --ff-only upstream/devel` | 로컬 `devel` fast-forward 완료 |
| `gh issue view 1547 --json state` | `OPEN` |

## 4. 후속 상태

- PR #1550은 2026-06-26 22:15:43 KST에 merge됐다.
- PR 본문은 `관련 이슈: #1547`만 포함하고 `Closes #1547` 형식이 아니므로 #1547은 자동 close되지 않았다.
- #1547 수동 close와 PR/Issue 코멘트는 작업지시자 별도 승인 후 처리한다.
- PR #1550이 도입한 fast-pass의 실제 review 문서 전용 후속 커밋 E2E 검증은 별도 PR에서 확인해야 한다.

## 5. fast-pass 후속 검증

- #1574/#1576 보정 후 review 문서 전용 PR에서 preflight 생성과 heavy job skip 여부를 재확인한다.
- 최종 확인은 base SHA의 CI/CodeQL push check가 green인 상태에서 문서 전용 후속 커밋으로 수행한다.
