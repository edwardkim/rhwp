# Task #2010 Stage 2

## 목표

실제 대형 문서 입력 랙을 재현한 결과 자동저장보다 셀 bbox 조회가 더 큰 병목임을 확인했다.
Stage 2에서는 일반 커서 이동/텍스트 입력 경로에서 셀 bbox 전체 조회를 피한다.

## 재현 결과

샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwp`

- 자동저장 기본 10초: 한 글자 입력 약 3.6초, long task 최대 약 2.2초
- 자동저장 비활성: 한 글자 입력 약 3.8초, long task 최대 약 2.3초
- 자동저장 `exportHwp()` 호출은 발생하지 않았다.
- `wasm.getTableCellBboxes(0, 0, 2, 0)` 호출이 클릭/편집 후 각각 약 2.0초 걸렸다.

## 원인

`InputHandler.emitCursorFormatState()`가 셀 내부 커서 상태를 알릴 때 눈금자 셀 폭 표시를 위해
`getTableCellBboxes()`를 rAF에서 호출한다. 대형/중첩 표 문서에서는 이 API가 렌더 트리/표 bbox를 크게
순회하면서 main thread를 장시간 점유한다.

## 구현 계획

- 일반 커서 갱신/텍스트 입력 경로에서는 셀 bbox 전체 조회를 하지 않는다.
- 셀을 벗어난 상태는 기존처럼 `cursor-cell-changed`로 알린다.
- 셀 너비 눈금자 표시가 꼭 필요한 별도 표 조작 경로는 후속으로 좁은 API 또는 명시적 조회 지점에서 다룬다.
- 표 셀 내부 단일 텍스트 입력은 현재 페이지를 먼저 갱신하고, 전체 페이지네이션은 idle 시점에 한 번만 flush한다.

## 구현 결과

- `emitCursorFormatState()`는 새 `getTableCellBboxes()`를 호출하지 않고 기존 bbox 캐시만 사용한다.
- passive hover와 일반 mousedown 리사이즈 판정은 새 bbox 생성을 하지 않는다.
- `insertTextInCellDeferredPagination()` WASM API를 추가해 page-local 단일 셀 입력에서 즉시 전체 페이지네이션을 생략한다.
- Studio는 page-local 입력 후 1.2초 idle 타이머로 `flushDeferredPagination()`을 호출해 페이지 수/전체 layout을 따라잡는다.

## 실측 결과

샘플: `samples/issue1949_giant_cell_nested_tables_perf.hwp` (115쪽)

| 항목 | 수정 전/중간 | 수정 후 |
| --- | ---: | ---: |
| 일반 클릭 | 약 2.48초 | 약 0.34초 |
| `getTableCellBboxes` | 1회, 약 2.15초 | 0회 |
| 한 글자 입력 API | `insertTextInCell` 약 1.01초 | `insertTextInCellDeferredPagination` 약 2ms |
| 키 입력 반환 | 약 1초대 블로킹 | 약 37ms |
| 5글자 연속 입력 | 입력 중 long task 발생 | 입력 중 long task 0회, 523ms |
| 전체 페이지네이션 | 입력마다 즉시 수행 | idle 후 1회, 약 0.93~0.96초 |

## 검증 계획

- 같은 샘플에서 한 글자 입력 시 long task와 입력 시간을 재측정한다.
- `cd rhwp-studio && npm test`
- `cd rhwp-studio && npm run build`
- `git diff --check`
