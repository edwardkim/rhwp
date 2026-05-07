# Task #674 Stage 1 단계별 보고서 — 본질 진단

## 진단 결과 요약

`samples/계획서.hwp` 셀 [21] paragraph layout 시작 위치 (`text_y_start = 379.37`) 가 이론값 (`cell_y + pad_top = 355.37`) 보다 **24.00 px** 큼. 본질 위치 식별 완료.

## 1. 디버그 출력 결과

`src/renderer/layout/table_layout.rs:1374-1388` 환경변수 디버그:

```
[T674] cell[5,1] cell_y=353.49 pad_top=1.88 pad_bottom=1.88
[T674]   has_nested=false first_line_vpos=None effective_valign=Center
[T674]   inner_height=64.00 total_content_height=16.00
[T674]   text_y_start=379.37 (cell_y+pad_top=355.37, diff=+24.00)
```

## 2. 본질 식별

| 항목 | 값 | 분석 |
|------|------|------|
| has_nested_table | false | 중첩 표 없음 (정상) |
| first_line_vpos | None | line_segs 부재 (정상) |
| effective_valign | **Center** | 셀 vertical_align = Center |
| inner_height | 64.00 | (정확) cell_h - pad |
| total_content_height | **16.00** | **잘못된 값 (3줄인데 16!)** |
| mechanical_offset | (64-16)/2 = **24.00** | 잘못된 큰 offset |
| text_y_start | 355.37 + 24 = **379.37** | mechanical_offset 영향 |

## 3. 본질 결함

### 3.1 `total_content_height = 16.00` 의 본질

`calc_composed_paras_content_height` → `calc_para_lines_height` 호출:

```rust
fn calc_para_lines_height(...) -> f64 {
    ...
    let lines_total: f64 = lines.iter()
        .map(|(i, line)| {
            let h = hwpunit_to_px(line.line_height, self.dpi);  // raw line_height만 사용!
            ...
        })
        .sum();
    ...
}
```

**`line.line_height` 그대로 사용** — `corrected_line_height` 보정 누락. line_segs 부재 paragraph 의 fallback `line_height = 400 HU = 5.33 px` 그대로 사용.

→ 3줄 × 5.33 = **16 px** 잘못된 측정.

### 3.2 paragraph_layout vs measurement 불일치

| 영역 | line_height | 3줄 합 |
|------|-------------|---------|
| `calc_para_lines_height` (측정) | 5.33 (raw) | **16.00** ❌ |
| `paragraph_layout::layout_composed_paragraph` (실제) | 21.33 (corrected) | **64.00** ✅ |
| `height_measurer` (row_heights 결정) | 21.33 (corrected) | **64.00** ✅ |

→ table_layout 의 `calc_para_lines_height` 만 corrected_line_height 보정 누락. height_measurer 와 paragraph_layout 은 정상.

### 3.3 결함 발현 흐름

```
table_layout.rs::layout_table:
  composed_paras = compose_paragraph(p)
  recompose_for_cell_width(comp, p, inner_width, styles)
  ↓
  total_content_height = calc_composed_paras_content_height(...)
                       → calc_para_lines_height (corrected_line_height 누락)
                       → 16.00 (잘못된 측정)
  ↓
  effective_valign = Center
  mechanical_offset = (64.00 - 16.00) / 2 = 24.00 (잘못된 큰 offset)
  ↓
  text_y_start = cell_y + pad_top + 24.00 = 379.37 (24px 위로 밀림)
  ↓
  paragraph_layout 줄 위치:
    줄 0 y = 379.37
    줄 1 y = 400.71
    줄 2 y = 422.04 (cell-clip 끝 421.25 초과 → 클립)
```

## 4. 다른 호출 위치 영향 평가

`calc_para_lines_height` 호출자:

| 파일:줄 | 함수 | 용도 |
|---------|------|------|
| `table_layout.rs:710` | calc_cell_paragraphs_content_height | 셀 높이 측정 (resolve_row_heights) |
| `table_layout.rs:728` | calc_composed_paras_content_height | total_content_height 계산 (text_y_start) |

두 함수 모두 영향. 정정 시 모두 corrected_line_height 적용 필요.

## 5. Stage 2 정정 방향

### 5.1 정정 위치

`src/renderer/layout/table_layout.rs:746-781` `calc_para_lines_height`:

- 시그니처에 `styles: &ResolvedStyleSet` 추가
- `corrected_line_height` 보정 적용 (height_measurer.rs:570-587 와 동일 로직)
- `cell_ls_val` / `cell_ls_type` ParaShape 기반 결정
- max_fs (line.runs 의 max font_size) 측정

### 5.2 회귀 위험 영역 좁힘

- **본질 정정** — 모든 호출 위치에 보정 적용 (height_measurer 와 일관성)
- 정상 line_segs 인코딩 paragraph: line.line_height 가 corrected 값과 비슷 → 영향 미미
- 본 case (line_segs 부재) 만 큰 영향 (5.33 → 21.33 보정)
- 광범위 sweep 차이 0 보장

## 6. Stage 1 산출물

본 단계별 보고서 — `text_y_start = 379.37` 의 24.00 px 오프셋 본질 (corrected_line_height 누락) 식별.

## 7. Stage 2 진행 승인 요청

본 진단 결과 + Stage 2 정정 방향 (calc_para_lines_height 시그니처 변경 + corrected_line_height 적용) 승인 요청.
