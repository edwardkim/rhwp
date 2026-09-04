# Task #2938 처리 결과 — 구역 설정 대화상자 단 간격·탭 간격 clamp 누락 수정

## 이슈

- #2938: [rhwp-studio] 구역 설정 대화상자 단 간격·탭 간격 입력에 하한 clamp 누락 — #2845/#2847과 동일 패턴

## 문제

`rhwp-studio/src/ui/section-settings-dialog.ts`의 `onConfirm()`에서 `columnSpacing`
(단 사이 간격)과 `defaultTabSpacing`(기본 탭 간격) 두 필드가 `numberInput()` 헬퍼로
`min='0'`을 갖고 있음에도, `onConfirm()`이 `checkValidity()`를 호출하지 않고
`.value`를 직접 `parseFloat()`하여 `wasm.setSectionDef` / `wasm.setSectionDefAll`로
전달했다. HTML `min` 속성은 `.value`를 자동으로 clamp하지 않으므로, 사용자가 입력창에
직접 `-5` 같은 음수를 타이핑하고 확인을 누르면 음수 HWPUNIT 값이 검증 없이 WASM
경계를 넘어갔다.

이는 #2845(수정 PR #2847)에서 지적된 `page-setup-dialog.ts` / `table-cell-props-dialog.ts`의
동일 패턴, 그리고 #2934(수정 PR #2937)의 `char-shape-dialog.ts` 장평/자간 clamp 누락과
같은 종류의 비대칭 가드 누락이었다. 같은 파일 안에서도 `pageNum` 필드(L137)는 이미
`Math.max(1, parseInt(...) || 1)`로 clamp되어 있어, `columnSpacing`/`defaultTabSpacing`
두 필드만 가드가 빠진 상태였다.

## 재현/근거

수정 전 (`onConfirm()`, L146-147):

```ts
columnSpacing: ptToHwpunit(parseFloat(this.columnSpacingInput.value) || 0),
defaultTabSpacing: ptToHwpunit(parseFloat(this.defaultTabSpacingInput.value) || 0),
```

`parseFloat('-5') || 0`는 `-5`(truthy)이므로 `|| 0` fallback으로 걸러지지 않는다.

## 수정

`onConfirm()`에서 두 필드 계산에 `Math.max(0, ...)` 하한 clamp를 추가했다
(`column-settings-dialog.ts`의 `spacingHu` clamp 패턴과 동일):

```ts
columnSpacing: Math.max(0, ptToHwpunit(parseFloat(this.columnSpacingInput.value) || 0)),
defaultTabSpacing: Math.max(0, ptToHwpunit(parseFloat(this.defaultTabSpacingInput.value) || 0)),
```

diff는 2줄 수정 + 주석 1줄로 총 3줄.

## 테스트 (red → green)

`rhwp-studio/tests/section-settings-spacing-clamp.test.ts` 신규 추가. `onConfirm()`
소스 본문을 정규식으로 읽어 `columnSpacing`/`defaultTabSpacing` 계산식이
`Math.max(0, ...)` 형태인지 검증하는 source-guard 테스트 2건.

- **Red**: 수정 전 `origin/devel` 소스(`ptToHwpunit(parseFloat(...) || 0)`, `Math.max` 없음)에
  대해 정규식이 매치하지 않아 실패함을 확인 (`git show origin/devel:...`로 대조).
- **Green**: 수정 후 `npx tsx --test tests/section-settings-spacing-clamp.test.ts` →
  2/2 통과.

## 검증

- `npm test`: 501 tests, 500 pass, 1 fail — 실패한 테스트는 기존에 알려진
  `tests/cell-flow-boundary.test.ts` baseline 실패 1건뿐이며, 이번 변경과 무관.
- `npx tsc --noEmit`: `TS2307` (`@wasm/rhwp.js` 모듈 없음) 2건, 모두 기존 baseline이며
  이번 변경으로 인한 신규 오류 없음.

## 변경 파일

- `rhwp-studio/src/ui/section-settings-dialog.ts` (수정, 3줄)
- `rhwp-studio/tests/section-settings-spacing-clamp.test.ts` (신규 테스트)
- `mydocs/report/task_m100_2938_report.md` (본 문서)
