# Task #722 Stage 5 단계별 보고서 — inter-image-text gap 본질 진단

## 진단 결과 요약

페이지 48 paragraph 1394 (Stage 4 후 wrap zone 회복) 의 image 끝 → 텍스트 시작 사이 gap 차이의 본질:

**모든 wrap=Square image 의 outer margin = 852 HU = 3.0mm** (좌우상하 동일).
한컴 viewer 는 inter-image-text gap 으로 image margin_right 를 추가 적용. 본 환경 미적용.

## 1. 임시 디버그 진단 (DEBUG_T722_MARGIN)

`src/renderer/typeset.rs` 의 anchor host paragraph register 분기에 임시 print 추가하여 wrap=Square image 의 margin/cs/sw 출력:

```
[T722_MARGIN] pi=74   margin l=852 r=852 t=852 b=852 | cs=35460 sw=15564 | h_off=3872 v_off=5720 w=35840 h=26788
[T722_MARGIN] pi=175  margin l=852 r=852 t=852 b=852 | cs=24560 sw=26464 | h_off=3572 v_off=18680 w=25240 h=13172
[T722_MARGIN] pi=440  margin l=852 r=852 t=852 b=852 | cs=21096 sw=29928 | h_off=3992 v_off=18560 w=21356 h=15352
[T722_MARGIN] pi=599  margin l=852 r=852 t=852 b=852 | cs=26256 sw=24768 | h_off=3152 v_off=5668 w=27356 h=26440
[T722_MARGIN] pi=773  margin l=852 r=852 t=852 b=852 | cs=38876 sw=12148 | h_off=3512 v_off=5668 w=39616 h=6628
[T722_MARGIN] pi=779  margin l=852 r=852 t=852 b=852 | cs=24724 sw=26300 | h_off=3572 v_off=15400 w=25240 h=13356
[T722_MARGIN] pi=1394 margin l=852 r=852 t=852 b=852 | cs=17236 sw=33788 | h_off=3572 v_off=5668 w=17916 h=28180
```

모든 wrap=Square image margin = 852 HU = **3.0mm 좌우상하 동일**.

## 2. paragraph 1394 inter-image-text gap 분석

**본 환경 layout (Stage 4 적용 후)**:
- image 끝 x px = `col_area.x(56.69) + cs_px(229.81)` = 286.51 (= image x_offset 47.63 + width 238.88 = 286.51 ✓)
- "￼" placeholder x = 286.51 (= image 끝)
- "접" 첫 가시 char x = 292.51 (= 286.51 + 6 px placeholder width)

**한컴 정합 추정** (`col_area.x + cs + image_margin_right`):
- text 시작 x = `56.69 + (17236+852)/7200×96` = `56.69 + 240.83` = 297.52
- 차이 ≈ 297.52 - 292.51 = **5.01 px ≈ 1.33mm**

inter-image-text gap = 0mm (본 환경) vs ~3mm (한컴 정합). image margin_right (3mm) 만큼 차이.

## 3. paragraph_layout 의 wrap_anchor 처리 영역

`src/renderer/layout/paragraph_layout.rs:1018~1027` 에서 wrap_anchor.is_some() 시 LINE_SEG.cs/sw 직접 사용:

```rust
let (line_cs_offset, line_avail_w_override) = if wrap_anchor.is_some() {
    let seg = para.and_then(|p| p.line_segs.get(line_idx));
    let cs = seg.map(|s| s.column_start as i32).unwrap_or(0);
    let sw = seg.map(|s| s.segment_width as i32).unwrap_or(0);
    let cs_px = crate::renderer::hwpunit_to_px(cs, self.dpi);
    let sw_px = if sw > 0 { Some(crate::renderer::hwpunit_to_px(sw, self.dpi)) } else { None };
    (cs_px, sw_px)
} else {
    (0.0, None)
};
```

LINE_SEG.cs 는 image 끝 x (col_area.x 기준) HU 와 거의 일치 (cs HU = image_x_offset + image_width - col_area.x). image margin_right 미적용 → text 시작이 image 끝 정합.

## 4. 정정 방향 (Stage 6)

### 옵션 A: WrapAnchorRef 확장 + paragraph_layout 보정

1. `src/renderer/pagination.rs` — WrapAnchorRef 에 `anchor_image_margin_right: i32` 필드 추가
2. `src/renderer/typeset.rs` — anchor host paragraph register 시 image margin_right 추출 + register
3. `src/renderer/layout/paragraph_layout.rs` — wrap_anchor 처리에서 cs px 에 margin_right_px 추가, sw px 에서 margin_right_px 차감

장점:
- 기존 case 가드 영역 그대로 활용
- WrapAnchorRef 확장 명시적
- image margin 본질 데이터 명확

### 옵션 B: paragraph_layout 단 image 직접 lookup

paragraph_layout 에서 anchor paragraph 의 controls iterate 후 image margin_right 추출. WrapAnchorRef 미확장.

단점: paragraph_layout 이 anchor paragraph 까지 알아야 함 (책임 증가).

### 권고: 옵션 A

회귀 위험 좁힘 + 데이터 모델 명시 영역. typeset 단에서 margin 추출 후 register 하면 paragraph_layout 은 단순히 사용.

## 5. Stage 6 진행 승인 요청

본 진단 결과 + Stage 6 정정 방향 (옵션 A: WrapAnchorRef 확장 + paragraph_layout cs/sw 보정) 승인 요청.
