# Task #1138 Stage 1 완료 보고서 — render_tree + layout cell 정보 채우기

- 이슈: [#1138](https://github.com/edwardkim/rhwp/issues/1138)
- 수행계획서: [task_m100_1138.md](../plans/task_m100_1138.md) (v2, 5-layer)
- 브랜치: `local/task1138`
- 단계 commits: `5e6209c4` (plan v2) + `82cd36bb` / `6697ffb4` / `953cd8a0` (cherry-pick 197줄) + `ba61f602` (picture sites)

## 1. 목표

render_tree 5 노드에 cell 정보 필드 추가 + layout 시점 채우기 + getPageControlLayout export. 표 셀 내부 그림/도형 객체에 대한 hit-test cell 정보 propagate 의 선행 작업.

## 2. 작업 결과

### 2.1 render_tree.rs (5 노드 필드 추가)

ImageNode/RectangleNode/LineNode/EllipseNode/PathNode 각각에 신규 필드:
```rust
#[serde(default)]
pub cell_index: Option<usize>,
#[serde(default)]
pub cell_para_index: Option<usize>,
#[serde(default)]
pub outer_table_control_index: Option<usize>,
```

각 노드 `::new()` 호환 유지, `#[serde(default)]` 로 deserialize 하위 호환.

### 2.2 layout_shape_object 시그니처 확장 (shape_layout.rs)

신규 매개변수 `table_cell_ref: Option<(usize, usize, usize)>` = `(cell_idx, cell_para_idx, outer_table_ctrl_idx)`. 5 노드 생성 시 cell 정보 채움. 내부 재귀 호출 propagate (자식 / group / textbox-shape).

### 2.3 layout_cell_shape 시그니처 확장 (table_cell_content.rs)

신규 매개변수 `table_cell_ctx: Option<(usize, usize, usize, usize, usize, usize)>` = `(sec, outer_para, outer_table_ctrl, cell, cell_para, inner_ctrl)`. layout_shape_object 호출 시 cell 정보 전달.

### 2.4 layout_picture_full 시그니처 확장 (picture_footnote.rs)

신규 매개변수 `table_cell_ref` 추가. ImageNode 생성 시 cell 정보 채움. layout_picture wrapper 는 None propagate (호환).

### 2.5 호출자 업데이트

| 위치 | 변경 |
|------|------|
| `table_layout.rs:2549, 2570` (layout_cell_shape 호출) | `table_meta` 에서 cell ctx 추출하여 전달 |
| `table_partial.rs:995, 1017` (분할 표 layout_cell_shape) | 동일 패턴 |
| `shape_layout.rs:446` (layout_shape 본문 도형) | None 전달 (셀 외부) |
| `shape_layout.rs:688, 1361` (자식/group 도형) | `table_cell_ref` propagate |
| `shape_layout.rs:2132` (textbox 내 nested) | None (TODO 별도 후속) |
| `table_layout.rs:2224, 2283` (셀 안 picture 2 사이트) | `layout_picture` → `layout_picture_full` + cell ctx |
| `table_partial.rs:945, 968` (분할 표 셀 안 picture) | 동일 |
| `table_cell_content.rs:714` (layout_embedded_table 안 ImageNode struct literal) | `enclosing_ctx` 에서 cell 정보 추출하여 직접 채움 |

### 2.6 rendering.rs export (5 노드)

`get_page_control_layout_native` 의 Image/Rectangle/Line/Ellipse/Path 5 노드 export JSON 에 `cellIdx`/`cellParaIdx`/`outerTableControlIdx` 필드 추가 (Equation 패턴 따라).

### 2.7 노드 생성 site cell 정보 (전체 8+ 사이트)

| 사이트 | 노드 | 결과 |
|--------|------|------|
| shape_layout.rs:765 | Rectangle (struct literal) | cell info 채워짐 |
| shape_layout.rs:933~944 | Path (connector arc) | cell info 채워짐 |
| shape_layout.rs:977~990 | Line (connector no-arc) | cell info 채워짐 |
| shape_layout.rs:996~1011 | Line (일반 직선) | cell info 채워짐 |
| shape_layout.rs:1031~1038 | Ellipse | cell info 채워짐 |
| shape_layout.rs:1146 + × 3 | Path (16 spaces × 3) | cell info 채워짐 |
| picture_footnote.rs:149 | Image (layout_picture_full) | cell info 채워짐 |
| table_cell_content.rs:727 | Image (layout_embedded_table) | cell info 채워짐 |

## 3. 미완 작업 (사용자 시나리오 외)

| 항목 | 위치 | 이유 |
|------|------|------|
| 글상자 안 picture cell propagate | `shape_layout.rs:2175, 2195` | 셀 안 글상자 안 picture (드문 nested) — 별도 후속 |
| `layout_group_child_affine` propagate | `shape_layout.rs:513-` | 글상자 grouping case — 별도 후속 |
| `layout_textbox_content` propagate | `shape_layout.rs:1597-` | 글상자 nested case — 별도 후속 |
| 마스터 페이지 picture | `layout.rs:1434` | 셀과 무관 (None 유지) |

→ 사용자 시나리오 (`inner-table-01.hwp` 셀[5] 사각형 = Rectangle path) 는 Stage 1 로 cell 정보 채워짐.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 통과 (2분) |
| `cargo test --release --lib` | **1405 passed, 0 failed, 6 ignored** (회귀 0) |

## 5. 다음 단계 (Stage 2)

- `src/document_core/commands/object_ops.rs` 에 shape helper 2개 (`format_shape_props_inner`, `apply_shape_props_inner`) 신규
- get/set_cell_picture_properties_by_path_native (이전 폐기 commit 16282af3 cherry-pick 또는 재작성)
- get/set_cell_shape_properties_by_path_native (shape helper 활용)
- WASM API binding 4개
- `tests/issue_1138.rs` 단위 테스트
