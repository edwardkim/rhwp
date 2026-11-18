# PR #8389/#8397 보호 입력의 최종 Native 검증

제품 source `0261687265f3a10a64e51809d006b2305a36d950`, Native Skia binary SHA-256 `968f935658b308dab4a443ea0f5e5e8723c2e24017fa6b5b71f533da50a8d598`. 96 DPI·print profile·2px raw 실루엣으로 17조합·1,770쪽을 실제 검증했다. 전쪽 90% 이상과 독립 Print 쪽수 일치를 확인했다. 원본 TSV의 바이트는 바꾸지 않았다.

| 입력 키 | 전쪽 | Native 최저 raw | 원본 TSV |
| --- | ---: | ---: | --- |
| sample16-hwp2022 | 64 | 100.00% | [전쪽 TSV](candidate/sample16-hwp2022/full.tsv) |
| sample16-hwp2024 | 64 | 99.99% | [전쪽 TSV](candidate/sample16-hwp2024/full.tsv) |
| sample16-hwpx | 64 | 99.99% | [전쪽 TSV](candidate/sample16-hwpx/full.tsv) |
| issue7905 | 6 | 98.86% | [전쪽 TSV](candidate/issue7905/full.tsv) |
| issue8040-insert | 4 | 99.99% | [전쪽 TSV](candidate/issue8040-insert/full.tsv) |
| issue8040-insert-delete | 4 | 99.99% | [전쪽 TSV](candidate/issue8040-insert-delete/full.tsv) |
| handbook-hwp | 383 | 99.96% | [전쪽 TSV](candidate/handbook-hwp/full.tsv) |
| seo-body-list-typed | 1 | 100.00% | [전쪽 TSV](candidate/seo-body-list-typed/full.tsv) |
| seo-helvetica-styles | 1 | 100.00% | [전쪽 TSV](candidate/seo-helvetica-styles/full.tsv) |
| seo-pic-crop-01 | 1 | 100.00% | [전쪽 TSV](candidate/seo-pic-crop-01/full.tsv) |
| seo-shape-group-02 | 1 | 100.00% | [전쪽 TSV](candidate/seo-shape-group-02/full.tsv) |
| seo-re-03-latin | 1 | 100.00% | [전쪽 TSV](candidate/seo-re-03-latin/full.tsv) |
| yeongwol | 4 | 99.97% | [전쪽 TSV](candidate/yeongwol/full.tsv) |
| outline | 9 | 99.99% | [전쪽 TSV](candidate/outline/full.tsv) |
| nested | 17 | 94.96% | [전쪽 TSV](candidate/nested/full.tsv) |
| sample10 | 763 | 99.41% | [전쪽 TSV](candidate/sample10/full.tsv) |
| handbook-hwpx | 383 | 99.96% | [전쪽 TSV](candidate/handbook-hwpx/full.tsv) |

이 문서는 Native 실행 결과다. fresh WASM과 최신 base의 실제 대조 결과는 최종 공통 증적에서 별도로 확인한다. 기존 결과를 현재 WASM의 새 실행으로 표시하지 않는다.
