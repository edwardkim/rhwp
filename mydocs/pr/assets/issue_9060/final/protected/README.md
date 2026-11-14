# 최종 전쪽 보호 검증 원본

Native·fresh WASM 각 18개 입력·1,771쪽, 총 36개 데이터셋·3,542쪽을 같은 조건으로 대조했다.
기존 정본 대비 **하락 0쪽·상승 30쪽(Native 15·WASM 15)**이다.
제품 소스는 `43adf3f7fe412e71797456887e2be44b8ef8d14f`다.

[수용 조건 계약](../protected-validation.json)은 정본 보호 baseline의 관리 대상 manifest다.
입력/PDF·Native binary·WASM package의 SHA, 실제 글꼴 공급·명시 환경·raster 조건과
전체 TSV SHA·범위·최저값·필수 게이트 결과를 연결한다. 중간 run manifest·원시 로그는 포함하지 않는다.

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

각 입력·backend의 전체 결과는 같은 최종 실행에서 선택했다. 페이지별 최댓값을 이어 붙이지 않았다.
편람 HWPX Native 383쪽은 별도 실제 `--handbook-gate` 실행의 전체 TSV를 같은 입력의 보호 대조에도 재사용했다.
gate는 1,801.002초에 종료 코드 0으로 통과했고 공식 최저값은 99.96%다.
기존 WMF 차이는 숨기지 않으며 전쪽 범위에 포함한다.
원시 실행 로그·분석 JSON은 ignored output에서만 관리한다.
원격 CI·통합 PR·병합 완료를 뜻하지 않는다.
