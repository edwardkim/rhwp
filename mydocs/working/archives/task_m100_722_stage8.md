# Task #722 Stage 8 단계별 보고서 — HWP5 변환본 paragraph 441 본질 진단

## 진단 결과 요약

hwp3-sample5-hwp5.hwp (HWP3→HWP5 한컴 변환본) 페이지 16 의 paragraph 441 wrap zone 매칭 실패. typeset.rs 의 매칭 가드가 HWP5 변환본 IR 패턴 (anchor host cs=0 + 전체 폭 sw) 미지원.

baseline 과 Stage 6 적용 PNG 동일 → Stage 1~6 정정과 무관한 사전 결함이지만, 본 task #722 (hwp3-sample5 wrap=Square 그림 paragraph 시각 정합) 영역 안.

## 1. IR 비교 (HWP5 변환본 페이지 16)

paragraph 440 (anchor host, "Figure 4-1은 하드디스크안의 중요부분의 개략도이다."):

```
ls[0]: ts=0, vpos=11520, lh=900, th=900, bl=765, ls=540, cs=0, sw=51024
[0] 그림: bin_id=3, common=21356×15352 (75.3×54.2mm)
    위치: h_off=14.1mm(3992), v_off=65.5mm(18560)
    배치: 어울림(Square), tac=false
```

paragraph 441 (wrap text, "하드디스크는 하나 이상의..."):

```
ls[0..7]: ts=0,39,73,114,158,193,231,267
ls[i].cs=22800, sw=28224 (모두 동일)
```

## 2. 본질 분석

### paragraph 441 cs/sw 정합 영역

- col_area: x=15mm, width=180mm (51024 HU)
- image: x_offset=14.1mm, width=75.3mm → image 끝 x=89.4mm
- paragraph 441 cs=22800 HU=80.4mm → col_area.x + cs = 95.4mm (image 끝 + 6mm = 2 × image margin)
- paragraph 441 sw=28224 HU = 99.6mm = col_area_width(180mm) - cs(80.4mm) ✓

paragraph 441 LINE_SEG 자체가 image 우측 wrap zone 을 정확히 인코딩.

### typeset 매칭 가드 분석 (`src/renderer/typeset.rs:495-497`)

```rust
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match {
    // 매칭 → wrap_anchors 등록
}
```

paragraph 441 매칭 시도:
- exact match: 22800 vs 0 (cs), 28224 vs 51024 (sw) → 모두 불일치
- any_seg_matches: ls[i].cs=22800 vs anchor 0 / sw=28224 vs anchor 51024 → 모두 불일치
- sw0_match: anchor sw=51024 ≠ 0 → 미해당

**매칭 실패** → wrap zone 종료 → paragraph 441 wrap_anchors 미등록 → col_area 전체 폭 layout 의도, 그러나 ComposedLine 의 LINE_SEG sw 가 paragraph_layout 또는 composer 단에서 직접 사용되어 좁은 폭 분할 결과.

## 3. HWP3 native vs HWP5 변환본 anchor host IR 차이

| 항목 | HWP3 native (paragraph 175) | HWP5 변환본 (paragraph 440) |
|------|------------------------------|------------------------------|
| anchor host LINE_SEG.cs | 24560 (wrap zone 인코딩) | 0 (col_area 전체) |
| anchor host LINE_SEG.sw | 26464 | 51024 (col_area 전체 폭) |
| 다음 paragraph LINE_SEG.cs/sw | 24560/26464 (anchor 와 동일) | 22800/28224 (다른 값) |

HWP3 native: anchor host 자체가 wrap zone 텍스트 → 다음 paragraph 도 같은 cs/sw → exact match.

HWP5 변환본: anchor host 가 col_area 전체 폭 caption 텍스트 → 다음 paragraph 가 자체 wrap zone 인코딩 → exact match 실패.

## 4. 정정 방향 (Stage 9)

typeset 매칭 가드 확장:

```rust
// HWP5 변환본 case: anchor host cs=0 + sw=body_w (전체 폭 caption) +
// 다음 paragraph cs>0 (자체 wrap zone 인코딩) → wrap zone 매칭
let body_w_hu = (page_def.width - page_def.margin_left - page_def.margin_right) as i32;
let anchor_full_width_match = st.wrap_around_cs == 0
    && (st.wrap_around_sw - body_w_hu).abs() < 200  // tolerance
    && para_cs > 0
    && para_sw > 0
    && para_cs + para_sw <= body_w_hu + 200;
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_full_width_match {
    // 매칭 → wrap_anchors 등록
}
```

### 회귀 위험 영역

- HWP5 변환본 anchor host cs=0 + 전체 폭 case 한정 (HWP3 native 미발현)
- 다음 paragraph 의 cs+sw <= body_w (정합) 추가 가드
- 광범위 fixture sweep 으로 회귀 검증

## 5. Stage 9 진행 승인 요청

본 진단 결과 + Stage 9 정정 방향 (anchor full-width 매칭 가드 추가) 승인 요청.
