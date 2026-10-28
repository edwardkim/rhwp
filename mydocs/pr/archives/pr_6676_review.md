---
kind: pr-review
pr: 6676
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6676 review - first paragraph line body-bottom fit

## 판정: 승인

`#6568`의 첫 문단 line이 body bottom을 넘을 때 다음 페이지로 넘기는 layout 보정을 최신 `upstream/devel` 통합 후보에 `-x`로 적용했다. source head는 `74275da273a4c58beb0a66604203bc054b390089`이며, 같은 source head의 최종 full-lane CI 성공을 확인했고 reviewer `jangster77`을 사전 지정했다.

## 검토 및 검증

- `samples/issue6542/156678235_mid_para_vpos_rewind.hwp` regression test를 포함한 전체 integration test가 성공했다.
- 공통 검증은 fmt, all-features clippy, release-test integration suite, WASM web release build까지 모두 성공했다.
- 2020 Hancom 기준 PDF `pdf/issue6542-156678235-mid-para-vpos-rewind-2020.pdf`와 7/7쪽을 visual sweep으로 비교했다. 구조 경고 페이지는 없고, 문단/본문 외곽 overflow는 탐지되지 않았다.

## 시각 증적

- [PR #6676/#6678 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6676_6678_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

all-features 검증을 막던 `vello`/`vello_svg` Scene version 불일치는 통합 후보의 `Cargo.toml`/`Cargo.lock`에서 0.9 계열로 정렬했다. 이 보정은 본 문단 배치 변경과 독립적이다.

## Merge 후 contributor PR comment 계획

통합 merge와 `devel` CI 성공 뒤에만 source PR에 통합 merge SHA, 실제 CI, stable PNG/visual record 링크와 체리픽 통합 수용 사실을 한 번 기록한다.
