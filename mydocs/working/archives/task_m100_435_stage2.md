# Task #435 Stage 2: #393 (1) col 1 reserve 정정 — 완료보고서

> **이슈**: [#435](https://github.com/edwardkim/scope/rhwp/issues/435), [#393](https://github.com/edwardkim/rhwp/issues/393)
> **브랜치**: `local/task435`
> **단계**: 2/5 (#393 옵션 A col 1 reserve 정정)
> **완료일**: 2026-04-29

---

## 통과 조건 체크

- [x] exam_kor.hwp **24 → 22 페이지** (page 2, 15 orphan 사라짐)
- [x] page 1 단 1 used: 1207.6 → 1035.8px (over-fill 해소)
- [x] page 14 (구) 단 1 → page 13 (신) 단 1 used: 1193.5 → 1076.9px (over-fill 해소)
- [x] 회귀 대상 4 문서 페이지 수 베이스라인 일치
- [x] `cargo test` 전체 통과 (1062 + 추가 테스트 모두 ok)

---

## 코드 변경

### `src/renderer/typeset.rs:2127-2174` `compute_body_wide_top_reserve_for_para`

**변경 전:**

```rust
// Paper 기준 도형: 본문과 겹치지 않을 때(=머리말 영역만 점유)만 제외.
if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = ...vertical_offset...;
    let shape_bottom_abs = shape_top_abs + ...height...;
    if shape_bottom_abs <= body_top { continue; }
}
let shape_w = ...;
if shape_w < body_w * 0.8 { continue; }
let shape_h = ...;
let shape_y_offset = ...vertical_offset...;
if shape_y_offset > body_h / 3.0 { continue; }
let outer_bottom = ...margin.bottom...;
let bottom = shape_y_offset + shape_h + outer_bottom;  // ❌ Paper-rel 좌표를 body-rel 처럼 누적
```

**변경 후:**

```rust
let shape_w = ...;
if shape_w < body_w * 0.8 { continue; }
let shape_h = ...;
let raw_v_offset = ...vertical_offset...;

// body-rel 기준 시작/끝 y 계산.
// - VertRelTo::Paper: vertical_offset 이 용지 상단(= 0) 기준 → body_top 차감.
//   본문과 전혀 겹치지 않으면(머리말만 점유) 제외.
//   본문 위쪽으로 일부 빠져나가면(shape_top_abs < body_top) 본문 침범 영역만 reserve.
// - VertRelTo::Page / Para: vertical_offset 이 본문/단 top 기준 → body-rel 그대로.
let (body_y, body_bottom) = if matches!(common.vert_rel_to, VertRelTo::Paper) {
    let shape_top_abs = raw_v_offset;
    let shape_bottom_abs = shape_top_abs + shape_h;
    if shape_bottom_abs <= body_top { continue; }
    ((shape_top_abs - body_top).max(0.0), shape_bottom_abs - body_top)
} else {
    (raw_v_offset, raw_v_offset + shape_h)
};

if body_y > body_h / 3.0 { continue; }
let outer_bottom = ...margin.bottom...;
let bottom = body_bottom + outer_bottom;  // ✅ body-rel
```

### 변경 요지

1. **Paper-rel `vertical_offset` 의 body-rel 변환** — `body_top` 차감하여 본문 침범 영역만 reserve
2. body 와 전혀 겹치지 않는 (header 영역만) 케이스 skip 로직 유지
3. body 일부 침범 (`shape_top_abs < body_top < shape_bottom_abs`) 케이스: 침범 영역만 reserve, 위쪽 빠진 부분 제외
4. VertRelTo::Page / Para 분기는 `vertical_offset` 이 이미 body-rel/col-rel 이므로 기존 동작 유지

### 영향 범위

- 본 함수는 `typeset.rs:613` 1 곳에서만 호출 — 다단 첫 paragraph 진입 시 col 1+ reserve 산정
- `layout.rs:1178` 의 `calculate_body_wide_shape_reserved` 는 paper-rel `bottom_y` 를 그대로 사용하지만, layout 측도 paper-rel y_offset 으로 일관 사용하므로 수정 불필요 (실증으로 layout 동작 변경 없음을 확인)

---

## 결과 비교

### exam_kor 페이지 수: 24 → 22

| 메트릭 | Stage 1 | Stage 2 | Δ |
|---|---|---|---|
| 총 페이지 | 24 | **22** | -2 |
| Orphan 페이지 | 2개 (page 2: 49px, page 15: 104px) | **없음** | -2 |
| pi=0.30 split | PartialParagraph (lines 0..1, 1..3) | **FullParagraph** (vpos=84992..89009) | 해소 |
| pi=1.25 split | PartialParagraph (lines 0..1, 1..2) | **FullParagraph** (vpos=80402..82240) | 해소 |

### col 1 reserve 산정 변화

| 위치 | Stage 1 cur_h | Stage 2 cur_h | Δ |
|---|---|---|---|
| section 0 page 1 단 1 (pi=12 진입) | 306.1 | **94.4** | -211.7 |
| section 1 page 14 단 1 (pi=12 진입) | 306.1 | **94.4** | -211.7 |

(94.4 ≈ 94.5 = HWP 실제 col 1 시작 vpos=7085 HU px 환산값)

### 회귀 검증

```
exam_kor.hwp:    24페이지 → 22페이지   (목표 진행 중)
exam_eng.hwp:     8페이지 → 8페이지    ✓ 유지
k-water-rfp.hwp: 28페이지 → 28페이지   ✓ 유지
hwpspec.hwp:    177페이지 → 177페이지  ✓ 유지
synam-001.hwp:   35페이지 → 35페이지   ✓ 유지
```

`cargo test --release`: **1062 passed, 0 failed** (전체 테스트 + integration tests 통과)

---

## 잔여 이슈

Stage 1 의 (2) (3) 항목은 Stage 3, 4 에서 처리:

### (2) 일반 페이지 누적 -100~-300px (Stage 3 대상)

페이지 3, 4, 5, 7, 8, 10, 11, 12, 13, 17, 19, 20, 21 등에서 \|diff\| > 100px. 표/도형 (Shape) 배치 후 컬럼 잔여 공간 산정 로직 정정 필요. exam_kor 22 → 20 페이지 도달 목표.

### (3) Square wrap 표 + page 14 (구 page 16) 컬럼 불균형 (Stage 4 조건부)

| 단 | used | 비고 |
|---|---|---|
| 단 0 | 1225.8 | 본문 1211.3 초과 (overflow 14px) |
| 단 1 | 64.3 (items=2) | hwp_used 1189.0 / diff -1124.7 |

Square wrap 표 4 개 (`pi=33, 37, 40, 47`) over-fill 보호 로직 검토. Stage 3 에서 자연 해소되지 않으면 Stage 4 진입.

---

## 다음 단계

**Stage 3 — (2) 표/도형 후 컬럼 잔여 공간 정정** 진입.

수정 영역 후보:
- `pagination/engine.rs` 의 `process_controls` (`engine.rs:933`) — Shape/Table 배치 후 height 누적
- `pagination/engine.rs` 의 `paginate_table_control` (`engine.rs:1055`)
- `pagination/engine.rs` 의 `place_table_fits` (`engine.rs:1310`)

목표: exam_kor 22 → 20 페이지, 일반 페이지 \|diff\| < 100px.
