# PR #9427 검토 — 한컴 저장 HWPX의 단 첫 표 위 간격

## 최종 판정

**수용 가능.** 원 PR의 변경 동작을 재현했다. 리베이스한 통합 후보의 시각 비교에서 최신 devel과 원 PR보다 낮아진 쪽이 없었다. 하향 반영 정본 4,046행을 모두 충족했다. 화학물질 표시 기준 WASM 26쪽은 최신 devel과 후보 모두 99.99%였다.

## 접수 정보와 체리픽

- 원 PR [#9427](https://github.com/edwardkim/rhwp/pull/9427), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head f38a68e1988fdffc2c0a2021261a2c1486212a7a
- 리베이스 후 통합 기준 `48a678399072d0df11380e1ad6a3b6aa77dd677a`와 후보 `afb7b2513c9c821d747484cd85a9ce2bf7b8ae52`
- 전쪽 실행의 리베이스 전 기준 `3fdaa042588cce42006d17e958a7f34d7c205b2e`와 검토 후보 `1959e6160549cb4ae4df9bf8897ab8d848e39556`
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| f70f6fe602d065ba9bd8faceb2dd1a0d254dfafb | 4619ac03ad6a8ff495e234dfc0f0002b13dbeb0c |
| c4fc3f0b2adc49f5d334aaaaee1bcea14b87ff65 | 8c700432b924a051039a27160df10d6fa23d8e95 |
| 4471dae889a670c2efeb0c9e65d487dbf4622ace | 22a6e7aa37e34d9ee4bff7c709070688fd9310a8 |
| f38a68e1988fdffc2c0a2021261a2c1486212a7a | 90329bd861d9b7717ac35f4f0ad469d2e6233fa8 |

## 변경과 직접 검증

21·23쪽에서 저장 첫 줄이 나타내는 600 HU를 원점 설정과 host 간격 계산에 함께 적용한다. 두 쪽의 통합 후보는 Native 98.52·100.00%, WASM 99.23·100.00%였다. HWP와 HWPX를 새로 인쇄한 PDF의 54쪽 래스터는 픽셀까지 같다. 36쪽은 작성자 기존 PDF의 수정 전·후 Native 95.96%다. 같은 로컬 PNG를 고정한 비교는 기존 PDF 95.86%, 새 PDF 95.37%였다. 새 PDF에서 devel과 후보의 36쪽 Native PNG는 바이트가 같다. HWP 대조군의 36쪽은 두 경로 모두 100.00%였고 HWPX 글상자는 #9422로 남긴다. 최종 비교에는 확인된 `SPMJ.otf`를 포함한 27개 글꼴 프로그램을 공급했다. 리베이스 전후 WASM 42–54쪽 PNG는 쪽 아래의 작은 영역에서 달랐다. 최신 devel과 후보에서는 이 13쪽 PNG가 바이트 단위로 일치했고, 전 54쪽의 표시 일치율도 리베이스 전과 일치했다. 새 HWPX PDF를 사용한 54쪽 대조에서 원 PR 대비 하락은 양쪽 모두 0쪽이다. HWP 원본으로 다시 인쇄한 54쪽 PDF를 pdf/issue9379/mcp-11.0.0.9172/kaf_appendix-mcp-print-2020.pdf에 추가했다.

## 전쪽 검증

원본은 samples/issue9419/kaf_appendix_2020.hwpx다. 기준은 한컴 MCP 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄 출력이며 SHA-256은 d1760ac55a47df2f477e2155ecf7cf21f69dbfa58aca5fdd8efec78719c25d83이다. compat 2022, PDFium 96 DPI, 2px raw 실루엣으로 비교했다.

| 출력 | 쪽수 | devel → 후보 최저 | 90% 미만 | devel 대비 하락 | 원 PR 대비 |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia | 54 | 60.56% → 95.37% | 0쪽 | 0쪽 | 하락 0쪽 |
| fresh WASM Canvas | 54 | 60.50% → 95.37% | 0쪽 | 0쪽 | 하락 0쪽 |

원본 SHA-256은 `57f6d3a62073e6b026e7e81119551ef2737080c8ea21fbb5a0a38d1c4e562676`다. [검증 입력](../../../samples/issue9419/kaf_appendix_2020.hwpx)

실제 비교에 사용한 인쇄본은 `1959e61605`에 포함했다. [한컴 Print PDF](../../../pdf/issue9419/mcp-11.0.0.9172/kaf_appendix_2020-2020.pdf)

작성자 제출값을 보존했다. devel·원 PR head·통합 후보는 하나의 인쇄본과 동일한 글꼴 조건으로 재측정했다. 원 PR 대비 판정은 같은 검증 조건의 측정값을 사용했다. [Native 전쪽 비교](../assets/planet6897_20261125/integration-review/9427/native-page-comparison.tsv) · [WASM 전쪽 비교](../assets/planet6897_20261125/integration-review/9427/wasm-page-comparison.tsv)

## 공통 로컬 검증

시각 검증의 Native·WASM 빌드에 `5c216e8e657309f9bb33928c6a6cd2747df77217`의 소스를 사용했다. `1959e61605`까지의 추가 변경은 한컴 Print PDF 세 건이다. 호스트에서 새 WASM 패키지를 빌드·최적화한 뒤 Canvas에서 실행했다. 최소 글꼴 환경의 집중 nextest 12건과 전체 nextest 14,194건이 통과했고 전체 검사에서 45건이 건너뛰어졌다. format·Clippy·workspace build와 Native Skia 라이브러리 및 CI 통합 모듈 검사도 통과했다.

## 하향 정본과 리베이스 확인

정본 TSV의 SHA-256은 `5d5ef7ac531f3d038757636920cb4af0267796f3631a68d2b5fece77291a5e13`이다. 새 한컴 2020 Print PDF의 화학물질 전쪽 결과와 한컴바탕 1.21을 공급한 중첩 표 결과를 반영해 쪽별 하한을 다시 대조했다. 중첩 표 17쪽의 Native·WASM 34행은 하한을 모두 충족했다. [정본 TSV](../assets/visual-protected-baseline/final.tsv) · [Print 교체 영수증](../assets/visual-protected-baseline/updates/issue6782-mcp-print-20261126.json) · [WASM 26쪽 조정 영수증](../assets/visual-protected-baseline/updates/issue6782-wasm-p26-20261126.json) · [화학물질 쪽별 대조](../assets/issue_6782/mcp-print-20261126/page-comparison.tsv)

최신 devel로 리베이스했고 23개 커밋의 적용 패치가 모두 일치했다. 리베이스 후 후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native 제출 샘플 210쪽은 최신 devel과 원 PR 대비 하락이 없었다. 화학물질 WASM 103쪽은 최신 devel·후보의 PNG 해시가 모두 같았고, 26쪽은 양쪽에서 99.99%였다. Chrome에서 #9424의 자동 줄바꿈 본문 2줄·표 셀 3줄, 본문 선두 수식과 뒤 글자의 선택 영역을 대조해 모두 같은 결과를 확인했다. [#9424 검토](pr_9424_review.md)

## 리베이스 후 최종 로컬 검증

후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native·fresh WASM 각각 54쪽을 다시 출력했다. 최신 devel과 원 PR 대비 raw 실루엣 하락은 0쪽이다. 집중 nextest 19건, 전체 nextest 14,196건, Native Skia 4,225건과 Rustfmt·세 가지 Clippy 검사·workspace build를 통과했다. 공개 공통 보호 8행도 통과했다. [최종 검증](../assets/planet6897_20261125/integration-review/rebase-validation.json) · [정본 보호 4,046행](../assets/planet6897_20261125/integration-review/protected-page-comparison.tsv)

이 PR의 한컴 2020 Print와 후보 PNG로 대표 판독 자료를 만들었다. [검증 PDF](../../../pdf/issue9419/mcp-11.0.0.9172/kaf_appendix_2020-2020.pdf) · [대표 review](../assets/planet6897_20261125/integration-review/9427/final-native/review_021.png) · [대표 overlay](../assets/planet6897_20261125/integration-review/9427/final-native/overlay_021.png) · [페이지별 픽셀 지표와 이미지 SHA-256](../assets/planet6897_20261125/integration-review/9427/final-native/evidence.json)

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 작성자 참고 PNG의 원래 경로는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_9419/native/review_021.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_9419/native/review_021.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment) 게시할 review와 overlay는 위 리베이스 후 최종 로컬 검증의 동일 페이지 자료를 사용한다.
