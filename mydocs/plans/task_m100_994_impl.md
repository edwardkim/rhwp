# Task #994 구현 계획서 — G1 (paragraph_layout fallback line_height 합성)

- 이슈: [#994](https://github.com/edwardkim/rhwp/issues/994)
- 선행: [Stage 1 진단](../working/task_m100_994_stage1.md)
- 브랜치: `local/task994`

## 1. 변경 위치

[src/renderer/layout/paragraph_layout.rs:334-398](src/renderer/layout/paragraph_layout.rs#L334-L398)

## 2. 구체적 변경

### Before
```rust
let line_height = if let Some(ls) = para.line_segs.first() {
    hwpunit_to_px(ls.line_height, self.dpi)
} else {
    hwpunit_to_px(400, self.dpi)   // ← 5.33 px 고정 (overlap)
};

let baseline_dist = if let Some(ls) = para.line_segs.first() {
    ensure_min_baseline(hwpunit_to_px(ls.baseline_distance, self.dpi), para_max_font_size)
} else {
    line_height * 0.8
};

let line_step = ... else {
    baseline_dist * 1.5
};
```

### After
```rust
// [Task #994] line_segs 누락 시 ParaShape line_spacing 으로 line_height 동적 계산.
// HWP3→HWP5 변환 시 일부 paragraph 의 PARA_LINE_SEG 가 누락 (sample16 의 59개).
// 기존 fallback (400 HU = 5.33 px) 으로 단일 ComposedLine 생성 후
// paragraph_layout 의 text wrap 시 line_step=6.4 px 로 겹침 발생.
let line_height = if let Some(ls) = para.line_segs.first() {
    hwpunit_to_px(ls.line_height, self.dpi)
} else {
    // ParaShape line_spacing_type + max font_size 로 합성
    let para_style = ...;
    let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
    let ls_type = para_style.map(|s| s.line_spacing_type).unwrap_or(LineSpacingType::Percent);
    let computed = match ls_type {
        LineSpacingType::Percent   => para_max_font_size * ls_val / 100.0,
        LineSpacingType::Fixed     => ls_val.max(para_max_font_size),
        LineSpacingType::SpaceOnly => para_max_font_size + ls_val,
        LineSpacingType::Minimum   => ls_val.max(para_max_font_size),
    };
    computed.max(para_max_font_size)
};

let baseline_dist = if let Some(ls) = para.line_segs.first() {
    ensure_min_baseline(hwpunit_to_px(ls.baseline_distance, self.dpi), para_max_font_size)
} else {
    // 폰트 사이즈 기반 baseline (대체로 line_height 의 85%)
    ensure_min_baseline(line_height * 0.85, para_max_font_size)
};

let line_step = ... else {
    // line_height 자체 사용 (line_spacing 이미 포함됨)
    line_height
};
```

## 3. 의존성

- `para_max_font_size` 가 line_height 전에 계산되어야 함 → 순서 조정
- `LineSpacingType` import 필요
- `para_style` 조회 — `composed.map(|c| c.para_style_id as usize).unwrap_or(0)` 사용

## 4. 영향 추정

### 적용 대상
- HWP5 의 line_segs 누락 paragraph (sample16 약 59개, 다른 sample 도 가능)
- HWPX 변종 영향 가능 (parser path 확인 필요)

### 비영향
- HWP3 paragraphs (line_segs 보유) — fallback 미진입
- HWP5/HWPX 의 line_segs 있는 paragraph — fallback 미진입

## 5. Stage 진행

| Stage | 내용 | 검증 |
|-------|------|------|
| 2 | G1 구현 | rustfmt 통과 |
| 3 | cargo test --release --lib | 통과 목표 |
| 4 | 240 sample 페이지 수 회귀 | 변동 0 목표 |
| 5 | HWP5 sample16 page 19+ 시각 | 겹침 해소 목표 |
| 6 | commit + PR | base: devel |

## 6. 회귀 방어

- 기존 fallback (400 HU) 으로 동작하던 paragraph 가 있다면 line_height 증가 — paragraph 자체 height 증가 → 페이지 수 증가 가능
- 빈 paragraph (text 없음) 미영향
- ensure_min_baseline 가드 유지
