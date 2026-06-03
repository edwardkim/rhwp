# Task M100 #717 Stage 3 완료보고서

## 단계 목표

셀 내부 클릭으로 판정된 뒤 해당 셀의 직접 TextRun을 찾지 못하거나, 여러 셀 bbox가 겹칠 때 본문 전체/다른 표 fallback으로 빠지는 위험을 점검하고 필요한 방어를 추가한다.

## 수행 내용

- `tests/issue_717_table_cell_hit_test.rs` 회귀 테스트 보강.
  - 공통 `assert_table_hit()` 헬퍼 추가.
  - 대상 자료 표 제목 행 빈 영역 좌표 검증 유지.
  - `<보기>` 표 빈 영역 좌표 추가 검증.
  - 반환 컨텍스트뿐 아니라 `cursorRect.y`가 클릭한 표 bbox 범위 안에 남는지도 검증.
- `src/document_core/queries/cursor_rect.rs::hit_test_native()`의 셀 bbox 선택 로직 보강.
  - 클릭 좌표가 포함되는 셀 bbox 후보 중 `has_meta`가 있는 후보만 사용.
  - 여러 bbox가 동시에 hit될 경우 bbox 면적이 가장 작은 셀을 선택.

## 정정 내용

Stage 2에서는 셀 bbox 메타 보완 오염을 막았다.

Stage 3에서는 그 이후의 셀 bbox 선택 자체를 보강했다.

정정 전:

- `cell_bboxes.iter().find(...)`가 첫 번째 포함 bbox를 선택했다.
- 중첩 표/외곽 표처럼 bbox가 겹치는 경우, 렌더 트리 순서에 따라 더 큰 외곽 bbox가 먼저 선택될 수 있었다.

정정 후:

- 메타가 있는 bbox만 후보로 사용한다.
- 후보가 여러 개면 면적이 가장 작은 bbox를 선택한다.
- 셀 내부로 판정된 좌표가 클릭한 표/셀 컨텍스트 안에 더 안정적으로 고정된다.

## 검증 결과

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 통과, 2 passed |
| `cargo test --test issue_595` | 통과, 5 passed |
| `cargo test --test issue_658_text_selection_rects` | 통과, 2 passed |

## 변경 파일

- `src/document_core/queries/cursor_rect.rs`
- `tests/issue_717_table_cell_hit_test.rs`
- `mydocs/working/task_m100_717_stage3.md`
- `mydocs/orders/20260508.md`

## 정리 사항

Stage 3 중 `cargo fmt`가 저장소 전체의 기존 포맷 기준과 맞지 않아 다수 tracked 파일을 변경했다. 이는 작업 범위 외 포맷 churn이므로 작업지시자 승인 후 tracked 변경을 되돌리고, #717 관련 변경만 다시 적용했다.

최종 현재 tracked diff는 다음 두 파일로 제한된다.

- `src/document_core/queries/cursor_rect.rs`
- `mydocs/orders/20260508.md`

신규 산출물은 untracked 상태로 유지된다.

## 다음 단계 요청

Stage 4 진행 승인을 요청한다.

Stage 4에서는 통합 검증을 수행하고, 필요하면 `rhwp-studio` 빌드/WASM 확인 후 최종 보고서를 작성한다.
