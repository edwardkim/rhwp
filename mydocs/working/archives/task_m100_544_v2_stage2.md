# Task #544 v2 Stage 2 보고서

## 목적

Phase A 재적용 — `paragraph_layout.rs` 의 두 영역 정정으로 작업지시자 보고 증상 (글상자 우측 시프트) + 박스 안 본문 텍스트 좌측 inset 이중 적용 동시 해소.

## 적용 변경

### 1. inner_pad 분기 제거 (Task #547, paragraph_layout.rs:693~717 → 693~700)

**Before** (28 LOC):
```rust
let para_border_fill_id_pre = para_style.map(|s| s.border_fill_id).unwrap_or(0);
let has_visible_stroke = if para_border_fill_id_pre > 0 {
    let idx = (para_border_fill_id_pre as usize).saturating_sub(1);
    styles.border_styles.get(idx)
        .map(|bs| bs.borders.iter().any(|b|
            !matches!(b.line_type, crate::model::style::BorderLineType::None) && b.width > 0))
        .unwrap_or(false)
} else {
    false
};
let bs_left_px = para_style.map(|s| s.border_spacing[0]).unwrap_or(0.0);
let bs_right_px = para_style.map(|s| s.border_spacing[1]).unwrap_or(0.0);
let (inner_pad_left, inner_pad_right) = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
    (box_margin_left, box_margin_right)
} else {
    (0.0, 0.0)
};
let margin_left = box_margin_left + inner_pad_left;
let margin_right = box_margin_right + inner_pad_right;
```

**After** (8 LOC):
```rust
// [Task #547] paragraph margin_left/right 는 텍스트 좌/우 inset 으로 한 번만
// 적용. Task #544 후 box outline = col_area (margin 미적용) 이므로 박스 안
// 좌측 여백 = box_margin_left (PDF 한컴 2010 정합).
// 이전 코드는 paragraph border + border_spacing=0 인 경우 inner_pad_left =
// box_margin_left 로 한 번 더 더해 이중 inset 부작용 발생 (Task #544 전 박스도
// margin 적용했을 때만 의미가 있던 분기).
let margin_left = box_margin_left;
let margin_right = box_margin_right;
```

→ 단일 룰: `margin_left = box_margin_left` (paragraph_layout.rs 의 텍스트 inset 산식과 일관).

### 2. box_x/w 산식 정정 (Task #544 (1), paragraph_layout.rs:2687-2691)

**Before**:
```rust
let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
    (ox + box_margin_left, ow - box_margin_left)
} else {
    (col_area.x + box_margin_left, col_area.width - box_margin_left - box_margin_right)
};
```

**After**:
```rust
let (box_x, box_w) = if let Some((ox, ow)) = self.border_box_override.get() {
    (ox, ow)
} else {
    (col_area.x, col_area.width)
};
```

→ paragraph border 박스 outline = col_area 전체 (PDF 한컴 2010 정합). margin 은 텍스트 inset 으로만 사용.

### 3. test_544 / test_547 의 `#[ignore]` 제거

RED → GREEN 전환.

## 검증

### 단위 테스트

```
cargo test --lib
test result: ok. 1121 passed; 0 failed; 3 ignored; 0 measured
```

- baseline (Stage 1) 1119 passed / 5 ignored
- 현재 1121 passed / 3 ignored
- Δ = **+2 GREEN, -2 ignored** (test_544 + test_547)
- 회귀 **0건**

### 핵심 회귀 테스트 무회귀 확인

| 테스트 | 결과 |
|--------|------|
| `test_544_passage_box_coords_match_pdf_p4` | **GREEN** (3 assertion 모두 통과) |
| `test_547_passage_text_inset_match_pdf_p4` | **GREEN** |
| `test_552_passage_box_top_gap_p2_4_6` (`--ignored`) | **GREEN** |
| `test_548_cell_inline_shape_first_line_indent_p8` | RED (Stage 3 대상, ignored 유지) |

Task #552 무회귀 = Stage 1 finding (Task #552 가 #544 (2) 효과 흡수) 확정 검증.

### test_544 측정값 변화

| 좌표 | Stage 1 (수정 전) | Stage 2 (수정 후) | PDF 기대 |
|------|--------------------|-------------------|----------|
| box_top_y | (이미 정합, ~233.8) | (정합 유지) | 233.8 |
| box_left_x | 128.51 | **117.0 ±2** | 117.0 |
| box_width | (미측정, fail by left_x) | **425.1 ±2** | 425.1 |

### test_547 측정값 변화

| 좌표 | Stage 1 (수정 전) | Stage 2 (수정 후) | PDF 기대 |
|------|--------------------|-------------------|----------|
| 박스 안 본문 min_x | 139.89 | **128.5 ±2** | 128.5 |

## 코드 영향

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/renderer/layout/paragraph_layout.rs` | -22 LOC (28→8) + box_x/w 4 LOC | 정정 본문 |
| `src/renderer/layout/integration_tests.rs` | -2 ignore attribute | RED → GREEN |

## 잔존

- Stage 3: Phase C (#548 셀 inline TAC Shape margin)
- Stage 4: 광범위 회귀 + 시각 판정 + 최종 보고서

## 다음 단계

Stage 3 — Phase C 재적용:
- `src/renderer/layout/table_layout.rs` 에 `effective_margin_left_line` 헬퍼 추가
- inline_x 산출 3 분기에 line_margin 가산 (Left/Justify 케이스)
- test_548 의 `#[ignore]` 제거
