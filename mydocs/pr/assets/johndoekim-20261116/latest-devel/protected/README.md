# 최신 devel 특별 보호 검증

실제 source `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`, Native/fresh WASM 각각 1,771쪽, 총 3,542쪽이다. 새 정본 SHA-256은 `2280e97844b2cb262ea73e9c49f09f56c434097fed49af75fd9a7f6198d43da7`다.

**예외 승인:** Native 중첩 표 10쪽 한 행은 보호값 99.86%에 대해 99.85%다. [작업지시자의 한정 승인](../../native-floor-decision.md)에 따라 이 한 행만 수용한다. 예외 외 하락·누락·쪽수 불일치·측정 실패는 0건이다. 상승은 0행이므로 보호 정본은 변경하지 않는다.

중첩 표는 독립 HFT 기호를 복원한 뒤 전 17쪽을 두 backend에서 다시 측정한 **전체 데이터셋**을 사용했다. 첫 진단 실행의 누락 기호 결과를 통과본으로 쓰지 않았고, 서로 다른 실행의 페이지별 최댓값을 조합하지 않았다. 다른 17개 데이터셋도 같은 source의 실제 완료 전쪽 TSV다.

| 문서 | backend | 전체 쪽 | 최저 표시값 | 하락 | 전쪽 TSV SHA-256 |
| --- | --- | ---: | ---: | ---: | --- |
| [yeongwol](native/yeongwol.tsv) | native | 4 | 100.00 | 0 | `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1` |
| [seo-body-list-typed](native/seo-body-list-typed.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [handbook-hwp](native/handbook-hwp.tsv) | native | 383 | 99.97 | 0 | `261cbaa3c053a992e7058701416c4b38fb504bef61e8df22e44e5c5e7fb9a590` |
| [seo-helvetica-styles](native/seo-helvetica-styles.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [sample16-hwpx](native/sample16-hwpx.tsv) | native | 64 | 99.99 | 0 | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [sample16-hwp2022](native/sample16-hwp2022.tsv) | native | 64 | 100.00 | 0 | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [seo-re-03-latin](native/seo-re-03-latin.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [angle-narrow](native/angle-narrow.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [outline](native/outline.tsv) | native | 9 | 100.00 | 0 | `46ca210065f8f67bc6a79bfc1aeca1b1d36505f9c8749da7c93ff198a3c40675` |
| [issue8040-insert](native/issue8040-insert.tsv) | native | 4 | 99.99 | 0 | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [nested](native/nested.tsv) | native | 17 | 99.85 | 1 | `7dee20a740ff0a9b60101ff3f2b40dac288f84f192c16b0a0471f7c62153d59f` |
| [sample16-hwp2024](native/sample16-hwp2024.tsv) | native | 64 | 99.99 | 0 | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [handbook-hwpx](native/handbook-hwpx.tsv) | native | 383 | 99.97 | 0 | `261cbaa3c053a992e7058701416c4b38fb504bef61e8df22e44e5c5e7fb9a590` |
| [issue7905](native/issue7905.tsv) | native | 6 | 98.86 | 0 | `b828c10e23bcef9c5e7078451d3c8d92e1cb9811a7ac3a952c084a741f6ac6fe` |
| [sample10](native/sample10.tsv) | native | 763 | 99.57 | 0 | `248740c943667ee2121afdfc125ed838ae7e196e1c90265284613afc48f508bd` |
| [issue8040-insert-delete](native/issue8040-insert-delete.tsv) | native | 4 | 99.99 | 0 | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [seo-pic-crop-01](native/seo-pic-crop-01.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-shape-group-02](native/seo-shape-group-02.tsv) | native | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [yeongwol](wasm/yeongwol.tsv) | wasm | 4 | 100.00 | 0 | `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1` |
| [seo-body-list-typed](wasm/seo-body-list-typed.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [handbook-hwp](wasm/handbook-hwp.tsv) | wasm | 383 | 99.96 | 0 | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [seo-helvetica-styles](wasm/seo-helvetica-styles.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [sample16-hwpx](wasm/sample16-hwpx.tsv) | wasm | 64 | 99.99 | 0 | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [sample16-hwp2022](wasm/sample16-hwp2022.tsv) | wasm | 64 | 100.00 | 0 | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [seo-re-03-latin](wasm/seo-re-03-latin.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [angle-narrow](wasm/angle-narrow.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [outline](wasm/outline.tsv) | wasm | 9 | 100.00 | 0 | `46ca210065f8f67bc6a79bfc1aeca1b1d36505f9c8749da7c93ff198a3c40675` |
| [issue8040-insert](wasm/issue8040-insert.tsv) | wasm | 4 | 99.99 | 0 | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [nested](wasm/nested.tsv) | wasm | 17 | 99.86 | 0 | `4144f4d00c98c54b8c35b26309c00c86caedeee413aa78059bef0bc231ae1fcc` |
| [sample16-hwp2024](wasm/sample16-hwp2024.tsv) | wasm | 64 | 99.99 | 0 | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [handbook-hwpx](wasm/handbook-hwpx.tsv) | wasm | 383 | 99.96 | 0 | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [issue7905](wasm/issue7905.tsv) | wasm | 6 | 98.86 | 0 | `8da442a73d4de191337083da11f5c507b66d668d80821b86f82f29d4fa556d98` |
| [sample10](wasm/sample10.tsv) | wasm | 763 | 99.57 | 0 | `f0ff5f22ed2dfb65f1063f73a3b80fad44ba4f331d714a4acc16657628498de5` |
| [issue8040-insert-delete](wasm/issue8040-insert-delete.tsv) | wasm | 4 | 99.99 | 0 | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [seo-pic-crop-01](wasm/seo-pic-crop-01.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-shape-group-02](wasm/seo-shape-group-02.tsv) | wasm | 1 | 100.00 | 0 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |

Native 10쪽은 새 devel과 후보의 PNG가 동일하며, 선행 검증과 현재의 글꼴 파일 신원·호스트 조건 차이는 [별도 기록](../../latest-devel-revalidation.md)에 남겼다. 원인 미확정을 새 회귀나 미실행의 면제 근거로 확대하지 않는다.
