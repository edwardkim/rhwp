---
kind: pr-review
pr: 6692
source: planet6897
reviewed_at: 2026-09-03
---

# PR #6692 review - sibling tables do not recharge anchor offset

## 판정: 승인

`#5585` axis 2의 sibling tables anchor-offset 보정을 최신 `upstream/devel` 통합 후보에 `-x`로 적용했다. source head는 `c21a6fd25307368b24eb02b22cd3882ecf3eb1ab`이며 reviewer `jangster77`을 사전 지정했다.

## 검토 및 검증

- contributor corpus `1351000-201000123_D0150004-2-002_02. 지표정의서- 주요정책부문.hwp`를 `RHWP_ISSUE5585B_SAMPLE`로 설정해 regression test를 포함한 전체 integration suite를 실행했다.
- 공통 fmt, all-features clippy, release-test integration suite, WASM web release build가 성공했다.

## 시각 증적과 한계

- 2020 Hancom PDF `pdf/issue5585-sibling-tables-2020.pdf`는 43 physical pages이고 후보 SVG/render tree는 86 logical pages다.
- 42–43쪽 review contact sheet는 이 N-up physical-page 구조를 직접 보존한다. 따라서 기본 page-number overlay의 `content_bottom_drift` 2건은 1:1 page 대응이 성립하지 않아 pixel pass/fail 근거로 사용하지 않는다.
- 이 제한은 sibling-table regression test의 성공과 구분한다. source 변경의 검증은 Rust regression contract로, visual artifact는 N-up 기준 자료의 물리 구조와 비교 불가 범위를 명시하는 증적으로 사용한다.

## 시각 증적

- [PR #6692 N-up mapping contact sheet](../assets/pr_6675_6692_planet6897_integration_20260903/stable/pr_6692_nup_mapping_contact_sheet.png)
- [batch visual sweep record](pr_6675_6692_planet6897_visual_sweep.md)

## 공통 메인터너 보정

통합 all-features 검증을 위해 `vello`/`vello_svg` Scene version을 0.9 계열로 정렬했다. sibling-table layout logic에는 변경이 없다.

## Merge 후 contributor PR comment 계획

통합 merge 및 `devel` CI 성공 뒤에만 source PR에 merge SHA, 실제 CI, stable PNG/visual record 링크, N-up visual scope의 한계와 체리픽 수용 사실을 한 번 남긴다.
