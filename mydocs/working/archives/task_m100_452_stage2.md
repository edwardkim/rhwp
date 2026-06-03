# Task #452 Stage 2: 코드 수정 + 단위 검증 보고서

## 1. 코드 수정

`src/renderer/layout/paragraph_layout.rs:2511-2520` (변경 후 동일 위치):

```rust
// Before
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
let is_para_last_line = cell_ctx.is_none()
    && line_idx + 1 == end
    && end == composed.lines.len();
if (is_cell_last_line && cell_ctx.is_some()) || is_para_last_line {
    y += line_height;
} else {
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}

// After
let is_cell_last_line = is_last_cell_para && line_idx + 1 >= end;
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else {
    let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
    y += line_height + line_spacing_px;
}
```

`is_para_last_line` 변수 제거. 본문 단락 마지막 줄도 trailing line_spacing 가산.

## 2. 단위 검증 (exam_kor 1페이지)

| 측정 | Before (Stage 1) | After (Stage 2) | 기대 |
|------|------------------|------------------|------|
| pi=1.line0 baseline | 358.89 px | 365.03 px | (절대 위치는 상위 단락 trailing ls 가산으로 시프트, 정상) |
| pi=1.line9 baseline (찾) | 579.45 px | 585.59 px | — |
| pi=2.line0 baseline (통) | 594.79 px | 610.09 px | — |
| **pi=1.line9 ↔ pi=2.line0 step** | **15.34 px (버그)** | **24.50 px** | **24.51 px ✓** |
| 단락내 step | 24.51 px | 24.50 px | 24.51 px ✓ |

**핵심 결과**: pi=1↔pi=2 경계 step 이 단락내 step 과 동일하게 24.50 px 로 정합. 버그 해소.

## 3. cargo test 결과

- **lib**: 1066 passed (회귀 0)
- **svg_snapshot**: 2건 baseline 갱신 (`UPDATE_GOLDEN=1` 으로 갱신, 갱신 후 6/6 passed)
  - `tests/golden_svg/issue-147/aift-page3.svg`
  - `tests/golden_svg/issue-157/page-1.svg`
  - 변경 내용: 모든 y 좌표가 trailing ls 누적분(약 +9.6px ~ 누적) 만큼 시프트 다운. 텍스트/도형 내용 동일, 위치만 일관 시프트.
  - 시각 검토: PNG 렌더링 결과 두 페이지 모두 콘텐츠 정상 표시, 잘림 없음.
- **기타 테스트**: 모두 통과

## 4. LAYOUT_OVERFLOW 메시지

snapshot 갱신 전 stderr 메시지:
```
LAYOUT_OVERFLOW_DRAW: section=0 pi=28 line=0 y=1094.6 col_bottom=1093.3 overflow=1.3px
LAYOUT_OVERFLOW: page=1, col=0, para=28, type=FullParagraph, y=1104.2, bottom=1093.3, overflow=10.9px
```

해석: pi=28(페이지 마지막 단락) 의 trailing line_spacing(~10.9 px ≈ 1 ls) 이 col_bottom 을 살짝 넘어 그려짐. 그러나 trailing ls 는 빈 공간이므로 실제 텍스트는 col_bottom 안에 위치. **시각적 무영향**. pagination engine 의 `effective_trailing` 처리는 fit 판정 시 trailing 을 제외하므로 페이지 분배는 유지됨.

## 5. 산출물

- `src/renderer/layout/paragraph_layout.rs` 수정 (1 파일, ~10줄)
- `tests/golden_svg/issue-147/aift-page3.svg` baseline 갱신
- `tests/golden_svg/issue-157/page-1.svg` baseline 갱신
- `/tmp/task_452_after/exam_kor_001.svg` 검증용 SVG

다음 단계: Stage 3 (광범위 회귀 검증).
