# Task #969 Stage 1 — 진단 정밀화

- 이슈: [#969](https://github.com/edwardkim/cargo/rhwp/issues/969)
- 부모: [#942](https://github.com/edwardkim/rhwp/issues/942)
- 브랜치: `local/task969`

## 1. 영향 범위 (HWPX sample16-hwp5)

`rhwp ir-diff samples/hwp3-sample16-hwp5.hwpx samples/hwp3-sample16-hwp5.hwp` 결과:

- **line_segs count 차이: 59 건** (모두 A=1, B=0 패턴)
- 영향 paragraph: 376, 380, 383, 386, 395, 400, 403, 407, 410, 417, 424, 426, 431, 437~439, 442~446, 449~455, 457, 460~462, 464, 469~471, 474~478, 481, 485~486, 641, 651~654, 816, 820, 829, 839, 859~860, 874, 879, 893, 897 …

**공통 패턴**: 거의 모두 `󰏅` (PUA `\u{f03c5}`, HWP 항목 부호) 시작.

## 2. 3 가지 변종 비교 (pi=0.395 기준)

| 항목 | HWP3 (`hwp3-sample16.hwp`) | HWPX (`hwp3-sample16-hwp5.hwpx`) | HWP5 (`hwp3-sample16-hwp5.hwp`) |
|------|---|---|---|
| 페이지 수 | **64 (= PDF)** ✓ | **72 (+8)** ✗ | 62 (-2) |
| 텍스트 첫 글자 | `○` (U+25CB) | `󰏅` (U+F03C5 PUA) | `󰏅` (U+F03C5 PUA) |
| ParaShape spacing_before | 1132 HU | **2264 HU** (= 2× HWP3) | 2264 HU |
| ParaShape margins.left | 6000 HU | **8000 HU** | 8000 HU |
| line_segs count | **1** (lh=1300, ls=780) | **1** (lh=1299, ls=779) | **0** |

## 3. format_paragraph 의 3 단계 분기

[src/renderer/typeset.rs:1190-1229](src/renderer/typeset.rs#L1190-L1229)

```rust
let (line_heights, line_spacings) = if let Some(comp) = composed {
    // [1] composed (text-shaping engine 결과) 가 있으면 사용 — Primary
    comp.lines.iter().map(|line| {
        // raw_lh = line.line_height
        // max_fs = max font_size in runs
        // lh = if raw_lh < max_fs: ParaShape line_spacing 기반 재계산
        //      else: raw_lh 유지
    })
} else if !para.line_segs.is_empty() {
    // [2] composed 없으면 preset line_segs 사용 — Fallback
    para.line_segs.iter().map(|seg| (seg.line_height, seg.line_spacing))
} else {
    // [3] 둘 다 없으면 default 400 HU
    (vec![400 HU], vec![0])
}
```

| 변종 | 어느 branch ? |
|------|--------------|
| HWP3 | [1] composed 또는 [2] preset (lh=1300) — 둘 다 ~27.7 px |
| HWPX | [2] preset (lh=1299) — 27.7 px |
| HWP5 | [3] default 400 HU = **5.33 px** |

## 4. Stage 7.8 진단과 정합

Stage 7.8 의 `pi=395 cur_h=985.5 > avail=967.3 (18.2 px overflow)` 와 일치:
- HWPX 가 paragraph 당 ~22 px 더 큰 height 산출 (vs HWP5 의 5.33 px)
- 8 개 paragraph 누적 시 한 페이지 추가 발생 가능

## 5. HWP3 = 64 페이지 (정답) 인 이유

HWP3 도 line_segs 가 있고 height 가 HWPX 와 거의 동일 (lh=1300 vs 1299).
하지만 **ParaShape spacing_before 가 절반** (1132 vs 2264 HU = 약 15 px 차이).

59 paragraph × 15 px ≈ 885 px ≈ 0.9 페이지 차이 — 직접 비교 불가능.

→ HWP3 의 정답 페이지 수 (64) 는 **다른 paragraph 의 spacing 차이** 도 종합된 결과.

## 6. 결론

**Root cause**: HWPX 파서가 `<hp:linesegarray>` 의 `vertsize/spacing` 값을 IR LineSeg 로 그대로 emit. typeset 이 composed 없을 때 [2] preset branch 사용 → HWP5 의 [3] default 400 HU 와 ~22 px/para 차이.

**불확실 영역**:
- composed (text-shaping engine 결과) 이 이 paragraph 들에 대해 **None** 으로 떨어지는 이유 — `composed.get(para_idx)` 가 None 반환 또는 composed.lines 가 empty?
- composed 가 정상 작동했다면 HWPX/HWP5 모두 [1] primary branch 로 정합 가능

## 7. 다음 Stage (2) 입력

- 후보 평가 시 D1 (HWPX 파서 preset emit 차단), D2 (format_paragraph preset branch 무시), D3 (preset 값 보정), D4 (preset position 유지 + height 재계산) + **D5 (composed 없음 case 분석 → composed 가 항상 작동하도록 보정)** 추가 검토
- 가장 직관적: D5 가 fundamentally correct (HWP3/HWP5/HWPX 모두 동일 typeset 결과 보장)
- D1 은 HWPX→HWP5 정합 효과 빠르지만 다른 영향 가능
