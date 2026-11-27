# 이슈 #7777–#7787 인쇄 검증

후보 `930e6f77a5433d6b254969acda078cd73f8b5597`에서 독립 한컴 Print PDF를 96 DPI로 대조했다. 표에는 색상 경계 보정 전 2px 실루엣의 전쪽 최저값을 적었다.

| 대상 | 쪽수 | Native 최저 | fresh WASM 최저 | 대표 확대 비교 |
| --- | ---: | ---: | ---: | --- |
| 7777-37787 | 49 | [99.21%](native/7777-37787/silhouette.tsv) | [99.14%](wasm/7777-37787/silhouette.tsv) | 14쪽 [Native](native/7777-37787/review_014.png) · [WASM](wasm/7777-37787/review_014.png) / 27쪽 [Native](native/7777-37787/review_027.png) · [WASM](wasm/7777-37787/review_027.png) / 28쪽 [Native](native/7777-37787/review_028.png) · [WASM](wasm/7777-37787/review_028.png) |
| 7778-2022-print | 37 | [98.48%](native/7778-2022-print/silhouette.tsv) | [98.44%](wasm/7778-2022-print/silhouette.tsv) | — |
| 7778-2024-print | 37 | [98.16%](native/7778-2024-print/silhouette.tsv) | [98.16%](wasm/7778-2024-print/silhouette.tsv) | 10쪽 [Native](native/7778-2024-print/review_010.png) · [WASM](wasm/7778-2024-print/review_010.png) / 36쪽 [Native](native/7778-2024-print/review_036.png) · [WASM](wasm/7778-2024-print/review_036.png) / 37쪽 [Native](native/7778-2024-print/review_037.png) · [WASM](wasm/7778-2024-print/review_037.png) |
| 7779-76076-hwpx | 82 | [98.52%](native/7779-76076-hwpx/silhouette.tsv) | [98.53%](wasm/7779-76076-hwpx/silhouette.tsv) | 6쪽 [Native](native/7779-76076-hwpx/review_006.png) · [WASM](wasm/7779-76076-hwpx/review_006.png) / 7쪽 [Native](native/7779-76076-hwpx/review_007.png) · [WASM](wasm/7779-76076-hwpx/review_007.png) / 33쪽 [Native](native/7779-76076-hwpx/review_033.png) · [WASM](wasm/7779-76076-hwpx/review_033.png) |
| 7784-76076-hwp | 82 | [98.91%](native/7784-76076-hwp/silhouette.tsv) | [98.94%](wasm/7784-76076-hwp/silhouette.tsv) | 33쪽 [Native](native/7784-76076-hwp/review_033.png) · [WASM](wasm/7784-76076-hwp/review_033.png) / 34쪽 [Native](native/7784-76076-hwp/review_034.png) · [WASM](wasm/7784-76076-hwp/review_034.png) / 38쪽 [Native](native/7784-76076-hwp/review_038.png) · [WASM](wasm/7784-76076-hwp/review_038.png) |
| 7785-hml | 1 | [100.00%](native/7785-hml/silhouette.tsv) | [100.00%](wasm/7785-hml/silhouette.tsv) | 1쪽 [Native](native/7785-hml/review_001.png) · [WASM](wasm/7785-hml/review_001.png) |
| 7785-hwp | 1 | [100.00%](native/7785-hwp/silhouette.tsv) | [100.00%](wasm/7785-hwp/silhouette.tsv) | — |
| 7785-hwpx | 1 | [100.00%](native/7785-hwpx/silhouette.tsv) | [100.00%](wasm/7785-hwpx/silhouette.tsv) | — |
| 7786-ktx | 27 | [100.00%](native/7786-ktx/silhouette.tsv) | [99.87%](wasm/7786-ktx/silhouette.tsv) | 2쪽 [Native](native/7786-ktx/review_002.png) · [WASM](wasm/7786-ktx/review_002.png) |
| 7787-bottom | 1 | [100.00%](native/7787-bottom/silhouette.tsv) | [100.00%](wasm/7787-bottom/silhouette.tsv) | — |
| 7787-center | 1 | [100.00%](native/7787-center/silhouette.tsv) | [100.00%](wasm/7787-center/silhouette.tsv) | — |
| 7787-top | 1 | [98.27%](native/7787-top/silhouette.tsv) | [98.27%](wasm/7787-top/silhouette.tsv) | 1쪽 [Native](native/7787-top/review_001.png) · [WASM](wasm/7787-top/review_001.png) |

[Native 매니페스트](native/manifest.json)와 [fresh WASM 매니페스트](wasm/manifest.json)에 원본·독립 PDF 경로, TSV와 대표 PNG의 SHA-256을 기록했다.

KTX는 독립 한컴 2022 Print도 대조했다. 전 27쪽 최저값은 [Native 100.00%](ktx-2022/native.tsv), [fresh WASM 99.87%](ktx-2022/wasm.tsv)였다. 두 출력의 2쪽 목차 번호 17개는 독립 PDF와 순서와 셀 소속이 일치했고 오른쪽 끝 차이는 모두 0.058px 이내였다.

[보호 문서 전쪽 대조](protected-comparison.tsv)는 경로별 2,023쪽의 정본·후보 값을 담았다. [실행 SHA-256과 집계](protected-validation.json)에서 검증 대상을 확인할 수 있다.
