# Task #534 Stage 3 + 4 — 코드 적용 + 회귀 검증

**작성일**: 2026-05-02
**이슈**: [#534](https://github.com/edwardkim/rhwp/issues/534)
**브랜치**: `local/task534`

## 1. 결론

> Stage 1 의 root cause (layout_shape_item TAC Picture x 좌표 inner_pad 누락) 정정. exam_kor p18 pi=50/56 image x: **593.39 → 604.72** (+11.33 px = inner_pad). 광범위 8 샘플 192 페이지 중 **190 byte-identical**, 변경 2 페이지 (exam_kor p18 + p19) 모두 동일 본질 의도된 정정. lib 1116 / svg_snapshot 6 / issue_418/501 통과.

## 2. Root Cause (Stage 1 추가 조사 통합)

임시 RHWP_TASK534_DBG 로깅으로 emit code path 식별:

| pi | emit path | effective_margin_left | image x |
|----|-----------|----------------------|---------|
| 46 (단독 그림) | **PARA_LAYOUT_TAC_PIC_EMPTY** (paragraph_layout.rs) | 72.00 (= ml 11.33 + inner_pad 11.33 + indent 49.33) | 654.05 ✓ |
| 50 (Square 표 + 그림) | **SHAPE_ITEM_TAC_PIC** (layout.rs::layout_shape_item) | 11.33 (= ml only) | 593.39 ✗ |
| 54 (단독 그림) | PARA_LAYOUT_TAC_PIC_EMPTY | 72.00 | 654.05 ✓ |
| 56 (Square 표 + 그림) | SHAPE_ITEM_TAC_PIC | 11.33 | 593.39 ✗ |

본질: **paragraph_layout.rs (line 711-716) 의 `inner_pad_left = box_margin_left` (visible stroke + border_spacing[0,1]=0 인 paragraph 의 inner padding) 이 layout_shape_item.rs 에 정합 미적용**.

wrap_host paragraph (Square wrap 표 보유) 는 layout_paragraph 미호출 (early return) → paragraph_layout 의 TAC Picture emit path 미작동 → layout_shape_item 만 emit → inner_pad 누락된 위치.

## 3. 변경

### 3-1. `src/renderer/layout.rs::layout_shape_item` (TAC Picture 분기)

```rust
// [Task #534] paragraph_layout 의 effective_margin_left 정합:
// visible stroke 보유 + border_spacing[0,1]=0 인 paragraph 는
// box_margin_left 를 inner padding 으로 추가 가산.
let para_border_fill_id_pre = para_style_ref.map(|s| s.border_fill_id).unwrap_or(0);
let has_visible_stroke = if para_border_fill_id_pre > 0 {
    let idx = (para_border_fill_id_pre as usize).saturating_sub(1);
    styles.border_styles.get(idx)
        .map(|bs| bs.borders.iter().any(|b|
            !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0))
        .unwrap_or(false)
} else { false };
let bs_left_px = para_style_ref.map(|s| s.border_spacing[0]).unwrap_or(0.0);
let bs_right_px = para_style_ref.map(|s| s.border_spacing[1]).unwrap_or(0.0);
let inner_pad_left = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
    para_margin_left
} else {
    0.0
};
let effective_margin_left = if para_indent > 0.0 {
    para_margin_left + para_indent + inner_pad_left
} else {
    para_margin_left + inner_pad_left
};
```

### 3-2. 변경량

| 영역 | 추가 | 삭제 | 수정 |
|------|------|------|------|
| `src/renderer/layout.rs` | 19 | 0 | 4 (effective_margin_left 분기) |

## 4. 시각 검증

### 4-1. exam_kor p18 우측 단

| pi | image x (수정 전) | image x (수정 후) | shift |
|----|------------------|------------------|-------|
| 46 (단독 그림) | 654.05 | 654.05 | 0 (paragraph_layout 우선 변동 없음) |
| **50 (Square 표 + 그림)** | **593.39** | **604.72** | **+11.33 px ✓** |
| 54 (단독 그림) | 654.05 | 654.05 | 0 |
| **56 (Square 표 + 그림)** | **593.39** | **604.72** | **+11.33 px ✓** |

### 4-2. exam_kor p19 (동일 본질 추가 발견)

```
Table   pi=60 ci=0  3x2  23.0x71.8px  wrap=Square tac=false  vpos=0
Shape   pi=60 ci=1  그림 tac=true  vpos=0
```

| 위치 | image x (수정 전) | image x (수정 후) | shift |
|------|------------------|------------------|-------|
| pi=60 그림 (col 0) | 128.51 | 139.84 | +11.33 px ✓ |

## 5. 광범위 회귀 검증

```bash
scripts/svg_regression_diff.sh build b669ab5 4abee04 \
    exam_kor exam_eng exam_science exam_math \
    synam-001 aift 2010-01-06 exam_math_no
```

| 샘플 | total | same | diff |
|------|-------|------|------|
| 2010-01-06 | 6 | 6 | 0 |
| aift | 77 | 77 | 0 |
| exam_eng | 8 | 8 | 0 |
| **exam_kor** | **20** | **18** | **2** (p18 + p19) |
| exam_math_no | 20 | 20 | 0 |
| exam_math | 20 | 20 | 0 |
| exam_science | 6 | 6 | 0 |
| synam-001 | 35 | 35 | 0 |
| **합계** | **192** | **190** | **2** |

→ 변경 2 페이지 모두 동일 본질 (Square wrap 표 + TAC 그림 동일 paragraph) 의도된 정정.

## 6. 단위/통합 테스트 게이트

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed** |
| `cargo test --test svg_snapshot` | **6/6** 통과 |
| `cargo test --test issue_418` | 1/1 통과 |
| `cargo test --test issue_501` | 1/1 통과 |

## 7. 회귀 차단 가드 검증

| 가드 | 보호 영역 | 측정 결과 |
|------|----------|----------|
| `has_visible_stroke` | 테두리 없는 paragraph | 회귀 0 |
| `border_spacing[0,1]==0` | 직접 border_spacing 보유 paragraph | 회귀 0 |
| `tac=true` | non-TAC Picture | 동일 paragraph_layout 경로, 영향 없음 |
| `paragraph_layout 우선` | 단독 TAC Picture (텍스트 없음) | already_registered=true 로 skip ✓ |

## 8. 다음 단계

작업지시자 시각 판정 (Stage 5) 후 close 흐름:
- `local/task534` → `local/devel` merge
- `local/devel` → `devel` merge + push
- issue #534 close

시각 판정 자료:
- `/tmp/p18_fix/exam_kor_018.svg` (수정 후 p18)
- exam_kor p19 (좌측 단 pi=60 그림)

## 9. 승인 게이트

- [x] Root cause 위치 확정 (layout_shape_item TAC Picture inner_pad 누락)
- [x] 코드 적용 (+19 라인 단일 가산)
- [x] 단위/통합 테스트 통과 (1116 / 6 / issue_418/501)
- [x] 광범위 회귀 (192 페이지 / 2 변경 / 모두 의도)
- [x] 회귀 차단 가드 측정 검증 (190 byte-identical)
