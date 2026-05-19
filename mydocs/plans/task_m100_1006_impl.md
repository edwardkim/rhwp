# Task #1006 구현 계획서

이슈: [#1006](https://github.com/edwardkim/rhwp/issues/1006)
수행 계획서: [`task_m100_1006.md`](task_m100_1006.md)

## Stage 1 — 모델/파서 contract 분리

### 1-1. `PageBorderFill` 모델 확장

```rust
// src/model/page.rs
#[derive(Debug, Clone, Default)]
pub struct PageBorderFill {
    pub attr: u32,
    pub spacing_left: HwpUnit16,
    // ... (기존)
    pub border_fill_id: u16,
    pub basis: PageBorderBasis, // [Task #1006] 신규
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PageBorderBasis {
    #[default]
    BodyBased,
    PaperBased,
}
```

### 1-2. 파서 3곳 basis 주입

- `src/parser/hwp3/mod.rs::2828` — `basis: PageBorderBasis::BodyBased`
- `src/parser/body_text.rs::parse_page_border_fill` (HWP5) — `pbf.basis = PaperBased`
- `src/parser/hwpx/section.rs::parse_page_border_fill_empty` (HWPX) — `page_border_fill.basis = PaperBased`

## Stage 2 — 렌더러 분기 정정 + 변환본 cover logo 보호

### 2-1. `build_page_borders` 의 attr bit 0 분기 제거

```rust
// Before:
let paper_based = (pbf.attr & 0x01) != 0;

// After:
use crate::model::page::PageBorderBasis;
let paper_based = matches!(pbf.basis, PageBorderBasis::PaperBased);
```

footer baseline 산출 (`page_footer_baseline_y`) 도 동일 변경.

### 2-2. 변환본 cover header logo 감지 + outline top clip skip

```rust
if !header_inside {
    let has_header_obj =
        self.detect_header_object_bottom(page_content, paragraphs, layout) > 0.0;
    if !has_header_obj {
        let header_bottom = layout.body_area.y;
        if by < header_bottom {
            bh -= header_bottom - by;
            by = header_bottom;
        }
    }
    // 머리말 logo 존재 시 clip 미적용 — outline 이 paper-spacing edge 까지 확장
}
```

신규 helper:

```rust
fn detect_header_object_bottom(
    &self,
    page_content: &PageContent,
    paragraphs: &[Paragraph],
    layout: &PageLayoutInfo,
) -> f64 {
    // 현재 페이지의 paragraph 인덱스 → controls 의 Picture/Shape 중
    // vert_rel_to == VertRelTo::Paper && top_px < body_top → max bottom 반환
}
```

## Stage 3 — 회귀 검증

검증 fixture:
- `samples/hwp3-sample16.hwp` (HWP3, BodyBased) — 외곽선 body-based 정합 (#987 baseline)
- `samples/hwp3-sample16-hwp5.hwp` (HWP5 변환본, PaperBased) — cover logo overlap 해소
- `samples/3-09월_교육_통합_2022.hwp` (HWP5, PaperBased) — paper-based 정합 (#956 baseline)
- `samples/3-09월_교육_통합_2022.hwpx` (HWPX, PaperBased) — paper-based 정합

회귀 sweep:
- hwp3-sample10, hwp3-sample14, exam_kor, aift, biz_plan — 페이지 수 보존

빌드/테스트:
- `cargo build --release`
- `cargo test --release --lib`
- `cargo clippy --release -- -D warnings`

## Stage 4 — 최종 보고서 + PR

- 최종 결과 보고서 작성
- PR 생성 (`closes #1006`)
- WASM 빌드

## 결정 사항

- `PageBorderFill::basis` default 는 `BodyBased` — struct default 시 안전 fallback (HWP3 와 동일 동작)
- attr 필드는 유지 — bit 1/2 (header_inside/footer_inside) clip 처리에 여전히 사용
- `detect_header_object_bottom` 의 반환값 > 0 만 boolean trigger 로 사용 — 향후 micro-tuning (실제 push-down) 으로 확장 가능
