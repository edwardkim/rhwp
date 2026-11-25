# PR #9400 검토 — KAF 부록의 WMF 자르기와 글 상자 줄 높이

## 최종 판정

메인터너 보정 뒤 새 MCP 인쇄본 기준 24·36쪽은 Native·fresh WASM 모두 **100.00%**, 38쪽은
**100.00%·99.93%**다. 후보에서 기록한 글꼴 순서로 원 PR을 다시 실행해 Native 54쪽 하락
0쪽을 확인했다. 같은 조건의 fresh WASM 54쪽도 하락 0쪽이다. 31쪽에서 새 MCP PDF의 글꼴
자원 오류를 재현했으며, 유효한 최초 한컴 2020 Print에 대조한 전쪽 최저는 Native
98.23%·fresh WASM 98.22%다.
[최종 후보의 두 Print 기준과 증적](../assets/planet6897_20261125/kaf-print-repair/README.md)

[원 PR #9400](https://github.com/edwardkim/rhwp/pull/9400) · [이슈 #9379](https://github.com/edwardkim/rhwp/issues/9379) · [글 상자 높이 후속 이슈 #9395](https://github.com/edwardkim/rhwp/issues/9395)

KAF 회의자료의 33~44쪽 표를 제거한 54쪽 파생본에 두 계약을 적용한다. imgDim이 없고 저장 자르기가 WMF 원래 범위 전체와 같은 그림은 화면·인쇄에서 전체 범위를 사용한다. 부분 자르기는 기존 창을 유지한다. 글 상자는 bit 22, 확인된 글꼴의 세로 상자, 저장 줄·간격을 함께 확인하고 구형 저장본의 줄 피치만 보정한다.

원 PR의 36쪽은 Native 91.77%·WASM 92.09%이며, 한컴에서 상자가 내용에 맞춰 늘어나는 차이와 첫 줄의 `such` 줄바꿈 차이가 남아 있다. 기하 차이는 #9395에서 추적 중이다. 줄 피치 테스트는 원 출력의 3243 HU를 96 DPI에서 43.24±0.5px로 비교한다. 이 값은 줄 사이 상대 거리의 고정 수치이며, 36쪽의 페이지 일치율과 적용 가능한 회귀 조건을 함께 확인해야 한다.

첨부 PDF는 PDF 1.4, A4 가로 54쪽이다. `Creator`에 표시된 `Hwp 2018 11.0.0.1623`은 저장소에서 한컴 2020으로 실측한 빌드다. 이전 MCP 2020 Print PDF의 Creator·Producer도 각각 `Hwp 2018 11.0.0.1623`과 `Hancom PDF 1.3.0.404`다. 원 PR의 동일 파생본에 대한 Hancom PDF Print 기록과 대조해 첨부 PDF를 재사용했다. 66쪽 전체 원본의 MCP 재출력은 PDF 검증 단계에서 실패했다. [제품 버전 실측](../../manual/verification/hangul_version_oracle.md#51-설치된-버전-확인) · [MCP 대조 기록](pr_9317_review.md#입력과-독립-인쇄-기준)

갱신된 MCP에서 같은 원본을 한컴 2020으로 다시 출력했다. client 0.10.0에서 `start --engine 2020 --target pdf --timeout-seconds 1800`을 호출하고, `status`가 성공한 뒤 같은 job을 `download`로 받았다. `status.result`와 `download.server`의 `pdf_output_mode`는 `hancom2020_pdf_driver_one_up`, `pdf_print_method`는 `0`이다. MCP는 PDF 글꼴 이름의 공백 문법을 보정하고 전체 54쪽을 확인했다. 새 한컴 빌드 `11.0.0.9172`의 인쇄본을 원 PR 인쇄본과 96 DPI로 비교하면 표시값이 100.00% 미만인 쪽이 23쪽이다. 기존 인쇄본으로 최초 제출 수치를 재현했다. 벤치마크의 인쇄 조건을 통일하기 위해 원 PR과 통합 후보 양쪽을 이번 MCP 인쇄본에 전쪽 대조했다. [두 인쇄본 대조 및 MCP 영수증](../assets/planet6897_20261125/mcp-print-reissue.json) · [MCP 기준 PDF 인쇄 계약](../../manual/mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)

## 글 상자의 셋째 줄 잘림

36쪽의 Native 원본 PNG를 직접 대조했다. devel에서는 셋째 줄 `Cadastre’` 전체가 상자 안에 보인다. 후보에서 피치 보정으로 셋째 줄 상단이 y=245.1에서 265.1로 내려가면서, 저장 크기로 고정된 상자가 글자 하단을 자른다. 첨부 PDF의 상자는 더 높고 셋째 줄 전체가 보인다. 줄 피치 계산 개선과 별개로 실제 보이는 내용이 줄어든다. 전체 자르기 WMF 38·39·52쪽은 같은 SVG 경로에서 이미지 1개를 유지한 채 자르기 창이 1개에서 0개로 줄었고, 부분 자르기 36쪽은 창 1개를 유지했다.

메인터너 보정 `049fd5ed2f`는 열기 단계에서 글꼴 세로 치수를 실제로 복원한 독립 가로 글 상자에서 글줄 끝과 아래 안 여백을 테두리·인쇄 경계에 반영한다. 36쪽 SVG에서 사각형 원점과 폭을 유지하면서 인쇄 경계 높이가 104.73px에서 116.64px로 늘어났다. 마지막 글줄과 아래 안 여백이 인쇄 경계 안에 있는지 확인하는 관계형 회귀를 추가했다.

같은 첨부 PDF로 재측정한 Native 36쪽은 91.58%에서 95.13%로 올랐다. 셋째 줄 전체가 보이며 아래 본문의 시작 위치도 유지됐다. 상자 안의 `such`와 `and`의 줄 소속은 PDF와 다르다. Native 전쪽 54쪽의 리베이스 전 후보 대비 하락은 0쪽이며, 38·39·40·42·52쪽은 각각 97.86%·100.00%·100.00%·100.00%·100.00%로 유지됐다.

- 보정 뒤 36쪽 Native [review](../assets/planet6897_20261125/kaf-appendix/maintainer/native/review_036.png) · [overlay](../assets/planet6897_20261125/kaf-appendix/maintainer/native/overlay_036.png) · [54쪽 TSV](../assets/planet6897_20261125/kaf-appendix/maintainer/native/silhouette.tsv)
- 보정 뒤 36쪽 fresh WASM [review](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/review_036.png) · [overlay](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/overlay_036.png) · [최종 글꼴 공급 54쪽 TSV](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/silhouette-verified-hmkmm.tsv)

- 36쪽 [devel PNG](../assets/planet6897_20261125/kaf-appendix/native/base-036.png) · [후보 PNG](../assets/planet6897_20261125/kaf-appendix/native/candidate-036.png) · [review](../assets/planet6897_20261125/kaf-appendix/native/review_036.png) · [overlay](../assets/planet6897_20261125/kaf-appendix/native/overlay_036.png)
- 38쪽 [review](../assets/planet6897_20261125/kaf-appendix/native/review_038.png) · [overlay](../assets/planet6897_20261125/kaf-appendix/native/overlay_038.png)
- [출력 위치 증적](../assets/planet6897_20261125/claim-native-structural-evidence.json) · [WMF 자르기 창 확인](../assets/planet6897_20261125/kaf-appendix-svg-consumer.json)

기존 비교는 `0fbee2c04e`와 통합 후보 `53b15ed46f`의 결과다. #9298이 반영된 `fb98e8bff3` 위로 충돌 없이 리베이스했으며, 기존 PDF와 PNG를 같은 입력에서의 비교 자료로 사용했다.

36쪽은 같은 Mac의 이전 후보 대비 fresh WASM 91.59→95.03%로 올랐다. 휴먼명조 TrueType을 공급한 Native·WASM 전체 54쪽 비교에서 원 PR 제출값보다 낮은 값이 남은 곳은 Native 38쪽(−0.01%p), WASM 21쪽(−0.11%p), 38·39쪽(각 −0.02%p)이다. WASM 38·39쪽 PNG는 리베이스 전 후보와 같다. 전체 Canvas 내보내기에서 윤곽 글꼴 세션 한도에 닿아, 같은 입력·글꼴·WASM으로 6개 세션에서 직접 출력했다. 54쪽의 빠진 쪽·중복을 검사한 후 Visual Sweep으로 전쪽 TSV를 계산했다. [페이지별 결과](../assets/planet6897_20261125/kaf-font-recheck.json)

## 보정 전 새 MCP 인쇄본으로 전쪽 대조

한컴 2020 `11.0.0.9172`에서 다시 인쇄한 [kaf_appendix-2020.pdf](../../../pdf/issue9379/mcp-11.0.0.9172/kaf_appendix-2020.pdf)로 원 PR과 통합 후보의 54쪽을 각각 대조했다. 같은 96 DPI·compat 2022에서 원 PR 대비 Native 2쪽, fresh WASM 2쪽이 상승했다. 두 경로에 하락은 0쪽이다.

| 98% 미달 쪽 | 통합 후보 Native | 통합 후보 fresh WASM |
| --- | --- | --- |
| 24 | 97.05% | 97.10% |
| 31 | 95.73% | 95.75% |
| 36 | 94.65% | 94.39% |
| 38 | 96.01% | 96.01% |

- 원 PR 전쪽 [Native TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/native/original.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/original.tsv)
- 통합 후보 전쪽 [Native TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/native/candidate.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/candidate.tsv)
- fresh WASM 36쪽 [review](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/review_036.png) · [overlay](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/overlay_036.png)
- fresh WASM 38쪽 [review](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/review_038.png) · [overlay](../assets/planet6897_20261125/mcp-print-comparison/kaf-appendix/wasm/overlay_038.png)

[기준 PDF 해시와 집계](../assets/planet6897_20261125/mcp-print-comparison/results.json)

## 메인터너 보정 검증

36쪽에서는 늘어난 독립 글상자의 현재 폭으로 줄 소속을 다시 계산하고 보존된 세로 피치를 유지한다.
둘째 줄의 `such as`와 셋째 줄의 `and Cadastre` 및 테두리 아래 본문이 새 인쇄본과 같은 위치에 있다.
38쪽의 순수 단일 DIB는 원점에서 논리 창의 마지막 픽셀까지 포함하므로 전체 인쇄 틀로 재생한다.
부분 자르기와 벡터·다중 그림이 합성된 WMF는 기존 자르기 경계를 사용한다. 테스트는 24쪽의 글줄
소속과 38쪽의 안쪽 DIB 틀이 원본 바깥 인쇄 틀과 같은 관계를 검사한다.

| 쪽 | 새 2020 Print Native | 새 2020 Print fresh WASM | 대조 범위 |
| --- | --- | --- | --- |
| 24 | 97.05→100.00% | 97.10→100.00% | 축소된 인라인 글상자의 4개 줄 |
| 36 | 94.65→100.00% | 94.39→100.00% | 셋째 줄과 상자 안 여백 |
| 38 | 96.01→100.00% | 96.01→99.93% | 단일 DIB 크기와 클립 |

39쪽의 그림 폭은 신규 인쇄본 410.19pt, 기존 인쇄본 408.482pt로 빌드별 차이가 있다.
후보 Native는 각각 100.00%·98.23%이며 기존 인쇄본의 보정 전 점수는 100.00%였다.
31쪽의 신규 Print 비교와 유효한 기존 Print의 전쪽 비교를 함께 남겼다.
[해시 및 24·36·38쪽 review·overlay](../assets/planet6897_20261125/kaf-print-repair/README.md)

## 최초 제출 PDF로 원 PR 수치 재현

원 PR head `d53cf0751b`와 통합 후보 `049fd5ed2fa1`를 같은 macOS에서 빌드했다. 같은 원본·한컴 Print PDF·공급 글꼴 순서와 96 DPI, compat 2022로 각각 54쪽을 비교했다. Native는 2쪽, fresh WASM은 2쪽이 개선됐다. 메인터너 보정을 적용한 36쪽은 Native 91.58→95.13%, fresh WASM 91.59→95.03%다.

제출 TSV보다 낮았던 Native 38쪽 97.86%와 fresh WASM 21·38·39쪽 99.23%·97.85%·99.98%는 동일 조건의 원 PR head에서도 같은 점수와 PNG를 재현했다. 제출된 38·39쪽 review의 PDF 영역은 이번 PDF 래스터와 픽셀이 같다. Native 38쪽은 제출 이미지와 비교해 WMF 프레임 내부 픽셀이 같았고, fresh WASM 39쪽은 차이가 주로 그림 프레임에 있었다. 제출자의 구체적인 실행 환경은 확인되지 않았다.

- 같은 조건 전쪽 비교 [Native TSV](../assets/planet6897_20261125/kaf-appendix/maintainer/native/same-environment-head-comparison.tsv) · [fresh WASM TSV](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/same-environment-head-comparison.tsv) · [양쪽 head와 PNG 대조 기록](../assets/planet6897_20261125/same-environment-head-comparison.json)
- fresh WASM 38쪽 [review](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/review_038.png) · [overlay](../assets/planet6897_20261125/kaf-appendix/maintainer/wasm/overlay_038.png)

[원 PR·통합 SHA 및 빌드 출처](../assets/planet6897_20261125/review-provenance.json) · [입력·기준 PDF 출처](../assets/planet6897_20261125/print-provenance.json)

## Merge 후 contributor PR comment 계획

[Visual Sweep 게시 기준](../../manual/verification/visual_sweep_guide.md#github-merge-comment)

아래 6쪽을 대조했다. 90% 하한에 걸린 쪽은 0쪽이다.
raw는 2px 이웃 실루엣 판정값이며, 픽셀·내용 픽셀 일치율은 자동 보조값이다.
페이지별 상세 근거의 배치·획·그림 판독을 함께 사용한다.

| 문서·기준·출력 | 쪽 | raw | 픽셀 일치 | 내용 픽셀 일치 | 증적 |
| --- | ---: | ---: | ---: | ---: | --- |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 36 | 100.00% | 97.50446% | 43.45592% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_036.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_036.png) |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 38 | 99.93% | 99.74474% | 96.42672% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_038.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_038.png) |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 39 | 99.97% | 99.55538% | 85.41605% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_039.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_039.png) |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 40 | 100.00% | 96.18900% | 24.07246% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_040.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_040.png) |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 42 | 100.00% | 97.21463% | 38.62478% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_042.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_042.png) |
| kaf-appendix · Hancom PDF 2020 (MCP 재출력) · fresh WASM | 52 | 100.00% | 98.51897% | 84.67044% | [review](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/review_052.png) · [overlay](../assets/planet6897_20261125/final-claim-pages/kaf-appendix/overlay_052.png) |

병합 SHA를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/`에 넣고
위 링크의 저장소 경로를 이어 review와 overlay를 이미지로 표시한다. devel 반영, raw PNG 형식·해시를
확인한 뒤 `--body-file`로 게시하고 GitHub API에서 본문과 이미지 URL을 대조한다.
