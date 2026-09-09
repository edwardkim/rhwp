# Task M100 #1491 구현 계획

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-23

## 구현 방향

1. 현재 명령 경로를 최소 재현으로 고정한다.
   - 선택 행에 서로 다른 cell width를 가진 표를 구성한다.
   - 평균 반올림 후 delta 합이 0이 아닌 폭 배열을 사용한다.
   - `resizeTableCells` 적용 후 선택 행의 렌더 bbox 폭을 검사한다.

2. `셀 너비를 같게`이 만든 다중 셀 폭 변경을 local row 힌트로 보존한다.
   - `resize_table_cells_native`에서 같은 행의 셀 폭이 2개 이상 바뀐 경우, delta 합이 0이 아니어도 행별 독립 segment 목적일 수 있다.
   - 단순 전체 컬럼 resize와 구분하기 위해 필요한 경우 프런트 명령 payload에 의도를 명시하는 방법을 우선 검토한다.
   - 가로 병합 셀(`colSpan > 1`)도 선택된 실제 셀 하나로 보고 bbox 표시 폭 기준 평균에 포함한다.
   - 기존 Shift/일반 resize의 `localResize` 및 `renderWidth` 힌트와 충돌하지 않게 한다.

3. 렌더러의 표 외곽 폭 보정 조건을 유지한다.
   - 저장 파일 로드 시 보조 `cell.width` 때문에 표가 깨지는 것을 막는 `target_total` 검사는 유지한다.
   - 명령 실행으로 명시된 local row 힌트가 있는 경우에만 행별 누적 x를 우선한다.

## 예상 수정 파일

- `rhwp-studio/src/command/commands/table.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`
- `rhwp-studio/tests/table-cell-width-equal-1491.test.ts`
- `rhwp-studio/tests/table-mouse-resize-1491.test.ts`
- `tests/issue_493_cell_attrs.rs`

## 우선 검증 명령

```bash
cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
cargo fmt --check
git diff --check
```

필요 시:

```bash
wasm-pack build --target web --out-dir pkg
cargo test --profile release-test --test svg_snapshot -- --nocapture
```
