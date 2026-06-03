# Task #1151 v5 Stage 9 완료 보고서 — set_picture_properties_native cache invalidate 누락 fix

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) (v5 는 v4 PR 머지 전 사용자 추가 발견 결함의 same-PR fix) · 상위 Stage 7 보고서: [task_m100_1151_v4_stage7.md](task_m100_1151_v4_stage7.md)

## 1. 사용자 보고

> "hwp에서 verify/scenario 문서들을 만들고 그걸 rhwp에서 여는 작업을 했잖아. 그건 동일하게 진행되거든? 그런데 반대로 rhwp에서 셀을 하나 만들고 이미지를 넣으면 그대로 적용이 안됨" (2026-05-30)

명확화 (사용자 답변): **토글 시 v2 migration 미적용 (시각 변화 없음)** — picture 가 toggle 전 floating 위치 그대로 유지.

## 2. 진단 단계

stale WASM 가능성 → 사용자가 "v4 Stage 7 시연에서 click hit-test 작동 = WASM 최신" 으로 배제 (정확한 지적). 본격 Rust layer 진단.

### 진단 1 — model 정합 (단위 테스트 PASS)
`tac_toggle_table_sibling_floating_to_inline` (v2 Stage 1 단위 테스트) 가 정확히 v1 path → tac toggle → 한컴 정합 4 필드 + LINE_SEG[0] 갱신 검증하고 있고 PASS 중. → **model 자체는 정합**.

### 진단 2 — composer 정합 (eprintln 진단)
```
para.controls = [Table(wrap=TopAndBottom, h=1282, om_top=283, om_bot=283),
                 Picture(tac=true, wrap=Square, h=5331)]
composed.tac_controls = [(1, 5977, 1)]
composed.lines[0].line_height = 5331, runs.len = 0
v3 helper sibling_topandbottom_table_reserved_hu = 1848
```
→ composer 가 빈 paragraph + sibling Table+tac picture 를 정확히 처리.

### 진단 3 — paragraph_layout / render tree 정합 (eprintln 진단)
```
table bbox = (113.4, 132.3, 559.4, 17.1) → 표 끝 y ≈ 149.4
image bbox = (113.4, 156.9,  79.7, 71.1) → 표 + reservation 직후
```
→ paragraph_layout 의 v3 path (line 3169-3173) 가 정확히 호출되어 picture 가 표 아래 inline 위치에 그려짐.

### 진단 4 — root cause 확정
`set_picture_properties_native` (line 222-313) 의 line 292 `paginate_if_needed()` 직후 **`invalidate_page_tree_cache()` 호출 누락**. 다른 picture/shape setter 들 (line 2529 셀 shape by_path / 3039 셀 picture by_path / 3124 header-footer / 5539 / 5646 / 5863 shape) 은 **모두** 호출. 본 본문 picture setter 만 일관성 누락.

→ studio 가 `setPictureProperties` 호출 후 build_page_tree_cached 가 stale cache 반환 → picture 시각 변화 없음.

## 3. Fix

### 변경
`src/document_core/commands/object_ops.rs:292` (set_picture_properties_native): `paginate_if_needed()` 호출 직후 `self.invalidate_page_tree_cache();` 추가.

```rust
// 리플로우
let section = &mut self.document.sections[section_idx];
section.raw_stream = None;
self.recompose_section(section_idx);
self.paginate_if_needed();
// [Task #1151 v5] page tree cache invalidate — 다른 picture/shape setter (셀 shape
// by_path / 셀 picture by_path / header-footer picture / shape 등) 모두 호출하나
// 본 본문 picture setter 만 누락되어 있어 studio 가 stale page tree 반환 → tac toggle
// 후 시각 변화 없음 증상의 root cause.
self.invalidate_page_tree_cache();
```

### 회귀 테스트
`v5_tac_toggle_invalidates_page_tree_and_emits_inline_picture_below_table` 신규 추가:
- v1 path 로 표 1×1 + 셀 안 floating picture (h=5331) 삽입
- toggle 전 `build_page_tree_cached(0)` 호출 → cache 채움, picture y 기록
- `set_picture_properties_native(treatAsChar:true)` 호출
- toggle 후 `build_page_tree_cached(0)` 호출 → 새 picture y 기록
- (A) `assert! (y_before - y_after).abs() > 0.5` — cache invalidate 검증
- (B) `assert! y_after > table_bottom` — picture 가 표 아래 위치 (한컴 정합)

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1443 passed, 0 failed, 6 ignored** (v5 신규 1 추가, 회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v2 Stage 1 단위 테스트 6 (model 정합) | PASS 유지 |
| v2 Stage 2 통합 테스트 4 (한컴 산출물 양방향 정합) | PASS 유지 |
| v3 helper 단위 테스트 6 (시각 layout) | PASS 유지 |
| v5 신규 regression `v5_tac_toggle_invalidates_page_tree_and_emits_inline_picture_below_table` | PASS |

## 5. 변경 파일 (v5 Stage 9)

- `src/document_core/commands/object_ops.rs:292` — fix (1 줄 추가)
- `src/document_core/commands/object_ops.rs` — v5 regression 테스트 1 개 추가

## 6. 사용자 시연 검증 절차 (WASM 빌드 후)

1. `docker compose --env-file .env.docker run --rm wasm` — WASM 재빌드
2. rhwp-studio dev server 재시작 + 브라우저 새로고침
3. 신규 빈 문서 → 표 1×1 → 셀 안 picture 삽입 (작은 / 큰 각각)
4. 그림 속성 → "글자처럼 취급" 체크 → 확인
5. 기대: picture 가 표 아래 inline 위치로 이동 + 표가 picture 만큼 안 밀려나도 picture 가 표 직하 위치 (한컴 정합)

## 7. Stage 10 진입 조건

- root cause 확정 + 단순 fix (1 줄) + regression 테스트 1 개 ✓
- 전수 회귀 0 ✓
- v4 의 click hit-test scope 완성 + v5 의 cache invalidate fix 완성 ✓

→ Stage 10 (통합 PR v1+v2+v3+v4+v5 + 최종 보고서) 진행.
