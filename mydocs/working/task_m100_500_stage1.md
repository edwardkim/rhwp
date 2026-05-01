# Task #500 단계 1 보고서 — Shape 분기 target_line/리셋 로직 적용

**이슈**: #500
**브랜치**: `local/task500`
**상태**: 단계 1 완료

## 1. 변경 내용

`src/renderer/layout/table_layout.rs` `Control::Shape` `treat_as_char` 분기에 Picture 분기와 동일한 `target_line` 산출 + `inline_x/tac_img_y` 리셋 로직 적용.

```rust
// [Task #500] Picture 분기와 정합
let target_line = if all_runs_empty && para.line_segs.len() > 1 {
    let li = tac_seq_index.min(para.line_segs.len() - 1);
    tac_seq_index += 1;
    li
} else {
    composed.tac_controls.iter()
        .find(|&&(_, _, ci)| ci == ctrl_idx)
        .map(|&(abs_pos, _, _)| {
            composed.lines.iter().enumerate().rev()
                .find(|(_, line)| abs_pos >= line.char_start)
                .map(|(li, _)| li).unwrap_or(0)
        }).unwrap_or(0)
};
if target_line > current_tac_line {
    current_tac_line = target_line;
    let line_w = tac_line_widths.get(target_line).copied().unwrap_or(0.0);
    inline_x = match para_alignment { ... };
    if let Some(seg) = para.line_segs.get(target_line) {
        tac_img_y = para_y_before_compose + hwpunit_to_px(seg.vertical_pos, self.dpi);
    }
}
```

shape_area.y / layout_cell_shape 의 para_y 인자를 `para_y_before_compose` 에서 `tac_img_y` 로 변경.

## 2. 검증

### 2-1. 핵심 회귀 케이스 (exam_science p2 7번 박스)

dump 분석:
- `p[1] ps_id=71 ctrls=2 text_len=84 ls[0] vpos=1610 lh=1150 ls=460, ls[1] vpos=3220 lh=1716 ls=460`
- `ctrl[1] 사각형: tac=true, wrap=TopAndBottom`
- ls[1].lh=1716 = shape height → 사각형이 ls[1] 라인을 차지

| 측정 | Before | After |
|------|--------|-------|
| 사각형 y | 206.74 (= para_y_before_compose) | **249.68** (= para_y_before_compose + ls[1].vpos) |
| ㉠ baseline y | 222.78 | 265.72 |
| 사각형 x | 104.07 | **97.07** (ls[1] 시작) |

PDF 시각 비교 (`samples/pdf/hwp2022/exam_science.pdf` p2 7번 박스):
- "분자당 구성 원자 수가 3인 분자의 분자 모양은 모두" — ls[0] (첫 줄)
- "[㉠] 이다." — ls[1] (둘째 줄, 사각형 위치)

→ **PDF 와 정합**.

### 2-2. 단위 + 통합 테스트

- `cargo test --release --lib`: **1103 passed; 0 failed; 1 ignored**
- `cargo test --release --tests`: 모든 통합 테스트 통과

### 2-3. 광범위 회귀 (7 샘플 byte diff)

| 샘플 | total | same | diff |
|------|-------|------|------|
| exam_kor | 20 | 20 | 0 |
| exam_eng | 8 | 8 | 0 |
| **exam_science** | **4** | **3** | **1 (의도된 정정)** |
| exam_math | 20 | 20 | 0 |
| synam-001 | 35 | 35 | 0 |
| aift | 77 | 77 | 0 |
| 2010-01-06 | 6 | 6 | 0 |

**170 페이지 중 1 페이지 정정 (의도), 169 페이지 byte 동일** — 회귀 0건.

exam_science p2 diff = 사각형 + ㉠ 텍스트 좌표만 변경 (의도된 정정).

## 3. 결론

수정 적용 후:
- 핵심 회귀 케이스 (exam_science p2 7번 박스 ㉠ 사각형) 정상 위치 (ls[1])
- PDF 시각 일치
- 단위 + 통합 + 7 샘플 광범위 회귀 0건

단계 1 완료. 최종 보고서로 진행.
