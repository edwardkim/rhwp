# Task #1151 v2 Stage 1 완료 보고서 — floating→inline tac migration helper + 6 단위 테스트

수행계획서: [task_m100_1151_v2.md](../plans/task_m100_1151_v2.md) · 구현계획서: [task_m100_1151_v2_impl.md](../plans/task_m100_1151_v2_impl.md) · 한컴 동작 검증: [hancom_picture_tac_toggle.md](../tech/hancom_picture_tac_toggle.md)

## 1. 변경 내용

### `src/document_core/commands/object_ops.rs`

#### 신규 helper — `DocumentCore::migrate_picture_floating_to_inline`

floating picture 의 `treat_as_char` 가 false→true 로 토글될 때 한컴 정합 4가지 갱신을 수행하는 associated function (라인 395-447 근방).

```rust
pub(crate) fn migrate_picture_floating_to_inline(
    line_segs: &mut Vec<crate::model::paragraph::LineSeg>,
    pic: &mut crate::model::image::Picture,
)
```

처리:
1. picture: `horz_rel_to=Para`, `vert_rel_to=Para`, `horizontal_offset=0`, `vertical_offset=0`. (tac 비트는 `apply_picture_props_inner` 가 처리.)
2. `line_segs[0]`: `line_height = picture.common.height`, `text_height = picture.common.height`, `baseline_distance = round(line_height × 0.85)`.
3. `line_segs` 가 비어있으면 신설 (line_spacing=600).
4. paragraph.text / char_offsets / char_shapes / paragraph 수 / picture 위치 — 변경 없음.

baseline 비율 0.85 는 한컴 산출물 4 시나리오 (5331/16038/4847/19019) 모두 정확 관찰.

#### `set_picture_properties_native` 의 migration 분기

라인 244-269 근방:
- `was_tac` snapshot (apply_picture_props_inner 호출 전 picture 의 현재 treat_as_char).
- apply_picture_props_inner 호출 후 `now_tac` 확인.
- `!was_tac && now_tac` 이면 split borrow (`Paragraph { line_segs, controls, .. }`) 로 helper 호출.
- WASM/TS API 변경 없음.

#### 단위 테스트 모듈 `issue_1151_v2_tac_toggle_tests`

라인 5843 이후. 6 케이스 + 1 LineSeg::default 검증 = 총 7 테스트.

| 테스트 | 한컴 산출물 등가 | 핵심 단언 |
|--------|------------------|-----------|
| `tac_toggle_table_sibling_floating_to_inline` | scenario-a (1×1, 작은 picture, pic_h=5331) | picture 위치 불변, h/v_rel_to=Para, offset=0, line_segs[0].line_height=5331, baseline_distance=round(5331×0.85)=4531, text/char_offsets 불변 |
| `tac_toggle_body_floating_to_inline` | scenario-d (본문 floating, pic_h=19019) | 동일, lh=19019, baseline=16166 |
| `tac_toggle_3x3_center_cell_floating_to_inline` | scenario-c (3×3 (1,1) 셀, pic_h=4847) | 동일, lh=4847, baseline=round(4847×0.85)=4120 |
| `tac_toggle_when_already_tac_true_no_migration` | — | 이미 tac=true 인 picture 의 다른 속성 변경 시 line_height 추가 변동 없음, brightness 만 적용 |
| `tac_toggle_true_to_false_no_migration_this_pr` | — | tac=true→false 토글은 한 방향만이므로 migration 미진입, line_height 그대로 |
| `tac_toggle_with_empty_line_segs_creates_new_seg` | — | 빈 line_segs paragraph 의 토글 시 line_segs 신설 |

test helper: `make_test_core`, `minimal_png`, `parse_idx`, `push_body_floating_picture`, `expected_baseline`.

## 2. TDD 사이클

| Step | 결과 |
|------|------|
| RED | 6 단위 테스트 작성 → migration helper 부재로 6/6 FAIL (apply_picture_props_inner 가 h/v_rel_to/offset/line_segs 갱신 안 함) ✓ |
| GREEN | helper 신설 + set_picture_properties_native 분기 추가 → 6/6 PASS ✓ |
| REFACTOR | doc comment 의 nested list 를 clippy doc_lazy_continuation / doc_overindented_list_items 경고 회피용 flat 텍스트로 정리 |

## 3. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib issue_1151_v2_tac_toggle` | **7 passed, 0 failed** ✓ |
| `cargo test --lib` (전수) | **1432 passed, 0 failed, 6 ignored** (회귀 0) ✓ |
| `cargo clippy --lib -- -D warnings` | clean ✓ |
| `cargo fmt --all -- --check` | clean ✓ |

## 4. 한컴 정합 검증

`samples/tac-verify/scenario-{a,b,c,d}-after.hwp` 의 dump 분석과 model 단언 일치:

| 항목 | 한컴 산출물 | rhwp 단위 테스트 결과 |
|------|------------|----------------------|
| picture 의 paragraph 위치 | 같은 paragraph 의 같은 control_idx | 단언 (`controls.len()` 불변, `Control::Picture` 위치 유지) ✓ |
| `treat_as_char` | true | 단언 ✓ |
| `horz_rel_to / vert_rel_to` | Para / Para | 단언 ✓ |
| `h/v_offset` | 0 / 0 | 단언 ✓ |
| `line_segs[0].line_height` | = picture height (1000→5331 등) | 단언 ✓ |
| `line_segs[0].baseline_distance` | round(lh × 0.85) | 단언 ✓ |
| `paragraph.text` / `char_offsets` | 불변 | 단언 ✓ |
| paragraph 수 | 불변 | 단언 ✓ |

## 5. Stage 2 진입 조건

- migration helper + 분기 시그니처 확정 ✓
- 본문 회귀 / v1 회귀 0 ✓
- 단위 검증 통과 ✓
- 한컴 산출물 dump 와 model 단언 정합 ✓

→ Stage 2 (WASM 빌드 + 브라우저 시각 검증 + 한컴 산출물 SVG 1:1 비교) 진행 가능.
