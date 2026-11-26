# PR #9409 검토 — 표 조각 뒤 원점과 저장 간격의 그림

## 최종 판정

**머지 보류.** 원 PR의 변경 동작을 재현했다. 정본 보호 4,046행 중 2,597행에서 하락이 없었다. 기준 글꼴 28개가 필요한 정본 보호 1,449행의 검증이 남았다.

## 접수 정보와 체리픽

- 원 PR [#9409](https://github.com/edwardkim/rhwp/pull/9409), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head fef9edd73f61b6caa4f0a1b4970a674e6c1806ae
- 통합 기준 3fdaa042588cce42006d17e958a7f34d7c205b2e
- 검증한 통합 후보 1959e6160549cb4ae4df9bf8897ab8d848e39556
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| afad62154c331ef2c1871636a60ff0b92fc961d5 | 54f84c34b25a8fc1b7a1c1789b7410e00cb3045d |
| 993f550d9c84059333a09229293022bd908bf6f6 | 6167915acf5e572e071cbbc6cec3943e1f43a13e |
| 95e5b6ad775bbb667884ba1ef890671122b2429a | 403aedb195d9c1de45d5bfa4a73aebe1d6a8e15e |
| dec6a8b0b8ed2bab51ea911ddd1a9d9ac06730f5 | 52b0b32d55290d9e6d61d237524bf9b36c434fe5 |
| add031214ce189d070bbe75f2cfbcb23470ab9b6 | ac6ca1e61f40493021d8faf68219a32238f9fd08 |
| 768517e716ac3829ee5df1a74080b58d5e6b2ec4 | 58a1dd50e3f4db1f8240746ff3729f07c97f837e |
| fef9edd73f61b6caa4f0a1b4970a674e6c1806ae | 758149ce140452a370389cd0a83ec8a99d19a0f0 |

## 변경과 직접 검증

문단 기준 표의 원점을 저장 첫 줄에서 문단 위 간격을 되돌려 구한다. 저장 간격에 들어가는 그림은 상자 바닥이 저장 줄의 윗변에 닿는다. 새 한컴 인쇄본과 15·66·73·134쪽을 확대 대조했다. Native 일치율은 각각 100.00·99.99·99.83·100.00%였다. 98% 미만인 2–4쪽 Native PNG는 현행 devel과 바이트가 같다.

## 전쪽 검증

원본은 samples/issue8732/1741000-202200045_second_interim_report.hwp다. 기준은 한컴 MCP 2020 11.0.0.9172의 Hancom PDF 한 쪽씩 인쇄 출력이며 SHA-256은 9d6dedc626049e4ba04e71b6853ac53171c1d9470aa9bbeff8220f80f1e640bb이다. compat 2022, PDFium 96 DPI, 2px raw 실루엣으로 비교했다.

| 출력 | 쪽수 | devel → 후보 최저 | 90% 미만 | devel 대비 하락 | 원 PR 대비 |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia | 138 | 27.04% → 95.77% | 0쪽 | 0쪽 | 하락 0쪽 |
| fresh WASM Canvas | 138 | 27.19% → 95.65% | 0쪽 | 0쪽 | 하락 0쪽 |

원본 SHA-256은 `a8342140eafc10194f9beef693f5da8658f3fc1d170111799aae4b23d8351cae`다. [검증 입력](../../../samples/issue8732/1741000-202200045_second_interim_report.hwp)

실제 비교에 사용한 새 인쇄본은 로컬 증적에 보존했다. 저장소에는 작성자가 사용한 인쇄본이 포함돼 있다. [작성자 기준 PDF](../../../pdf/issue8732/1741000-202200045_second_interim_report-2020.pdf)

작성자 제출값을 보존했다. devel·원 PR head·통합 후보는 하나의 인쇄본과 동일한 글꼴 조건으로 재측정했다. 원 PR 대비 판정은 같은 검증 조건의 측정값을 사용했다. [Native 전쪽 비교](../assets/planet6897_20261125/integration-review/9409/native-page-comparison.tsv) · [WASM 전쪽 비교](../assets/planet6897_20261125/integration-review/9409/wasm-page-comparison.tsv)

## 공통 로컬 검증

시각 검증의 Native·WASM 빌드에 `5c216e8e657309f9bb33928c6a6cd2747df77217`의 소스를 사용했다. `1959e61605`까지의 추가 변경은 한컴 Print PDF 세 건이다. 호스트에서 새 WASM 패키지를 빌드·최적화한 뒤 Canvas에서 실행했다. 최소 글꼴 환경의 집중 nextest 12건과 전체 nextest 14,194건이 통과했고 전체 검사에서 45건이 건너뛰어졌다. format·Clippy·workspace build와 Native Skia 라이브러리 및 CI 통합 모듈 검사도 통과했다.

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 대표 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_8735/native/review_066.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_8735/native/review_066.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
