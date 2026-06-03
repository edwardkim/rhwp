# Task #722 Stage 4 단계별 보고서 — 페이지 27 본질 정정 (LINE_SEG ≥ 2 case 가드)

## 개요

Stage 3 진단 결과에 따라 정정안 E 의 anchor host self register 분기에 case 가드 추가.

## 1. 정정 — `src/renderer/typeset.rs:687~710`

```rust
if has_non_tac_pic_square {
    let anchor_cs = ...;
    let anchor_sw = ...;
    if anchor_cs > 0 || anchor_sw > 0 {
        st.wrap_around_cs = anchor_cs;
        st.wrap_around_sw = anchor_sw;
        st.wrap_around_table_para = para_idx;
        st.wrap_around_any_seg = true;
        // [Task #722] anchor host paragraph 자체도 wrap_anchors 등록.
        // Case 가드 (Stage 3 진단): LINE_SEG 갯수 ≥ 2 일 때만 자기 등록.
        // - LINE_SEG 1 → caption-style (image 위 자유 영역, col_area 전체 폭)
        // - LINE_SEG 2+ → multi-line wrap zone (image 우측, cs/sw 적용)
        if para.line_segs.len() >= 2 {
            st.current_column_wrap_anchors.insert(
                para_idx,
                crate::renderer::pagination::WrapAnchorRef {
                    anchor_para_index: para_idx,
                    anchor_cs,
                    anchor_sw,
                },
            );
        }
    }
}
```

추가 1줄 (가드 if 분기), 기존 register 한 줄 들여쓰기.

## 2. 시각 판정 (rsvg-convert PNG)

### 페이지 8 (paragraph 175 LINE_SEG 2)

- **stage4_p8.png** — "아래에 디렉토리 트리 각 부분의 역할에 대하여 설명하였 다." 가 image 우측 wrap zone 첫 줄 ✓
- Stage 2 정정 보존 (case 가드 LINE_SEG ≥ 2 통과)
- PDF 권위 자료 정합

### 페이지 27 (paragraph 779 LINE_SEG 1)

- **stage4_p27.png** — "Figure 4-4. 마운트된 /home과 /usr." 가 image 위 자유 영역 좌측 정렬 ✓
- Stage 4 정정 발현 (case 가드 LINE_SEG 1 → 자기 미등록 → col_area 전체 폭)
- "마운트는 다음과 같이 행해질 수 있다. ..." paragraph 781 가 image 우측 wrap zone (정합 보존)
- PDF 권위 자료 정합

## 3. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |

## 4. 회귀 위험 영역 확인

- typeset.rs 단일 분기 (case 가드 한 줄 추가)
- IR 무수정
- LINE_SEG 1개 이고 자체 wrap=Square 그림 anchor host paragraph 만 미등록 (caption 처리)
- 다른 paragraph 영향 없음

## 5. Stage 5 진행 승인 요청

광범위 페이지네이션 sweep + 최종 검증 진행 승인 요청.

---

## 6. Stage 4 갱신 (페이지 48 결함 발견 후, 2026-05-08)

페이지 48 paragraph 1394 ("접근 제어") 가 baseline 에도 있던 사전 결함. 한컴 정합 = image 우측 wrap zone (LINE_SEG 1 이지만 image 위 caption 영역 없음 → wrap 강제).

기존 LINE_SEG ≥ 2 만의 가드 부족. **caption 영역 존재 여부** 가드 추가.

### 갱신된 정정 — `src/renderer/typeset.rs:687~728`

```rust
let body_top_hu = page_def.margin_top as i32;
let line_height_hu = para.line_segs.first()
    .map(|s| s.line_height as i32).unwrap_or(900);
let image_voff_hu = para.controls.iter().find_map(|c| {
    let cm = match c {
        Control::Picture(p) => Some(&p.common),
        Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() {
            Some(&p.common)
        } else { None },
        _ => None,
    };
    cm.filter(|cm| !cm.treat_as_char
        && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
        .map(|cm| cm.vertical_offset as i32)
}).unwrap_or(0);
let caption_room_hu = image_voff_hu - body_top_hu;
let is_caption_style = para.line_segs.len() == 1
    && caption_room_hu > line_height_hu;
if !is_caption_style {
    st.current_column_wrap_anchors.insert(...);
}
```

### 3 paragraph 정합 검증

| paragraph | LINE_SEG | image vert_offset | caption_room | is_caption_style | 결과 | PDF 정합 |
|-----------|----------|-------------------|--------------|------------------|------|---------|
| 175 (페이지 8) | 2 | 18680 | 13000 | false (LINE_SEG ≥ 2) | wrap zone | ✓ |
| 779 (페이지 27) | 1 | 15400 | 9720 | true (room > 900) | caption | ✓ |
| 1394 (페이지 48) | 1 | 5668 | -12 | false (room ≤ 900) | wrap zone | ✓ |

### 시각 판정 (rsvg-convert PNG)

- **v2_p8.png**: paragraph 175 wrap zone (보존) ✓
- **v2_p27.png**: paragraph 779 image 위 caption (보존) ✓
- **v2_p48.png**: paragraph 1394 image 우측 wrap zone (회복) ✓

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
