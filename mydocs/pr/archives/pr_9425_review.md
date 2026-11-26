# PR #9425 검토 — 홀짝 쪽 번호와 머리말

## 최종 판정

**수용 가능.** 원 PR의 변경 동작을 재현했다. 리베이스한 통합 후보의 시각 비교에서 최신 devel과 원 PR보다 낮아진 쪽이 없었다. 하향 반영 정본 4,046행을 모두 충족했다. 화학물질 표시 기준 WASM 26쪽은 최신 devel과 후보 모두 99.99%였다. 별도 264쪽 대조군은 작성자의 재저장본·Print와 바이트가 일치하는 입력을 확보하지 못했다.

## 접수 정보와 체리픽

- 원 PR [#9425](https://github.com/edwardkim/rhwp/pull/9425), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head b94adac40e78a230c810a3f816e27972fcf5ef7b
- 리베이스 후 통합 기준 `48a678399072d0df11380e1ad6a3b6aa77dd677a`와 후보 `afb7b2513c9c821d747484cd85a9ce2bf7b8ae52`
- 전쪽 실행의 리베이스 전 기준 `3fdaa042588cce42006d17e958a7f34d7c205b2e`와 검토 후보 `1959e6160549cb4ae4df9bf8897ab8d848e39556`
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| be8f6a21a39b685a3468f6e12b94b914f2a6c5c5 | 09b1a0a3b24de933a2373dbf3394d0682cf801cb |
| b94adac40e78a230c810a3f816e27972fcf5ef7b | eb1154a32939caa6cc747ace4a37db5bb4f5d169 |

## 변경과 직접 검증

두 쪽 생성 경로가 같은 번호 할당기를 쓴다. 새 번호를 적용한 뒤 pgct와 홀짝이 다를 때 번호를 하나 건너뛴다. 11쪽 발췌본의 Native 변경 영역은 6–11쪽 머리말 띠에 한정되며 머리말 위치와 내용이 새 인쇄본과 일치한다. 작성자가 제출한 264쪽 TSV는 Native 최저 53.36%, fresh WASM 최저 52.46%이며 양쪽에서 같은 29쪽이 90% 미만이다. 해당 쪽은 모두 원 PR의 변경 전에도 90% 미만이었고, 전쪽에서 하락이 없다. [원 PR의 264쪽 TSV](https://github.com/edwardkim/rhwp/pull/9425)

동일 HWP3의 HWP5 변환본을 한컴 2020 11.0.0.9172로 인쇄했다. 변환본 SHA-256은 `2b369d26c8bae00d36e9cb263cc3962798289312068b8d3b39131ae9d7e87858`이며 PDF SHA-256은 `8f8decf1b86e3dc9255fcaa588f7d1680aff58a283c1c0d38af6024753157fcc`다. 원 PR head와 통합 후보를 이 입력으로 다시 실행한 전쪽 TSV는 바이트 단위로 일치했다. 이 재현 조건에서는 Native 최저 50.99%, WASM 최저 51.04%이며 양쪽에 90% 미만 14쪽이 있다. 264쪽 전체를 대상으로 독립 한컴 PDF와 머리말·인쇄 쪽 번호를 대조했다. Native와 fresh WASM 각각 devel에서 번호 불일치 129쪽과 머리말 불일치 116쪽이 있었고, 원 PR head와 통합 후보에서는 둘 다 0쪽이었다. 136쪽에서 141번으로 건너뛰고 210쪽은 215번과 홀수 머리말이 일치했다. 90% 미만 14쪽에도 번호와 머리말 불일치가 없다. 256–264쪽의 라틴 낱말 간격과 줄 끝 하이픈은 별도로 등록돼 있다. [#9426](https://github.com/edwardkim/rhwp/issues/9426)

공공·민간 복합사업 문서의 물리 131쪽은 devel과 통합 후보 모두 머리말이 표시됐다. Native 렌더 트리도 바이트 단위로 일치했다. [이슈의 원 PR 확인 기록](https://github.com/edwardkim/rhwp/issues/8482#issuecomment-7184017608)

## 전쪽 검증

원본은 samples/issue8482/germany_law_ch3_resaved2020.hwp다. 기준은 한컴 MCP 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄 출력이며 SHA-256은 59659c43d4951f54857a88c31ec52f2c729677fa4d0c74bd8737bead7fadeb1c이다. compat 2022, PDFium 96 DPI, 2px raw 실루엣으로 비교했다.

| 출력 | 쪽수 | devel → 후보 최저 | 90% 미만 | devel 대비 하락 | 원 PR 대비 |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia | 11 | 93.73% → 95.53% | 0쪽 | 0쪽 | 하락 0쪽 |
| fresh WASM Canvas | 11 | 93.74% → 95.54% | 0쪽 | 0쪽 | 하락 0쪽 |

원본 SHA-256은 `945183c9102bb7c62c137779d0765c6ab4fa5f36453ba7990f1e91ca3b98401f`다. [검증 입력](../../../samples/issue8482/germany_law_ch3_resaved2020.hwp)

실제 비교에 사용한 새 인쇄본을 `08b3c4e08c`에 포함했다. [한컴 Print PDF](../../../pdf/issue8482/mcp-11.0.0.9172/germany_law_ch3_resaved2020-2020.pdf) · [작성자 기준 PDF](../../../pdf/issue8482/germany_law_ch3-2020.pdf)

작성자 제출값을 보존했다. devel·원 PR head·통합 후보는 하나의 인쇄본과 동일한 글꼴 조건으로 재측정했다. 원 PR 대비 판정은 같은 검증 조건의 측정값을 사용했다. [Native 전쪽 비교](../assets/planet6897_20261125/integration-review/9425/native-page-comparison.tsv) · [WASM 전쪽 비교](../assets/planet6897_20261125/integration-review/9425/wasm-page-comparison.tsv)

## 공통 로컬 검증

시각 검증의 Native·WASM 빌드에 `5c216e8e657309f9bb33928c6a6cd2747df77217`의 소스를 사용했다. `1959e61605`까지의 추가 변경은 한컴 Print PDF 세 건이다. 호스트에서 새 WASM 패키지를 빌드·최적화한 뒤 Canvas에서 실행했다. 최소 글꼴 환경의 집중 nextest 12건과 전체 nextest 14,194건이 통과했고 전체 검사에서 45건이 건너뛰어졌다. format·Clippy·workspace build와 Native Skia 라이브러리 및 CI 통합 모듈 검사도 통과했다.

## 하향 정본과 리베이스 확인

정본 TSV의 SHA-256은 `5d5ef7ac531f3d038757636920cb4af0267796f3631a68d2b5fece77291a5e13`이다. 새 한컴 2020 Print PDF의 화학물질 전쪽 결과와 한컴바탕 1.21을 공급한 중첩 표 결과를 반영해 쪽별 하한을 다시 대조했다. 중첩 표 17쪽의 Native·WASM 34행은 하한을 모두 충족했다. [정본 TSV](../assets/visual-protected-baseline/final.tsv) · [Print 교체 영수증](../assets/visual-protected-baseline/updates/issue6782-mcp-print-20261126.json) · [WASM 26쪽 조정 영수증](../assets/visual-protected-baseline/updates/issue6782-wasm-p26-20261126.json) · [화학물질 쪽별 대조](../assets/issue_6782/mcp-print-20261126/page-comparison.tsv)

최신 devel로 리베이스했고 23개 커밋의 적용 패치가 모두 일치했다. 리베이스 후 후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native 제출 샘플 210쪽은 최신 devel과 원 PR 대비 하락이 없었다. 화학물질 WASM 103쪽은 최신 devel·후보의 PNG 해시가 모두 같았고, 26쪽은 양쪽에서 99.99%였다. Chrome에서 #9424의 자동 줄바꿈 본문 2줄·표 셀 3줄, 본문 선두 수식과 뒤 글자의 선택 영역을 대조해 모두 같은 결과를 확인했다. [#9424 검토](pr_9424_review.md)

## 리베이스 후 최종 로컬 검증

후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native·fresh WASM 각각 11쪽을 다시 출력했다. 최신 devel과 원 PR 대비 raw 실루엣 하락은 0쪽이다. 집중 nextest 19건, 전체 nextest 14,196건, Native Skia 4,225건과 Rustfmt·세 가지 Clippy 검사·workspace build를 통과했다. 공개 공통 보호 8행도 통과했다. [최종 검증](../assets/planet6897_20261125/integration-review/rebase-validation.json) · [정본 보호 4,046행](../assets/planet6897_20261125/integration-review/protected-page-comparison.tsv)

이 PR의 한컴 2020 Print와 후보 PNG로 대표 판독 자료를 만들었다. [검증 PDF](../../../pdf/issue8482/mcp-11.0.0.9172/germany_law_ch3_resaved2020-2020.pdf) · [대표 review](../assets/planet6897_20261125/integration-review/9425/final-native/review_006.png) · [대표 overlay](../assets/planet6897_20261125/integration-review/9425/final-native/overlay_006.png) · [페이지별 픽셀 지표와 이미지 SHA-256](../assets/planet6897_20261125/integration-review/9425/final-native/evidence.json)

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 작성자 참고 PNG의 원래 경로는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_8482/native/review_006.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_8482/native/review_006.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment) 게시할 review와 overlay는 위 리베이스 후 최종 로컬 검증의 동일 페이지 자료를 사용한다.
