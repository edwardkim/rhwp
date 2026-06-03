# Task M100 #717 Stage 2 완료보고서

## 단계 목표

`CellBboxInfo`가 다른 표의 동일 `cell_index` TextRun으로 덮이지 않도록 `hit_test_native()`의 셀 bbox 메타 보완 로직을 정정한다.

## 수행 내용

- `src/document_core/queries/cursor_rect.rs::hit_test_native()`의 `cell_bboxes` 보완 로직 수정.
- 기존 동작은 Table 노드에서 이미 채워진 메타도 TextRun으로 다시 덮어썼다.
- 정정 후 동작:
  - `cb.has_meta == true`이면 Table 노드 메타를 권위값으로 유지한다.
  - 메타가 없는 bbox만 TextRun으로 보완한다.
  - 보완 시에도 `cell_index` 단독 매칭을 사용하지 않고, TextRun bbox가 셀 bbox 내부에 있는 경우로 제한한다.
- `tests/issue_717_table_cell_hit_test.rs`에 `cursorRect.y` 검증 추가.
  - 대상 표 bbox 내부 y 범위 유지 확인.
  - 하단 번호 표(`y≈1393.7`)로 튀는 회귀 차단.

## 핵심 정정

정정 전:

- `cell_index == 0`인 셀 bbox가 첫 번째 `cell_index == 0` TextRun 메타로 덮였다.
- 여러 표와 바탕쪽 표가 동일한 낮은 `cell_index`를 반복 사용하므로 컨텍스트가 오염됐다.
- #717 좌표는 `s0:pi=1 ci=0` 표 내부인데도 `parentParaIndex=0/controlIndex=1`로 반환됐다.

정정 후:

- 대상 좌표가 `parentParaIndex=1/controlIndex=0`으로 반환된다.
- caret y 좌표도 대상 표 bbox 범위 안에 남는다.

## 검증 결과

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 통과, 1 passed |
| `cargo test --test issue_595` | 통과, 5 passed |
| `cargo test --test issue_658_text_selection_rects` | 통과, 2 passed |

## 변경 파일

- `src/document_core/queries/cursor_rect.rs`
- `tests/issue_717_table_cell_hit_test.rs`
- `mydocs/working/task_m100_717_stage2.md`
- `mydocs/orders/20260508.md`

## 위험도 평가

- 변경 범위는 `hit_test_native()`의 셀 bbox 메타 보완 루프 한 곳이다.
- 렌더링, 페이지네이션, 표 크기 계산은 변경하지 않았다.
- 기존 #595 header/footer hitTest 회귀 테스트와 #658 selection rect 회귀 테스트는 통과했다.
- 표 객체 선택 분기는 반환 JSON의 `parentParaIndex/controlIndex/cellIndex` 형태를 유지하므로 기존 TypeScript 경로와 호환된다.

## 다음 단계 요청

Stage 3 진행 승인을 요청한다.

Stage 3에서는 셀 내부 클릭 분기의 fallback 동작을 점검하고, TextRun이 없는 셀 또는 클릭 y 범위에 직접 맞는 run이 없는 셀에서도 본문 전체 fallback으로 빠지지 않도록 추가 방어가 필요한지 확인한다.
