# Task 269 구현 계획서: 표 선택 동작 기능 추가

## 1단계: 셀 선택 단계 관리 (cursor.ts)

### 변경 파일
- `rhwp-studio/src/engine/cursor.ts`

### 상세
- `_cellSelectionPhase: number` 필드 추가 (1=단일셀, 2=범위, 3=전체)
- `enterCellSelectionMode()`: phase=1로 시작 (기존 동작 유지)
- `advanceCellSelectionPhase()`: 신규 메서드
  - phase 1→2: anchor 고정, 범위 선택 모드 전환
  - phase 2→3: anchor=(0,0), focus=(maxRow-1, maxCol-1) 전체 선택
- `exitCellSelectionMode()`: phase 초기화
- `getCellSelectionPhase(): number` getter

## 2단계: F5 + 방향키 키보드 처리 (input-handler-keyboard.ts)

### 변경 파일
- `rhwp-studio/src/engine/input-handler-keyboard.ts`

### 상세
- F5 처리 수정:
  ```
  if (이미 셀 선택 모드) {
    cursor.advanceCellSelectionPhase();
    updateCellSelection();
  } else {
    cursor.enterCellSelectionMode();
  }
  ```
- 셀 선택 모드 방향키 분기:
  - phase 1: `moveCellSelection(dr, dc)` (기존: 단일 셀 이동)
  - phase 2: `expandCellSelection(dr, dc)` (anchor 고정, focus 확장)
  - phase 3 + Ctrl: `resizeTable(axis, delta)` 호출

## 3단계: 범위 확장 + 전체 선택 (cursor.ts)

### 변경 파일
- `rhwp-studio/src/engine/cursor.ts`

### 상세
- `expandCellSelection(deltaRow, deltaCol)`: anchor 유지, focus만 이동
  - `cellFocus.row += deltaRow`, `cellFocus.col += deltaCol` (범위 제한)
- `selectAllCells()`: anchor=(0,0), focus=(rowCount-1, colCount-1)

## 4단계: 표 비율 리사이즈

### 변경 파일
- `src/document_core/commands/table_ops.rs` (Rust)
- `src/wasm_api.rs` (WASM 바인딩)
- `rhwp-studio/src/core/wasm-bridge.ts` (TS 브릿지)
- `rhwp-studio/src/engine/input-handler-keyboard.ts` (키 처리)

### 상세
- Rust API: `resize_table_proportional_native(sec, para, ci, axis, delta_hu)`
  - axis=0: 열 너비 비율 증감, axis=1: 행 높이 비율 증감
  - delta_hu: HWPUNIT 단위 증감값
  - 모든 열/행을 동일 비율로 조정
- Ctrl+←→: 열 너비 비율 증감 (delta = ±200 HU)
- Ctrl+↑↓: 행 높이 비율 증감 (delta = ±200 HU)

## 5단계: 셀 선택 렌더링 갱신

### 변경 파일
- `rhwp-studio/src/engine/cell-selection-renderer.ts`

### 상세
- phase 3(전체 선택)일 때 전체 표 영역 하이라이트
- 기존 범위 선택 하이라이트 로직은 phase 1, 2에서 동일하게 사용

## 테스트 시나리오

1. 표 내 셀에서 F5 → 현재 셀 선택 (phase 1)
2. F5 다시 → 범위 선택 모드 (phase 2), 방향키로 범위 확장
3. F5 다시 → 전체 선택 (phase 3), 모든 셀 하이라이트
4. Ctrl+→ → 전체 열 너비 증가
5. Ctrl+↓ → 전체 행 높이 증가
6. Escape → 셀 선택 모드 탈출
