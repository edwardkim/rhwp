# 최종 통합 후보의 전쪽 Native·fresh WASM TSV

- 검증 기준 devel: `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 최종 렌더 source: `4adb7faf48ec055d45e1db5a0bd2160ff6e6b995`. 이후 `626d21cd33`은 Vite 경고 설정, `b9ced69cbd`·`c0722f7921`은 회귀 검사·수집 정책·증적 변경이며 Rust 런타임 구현을 바꾸지 않았다.
- 양쪽 각각 49조건·1772쪽, 독립 Print PDF 쪽수 일치, 누락·측정 불가·90% 미만 없음.
- Native 최저: original-sample11-user-print p50 92.42855%; fresh WASM 최저: 같은 쪽 92.47460%.
- Native 최대 하락 0.46069%p, fresh WASM 최대 하락 0.49098%p. 현행 기준의 2%p 초과 하락은 양쪽 모두 0쪽이다. 진단용 0.1%p 초과 목록은 Native 44쪽 / fresh WASM 29쪽으로 구분해 evidence.json에 보존했다.
- base/candidate·Native/fresh WASM TSV 196개는 생성 원본의 바이트 그대로다. `base_candidate_comparison.tsv`는 십진수 차이의 전쪽 비교표다. 입력·기준 PDF·TSV SHA는 evidence.json에 있다.
- base 엔진의 6097253468 source는 현재 정책 base 9418ed0c7의 Rust 런타임과 동등하다. 그 사이 #8195는 Studio 전용이며 실제 base/후보 브라우저 검증은 별도로 수행했다. 해당 diff·등가성은 output의 rebase-9418-3124-result.json과 검토 기록에 남긴다.
- 각 기준 PDF의 기존 출처·Print 계약을 유지했다. 새 #7518 원본·중첩 표·후속 본문 Print 3종은 pdf/pr7518_review/print-20261024의 provenance를 따른다.
- 원본 p2 및 중첩 표 p4/p5의 대표 review·overlay와 9조건52쪽 양쪽 증거는 [종료 경계 증거](../issue_7500_reflow_closing_band_20261024/README.md)에 있다. 기존 변경 영역의 직접 판독 증거는 각 원 PR 리뷰의 파일·source 연결을 유지한다.
- 실행 로그와 중간 JSON은 output에만 보존한다. 이 전쪽 결과는 전체 Rust·lint·Native Skia 게이트의 통과를 뜻하지 않는다.

## 입력별 TSV

| 조건 | 쪽수 | Native 후보 TSV | fresh WASM 후보 TSV |
| --- | ---: | --- | --- |
| sample16-2022 | 64 | [TSV](native/candidate/sample16-2022_silhouette.tsv) | [TSV](fresh-wasm/candidate/sample16-2022_silhouette.tsv) |
| pii66-declared | 66 | [TSV](native/candidate/pii66-declared_silhouette.tsv) | [TSV](fresh-wasm/candidate/pii66-declared_silhouette.tsv) |
| worklife6 | 6 | [TSV](native/candidate/worklife6_silhouette.tsv) | [TSV](fresh-wasm/candidate/worklife6_silhouette.tsv) |
| asset60 | 60 | [TSV](native/candidate/asset60_silhouette.tsv) | [TSV](fresh-wasm/candidate/asset60_silhouette.tsv) |
| rowbreak18 | 18 | [TSV](native/candidate/rowbreak18_silhouette.tsv) | [TSV](fresh-wasm/candidate/rowbreak18_silhouette.tsv) |
| biotoxin205 | 205 | [TSV](native/candidate/biotoxin205_silhouette.tsv) | [TSV](fresh-wasm/candidate/biotoxin205_silhouette.tsv) |
| regulatory17 | 17 | [TSV](native/candidate/regulatory17_silhouette.tsv) | [TSV](fresh-wasm/candidate/regulatory17_silhouette.tsv) |
| ktx003 | 1 | [TSV](native/candidate/ktx003_silhouette.tsv) | [TSV](fresh-wasm/candidate/ktx003_silhouette.tsv) |
| low-slack6535 | 1 | [TSV](native/candidate/low-slack6535_silhouette.tsv) | [TSV](fresh-wasm/candidate/low-slack6535_silhouette.tsv) |
| press12 | 12 | [TSV](native/candidate/press12_silhouette.tsv) | [TSV](fresh-wasm/candidate/press12_silhouette.tsv) |
| project2097-declared | 2 | [TSV](native/candidate/project2097-declared_silhouette.tsv) | [TSV](fresh-wasm/candidate/project2097-declared_silhouette.tsv) |
| issue265 | 16 | [TSV](native/candidate/issue265_silhouette.tsv) | [TSV](fresh-wasm/candidate/issue265_silhouette.tsv) |
| hwp3-alias | 16 | [TSV](native/candidate/hwp3-alias_silhouette.tsv) | [TSV](fresh-wasm/candidate/hwp3-alias_silhouette.tsv) |
| converted16 | 16 | [TSV](native/candidate/converted16_silhouette.tsv) | [TSV](fresh-wasm/candidate/converted16_silhouette.tsv) |
| tac3 | 3 | [TSV](native/candidate/tac3_silhouette.tsv) | [TSV](fresh-wasm/candidate/tac3_silhouette.tsv) |
| civil2 | 2 | [TSV](native/candidate/civil2_silhouette.tsv) | [TSV](fresh-wasm/candidate/civil2_silhouette.tsv) |
| ole-hwp | 1 | [TSV](native/candidate/ole-hwp_silhouette.tsv) | [TSV](fresh-wasm/candidate/ole-hwp_silhouette.tsv) |
| ole-hwpx | 1 | [TSV](native/candidate/ole-hwpx_silhouette.tsv) | [TSV](fresh-wasm/candidate/ole-hwpx_silhouette.tsv) |
| rnote-cp01 | 3 | [TSV](native/candidate/rnote-cp01_silhouette.tsv) | [TSV](fresh-wasm/candidate/rnote-cp01_silhouette.tsv) |
| rnote-cp02 | 4 | [TSV](native/candidate/rnote-cp02_silhouette.tsv) | [TSV](fresh-wasm/candidate/rnote-cp02_silhouette.tsv) |
| table32 | 1 | [TSV](native/candidate/table32_silhouette.tsv) | [TSV](fresh-wasm/candidate/table32_silhouette.tsv) |
| table33 | 2 | [TSV](native/candidate/table33_silhouette.tsv) | [TSV](fresh-wasm/candidate/table33_silhouette.tsv) |
| table40 | 2 | [TSV](native/candidate/table40_silhouette.tsv) | [TSV](fresh-wasm/candidate/table40_silhouette.tsv) |
| table30 | 2 | [TSV](native/candidate/table30_silhouette.tsv) | [TSV](fresh-wasm/candidate/table30_silhouette.tsv) |
| ole-enter-hwp | 1 | [TSV](native/candidate/ole-enter-hwp_silhouette.tsv) | [TSV](fresh-wasm/candidate/ole-enter-hwp_silhouette.tsv) |
| ole-enter-hwpx | 1 | [TSV](native/candidate/ole-enter-hwpx_silhouette.tsv) | [TSV](fresh-wasm/candidate/ole-enter-hwpx_silhouette.tsv) |
| roadmap302 | 302 | [TSV](native/candidate/roadmap302_silhouette.tsv) | [TSV](fresh-wasm/candidate/roadmap302_silhouette.tsv) |
| disaster | 2 | [TSV](native/candidate/disaster_silhouette.tsv) | [TSV](fresh-wasm/candidate/disaster_silhouette.tsv) |
| empty-cell | 1 | [TSV](native/candidate/empty-cell_silhouette.tsv) | [TSV](fresh-wasm/candidate/empty-cell_silhouette.tsv) |
| land-table | 5 | [TSV](native/candidate/land-table_silhouette.tsv) | [TSV](fresh-wasm/candidate/land-table_silhouette.tsv) |
| ledger | 1 | [TSV](native/candidate/ledger_silhouette.tsv) | [TSV](fresh-wasm/candidate/ledger_silhouette.tsv) |
| lighting | 2 | [TSV](native/candidate/lighting_silhouette.tsv) | [TSV](fresh-wasm/candidate/lighting_silhouette.tsv) |
| none-image-fill | 14 | [TSV](native/candidate/none-image-fill_silhouette.tsv) | [TSV](fresh-wasm/candidate/none-image-fill_silhouette.tsv) |
| yeongwol4 | 4 | [TSV](native/candidate/yeongwol4_silhouette.tsv) | [TSV](fresh-wasm/candidate/yeongwol4_silhouette.tsv) |
| court2 | 2 | [TSV](native/candidate/court2_silhouette.tsv) | [TSV](fresh-wasm/candidate/court2_silhouette.tsv) |
| superscript3 | 3 | [TSV](native/candidate/superscript3_silhouette.tsv) | [TSV](fresh-wasm/candidate/superscript3_silhouette.tsv) |
| field-marker1 | 1 | [TSV](native/candidate/field-marker1_silhouette.tsv) | [TSV](fresh-wasm/candidate/field-marker1_silhouette.tsv) |
| broken-ladder1 | 1 | [TSV](native/candidate/broken-ladder1_silhouette.tsv) | [TSV](fresh-wasm/candidate/broken-ladder1_silhouette.tsv) |
| missing-lineseg1 | 1 | [TSV](native/candidate/missing-lineseg1_silhouette.tsv) | [TSV](fresh-wasm/candidate/missing-lineseg1_silhouette.tsv) |
| quote1 | 1 | [TSV](native/candidate/quote1_silhouette.tsv) | [TSV](fresh-wasm/candidate/quote1_silhouette.tsv) |
| giant4138-original | 115 | [TSV](native/candidate/giant4138-original_silhouette.tsv) | [TSV](fresh-wasm/candidate/giant4138-original_silhouette.tsv) |
| giant4138-tab-period | 197 | [TSV](native/candidate/giant4138-tab-period_silhouette.tsv) | [TSV](fresh-wasm/candidate/giant4138-tab-period_silhouette.tsv) |
| giant4138-current-ascii | 197 | [TSV](native/candidate/giant4138-current-ascii_silhouette.tsv) | [TSV](fresh-wasm/candidate/giant4138-current-ascii_silhouette.tsv) |
| pii66-default | 66 | [TSV](native/candidate/pii66-default_silhouette.tsv) | [TSV](fresh-wasm/candidate/pii66-default_silhouette.tsv) |
| giant4138-hwpx115 | 115 | [TSV](native/candidate/giant4138-hwpx115_silhouette.tsv) | [TSV](fresh-wasm/candidate/giant4138-hwpx115_silhouette.tsv) |
| original-sample16 | 64 | [TSV](native/candidate/original-sample16_silhouette.tsv) | [TSV](fresh-wasm/candidate/original-sample16_silhouette.tsv) |
| original-sample11-user-print | 151 | [TSV](native/candidate/original-sample11-user-print_silhouette.tsv) | [TSV](fresh-wasm/candidate/original-sample11-user-print_silhouette.tsv) |
| new-upstream-field01 | 3 | [TSV](native/candidate/new-upstream-field01_silhouette.tsv) | [TSV](fresh-wasm/candidate/new-upstream-field01_silhouette.tsv) |
| receipt7017 | 1 | [TSV](native/candidate/receipt7017_silhouette.tsv) | [TSV](fresh-wasm/candidate/receipt7017_silhouette.tsv) |
