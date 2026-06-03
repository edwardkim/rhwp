# Task #722 Stage 6 단계별 보고서 — inter-image-text gap 본질 정정 (옵션 A)

## 개요

Stage 5 진단에 따라 옵션 A (WrapAnchorRef 확장 + paragraph_layout cs/sw 보정) 정정.

## 1. 정정 영역

### `src/renderer/pagination.rs`

`WrapAnchorRef` 에 `anchor_image_margin_right: i32` 필드 추가:

```rust
pub struct WrapAnchorRef {
    pub anchor_para_index: usize,
    pub anchor_cs: i32,
    pub anchor_sw: i32,
    /// [Task #722] anchor image 의 outer margin_right (HU)
    pub anchor_image_margin_right: i32,
}
```

PaginationResult shift 영역에 anchor_image_margin_right 전파.

### `src/renderer/typeset.rs`

WrapAnchorRef 구성 2 곳 (다음 paragraph register + anchor host self register) 에서 anchor image 의 `cm.margin.right` 추출 + register.

### `src/renderer/layout/paragraph_layout.rs`

wrap_anchor 처리에서 LINE_SEG cs px 에 +margin_right_px, sw px 에서 -margin_right_px 보정:

```rust
let (line_cs_offset, line_avail_w_override) = if let Some(anchor) = wrap_anchor {
    let seg = para.and_then(|p| p.line_segs.get(line_idx));
    let cs = seg.map(|s| s.column_start as i32).unwrap_or(0);
    let sw = seg.map(|s| s.segment_width as i32).unwrap_or(0);
    let mr = anchor.anchor_image_margin_right;
    let cs_px = crate::renderer::hwpunit_to_px(cs + mr, self.dpi);
    let sw_px = if sw > 0 {
        Some(crate::renderer::hwpunit_to_px((sw - mr).max(0), self.dpi))
    } else { None };
    (cs_px, sw_px)
} else {
    (0.0, None)
};
```

## 2. 위치 시프트 검증

| paragraph | Stage 4 "￼" x | Stage 6 "￼" x | 차이 (px) | 차이 (mm) |
|-----------|----------------|----------------|----------|-----------|
| 175 ("아래에...") | 384.16 | **395.52** | +11.36 | 3.0 |
| 1394 ("접근 제어") | 286.51 | **297.87** | +11.36 | 3.0 |

image margin_right = 852 HU = 3.0mm = 11.36 px (96 dpi). 두 paragraph 모두 정확한 보정량 시프트. PDF 정합.

## 3. 시각 판정 (rsvg-convert PNG)

- **stage6_p8.png** — paragraph 175 image 우측 inter-gap 추가 → PDF 정합
- **stage6_p27.png** — paragraph 779 caption 영역 보존 (caption-style 무영향)
- **stage6_p48.png** — paragraph 1394 image 우측 inter-gap 추가 → PDF 정합

## 4. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |

## 5. 회귀 위험 영역 확인

- WrapAnchorRef 데이터 모델 확장 (필드 1개 추가)
- typeset.rs 2 곳 + paragraph_layout.rs 1 곳 변경 (모두 wrap_anchor 발현 영역)
- IR 무수정
- wrap=Square 그림 미포함 paragraph 무영향
- Task #604 영역 보존

## 6. Stage 7 진행 승인 요청

광범위 페이지네이션 sweep + 최종 검증 진행 승인 요청.
