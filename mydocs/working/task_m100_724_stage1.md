# Task #724 Stage 1 단계별 보고서 — 본질 진단

## 진단 결과 요약

`hwp3-sample5-hwp5.hwp` (HWP3→HWP5 한컴 변환본) 페이지 16 paragraph 441 wrap zone 매칭 실패의 본질 + Stage 9 회귀 (페이지 분할 왜곡) 본질 식별. 가설 A (image expected_cs 정확 일치) + 가설 D (vpos-reset wrap_around 강제 종료) 결합 정정 방향 도출.

## 1. paragraph 441 wrap zone 매칭 실패 본질

### IR 패턴 비교

| 항목 | HWP3 native (paragraph 175) | HWP5 변환본 (paragraph 440/441) |
|------|------------------------------|----------------------------------|
| anchor host LINE_SEG.cs | 24560 (wrap zone) | **0** (col_area 전체 폭) |
| anchor host LINE_SEG.sw | 26464 | **51024** (col_area_width) |
| 다음 paragraph LINE_SEG.cs | 24560 (anchor 동일) | **22800** (자체 wrap zone) |
| 다음 paragraph LINE_SEG.sw | 26464 (anchor 동일) | **28224** |

HWP5 변환본 한컴 viewer 는 anchor host caption-style + 다음 paragraph 자체 wrap zone 인코딩한 IR 정합 처리. 본 환경 typeset wrap_around state machine 매칭 가드 부재.

### 본 환경 매칭 가드 분석 (`src/renderer/typeset.rs:495-497`)

paragraph 441 매칭:
- exact match: 22800 vs 0 (cs), 28224 vs 51024 (sw) → 모두 불일치
- any_seg_matches: 모두 불일치
- sw0_match: anchor sw=51024 ≠ 0 → 미해당

→ **매칭 실패** → wrap_anchors 미등록 → composer/paragraph_layout 의 col_area 전체 폭 사용 → image 영역 침범 → image z-order 후 그려져 텍스트 가려짐 (visual: "이타를", "마다", "head)" 등 짧은 보이는 부분만).

## 2. 가설 A 검증 — image expected_cs 정확 계산

paragraph 440 image: x_offset=3992, width=21356, margin=852, body_left=4252.

```
expected_cs = (image_x_offset + image_width + 2 × image_margin) - body_left
            = (3992 + 21356 + 2 × 852) - 4252
            = 22800
```

**paragraph 441/442/443 LINE_SEG cs = 22800 정확 일치** ✓

다른 paragraph 의 `cs` 값과 비교하면 가드 조건 (anchor cs=0 + para cs=expected_cs) 이 정확히 본 case 한정 적용.

## 3. Stage 9 회귀 (페이지 분할 왜곡) 본질

### 회귀 증상 (Task #722 Stage 9)

`anchor_full_width_match` broad 가드 (anchor cs=0 + sw=body_w + para cs/sw>0) 적용 시:
- baseline: paragraph 599 (MBR image) 페이지 22 시작
- Stage 9: paragraph 599 페이지 21 끝 + paragraph 603 lines 0..3 페이지 21 진입 (페이지 분할 왜곡)
- dump-pages hwp_used=127.2 vs used=998.4 (페이지 21 vpos 흐름 비정상)

### 본질 — Task #321 vpos-reset 가드의 wrap_around 무시 조건

`src/renderer/typeset.rs:419~440` 의 vpos-reset 가드:

```rust
if para_idx > 0 && !st.current_items.is_empty() && st.wrap_around_cs < 0 {
    // 현재 문단 vpos=0 + 직전 vpos > 5000 → 페이지 reset
    if trigger {
        st.advance_column_or_new_page();
    }
}
```

**`st.wrap_around_cs < 0` 조건** (Task #362) 으로 wrap_around active 시 vpos-reset 무시.

paragraph 442/443 매칭 → wrap_around 유지 → paragraph 599 vpos=0 시점에도 wrap_around active → vpos-reset 무시 → 페이지 분할 위반.

## 4. 가설 D — vpos-reset 시 wrap_around 강제 종료

**조건 가드**: anchor cs=0 (HWP5 변환본 caption-style) 한정. 일반 wrap_around (anchor cs>0) 는 기존 Task #362 동작 유지.

```rust
if trigger {
    // [Task #724] anchor cs=0 (HWP5 변환본 caption-style) 한정 wrap_around 종료
    if st.wrap_around_cs == 0 {
        st.wrap_around_cs = -1;
        st.wrap_around_sw = -1;
        st.wrap_around_any_seg = false;
    }
    if st.wrap_around_cs < 0 {
        st.advance_column_or_new_page();
    }
}
```

광범위 sweep 회귀 검증: anchor cs=0 case 한정으로 좁혀 다른 fixture (kps-ai 등) 무영향.

## 5. Stage 2 진행 승인 요청

본 진단 결과 + Stage 2 정정 방향 (가설 A + D 결합, 모두 anchor cs=0 한정 가드) 승인 요청.
