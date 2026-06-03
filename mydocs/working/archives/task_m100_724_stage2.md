# Task #724 Stage 2 단계별 보고서 — 본질 정정 (가설 A + D)

## 개요

Stage 1 진단에 따라 typeset.rs 의 wrap_around state machine 에 가설 A (image expected_cs 정확 일치 매칭 가드) + 가설 D (vpos-reset 시 wrap_around 강제 종료) 결합 정정.

## 1. 가설 A 정정 — `src/renderer/typeset.rs:495~520`

기존 매칭 분기 (`exact match || any_seg_matches || sw0_match`) 에 `anchor_image_match` 가드 추가:

```rust
// [Task #724] HWP5 변환본 case: anchor host 의 wrap=Square image 위치/폭/margin
// 으로 expected_cs 정확 계산 후 para_cs 일치 확인. anchor cs=0 (caption-style)
// 한정 가드. expected_cs = (image_x_offset + width + 2*margin) - body_left.
let anchor_image_match = if st.wrap_around_cs == 0 {
    let body_left = page_def.margin_left as i32;
    let expected_cs_hu = paragraphs.get(st.wrap_around_table_para)
        .and_then(|p| p.controls.iter().find_map(|c| {
            let cm = match c {
                Control::Picture(pic) => Some(&pic.common),
                Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                    Some(&pic.common)
                } else { None },
                _ => None,
            };
            cm.filter(|cm| !cm.treat_as_char
                && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
                .map(|cm| cm.horizontal_offset as i32 + cm.width as i32
                    + 2 * cm.margin.right as i32 - body_left)
        }))
        .unwrap_or(0);
    expected_cs_hu > 0
        && (para_cs - expected_cs_hu).abs() < 200
        && para_sw > 0
        && para_cs + para_sw <= body_w + 200
} else { false };
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_image_match {
```

paragraph 441/442/443 모두 cs=22800 = expected_cs ✓ 매칭 → wrap_anchors 등록.

## 2. 가설 D 정정 — `src/renderer/typeset.rs:417~445`

기존 vpos-reset 가드 분기 (Task #321 + Task #362) 의 `st.wrap_around_cs < 0` 조건 제거 + anchor cs=0 한정 강제 종료 분기 추가:

```rust
// [Task #362] wrap-around zone 활성 중에는 vpos-reset 가드 무시 (기존).
// [Task #724] vpos-reset trigger 발동 시 wrap_around 강제 종료 (신규):
if para_idx > 0 && !st.current_items.is_empty() {
    let prev_para = &paragraphs[para_idx - 1];
    let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
    let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
    if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
        let trigger = if st.col_count > 1 {
            cv < pv && pv > 5000
        } else {
            cv == 0 && pv > 5000
        };
        if trigger {
            // [Task #724] wrap_around active 시 강제 종료 — anchor cs=0
            // (HWP5 변환본 caption-style) 한정. 일반 wrap_around (anchor cs>0)
            // 는 기존 동작 (Task #362 vpos-reset 무시) 유지.
            if st.wrap_around_cs == 0 {
                st.wrap_around_cs = -1;
                st.wrap_around_sw = -1;
                st.wrap_around_any_seg = false;
            }
            if st.wrap_around_cs < 0 {
                st.advance_column_or_new_page();
            }
        }
    }
}
```

가드 좁힘 영역:
- anchor cs=0 case 한정 — HWP5 변환본 caption-style 만 강제 종료
- 일반 wrap_around (anchor cs>0) — 기존 Task #362 동작 (vpos-reset 무시) 유지

## 3. 가드 좁힘 검증

| fixture | baseline | 가설 D 광범위 (kps-ai 회귀) | 가설 D 좁힘 (현재) |
|---------|----------|-----------------------------|---------------------|
| `kps-ai.hwp` | 79 | **80** (회귀) | 79 ✓ |
| `hwp3-sample5-hwp5.hwp` 페이지 22 | paragraph 599 시작 | paragraph 599 시작 | paragraph 599 시작 ✓ |

가드 좁힘으로 kps-ai.hwp 회귀 0 + hwp3-sample5-hwp5.hwp 페이지 분할 정합 동시 달성.

## 4. 시각 판정 (rsvg-convert PNG)

- **final_p16.png**: paragraph 441 + 442 + 443 모두 image 우측 wrap zone 정합 (PDF 정합) ✓
- **final_p22.png**: MBR image (paragraph 599) + 후속 paragraph 정상 layout (paragraph 599 페이지 22 시작 정합) ✓
- HWP3 native 페이지 8/27/48 (PR #723) 정합 보존 ✓

## 5. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1166 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep (209 fixture) | **DIFF 0** (회귀 0) |

## 6. Stage 3 진행 승인 요청

광범위 sweep + 결정적 검증 + 시각 판정 통과. 최종 결과 보고서 작성 + commit + PR 준비 진행 승인 요청.
