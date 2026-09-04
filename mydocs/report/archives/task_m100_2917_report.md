# 완료 보고서 — Task M100-2917

- 이슈: #2917
- 제목: rhwp-studio 툴바 글자 크기 입력에 상한 clamp 누락 — char-shape-dialog와 불일치
- 작성일: 2026-07-22
- 브랜치: `task/m100-2917-toolbar-fontsize-clamp`

## 1. 배경

`rhwp-studio/src/ui/char-shape-dialog.ts`의 `collectMods()`는 기준 크기 입력값을
wasm의 `applyCharFormat` 경로로 넘기기 전에 `fontSize`를 100~409600
(half-point 단위, pt로는 1~4096pt) 범위로 clamp한다.

반면 메인 툴바 `rhwp-studio/src/ui/toolbar.ts`의 글자 크기 `<input id="font-size">`는
같은 `CharProperties.fontSize` 필드를 채워 동일한 `format-char` 이벤트로
내보내는데도 상한 clamp가 전혀 없었다. Enter 키 확정 경로는 `pt > 0`만
검사했고, `+` 버튼(`btnSizeUp`)은 하한/상한 모두 없이 클릭할 때마다
무한정 증가했다.

같은 UI 계층에서 같은 필드를 다루는데 진입 경로(다이얼로그 vs 툴바)에 따라
검증 범위가 달라지는 불일치였고, 오늘 numbering(#2838/#2842)과 다이얼로그
(#2845/#2847)에서 고친 min/max clamp 누락과 같은 패턴이다.

## 2. 완료 내용

`toolbar.ts`의 글자 크기 Enter 키 경로와 `+` 버튼 경로에 `char-shape-dialog.ts`와
동일한 1~4096pt clamp를 적용했다. `-` 버튼(`btnSizeDown`)은 기존에 이미
`Math.max(1, pt - 1)`로 하한 clamp가 있어 변경하지 않았다.

## 3. 주요 변경

- `rhwp-studio/src/ui/toolbar.ts`
  - Enter 키 확정 경로: `pt`를 `Math.min(4096, Math.max(1, pt))`로 clamp한 뒤
    입력창 값도 clamp된 값으로 갱신하고 `format-char`를 emit
  - `btnSizeUp` 경로: `newPt`를 `Math.min(4096, pt + 1)`로 clamp
- `rhwp-studio/tests/toolbar-font-size-clamp.test.ts` (신규)
  - source-guard 테스트: 두 clamp 표현식이 소스에 존재하는지 정규식으로 검증

## 4. 검증 결과 (red → green)

- 수정 전: `toolbar-font-size-clamp.test.ts`가 기대하는 clamp 표현식이
  `toolbar.ts`에 존재하지 않아 실패(red).
- 수정 후:
  - `npm test` — 전체 352개 중 350개 통과. 실패 1건은 사전에 알려진
    `tests/cell-flow-boundary.test.ts` 뿐이며 이번 변경과 무관.
    (테스트 전 `node_modules`가 없어 `npm ci` 실행, 실행 후에는
    `tests/canvaskit-resource-key.test.ts`도 통과)
  - 신규 테스트 `toolbar-font-size-clamp.test.ts` 통과(green)
  - `npx tsc --noEmit` — 기존 baseline과 동일하게 `@wasm/rhwp.js` 관련
    TS2307 2건만 남고 신규 타입 오류 없음

## 5. 남은 이슈

없음. 범위가 작고 독립적인 clamp 수정으로 완료.
