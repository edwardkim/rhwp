# Task #1037 Stage 1 완료 보고서 — Root cause 단언

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정](https://github.com/edwardkim/rhwp/issues/1037)
**Branch**: `local/task1037`
**작업 내용**: HWP3 vs HWP5 변환본 paragraph 데이터 비교 + height 산출 경로 추적 + root cause 단언

---

## 1. 진단 test 결과 (`tests/diag_1037_height.rs`)

sample16 pi=450 비교:

```
=== HWP3 pi=450 ===
  CharShape: base_size=1000 spacings[0]=0  ratios[0]=100 font_ids[0]=1
  ParaShape: line_spacing=160 (Percent) spacing_before=1132 margin_l=6000 indent=-2000
  LineSegs.len() = 3 (있음)
    [0] vpos=0    lh=1300 th=1300 bl=1105 ls=780
    [1] vpos=2080 lh=1300 th=1300 bl=1105 ls=780
    [2] vpos=4160 lh=1300 th=1300 bl=1105 ls=780
  text_len = 139

=== HWP5 변환본 pi=450 ===
  CharShape: base_size=1300 spacings[0]=-12 ratios[0]=100 font_ids[0]=7
  ParaShape: line_spacing=160 (Percent) spacing_before=2264 margin_l=8000 indent=-4000
  LineSegs.len() = 0 (★ 빈 — encoder 가 typeset 안 함)
  text_len = 140
```

**핵심 발견**:
1. **HWP5 변환본 LineSegs.len() = 0** (모든 paragraph) — encoder 가 라인 분할 안 함
2. CharShape base_size: 1000 → **1300** (HWP3 10pt → HWP5 변환본 13pt, +30%)
3. ParaShape spacing_before: 1132 → **2264** (×2, variant_div=4 패턴)
4. ParaShape margin_l: 6000 → 8000

---

## 2. height 산출 경로 추적

### 2.1 composer.rs:441 fallback (LineSegs 빈 경우)

```rust
if para.line_segs.is_empty() {
    // ...
    const CHARS_PER_LINE: usize = 45;
    // 텍스트를 45 chars 단위로 word wrap synth line 생성
    lines.push(ComposedLine {
        ...
        line_height: 400,        // 하드코딩 (HU)
        baseline_distance: 320,
        ...
    });
}
```

### 2.2 layout/paragraph_layout.rs:1160~1167 corrected_line_height

```rust
let raw_lh = hwpunit_to_px(comp_line.line_height, self.dpi);  // 400 HU → 5.33 px
let line_height = corrected_line_height(raw_lh, max_fs, ls_type, ls_val);
// = max_fs * ls_val / 100 (Percent type, raw_lh < max_fs)
```

### 2.3 corrected_line_height (mod.rs:544)

```rust
if max_fs > 0.0 && raw_lh < max_fs {
    match ls_type {
        LineSpacingType::Percent => max_fs * ls_val / 100.0,  // ← 변환본 적용 경로
        ...
    }
} else {
    raw_lh  // ← HWP3 actual 적용 경로
}
```

---

## 3. Root cause 단언 (정량)

| | HWP3 | HWP5 변환본 |
|---|------|------------|
| LineSegs | 3 (actual) | 0 (synth fallback) |
| raw_lh | 1300 HU | 400 HU (synth) |
| max_fs (= base_size) | 1000 HU = 13.3 px | 1300 HU = 17.3 px |
| corrected_line_height | raw_lh=1300 > max_fs=1000 → **no correction → 1300** | raw_lh=400 < max_fs=1300 → **correction: 1300 × 160/100 = 2080** |
| per-line height (HU) | 1300 | **2080 (×1.6)** |
| 라인 수 (140 chars) | 3 (실제) | 4 (synth: 140/45) |
| 총 lines height (HU) | 3 × 1300 = **3900** | 4 × 2080 = **8320** (**×2.13**) |
| 총 lines height (px) | 52.0 | 110.9 |

**Root cause = 2 요소 곱**:
1. **per-line height**: HWP5 변환본 corrected_line_height (= 2080) > HWP3 actual lh (= 1300), 비율 ×1.6
2. **라인 수**: HWP5 변환본 synth (= 4) > HWP3 actual (= 3), 비율 ×1.33

합산: ×1.6 × ×1.33 ≈ ×2.13 (관찰값 정합).

---

## 4. Fix 방향 (Stage 2 후보)

### 4.1 후보 A — composer fallback line_height = max_fs (per-line height 정합)

`composer.rs:441` 의 synth line_height 를 `max_fs` 로 설정 (max_fs 정보를 composer 에 전달). 또는 sentinel 값 (예: 0) 으로 marker 한 후 layout 에서 `corrected_line_height` 가 인식.

effect: HWP5 변환본 corrected_line_height → max_fs = 1300 (HWP3 actual 동등). 비율 ×1.6 제거.

### 4.2 후보 B — composer fallback CHARS_PER_LINE 조정 (라인 수 정합)

`CHARS_PER_LINE` 45 → 50 (PR #1009 claim, actual diff 미적용). HWP5 변환본 pi=450 text_len=140 → 140/50 = 2.8 → 3 lines (HWP3 actual 동등). 비율 ×1.33 제거.

### 4.3 후보 A + B 결합 (권고)

- A: line_height = max_fs
- B: CHARS_PER_LINE = 50

결과 (HWP5 변환본 pi=450): 3 lines × 1300 HU = 3900 HU = **HWP3 actual 정합** ✓

### 4.4 variant 가드

위 fix 를 variant 한정 (is_hwp3_variant) 으로 적용 또는 fallback 전체 (line_segs.is_empty()) 적용:
- variant 한정: 다른 일반 HWP5 fixture 무영향
- fallback 전체: line_segs 누락 모든 case (Task #994 fix scope) 영향

**권고**: line_segs.is_empty() fallback 자체에 적용 — 어차피 fallback 은 variant 한정 trigger.

---

## 5. 다음 단계 (Stage 2)

후보 A + B 결합 fix 적용 + 회귀 sweep 측정.

### 5.1 Stage 2 변경 위치 후보

| 항목 | 위치 |
|------|------|
| CHARS_PER_LINE 45 → 50 | `src/renderer/composer.rs:462` |
| synth line_height = max_fs (또는 sentinel) | `src/renderer/composer.rs:483~502` |
| corrected_line_height 보정 (필요 시) | `src/renderer/mod.rs:544` 또는 `paragraph_layout.rs:1166` |

### 5.2 검증 단계

- HWP5 변환본 pi=450 paragraph height = HWP3 동등 단언 (52.0 px)
- p23 PartialParagraph split 회피 (pi=460 FullParagraph fit)
- alignment 정합률 측정 (PR #1036 의 60/64 유지 또는 향상)
- 변환본 + 일반 fixture 회귀 sweep
