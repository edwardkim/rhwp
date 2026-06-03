# Task #994 Stage 1 — 정밀 진단 결과

- 이슈: [#994](https://github.com/edwardkim/rhwp/issues/994)
- 브랜치: `local/task994`

## 1. 영향 paragraph 측정 (HWP5 sample16)

```
Total 文단: 529
empty line_segs + text 보유: 59 (#969 의 line_segs count diff 59 와 동일)
  Short (text_len ≤ 20): 10
  Medium (20 < text_len ≤ 50): 15
  Long (text_len > 50): 34  ← 시각 겹침 발생
```

Long paragraphs (text_len > 50) 의 예시:
- pi=380 (text_len=54)
- pi=407 (text_len=87)
- pi=410 (text_len=96)
- pi=437 (text_len=82)
- pi=438 (text_len=118)
- pi=442 (text_len=214)
- pi=443 (text_len=163)
- pi=444 (text_len=134)
- pi=445 (text_len=156)
- pi=446 (text_len=196)

## 2. Root cause 코드 위치

[src/renderer/layout/paragraph_layout.rs:334-398](src/renderer/layout/paragraph_layout.rs#L334-L398):

```rust
let line_height = if let Some(ls) = para.line_segs.first() {
    hwpunit_to_px(ls.line_height, self.dpi)
} else {
    hwpunit_to_px(400, self.dpi)   // ← 400 HU = 5.33 px (너무 작음)
};

let baseline_dist = if let Some(ls) = para.line_segs.first() {
    ensure_min_baseline(...)
} else {
    line_height * 0.8              // = 4.27 px
};

let line_step = if para.line_segs.len() > 1 {
    ...
} else if let Some(ls) = para.line_segs.first() {
    ...
} else {
    baseline_dist * 1.5            // = 6.4 px (너무 작음, 글자 높이 ~17 px)
};
```

→ wrap 시 `current_y += line_step (6.4 px)` — 글자 높이 미만 → wrap 라인 시각 겹침.

## 3. 수정 방향 (G1 확정)

`line_segs.is_empty()` case 에서 ParaShape 의 `line_spacing_type` + `line_spacing` + `para_max_font_size` 로 line_height 동적 계산.

기존 [typeset.rs:1205-1216](src/renderer/typeset.rs#L1205-L1216) (composed branch) 의 식 재사용:
```rust
match ls_type {
    LineSpacingType::Percent   => max_fs * ls_val / 100.0,
    LineSpacingType::Fixed     => ls_val.max(max_fs),
    LineSpacingType::SpaceOnly => max_fs + ls_val,
    LineSpacingType::Minimum   => ls_val.max(max_fs),
}
```

pi=443 예 (ParaShape ps_id=32, line=160/Percent, font=13pt):
- max_fs = 17.3 px
- ls_val = 160 (%)
- ls_type = Percent
- computed = 17.3 * 160 / 100 = **27.7 px** (vs 현재 5.33 px)

→ wrap 시 line_step ≈ 27.7 px 가 되어 겹침 해소.

## 4. 영향 분석

### 적용 대상
- `para.line_segs.is_empty()` AND `!para.text.is_empty()` paragraph
- HWP5 변환본의 일부 paragraph 가 해당 (59 개 / 529 = 11%)

### 비영향 대상
- HWP3 / HWPX (line_segs 존재) — fallback branch 미진입
- HWP5 의 line_segs 있는 paragraph — fallback branch 미진입

### 회귀 위험
- 다른 HWP5 sample 의 line_segs 누락 paragraph 도 동일 적용 (개선)
- 빈 paragraph (text 없음) 는 영향 없음 (text reflow 미발생)

## 5. Stage 2 진입 준비

G1 구현 위치:
1. `paragraph_layout.rs` 의 line_height/baseline_dist/line_step 계산 (line 334-398)
2. `compose_lines` 의 fallback line_height (composer.rs:319) — optional 추가 보정
