# Task M100 #1483 구현계획서 — 표 줄/칸 지우기 후 커서 cellIndex 보정

- 이슈: #1483
- 브랜치: `local/task1483`
- 작성일: 2026-06-23
- 수행계획서: `mydocs/plans/task_m100_1483.md`

## 구현 개요

`rhwp-studio/src/command/commands/table.ts` `applyTableDeleteRowColumn` 의 `operation`
콜백에서 삭제 후 반환 커서 위치를 보정한다. `deleteTableRow/Column` 의 `{ rowCount, colCount }`
로 새 표 크기를 알고, clamp 한 (row, col) 을 `getTableCellBboxes` 로 cellIndex 역조회한다.

executeOperation 의 `operation` 반환값이 커서 위치로 사용되는 기존 계약(`return pos` /
`return {...cellIndex:0}`)을 그대로 활용한다.

---

## 1단계 — 삭제 후 커서 보정 헬퍼 + 적용

**대상**: `rhwp-studio/src/command/commands/table.ts`

- 삭제 보정 헬퍼 추가:
  ```ts
  // 삭제 후 표 크기(rowCount/colCount) 내로 (row,col) clamp → cellIndex 역조회
  function clampedCellAfterDelete(
    wasm, sec, parentPara, controlIdx,
    origRow, origCol, rowCount, colCount,
  ): { cellIndex: number; cellParaIndex: number } | null {
    if (rowCount <= 0 || colCount <= 0) return null; // 표 소멸
    const row = Math.min(origRow, rowCount - 1);
    const col = Math.min(origCol, colCount - 1);
    const bboxes = wasm.getTableCellBboxes(sec, parentPara, controlIdx);
    // (row,col)을 포함하는 셀(병합 고려: row/col ∈ [r, r+rowSpan), [c, c+colSpan))
    const hit = bboxes.find((b) =>
      row >= b.row && row < b.row + b.rowSpan &&
      col >= b.col && col < b.col + b.colSpan);
    const cellIndex = hit ? hit.cellIdx : 0;
    return { cellIndex, cellParaIndex: 0 };
  }
  ```
- `applyTableDeleteRowColumn` 의 `operation` 콜백 수정:
  ```ts
  operation: (wasm) => {
    const res = mode === 'row'
      ? wasm.deleteTableRow(...cellInfo.row)
      : wasm.deleteTableColumn(...cellInfo.col);
    if (!res.ok) return pos;
    const corrected = clampedCellAfterDelete(
      wasm, pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
      cellInfo.row, cellInfo.col, res.rowCount, res.colCount);
    if (!corrected) {
      // 표 소멸 → 본문 위치 폴백 (parentPara 등 해제)
      return { sectionIndex: pos.sectionIndex, paragraphIndex: pos.parentParaIndex ?? 0, charOffset: 0 };
    }
    return { ...pos, charOffset: 0, ...corrected };
  },
  ```

**완료 기준**: tsc 통과. 수동 재현(표 생성 → 마지막 칸/줄 지우기)에서 `updateRect` 오류 무발생.

## 2단계 — 회귀 검증 (E2E/수동 + 빌드)

- `rhwp-studio` tsc + 기존 표 편집 단위/E2E 테스트 통과.
- 재현 시나리오 수동 확인:
  - 행 삭제(마지막 행 커서) → 커서가 유효 셀로 이동, 콘솔 무경고.
  - 열 삭제(마지막 열 커서) → 동일.
  - 1행/1열 표에서 마지막 행/열 삭제(표 소멸) → 본문 폴백, 오류 없음.
- WASM 재빌드 불필요(TS만 변경). 확장 빌드는 최종 단계에서.

**완료 기준**: tsc + 표 편집 회귀 없음 + 3개 시나리오 수동 통과.

## 3단계 — 최종 검증 + 보고서 + 확장 반영

- `rhwp-studio` 빌드(`npm run build`) 성공.
- 최종 보고서 작성, 오늘할일(#1483) 상태 갱신.
- (릴리즈 별개) 확장 재패키징은 본 수정이 main 반영된 후 별도로 수행.

**완료 기준**: 빌드 통과 + 보고서/오늘할일 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `rhwp-studio/src/command/commands/table.ts` | 삭제 후 커서 보정 헬퍼 + `applyTableDeleteRowColumn` 적용 (~25줄) |
| `mydocs/working/task_m100_1483_stage{N}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1483_report.md` | 최종 보고서 |

## 위험 / 주의

- `getTableCellBboxes` 가 pageHint 없이 전체 셀 매핑을 반환하는지 확인(셀 병합 표에서 cellIdx↔row/col).
- 병합 셀 경계의 (row,col) 매칭은 rowSpan/colSpan 범위 포함 검사로 처리.
- 표 소멸 케이스의 본문 폴백 위치(parentParaIndex 기준)가 유효한지 확인.
