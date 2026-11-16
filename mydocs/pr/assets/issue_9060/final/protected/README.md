# 최종 전쪽 보호 검증 원본

제품 source: `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`. Native·fresh WASM 각각 18개 입력·1,771쪽, 총 36개 데이터셋·3,542행을 같은 조건으로 대조했다.
상향된 정본 `8f073dddd67cf8f44bea7c0315c3f9950ba73641e9393747cdbc52d24b33f9a1` 대비 하락 0행·누락 0행·추가 상승 0행이다. 앞서 수용한 30행의 상향 하한을 유지한다.

[수용 조건 계약](../protected-validation.json)에 입력·독립 PDF·생산 프로그램·실제 글꼴 공급·계산 조건과 전쪽 TSV 해시를 연결했다. 중간 run manifest와 원시 로그는 제출하지 않는다.

| 입력 식별자 | 쪽수 | Native 최저 | fresh WASM 최저 | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| `angle-narrow` | 1 | 100.00% | 100.00% | [Native](native/angle-narrow.tsv) · [WASM](wasm/angle-narrow.tsv) |
| `handbook-hwp` | 383 | 99.96% | 99.96% | [Native](native/handbook-hwp.tsv) · [WASM](wasm/handbook-hwp.tsv) |
| `handbook-hwpx` | 383 | 99.96% | 99.96% | [Native](native/handbook-hwpx.tsv) · [WASM](wasm/handbook-hwpx.tsv) |
| `issue7905` | 6 | 98.86% | 98.86% | [Native](native/issue7905.tsv) · [WASM](wasm/issue7905.tsv) |
| `issue8040-insert` | 4 | 99.99% | 99.99% | [Native](native/issue8040-insert.tsv) · [WASM](wasm/issue8040-insert.tsv) |
| `issue8040-insert-delete` | 4 | 99.99% | 99.99% | [Native](native/issue8040-insert-delete.tsv) · [WASM](wasm/issue8040-insert-delete.tsv) |
| `nested` | 17 | 98.59% | 98.59% | [Native](native/nested.tsv) · [WASM](wasm/nested.tsv) |
| `outline` | 9 | 100.00% | 100.00% | [Native](native/outline.tsv) · [WASM](wasm/outline.tsv) |
| `sample10` | 763 | 99.57% | 99.57% | [Native](native/sample10.tsv) · [WASM](wasm/sample10.tsv) |
| `sample16-hwp2022` | 64 | 100.00% | 100.00% | [Native](native/sample16-hwp2022.tsv) · [WASM](wasm/sample16-hwp2022.tsv) |
| `sample16-hwp2024` | 64 | 99.99% | 99.99% | [Native](native/sample16-hwp2024.tsv) · [WASM](wasm/sample16-hwp2024.tsv) |
| `sample16-hwpx` | 64 | 99.99% | 99.99% | [Native](native/sample16-hwpx.tsv) · [WASM](wasm/sample16-hwpx.tsv) |
| `seo-body-list-typed` | 1 | 100.00% | 100.00% | [Native](native/seo-body-list-typed.tsv) · [WASM](wasm/seo-body-list-typed.tsv) |
| `seo-helvetica-styles` | 1 | 100.00% | 100.00% | [Native](native/seo-helvetica-styles.tsv) · [WASM](wasm/seo-helvetica-styles.tsv) |
| `seo-pic-crop-01` | 1 | 100.00% | 100.00% | [Native](native/seo-pic-crop-01.tsv) · [WASM](wasm/seo-pic-crop-01.tsv) |
| `seo-re-03-latin` | 1 | 100.00% | 100.00% | [Native](native/seo-re-03-latin.tsv) · [WASM](wasm/seo-re-03-latin.tsv) |
| `seo-shape-group-02` | 1 | 100.00% | 100.00% | [Native](native/seo-shape-group-02.tsv) · [WASM](wasm/seo-shape-group-02.tsv) |
| `yeongwol` | 4 | 100.00% | 100.00% | [Native](native/yeongwol.tsv) · [WASM](wasm/yeongwol.tsv) |

각 입력·backend의 전체 결과는 같은 최종 실행에서 선택했다. 페이지별 최댓값을 조합하지 않았다. 편람 HWPX Native 383쪽은 실제 `--handbook-gate` 실행 결과이며 종료 코드 0이다. 130쪽의 기존 WMF 차이를 포함하고 마스킹하지 않는다.

공식 두 자리 raw는 2px 내용 실루엣의 보호 판정값이며 RGB 픽셀 동일성이나 모든 기존 그래픽 차이의 해결을 뜻하지 않는다. 원격 CI·통합 PR·병합 결과는 PR review에서 별도로 기록한다.
