# task-m100-2925 처리 결과: 툴바 줄 간격 500% 상한 clamp 누락 수정

## 이슈

- 이슈: https://github.com/edwardkim/rhwp/issues/2925
- 배경: 직전 작업(#2917/#2920)에서 `rhwp-studio/src/ui/toolbar.ts`의 글자 크기 입력에
  char-shape-dialog.ts와 동일한 1~4096pt clamp를 적용했다. 그 후속으로 toolbar.ts의 다른
  숫자 입력 경로(줌 배율, 들여쓰기 증감 버튼, 줄 간격 퀵셀렉트 등)를 훑어 같은 유형의
  clamp 누락이 더 있는지 점검했다.

## 원인

`setupLineSpacingDropdown()` 안에 줄 간격 값을 바꾸는 경로가 세 가지 있다.

1. 셀렉트 `change` 이벤트 (프리셋 값만 선택 가능 — 범위 밖 값 없음, 안전)
2. 더블클릭 → 직접 입력(텍스트 input) → Enter/blur로 commit
3. ▲/▼ 증감 버튼 (5%씩 증감)

같은 "줄 간격 늘리기" 동작이 단축키(Alt+Shift+Z)로 실행되는 경로인
`rhwp-studio/src/command/commands/format.ts`의 `format:line-spacing-increase` 커맨드는

```ts
const newValue = Math.min(500, current + 10);
```

로 500%를 상한으로 clamp한다. 그런데 toolbar.ts의 직접 입력 commit(구 189행 부근)과
▲ 버튼(구 209~210행 부근)에는 이 상한 검사가 전혀 없어, 더블클릭 입력으로 `9999` 같은
값을 넣거나 ▲ 버튼을 연타하면 500%를 훨씬 초과하는 줄 간격이 그대로 적용됐다.
▼ 버튼은 `Math.max(5, cur - 5)`로 하한만 존재해 대칭성도 어긋나 있었다.

## 수정 (rhwp-studio/src/ui/toolbar.ts, +/-diff 약 6줄)

- 직접 입력 commit: `parseInt` 결과를 `Math.min(500, num)`으로 clamp한 뒤 `ensureLsOption` /
  `lsSelect.value` / dispatch에 사용하도록 변경.
- ▲ 버튼(`btnLsUp`): `next = cur + 5` → `next = Math.min(500, cur + 5)`.

커맨드 레이어(`format:line-spacing-increase`)의 기존 500% 상한과 동일한 값으로 맞췄다.

## 테스트 (Red → Green)

- 신규 소스-가드 테스트: `rhwp-studio/tests/toolbar-line-spacing-clamp.test.ts`
  - `toolbar.ts` 소스에서 `Math.min(500, num)`, `Math.min(500, cur + 5)` 패턴 존재를 정규식으로 확인.
  - Red: 수정 전 소스에는 해당 패턴이 없어 실패.
  - Green: 수정 후 통과.
- `npm test`: 500개 중 499 pass, 1 fail(`tests/cell-flow-boundary.test.ts` — 기존에도 실패하던
  무관 테스트, 베이스라인과 동일).
- `npx tsc --noEmit`: `@wasm/rhwp.js` 관련 TS2307 2건만 남음(기존 베이스라인과 동일, 신규 에러 없음).

## 커밋 / PR

- 브랜치: `task/m100-2925-linespacing-clamp` (origin/devel 기준)
- 커밋: `fix(studio): 툴바 줄 간격 직접입력/증가버튼에 500% 상한 clamp 적용`
