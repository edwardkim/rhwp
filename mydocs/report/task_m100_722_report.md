# Task #722 최종 결과 보고서

## 1. 이슈 요약

**Issue #722**: hwp3-sample5.hwp wrap=Square 그림 paragraph 시각 정합 결함

**증상**:
- 페이지 8 paragraph 175 ("아래에 디렉토리 트리 각 부분의 역할에 대하여 설명하였다.") — image 영역 침범, image z-order 후 그려져 텍스트 가려짐
- 페이지 27 paragraph 779 ("Figure 4-4. 마운트된 /home과 /usr.") — Stage 4 후 image 위 caption 영역 회귀 (정정안 E LINE_SEG 갯수 가드 부족)
- 페이지 48 paragraph 1394 ("접근 제어") — image 영역 침범, image 가 텍스트 가려짐
- 모든 paragraph: inter-image-text gap (image outer margin 3mm) 미적용

## 2. 본질 정정 영역

### `src/renderer/typeset.rs:519~539` — anchor 다음 paragraph register

paragraph_layout 의 wrap_anchor 처리에서 LINE_SEG cs 보정용 image margin_right 추출 + register.

```rust
let anchor_margin_right = paragraphs.get(st.wrap_around_table_para)
    .and_then(|p| p.controls.iter().find_map(|c| {
        // wrap=Square Picture 의 cm.margin.right 추출
    })).unwrap_or(0);
st.current_column_wrap_anchors.insert(
    para_idx,
    crate::renderer::pagination::WrapAnchorRef {
        anchor_para_index: st.wrap_around_table_para,
        anchor_cs: st.wrap_around_cs,
        anchor_sw: st.wrap_around_sw,
        anchor_image_margin_right: anchor_margin_right,
    },
);
```

### `src/renderer/typeset.rs:687~754` — anchor host self register

paragraph 175 (LINE_SEG ≥ 2) / paragraph 779 (LINE_SEG 1, caption_room>line_height → caption-style → 미등록) / paragraph 1394 (LINE_SEG 1, caption_room≤line_height → 등록) case 가드.

```rust
let body_top_hu = page_def.margin_top as i32;
let line_height_hu = para.line_segs.first()
    .map(|s| s.line_height as i32).unwrap_or(900);
let (image_voff_hu, image_margin_right_hu) = ... ;
let caption_room_hu = image_voff_hu - body_top_hu;
let is_caption_style = para.line_segs.len() == 1
    && caption_room_hu > line_height_hu;
if !is_caption_style {
    st.current_column_wrap_anchors.insert(
        para_idx,
        crate::renderer::pagination::WrapAnchorRef {
            anchor_para_index: para_idx,
            anchor_cs,
            anchor_sw,
            anchor_image_margin_right: image_margin_right_hu,
        },
    );
}
```

### `src/renderer/pagination.rs:154~165` — WrapAnchorRef 데이터 모델 확장

`anchor_image_margin_right: i32` 필드 추가. PaginationResult shift 영역 전파.

### `src/renderer/layout/paragraph_layout.rs:915~927` — wrap_anchor 처리 cs/sw 보정

LINE_SEG cs px 에 +margin_right_px, sw px 에서 -margin_right_px 보정 (inter-image-text gap 정합).

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

## 3. 수행 단계 요약

| Stage | 영역 | 결과 |
|-------|------|------|
| 1 | 본질 진단 — wrap_anchor 매칭 분기 식별 | 옵션 A 결정 |
| 2 | anchor host self register (정정안 E) | 페이지 8 정합 |
| 3 | 페이지 27 진단 — LINE_SEG 갯수 가드 도출 | LINE_SEG ≥ 2 가드 |
| 4 | LINE_SEG ≥ 2 + caption_room 가드 추가 | 페이지 27/48 정합 |
| 5 | inter-image-text gap 진단 — image margin 발견 (852 HU = 3mm) | 옵션 A (WrapAnchorRef 확장) |
| 6 | image margin_right 추출 + paragraph_layout cs/sw 보정 | 페이지 8/27/48 미세 차이 정합 |
| 7 | 광범위 sweep + 결정적 검증 | 209 fixture DIFF 0 |
| 8~9 | HWP5 변환본 paragraph 441 시도 | 페이지 분할 왜곡 회귀 → rollback, 별도 issue 분리 |

## 4. 검증 결과

### 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** |
| `cargo test --release` 전체 binary tests | 전체 GREEN |
| `cargo clippy --release` | 신규 경고 0 |

### 광범위 페이지네이션 sweep

209 fixture (samples/ 전체) 페이지 수 차이 **DIFF 0** — 회귀 0.

### 시각 판정 (rsvg-convert PNG, PDF 권위 자료 정합)

| 페이지 | paragraph | LINE_SEG | image | 결과 | PDF 정합 |
|--------|-----------|----------|-------|------|---------|
| 8 | 175 | 2 | y_voff=18680, mr=852 | image 우측 wrap zone + gap 3mm | ✓ |
| 27 | 779 | 1, room=9720 | y_voff=15400, mr=852 | image 위 자유 영역 좌측 정렬 | ✓ |
| 48 | 1394 | 1, room=-12 | y_voff=5668, mr=852 | image 우측 wrap zone + gap 3mm | ✓ |

## 5. 회귀 위험 영역 정합

- typeset.rs 2 곳 (다음 paragraph register + anchor host self register) 변경
- pagination.rs WrapAnchorRef 데이터 모델 1 필드 추가
- paragraph_layout.rs wrap_anchor 처리 1 곳 변경 (cs/sw 보정)
- IR 무수정 (HWP3 파서 / Document model 무영향)
- Task #604 영역 보존
- 209 fixture 페이지 수 차이 0

## 6. 후속 task

HWP5 변환본 (hwp3-sample5-hwp5.hwp) paragraph 441 wrap zone 매칭 결함은 별도 issue 로 분리.

본질: anchor host paragraph (cs=0, sw=body_w 인 caption-style) 와 다음 paragraph (cs>0 wrap zone 인코딩) 의 매칭 가드 부재. Stage 8~9 시도 시 가드가 broad 매칭 (paragraph 442/443 까지) → 페이지 분할 왜곡. 본질 정정에는 image 위치/폭/margin 으로 expected_cs 정확 계산 + 첫 wrap text paragraph 만 매칭하는 알고리즘 영역 필요.

## 7. closes #722

- hwp3-sample5.hwp 페이지 8/27/48 wrap=Square 그림 paragraph 시각 정합 회복
- 한컴 PDF 권위 자료 정합 검증
- 광범위 fixture 회귀 0
