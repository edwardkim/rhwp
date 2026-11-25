# PR #9386 검토 — 중첩 표의 쪽 경계와 행 프레임

## 최종 판정

메인터너 보정 뒤 최신 MCP 기준 24쪽은 Native·fresh WASM 모두 **100.00%**다. 같은 한컴 2020
재인쇄본으로 원 PR과 최종 후보를 대조한 44쪽에서 양 backend 모두 하락 0쪽이다. 31쪽에는 새
한컴 2020·2024 MCP PDF의 CID 참조 오류가 있으며, 유효한 최초 한컴 2020 Print에서는 양쪽
**98.84%**였다. 최초 인쇄본 기준 전쪽 최저는 Native 98.33%·fresh WASM 98.38%다.
[KAF 최종 후보 비교](../assets/planet6897_20261125/kaf-print-repair/README.md)

[원 PR #9386](https://github.com/edwardkim/rhwp/pull/9386) · [이슈 #9000](https://github.com/edwardkim/rhwp/issues/9000)

KAF 회의자료에서 표 구간을 남긴 44쪽 파생본이다. 코드는 칸마다 저장 프레임을 열었던 유닛을 찾아 같은 순번에서 조각을 닫는다. 이어지는 조각은 쪽 프레임 아래끝까지 점유하고, 마지막 소진 조각은 남은 높이에서 닫는다. 검토 경계는 33~44쪽 표와 경계 글 소속, 비어 있는 선행 문단의 앞 간격, 이어받는 중첩 표의 위 바깥 여백이다.

원 PR의 전쪽 최저는 Native 94.32%·WASM 93.13%로 1쪽 표지에서 발생했고, 주장하는 33~44쪽은 양쪽 98% 이상이었다. 첨부 PDF는 PDF 1.4, A4 가로 44쪽이며 `Creator`에는 `Hwp 2018 11.0.0.1623`, `Producer`에는 `Hancom PDF 1.3.0.404`가 기록됐다. 저장소의 실행 파일 실측에서 이 빌드는 한컴 2020이며, 기존 MCP 2020 Print PDF에서도 같은 Creator·Producer를 확인했다. 원 PR이 기록한 동일 파생본의 Hancom PDF Print와 SHA-256이 첨부본에 맞아 기존 PDF를 기준으로 재사용했다. [제품 버전 실측](../../manual/verification/hangul_version_oracle.md#51-설치된-버전-확인) · [MCP 대조 기록](pr_9317_review.md#입력과-독립-인쇄-기준)

갱신된 MCP에서 같은 원본을 한컴 2020으로 다시 출력했다. client 0.10.0에서 `start --engine 2020 --target pdf --timeout-seconds 1800`을 호출하고, `status`가 성공한 뒤 같은 job을 `download`로 받았다. `status.result`와 `download.server`의 `pdf_output_mode`는 `hancom2020_pdf_driver_one_up`, `pdf_print_method`는 `0`이다. MCP는 PDF 글꼴 이름의 공백 문법을 보정하고 전체 44쪽을 확인했다. 새 한컴 빌드 `11.0.0.9172`의 인쇄본을 원 PR 인쇄본과 96 DPI로 비교하면 표시값이 100.00% 미만인 쪽이 22쪽이다. 기존 인쇄본으로 최초 제출 수치를 재현했다. 벤치마크의 인쇄 조건을 통일하기 위해 원 PR과 통합 후보 양쪽을 이번 MCP 인쇄본에 전쪽 대조했다. [두 인쇄본 대조 및 MCP 영수증](../assets/planet6897_20261125/mcp-print-reissue.json) · [MCP 기준 PDF 인쇄 계약](../../manual/mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)

## 문구의 쪽 소속 재현

`Standard: KAF address standard`, `landmark`, `surname, given`은 devel 34·38·43쪽에서 후보 33·37·42쪽으로 옮겨갔다. 첨부 PDF에서도 각각 33·37·42쪽에 1회씩 있다. PDF·devel·후보 모두 44쪽이다. 34쪽의 표 아래 괘선, 칸 경계, 마지막 행의 위치를 확대해 확인했다. 34쪽의 일부 겹친 글줄은 PDF에서도 겹친다.

- [쪽 소속 증적](../assets/planet6897_20261125/kaf-table-text-ownership.json)
- 34쪽 [review](../assets/planet6897_20261125/kaf-table/native/review_034.png) · [overlay](../assets/planet6897_20261125/kaf-table/native/overlay_034.png)

기존 비교는 `0fbee2c04e`와 통합 후보 `53b15ed46f`의 결과다. #9298 반영 뒤 첨부 PDF에 다시 대조한 Native 44쪽에서 점수가 하락한 쪽은 없었다. 바뀐 31쪽은 표 칸 인라인 그림의 x 위치가 253.1px에서 265.1px로 옮겨가며 일치율이 올랐다. 나머지 43쪽은 PNG와 렌더 트리가 이전 후보와 각각 바이트 단위로 같다. [Native 재측정 기록](../assets/planet6897_20261125/native-rebase-verification.json)

원본과 PDF가 사용하는 휴먼명조 TrueType을 공급한 뒤 Native와 fresh WASM 전체 44쪽을 다시 비교했다. 26쪽 Native는 92.76%에서 99.93%로 회복했다. 전쪽 최저는 24쪽에서 Native 96.88%, WASM 96.93%다. 33~44쪽은 두 출력 모두 98% 이상이다. 원 PR 제출값 대비 남은 차이는 Native 35쪽 99.19→99.18%, WASM 21쪽 99.34→99.23%와 36쪽 98.42→98.38%다. 같은 글꼴을 공급한 리베이스 전후 후보의 Native 35쪽과 WASM 21쪽 PNG는 바이트 단위로 같고, WASM 36쪽은 렌더 트리와 점수가 같았다. [글꼴 공급 및 페이지별 결과](../assets/planet6897_20261125/kaf-font-recheck.json)

## 보정 전 새 MCP 인쇄본으로 전쪽 대조

한컴 2020 `11.0.0.9172`에서 다시 인쇄한 [kaf_table_section-2020.pdf](../../../pdf/issue9000/mcp-11.0.0.9172/kaf_table_section-2020.pdf)로 원 PR과 통합 후보의 44쪽을 각각 대조했다. 같은 96 DPI·compat 2022에서 원 PR 대비 Native 1쪽, fresh WASM 1쪽이 상승했다. 두 경로에 하락은 0쪽이다.

| 98% 미달 쪽 | 통합 후보 Native | 통합 후보 fresh WASM |
| --- | --- | --- |
| 24 | 97.05% | 97.10% |
| 31 | 95.73% | 95.75% |

- 원 PR 전쪽 [Native TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/native/original.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/wasm/original.tsv)
- 통합 후보 전쪽 [Native TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/native/candidate.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/wasm/candidate.tsv)
- fresh WASM 31쪽 [review](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/wasm/review_031.png) · [overlay](../assets/planet6897_20261125/mcp-print-comparison/kaf-table/wasm/overlay_031.png)

[기준 PDF 해시와 집계](../assets/planet6897_20261125/mcp-print-comparison/results.json)

## 메인터너 보정 검증

독립 인라인 글상자에 남은 24쪽의 줄 경계를 보정했다. 저장된 세로 4개 행을 유지하고 축소된
현재 내부 폭을 반영해 ‘법률’과 ‘도로명주소로’가 다음 글줄에 모두 보이게 했다. 새 2020 MCP
Print의 44쪽 검증에서 24쪽은 Native 97.05→100.00%, fresh WASM 97.10→100.00%다.
31쪽의 Native 95.73%는 새 인쇄본의 CID 참조 오류를 포함한 측정이다. 기존 한컴 2020 Print에
같은 후보 출력을 대조한 44쪽은 Native 전쪽 98% 이상이며 31쪽은 98.84%다.
[Native 전쪽 TSV와 31쪽 증적](../assets/planet6897_20261125/kaf-print-repair/README.md#native-전쪽-증적)

## 최초 제출 PDF로 원 PR 수치 재현

원 PR head `f3edd5acaea4`와 통합 후보 `049fd5ed2fa1`를 같은 macOS에서 빌드했다. 같은 원본·한컴 Print PDF·공급 글꼴 순서와 96 DPI, compat 2022로 각각 44쪽을 비교했다. 두 출력 경로 모두 43쪽의 PNG가 바이트 단위로 같다. 31쪽은 Native 97.27→98.84%, fresh WASM 97.28→98.84%로 개선됐다.

제출 TSV보다 낮았던 Native 35쪽 99.18%, fresh WASM 21쪽 99.23%와 36쪽 98.38%는 동일 조건의 원 PR head에서도 같은 표시값과 PNG를 재현했다. 맑은 고딕의 PDF 글리프 윤곽과 일치하는 `MALGUN.TTF`를 앞에 공급해도 Native 35쪽과 fresh WASM 36쪽은 같은 점수였다.

- 같은 조건 전쪽 비교 [Native TSV](../assets/planet6897_20261125/kaf-table/maintainer/native/same-environment-head-comparison.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/kaf-table/maintainer/wasm/same-environment-head-comparison.tsv)
- fresh WASM 21쪽 [review](../assets/planet6897_20261125/kaf-table/maintainer/wasm/review_021.png) · [overlay](../assets/planet6897_20261125/kaf-table/maintainer/wasm/overlay_021.png)

[원 PR·통합 SHA 및 빌드 출처](../assets/planet6897_20261125/review-provenance.json) · [입력·기준 PDF 출처](../assets/planet6897_20261125/print-provenance.json)

## Merge 후 contributor PR comment 계획

[Visual Sweep 게시 기준](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

아래 12쪽을 대조했다. 90% 하한에 걸린 쪽은 0쪽이다.
raw는 2px 이웃 실루엣 판정값이며, 픽셀·내용 픽셀 일치율은 자동 보조값이다.
페이지별 상세 근거의 배치·획·그림 판독을 함께 사용한다.

| 문서·기준·출력 | 쪽 | raw | 픽셀 일치 | 내용 픽셀 일치 | 증적 |
| --- | ---: | ---: | ---: | ---: | --- |
| kaf-table · Hancom PDF 2020 (최초) · Native | 33 | 100.00% | 95.24161% | 46.10788% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_033.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_033.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 34 | 99.97% | 94.80305% | 19.50414% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_034.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_034.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 35 | 99.18% | 97.27928% | 19.83198% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_035.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_035.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 36 | 98.44% | 97.30252% | 18.87996% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_036.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_036.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 37 | 99.97% | 97.55216% | 19.67364% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_037.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_037.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 38 | 99.02% | 97.35786% | 19.04107% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_038.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_038.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 39 | 99.98% | 97.33552% | 20.74193% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_039.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_039.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 40 | 100.00% | 97.44912% | 19.11372% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_040.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_040.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 41 | 99.96% | 96.87866% | 19.50266% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_041.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_041.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 42 | 99.99% | 97.00730% | 18.70407% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_042.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_042.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 43 | 100.00% | 97.95626% | 14.69734% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_043.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_043.png) |
| kaf-table · Hancom PDF 2020 (최초) · Native | 44 | 100.00% | 98.95843% | 24.03602% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-table/review_044.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-table/overlay_044.png) |

병합 SHA를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 넣고
위 링크의 저장소 경로를 이어 review와 overlay를 이미지로 표시한다. devel 반영, raw PNG 형식·해시를
확인한 뒤 `--body-file`로 게시하고 GitHub API에서 본문과 이미지 URL을 대조한다.
