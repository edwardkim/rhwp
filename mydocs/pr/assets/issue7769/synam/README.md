# 국민임대 안내문의 셀 이어받기와 후속 본문

2026-11-23, 96 DPI에서 보정 전 2px 실루엣을 측정했다. 독립 한컴 2020 Print와
동일하게 35쪽을 출력했다. Native 최저는 98.99%, fresh WASM 최저는 99.00%다.

| 쪽 | Native | fresh WASM |
| ---: | ---: | ---: |
| 5 | 98.99% | 99.00% |
| 6 | 99.99% | 100.00% |
| 8 | 100.00% | 100.00% |
| 22 | 100.00% | 100.00% |
| 30 | 99.82% | 99.81% |

5쪽의 금융자산 셀에 있는 ‘비상장주’는 6쪽 같은 셀의 ‘식의 평가에 관하여는’으로
이어진다. 한 쪽에 셀 조각을 중복 출력하지 않는 관계를 검사한다. 8쪽 한점 리더와
22쪽 이어받은 셀 아래 괘선, 30쪽 ‘7. [필수]’ 제목과 동의서 표·서명 날짜의 순서를
독립 Print와 대조했다.

[Native TSV](native.tsv), [fresh WASM TSV](wasm.tsv),
[5→6쪽 확대 비교](pages-05-06-financial-assets.png),
[30쪽 확대 비교](page-30-consent-order.png)

| 자료 | SHA-256 |
| --- | --- |
| `samples/synam-001.hwp` | `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b` |
| `pdf/synam-001-hwp-2020.pdf` | `d199640be90cb39acfde965bbeb66c787d01d179b6759c5d797cf91043c7d09d` |
| Native TSV | `3b82feced7250f5469b9ab2b2d5744ee8fa8223dff28e585aa8a658b461240dc` |
| fresh WASM TSV | `2f2030d519ad6fc7ed0362259a3dced53f50ac964a3c8f997dbc3b4bf4651288` |
