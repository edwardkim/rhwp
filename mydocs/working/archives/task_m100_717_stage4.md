# Task M100 #717 Stage 4 완료보고서

## 단계 목표

Rust native hitTest 정정을 통합 검증하고, WASM 및 `rhwp-studio` 빌드 경로에서 산출물이 정상 생성되는지 확인한다.

## 수행 내용

- #717 native 회귀 테스트 실행.
- 관련 hitTest/selection 회귀 테스트 실행.
- 전체 lib release 테스트 실행.
- Docker WASM 빌드 실행.
- WASM 빌드 후 `rhwp-studio` production build 재실행.
- 오늘할일 상태를 완료로 갱신.

## 검증 결과

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_717_table_cell_hit_test -- --nocapture` | 통과, 2 passed |
| `cargo test --test issue_595` | 통과, 5 passed |
| `cargo test --test issue_658_text_selection_rects` | 통과, 2 passed |
| `cargo test --lib --release` | 통과, 1165 passed / 2 ignored |
| `docker-compose --env-file .env.docker run --rm wasm` | 통과, `pkg/` 생성 |
| `cd rhwp-studio && npm run build` | 통과, `rhwp_bg-TOnX6gki.wasm` 산출 |

참고:

- 이 환경에서는 `docker compose --env-file ...`가 지원되지 않아 `docker-compose --env-file ...`로 실행했다.
- `cargo test --lib --release`에서 기존 warning 5건이 출력됐으나 실패는 없었다.
- `rhwp-studio` 빌드는 Vite chunk size warning을 출력했으나 실패는 없었다.

## 변경 요약

- `src/document_core/queries/cursor_rect.rs`
  - `cell_bboxes` 메타 보완 시 Table 노드 메타가 이미 있으면 덮어쓰지 않도록 정정.
  - 메타 없는 bbox 보완은 TextRun bbox가 셀 bbox 내부에 있는 경우로 제한.
  - 클릭 좌표에 포함되는 셀 bbox 후보는 `has_meta`가 있는 셀로 제한.
  - 여러 셀 bbox가 겹치면 가장 작은 bbox를 선택.
- `tests/issue_717_table_cell_hit_test.rs`
  - `samples/exam_social.hwp` 기반 native hitTest 회귀 테스트 추가.
  - 문제 좌표 `(page=0, x=191, y=356)` 검증.
  - `<보기>` 표 빈 영역 좌표 `(page=0, x=110, y=865)` 검증.
  - 반환 컨텍스트와 `cursorRect.y`가 클릭한 표 bbox 안에 남는지 검증.

## 산출물

- `mydocs/plans/task_m100_717.md`
- `mydocs/plans/task_m100_717_impl.md`
- `mydocs/working/task_m100_717_stage1.md`
- `mydocs/working/task_m100_717_stage2.md`
- `mydocs/working/task_m100_717_stage3.md`
- `mydocs/working/task_m100_717_stage4.md`
- `tests/issue_717_table_cell_hit_test.rs`
- `mydocs/report/task_m100_717_report.md`

## 잔여 사항

- 실제 web editor 화면에서 클릭 후 커서가 대상 표/셀 내부에 들어가는지 작업지시자 시각 판정이 필요하다.
- 코드 레벨 native hitTest와 WASM/빌드 검증은 통과했다.
