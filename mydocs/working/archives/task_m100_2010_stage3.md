# Task m100 #2010 Stage 3

## 목표

- 대형 문서 표 셀 입력 후 예약된 `flushDeferredPagination()`이 4~5초 동안 UI thread를 막는 문제를 완화한다.
- 작은 문서는 자동 페이지 재계산을 유지하되, 대형 문서는 편집 중 자동 flush를 피하고 저장/인쇄 같은 명시 동작에서 정리한다.

## 관찰

- 실제 Chrome 탭에서 `Input.insertText` 기준 텍스트 삽입은 약 113ms 안에 끝났다.
- `insertTextInCellDeferredPagination` 자체는 0.3ms 수준이었다.
- 1.2초 뒤 실행된 `flushDeferredPagination()`이 약 4.7초 걸리며 long task를 만들었다.
- 따라서 남은 렉은 입력 반영이 아니라 너무 빠른 deferred pagination flush 정책 때문이다.

## 수정 방향

- `InputHandler`에 deferred pagination pending 상태와 명시 flush API를 둔다.
- 대형 문서에서는 page-local 편집 직후 자동 flush timer를 걸지 않는다.
- 저장/다른 이름 저장/인쇄 전에 pending pagination을 명시적으로 flush한다.

## 구현

- `rhwp-studio/src/engine/input-handler.ts`
  - 30쪽 초과 문서는 page-local 입력 후 자동 `flushDeferredPagination()` timer를 걸지 않는다.
  - 30쪽 이하 문서만 10초 idle 뒤 자동 flush한다.
  - `flushDeferredPaginationIfNeeded()`와 `hasDeferredPaginationPending()`을 추가했다.
- `rhwp-studio/src/command/commands/file.ts`
  - 저장, 다른 이름 저장, 인쇄 직전에 pending pagination을 명시적으로 flush한다.

## 검증

- `git diff --check`: 통과.
- `rhwp-studio npm run build`: 통과.
- `rhwp-studio npm test`: 171개 통과.
- `issue1949_giant_cell_nested_tables_perf.hwp` 115쪽 문서 headless 검증:
  - 입력 1회: 40ms.
  - `insertTextInCellDeferredPagination`: 0.9ms.
  - 2.2초 대기 중 `flushDeferredPagination`: 0회.
  - long task: 0회.
- 실제 Chrome 탭 검증:
  - 115쪽 문서 로드 후 표 셀 입력 1회: 156ms.
  - `insertTextInCellDeferredPagination`: 8.3ms.
  - 2.2초 대기 중 `flushDeferredPagination`: 0회.
  - long task: 78ms 1회.
- 명시 flush API 검증:
  - 대형 문서 pending 생성 후 1.5초 대기: 자동 flush 0회.
  - `flushDeferredPaginationIfNeeded('test-explicit')`: pending 해제 확인.
