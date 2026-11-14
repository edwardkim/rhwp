# 최종 전쪽 보호 검증 원본

[검증 기록](../../verification-record.md#보호-전쪽-검증과-기준-상향)을 따른다.
최종 fresh WASM은 18개 입력·1,771쪽 모두 완료했고 기존 보호값 대비 하락 0쪽·상승 15쪽이다.
제품 소스는 `43adf3f7fe412e71797456887e2be44b8ef8d14f`다.
각 입력마다 같은 최종 실행의 전체 TSV를 보존하며 페이지별 최댓값을 조합하지 않았다.

| 입력 식별자 | 쪽수 | fresh WASM 최저 | 전쪽 TSV |
| --- | ---: | ---: | --- |
| `angle-narrow` | 1 | 100.00% | [TSV](wasm/angle-narrow.tsv) |
| `handbook-hwp` | 383 | 99.96% | [TSV](wasm/handbook-hwp.tsv) |
| `handbook-hwpx` | 383 | 99.96% | [TSV](wasm/handbook-hwpx.tsv) |
| `issue7905` | 6 | 98.86% | [TSV](wasm/issue7905.tsv) |
| `issue8040-insert` | 4 | 99.99% | [TSV](wasm/issue8040-insert.tsv) |
| `issue8040-insert-delete` | 4 | 99.99% | [TSV](wasm/issue8040-insert-delete.tsv) |
| `nested` | 17 | 98.59% | [TSV](wasm/nested.tsv) |
| `outline` | 9 | 100.00% | [TSV](wasm/outline.tsv) |
| `sample10` | 763 | 99.57% | [TSV](wasm/sample10.tsv) |
| `sample16-hwp2022` | 64 | 100.00% | [TSV](wasm/sample16-hwp2022.tsv) |
| `sample16-hwp2024` | 64 | 99.99% | [TSV](wasm/sample16-hwp2024.tsv) |
| `sample16-hwpx` | 64 | 99.99% | [TSV](wasm/sample16-hwpx.tsv) |
| `seo-body-list-typed` | 1 | 100.00% | [TSV](wasm/seo-body-list-typed.tsv) |
| `seo-helvetica-styles` | 1 | 100.00% | [TSV](wasm/seo-helvetica-styles.tsv) |
| `seo-pic-crop-01` | 1 | 100.00% | [TSV](wasm/seo-pic-crop-01.tsv) |
| `seo-re-03-latin` | 1 | 100.00% | [TSV](wasm/seo-re-03-latin.tsv) |
| `seo-shape-group-02` | 1 | 100.00% | [TSV](wasm/seo-shape-group-02.tsv) |
| `yeongwol` | 4 | 100.00% | [TSV](wasm/yeongwol.tsv) |

Native CLI는 WASM의 보조 메타데이터에만 사용했다. 실제 PNG는 새 WASM에서 출력했다.
새 Native 빌드·전쪽 측정은 별도 실행 중이며 위 WASM 측정으로 대신하지 않는다.
양쪽과 필수 검증이 완료되면 정본 보호 baseline의 수용 계약·manifest·상향 영수증을 함께 반영한다.
원시 run manifest·로그는 제출하지 않는다. 원격 CI·통합 PR·병합 완료를 뜻하지 않는다.
