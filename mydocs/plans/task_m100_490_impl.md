# Task #490 구현계획서

## 1. 변경 위치

`src/renderer/layout/paragraph_layout.rs:2227~2232` (빈 runs + TAC 수식 분기)

## 2. 변경 내용

기존:
```rust
if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
    let line_start_char = comp_line.char_start;
    let line_end_char = composed.lines.get(line_idx + 1)
        .map(|l| l.char_start)
        .unwrap_or(usize::MAX);
    let mut inline_x = effective_col_x + effective_margin_left;
    ...
}
```

수정:
```rust
if comp_line.runs.is_empty() && !tac_offsets_px.is_empty() {
    let line_start_char = comp_line.char_start;
    let line_end_char = composed.lines.get(line_idx + 1)
        .map(|l| l.char_start)
        .unwrap_or(usize::MAX);
    // [Task #490] paragraph alignment 적용
    let line_tac_width: f64 = tac_offsets_px.iter()
        .filter(|(pos, _, _)| *pos >= line_start_char && *pos < line_end_char)
        .map(|(_, w, _)| *w)
        .sum();
    let align_offset = match alignment {
        Alignment::Center | Alignment::Distribute => {
            (available_width - line_tac_width).max(0.0) / 2.0
        }
        Alignment::Right => {
            (available_width - line_tac_width).max(0.0)
        }
        _ => 0.0,
    };
    let mut inline_x = effective_col_x + effective_margin_left + align_offset;
    ...
}
```

## 3. 단계 (3 단계)

| 단계 | 내용 |
|------|------|
| Stage 1 | 코드 수정 + 단위 테스트 1 추가 + 빌드/테스트 통과 |
| Stage 2 | 광범위 byte 비교 회귀 점검 (9 종 샘플 263 페이지) + 차이 페이지 시각 점검 |
| Stage 3 | 보고서 + orders 갱신 + merge + push + close |

## 4. 단위 테스트

`integration_tests.rs:test_490_empty_para_with_tac_equation_respects_alignment`:
- exam_science p1 SVG 에서 y∈[1040,1090] (3번 표 영역) 의 ">28<" 직전 group transform x 좌표 검증
- 수정 전 x≈358 → 수정 후 x≥380 (alignment 적용 확인)
