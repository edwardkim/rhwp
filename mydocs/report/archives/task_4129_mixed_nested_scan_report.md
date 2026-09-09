# 수정 보고서 — Task #4129 mixed_nested_flow_extra_from_cut O(P×U) 재스캔 제거

## 이슈

- Issue: https://github.com/edwardkim/rhwp/issues/4129
- PR: https://github.com/edwardkim/rhwp/pull/4131 (stacked on #4127)
- 선행: #1949 (렌더 트리 쪽 O(pages×cell) — cell_units 메모이즈로 해소, 같은 계열의
  별개 병목), #4126/#4128 (커서 질의 쪽 O(pages) — 본 트랙과 근본 원인 다름)
- 계측 출처: `RHWP_2424_PROFILE` 하네스 (`paginate_pass`, #2424 계보)

## 결론 (요약)

분할 표 컷 높이 평가(`row_cut_content_height`)가 fragment 마다 부르는
`mixed_nested_flow_extra_from_cut` 의 **per-para × 전체 유닛 이중 루프(O(P×U))** 를
mixed 유닛의 para_idx 단조성을 이용한 **1-pass run-walk(O(U))** 로 재작성했다.
115-fragment 거대 셀 문서의 paginate() **16,923ms → 431ms**(native debug, 39×),
studio(wasm) 문서 open 의 main-thread 블록 **~17s → 최장 207ms**. 페이지 수 115 동일,
corpus 355개 문서 전수 비트 동일.

## 병목 함수 (정확한 이름·경로)

| 역할 | 함수 | 위치 |
|---|---|---|
| paginate 진입 | `DocumentCore::paginate` → `paginate_pass` | `src/document_core/queries/rendering.rs` |
| 구역 조판 (99% 소비 지점) | `TypesetEngine::typeset_section_with_variant` | `src/renderer/typeset.rs` |
| 표 문단 경로 | `TypesetEngine::typeset_table_paragraph` → `typeset_block_table` → `typeset_block_table_inner` | `src/renderer/typeset.rs` |
| fragment 루프 (115회) | `TypesetEngine::step_block_table_continuation` | `src/renderer/typeset.rs` |
| fragment 당 행 스캔 | `TypesetEngine::scan_block_table_split_rows` | `src/renderer/typeset.rs` |
| 컷 높이 평가 (호출부) | `LayoutEngine::row_cut_content_height` | `src/renderer/layout/table_layout.rs` |
| **병목 본체 · 수정 지점** | `LayoutEngine::mixed_nested_flow_extra_from_cut` | `src/renderer/layout/table_layout.rs` |
| 무죄 판정된 프리미티브 | `LayoutEngine::advance_row_cut` · `cell_units` (#1949 캐시 유효) | `src/renderer/layout/table_layout.rs` |

## 진단 경로 (3단계 계측 + 샘플링)

재현: `samples/issue1949_giant_cell_nested_tables_perf.hwp` (3×1 RowBreak 표,
cell[2]=2507문단, 115쪽). `RHWP_2424_PROFILE=1` + native CLI `dump` 로 wasm 재빌드
없이 반복 계측.

1. **typeset 하위 단계 버킷** (`RHWP_2424_TYPESET_PROFILE`, typeset.rs 신설):
   문서는 구역 1·문단 1 — 16.75s 전부가 `typeset_table_paragraph` 1회.
   `table=16754ms/1`, text/wrap/flush/tail ≈ 0.
2. **fragment 루프 버킷** (`RHWP_2424_STEP_PROFILE`, `step_block_table_continuation`):
   `scan_block_table_split_rows` = **16,941ms/115회** (~147ms/fragment).
   프리미티브는 무죄 — `advance_row_cut` 13.9ms/228회, `cell_units` 캐시 정상
   (799 hit / 3 miss / 166ms — #1949 수정 유효).
3. **macOS `sample` 8초 (6,557 samples)**: 6,551개가 scan 의 두 호출지점 →
   `row_cut_content_height` → **`mixed_nested_flow_extra_from_cut` ≈75%**,
   시간은 slice `Iter::next` (전체 유닛 재스캔)에 집중.

원인 코드 (수정 전):

```rust
for para_idx in 0..cell.paragraphs.len() {      // O(P)
    for (idx, unit) in units.iter().enumerate() // O(U) — 매번 전체 재스캔
```

호출당 O(P×U), fragment 당 2~3회 호출 × 115 fragments. 실측 유닛 방문 총량
**2,501,304,096회** (아래 red 검증).

## 수정

- **run-walk 재작성**: mixed 유닛은 `cell_units_uncached` 의 단일 문단 루프
  (ascending `pi`)에서만 생성되어 para_idx 가 유닛 순서상 단조 비감소 — 문단별
  mixed run 이 연속 구간이므로 units 1-pass 로 run 단위 판정. per-para 지역변수와
  판정 수식은 종전 그대로 이동 (부동소수 합산 순서 동일 → 비트 동일).
- 범위 밖 para_idx 유닛은 종전 outer 루프(0..P)가 방문하지 않던 것과 동일하게 무시.
- 단조성은 `debug_assert` 로 상시 보증.
- 검증 기간 동안 종전 구현을 `_reference` 로 유지하고 `RHWP_2424_SHADOW=1` 호출 단위
  비트 대조 게이트를 운용 — corpus 검증 완료 후 같은 스택의 후속 레이어에서 제거.

## 검증

- **A/B 비트 대조**: `RHWP_2424_SHADOW=1` 로 corpus 355개(.hwp/.hwpx 전수) 스윕 —
  **0 divergence** (모든 호출에서 `f64::to_bits` 동일).
- **페이지네이션 불변**: 샘플 115쪽 동일, `RHWP_2424_STEP_PROFILE` iters=115 동일.
- **red→green 회귀 테스트**: `tests/issue_4129_mixed_nested_scan_budget.rs` —
  함수 내부 루프의 실제 유닛 방문 수를 카운터로 누적, 문서 열기 1회 총량 상한 20M.
  카운터를 유지한 채 per-para 본문으로 되돌린 실측 **2,501,304,096회 → FAIL**,
  run-walk **수십만 수준 → PASS**. 미배선 공회전 방지 하한(>10k) 포함.
- **게이트**: nextest 5,277/5,277 (46 slow, 28 skipped), fmt, clippy
  (root + `--workspace --all-targets`), `cargo check --target wasm32-unknown-unknown`.
- **실측**:

| 지표 | 수정 전 | 수정 후 |
|---|---|---|
| paginate() 전체 (native debug) | 16,923ms | 431ms (게이트 제거 후 358ms) |
| typeset 단계 | 16,756ms | 234ms |
| fragment scan 누적 | 16,941ms | 13.7ms |
| studio(wasm) 최장 main-thread 블록 | ~17s | 207ms |
| mixed_nested 유닛 방문 총량 | 2.50B | < 20M 상한 내 |

## 남긴 것

- `RHWP_2424_PROFILE` 하위 단계 계측(typeset.rs 버킷·table_layout.rs 프리미티브
  카운터)은 off 시 동작 불변으로 유지 — 기존 `paginate_pass` 하네스와 같은 패턴.
- 상대깊이 2+ 중첩 표는 `calc_nested_table_height` 럼프(기존 설계 경계)로, 본 수정과
  무관하게 페이지 중간 분할 불가 — 별도 개선 여지로만 기록해 둔다.
