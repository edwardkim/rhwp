# PR #9427 검토 — 한컴 저장 HWPX의 단 첫 표 위 간격

## 최종 판정

**머지 보류.** 원 PR의 변경 동작을 재현했다. 정본 보호 4,046행 중 2,597행에서 하락이 없었다. 기준 글꼴 28개가 필요한 정본 보호 1,449행의 검증이 남았다.

## 접수 정보와 체리픽

- 원 PR [#9427](https://github.com/edwardkim/rhwp/pull/9427), planet6897 → devel, MERGEABLE, reviewer jangster77
- 원 head f38a68e1988fdffc2c0a2021261a2c1486212a7a
- 통합 기준 3fdaa042588cce42006d17e958a7f34d7c205b2e
- 검증한 통합 후보 1959e6160549cb4ae4df9bf8897ab8d848e39556
- #9409·#9413·#9425·#9427 순으로 16개 커밋을 충돌 없이 적용했다.

| 이 PR의 원본 SHA | 적용 SHA |
| --- | --- |
| f70f6fe602d065ba9bd8faceb2dd1a0d254dfafb | 4b27177e203e881c9260d68bd145a05ca90a21a2 |
| c4fc3f0b2adc49f5d334aaaaee1bcea14b87ff65 | a088c2b1a487ebcb5ca1e5cedcf33dfc138e7a71 |
| 4471dae889a670c2efeb0c9e65d487dbf4622ace | 1c826c4e39453833e60d6eabe5a290d43bfcbd95 |
| f38a68e1988fdffc2c0a2021261a2c1486212a7a | 5c216e8e657309f9bb33928c6a6cd2747df77217 |

## 변경과 직접 검증

21·23쪽에서 저장 첫 줄이 나타내는 600 HU를 원점 설정과 host 간격 계산에 함께 적용한다. 두 쪽의 통합 후보는 Native 98.52·100.00%, WASM 99.23·100.00%였다. HWP와 HWPX를 새로 인쇄한 PDF의 54쪽 래스터는 픽셀까지 같다. 36쪽은 작성자 기존 PDF의 수정 전·후 Native 95.96%다. 같은 로컬 PNG를 고정한 비교는 기존 PDF 95.86%, 새 PDF 95.37%였다. 새 PDF에서 devel과 후보의 36쪽 Native PNG는 바이트가 같다. HWP 대조군의 36쪽은 두 경로 모두 100.00%였고 HWPX 글상자는 #9422로 남긴다. 최종 비교는 확인된 SPMJ.otf와 고정한 26개 글꼴을 공급했다. 새 HWPX PDF를 사용한 54쪽 대조에서 원 PR 대비 하락은 양쪽 모두 0쪽이다. HWP 원본으로 다시 인쇄한 54쪽 PDF를 pdf/issue9379/mcp-11.0.0.9172/kaf_appendix-mcp-print-2020.pdf에 추가했다.

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

## 병합 후 기여자 안내

병합 SHA와 CI를 확인한 뒤 원 PR에 동일 조건의 전쪽 비교와 변경 영역의 판독 결과를 한국어 존댓말로 안내한다. 대표 PNG는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_9419/native/review_021.png` 형식으로 연결한다. 게시 시 `--body-file`을 사용하고 API로 본문을 재확인한다. [원 PR 제출 대표 PNG](../assets/issue_9419/native/review_021.png) · [Visual Sweep 안내](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
