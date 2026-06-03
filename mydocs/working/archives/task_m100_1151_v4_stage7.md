# Task #1151 v4 Stage 7 완료 보고서 — 셀 안 inline picture click hit-test 완성

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) · 구현계획서: [task_m100_1151_v4_impl.md](../plans/task_m100_1151_v4_impl.md) · Stage 6 보고서: [task_m100_1151_v4_stage6.md](task_m100_1151_v4_stage6.md)

## 1. 진단 단계 — 진짜 진입점 탐색

Stage 6 의 1차 fix (paragraph_layout 3 곳 + cursor_rect 분기 + rendering.rs JSON) 적용 후 사용자 시연:
- 1차 시연: 클릭 안 됨 → cell_ctx 전달 path 추적 → layout_picture_full (`picture_footnote.rs:57`) 이 진짜 진입점 확정 (셀[5] picture SVG 좌표 정확 일치).
- 2차 시연: 클릭 ✓ 통과 → 우클릭 → 개체 속성 → 에러 "지정된 컨트롤이 그림이 아닙니다" → setter/getter 가 본문 path 호출 결함 진단.
- 3차 시연: 동일 에러 → `insert.ts:190` 의 cellPath 구성 조건이 picture 제외 결함 + outerTableControlIdx JSON 노출 누락 진단.
- 4차 시연: 12 페이지만 잔존 → paragraph_layout 의 outer_table_control_index / para_index 정규화 진단.
- 5차 시연: 12페이지 ✓ + 6/7 페이지 잔존 → 본문 paragraph 의 사각형 글상자 안 picture (이중 nested) — 별개 결함 영역 확정.

## 2. 변경 내용

### Rust 측 (사용자 layer)

