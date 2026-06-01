# Task #1151 v8/v9 통합 Stage 28 완료 보고서 — 한컴 native 정합 (결함 A~F)

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) (v5~v9 모두 v4 PR #1173 머지 전 same-PR fix) · 상위 v7 Stage 16: [task_m100_1151_v7_stage16.md](task_m100_1151_v7_stage16.md)

## 1. 사용자 한컴 native 시연 (2026-05-30)

PR #1173 (v7 까지) 발행 후 사용자가 HWP 5.0 스펙 + 한컴 native 동작과 정합 여부 검토 요청 + 한컴 윈도우 편집기에서 직접 시연. **6 개 결함 확정**.

## 2. 결함별 fix 요약

| # | 결함 | Stage | 위치 | Fix |
|---|------|-------|------|-----|
| **A** | v1 셀 분기 picture rel_to=Page (한컴 native = Paper) | 18 | object_ops.rs:1634 | attr bits + typed field 모두 Paper |
| **B** | dialog horzRelSelect 옵션에 Para 누락 | 19 | picture-props-dialog.ts:495 | Para 옵션 추가 |
| **C** | picture 위치 = 셀 좌상단 (한컴은 사용자 클릭 위치) | 20 | object_ops.rs / wasm_api / wasm-bridge / input-handler-table | insertPicture 시그니처에 paper_offset_x/y_hu (Option<i32>) 추가, studio 가 drag 좌표 변환 후 전달 |
| **D** | 동일 paragraph sibling 2 TAC picture 가 가로 분배 안 됨 | 22-25 + 추가 fix | layout.rs:layout_shape_item + paragraph_layout.rs | ParaInlineState struct + collect_sibling_tac_picture_widths_px helper + 가로 분배 cursor + alignment + line wrap. pic_y 결정을 pic_x wrap 처리 뒤로 옮김 (wrap 순서 결함 추가 fix) |
| **E** | 본문 picture default tac=true (한컴 native = tac=false) | 27a + 27b | object_ops.rs:1707~ + input-handler-table.ts | 본문 inline path 전체 삭제 + 셀 분기와 통합 (floating sibling, tac=false, Paper offset). studio 의 paper offset 변환 inCell 제한 제거 → 본문에서도 사용자 클릭 좌표 전달 |
| **F** | insert_picture_native 의 invalidate_page_tree_cache 호출 누락 (v5 와 동일 패턴) | 26 | object_ops.rs 셀 분기 + 본문 분기 | invalidate_page_tree_cache() 호출 추가 (1 줄 × 2) |

`attr stale 결함 가설 (v2 migrate 의 attr bits 미갱신)` → 사용자 시연 결과 한컴/rhwp 동일 동작으로 fix 불필요 reject.

## 3. v9 핵심 — 동일 paragraph sibling 2 TAC picture 가로 분배 (결함 D)

### 시나리오
- 1×1 표 + 동일 셀에 picture 2 장 (v1 셀 분기 → outer paragraph 의 sibling controls)
- 둘 다 글자처럼 토글 (tac=true)
- 한컴 native = 가로로 inline 분배 (inline glyph 처럼) + 가용 폭 초과 시 wrap

### Root cause (Phase 1 Explore + Phase 2 Plan agent)
- paragraph_layout.rs 의 v3 path 가 호출 안 됨 — 진짜 path = `layout.rs:layout_shape_item` (build_columns → layout_column_item → 각 control 별 호출)
- `has_prior_tac_in_para` 가 모든 TAC 종류 동일 처리 → Pic2 호출 시 `para_start_y` 갱신 → 세로 진행

### Fix 설계 (Option 4)
1. `ParaInlineState` struct + `collect_sibling_tac_picture_widths_px` helper (Stage 22)
2. `layout_column_item` + `layout_shape_item` 시그니처 확장 — `para_inline_state: HashMap<usize, ParaInlineState>` 추가
3. `has_prior_tac_in_para` 분리 — Table/Shape vs Picture (Picture-only 시퀀스에서 `para_start_y` 갱신 X)
4. Picture + tac=true 분기 안에 시퀀스 위치 판별:
   - `is_single_pic` / `is_first_in_seq` / `is_subsequent_in_seq` / `is_last_in_seq`
   - 단일 picture: 기존 alignment + result_y 진행 그대로 (회귀 0)
   - 시퀀스 첫 picture: total_tac_width 기반 alignment + state 초기화
   - 시퀀스 후속 picture: state.cursor_x 사용 (가로 누적) + wrap 처리 (Stage 24)
   - 시퀀스 중간 picture: result_y = y_offset (유지)
   - 시퀀스 마지막 picture: result_y = line_top_y + max(line_height, line_advance)
5. **pic_y 결정을 pic_x 처리 뒤로 옮김** — wrap 후 갱신된 `state.line_top_y` 사용 (사용자 시연으로 발견한 추가 결함)

### Regression tests (v9)
- `v9_two_tac_pictures_horizontal_distribute`: 작은 picture (75.6 px each) — y 동일, x 누적 ✓
- `v9_two_large_pictures_wrap_to_next_line`: 큰 picture (302.4 px each, 합 > avail_w) — y_diff = pic_h (wrap), x 동일 (다음 line 좌측) ✓
- `issue1151_v9_insert_picture_body_floating_default`: 본문 picture default = tac=false, rel_to=Paper, sibling control ✓

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1454 passed, 0 failed, 6 ignored** (v8 신규 1 + v9 신규 3 추가, 회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v1 셀 안 picture 신규 삽입 시각 시연 | PASS (v8 결함 A+B+C fix 후) |
| 셀 안 picture 글자처럼 토글 → 표 아래 inline | PASS (v6 + v9 cache invalidate) |
| **동일 셀 picture 2 장 + 토글 → 가로 분배 / wrap** | **PASS (v9 결함 D fix)** |
| **본문 picture 신규 삽입 → floating 미체크 + 종이 기준 + 사용자 그린 위치** | **PASS (v9 결함 E + 27b studio fix)** |

## 5. 변경 파일 (v8/v9 통합)

### Rust
- `src/document_core/commands/object_ops.rs`
  - v8 결함 A: 셀 분기 Paper rel_to
  - v8 결함 C: insert_picture_native + 시그니처에 paper_offset_x/y_hu
  - v9 결함 E: 본문 inline path 전체 삭제 → 셀 분기와 통합 (sibling control, tac=false)
  - v9 결함 F: invalidate_page_tree_cache 호출 (셀 분기 + 본문 분기)
- `src/wasm_api.rs`: insert_picture 시그니처 paper_offset 매개변수 추가 (v8 C)
- `src/renderer/layout.rs`: layout_column_item + layout_shape_item 시그니처 + Picture + tac=true 분기 가로 분배 로직 (v9 D)
- `src/renderer/layout/paragraph_layout.rs`:
  - `collect_sibling_tac_picture_widths_px` helper (v9 D Stage 22)
  - `ParaInlineState` struct (v9 D Stage 22)

### Studio
- `rhwp-studio/src/ui/picture-props-dialog.ts:495`: horzRelSelect 옵션에 Para 추가 (v8 B)
- `rhwp-studio/src/core/wasm-bridge.ts`: insertPicture wrapper paperOffsetXHu/YHu 매개변수 추가 (v8 C)
- `rhwp-studio/src/engine/input-handler-table.ts:finishImagePlacement`:
  - paper offset 변환 path 추가 (v8 C)
  - inCell 제한 제거 → 본문에도 적용 (v9 E Stage 27b)

### 테스트 (신규)
- v8: `v8_cell_floating_picture_uses_paper_rel_to`
- v9 helper (Stage 22, 6 단위): `issue_1151_v9_helper_tests`
- v9 결함 D (Stage 23+24+25): `v9_two_tac_pictures_horizontal_distribute`
- v9 결함 D fix v2 (wrap 순서): `v9_two_large_pictures_wrap_to_next_line`
- v9 결함 E (Stage 27a): `issue1151_v9_insert_picture_body_floating_default`

## 6. Stage 28 진입 조건

- 결함 A+B+C+D+E+F 모두 fix + 사용자 시연 정합 통과 ✓
- 자동 검증 (test/clippy/fmt) clean ✓
- 회귀 0 (1446 → 1454, 신규 8) ✓
- 사용자 시연:
  - 본문 picture 신규 삽입 → 위치 = 사용자가 그린 곳 ✓
  - 동일 셀 picture 2 장 + 토글 → 가로 분배 (작은) / wrap (큰) ✓
  - 셀 안 picture 시연 (v8 까지의 결과) 회귀 0 ✓

→ Stage 28: v8/v9 통합 최종 보고서 + orders 갱신 + PR #1173 본문 갱신 + commit + push origin.
