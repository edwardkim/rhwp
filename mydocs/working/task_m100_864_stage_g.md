# Task #864 Stage G inline TAC picture 중복 emit 정정

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## 배경

Stage F 정정 (caption 후 result_y 진행) 후, 페이지 4 의 caption "Visual Block을 이용한 대소문자 변경" 과 다음 본문 "먼저 원하는 구간을..." 사이의 간격이 한컴보다 좁음.

추가 진단으로 본질을 발견: **inline (TAC) picture 가 두 곳에서 emit 되어 중복 그려짐**.

## D.5 추가 진단 — 중복 image emit

페이지 3, 4 의 outer SVG 분석:

| pic | Image #1 (top-aligned) | Image #2 (baseline-aligned) |
|---|---|---|
| page 3 pi=29 | y=132.27 (pic_y) | y=156.24 (pic_y + baseline - pic_h) |
| page 4 pi=49 | y=594.19 (pic_y) | y=622 (pic_y + baseline - pic_h) |

두 emit 위치:

1. **`src/renderer/layout.rs:2929`** (TAC inline branch): `BoundingBox::new(pic_x, pic_y, pic_w, pic_h)` — top-aligned at pic_y.
2. **`src/renderer/layout/paragraph_layout.rs:1859`** (paragraph inline image): `BoundingBox::new(x, img_y, tac_w, pic_h)` where `img_y = (y + baseline - pic_h).max(y)` — baseline-aligned.

`layout.rs` branch 의 `already_registered` 체크는 `tree.get_inline_shape_position` 로 등록된 위치를 확인하나, **`paragraph_layout` 가 picture 에 대해 `set_inline_shape_position` 을 호출하지 않음** (line 1888 의 등록은 Shape 만 대상). → `already_registered = false` 항상 → 두 곳 모두 emit.

결과:
- Image #1 (top-aligned) 가 caption 영역을 가림 (특히 line_height > pic_h 인 경우).
- Caption 이 두 image 사이 또는 Image #2 안쪽에 위치하여 보이지 않음.

## 정정

### G.1 paragraph_layout 의 picture 위치 등록

```rust
// src/renderer/layout/paragraph_layout.rs:1876+
line_node.children.push(img_node);
// [Task #864 Stage G] inline TAC picture 의 위치 등록.
tree.set_inline_shape_position(
    section_index, para_index, tac_ci,
    cell_ctx.as_ref(), x, img_y,
);
```

→ `layout.rs` 의 `already_registered` 체크 통과 → top-aligned 중복 emit 스킵.

### G.2 caption y 를 image_bottom 정합 (Stage E v2)

paragraph_layout 의 image 가 baseline-aligned 이므로 caption 도 그 image 의 bottom 위치 (= `pic_y + baseline_distance`) 에 위치시켜야 한컴 시각 정합.

```rust
// src/renderer/layout.rs:2982+
let baseline_px = para.line_segs.first()
    .map(|ls| hwpunit_to_px(ls.baseline_distance, self.dpi))
    .unwrap_or(pic_h);
let image_bottom = pic_y + baseline_px.max(pic_h);
let cap_y = match caption.direction {
    CaptionDirection::Bottom => image_bottom + caption_spacing,
    ...
};
```

기존 Stage E 의 `pic_y + line_height` 보다 작은 값 (baseline ≈ line_height × 0.85) → caption 이 image 바로 아래 자연 위치, body 와의 간격 확보.

## 검증

- cargo build --release: ✓
- cargo test --release --lib: 1230 passed (회귀 0)
- cargo clippy --release --lib: 경고 0

### 시각 정합

| 페이지 | 결함 | 정정 후 |
|---|---|---|
| page 2 | (Stage A-C 정정) | image 위, caption 아래 정합 ✓ |
| page 3 "Cut&Paste 할 영역" | image 안 가려짐 | image 아래, body 와 자연 간격 ✓ |
| page 4 "Visual Block을 이용한 대소문자 변경" | body 와 너무 좁음 | 한컴 정합 (적절 간격) ✓ |

### outer SVG image element 개수 (중복 제거 확인)

| 페이지 | 변경 전 | 변경 후 |
|---|---|---|
| page 3 | 4 (= 2 picture × 2 emit) | **2** (= 2 picture × 1 emit) |
| page 4 | 4 | **2** |

## 영향 범위

`paragraph_layout` 가 inline TAC picture 를 emit 하는 모든 케이스. 기존 layout.rs 의 TAC branch 가 중복 emit 하던 결함 정정. 모든 inline image (HWP3 / HWP5 / HWPX 공통) 에 적용.

## Stage G 결론

inline TAC picture 의 중복 emit 본질 (paragraph_layout 의 set_inline_shape_position 누락) 정정. caption 위치를 image bottom (baseline-aligned) 에 정합. HWP3 sample14 의 page 3, 4 모두 한컴 시각 정합 달성.

📋 **Stage G 완료. 종합 보고서 갱신 + 커밋 승인 진행합니다.**
