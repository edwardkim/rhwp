# Stage 3 — 선택 오버레이와 잔여 절대색 최소화

- 이슈: https://github.com/edwardkim/rhwp/issues/1158
- 브랜치: `local/task_m100_1158`
- 작성일: 2026-06-16

## 1. 목적

Stage 1, 2 이후 실제 UI CSS에 남은 절대색은 거의 정리됐다. 이번 Stage 3에서는 dark mode와
직접 충돌할 수 있는 표 선택 오버레이 계열을 token 기반으로 전환하고, 의도된 실제 색상 샘플만
잔여값으로 남긴다.

## 2. 범위

- `rhwp-studio/src/styles/table-selection.css`
- 필요 시 `rhwp-studio/src/styles/base.css` token 보강

제외:

- `style-bar.css`의 글자색/형광펜 미리보기 막대 색상
  - 실제 선택 색을 보여주는 샘플이라 고정 색상 유지

## 3. 현재 판단

- 표/셀 선택 오버레이는 문서 흰 종이 위에 그려지므로 dark theme에서도 충분히 보여야 한다.
- 따라서 workspace 배경색이 아니라 문서 선택/포커스 계열 token에 맞춰 정리하는 것이 안전하다.

## 4. 예정 검증

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

## 5. 실행 결과

반영 내용:

- `table-selection.css`의 셀 선택 채움/표 객체 외곽선/핸들을 token 기반으로 전환
- `base.css`에 다음 token 추가
  - `--table-selection-fill`
  - `--table-selection-stroke`

판단 정리:

- 남은 절대색은 `style-bar.css`의 글자색/형광펜 미리보기 막대뿐이며, 이는 사용자가 고른 실제 색상
  샘플을 그대로 보여주는 값이라 유지한다.
- 따라서 실제 UI CSS에서 dark mode와 충돌하는 잔여 절대색은 Stage 3 기준으로 정리 완료로 본다.

검증 결과:

- `cd rhwp-studio && npm run build` 통과
- `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless` 통과

## 6. 작업지시자 확인 대기

- Stage 3 커밋 및 최종 보고서 정리 진행

- Stage 3 수정 후 기본 빌드/스모크 재검증
