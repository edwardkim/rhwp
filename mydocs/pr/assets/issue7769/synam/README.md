# 금융자산 설명 행의 5→6쪽 이어받기

2026-11-23, 96 DPI에서 보정 전 2px 실루엣을 측정했다. 독립 한컴 2020 Print와
동일한 35쪽을 출력했다. Native 최저는 95.84%, fresh WASM 최저는 96.05%다.

| 쪽 | Native | fresh WASM |
| ---: | ---: | ---: |
| 5 | 98.72% | 98.73% |
| 6 | 99.99% | 100.00% |
| 8 | 95.84% | 96.05% |
| 22 | 97.68% | 97.56% |

5쪽의 금융자산 셀에 있는 “비상장주”는 6쪽 같은 셀의 “식의 평가에 관하여는”으로
이어진다. 해당 내용을 줄 단위로 확대 대조했으며 한 쪽에서 셀 조각을 중복 출력하지
않는 관계를 검사한다. 8쪽과 22쪽의 차이는 별도로 조사 중이다.

[Native 전쪽 TSV](native.tsv), [fresh WASM 전쪽 TSV](wasm.tsv),
[5→6쪽 확대 비교](pages-05-06-financial-assets.png)

| 자료 | SHA-256 |
| --- | --- |
| `samples/synam-001.hwp` | `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b` |
| `pdf/synam-001-hwp-2020.pdf` | `d199640be90cb39acfde965bbeb66c787d01d179b6759c5d797cf91043c7d09d` |
| Native 전쪽 TSV | `cf99249a4ae0ed1274392d7d6fd6891da68ff8e276a96d9440d205b7ea771016` |
| fresh WASM 전쪽 TSV | `87744bb61e25ddcade7db897da5771a5c835aeed205e8ce9fc810bbb33b688dd` |
