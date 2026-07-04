# PR #1455 리뷰 - rhwp-studio 쪽 테두리 미리보기 버튼 토글 복구

- PR: https://github.com/edwardkim/rhwp/pull/1455
- 제목: `Task #1426: rhwp-studio 쪽 테두리 미리보기 버튼 토글 복구`
- 작성일: 2026-06-21
- 작성자: `postmelee`
- 관련 이슈: #1426 `rhwp-studio 쪽 테두리/배경 미리보기 버튼 토글 불가`
- base: `devel`
- head: `task_m100_1426`
- 처리 경로: collaborator self-merge 후보 예외 경로

## 1. 요약 판단

PR #1455는 #1426에서 보고된 쪽 테두리/배경 대화상자의 미리보기 버튼 토글 불가 문제를 수정한다.

핵심 수정은 `PageBorderDialog`의 방향별 테두리 상태를 `borderEdits[side].type !== 0` 기준으로
판정하고, 개별 방향 버튼과 전체 버튼을 상태 기반 토글로 바꾸는 것이다. `테두리 사용 안 함` 체크는
preview만 숨기는 상태가 아니라 네 방향 내부 상태를 모두 해제하는 조작으로 정의했다.

사용자 조작 동작이 직접 바뀌므로 E2E 회귀 테스트를 추가했다.

## 2. 최종 조작 규칙

| 조작 | 규칙 |
|------|------|
| 개별 방향 버튼 | 해당 방향이 켜져 있으면 해제, 꺼져 있으면 현재 선 속성으로 적용 |
| 전체 버튼 | 네 방향이 모두 켜져 있으면 전체 해제, 하나라도 꺼져 있으면 전체 적용 |
| `테두리 사용 안 함` 체크 | 네 방향 내부 상태를 모두 해제하고 preview 선을 숨김 |
| none 상태에서 개별 방향 클릭 | 클릭한 방향만 적용 |
| none 상태에서 전체 클릭 | 네 방향을 한 번에 적용 |
| `선 모양 바로 적용` | 현재 켜진 방향에만 선 속성 변경 반영 |

## 3. 변경 범위

| 파일 | 내용 |
|------|------|
| `rhwp-studio/src/ui/page-border-dialog.ts` | 개별/전체 버튼 토글, `테두리 사용 안 함` 체크 동기화와 내부 상태 초기화, 활성 방향 즉시 적용 |
| `rhwp-studio/e2e/page-border-toggle.test.mjs` | 쪽 테두리 preview 토글 및 none 상태 버튼 동작 회귀 테스트 |
| `mydocs/plans/task_m100_1426.md` | 수행계획서 |
| `mydocs/plans/task_m100_1426_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_1426_stage1.md` | 1단계 완료보고서 |
| `mydocs/working/task_m100_1426_stage2.md` | 2단계 완료보고서 |
| `mydocs/working/task_m100_1426_stage3.md` | 3단계 완료보고서 |
| `mydocs/report/task_m100_1426_report.md` | 최종 보고서 |
| `mydocs/orders/20260621.md` | 오늘할일 #1426 상태 기록 |
| `mydocs/pr/archives/pr_1455_review.md` | 본 PR review 문서 |
| `mydocs/pr/archives/pr_1455_review_impl.md` | 본 PR 처리 계획 |

## 4. 하이퍼-워터폴 준수 확인

| 항목 | 확인 |
|------|------|
| 이슈 | #1426 확인, issue는 merge 전까지 open 유지 |
| 브랜치 | 최신 `upstream/devel` 기반 `local/task1426` -> `upstream/task_m100_1426` |
| 오늘할일 | `mydocs/orders/20260621.md`에 #1426 기록 |
| 수행계획서 | `mydocs/plans/task_m100_1426.md` |
| 구현계획서 | `mydocs/plans/task_m100_1426_impl.md` |
| 단계별 보고서 | stage1, stage2, stage3 작성 |
| 최종 보고서 | `mydocs/report/task_m100_1426_report.md` |
| 단계별 커밋 | 계획, 구현, 회귀 테스트, 최종 보고, PR review 문서로 분리 |
| review 문서 | collaborator self-merge 후보 예외 경로에 따라 archive 경로에 포함 |

## 5. 커밋 구성

1. `Task #1426 Plan: Add execution and implementation plans`
2. `Task #1426 Stage 1: Implement page border state rules`
3. `Task #1426 Stage 2: Add regression coverage`
4. `Task #1426 Stage 3: Final report and daily order update`
5. `PR #1455 Review: Add self-merge review records`

## 6. 검증

| 명령 | 결과 |
|------|------|
| `cd rhwp-studio && node --check e2e/page-border-toggle.test.mjs` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/page-border-toggle.test.mjs --mode=headless` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/dialog-theme.test.mjs --mode=headless` | 통과 |
| `git diff --check` | 통과 |

`npm run build`에서 Vite chunk size 경고가 출력되었으나 빌드는 정상 완료되었다.

## 7. PR 상태값 기록 원칙

이 review 문서는 PR head에 포함되어 merge 후에도 보존된다. 따라서 `draft`, `mergeable`, `head SHA`,
`CI 상태`는 확정 사실처럼 기록하지 않는다.

merge 전에는 다음을 최신 상태로 확인한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR diff에 본 review 문서와 처리 계획서 포함
- merge 가능 상태
- 작업지시자 승인

## 8. 리스크

| 리스크 | 판단 |
|------|------|
| 대화상자 저장 모델 변경 | 낮음. 문서 모델/WASM bridge/serializer는 변경하지 않았다. |
| 꺼진 방향이 즉시 적용으로 다시 켜짐 | 낮음. 활성 방향만 갱신하도록 E2E로 검증했다. |
| `테두리 사용 안 함` 후 내부 상태 잔존 | 낮음. 체크 시 내부 방향 상태를 전체 해제하고 E2E로 검증했다. |
| 기존 다크 테마 preview 정책 회귀 | 낮음. `dialog-theme.test.mjs` 통과. |
| 오늘할일 rebase 충돌 | 해소됨. upstream #1328 항목과 #1426 항목을 모두 보존했다. |

## 9. 권고

GitHub Actions가 PR head 최신 커밋 기준으로 통과하고, 작업지시자가 승인하면 collaborator self-merge
후보로 merge 가능하다고 판단한다.

merge 후 #1426 close 여부를 확인하고, 자동 close가 되지 않으면 작업지시자 승인 후 수동 close한다.
