# PR #9413 검토 — 마지막 앵커 줄과 그림의 쪽 소속

## 최종 판정

**리베이스 후 재검증 중.** 원 PR의 변경 동작을 재현했다. 하향 반영 정본을 적용한 리베이스 전 실행 증적의 재대조에서 4,046행이 모두 하한을 충족했다. 화학물질 표시 기준 19·20·21·91·92쪽의 Native·WASM 10행과 WASM 26쪽 99.99%를 적용했다.

## 접수 정보와 체리픽

- 원 PR [#9413](https://github.com/edwardkim/rhwp/pull/9413), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head 60cecddd2c03fc9dee7b9e73dc37db30e222f9a2
- 리베이스 후 통합 기준 `48a678399072d0df11380e1ad6a3b6aa77dd677a`와 후보 `afb7b2513c9c821d747484cd85a9ce2bf7b8ae52`
- 전쪽 실행의 리베이스 전 기준 `3fdaa042588cce42006d17e958a7f34d7c205b2e`와 검토 후보 `1959e6160549cb4ae4df9bf8897ab8d848e39556`
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| 52cbe212bde0afccadc0121ba2b529d0b0f650c2 | 698c7a1e7e87e0846e610cb66d52b8aedd190217 |
| 99ad1b3b77e75857ab331b86b060b8854a6420d6 | 7aff25f14c4ed8c6210247f24f7aca828e8fa7a4 |
| 60cecddd2c03fc9dee7b9e73dc37db30e222f9a2 | 674377db1dc3dcf2611c72b39820bb0cd3d2eb84 |

## 변경과 직접 검증

저장 앵커 줄 18,000 HU와 세로 오프셋 2,161 HU, 그림 높이 9,900 HU의 합이 다음 문단의 저장 위치 30,061 HU와 일치한다. 쪽 소속과 배치가 이 관계를 함께 쓴다. 그림은 4쪽 꼬리말에서 사라지고 5쪽 앵커 줄 뒤에 한 번 나온다. 4쪽 Native 99.93%, WASM 99.94%이고 5쪽은 양쪽 100.00%였다. #6146 대조 조건도 집중 회귀에서 통과했다. 새 PDF는 1959e61605에 포함했다.

## 전쪽 검증

원본은 samples/issue9396/fire_inspection_manual_p71-75.hwp다. 기준은 한컴 MCP 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄 출력이며 SHA-256은 a7924790f2d5e1e49c3746a8c3983cf370424e7badc637a63a1289103b1d68d2이다. compat 2022, PDFium 96 DPI, 2px raw 실루엣으로 비교했다.

| 출력 | 쪽수 | devel → 후보 최저 | 90% 미만 | devel 대비 하락 | 원 PR 대비 |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia | 7 | 84.55% → 99.93% | 0쪽 | 0쪽 | 하락 0쪽 |
| fresh WASM Canvas | 7 | 84.56% → 99.94% | 0쪽 | 0쪽 | 하락 0쪽 |

원본 SHA-256은 `f22c770b10965a9fb55e6da7fe93f000d6506ba426434d583ee195a3331a3115`다. [검증 입력](../../../samples/issue9396/fire_inspection_manual_p71-75.hwp)

실제 비교에 사용한 인쇄본은 `1959e61605`에 포함했다. [한컴 Print PDF](../../../pdf/issue9396/mcp-11.0.0.9172/fire_inspection_manual_p71-75-2020.pdf)

작성자 제출값을 보존했다. devel·원 PR head·통합 후보는 하나의 인쇄본과 동일한 글꼴 조건으로 재측정했다. 원 PR 대비 판정은 같은 검증 조건의 측정값을 사용했다. [Native 전쪽 비교](../assets/planet6897_20261125/integration-review/9413/native-page-comparison.tsv) · [WASM 전쪽 비교](../assets/planet6897_20261125/integration-review/9413/wasm-page-comparison.tsv)

## 공통 로컬 검증

시각 검증의 Native·WASM 빌드에 `5c216e8e657309f9bb33928c6a6cd2747df77217`의 소스를 사용했다. `1959e61605`까지의 추가 변경은 한컴 Print PDF 세 건이다. 호스트에서 새 WASM 패키지를 빌드·최적화한 뒤 Canvas에서 실행했다. 최소 글꼴 환경의 집중 nextest 12건과 전체 nextest 14,194건이 통과했고 전체 검사에서 45건이 건너뛰어졌다. format·Clippy·workspace build와 Native Skia 라이브러리 및 CI 통합 모듈 검사도 통과했다.

## 하향 정본과 리베이스 확인

정본 TSV의 SHA-256은 `5d5ef7ac531f3d038757636920cb4af0267796f3631a68d2b5fece77291a5e13`이다. 새 한컴 2020 Print PDF의 화학물질 전쪽 결과와 한컴바탕 1.21을 공급한 중첩 표 결과를 반영해 쪽별 하한을 다시 대조했다. 중첩 표 17쪽의 Native·WASM 34행은 하한을 모두 충족했다. [정본 TSV](../assets/visual-protected-baseline/final.tsv) · [Print 교체 영수증](../assets/visual-protected-baseline/updates/issue6782-mcp-print-20261126.json) · [WASM 26쪽 조정 영수증](../assets/visual-protected-baseline/updates/issue6782-wasm-p26-20261126.json) · [화학물질 쪽별 대조](../assets/issue_6782/mcp-print-20261126/page-comparison.tsv)

최신 devel로 리베이스했고 23개 커밋의 적용 패치가 모두 일치했다. 리베이스 후 후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native 제출 샘플 210쪽은 최신 devel과 원 PR 대비 하락이 없었다. 화학물질 WASM 103쪽은 최신 devel·후보의 PNG 해시가 모두 같았고, 26쪽은 양쪽에서 99.99%였다. Chrome에서 #9424의 자동 줄바꿈 본문 2줄·표 셀 3줄, 본문 선두 수식과 뒤 글자의 선택 영역을 대조해 모두 같은 결과를 확인했다. [#9424 검토](pr_9424_review.md)

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 대표 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_9396/native/review_p005.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_9396/native/review_p005.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
