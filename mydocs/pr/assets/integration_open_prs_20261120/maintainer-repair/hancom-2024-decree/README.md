# 가상융합산업 진흥법 시행령 한컴 2024 인쇄 대조

제품 소스 `3901aa782d4445efb171ec4ffb026f2a459c2daf`에서 같은 원본의 한컴 2024
(`13.0.0.3901`) Hancom PDF 인쇄본에 Native Skia와 최적화한 fresh WASM을 대조했다.
96 DPI, 2px 관용 raw 실루엣 기준으로 두 경로 모두 74쪽 전체가 90% 이상이다.
[Native 실행 기록](native/evidence.json) · [Native TSV](native/silhouette.tsv) ·
[WASM 실행 기록](wasm/evidence.json) · [WASM TSV](wasm/silhouette.tsv).

| 쪽 | Native raw | WASM raw | Native 증적 | WASM 증적 |
| --- | ---: | ---: | --- | --- |
| 4 | 90.59% | 90.51% | [review](native/review-004.png) · [overlay](native/overlay-004.png) | [review](wasm/review-004.png) · [overlay](wasm/overlay-004.png) |
| 7 | 100.00% | 100.00% | [review](native/review-007.png) · [overlay](native/overlay-007.png) | [review](wasm/review-007.png) · [overlay](wasm/overlay-007.png) |
| 16 | 99.99% | 99.99% | [review](native/review-016.png) · [overlay](native/overlay-016.png) | [review](wasm/review-016.png) · [overlay](wasm/overlay-016.png) |

7쪽의 문단 끝 ‘당’과 16쪽의 ‘링’은 기준 인쇄본과 같은 줄에 표시된다.
최저값을 기록한 4쪽은 조문 대비표의 개정안 셀에 일부 줄바꿈 차이가 남는다.
표의 행·셀 소속과 본문 순서, 전체 쪽수는 유지됐다.
