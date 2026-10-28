# M100 #2308 Stage 5 — 무효화·재사용·회귀 검증

## 기준

- 브랜치: `issue-2308-render-normalized-derived-state`
- 선행 Stage: `task_m100_2308_stage4.md`
- 완료일: 2026-07-23

## 보정과 회귀 고정

- deferred edit은 logical path revision만 증가시키고 section revision은 유지한다.
- immediate edit에서 #2004 projection이 존재하면 section revision을 올린 뒤 pagination에서
  source IR로 재파생한다.
- invalid path는 stale projection을 반환하지 않고 `RenderError`를 반환한다.
- #2195 overlay scale을 페이지 분할용 `nested_table_mixed_fragment_heights()`와 병합 셀,
  row-cut, partial nested table width 소비부까지 전달했다.
- `76076_regulatory_analysis.hwp` 33/34쪽의 중첩 1×1 표 fragment height를 통합 테스트로
  고정했다.
- source path 삭제, stable path 반복, unrelated edit 뒤 sibling projection의 `Arc` identity를
  unit test로 고정했다.

## focused 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --test issue_2308_render_normalized_derived_state` | 1 passed |
| `cargo test --test issue_2308_render_normalized_guard` | 1 passed |
| `cargo test --lib issue_2308_` | 3 passed |
| `cargo test --lib issue2308` | 2 passed |
| `cargo test --lib issue2214_deferred_table_caption_reports_flow_change` | 1 passed |
| `cargo test --test issue_2214_page_local_repaint` | 3 passed, 72.07s |
| `cargo test --test issue_2004_cell_image_stack_pagination` | 2 passed |
| `cargo test --test issue_1195_cell_table_empty_line` | 1 passed |
| `cargo test --test issue_1891` | 3 passed |
| `cargo test --test issue_1949_giant_cell_render_perf` | 1 passed, 65.55s |
| `cargo fmt --all` | PASS |

#1949 시간은 debug focused test의 단일 관측값이며 성능 SLA가 아니다. 기존 timeout과
페이지네이션 계약을 통과했다. 시각 geometry의 before/after 비교와 장기 기술 문서 반영은 Stage 6에서
닫는다.
