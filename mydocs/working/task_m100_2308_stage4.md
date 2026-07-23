# M100 #2308 Stage 4 — #2195 중첩 표 너비 정규화 이전

## 기준

- 브랜치: `issue-2308-hyper-waterfall-rebuild`
- 선행 Stage: `task_m100_2308_stage3.md`
- 완료일: 2026-07-23

## 구현

- #2195 `has_nested_stretch`와 `stretch_nested_tables_to_parent_cell()` clone 변환을 제거했다.
- source `Table.common.width`와 nested `Cell.width`는 수정하지 않는다.
- `NestedTableWidthProjection`에 source width, effective width, width scale만 저장한다.
- `HeightMeasurer`와 `LayoutEngine`이 같은 `RenderNormalizationOverlay`를 받는다.
- 열 폭·colspan constraint, cell content available width, border geometry, partial table 경로가
  source table과 overlay scale을 조합해 effective 값을 계산한다.
- `compute_render_normalized()`는 현재 source IR에서 overlay pointer index를 다시 만들고,
  stable logical path의 projection `Arc`를 재사용한다.
- deferred paragraph mirror와 clone용 caption sentinel 비교를 제거했다.

이 Stage가 끝난 뒤 일반 section과 #2195-only section에는 paragraph clone이 생성되지 않는다.
#2004 구조 projection만 revision 검증 immutable `Arc`로 남는다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --test issue_2308_render_normalized_guard` | 1 passed |
| `cargo test --lib render_normalization::tests` | 4 passed |
| `cargo test --test issue_1195_cell_table_empty_line` | 1 passed |
| `cargo test --test issue_1891` | 3 passed |
| `cargo fmt --all` | PASS |

source guard는 mutable section clone stretch, deferred mirror 함수, cache key의 caption sentinel이
모두 제거됐음을 확인한다. 시각 geometry와 deferred edit의 세부 회귀는 Stage 5에서 확장 검증한다.
