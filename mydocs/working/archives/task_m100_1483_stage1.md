# Task M100 #1483 1단계 완료보고서 — 삭제 후 커서 보정 구현

- 이슈: #1483
- 브랜치: `local/task1483`
- 작성일: 2026-06-23
- 단계: 1/3

## 변경 내용

`rhwp-studio/src/command/commands/table.ts`:

1. `clampedCellAfterDelete` 헬퍼 추가
   - 삭제 후 표 크기(`rowCount`/`colCount`) 내로 (row,col) clamp.
   - `getTableCellBboxes` 로 clamp 된 (row,col) 의 cellIdx 역조회 (병합 셀은 rowSpan/colSpan 범위 매칭).
   - 표 소멸(rowCount/colCount<=0) 시 null 반환.

2. `applyTableDeleteRowColumn` 의 `operation` 콜백 수정
   - `deleteTableRow/Column` 반환 `{ ok, rowCount, colCount }` 활용.
   - `res.ok` 면 `clampedCellAfterDelete` 로 보정한 cellIndex/cellParaIndex 를 커서에 반영.
   - 표 소멸 시 표 밖 본문 위치(parentParaIndex 기준)로 폴백.
   - 기존 `return pos`(보정 누락) 제거.

## 검증

- `rhwp-studio` tsc: 통과 (오류 없음).
- WASM/Rust 변경 없음 (기존 `deleteTable*` 반환값 + `getTableCellBboxes` 재사용).
- 줄/칸 추가 경로 무영향 (수정은 삭제 함수에 한정).

## 다음 단계

2단계: 표 편집 회귀(tsc/E2E) + 재현 시나리오 3종(행 삭제·열 삭제·표 소멸) 검증.
