# Task #1728 (부분) 최종 보고서 — 자동 쪽번호 세로 위치 margin_footer 정합

## 요약
RowBreak 표 continuation 페이지 시각 차이(#1728) 중 **footer 쪽번호 세로 위치** 갈래를 수정.
`footer_page_number_y` 가 `margin_bottom/2` 대신 `margin_footer/2` 를 쓰도록 정정 —
giant cell 18px→1px, KTX 7px→3px, aift 보존.

## 원인 (렌더 측정 + footer_area 계산 규명)
- 자동 쪽번호는 `build_page_number`→`page_number_baseline_y`(BodyBased 테두리)→`footer_page_number_y`
  (footer_area 세로중앙) 경로. gc/ktx/aift 모두 이 center 공식 사용.
- `footer_area`(`model/page.rs`): `[body_bottom, page_height − margin_footer]`, **height = margin_bottom**.
- 종전 center = `fa.y + fa.h/2` = `body_bottom + margin_bottom/2`.
- HWP 실측: glyph = `body_bottom + margin_footer/2 + ~10px`. 즉 **margin_bottom 이 아니라 margin_footer 기준**.
  - aift(margin_footer==margin_bottom): 우연히 정합(2px)
  - giant cell(margin_footer=0): 18px 낮음
  - KTX(margin_footer≠margin_bottom): 7px 낮음
- `margin_footer = page_height − footer_area.bottom` 로 유도 가능.

## 수정
`src/renderer/layout.rs` `footer_page_number_y`:
```rust
// before: footer_area.y + footer_area.height / 2.0   (= body_bottom + margin_bottom/2)
let margin_footer = (layout.page_height - (footer_area.y + footer_area.height)).max(0.0);
footer_area.y + margin_footer / 2.0
```

## 검증 (96 DPI PNG glyph mid vs 한글 2024 PDF)
| 문서 | 수정 전 | 수정 후 | PDF | 차이 |
|------|--------|--------|-----|------|
| giant cell(#1718) | 1113 | **1094** | 1095 | **−1px** |
| KTX(#874) | 1066 | **1056** | 1059 | −3px |
| aift(#634) | 1075 | 1075 | 1077 | −2px (불변) |

- `cargo test --lib`: **2044 passed / 0 failed** (Task #634 aift/hwp3-sample 쪽번호 표시 테스트 포함).
- aift 는 margin_footer==margin_bottom 이라 수치 불변 → #634 하드코딩 y 테스트 무회귀.

## 범위 / 한계
- #1728 의 footer **세로 위치** 갈래만 수정(가장 일관·안전). 남은 갈래(continuation 상단 space-before
  트림, PDF 하단 rule 라인)는 별도 후속.
- KTX −3px 는 render baseline→glyph 미세 오프셋(폰트 메트릭) 잔차. 시각 영향 경미.
