# 새 base 기능의 wasm 실제 출력 보존

**14입력·41쪽**을 실제 f529 base 제품과 ba2 후보 제품으로 같은 호스트·독립 Print·글꼴 환경·96 DPI·Print profile에서 대조했다.
양쪽 모두 전쪽 raw 90% 이상이며, 후보의 0.1%p 초과 하락·쪽수 불일치·누락·입력/PDF/product 해시 오류는 0건이다.
완료한 own scope 10입력·43쪽의 직전 후보 대비 대조도 같은 감사에서 별도로 확인했다.

현재 base `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0`는 후속 문서와 이슈 자동 라벨링 경로만 추가됐고 f529와 렌더 제품 내용이 같다.
[문서 전용 리베이스의 내용 동일성](../latest-rebase.md)에 따라 아래 실제 제품과 증거를 현재 base 판정에 재사용한다.
평균값으로 미달을 상쇄하거나 새 head에서 다시 출력한 것으로 표시하지 않는다.

- base 실제 source `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
  rhwp.js SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
  rhwp_bg.wasm SHA-256 `d8e5619384e3963609867ae07b8eacd75f053dfee5c43b85d9102ab168ff9ada`.
- candidate 실제 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`.
  rhwp.js SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
  rhwp_bg.wasm SHA-256 `3a9125c0a3fa65e1ebbe45abfd0e2070248fbb96d1f5e083d6e3108baa62ed61`.

