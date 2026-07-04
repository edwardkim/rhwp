# PR #1424 리뷰 - rhwp-studio 다크모드 잔여 UI 대비 정리

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1424 |
| 제목 | rhwp-studio 다크모드 잔여 UI 대비 정리 |
| 작성자 | postmelee |
| 관련 이슈 | #1422 PR #1420 후속: rhwp-studio 다크모드 잔여 UI 대비 문제 |
| base | `devel` |
| head | `edwardkim:task_m100_1422` |
| draft | true (리뷰 문서 반영 후 ready 전환 예정) |
| mergeable | `CLEAN` |
| 현재 head | `359fbd3f` (리뷰 문서 커밋 전 기준) |
| 변경량 | 31 files, +1942 / -105 (리뷰 문서 커밋 전 기준) |

PR 본문에 `Closes #1422`가 포함되어 있으므로 merge 시 이슈 자동 close 대상이다.

이번 PR은 collaborator self-merge가 예정된 PR이다. 코드 작성자와 merge 수행 권한자가 같은 점을 보완하기 위해
다음 안전장치를 적용한다.

- PR 리뷰 문서와 처리 계획서를 PR diff에 포함한다.
- 작업지시자 확인 후 작업지시자가 GitHub merge 버튼을 직접 누른다.
- local 검증과 GitHub Actions 통과를 merge 전 조건으로 둔다.
- merge 후에는 GitHub 상태 확인만 수행하고, 별도 보고서 커밋으로 추가 merge를 만들지 않는다.

## 2. 변경 범위

핵심 변경:

- `rhwp-studio/src/styles/dialogs.css`
  - dialog input/select/textarea, fieldset/legend, preview button 계열 dark token 정리
  - 문서 preview용 `--doc-paper`, UI border/surface token 분리
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`, `table-cell-props.css`
  - 표/셀 속성 읽기 전용 너비/높이 입력 필드의 라이트 배경 제거
  - 테두리 선 샘플, 셀/문서 preview 주변 UI 색상 분리
- `rhwp-studio/src/ui/cell-border-bg-dialog.ts`
  - 셀 테두리/배경 preview 버튼, 보조선, label 색상 token화
- `rhwp-studio/src/ui/page-border-dialog.ts`
  - 중앙 SVG 문서 preview는 흰 종이로 유지
  - 사방 버튼, fieldset/legend, preview guide 대비 보정
- `rhwp-studio/src/styles/para-shape-dialog.css`, `para-shape-dialog.ts`, `para-shape-tab-builders.ts`
  - 문단 preview는 문서 색상 의미를 유지하고 주변 UI만 dark token 적용
- `rhwp-studio/src/ui/table-create-dialog.ts`, `endnote-shape-dialog.ts`, `toolbar.ts`, `validation-modal.ts`, `grid-settings-dialog.ts`
  - popup/dialog 잔여 라이트 surface와 inline 색상 정리
- `rhwp-studio/src/core/theme.ts`, `rhwp-studio/index.html`
  - 명시적 light/dark 선택 시 root/meta `color-scheme`을 `only light` / `only dark`로 동기화
  - 저장된 dark 테마를 stylesheet 로드 전 bootstrap으로 선반영
- `rhwp-studio/e2e/dialog-theme.test.mjs`
  - 주요 dialog/popup computed style과 문서 preview 색상 정책 회귀 가드 추가
- `rhwp-studio/e2e/theme-auto-dark.test.mjs`
  - Chrome Auto Dark Mode 환경에서 명시적 light/dark 테마 보존 검증 추가
- `rhwp-studio/e2e/theme-bootstrap.test.mjs`, `theme-mode.test.mjs`, `helpers.mjs`
  - 초기 bootstrap, reload, `color-scheme`, Chrome extra args 검증 보강
- `mydocs/plans/`, `mydocs/working/`, `mydocs/report/`, `mydocs/orders/`
  - #1422 하이퍼-워터폴 계획/단계 보고/최종 보고 기록

## 3. 시각 검토 반영 사항

작업지시자가 PR 본문에 before/after 스크린샷을 추가했다. 다음 8개 화면을 PR 본문에서 보존한다.

1. 표/셀 속성
2. 수식 편집
3. 표 만들기 팝업
4. 셀 테두리/배경
5. 쪽 테두리/배경
6. 문단 모양
7. 미주 모양
8. Chrome Auto Dark Mode + 밝게 테마

정책상 문서 종이, SVG 문서 preview, 색상 견본은 다크 토큰으로 강제 반전하지 않았다. 표/셀과 쪽 테두리의
중앙 preview는 실제 문서 의미를 유지하고, 조작 버튼/fieldset/legend 등 UI chrome만 dark token으로 보정했다.

## 4. 로컬 검증

최신 `upstream/devel`(`9cced48c`) 반영 후 PR 준비 단계에서 다음 검증을 완료했다.

| 명령 | 결과 |
|---|---|
| `cd rhwp-studio && npm run build` | 통과 |
| `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-bootstrap.test.mjs --mode=headless` | 통과 |
| `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless` | 통과 |
| `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/dialog-theme.test.mjs --mode=headless` | 1차 navigation timeout 후 동일 서버 재시도 통과 |
| `cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' CHROME_EXTRA_ARGS='--enable-features=WebContentsForceDark' node e2e/theme-auto-dark.test.mjs --mode=headless` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |

