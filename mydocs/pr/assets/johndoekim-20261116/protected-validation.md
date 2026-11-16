# 메인터너 정본 보호 결과

> 최신 devel 재검증 완료: 아래 수치는 **직전 후보 `105be723b2`의 역사적 기록**이다. 최종 source `4deadfa963`의 실제 13,632개·Native 4,073개·전쪽 보호 및 승인된 Native 1행 예외는 [최종 기록](latest-devel-revalidation.md)을 따른다.

18개 입력 × Native/fresh WASM, 전체 **3,542쪽**을 실제 실행했다. 누락·측정 불가·공식값 하락은 **0건**이다.
상승은 0행이다. 예외·허용치·기준값을 변경하지 않았다.

정본 SHA-256: `43c59cf77a4a86b3744d3df30eb71469de70b584ccc9cb165fa8f7b7884d2e1f`. 생산 source와 최종 test source의 제품/캡처 입력 동일성은 [실행 기록](validation-record.md)에 있다.

같은 compat·96dpi·독립 Print·실제 글꼴 프로그램 해시를 사용했다. 원래 정본 생산자의 [공급 기록](../jinho-von-choi-20261114/final-source-8ad/protected-validation.json)을 그대로 재현했고, 복구한 구형 수식 글꼴 1개의 새 컨테이너 해시와 동일한 77개 프로그램은 별도 기록했다.

sample10 WASM은 원래 기록에 있던 `oracle.gif`·`rdb02.gif`·`s1.jpg`도 같은 SHA로 명시 공급했다. 처음 실행에서 이 세 입력을 빠뜨려 표지 그림이 보이지 않았다. 원인을 확인한 뒤 입력 계약을 복원해 전체 763쪽을 다시 실행했으며, 최종 TSV에는 이 완전한 실행만 쓴다.

| 문서 | backend | 쪽수 | raw 최저 | 전쪽 TSV SHA-256 |
| --- | --- | ---: | ---: | --- |
| [yeongwol](protected/native/yeongwol.tsv) | native | 4 | 100.00% | `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1` |
| [seo-body-list-typed](protected/native/seo-body-list-typed.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [handbook-hwp](protected/native/handbook-hwp.tsv) | native | 383 | 99.97% | `261cbaa3c053a992e7058701416c4b38fb504bef61e8df22e44e5c5e7fb9a590` |
| [seo-helvetica-styles](protected/native/seo-helvetica-styles.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-re-03-latin](protected/native/seo-re-03-latin.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [angle-narrow](protected/native/angle-narrow.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [outline](protected/native/outline.tsv) | native | 9 | 100.00% | `46ca210065f8f67bc6a79bfc1aeca1b1d36505f9c8749da7c93ff198a3c40675` |
| [issue8040-insert](protected/native/issue8040-insert.tsv) | native | 4 | 99.99% | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [nested](protected/native/nested.tsv) | native | 17 | 98.63% | `5f4f11cc5d2e3495ac05b347bc74f9b9c186091d0175ea26f2595837722ac479` |
| [handbook-hwpx](protected/native/handbook-hwpx.tsv) | native | 383 | 99.97% | `261cbaa3c053a992e7058701416c4b38fb504bef61e8df22e44e5c5e7fb9a590` |
| [issue7905](protected/native/issue7905.tsv) | native | 6 | 98.86% | `b828c10e23bcef9c5e7078451d3c8d92e1cb9811a7ac3a952c084a741f6ac6fe` |
| [sample10](protected/native/sample10.tsv) | native | 763 | 99.57% | `248740c943667ee2121afdfc125ed838ae7e196e1c90265284613afc48f508bd` |
| [issue8040-insert-delete](protected/native/issue8040-insert-delete.tsv) | native | 4 | 99.99% | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [seo-pic-crop-01](protected/native/seo-pic-crop-01.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-shape-group-02](protected/native/seo-shape-group-02.tsv) | native | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [sample16-hwpx](protected/native/sample16-hwpx.tsv) | native | 64 | 99.99% | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [sample16-hwp2022](protected/native/sample16-hwp2022.tsv) | native | 64 | 100.00% | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [sample16-hwp2024](protected/native/sample16-hwp2024.tsv) | native | 64 | 99.99% | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [yeongwol](protected/wasm/yeongwol.tsv) | wasm | 4 | 100.00% | `4475a64f14dc6abd397b9c38d09fd626931074cd6cd12472af9ac2c101c929a1` |
| [seo-body-list-typed](protected/wasm/seo-body-list-typed.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [handbook-hwp](protected/wasm/handbook-hwp.tsv) | wasm | 383 | 99.96% | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [seo-helvetica-styles](protected/wasm/seo-helvetica-styles.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-re-03-latin](protected/wasm/seo-re-03-latin.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [angle-narrow](protected/wasm/angle-narrow.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [outline](protected/wasm/outline.tsv) | wasm | 9 | 100.00% | `46ca210065f8f67bc6a79bfc1aeca1b1d36505f9c8749da7c93ff198a3c40675` |
| [issue8040-insert](protected/wasm/issue8040-insert.tsv) | wasm | 4 | 99.99% | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [nested](protected/wasm/nested.tsv) | wasm | 17 | 98.65% | `451d72cddb6f90e6327e1602b046efaed1de4162e472ee989cac313fb0b64aa9` |
| [handbook-hwpx](protected/wasm/handbook-hwpx.tsv) | wasm | 383 | 99.96% | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` |
| [issue7905](protected/wasm/issue7905.tsv) | wasm | 6 | 98.86% | `8da442a73d4de191337083da11f5c507b66d668d80821b86f82f29d4fa556d98` |
| [sample10](protected/wasm/sample10.tsv) | wasm | 763 | 99.57% | `f0ff5f22ed2dfb65f1063f73a3b80fad44ba4f331d714a4acc16657628498de5` |
| [issue8040-insert-delete](protected/wasm/issue8040-insert-delete.tsv) | wasm | 4 | 99.99% | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` |
| [seo-pic-crop-01](protected/wasm/seo-pic-crop-01.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [seo-shape-group-02](protected/wasm/seo-shape-group-02.tsv) | wasm | 1 | 100.00% | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [sample16-hwpx](protected/wasm/sample16-hwpx.tsv) | wasm | 64 | 99.99% | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
| [sample16-hwp2022](protected/wasm/sample16-hwp2022.tsv) | wasm | 64 | 100.00% | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` |
| [sample16-hwp2024](protected/wasm/sample16-hwp2024.tsv) | wasm | 64 | 99.99% | `e19f299f16b0117e13d26b8ed77d5dfa038345b87b0421436feea2c09fa7a942` |
