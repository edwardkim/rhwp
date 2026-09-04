# Task M100 #1426 최종 보고서 — rhwp-studio 쪽 테두리/배경 미리보기 버튼 토글 복구

- 이슈: #1426
- 브랜치: `local/task1426`
- 작성일: 2026-06-21

## 1. 문제

`rhwp-studio`의 `쪽 > 쪽 테두리/배경` 대화상자에서 테두리 탭 미리보기 주변의
방향 버튼과 전체 버튼이 토글로 동작하지 않고, 항상 현재 선 속성을 적용하는 켜기 경로만 실행됐다.

또한 `선 모양 바로 적용`이 켜진 상태에서 선 종류/굵기/색을 바꾸면 꺼져 있던 방향까지
사방 전체가 켜질 수 있었다.

`테두리 사용 안 함` 체크박스도 preview만 숨기고 내부 방향 상태를 해제하지 않으면,
이후 전체/개별 버튼 클릭이 한 박자 늦거나 기존 전체 상태에서 일부만 제거된 것처럼 보일 수 있다.

## 2. 원인

`rhwp-studio/src/ui/page-border-dialog.ts`의 `sideButton()` 클릭 핸들러가 항상
`applyToSides()`만 호출했다. `applyToSides()`는 현재 선 속성을 대상 방향에 덮어쓰는 함수라,
이미 켜진 방향을 `noneBorder()`로 되돌리는 해제 경로가 없었다.

선 속성 변경 핸들러도 `['Left', 'Right', 'Top', 'Bottom']` 전체에 `applyToSides()`를 호출해,
활성 방향과 비활성 방향을 구분하지 못했다.

`테두리 사용 안 함` 체크 변경 핸들러가 `updateBorderPreview()`만 호출하고
`borderEdits`를 초기화하지 않아 발생했다.

## 3. 변경 내용

| 항목 | 변경 |
|------|------|
| 방향 상태 기준 | `borderEdits[side].type !== 0`을 켜짐 상태로 사용 |
| 개별 버튼 | 켜져 있으면 `noneBorder()`로 해제, 꺼져 있으면 `currentBorder()` 적용 |
| 전체 버튼 | 사방 모두 켜져 있으면 전체 해제, 일부라도 꺼져 있으면 전체 적용 |
| none 체크박스 | 활성 방향이 없으면 `테두리 사용 안 함` 체크, 하나라도 있으면 해제 |
| none 체크 동작 | 체크 시 내부 방향 상태도 모두 `noneBorder()`로 초기화 |
| 선 모양 바로 적용 | 현재 켜진 방향에만 선 속성 변경 적용 |
| 회귀 테스트 | `page-border-toggle.test.mjs` 신규 추가 및 `테두리 사용 안 함` 후 버튼 클릭 시나리오 보강 |

## 4. 최종 조작 규칙

| 조작 | 규칙 |
|------|------|
| 개별 방향 버튼 | 해당 방향이 켜져 있으면 해제, 꺼져 있으면 현재 선 속성으로 적용 |
| 전체 버튼 | 네 방향이 모두 켜져 있으면 전체 해제, 하나라도 꺼져 있으면 전체 적용 |
| `테두리 사용 안 함` 체크 | 내부 방향 상태를 모두 해제하고 preview 선을 숨김 |
| none 상태에서 개별 방향 클릭 | 클릭한 방향만 적용 |
| none 상태에서 전체 클릭 | 네 방향을 한 번에 적용 |
| `선 모양 바로 적용` | 현재 켜진 방향에만 선 속성 변경 반영 |

## 5. 검증 결과

| 명령 | 결과 |
|------|------|
| `cd rhwp-studio && node --check e2e/page-border-toggle.test.mjs` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/page-border-toggle.test.mjs --mode=headless` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/dialog-theme.test.mjs --mode=headless` | 통과 |
| `git diff --check` | 통과 |

`npm run build`의 Vite chunk size 경고는 기존 번들 크기 안내이며 빌드 실패가 아니다.

## 6. 완료 기준 대응

| 완료 기준 | 결과 |
|-----------|------|
| 위쪽 버튼 2회 클릭 후 line count 원복 | 신규 E2E 통과 |
| 전체 버튼 2회 클릭 후 line count 원복 | 신규 E2E 통과 |
| 위쪽만 켠 상태에서 선 종류 변경 시 line count 1 유지 | 신규 E2E 통과 |
| `테두리 사용 안 함` 후 전체 버튼 1회 클릭 시 전체 적용 | 신규 E2E 통과 |
| `테두리 사용 안 함` 후 개별 방향 버튼 클릭 시 해당 방향만 적용 | 신규 E2E 통과 |
| `테두리 사용 안 함` 체크 상태 유지 | 신규 E2E 통과 |
| 기존 다이얼로그 테마 검증 유지 | `dialog-theme.test.mjs` 통과 |

## 7. 변경 파일

- `rhwp-studio/src/ui/page-border-dialog.ts`
- `rhwp-studio/e2e/page-border-toggle.test.mjs`
- `mydocs/plans/task_m100_1426.md`
- `mydocs/plans/task_m100_1426_impl.md`
- `mydocs/working/task_m100_1426_stage1.md`
- `mydocs/working/task_m100_1426_stage2.md`
- `mydocs/working/task_m100_1426_stage3.md`
- `mydocs/report/task_m100_1426_report.md`
- `mydocs/orders/20260621.md`

## 8. 후속 조치

PR #1455의 최신 head 기준 GitHub Actions 통과와 작업지시자 승인 후 merge한다.
이슈 close는 별도 승인 후 수행한다.
