---
kind: pr-review
pr: 6680
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6680 review - reclassified float inline offset

## 판정: 메인터너 보정 됨 수용 가능

`#4068`의 reclassified floating picture x-offset 보정은 최신 `upstream/devel`과 충돌했다. source head `af0f7bc051d73a4c478995af512cb9661c3c0f9e`의 기능 변경은 통합했고 reviewer `jangster77`을 사전 지정했다.

## 메인터너 conflict 보정

- 최신 `devel`에는 normalizer가 남긴 `TAG_IMPLEMENTATION_PROPERTY`를 쓰는 canonical `inline_para_horizontal_offset_px`가 이미 존재했다.
- contributor의 1-control/1-line heuristic helper를 함께 유지하면 x-offset이 중복 적용될 수 있어, current `devel`의 tag-based 구현을 유지하고 contributor regression test/evidence를 통합했다.
- 이후 source follow-up `af0f7bc…`는 제거된 duplicate helper의 문서화만 대상으로 해 `git cherry-pick --skip`했다. 기능 누락이 아니라 conflict 해소 후 대상이 사라진 경우다.

## 검토 및 검증

- `samples/issue2004_cell_image_stack.hwp` 4–8쪽 regression scope를 포함한 전체 integration test가 성공했다.
- 2020 Hancom PDF `pdf/issue2004_cell_image_stack-hwp-2020.pdf`와 4–8쪽 visual sweep에서 구조 경고는 없었다.
- 공통 fmt, all-features clippy, release-test integration suite, WASM web release build가 성공했다.

## 시각 증적

- [PR #6680 review contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6680_review_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

all-features 검증을 막던 `vello 0.10`과 `vello_svg`의 `vello 0.9` Scene type 불일치는 통합 후보에서 0.9 계열로 정렬했다. 이는 #6680의 layout conflict 보정과 별개인 build-contract 보정이다.

## Merge 후 contributor PR comment 계획

통합 merge와 `devel` CI 성공 뒤에만 source PR에 canonical conflict resolution, 실제 merge SHA/CI, stable PNG/visual record 링크와 체리픽 수용 사실을 한 번 기록한다.