## 5. GitHub Actions

리뷰 문서 push 전 확인 상태:

| 체크 | 상태 |
|---|---|
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

리뷰 문서와 PR 본문 보강 후 GitHub Actions가 다시 실행될 수 있으므로, 최종 merge 판단은 재실행된 checks 기준으로 진행한다.

## 6. 리스크

| 항목 | 평가 |
|---|---|
| 변경 범위 | 중간. dialog/popup 표면은 넓지만 대부분 inline light 색상 제거와 token 치환이다. |
| 문서 preview 회귀 | 낮음. `--doc-paper`와 실제 색상 견본을 유지하고, focused e2e로 문서 종이 preview를 검증했다. |
| 수식 색상 의미 변경 | 낮음. 수식 색을 흰색으로 바꾸지 않고 preview 배경을 문서 종이로 맞췄다. |
| Chrome Auto Dark Mode | 중간. Chrome 실험 기능이므로 버전/flag 차이가 있을 수 있다. `only light`/`only dark`와 headless flag 검증으로 완화했다. |
| 초기 theme bootstrap | 낮음~중간. head inline script가 추가되었지만 저장 설정 읽기와 root/meta 선반영만 수행하며, `theme-bootstrap` e2e로 고정했다. |
| self-merge 절차 | 중간. 리뷰 문서/처리 계획 PR 포함, GitHub Actions 통과, 작업지시자 직접 merge로 보완한다. |

## 7. 최종 권고

현재 상태에서는 리뷰 문서와 PR 본문 보강 커밋을 push한 뒤 GitHub Actions 재실행이 통과하면 merge 가능으로 판단한다.

권고 순서:

1. archive 경로의 PR 리뷰 문서와 처리 계획서를 PR head에 push
2. PR 본문에서 `mydocs/pr/archives/pr_1424_review.md`, `mydocs/pr/archives/pr_1424_review_impl.md`를 실제 파일명으로 갱신
3. PR 본문에 추가된 before/after 스크린샷을 유지
4. GitHub Actions 재실행 완료 대기
5. PR을 ready 상태로 전환
6. 작업지시자가 GitHub merge 버튼으로 merge
7. #1422 close 여부 확인
8. `upstream/devel` 동기화

merge 후 별도 `pr_1424_report.md` 커밋은 만들지 않는다. 본 리뷰 문서와 `pr_1424_review_impl.md`,
`task_m100_1422_report.md`가 merge 판단과 처리 기록 역할을 맡는다.
