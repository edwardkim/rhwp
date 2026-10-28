---
kind: visual-sweep
scope: planet6897 layout integration
reviewed_at: 2026-09-03
---

# planet6897 PR #6675–#6692 visual sweep

## 실행 대상

- candidate branch: `review/planet6897-layout-batch-20260903`
- candidate binary: `target/pr-review/planet6897-layout-batch-20260903/release-test/rhwp`
- rasterizer: Chrome webfont path, 144 DPI
- reference conversion: HWP/HWPX metadata에 따라 Hancom `engine 2020`; 결과 PDF는 `pdf/`에 저장
- full local validation: fmt, all-features clippy, release-test integration suite, WASM web release build 성공

## stable evidence assets

- [#6675, issue1663, 2/2 pages](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6675_review_contact_sheet.png)
- [#6676/#6678, issue6542, 7/7 pages](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6676_6678_review_contact_sheet.png)
- [#6680, issue2004, pages 4–8](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6680_review_contact_sheet.png)
- [#6682, issue6599, 12/12 pages](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6682_review_contact_sheet.png)
- [#6688, issue5585 line boxes, 7/7 pages](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6688_review_contact_sheet.png)
- [#6692, issue5585 sibling tables, N-up mapping pages 42–43](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6692_nup_mapping_contact_sheet.png)

## 결과

| Source PR | Reference scope | Structural result | Interpretation |
| --- | --- | --- | --- |
| #6675 | 2/2 | flagged pages 0 | declared row-height/padding scope에서 frame overflow 없음 |
| #6676 | 7/7 | flagged pages 0 | first-paragraph body-bottom scope에서 structural flow collapse 없음 |
| #6678 | #6676과 같은 7/7 | flagged pages 0 | centered cell lead scope가 같은 sample에서 안정적 |
| #6680 | pages 4–8 | flagged pages 0 | reclassified float x-offset scope에서 frame overflow 없음 |
| #6682 | 12/12 | flagged pages 0 | nested-caption row units scope에서 structural warning 없음 |
| #6688 | 7/7 | flagged pages 0 | overlapping line boxes scope에서 structural warning 없음 |
| #6692 | physical pages 42–43 | N-up mapping mismatch | 43 physical PDF pages와 86 logical rhwp pages가 1:1이 아니므로 overlay diff를 pass/fail로 사용하지 않음 |

Pixel/ink match 수치는 renderer, font fallback, physical-page mapping의 영향을 받는 보조 관측값이다. 위 판정은 visual sweep의 structural signal, 관련 Rust regression tests, source review를 함께 사용한 것이며 pixel-perfect equivalence 주장이 아니다.

## 재현 산출물

전체 SVG, render tree, PDF PNG, overlay, page analysis JSON은 검토 전용 `target/pr-review/planet6897-layout-batch-20260903/visual-sweep-raw/`에 보존한다. PR에는 재검토에 필요한 stable contact sheet만 포함한다.
