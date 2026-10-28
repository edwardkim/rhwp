# 완료 보고서 — Task M100-1422

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 제목: rhwp-studio 다크모드 잔여 UI 대비 문제
- 브랜치: `local/task1422`
- 작업 시작 기준 커밋: `ab1879c94328cf49b569e2d687ae723b75f3acaa`
- PR 제출 기준 base: `9cced48c7b31e7e0f5b0215694a2b921f6de02c8`
- 작성일: 2026-06-17

## 1. 결과 요약

PR #1420 반영 후 남아 있던 rhwp-studio 다크모드 UI 대비 문제를 7개 stage로 정리했다.
작업 범위는 표/셀 속성, 셀 테두리/배경, 수식 편집, 쪽 테두리/배경, 표 만들기 popup,
문단/미주/글머리표/validation/grid 관련 잔여 surface, Chrome Auto Dark Mode 대응,
저장된 dark 테마 새로고침 초기 paint 보정이다.

핵심 정책은 UI chrome과 문서 preview 색상을 분리하는 것이다. 입력 필드, fieldset, popup,
preview 주변 버튼은 semantic UI token을 사용하도록 정리했고, 문서 종이, SVG 문서 preview,
색상 견본처럼 실제 문서 의미가 있는 영역은 흰 종이 또는 실제 색상값을 유지했다.

## 2. 단계별 완료 내역

| Stage | 커밋 | 내용 |
|---|---|---|
| 1 | `71879bd7` | 공통 dialog/control dark token 정리 |
| 2 | `dccb3968` | 표/셀 속성, 셀 테두리/배경 dark token 정리 |
| 3 | `5ee17a8c` | 수식 편집 preview와 쪽 테두리/배경 dark token 정리 |
| 3 보정 | `e3a82ec7` | 쪽 테두리 preview guide 대비 보정 |
| 4 | `3289c6c4` | table quick grid, endnote, para preview, bullet popup, validation, grid sweep |
| 5 | `e91fc07e` | dialog theme focused regression guard 추가 |
| 6 | `fcab3d12` | Chrome Auto Dark Mode에서 명시적 light/dark 의도 보존 |
| 7 | `47c7e43f` | 저장된 dark 테마를 stylesheet 전 bootstrap으로 선반영 |

## 3. 주요 변경 파일

| 파일 | 내용 |
|---|---|
| `rhwp-studio/src/styles/dialogs.css` | dialog input/select/fieldset/preview button 계열 색상 token화 |
| `rhwp-studio/src/styles/table-cell-props.css` | 표/셀 속성 preview 버튼과 token 연동 보강 |
| `rhwp-studio/src/styles/para-shape-dialog.css` | 문단 모양 preview와 active button 색상 정책 보강 |
| `rhwp-studio/src/core/theme.ts` | root/meta `color-scheme`을 명시 테마에 맞춰 `only light` / `only dark`로 동기화 |
| `rhwp-studio/index.html` | `color-scheme` meta 추가 및 저장 theme bootstrap inline script 추가 |
| `rhwp-studio/src/ui/table-cell-props-dialog.ts` | 읽기 전용 필드와 표/셀 preview 주변 UI dark 처리 |
| `rhwp-studio/src/ui/cell-border-bg-dialog.ts` | 셀 테두리/배경 preview 버튼, 보조선, legend dark 처리 |
| `rhwp-studio/src/ui/equation-editor-dialog.ts` | 수식 preview 문서 종이 배경 유지로 검은 수식 가독성 확보 |
| `rhwp-studio/src/ui/page-border-dialog.ts` | 중앙 SVG 문서 preview 유지, 주변 버튼/목차명/guide 대비 정리 |
| `rhwp-studio/src/ui/table-create-dialog.ts` | 표 만들기 quick grid popup token화 |
| `rhwp-studio/src/ui/endnote-shape-dialog.ts` | preview button/menu surface dark 처리 |
| `rhwp-studio/src/ui/para-shape-dialog.ts` | 문단 preview 표면과 UI chrome 색상 분리 |
| `rhwp-studio/src/ui/toolbar.ts` | 글머리표 popup surface/cell hover dark 처리 |
| `rhwp-studio/src/ui/validation-modal.ts` | validation modal 잔여 fieldset/control 색상 정리 |
| `rhwp-studio/src/ui/grid-settings-dialog.ts` | grid 설정 dialog 잔여 fieldset/control 색상 정리 |
| `rhwp-studio/e2e/helpers.mjs` | Chrome 추가 인자 주입 지원 |
| `rhwp-studio/e2e/dialog-theme.test.mjs` | 주요 dialog computed style 회귀 가드 추가 |
| `rhwp-studio/e2e/theme-auto-dark.test.mjs` | Chrome Auto Dark Mode 회귀 가드 추가 |
| `rhwp-studio/e2e/theme-bootstrap.test.mjs` | 저장 dark theme 초기 bootstrap 회귀 가드 추가 |
| `rhwp-studio/e2e/theme-mode.test.mjs` | `color-scheme` 및 reload 검증 보강 |

## 4. 검증

실행한 검증:

최신 `upstream/devel`(`9cced48c`) 반영 후 2026-06-17 16:16 기준으로 재검증했다.

```bash
cd rhwp-studio && npm run build
```

- 통과

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-bootstrap.test.mjs --mode=headless
```

- 통과
- DOMContentLoaded 시점부터 저장 dark mode, effective theme, root `dark only`, meta `only dark`, dark `theme-color` 확인
- `#menu-bar` computed background가 `rgb(43, 48, 55)`로 계산됨을 확인

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

- 통과
- light/dark/system 전환과 dark reload 후 `color-scheme` 유지 확인

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/dialog-theme.test.mjs --mode=headless
```

- 통과
- 1차 재실행은 최초 navigation 단계에서 timeout이 있었으나, 동일 서버 상태 확인 후 즉시 재시도에서 전체 assertion과 HTML 보고서 생성이 통과하고 정상 종료했다.

```bash
cd rhwp-studio && VITE_URL=http://127.0.0.1:7702 CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' CHROME_EXTRA_ARGS='--enable-features=WebContentsForceDark' node e2e/theme-auto-dark.test.mjs --mode=headless
```

- 통과
- Chrome Auto Dark Mode 활성화 조건에서 light menu pixel `(245, 245, 245)`, dark pixel `(43, 48, 55)` 확인

## 5. 현재 실행 상태

rhwp-studio dev server는 로컬에서 다음 주소로 떠 있다.

```text
http://127.0.0.1:7702/
```

초기 요청 포트 `7701`은 이미 사용 중이라 Vite가 `7702`로 전환했다.

## 6. 잔여 리스크

- Chrome `Auto Dark Mode for Web Contents`는 실험 기능이므로 사용자 Chrome의 버전/플래그 상태에서 수동 시각 확인을 한 번 더 권장한다.
- Stage 7 검증 중 일부 브라우저 cleanup 지연이 관찰된 적이 있다. PR 전 재검증은 정상 종료했지만, 재현 시 runner cleanup은 후속 점검 후보로 남긴다.
- 이슈 #1422 close는 작업지시자 승인 후에만 수행한다.

## 7. 결론

#1422에 등록된 대표 다크모드 대비 문제와 추가로 발견된 Chrome Auto Dark Mode, dark reload 초기 flash 대응까지
계획된 Stage 1~7 구현과 최신 `upstream/devel` 반영 후 검증을 완료했다. PR #1424에는 리뷰 문서와 처리 계획을
동반해 merge 후 추가 저장소 문서 커밋이 필요하지 않도록 한다.
