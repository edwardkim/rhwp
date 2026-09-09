---
kind: pr-review
pr: 6682
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6682 review - nested caption row-height units

## 판정: 승인

`#6599`의 nested caption이 table-row height units에 포함되도록 하는 변경을 최신 `upstream/devel` 통합 후보에 `-x`로 적용했다. source head는 `7c555c51a0a162185d5bb74a1fbd8c1108a9b813`이며 reviewer `jangster77`을 사전 지정했다.

## 검토 및 검증

- contributor가 지정한 2020 HWP corpus `2181727_[별표 1의2] 프레스 또는 전단기 방호장치의 시험방법…hwp`를 `RHWP_ISSUE6599_SAMPLE`로 설정해 regression test를 포함한 전체 integration suite를 실행했다.
- 2020 Hancom 기준 PDF `pdf/issue6599-press-shear-guard-2020.pdf`의 12/12쪽 visual sweep은 구조 경고 페이지 0건이었다.
- 공통 fmt, all-features clippy, release-test integration suite, WASM web release build가 성공했다.

## 시각 증적

- [PR #6682 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6682_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

통합 all-features build를 가능하게 하는 `vello`/`vello_svg` 0.9 alignment만 적용했으며, nested-caption layout 동작은 변경하지 않았다.

## Merge 후 contributor PR comment 계획

통합 merge 및 `devel` CI 성공 후 source PR에 merge SHA, 실제 CI, stable PNG/visual record와 체리픽 수용 사실을 한 번 남긴다.
