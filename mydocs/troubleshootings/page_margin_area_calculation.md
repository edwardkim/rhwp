---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 편집 용지 영역 계산 오류 (페이지 여백)

## 증상

k-water-rfp.hwp 13페이지에서 본문 시작 위치가 HWP 프로그램보다 약 10.6mm(40px) 아래에서 시작됨. 한 페이지에 들어가야 할 두 개의 표 중 두 번째 표가 다음 페이지로 밀림.

- 수정 전: 29쪽
- 수정 후: 27쪽 (정상)

## 원인

`PageAreas::from_page_def()` (src/model/page.rs)에서 본문 시작/끝 위치를 잘못 계산.

### HWP 쪽여백 구조 (편집 용지 대화상자 기준)

```
용지 상단 (y=0)
  ↓ margin_header (머리말, 예: 10.6mm) → 머리말 영역 시작
  ↓ margin_top (위쪽, 예: 19.4mm)      → 본문 영역 시작
  ...본문...
  ↓ page_height - margin_bottom (아래쪽, 예: 14.8mm) → 본문 영역 끝
  ↓ page_height - margin_footer (꼬리말, 예: 10.0mm) → 꼬리말 영역 끝
용지 하단 (y=page_height)
```

핵심: **margin_top은 용지 상단에서 본문까지의 거리**이며, margin_header는 용지 상단에서 머리말까지의 거리이다. 두 값은 독립적이고 합산하면 안 된다.

### 잘못된 코드

```rust
// 본문 시작 = margin_top + margin_header (잘못!)
let content_top = page_def.margin_top + page_def.margin_header;
// 본문 끝 = page_height - margin_bottom - margin_footer (잘못!)
let content_bottom = page_height - page_def.margin_bottom - page_def.margin_footer;

// 머리말 영역 (잘못된 위치)
let header_area = Rect {
    top: page_def.margin_top,      // margin_header여야 함
    bottom: content_top,            // margin_top이어야 함
};

// 꼬리말 영역 (잘못된 위치)
let footer_area = Rect {
    top: content_bottom,
    bottom: content_bottom + margin_footer,
};
```

### 올바른 코드

```rust
// 본문 시작 = margin_top (용지 상단에서 본문까지 거리)
let content_top = page_def.margin_top;
// 본문 끝 = page_height - margin_bottom
let content_bottom = page_height - page_def.margin_bottom;

// 머리말 영역: margin_header ~ margin_top
let header_area = Rect {
    top: page_def.margin_header,
    bottom: page_def.margin_top,
};

// 꼬리말 영역: (page_height - margin_bottom) ~ (page_height - margin_footer)
let footer_area = Rect {
    top: page_height - page_def.margin_bottom,
    bottom: page_height - page_def.margin_footer,
};
```

## k-water-rfp.hwp 편집 용지 설정 값

| 항목 | 값 | HWPUNIT (≈) |
|------|-----|-------------|
| 용지 크기 | A4 (210 x 297 mm) | 59528 x 84188 |
| 위쪽 (margin_top) | 19.4 mm | 5499 |
| 머리말 (margin_header) | 10.6 mm | 3005 |
| 왼쪽 (margin_left) | 21.2 mm | 6009 |
| 오른쪽 (margin_right) | 19.5 mm | 5527 |
| 아래쪽 (margin_bottom) | 14.8 mm | 4195 |
| 꼬리말 (margin_footer) | 10.0 mm | 2835 |
| 제본 (margin_gutter) | 0.0 mm | 0 |

## 오류 영향

| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| 본문 시작 (content_top) | margin_top + margin_header = 30.0mm | margin_top = 19.4mm |
| 본문 끝 (content_bottom) | page_height - 24.8mm | page_height - 14.8mm |
| 13페이지 첫 텍스트 y | 133.3px | 93.3px |
| 총 페이지 수 | 29쪽 | 27쪽 |
| 13페이지 표 2개 | 두 번째 표 다음 쪽으로 밀림 | 정상 포함 |

## 참고: 독립 문단의 measure_paragraph()

`measure_paragraph()`(height_measurer.rs)는 이미 `spacing_before + lines_total + spacing_after`로 올바르게 높이를 측정하고 있었음. 이번 이슈는 페이지 영역 계산 자체의 문제로, 셀 높이/문단 높이 측정과는 별개.

## 수정 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/model/page.rs` | `PageAreas::from_page_def()` content_top/bottom, header_area, footer_area 계산 수정 |
