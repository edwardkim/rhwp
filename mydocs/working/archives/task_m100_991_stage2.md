# Task #991 Stage 2 — F2 후보 시도 + 좁힘 조건 결정

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 선행: [Stage 1 진단](task_m100_991_stage1.md)
- 브랜치: `local/task991-fix`

## 1. F2 wide (초기 시도) — 조건 없음

```rust
if existing_markers >= inline_ctrl_count {
    return None;
}
// 모든 부족 case 에 synth 적용
```

**결과**: cargo test 5 fail
- test_548_cell_inline_shape_first_line_indent_p8 (puko box 위치 shift)
- test_521_tac_table_outer_margin_bottom_p2
- test_cursor_rect_after_line_break
- test_cursor_rect_after_line_break_at_end
- test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx

→ exam_eng p8 의 puko box x=168.60 (기대 155.60, -13px 어긋남).

## 2. F2 narrow 시도 1 — `n_leading >= 2`

```rust
if n_leading < 2 {
    return None;
}
```

**결과**: 3 fail
- test_548 (puko box x=162.60, -7px)
- test_cursor_rect_after_line_break*

→ 일부 sample 에 여전히 광범위 적용.

## 3. F2 narrow 시도 2 — `n_leading >= 2 AND inline_ctrl_count >= 3` ✓

```rust
if n_leading < 2 || inline_ctrl_count < 3 {
    return None;
}
```

**결과**: **0 fail, cargo test 전체 통과**

조건 의미:
- `inline_ctrl_count >= 3`: pi=394 (3 TAC) 패턴 — 일반적인 1-2 TAC paragraph 미해당
- `n_leading >= 2`: leading char_offsets gap 2+ extended ctrl — 대부분 paragraph 는 0-1
- 두 조건 AND → sample16 pi=394 특유의 경우만 catch

## 4. 최종 구현

```rust
fn synthesize_marker_paragraph(para: &Paragraph) -> Option<Paragraph> {
    // 1. inline-visible ctrl count (Header/Footer/Footnote/Endnote/HiddenComment 제외)
    let inline_ctrl_count = ...;
    if inline_ctrl_count == 0 { return None; }
    
    // 2. 기존 marker 검사 (HWP3 차단)
    let existing_markers = para.text.chars().filter(|c| *c == '\u{FFFC}').count();
    if existing_markers >= inline_ctrl_count { return None; }
    
    // 3. 좁힘 조건
    let first_off = para.char_offsets.first().copied().unwrap_or(0) as usize;
    let n_leading = first_off / 8;
    if n_leading < 2 || inline_ctrl_count < 3 { return None; }
    
    // 4. char_offsets gap 분석 + 마커 push
    // ...
}
```

## 5. 검증 (Stage 2 종결)

- cargo test --release --lib: **1297 passed, 0 failed**
- 정량적 좁힘 조건 검증 — 의도된 case 만 catch
