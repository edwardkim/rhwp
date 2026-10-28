---
kind: pr-review
pr: 6678
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6678 review - centered cell leading

## 판정: 승인

`#6569`의 centered table-cell lead 처리와 golden SVG 보정을 최신 `upstream/devel` 통합 후보에 `-x`로 적용했다. source head는 `1b8e93762d77b7da83c9b73af4f1d3bd00d6c2d8`이고 reviewer `jangster77`을 사전 지정했다.

## 검토 및 검증

- 원 PR의 renderer/layout 변경, focused regression test, `tests/golden_svg/issue-617/exam-kor-page5.svg`를 포함했다.
- 공통 fmt, all-features clippy, release-test integration suite, WASM web release build가 성공했다.
- #6676과 같은 2020 기준 PDF 및 7쪽 sample을 사용한 visual sweep에서 구조 경고 페이지는 없었다. 이 visual scope는 cell lead로 인한 text-flow/frame overflow 회귀를 확인하며, 픽셀 동일성을 주장하지 않는다.

## 시각 증적

- [PR #6676/#6678 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6676_6678_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

`vello`/`vello_svg` Scene version 불일치를 통합 후보에서 0.9 계열로 정렬해 all-features 검증을 가능하게 했다. 본 cell-leading 변경의 동작을 바꾸지 않는 의존성 보정이다.

## Merge 후 contributor PR comment 계획

통합 merge 및 `devel` CI 성공 뒤에만 source PR에 실제 merge SHA, CI, stable PNG/visual record와 체리픽 수용 사실을 한 번 기록한다.
