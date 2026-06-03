# Task 269 수행 계획서: 표 선택 동작 기능 추가

## 한컴 동작

| F5 횟수 | 동작 | 방향키 |
|---------|------|--------|
| 1회 | 현재 셀 선택 | 방향키로 셀 이동 (단일 셀) |
| 2회 | 셀 범위 선택 모드 | 방향키로 범위 확장 (anchor 고정) |
| 3회 | 전체 셀 선택 | Ctrl+방향키로 표 비율 리사이즈 |

## 현재 상태

- F5 1회: ✓ 구현됨 (enterCellSelectionMode)
- F5 2회+: ✗ 미구현 (이미 셀 선택 모드면 무시)

## 구현 계획

### 1단계: 셀 선택 단계 관리
- `cellSelectionPhase: 1 | 2 | 3` 필드 추가 (cursor.ts)
- F5 반복 시 phase 증가: 1→2→3
- phase 1: 단일 셀 (현재 동작)
- phase 2: 범위 선택 (anchor 고정, 방향키로 focus 확장)
- phase 3: 전체 셀 선택

### 2단계: 키보드 핸들러 수정
- phase 2에서 방향키: `expandCellSelection` (anchor 고정)
- phase 3에서 Ctrl+방향키: `resizeTable` API 호출
- Escape: 셀 선택 모드 탈출

### 3단계: 전체 셀 선택 UI
- phase 3 진입 시 anchor=(0,0), focus=(maxRow, maxCol)
- 전체 셀 하이라이트 렌더링

### 4단계: 표 비율 리사이즈
- Ctrl+↑↓: 행 높이 비율 증감
- Ctrl+←→: 열 너비 비율 증감
- WASM API: resizeTableProportional(sec, para, ci, axis, delta)

## 참조 파일

| 파일 | 역할 |
|------|------|
| rhwp-studio/src/engine/cursor.ts | cellSelectionPhase 상태 |
| rhwp-studio/src/engine/input-handler-keyboard.ts | F5 + 방향키 처리 |
| rhwp-studio/src/engine/cell-selection-renderer.ts | 선택 하이라이트 |
