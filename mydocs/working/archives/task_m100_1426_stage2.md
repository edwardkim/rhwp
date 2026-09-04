# Task M100 #1426 2단계 완료보고서 — E2E 회귀 테스트

- 이슈: #1426
- 브랜치: `local/task1426`
- 작성일: 2026-06-21
- 단계: 2단계 — E2E 회귀 테스트 추가

## 1. 작업 범위

1단계에서 정리한 방향별 상태 모델을 기준으로, 쪽 테두리/배경 미리보기 버튼 동작만 검증하는
독립 E2E 회귀 테스트를 추가했다. 테스트는 개별/전체 토글, 활성 방향 즉시 적용, `테두리 사용 안 함`
상태에서의 전체/개별 적용 규칙을 함께 검증한다.

## 2. 변경 내용

| 파일 | 변경 |
|------|------|
| `rhwp-studio/e2e/page-border-toggle.test.mjs` | 쪽 테두리 preview 토글 및 none 체크 회귀 테스트 신규 작성 |

## 3. E2E 검증 항목

신규 E2E는 다음 동작을 검증한다.

1. 새 문서의 쪽 테두리 preview는 기본 선 없음 상태다.
2. 위쪽 버튼 1회 클릭 시 선 1개, 2회 클릭 시 선 0개로 복귀한다.
3. 전체 버튼 1회 클릭 시 선 4개, 2회 클릭 시 선 0개로 복귀한다.
4. 위쪽만 켠 상태에서 선 종류를 바꾸어도 선 개수가 1개로 유지된다.
5. `테두리 사용 안 함` 체크 시 preview 선이 모두 사라지고 체크 상태가 유지된다.
6. none 상태에서 전체 버튼 1회 클릭 시 사방 선이 바로 적용된다.
7. none 상태에서 위쪽 버튼 클릭 시 위쪽 선만 적용된다.
8. 각 토글 후 `테두리 사용 안 함` 체크 상태가 활성 방향 존재 여부와 일치한다.

## 4. 검증 결과

| 명령 | 결과 |
|------|------|
| `cd rhwp-studio && node --check e2e/page-border-toggle.test.mjs` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/page-border-toggle.test.mjs --mode=headless` | 통과 |
| `cd rhwp-studio && CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" node e2e/dialog-theme.test.mjs --mode=headless` | 통과 |
| `git diff --check` | 통과 |

`npm run build`에서 Vite chunk size 경고가 출력되었으나 빌드는 정상 완료되었다.
`puppeteer-core` 환경상 headless E2E에는 `CHROME_PATH` 지정이 필요했다.

## 5. 산출물

- 신규 E2E 보고서: `output/e2e/page-border-toggle-report.html`
- 기존 다이얼로그 테마 E2E 보고서: `output/e2e/dialog-theme-report.html`

두 보고서는 `output/` 하위 산출물이므로 Git 추적 대상이 아니다.

## 6. 남은 작업

3단계에서 최종 검증과 보고를 진행한다.

1. 변경 파일 전체 diff를 재검토한다.
2. 최종 보고서 `mydocs/report/task_m100_1426_report.md`를 작성한다.
3. 오늘할일 상태를 최종 보고서 작성/승인 대기 상태로 갱신한다.

## 7. 승인 요청

2단계 변경을 검토한 뒤, 3단계 진행 여부를 승인받는다.
