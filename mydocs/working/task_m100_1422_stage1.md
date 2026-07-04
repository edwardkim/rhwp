# Task M100-1422 Stage 1 완료 보고서 — 공통 dialog/control 토큰 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 수행 계획서: `mydocs/plans/task_m100_1422.md`
- 구현 계획서: `mydocs/plans/task_m100_1422_impl.md`
- 브랜치: `local/task1422`
- 작성일: 2026-06-17

## 1. Stage 1 목표

공통 다이얼로그/폼 컨트롤이 다크모드에서 브라우저 기본 스타일이나 라이트 테마 하드코딩에 기대지 않고,
semantic token 기반 배경/글자/테두리/상태 색상을 갖도록 정리한다.

이번 단계는 공통 기반만 다루며, 표/셀 속성·셀 테두리/배경·수식 preview·쪽 테두리/배경의 개별 inline
style 정리는 Stage 2 이후로 남긴다.

## 2. 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `rhwp-studio/src/styles/dialogs.css` | 공통 input/select/button/textarea 계열의 background/color/caret/color-scheme/focus/disabled/read-only 상태를 token 기반으로 명시 |
| `mydocs/orders/20260617.md` | #1422 진행 비고 갱신 |

## 3. 구현 내용

1. `.dialog-input`, `.dialog-select`, `.dialog-text-input`에 `background`, `color`, `caret-color`, `color-scheme`을 명시했다.
2. 공통 입력 필드의 `:focus` 상태를 `--color-focus-border`와 `--ui-focus-soft` 기반으로 통일했다.
3. `:read-only`, `:disabled` 상태를 `--ui-surface-muted`, `--color-text-secondary`, `--color-text-disabled` 기반으로 명시했다.
4. `.dialog-btn`, `.dialog-btn-group button`, 수식 툴바 버튼, 검색 결과 버튼에 글자색과 disabled 상태를 보강했다.
5. checkbox/radio는 `accent-color: var(--color-primary)`를 사용하도록 했다.
6. 수식/누름틀/계산식 다이얼로그의 입력 필드와 textarea도 공통 토큰 체계에 맞춰 색상과 상태를 명시했다.
7. placeholder 색상을 `--color-text-placeholder`로 통일했다.

## 4. 의도적으로 남긴 범위

- `table-cell-props-dialog.ts`의 읽기 전용 width/height inline `#f5f5f5`는 Stage 2에서 처리한다.
- `cell-border-bg-dialog.ts`와 `table-cell-props-dialog.ts`의 SVG preview 절대색은 Stage 2에서 문서 preview/주변 UI로 분류한다.
- `.eq-preview` 배경 정책은 Stage 3에서 수식 preview 정책과 함께 처리한다.
- `page-border-dialog.ts`의 중앙 문서 preview와 주변 버튼/legend 분리는 Stage 3에서 처리한다.
- 표 만들기 popup, 미주 모양, 문단 모양, toolbar popup 등 추가 후보는 Stage 4에서 처리한다.

## 5. 검증

```bash
cd rhwp-studio && npm run build
```

- 결과: 통과
- 비고: Vite chunk size warning은 기존 번들 경고이며 이번 CSS 변경과 무관하다.

```bash
cd rhwp-studio && CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

- 결과: 통과
- 검증 항목:
  - 기본 theme mode `system`
  - dark 선택 및 localStorage 저장
  - dark에서 편집 용지 흰색 유지
  - 새로고침 후 dark 유지
  - light 선택 및 `color-scheme: light`
  - light에서 편집 용지 흰색 유지

참고:

- `CHROME_PATH` 없이 실행한 첫 e2e는 `puppeteer-core` 실행 경로 미지정으로 실패했다.
- `CHROME_PATH` 지정 후 샌드박스 안에서는 Chrome 프로세스 실행이 실패했다.
- 로컬 Chrome 실행이 필요한 e2e라 샌드박스 밖 실행으로 재검증했고 통과했다.

## 6. 잔여 사항

다음 단계(Stage 2)는 표/셀 속성 및 셀 테두리/배경 다이얼로그의 개별 inline style과 preview 주변 UI를
정리한다. 특히 `#f5f5f5`, `#333`, `#fff`, `#ccc` 계열 값을 문서 preview 유지 대상과 UI token 전환
대상으로 분류해야 한다.

## 7. 승인 요청

Stage 1 변경 범위와 검증 결과를 승인 요청한다. 승인 후 Stage 2를 진행한다.