| 반영 PR · 같은 원본 · 독립 Print · compat | 전쪽 | 실제 base TSV · 정밀 최저 raw | 실제 후보 TSV · 정밀 최저 raw |
| --- | --- | --- | --- |
| [PR #8347](https://github.com/edwardkim/rhwp/pull/8347) · [equal-height-partial-block.hwp](<../../../../../tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp>) · [equal-height-partial-block-hancom-mac-12.30.pdf](<../../../../../pdf/rowspan_staggered_heights/equal-height-partial-block-hancom-mac-12.30.pdf>) · 2022 | 7 | [TSV](<base/wasm/pr8347-equal-height-partial-block-461139ac.tsv>) · 99.98625% | [TSV](<candidate/wasm/pr8347-equal-height-partial-block-461139ac.tsv>) · 99.98625% |
| [PR #8358](https://github.com/edwardkim/rhwp/pull/8358) · [start-number-10.hwp](<../../../../../tests/fixtures/section_start_page_number/start-number-10.hwp>) · [start-number-10-hancom-mac-12.30.pdf](<../../../../../pdf/section_start_page_number/start-number-10-hancom-mac-12.30.pdf>) · 2022 | 4 | [TSV](<base/wasm/pr8358-start-number-10-9bcd1152.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8358-start-number-10-9bcd1152.tsv>) · 100.0% |
| [PR #8359](https://github.com/edwardkim/rhwp/pull/8359) · [tab-spacing-16000.hwp](<../../../../../tests/fixtures/section_default_tab_spacing/tab-spacing-16000.hwp>) · [tab-spacing-16000-hancom-mac-12.30.pdf](<../../../../../pdf/section_default_tab_spacing/tab-spacing-16000-hancom-mac-12.30.pdf>) · 2022 | 1 | [TSV](<base/wasm/pr8359-tab-spacing-16000-06bc6a05.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8359-tab-spacing-16000-06bc6a05.tsv>) · 100.0% |
| [PR #8363](https://github.com/edwardkim/rhwp/pull/8363) · [diagonal-shape-codes.hwp](<../../../../../tests/fixtures/cell_diagonal_shapes/diagonal-shape-codes.hwp>) · [diagonal-shape-codes-hancom-mac-12.30.pdf](<../../../../../pdf/cell_diagonal_shapes/diagonal-shape-codes-hancom-mac-12.30.pdf>) · 2022 | 1 | [TSV](<base/wasm/pr8363-diagonal-shape-codes-ae90d6aa.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8363-diagonal-shape-codes-ae90d6aa.tsv>) · 100.0% |
| [PR #8366](https://github.com/edwardkim/rhwp/pull/8366) · [borders.hwp](<../../../../../tests/fixtures/border_line_types/borders.hwp>) · [borders-hancom-mac-12.30.pdf](<../../../../../pdf/border_line_types/borders-hancom-mac-12.30.pdf>) · 2022 | 1 | [TSV](<base/wasm/pr8366-borders-ca2c2b9e.tsv>) · 96.79456% | [TSV](<candidate/wasm/pr8366-borders-ca2c2b9e.tsv>) · 96.79456% |
| [PR #8367](https://github.com/edwardkim/rhwp/pull/8367) · [header-numbering.hwp](<../../../../../tests/fixtures/header_footer_numbering/header-numbering.hwp>) · [header-numbering-hancom-mac-12.30.pdf](<../../../../../pdf/header_footer_numbering/header-numbering-hancom-mac-12.30.pdf>) · 2022 | 1 | [TSV](<base/wasm/pr8367-header-numbering-a3f5b197.tsv>) · 94.80661% | [TSV](<candidate/wasm/pr8367-header-numbering-a3f5b197.tsv>) · 94.80661% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [body-emphasis-36pt.hwp](<../../raised-decoration-hancom/body-emphasis-36pt.hwp>) · [body-emphasis-36pt-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/body-emphasis-36pt-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-body-emphasis-36pt-15fcd052.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8349-body-emphasis-36pt-15fcd052.tsv>) · 100.0% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [body-top-36pt.hwp](<../../raised-decoration-hancom/body-top-36pt.hwp>) · [body-top-36pt-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/body-top-36pt-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-body-top-36pt-ab79b1c0.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8349-body-top-36pt-ab79b1c0.tsv>) · 100.0% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [cell-top-10-36pt.hwp](<../../raised-decoration-hancom/cell-top-10-36pt.hwp>) · [cell-top-10-36pt-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/cell-top-10-36pt-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-cell-top-10-36pt-7a459180.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8349-cell-top-10-36pt-7a459180.tsv>) · 100.0% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [top-probe-shape-0.hwp](<../../raised-decoration-hancom/top-probe-shape-0.hwp>) · [top-probe-shape-0-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/top-probe-shape-0-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-top-probe-shape-0-7b63c288.tsv>) · 100.0% | [TSV](<candidate/wasm/pr8349-top-probe-shape-0-7b63c288.tsv>) · 100.0% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [top-probe-shape-5.hwp](<../../raised-decoration-hancom/top-probe-shape-5.hwp>) · [top-probe-shape-5-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/top-probe-shape-5-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-top-probe-shape-5-2e0913c7.tsv>) · 99.64478% | [TSV](<candidate/wasm/pr8349-top-probe-shape-5-2e0913c7.tsv>) · 99.64478% |
| [PR #8349](https://github.com/edwardkim/rhwp/pull/8349) · [top-probe-shape-8.hwp](<../../raised-decoration-hancom/top-probe-shape-8.hwp>) · [top-probe-shape-8-hancom-print.pdf](<../../../../../pdf/raised-decoration-hancom/top-probe-shape-8-hancom-print.pdf>) · 2024 | 1 | [TSV](<base/wasm/pr8349-top-probe-shape-8-93feec7b.tsv>) · 90.62866% | [TSV](<candidate/wasm/pr8349-top-probe-shape-8-93feec7b.tsv>) · 90.62866% |
| [PR #8326](https://github.com/edwardkim/rhwp/pull/8326) · [148776468_search_ad_terms_press_release.hwp](<../../../../../samples/issue6929/148776468_search_ad_terms_press_release.hwp>) · [148776468_search_ad_terms_press_release-hwp-2020.pdf](<../../../../../pdf/issue6929/148776468_search_ad_terms_press_release-hwp-2020.pdf>) · 2022 | 17 | [TSV](<base/wasm/pr8326-148776468_search_ad_terms_press_rel-cec2e2a8.tsv>) · 99.61951% | [TSV](<candidate/wasm/pr8326-148776468_search_ad_terms_press_rel-cec2e2a8.tsv>) · 99.61951% |
| [PR #8326](https://github.com/edwardkim/rhwp/pull/8326) · [NewYear_s_Day.hwp](<../../../../../samples/basic/NewYear_s_Day.hwp>) · [NewYear_s_Day-hwp-2020.pdf](<../../../../../pdf/basic/NewYear_s_Day-hwp-2020.pdf>) · 2022 | 3 | [TSV](<base/wasm/pr8326-NewYear_s_Day-6dd2fcb3.tsv>) · 98.4384% | [TSV](<candidate/wasm/pr8326-NewYear_s_Day-6dd2fcb3.tsv>) · 98.4384% |

같은 입력의 원본 SHA·독립 Print SHA와 각 실행의 제품 SHA·전체 PDF 쪽수·export 쪽수·정밀 raw와 TSV 표시를 대조했다.
Native는 실제 native-skia PNG이며 fresh WASM은 실제 고정 package를 사용한 PNG다. 원본·PDF·임계값·허용값을 바꾸지 않았다.
시각 비교에 설치 시스템 글꼴을 사용했으며 상용 글꼴을 숨긴 최소 글꼴 Rust 회귀와 구분한다.
전쪽 overlay를 다시 만들지 않고 이미 캡처한 PNG에서 필요한 경계만 canonical review·별도 overlay로 합성해 판독한다.
[재실행 절차](../visual-reproduction.md)와 [전체 진행 상태](../README.md)에 실행 source·프로필·범위를 연결한다.
이 결과는 새 외부 PR을 추가 체리픽한 것이 아니라 이미 반영된 base의 기능 보존 검사다.
