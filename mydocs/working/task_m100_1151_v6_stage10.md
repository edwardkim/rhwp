# Task #1151 v6 Stage 10 완료 보고서 — Table::update_ctrl_dimensions self.common 미동기화 fix

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) (v5/v6 모두 v4 PR 머지 전 사용자 추가 발견 결함의 same-PR fix) · Stage 9 보고서: [task_m100_1151_v5_stage9.md](task_m100_1151_v5_stage9.md)

## 1. 사용자 보고

> "[v5 WASM 재빌드 후 시연] 동일하게 했을때 한컴 정합과 다른 모습 [스크린샷: 표가 picture + 빈 영역 포함한 큰 박스 + picture 가 좌상단 inline]" (2026-05-30)

명확화: 셀 size 를 picture 보다 크게 조절한 후에도 동일 결함 재현. → v5 fix (cache invalidate) 정상이나 추가 layer 결함.

## 2. 진단 단계

### 진단 1 — 한컴 정합 (scenario-a-after.hwp) render tree baseline
```
table  = (113.4, 132.3, 181.7, 166.6) px  → 표 height 166.6 px = 12498 HU (cell.height 그대로)
image  = (113.4, 306.5,  79.7,  71.1) px  → 표 아래 정확 위치
```
→ paragraph_layout 자체는 한컴 정합 model 에 대해 정확히 동작 (표 박스를 셀 size 로 그리고 picture 는 표 아래에 sibling inline 으로 배치). **paragraph_layout 정상 확인**.

### 진단 2 — rhwp v1 path + 셀 height 조절 후 render tree (fix 적용 전)
```
table.common.height = 1282     ← resize 안 됨!
cell[0].height = 11498         ← resize 적용

table = (113.4, 132.3, 559.4, 153.3) px  ← height 153 px = cell.height (큰 박스)
image = (113.4, 156.9,  79.7,  71.1) px  ← 표 안 좌상단!
```
→ paragraph_layout 의 v3 helper `calc_sibling_topandbottom_table_reserved_hu` 가 `t.common.height` (작은 stale 값 1282) 사용 → reservation 부족 → picture y 가 표 박스 안으로 들어감. 사용자 스크린샷 정확 재현.

### 진단 3 — root cause 확정
`src/model/table.rs:255` `Table::update_ctrl_dimensions` 가 `raw_ctrl_data` 의 width/height bytes 만 갱신하고 **`self.common.width` / `self.common.height` 는 동기화하지 않음**.

`resize_table_cells_native` (table_ops.rs:800) 가 cell 갱신 후 `update_ctrl_dimensions` 호출하지만, paragraph_layout 의 v3 helper 가 `t.common.height` (model field) 사용 — raw_ctrl_data 와 분리되어 있어 stale.

## 3. Fix

`src/model/table.rs:Table::update_ctrl_dimensions`:
```rust
self.raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
self.raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
// [Task #1151 v6] self.common 동기화 — raw_ctrl_data 만 갱신하던 결함 fix.
self.common.width = total_width;
self.common.height = total_height;
```

## 4. Regression 테스트 (2개 추가)

### `v6_render_tree_scenario_a_after_baseline`
한컴 정합 model 의 render tree 가 표 + picture 분리 배치 (image y > table bottom) 임을 검증 — baseline 보장.

### `v6_resize_cell_then_tac_toggle_picture_below_table`
rhwp v1 path + 셀 height 조절 (cell[0].height += 11216) + tac toggle 후:
- (A) `table.common.height == 11498` — self.common 동기화 검증
- (B) render tree 의 image y > table bottom — 표 박스 그리기 + picture 위치 정합

Fix 적용 전 (B) FAIL (image y=156.9 < table bottom=285.6), Fix 후 PASS (image y=293.1 > 285.6).

## 5. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1445 passed, 0 failed, 6 ignored** (v5 신규 1 + v6 신규 2 추가, 회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v2 Stage 1 단위 테스트 6 | PASS 유지 |
| v2 Stage 2 통합 테스트 4 | PASS 유지 |
| v3 helper 단위 테스트 6 | PASS 유지 |
| v5 신규 regression 1 | PASS 유지 |
| v6 신규 regression 2 | PASS |

## 6. 사용자 시연 검증 절차 (WASM 재빌드 후)

1. `docker compose --env-file .env.docker run --rm wasm` — WASM 재빌드
2. 브라우저 새로고침
3. 신규 빈 문서 → 표 1×1 → **셀 크기를 picture 보다 크게 조절** → 셀 안 picture 삽입 → 그림 속성 "글자처럼 취급" 체크 → 확인
4. 기대: picture 가 표 아래 inline 위치로 정확 배치 (표 박스는 셀 size 그대로, picture 가 박스 안에 안 들어감)

## 7. 변경 파일 (v6 Stage 10)

- `src/model/table.rs:261-263` — update_ctrl_dimensions 에서 self.common.width/height 동기화 (2 줄 추가)
- `src/document_core/commands/object_ops.rs` — v6 regression 테스트 2 개 추가

## 8. Stage 11 진입 조건

- root cause 확정 + 단순 fix (2 줄) + regression 2 개 ✓
- 전수 회귀 0 (1442 → 1445) ✓
- v4 (click hit-test) + v5 (cache invalidate) + v6 (table common 동기화) 모두 완성 ✓

→ Stage 11 (통합 PR v1+v2+v3+v4+v5+v6 + 최종 보고서) 진행 가능.
