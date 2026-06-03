# Task #1035 Stage 2 완료 보고서 — Fix 적용

**Issue**: [#1035 HWP3 vs HWP5 변환본 페이지별 paragraph alignment 차이](https://github.com/edwardkim/regression-rhwp/issues/1035)
**Branch**: `local/task1035`
**작업 내용**: PR #1009 base + narrow 가드 적용 — sample16-hwp5 over-split 회피 + alignment 향상

---

## 1. 적용 변경

### 1.1 PR #1009 base (cherry-pick `71054c51`)

| 파일 | 변경 |
|------|------|
| `src/renderer/pagination/engine.rs` | +84 (variant_vpos_reset_break 도입) |
| `src/renderer/typeset.rs` | +136 (동일 로직 두 경로 정합) |
| `src/renderer/pagination.rs` | `PaginationOpts::is_hwp3_variant` 필드 |
| `src/document_core/queries/rendering.rs` | `is_hwp3_variant` 전달 |

variant 식별 인프라 (cfb_reader.rs, model/document.rs 등) 는 **#1005 이미 머지** — 재활용.

### 1.2 Task #1035 narrow 가드 (Stage 2 신규)

PR #1009 의 휴리스틱 2 항목 narrow:

```rust
// 변경 A: high_threshold 0.85 → 0.95 (보수적)
let high_threshold = body_height_hu_for_variant * 95 / 100;  // 종전 85

// 변경 B: aux_trigger 제거 (empty bridge 휴리스틱 false positive)
if main_trigger {                  // 종전 main_trigger || aux_trigger
    variant_vpos_reset_break = true;
}
```

엔진/typeset 두 경로 모두 동일 narrow.

---

## 2. 진단 → Fix 결정 근거

### 2.1 narrow 가드 후보 실험 (Stage 1 권고 D 변형)

| 시도 | sample16-hwp5 | alignment | 판정 |
|------|---------------|-----------|------|
| **PR #1009 그대로** (0.85 + aux) | 65 (+1) | 23/64 | over-split + 악화 |
| threshold 0.85 → 0.90 + aux | 65 | 23/64 | 변동 없음 |
| threshold 0.85 → 0.95 + aux | 65 | 23/64 | 변동 없음 |
| **all triggers disabled** (PR #1009 base 만) | 64 | 24/64 | devel 동일 (trigger 자체가 net -1) |
| **threshold 0.95 + aux 제거 (main만)** | **64** ✓ | **60/64** ✓ | **채택** |

→ **aux_trigger 자체가 over-split 의 직접 원인** (empty bridge 휴리스틱이 false positive). main_trigger 만 + threshold 0.95 로 narrow 시 정합률 대폭 향상 + over-split 회피.

### 2.2 main_trigger 발동 위치 단언 (RHWP_DEBUG_VARIANT_PAGEBREAK)

20+ 위치에서 main_trigger 발동. 대표:
```
pi=69  prev_end=72256 curr_first=852  main=true
pi=88  prev_end=71712 curr_first=568  main=true
pi=414 prev_end=72360 curr_first=1416 main=true (border case)
pi=473 prev_end=72472 curr_first=284  main=true
pi=440 prev_end=57252 curr_first=852  aux=true   ← over-split 원인 (제거됨)
```

---

## 3. Fix 결과

### 3.1 alignment 정합률

| 측정 | devel | PR #1009 그대로 | **Task #1035 fix** |
|------|-------|----------------|-------------------|
| sample16-hwp5 페이지 수 | 64 | 65 | **64** ✓ |
| alignment 정합 | 24/64 (37.5%) | 23/64 (35.9%) | **60/64 (93.75%)** ✓ |

→ 정합률 37.5% → **93.75%** (+56.25%).

### 3.2 회귀 sweep

| Sample | 페이지 |
|--------|-------|
| hwp3-sample-hwp5 | 16 |
| hwp3-sample4-hwp5 | 36 |
| hwp3-sample5-hwp5 (+v2018/v2024) | 64 |
| hwp3-sample10-hwp5 | 763 |
| hwp3-sample11-hwp5 | 151 |
| hwp3-sample13-hwp5 | 3 |
| hwp3-sample14-hwp5 | 11 |
| **hwp3-sample16-hwp5** | **64** (PR #1009 65 회귀 회피) |
| hwp3-sample19-hwp5 | 2 |
| hwp3-sample16-hwp5.hwpx | 71 |
| HWP3 native (sample16, etc) | 무변동 |
| exam_kor/eng/math, aift, biz_plan | 무변동 |

→ **모든 fixture 회귀 0**.

---

## 4. 자동 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ clean |
| **`cargo fmt --all -- --check`** | ✓ clean (feedback_cargo_fmt_all_required 정합) |
| `cargo test --release --lib` | ✓ 1308 passed (1 추가 — variant 인프라) |
| `cargo test --release --tests` | ✓ FAILED 0 (전체 integration) |
| `cargo test --release --test issue_1035_alignment` | ✓ 1 passed (회귀 가드) |

---

## 5. 단위 테스트 추가

`tests/issue_1035_alignment.rs::hwp3_sample16_hwp5_page_count_64`:
- sample16-hwp5 페이지 수 = 64 단언 (over-split 회귀 재발 방지)

---

## 6. 잔존 미정합 4 페이지

60/64 정합. 잔존 미정합 페이지 (4):

```
미정합 페이지 (HWP3 vs HWP5 변환본 첫 paragraph):
  p32  H3:pi=536    H5:pi=535    diff=-1
  p?   ...
```

대부분 ±1 paragraph 의 미세 차이. 작업지시자 시각 검증 시 한컴 정답 기준으로 추가 결정 가능 (별도 fix 또는 본 task 범위 외).

---

## 7. 성공 기준 충족

| 조건 | 기준 | 결과 |
|------|------|------|
| C1: alignment 정합률 향상 | 37.5% → ≥80% | ✓ **93.75%** (목표 초과) |
| C2: sample16-hwp5 64 유지 | over-split 회피 | ✓ |
| C3: 변환본 9 종 회귀 0 | 페이지 수 무변동 | ✓ |
| C4: 일반 HWP5/HWP3 회귀 0 | exam_*, aift, biz_plan | ✓ |
| C5: cargo test 1307+ passed | clean | ✓ (1308) |
| C6: clippy + fmt --all | clean | ✓ |
| C7: 작업지시자 시각 검증 | 한컴 정답 비교 | (Stage 4 시점) |

---

## 8. 다음 단계 (Stage 3)

(Stage 2 에서 회귀 sweep + alignment 측정 완료) → **Stage 3 생략 가능, Stage 4 (최종 보고 + PR) 로 직행** 권고. 또는 작업지시자 시각 검증 후 Stage 4 진행.
