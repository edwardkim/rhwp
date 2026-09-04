# #6301 투명선 표시 시 병합된 셀 내부에 남은 옛 경계 안내선 제거

## 무엇을

"보기 > 투명선"(편집 전용 안내선, `show_transparent_borders`)이 켜진 상태에서 표를 셀
합치기·삭제·라인 숨김 등으로 편집하면, 편집된 형태를 반영하지 않고 원본 그리드가 그대로
점선 안내선으로 표시됐다. 특히 셀 병합 시 이제는 존재하지 않는 옛 내부 경계 위치에도
안내선이 남아 사용자에게 "편집이 반영되지 않은 것처럼" 보였다.

## 왜 (원인)

`render_transparent_borders()`(`src/renderer/layout/border_rendering.rs`)는
`h_edges`/`v_edges: Vec<Vec<Option<BorderLine>>>` 그리드에서 `None`인 슬롯에 점선 안내선을
그린다. 이 그리드는 `collect_cell_borders()`가 **각 셀 자신의 span 경계**에만 값을 채우는
방식으로 만들어지는데, 셀이 병합되면 병합 이전에 존재했던 내부 경계 위치는 병합 후 어느
셀도 다시 방문하지 않아 계속 `None`으로 남는다.

`render_transparent_borders()`는 이 `None`이 다음 두 경우 중 어느 쪽인지 구분하지 못했다.

1. 실제로 테두리가 없는 자리(사용자가 "테두리 없음"으로 지정) — 안내선을 그려야 함.
2. 병합으로 셀 내부가 되어 사라진 옛 경계 — 안내선을 그리면 안 됨.

두 경우 모두 그리드 값이 동일하게 `None`이라 (1)만 그려야 할 안내선이 (2)에도 그려졌다.
표 레이아웃은 일반(`table_layout.rs`)·페이지 분할 조각(`table_partial.rs`)·셀 콘텐츠
(`table_cell_content.rs`) 세 경로로 나뉘어 있고 세 경로 모두 동일한 `h_edges`/`v_edges`
수집·렌더링 패턴을 공유하므로 동일하게 영향을 받는다.

## 어떻게 (변경)

`mark_cell_span_interior_covered()`(`border_rendering.rs`)를 새로 추가했다. 셀 레이아웃 시
`collect_cell_borders()` 호출 지점마다 함께 호출해, 병합 셀의 `col_span`/`row_span` 내부
(자기 자신의 경계를 뺀 나머지) 위치를 별도의 커버리지 그리드
(`h_span_covered`/`v_span_covered: Vec<Vec<bool>>`)에 표시한다.

`render_transparent_borders()`는 `h_covered`/`v_covered` 파라미터를 추가로 받아
`edge_opt.is_none() && !is_covered(..)` 조건으로 안내선 그리기를 건너뛴다 — 그리드 값이
`None`이면서 **커버리지 표시가 없는** 위치(= 실제로 테두리 없음)에만 그린다.

세 표 레이아웃 경로 모두 동일하게 적용했다:

- `src/renderer/layout/table_layout.rs` — `layout_table_cells()` 시그니처에
  `h_span_covered`/`v_span_covered` 추가, 셀 루프에서 `collect_cell_borders()` 뒤
  (`independent_col_row_y.is_none()` 조건) 커버리지 기록.
- `src/renderer/layout/table_cell_content.rs` — 동일 패턴, 조건 없이 무조건 기록.
- `src/renderer/layout/table_partial.rs` — 세로쓰기/가로쓰기 두 셀 루프 모두, 조각
  가시 행 범위(`fri`/`lri`)를 `border_style` 조건 **이전**에 계산해 `border_style`이
  `None`이어도(=병합 등으로 테두리 스타일 자체가 없는 셀도) 커버리지는 기록되도록
  구조를 바꿨다.
- `src/renderer/layout/border_rendering.rs` — `render_transparent_borders()` 시그니처에
  `h_covered`/`v_covered` 파라미터 추가, 수평/수직 엣지 루프 조건에 커버리지 확인 추가.

## 검증

### 회귀 테스트

`tests/cases/issue_6301_transparent_border_merge_guide.rs` — 빈 문서에 1×3 표(기본 실선
테두리)를 만들고, `show_transparent_borders`를 켠 상태에서 (0,0)~(0,1) 셀을 병합한다.
`build_page_render_tree()`로 얻은 렌더 트리에서 편집 전용(`editor_only`) `Line` 노드 개수를
병합 전후로 비교한다.

- 병합 전: 0개 (기본 실선 표라 전제 조건 확인용 기준선).
- 병합 후(수정 후): 0개.
- 병합 후(수정 전 코드로 일시 되돌린 상태): 1개 — `assertion left == right failed: left: 1,
  right: 0` 로 실패 확인. 확인 후 조건을 원복하고 재통과를 재확인했다.

### 실제 렌더링 비교 (수동, HWPX 표 편집 시나리오)

셀 병합·라인 숨김이 포함된 실제 HWPX 표 샘플로 SVG→PNG 렌더링 전후를 캡처해 비교했다.
편집된 영역(병합 셀 내부, 라인 숨김 위치)에서 수정 전에는 점선 안내선이 남아있었고
수정 후에는 사라졌다. 편집되지 않은 영역의 실선 테두리·기존 투명선 동작은 변화 없음을
확인했다.

- 수정 전: `~/Downloads/투명선_병합셀_수정전.png`
- 수정 후: `~/Downloads/투명선_병합셀_수정후.png`
- 전후 비교(diff 강조): `~/Downloads/투명선_병합셀_전후비교.png`

### 로컬 검증 게이트

- `cargo fmt --check` — 통과 (본 변경 4개 파일 기준; 저장소의 `tests/generated/*.rs`는
  review-worktree/CI에서만 생성되는 파생물이라 신선한 checkout에는 없음 — 무관한 사전
  조건이며 본 PR 범위 밖).
- `cargo clippy --lib -- -D warnings` — 경고 없음.
- `cargo test --test regression_suite_018 issue_6301` (로컬 review-worktree 전용
  `node scripts/rust-test-suite-manifest.mjs --prepare` + `run-rust-test.mjs`로 배정 확인
  후 실행) — 통과.

렌더러/레이아웃 변경의 전체 공식 회귀 범위(`release-test` 전체 `cargo nextest run`, Native
Skia 3종, `wasm-pack build`)는 로컬 환경에 `cargo-nextest`/Docker WASM 경로가 없어 이번
PR에서는 수행하지 못했다 — 리뷰 시 CI에서 확인 필요.
