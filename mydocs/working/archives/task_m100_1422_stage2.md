# Task M100-1422 Stage 2 완료 보고서 — 표/셀 속성 및 셀 테두리/배경 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 수행 계획서: `mydocs/plans/task_m100_1422.md`
- 구현 계획서: `mydocs/plans/task_m100_1422_impl.md`
- 브랜치: `local/task1422`
- 작성일: 2026-06-17
- 이전 단계: `mydocs/working/task_m100_1422_stage1.md`

## 1. Stage 2 목표

표/셀 속성과 셀 테두리/배경 다이얼로그에서 다크모드와 맞지 않던 읽기 전용 필드, 선 종류 샘플,
테두리 preview, 배경 preview 주변 색상을 정리한다.

문서 또는 셀 preview의 흰 배경은 다크 UI 토큰으로 바꾸지 않고 `--doc-paper`로 유지한다.

## 2. 변경 파일

| 파일 | 변경 내용 |
|---|---|
| `rhwp-studio/src/styles/table-cell-props.css` | 선 종류 SVG, 방향 버튼, preview SVG, 배경 preview의 토큰 기반 색상 보강 |
| `rhwp-studio/src/ui/table-cell-props-dialog.ts` | 선 샘플 `#333`, preview 배경 `#fff`, 보조선 `#ccc`, read-only 배경 `#f5f5f5` 하드코딩 제거 |
| `rhwp-studio/src/ui/cell-border-bg-dialog.ts` | 표/셀 속성과 같은 방식으로 선 샘플, preview 배경, 보조선, 배경 없음 preview 정리 |
| `mydocs/orders/20260617.md` | #1422 진행 비고 갱신 |

## 3. 구현 내용

1. `.tcp-border-preview-svg`와 `.tcp-bg-preview`의 배경을 `--doc-paper`로 지정했다.
2. `.tcp-line-type-item svg`는 `color: var(--color-text-secondary)`를 사용하고, SVG 선은 `currentColor`를 사용하도록 했다.
3. 테두리 preview 내부 흰 배경은 `var(--doc-paper)`로 유지했다.
4. preview 내부 십자 보조선은 `var(--ui-border-light)`로 전환했다.
5. 표/셀 속성 기본 탭의 읽기 전용 width/height 필드에서 inline `#f5f5f5` 배경을 제거했다.
6. 배경 없음 preview는 `var(--doc-paper)`를 사용하도록 했다.
7. color picker의 기본값 `#ffffff`은 실제 문서 색상 값이므로 유지했다.

## 4. 의도적으로 남긴 범위

- 수식 편집 `.eq-preview` 배경 정책은 Stage 3에서 처리한다.
- 쪽 테두리/배경의 fieldset/legend와 SVG 사방 버튼은 Stage 3에서 처리한다.
- 표 만들기 popup, 미주 모양, 문단 모양, toolbar popup 등 추가 후보는 Stage 4에서 처리한다.
- 실제 색상 견본과 사용자가 지정한 선/채움색 값은 테마에 의해 변경하지 않는다.

## 5. 검증

```bash
cd rhwp-studio && npm run build
```

- 결과: 통과
- 비고: Vite chunk size warning은 기존 번들 경고이며 이번 변경과 무관하다.

```bash
cd rhwp-studio && CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' node e2e/theme-mode.test.mjs --mode=headless
```

- 결과: 통과
- 확인 항목:
  - dark mode dataset/effective theme
  - dark mode에서 편집 용지 흰색 유지
  - 새로고침 후 dark 유지
  - light mode 복귀와 `color-scheme: light`

Browser 플러그인 확인:

- URL: `http://127.0.0.1:7700/`
- 제목: `rhwp-studio`
- 콘솔 error/warn: 없음
- dark 상태에서 CSS rule 확인:
  - `.tcp-border-preview-svg` background: `var(--doc-paper)`
  - `.tcp-bg-preview` background: `var(--doc-paper)`
  - `.tcp-line-type-item svg` color: `var(--color-text-secondary)`
  - `.tcp-border-preview-wrap .tcp-dir-btn` background/color는 UI token 기반

## 6. 잔여 사항

다음 단계(Stage 3)는 수식 편집 preview와 쪽 테두리/배경 다이얼로그를 처리한다.
특히 수식 미리보기는 저장 색상 변경 없이 preview 배경을 문서 종이 성격으로 조정하고,
쪽 테두리/배경은 중앙 SVG 문서 미리보기 흰 배경을 유지한 채 주변 fieldset/버튼만 다크모드 처리한다.

## 7. 승인 요청

Stage 2 변경 범위와 검증 결과를 승인 요청한다. 승인 후 Stage 3을 진행한다.
