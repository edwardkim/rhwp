# Task #722 Stage 9 단계별 보고서 — HWP5 변환본 매칭 가드 정정

## 개요

Stage 8 진단 결과에 따라 typeset 매칭 분기에 anchor full-width 가드 추가. HWP5 변환본 paragraph 441 wrap zone 매칭 회복.

## 1. 정정 — `src/renderer/typeset.rs:492~500`

```rust
// [Task #722 Stage 9] HWP5 변환본 case: anchor host cs=0 + sw=body_w
// (col_area 전체 폭 caption) + 다음 paragraph cs>0 (자체 wrap zone 인코딩)
// → wrap zone 매칭. HWP3 native 와 다른 한컴 변환본 IR 패턴 정합용.
let anchor_full_width_match = st.wrap_around_cs == 0
    && (st.wrap_around_sw - body_w).abs() < 200
    && para_cs > 0
    && para_sw > 0
    && para_cs + para_sw <= body_w + 200;
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_full_width_match {
```

기존 매칭 분기에 OR 조건 추가 (총 4 가드).

## 2. Case 가드 정합 검증

| paragraph | anchor cs/sw | para cs/sw | 가드 | 매칭 |
|-----------|-------------|-----------|------|------|
| HWP3 paragraph 175 | (어떤 paragraph) | 24560/26464 | exact match | ✓ |
| HWP3 paragraph 176 | 24560/26464 | 24560/26464 | exact match | ✓ |
| HWP5 paragraph 440 | (paragraph 자체 등록) | 0/51024 | self register (caption_room 가드) | caption-style 미등록 |
| HWP5 paragraph 441 | 0/51024 | 22800/28224 | **anchor_full_width_match** | ✓ (Stage 9) |

`anchor_full_width_match` 가드:
- anchor cs=0 → caption-style anchor host (텍스트가 col_area 전체 폭)
- anchor sw≈body_w → caption 텍스트가 col_area 전체 폭 인코딩
- para cs>0, sw>0 → 다음 paragraph 가 자체 wrap zone 인코딩
- para cs+sw ≤ body_w → wrap zone 폭이 col_area 안 (정합)

## 3. 시각 판정 (rsvg-convert PNG)

- **stage9_hwp5_p16.png** — paragraph 441 image 우측 wrap zone 정합 ✓
- **stage9_p8.png** — paragraph 175 wrap zone 보존
- **stage9_p27.png** — paragraph 779 caption 보존
- **stage9_p48.png** — paragraph 1394 wrap zone + gap 3mm 보존

## 4. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep (209 fixture) | **DIFF 0** (회귀 0) |

## 5. 회귀 위험 영역 확인

- typeset.rs 한 분기 (가드 OR 조건 추가)
- HWP5 변환본 한컴 IR 패턴 한정 (anchor cs=0 + 전체 폭 sw + 다음 para cs>0)
- HWP3 native 미발현 (HWP3 anchor host LINE_SEG 가 wrap zone 인코딩 → 매칭 가드 미발현)
- 광범위 sweep 페이지 수 차이 0

## 6. Stage 10 진행 승인 요청

광범위 sweep + 시각 판정 통과. 최종 결과 보고서 작성 + commit + closes #722 진행 승인 요청.