#### `src/renderer/render_tree.rs` — ImageNode struct 확장
`cell_index` / `cell_para_index` / `outer_table_control_index: Option<usize>` 3 필드 추가 ([Task #1138] Rectangle 패턴). `ImageNode::new()` default None.

#### `src/renderer/layout/picture_footnote.rs` — `layout_picture` / `layout_picture_full` 시그니처 확장
- `cell_ctx: Option<&CellContext>` 인자 추가.
- ImageNode 생성에 `cell_index` / `cell_para_index` / `outer_table_control_index` 설정.
- `tac=true` 인 경우 `set_inline_shape_position` 호출 — `inline_shape_positions` HashMap 에 등록.

#### `src/renderer/render_tree.rs:1002-1040` — `set_inline_shape_position` / `get_inline_shape_position`
[Task #1151 v4] 셀 안인 경우 InlineShapeKey 의 para 를 `cell_ctx.parent_para_index` (outer paragraph idx) 로 정규화. cursor_rect 의 `section.paragraphs.get(pi)` resolve 와 정합.

#### `src/renderer/layout/paragraph_layout.rs` — 3 곳 ImageNode 생성 갱신
- `para_index`: cell_ctx 가 Some 이면 outer paragraph idx 로 정규화 (TAC 표 안 picture 정합).
- `cell_index` / `cell_para_index` / `outer_table_control_index` 설정.
- `calc_sibling_topandbottom_table_reserved_hu` (v3) 동작 유지.

#### `src/renderer/layout/table_layout.rs:2253/2312` — 셀 안 picture caller
- 이전: `Some(section_index), None, None`
- 변경: `Some(section_index), cell_context.parent_para_index, Some(ctrl_idx), Some(&cell_context)`

#### `src/renderer/layout/table_partial.rs:960/983` — partial table 셀 안 picture caller
- 동일 패턴: `Some(section_index)`, outer para idx, inner ctrl idx, `Some(&cell_context)` 전달.

#### `src/renderer/layout/shape_layout.rs:2182/2203` — 글상자 내부 picture caller
- 이전: `(None, None, None, None)` → controls JSON 의 secIdx undefined → studio skip.
- 변경: `Some(section_index), Some(para_index), Some(ctrl_idx_in_para), None`.
- cell_ctx 는 None (글상자 자체 본문 path).

#### `src/renderer/layout.rs:871/1445` — 머리말/꼬리말 + 바탕쪽 picture caller
- 시그니처 호환 위해 `None` 추가. 기존 동작 보존.

### Rust 측 (API layer)

#### `src/document_core/queries/rendering.rs:1495-1510` — Image JSON 직렬화 확장
- `cellIdx` / `cellParaIdx` / `outerTableControlIdx` 노출 (Rectangle 패턴).

#### `src/document_core/queries/cursor_rect.rs:1218-1245` — 셀 안 inline shape hit-test 분기
- 이전: `if !cell_path.is_empty() { continue; }`
- 변경: cell_path 가 있으면 outer paragraph → Table → cell → cell paragraph 경로로 control resolve. 응답 JSON 에 `cellPath` / `innerControlIdx` 추가.

#### `src/document_core/commands/object_ops.rs` — 셀 안 picture 속성 by_path API
- `get_cell_picture_properties_by_path_native` 신설 — Shape 의 `get_cell_shape_properties_by_path_native` 패턴 정합.
- `set_cell_picture_properties_by_path_native` 신설 — Shape setter 패턴 정합.

#### `src/wasm_api.rs` — WASM export
- `getCellPicturePropertiesByPath`
- `setCellPicturePropertiesByPath`

### Studio 측

#### `rhwp-studio/src/core/wasm-bridge.ts`
- `getCellPicturePropertiesByPath` / `setCellPicturePropertiesByPath` wrapper 추가 (Shape 패턴 정합).

#### `rhwp-studio/src/ui/picture-props-dialog.ts:242, 2153`
- getter 분기 (line 242): cellPath 분기 추가 (picture 도 셀 안일 때 `getCellPicturePropertiesByPath` 호출).
- setter 분기 (line 2153): 동일 패턴.

#### `rhwp-studio/src/command/commands/insert.ts:187-198`
- cellPath 구성 조건에 `ref.type === 'image'` 추가 (이전 shape / line 만).

## 3. 검증 결과

### 자동 검증
| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1442 passed, 0 failed, 6 ignored** (회귀 0) ✓ |
| `cargo clippy --lib -- -D warnings` | clean ✓ |
| `cargo fmt --all -- --check` | clean ✓ |
| v2 통합 테스트 4 (model 정합) | PASS 유지 |
| v3 helper 단위 테스트 6 (시각 layout) | PASS 유지 |

### 사용자 시연 검증 (tac-img-02.hwp)
| 페이지 | picture 위치 | 클릭 결과 |
|---|---|---|
| 1 | 표 sibling paragraph (sec=0 para=1) | ✓ |
| 8 | 표 → 셀[0] (para 0.82) | ✓ |
| 9 | 표 → 셀[0] (para 0.93) | ✓ |
| 10 | 표 → 셀[0] (para 0.109) | ✓ |
| 11 | 표 → 셀[0] (para 0.120) | ✓ |
| 12 | **TAC 표** → 셀[0] (para 0.130) | ✓ (outer_table_control_index + para_index 정규화 fix) |
| 14 | 표 → 셀[0] 3 picture (para 0.165) | ✓ (3개 모두) |
| **6, 7** | **사각형 → 글상자 → picture (para 0.25, 0.44)** | ✗ (별도 결함, #1171 분리) |

## 4. 별도 후속 task (#1171 발급)

본 v4 진단 중 발견: **사각형(Shape, InFrontOfText) 안 글상자(text_box) 안 paragraph 의 picture (이중 nested)** click hit-test 결함. shape_layout fix 로 sec/para/control 인덱스는 정상 노출되지만:
- 사각형 Shape 자체가 InFrontOfText 본문 위 layer 라 picture 위 클릭이 Shape 에 hit
- nested ImageNode 의 select dispatch / dialog 호출 경로 부재 (Task #825 의 머리말/꼬리말 5-tuple 패턴 유사 필요)

→ 별도 이슈 [#1171](https://github.com/edwardkim/rhwp/issues/1171) 으로 분리. v4 PR 머지 후 후속 진행.

## 5. nested 구조 정리표 (검증 완료)

| bin_id | parent paragraph | nested 구조 | v4 click 가능 |
|---|---|---|---|
| 2 | 0.1 | 표(TopAndBottom) → 셀[5] → p[0] → ctrl[1] | ✓ |
| 3 | 0.25 | 사각형(Shape) → 글상자(text_box) → p[0] → ctrl[0] | ✗ (#1171) |
| 4 | 0.25 | 사각형 → 글상자 → p[0] → ctrl[1] | ✗ (#1171) |
| 5 | 0.44 | 사각형 → 글상자 → p[0] → ctrl[0] | ✗ (#1171) |
| 6 | 0.82 | 표 → 셀[0] → p[0] → ctrl[0] | ✓ |
| 7 | 0.93 | 표 → 셀[0] → p[0] → ctrl[0] | ✓ |
| 8 | 0.109 | 표 → 셀[0] → p[0] → ctrl[0] | ✓ |
| 9 | 0.120 | 표 → 셀[0] → p[0] → ctrl[0] | ✓ |
| 10 | 0.130 | TAC 표 → 셀[0] → p[0] → ctrl[0] | ✓ |
| 11/12/13 | 0.165 | 표 → 셀[0] → p[0] → ctrl[0/1/2] (3개) | ✓ |
| 14, 16, 17, 19 | body | 본문 inline | (본문 path) |
| 15, 18 | body | 본문 floating (tac=false) | (본문 path) |

## 6. Stage 8 진입 조건

- v4 의 scope (셀 안 inline picture click hit-test) 완성 ✓
- 회귀 0 (v2 model 정합 + v3 시각 layout + v1 셀 안 picture 삽입 모두 유지) ✓
- 자동 검증 통과 ✓
- 사용자 시연 검증: 7 개 표 셀 안 picture 모두 click ✓ (#1171 별도)

→ Stage 8 (통합 PR v1+v2+v3+v4 + 최종 보고서) 진행 가능.
