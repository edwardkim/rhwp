# 완료 보고서 — Task M100-2908

- 이슈: #2908
- 제목: [rhwp-studio] 표/셀 속성 다이얼로그 테두리 탭: 굵기/색/선종류 select가 문서 값과 동기화되지 않아 방향 버튼 재적용 시 서식 유실
- 작성일: 2026-07-22
- 브랜치: `task/m100-2908-border-tab-select-sync`

## 1. 완료 내용

`rhwp-studio/src/ui/table-cell-props-dialog.ts`의 표/셀 속성 다이얼로그 "테두리" 탭에서,
다이얼로그를 열 때 `populateBorderFromTarget()`이 문서의 실제 테두리 값을 `borderEdits`
배열과 SVG 미리보기에만 반영하고, `굵기`(`borderWidthSelect`)/`색`(`borderColorInput`)/
`선 종류`(`borderSelectedLineType` + 선 종류 격자 `active` 클래스) 컨트롤은
`buildBorderTab()`이 만든 하드코딩 기본값(`0.1mm`/`#000000`/실선)에 그대로 머무르는
결함을 수정했다.

`applyBorderToDirection()`(방향 버튼 클릭 핸들러)은 이 세 컨트롤의 "현재 값"을 그대로
읽어 `borderEdits`를 덮어쓰고, 그 값이 `onConfirm()`에서 검증 없이
`wasm.setTableProperties`/`setCellProperties`에 전달된다. 따라서 사용자가 미리보기만
보고 값을 바꾸지 않은 채 방향 버튼(특히 "모두")을 다시 누르면, 문서에 저장돼 있던
서식(예: `0.4mm/#FF0000/파선`)이 조용히 `0.1mm/#000000/실선`으로 대체될 수 있었다.

`populateBorderFromTarget()`에서 대표 테두리(왼쪽, `borderEdits[0]`)를 기준으로
`borderWidthSelect.value`, `borderColorInput.value`, `borderSelectedLineType` 및 선 종류
격자의 `active` 클래스를 함께 동기화하도록 수정해, 컨트롤이 항상 문서/셀의 현재 테두리
상태를 정확히 반영하도록 했다.

## 2. 주요 변경

- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
  - `populateBorderFromTarget()`: 대표 테두리(왼쪽) 값으로 `borderWidthSelect`,
    `borderColorInput`, `borderSelectedLineType`, 선 종류 격자 활성 항목을 동기화하는
    로직 추가.
- `rhwp-studio/tests/table-cell-props-border-tab-sync.test.ts` (신규)
  - `populateBorderFromTarget` 본문에 위 3개 컨트롤 동기화 대입문이 존재하는지 정적으로
    핀하는 소스 가드 테스트 추가.

## 3. 검증 결과

- Red → Green: 수정 전 `table-cell-props-border-tab-sync.test.ts`는
  `populateBorderFromTarget`에 `this.borderWidthSelect.value = ...` 등 대입문이 없어
  실패 (assert.match 미스매치), 수정 후 통과.
- `npm test` (rhwp-studio): 500개 중 499 통과, `cell-flow-boundary.test.ts` 1건만 실패
  (본 작업과 무관한 기존 baseline 실패로 허용됨).
- `npx tsc --noEmit` (rhwp-studio): 기존 baseline과 동일하게 `wasm-bridge.ts`,
  `hwpctl/index.ts`의 `@wasm/rhwp.js` 관련 TS2307 2건만 존재, 신규 타입 오류 없음.
- `.rs` 파일 변경 없음, `cargo build` 불필요.

## 4. 남은 이슈

없음. TypeScript 전용 변경이며 범위 밖 리팩터링은 하지 않았다.
