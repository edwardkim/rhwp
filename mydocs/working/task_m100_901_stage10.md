# Task #901 Stage 10 보고서 — vpos_lazy_base flow-around 정합

**Stage**: 10
**상태**: page 2 paragraph 22 한컴 정합 ✅ (1402 test 회귀 없음)

## 1. 진단

Stage 8 의 flow-around fix 로 paragraph 19 가 page 2 top 으로 이동했으나, 후속 paragraph (20, 21, 22) 의 vpos correction 이 이전 lazy_base 계산식 사용 → paragraph 22 가 y=959 (page 2 boundary 직전) 에 위치.

VPOS_CORR trace (Stage 9):
```
pi=20: base=62369 vpos_end=120391 end_y=905.89
pi=21: base=62369 vpos_end=121991 end_y=927.23
pi=22: base=62369 vpos_end=123591 end_y=948.56
```

base=62369 = `prev_vpos_end - y_delta_hu` (lazy 자연 계산) → file 의 iris vpos jump (~31071 HU) 미반영.

## 2. ROOT CAUSE

iris Shape item 의 처리가 paragraph 19 와 20 사이 `vpos_lazy_base = None` 으로 reset (`layout.rs:2027`). flow-around 의 post_jump 가 paragraph 20 에 도달하기 전에 무효화.

## 3. Fix

`src/renderer/layout.rs`:

1. `pending_topbottom_post_jump` 의 type: `Option<f64>` → `Option<(f64, i32)>` (bottom_y + anchor_first_vpos)
2. post-jump 적용 시 **Shape/Table item 은 skip** (vpos_lazy_base reset 회피)
3. paragraph item 도달 시 `vpos_lazy_base = Some(anchor_first_vpos)` 으로 직접 설정 → 후속 vpos correction 이 anchor 첫 vpos 기준 사용

```rust
let item_is_paragraph = matches!(item,
    PageItem::FullParagraph { .. } | PageItem::PartialParagraph { .. });
if item_is_paragraph {
    if let Some((bottom_y, anchor_first_vpos)) = pending_topbottom_post_jump.take() {
        if bottom_y > y_offset { y_offset = bottom_y; }
        vpos_lazy_base = Some(anchor_first_vpos);
        vpos_page_base = None;
    }
}
```

## 4. 결과

| paragraph | y (Stage 8) | y (Stage 10) | 개선 |
|-----------|-------------|--------------|------|
| 19 잔 (page 2 top) | 143 | 143 | 유지 |
| 22 집 | 959 | **678** | **-281 px** |
| paragraph 22 overflow | 3.5 px | **0 px** | 해소 |

VPOS_CORR trace (Stage 10):
```
pi=20: base=83520 end_y=623.88 ✓
pi=21: base=83520 end_y=645.21 ✓
pi=22: base=83520 end_y=666.55 ✓
```

base=83520 = paragraph 19 첫 line vpos. file 의 vpos 누적이 visual y 와 정확히 매핑.

## 5. 회귀 검증

- ✅ `cargo test --release --all-targets`: **1402 passed, 0 failed**
- ✅ page 2 paragraph 19 top + iris middle + paragraph 22 below iris (한컴 정합)
- ✅ pic2.hwp page 1 layout 유지 (Stage 1~7 통합 결과)

## 6. 종합 (Stage 1+2+3+5+6+7+8+10)

pic2.hwp 정합 progress:

| 항목 | Baseline | 최종 (Stage 10) |
|------|----------|------------------|
| paragraph 0 우 위치 | 좌측 | 우측 세로 ✅ |
| paragraph 0 line gap | 119 px | 60 px (한컴 정합) ✅ |
| paragraph 7 SK하이닉스 y | 788 | 571 |
| paragraph 11 올해 확정 | page 2 | page 1 ✅ |
| paragraph 19 page 2 위치 | iris 아래 | iris 위 ✅ |
| paragraph 22 page 2 위치 | overflow | iris 아래 fit ✅ |
| 1402 test | passed | passed ✅ |
