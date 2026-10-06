# task_m100_2845 처리 보고

## 이슈
edwardkim/rhwp#2845 — 편집 용지·표/셀 속성 대화상자 숫자 입력: HTML min 속성이 있어도 JS clamp
누락으로 음수가 WASM까지 전달됨.

## 배경
직전에 처리한 #2838(문단 번호 대화상자 시작 번호 clamp 누락)과 동일 패턴을 다른 대화상자에도
찾아보라는 지시에 따라 `table-property-dialog.ts`(존재하지 않음 → 실질적으로 대응하는
`table-cell-props-dialog.ts`), `bullet-dialog.ts`(존재하지 않음), `style-dialog.ts`,
`page-setup-dialog.ts` 를 검사했다.

## 조사 결과
- `style-dialog.ts`: `type="number"` 입력 없음(해당 없음).
- `table-create-dialog.ts`: 행/열 입력은 `Math.max(1, Math.min(256, ...))`로 이미 clamp됨(정상).
- `table-row-column-dialog.ts`: `countInput`은 `min`/`max` HTML 속성 + `input` 이벤트에서
  `clampInsertCount()`로 실제 clamp까지 수행(정상 — 대조군).
- **`page-setup-dialog.ts`**: `numberInput()` 헬퍼가 `min='0'`만 설정하고 `max` 없음. `onConfirm()`이
  `widthInput`/`heightInput`/`marginInputs[...]`의 `.value`를 검증 없이 `parseFloat` 후
  `wasm.setPageDef()`로 전달 — **버그 확인**.
- **`table-cell-props-dialog.ts`**: 동일한 `numberInput()` 헬퍼가 그대로 복제되어 있고, 셀 폭/높이/
  안쪽여백, 표 안쪽여백/바깥여백/세로·가로 오프셋/캡션 폭/간격 등 최소 14개 필드가 검증 없이
  `wasm.setCellProperties()` / `wasm.setTableProperties()`로 전달 — **버그 확인**.
- 회전각/기울이기 입력(`rotInput`/`skewH`/`skewV`)은 `disabled=true`라 사용자 입력 경로가 없어
  제외.

## 수정
두 파일의 `numberInput()` 헬퍼에 공통으로 `change` 리스너를 추가해, 생성된 `<input>`의
`min`/`max` 속성값을 기준으로 `.value`를 실제로 clamp한다(빈 문자열은 통과시켜 기존
`parseFloat(...) || 0` fallback과 충돌하지 않게 함). 헬퍼 하나만 고쳐서 두 대화상자가 쓰는
모든 숫자 필드가 함께 방어된다.

- `rhwp-studio/src/ui/page-setup-dialog.ts`
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`

## 테스트
`rhwp-studio/tests/dialog-numberinput-min-clamp.test.ts` 추가(소스 가드): 두 파일의
`numberInput()` 함수 본문에 `change` 리스너 + `Math.min(max, Math.max(min, v))` clamp가
있는지 정규식으로 검사.

- 수정 전(가드): 테스트 실패(리스너 없음) — 로컬에서 red 확인 후 fix 적용.
- 수정 후: `npm test` → 501 tests, 500 pass, 1 fail(`tests/cell-flow-boundary.test.ts`,
  기존에도 실패하던 것으로 이번 변경과 무관 — baseline).
- `npx tsc --noEmit` → 기존 baseline TS2307 2건(`@wasm/rhwp.js` 모듈 없음)만 남고 신규 에러 없음.

## 커밋/PR
- 브랜치: `task/m100-2845-dialog-number-min-clamp` (origin/devel 기준)
- PR: edwardkim/rhwp 로 devel 대상 생성 예정.
