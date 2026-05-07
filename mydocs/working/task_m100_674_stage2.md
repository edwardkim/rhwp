# Task #674 Stage 2 단계별 보고서 — 본질 정정

## 1. 정정 위치

`src/renderer/layout/table_layout.rs:746-781` — `calc_para_lines_height` 함수.

### 시그니처 변경

```rust
fn calc_para_lines_height(
    &self,
    lines: &[crate::renderer::composer::ComposedLine],
    pidx: usize,
    total_para_count: usize,
    para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    styles: &ResolvedStyleSet,  // [Task #674] 신규
) -> f64
```

### 본질 정정

```rust
let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
let cell_ls_type = para_style.map(|s| s.line_spacing_type)
    .unwrap_or(crate::model::style::LineSpacingType::Percent);
let lines_total: f64 = lines.iter()
    .enumerate()
    .map(|(i, line)| {
        let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
        let max_fs = line.runs.iter()
            .map(|r| styles.char_styles.get(r.char_style_id as usize)
                .map(|cs| cs.font_size).unwrap_or(0.0))
            .fold(0.0f64, f64::max);
        let h = crate::renderer::corrected_line_height(
            raw_lh, max_fs, cell_ls_type, cell_ls_val);
        let is_cell_last_line = is_last_para && i + 1 == line_count;
        if !is_cell_last_line {
            h + hwpunit_to_px(line.line_spacing, self.dpi)
        } else {
            h
        }
    })
    .sum();
```

`height_measurer.rs:570-587` 와 동일 로직 — 측정/layout 일관성 정합.

## 2. 호출자 시그니처 정정

| 파일:줄 | 함수 | 변경 |
|---------|------|------|
| `table_layout.rs:710` | calc_cell_paragraphs_content_height → calc_para_lines_height | styles 추가 |
| `table_layout.rs:728` | calc_composed_paras_content_height → calc_para_lines_height | styles 추가 |

## 3. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 4. 본 정정 효과

### total_content_height 정합

| 영역 | BEFORE | AFTER |
|------|--------|-------|
| `calc_para_lines_height` line_height 보정 | 없음 (raw 5.33) | **corrected (21.33)** ✅ |
| total_content_height (3줄) | 16.00 | **64.00** ✅ |
| mechanical_offset (Center) | (64-16)/2 = 24.00 | **(64-64)/2 = 0.00** ✅ |
| text_y_start | 379.37 (24px 위로 밀림) | **355.37 (정확)** ✅ |
| 마지막 줄 y (줄 2) | 422.04 (clip 끝 초과) | **397.71 (clip 안)** ✅ |

### 시각 판정 (PNG)

`samples/계획서.hwp` 1페이지:

| 셀 | BEFORE | AFTER |
|----|--------|-------|
| [21] "목적" 행 | 2줄 표시 (마지막 줄 클립) | **3줄 모두 표시** ✅ |
| [52] "특허 취득" 행 | 2개 paragraph (◦특허의뢰 누락) | **3개 paragraph 모두 표시** ✅ |
| 다른 셀 | — | 회귀 0 ✅ |

## 5. 회귀 위험 영역 좁힘

`corrected_line_height` 적용 영역:

- 정상 인코딩 paragraph: `line.line_height` (corrected 값과 비슷) → 보정 후 동일 또는 약간 큼 → 영향 미미
- line_segs 부재 paragraph: 큰 보정 (5.33 → 21.33) → 본 case 정정

광범위 sweep 차이 0 입증 (Stage 3).

## 6. Stage 3 진행 영역

- 광범위 페이지네이션 회귀 sweep (187 fixture)
- 결정적 검증 (cargo test, clippy)
- 시각 판정 게이트웨이 (작업지시자)
- 최종 보고서 작성

## 7. Stage 3 진행 승인 요청

본 Stage 2 결과 + 결정적 검증 + 시각 판정 통과 영역 입증 후 Stage 3 진행.
