# Task M100 #1483 2단계 완료보고서 — 회귀 검증 + 가드 테스트

- 이슈: #1483
- 브랜치: `local/task1483`
- 작성일: 2026-06-23
- 단계: 2/3

## 1. 신규 회귀 가드 테스트

`rhwp-studio/tests/table-delete-cursor-1483.test.ts` 추가 (소스 정적 검사 패턴, 기존
table-keyboard-navigation 테스트와 동일 방식).

- `applyTableDeleteRowColumn` 블록: `deleteTable*` 반환값 활용(res.ok/rowCount/colCount),
  `clampedCellAfterDelete` 호출, 보정 위치 반영(`...corrected`), 표 소멸 본문 폴백 가드.
  → "보정 없이 return pos만" 회귀를 막는다.
- `clampedCellAfterDelete` 헬퍼: 소멸 가드(rowCount/colCount<=0), row/col clamp,
  `getTableCellBboxes` 역조회, 병합 셀 rowSpan/colSpan 매칭.

## 2. 검증 결과

| 항목 | 결과 |
|---|---|
| 신규 `table-delete-cursor-1483` 테스트 | 2 passed / 0 failed |
| 전체 단위 테스트 (`npm test`) | **126 passed / 0 failed** (기존 124 + 신규 2) |
| 표 키보드 네비게이션 등 기존 표 테스트 | 회귀 없음 |
| tsc | 통과 |

## 3. 시나리오 커버 (코드 분기)

- 행 삭제(마지막 행 커서): row clamp → 유효 셀 cellIdx 역조회.
- 열 삭제(마지막 열 커서): col clamp → 동일.
- 표 소멸(1×1 마지막 행/열 삭제): rowCount/colCount<=0 → 본문 위치 폴백.

실제 브라우저 동작(updateRect 무경고)은 작업지시자 확장 테스트로 최종 확인.

## 4. 다음 단계

3단계: rhwp-studio 빌드 + 최종 보고서 + 오늘할일 갱신.
