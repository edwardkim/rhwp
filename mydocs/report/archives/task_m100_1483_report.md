# Task M100 #1483 최종 보고서 — 표 줄/칸 지우기 후 커서 cellIndex 보정

- 이슈: #1483 "[Bug] 표 줄/칸 지우기 후 커서 cellIndex 미보정 — updateRect '셀 인덱스 초과' 실패"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1483`
- 작성일: 2026-06-23

## 1. 개요

확장(크롬) 테스트 중 발견. 표 줄/칸 지우기로 셀 수가 줄면 커서가 더 이상 존재하지 않는
cellIndex를 가리켜 `updateRect`가 "경로[0]: 셀 인덱스 N 초과 (총 M개)"로 실패했다.
PR #1482(표 줄/칸 입력·지우기) 머지 후 드러난 커서 후처리 누락을 정정한다.

## 2. 원인

`rhwp-studio/src/command/commands/table.ts` `applyTableDeleteRowColumn` 의 `operation`
콜백이 삭제 후 `return pos` 로 삭제 전 커서 위치(원래 cellIndex)를 그대로 반환했다.
삭제로 셀 수가 줄면 cellIndex 가 새 범위를 초과 → `getCursorRectInCell` →
`field_query.rs` 셀 인덱스 검증에서 거부.

(줄/칸 추가는 셀 수가 늘어 기존 cellIndex 가 유효 → 무영향. "셀 전체 선택 후 크기 조절"은
cellIndex 미변경 → 무영향. 삭제 경로만 결함.)

## 3. 변경

### `rhwp-studio/src/command/commands/table.ts`

- `clampedCellAfterDelete` 헬퍼: 삭제 후 표 크기(rowCount/colCount) 내로 (row,col) clamp,
  `getTableCellBboxes` 로 cellIdx 역조회(병합 셀은 rowSpan/colSpan 범위 매칭), 표 소멸 시 null.
- `applyTableDeleteRowColumn`: `deleteTable*` 반환 `{ ok, rowCount, colCount }` 활용,
  보정된 cellIndex/cellParaIndex 를 커서에 반영, 표 소멸 시 본문 위치 폴백.

WASM/Rust 변경 없음 — 기존 반환값 + `getTableCellBboxes` 재사용.

### `rhwp-studio/tests/table-delete-cursor-1483.test.ts` (신규)

소스 정적 검사 회귀 가드 — 보정 로직과 헬퍼 구성을 검증하여 "return pos 단독" 회귀 차단.

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| 신규 `table-delete-cursor-1483` 테스트 | 2 passed |
| 전체 단위 테스트 (`npm test`) | 126 passed / 0 failed (기존 124 + 신규 2) |
| 기존 표 편집 테스트 | 회귀 없음 |
| `rhwp-studio` 빌드 (tsc + vite) | 통과 |

시나리오 커버(코드 분기): 행 삭제 / 열 삭제 / 표 소멸(본문 폴백).

## 5. 영향

- 표 줄/칸 지우기 후 커서가 항상 유효 셀(또는 표 소멸 시 본문)을 가리켜 `updateRect`
  "셀 인덱스 초과" 오류가 해소된다.
- rhwp-studio(웹·확장 공통) 한정 변경. 줄/칸 추가·크기 조절 경로 무영향.

## 6. 후속

- 본 수정의 확장 반영: main 반영 후 확장 재패키징 시 자동 포함(별도 릴리즈 단계).
- 실제 브라우저 동작(updateRect 무경고)은 작업지시자 확장 테스트로 최종 확인.

## 7. 산출물

- 수행계획서: `mydocs/plans/task_m100_1483.md`
- 구현계획서: `mydocs/plans/task_m100_1483_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1483_stage1.md`, `_stage2.md`
- 최종 보고서: 본 문서
- 소스: `rhwp-studio/src/command/commands/table.ts`, `tests/table-delete-cursor-1483.test.ts`
