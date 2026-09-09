---
kind: pr-review
pr: 6688
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6688 review - overlapping line boxes do not reset flow

## 판정: 승인

`#5585` axis 1의 overlapping line-boxes 처리 변경을 최신 `upstream/devel` 통합 후보에 `-x`로 적용했다. source head는 `4abdec13ba012568c6b87d5e4c884a1f15bca489`이며, #6682가 ancestor인 중복 commit은 한 번만 적용했다. reviewer `jangster77`을 사전 지정했다.

## 검토 및 검증

- contributor corpus `148738070_20120829_무학대선건 보도자료.hwp`를 `RHWP_ISSUE5585_SAMPLE`로 설정해 regression test를 포함한 전체 integration suite를 실행했다.
- 2020 Hancom 기준 PDF `pdf/issue5585-overlap-line-boxes-2020.pdf`와 7/7쪽 visual sweep에서 구조 경고 페이지는 없었다.
- 공통 fmt, all-features clippy, release-test integration suite, WASM web release build가 성공했다.

## 시각 증적

- [PR #6688 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6688_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

`vello`/`vello_svg` Scene version alignment은 all-features build contract만 보정하며, line-box flow 변경의 의미를 바꾸지 않는다.

## Merge 후 contributor PR comment 계획

통합 merge 및 `devel` CI 성공 뒤에만 source PR에 merge SHA, 실제 CI, stable PNG/visual record 링크 및 체리픽 수용 사실을 한 번 남긴다.
