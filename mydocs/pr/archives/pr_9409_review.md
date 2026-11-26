# PR #9409 검토 — 표 조각 뒤 원점과 저장 간격의 그림

## 최종 판정

**리베이스 후 재검증 중.** 원 PR의 변경 동작을 재현했다. 하향 반영 정본을 적용한 리베이스 전 실행 증적의 재대조에서 4,046행이 모두 하한을 충족했다. 화학물질 표시 기준 19·20·21·91·92쪽의 Native·WASM 10행과 WASM 26쪽 99.99%를 적용했다.

## 접수 정보와 체리픽

- 원 PR [#9409](https://github.com/edwardkim/rhwp/pull/9409), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head fef9edd73f61b6caa4f0a1b4970a674e6c1806ae
- 리베이스 후 통합 기준 `48a678399072d0df11380e1ad6a3b6aa77dd677a`와 후보 `afb7b2513c9c821d747484cd85a9ce2bf7b8ae52`
- 전쪽 실행의 리베이스 전 기준 `3fdaa042588cce42006d17e958a7f34d7c205b2e`와 검토 후보 `1959e6160549cb4ae4df9bf8897ab8d848e39556`
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| afad62154c331ef2c1871636a60ff0b92fc961d5 | c2be61e0d6976351c7624165cf32854348d5f54e |
| 993f550d9c84059333a09229293022bd908bf6f6 | 4c356bdd883dee19209be2bb5d8525911cd5d5b9 |
| 95e5b6ad775bbb667884ba1ef890671122b2429a | 28d6521b218a3c67c4536edc8b799d71ddcfffcb |
| dec6a8b0b8ed2bab51ea911ddd1a9d9ac06730f5 | b1e003dde6c9b12a3a713e1bf505361a03eb6c10 |
| add031214ce189d070bbe75f2cfbcb23470ab9b6 | 430361d50e32d66be41da32c50a66743806f3678 |
| 768517e716ac3829ee5df1a74080b58d5e6b2ec4 | 77d15af14d935b2ffefc28cecb3612815ffd86a9 |
| fef9edd73f61b6caa4f0a1b4970a674e6c1806ae | d4b640951c1e389b4767f2d7e332f05743c12cb0 |

## 변경과 직접 검증

문단 기준 표의 원점을 저장 첫 줄에서 문단 위 간격을 되돌려 구한다. 저장 간격에 들어가는 그림은 상자 바닥이 저장 줄의 윗변에 닿는다. 새 한컴 인쇄본과 15·66·73·134쪽을 확대 대조했다. Native 일치율은 각각 100.00·99.99·99.83·100.00%였다. 98% 미만인 2–4쪽 Native PNG는 현행 devel과 바이트가 같다.

## 전쪽 검증

원본은 samples/issue8732/1741000-202200045_second_interim_report.hwp다. 기준은 한컴 MCP 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄 출력이며 SHA-256은 9d6dedc626049e4ba04e71b6853ac53171c1d9470aa9bbeff8220f80f1e640bb이다. compat 2022, PDFium 96 DPI, 2px raw 실루엣으로 비교했다.

| 출력 | 쪽수 | devel → 후보 최저 | 90% 미만 | devel 대비 하락 | 원 PR 대비 |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia | 138 | 27.04% → 95.77% | 0쪽 | 0쪽 | 하락 0쪽 |
| fresh WASM Canvas | 138 | 27.19% → 95.65% | 0쪽 | 0쪽 | 하락 0쪽 |

원본 SHA-256은 `a8342140eafc10194f9beef693f5da8658f3fc1d170111799aae4b23d8351cae`다. [검증 입력](../../../samples/issue8732/1741000-202200045_second_interim_report.hwp)

실제 비교에 사용한 새 인쇄본을 `08b3c4e08c`에 포함했다. [한컴 Print PDF](../../../pdf/issue8732/mcp-11.0.0.9172/1741000-202200045_second_interim_report-2020.pdf) · [작성자 기준 PDF](../../../pdf/issue8732/1741000-202200045_second_interim_report-2020.pdf)

작성자 제출값을 보존했다. devel·원 PR head·통합 후보는 하나의 인쇄본과 동일한 글꼴 조건으로 재측정했다. 원 PR 대비 판정은 같은 검증 조건의 측정값을 사용했다. [Native 전쪽 비교](../assets/planet6897_20261125/integration-review/9409/native-page-comparison.tsv) · [WASM 전쪽 비교](../assets/planet6897_20261125/integration-review/9409/wasm-page-comparison.tsv)

## 공통 로컬 검증

시각 검증의 Native·WASM 빌드에 `5c216e8e657309f9bb33928c6a6cd2747df77217`의 소스를 사용했다. `1959e61605`까지의 추가 변경은 한컴 Print PDF 세 건이다. 호스트에서 새 WASM 패키지를 빌드·최적화한 뒤 Canvas에서 실행했다. 최소 글꼴 환경의 집중 nextest 12건과 전체 nextest 14,194건이 통과했고 전체 검사에서 45건이 건너뛰어졌다. format·Clippy·workspace build와 Native Skia 라이브러리 및 CI 통합 모듈 검사도 통과했다.

## 하향 정본과 리베이스 확인

정본 TSV의 SHA-256은 `5d5ef7ac531f3d038757636920cb4af0267796f3631a68d2b5fece77291a5e13`이다. 새 한컴 2020 Print PDF의 화학물질 전쪽 결과와 한컴바탕 1.21을 공급한 중첩 표 결과를 반영해 쪽별 하한을 다시 대조했다. 중첩 표 17쪽의 Native·WASM 34행은 하한을 모두 충족했다. [정본 TSV](../assets/visual-protected-baseline/final.tsv) · [Print 교체 영수증](../assets/visual-protected-baseline/updates/issue6782-mcp-print-20261126.json) · [WASM 26쪽 조정 영수증](../assets/visual-protected-baseline/updates/issue6782-wasm-p26-20261126.json) · [화학물질 쪽별 대조](../assets/issue_6782/mcp-print-20261126/page-comparison.tsv)

최신 devel로 리베이스했고 23개 커밋의 적용 패치가 모두 일치했다. 리베이스 후 후보 `74f3fc1eca55612f8c3675ce0811bdde0dd32dbc`에서 Native 제출 샘플 210쪽은 최신 devel과 원 PR 대비 하락이 없었다. 화학물질 WASM 103쪽은 최신 devel·후보의 PNG 해시가 모두 같았고, 26쪽은 양쪽에서 99.99%였다. Chrome에서 #9424의 자동 줄바꿈 본문 2줄·표 셀 3줄, 본문 선두 수식과 뒤 글자의 선택 영역을 대조해 모두 같은 결과를 확인했다. [#9424 검토](pr_9424_review.md)

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 대표 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_8735/native/review_066.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_8735/native/review_066.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
