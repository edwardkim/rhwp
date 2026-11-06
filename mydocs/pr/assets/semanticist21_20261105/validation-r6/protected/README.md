# PR #8389·#8397 전쪽 보호 검증

현재 제품 `86709ac16711820e61baa3b87e2c4ddaac23573e`의 Native/fresh WASM을 각 원 PR의 공개 TSV와 대조했다.
각 backend는 실제 고유 17문서·1770쪽을 측정했다.
원 PR의 공개 기준은 중복 보호 행을 포함해 backend별 2,153행이며, 두 경로 모두 표시값 하락·누락·오류 0이다.
원문 TSV를 복사해 현재 측정인 것처럼 재표기하지 않았다.

- [Native 전체 대조](native-comparison.md) · [fresh WASM 전체 대조](wasm-comparison.md)
- [현재 Native TSV 원문](native) · [현재 fresh WASM TSV 원문](wasm)

공통 조건은 같은 입력·독립 Print, 96 DPI, Print 프로필, 2px raw다.
Native 실제 binary SHA `e52882bc5acda6cec799e1cf784861bfda514a867f55daf3b5e1f720b35109e1`,
WASM SHA `4755dbfeda5bba99d72191a54e90342ae381ec17f0dd840926bbf1fdd75d1f30`.
기존 승인 대체 설정·전체 시스템 글꼴과 sample10 외부 참조 이미지 공급을 유지했다.
시각 검증의 글꼴 환경과 최소 공개 글꼴 회귀 환경을 구분한다.
공식 Native 편람 HWPX 383쪽 gate는 exit 0·regressions 0·webfont_fallback 0이다.
원시 JSON·실행 로그는 output의 로컬 검증 자료이며 제출하지 않는다.

| 문서 | backend | 전체 쪽수 | 전쪽 최저 raw | 현재 TSV | TSV SHA-256·bytes |
| --- | --- | ---: | ---: | --- | --- |
| issue7905 | native | 6 | 98.86043 | [원문](native/issue7905.tsv) | `b828c10e23bcef9c5e7078451d3c8d92e1cb9811a7ac3a952c084a741f6ac6fe` · 220 |
| issue8040-insert | native | 4 | 99.98832 | [원문](native/issue8040-insert.tsv) | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` · 190 |
| issue8040-insert-delete | native | 4 | 99.98832 | [원문](native/issue8040-insert-delete.tsv) | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` · 190 |
| sample16-hwpx | native | 64 | 99.99273 | [원문](native/sample16-hwpx.tsv) | `804fedcc655075b2739abaf7adfb4764c1938357fcaeaaceb62dca421ecb4f49` · 1446 |
| sample16-hwp2024 | native | 64 | 99.99273 | [원문](native/sample16-hwp2024.tsv) | `804fedcc655075b2739abaf7adfb4764c1938357fcaeaaceb62dca421ecb4f49` · 1446 |
| sample16-hwp2022 | native | 64 | 100.0 | [원문](native/sample16-hwp2022.tsv) | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` · 1447 |
| handbook-hwp | native | 383 | 99.96463 | [원문](native/handbook-hwp.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` · 8428 |
| handbook-hwpx | native | 383 | 99.96463 | [원문](native/handbook-hwpx.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` · 8428 |
| seo-body-list-typed | native | 1 | 100.0 | [원문](native/seo-body-list-typed.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-helvetica-styles | native | 1 | 100.0 | [원문](native/seo-helvetica-styles.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-pic-crop-01 | native | 1 | 99.99756 | [원문](native/seo-pic-crop-01.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-shape-group-02 | native | 1 | 100.0 | [원문](native/seo-shape-group-02.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-re-03-latin | native | 1 | 100.0 | [원문](native/seo-re-03-latin.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| sample10 | native | 763 | 99.41135 | [원문](native/sample10.tsv) | `6c0d5bc0bf5cec62683cad2892a6352c6039516fd6180c55dabb8d5e7cc7ed8b` · 16718 |
| yeongwol | native | 4 | 99.96664 | [원문](native/yeongwol.tsv) | `45a954603af2a2a28fb05b309e9ae53da615aa3931846819f60c62a338b10121` · 186 |
| outline | native | 9 | 99.99237 | [원문](native/outline.tsv) | `2e90560f2542d5150f6311d7a634c5a11e4901453b4730124a5957fd1d726c84` · 290 |
| nested | native | 17 | 94.96164 | [원문](native/nested.tsv) | `d8b72033b81c3cb82e1557089927e869f2833fec8b9e05678d8935972bcafc57` · 426 |
| issue7905 | wasm | 6 | 98.86659 | [원문](wasm/issue7905.tsv) | `e18b07f77903f2eb05b779457a1bb606a3c0ca6aa9854a94a6320c7993334942` · 228 |
| issue8040-insert | wasm | 4 | 99.98866 | [원문](wasm/issue8040-insert.tsv) | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` · 190 |
| issue8040-insert-delete | wasm | 4 | 99.98866 | [원문](wasm/issue8040-insert-delete.tsv) | `f570344a6869ef9e8380bc15e0ea6093f077b913079cf9d5a2d7c610b8fade08` · 190 |
| sample16-hwpx | wasm | 64 | 99.99273 | [원문](wasm/sample16-hwpx.tsv) | `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` · 1447 |
| sample16-hwp2024 | wasm | 64 | 99.99273 | [원문](wasm/sample16-hwp2024.tsv) | `b63dc2408d611890f7d966e88f036d930fb514dcaf3e7d82e017d10da28b0ffa` · 1447 |
| sample16-hwp2022 | wasm | 64 | 99.99816 | [원문](wasm/sample16-hwp2022.tsv) | `418d4bcbe4f906eea1d18e539b57a245dfaa26a79203c693a59af622ffa3caf7` · 1447 |
| handbook-hwp | wasm | 383 | 99.96402 | [원문](wasm/handbook-hwp.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` · 8428 |
| handbook-hwpx | wasm | 383 | 99.96402 | [원문](wasm/handbook-hwpx.tsv) | `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308` · 8428 |
| seo-body-list-typed | wasm | 1 | 100.0 | [원문](wasm/seo-body-list-typed.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-helvetica-styles | wasm | 1 | 100.0 | [원문](wasm/seo-helvetica-styles.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-pic-crop-01 | wasm | 1 | 94.65178 | [원문](wasm/seo-pic-crop-01.tsv) | `7678d6f2f5d362601ec7573b64be6cc0077e70485b9fbdd18272914d4058f420` · 130 |
| seo-shape-group-02 | wasm | 1 | 100.0 | [원문](wasm/seo-shape-group-02.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| seo-re-03-latin | wasm | 1 | 100.0 | [원문](wasm/seo-re-03-latin.tsv) | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` · 132 |
| sample10 | wasm | 763 | 99.3972 | [원문](wasm/sample10.tsv) | `3f55d35cb8f9f8ec5946ab5010f085c0518bb79a8c734c7a11c28227277d88a2` · 16730 |
| yeongwol | wasm | 4 | 99.97763 | [원문](wasm/yeongwol.tsv) | `b81ecaef16fbdec585263b66c710bcb0dc2727edf103ace7c8c431a6bbd874bf` · 188 |
| outline | wasm | 9 | 99.93728 | [원문](wasm/outline.tsv) | `70cc9459d29c3dc4f5a4f33d1d19d0efb7fa6b2e47bb39230779c46f68041010` · 288 |
| nested | wasm | 17 | 96.35271 | [원문](wasm/nested.tsv) | `f61db5f2635e7dc30c201e6d11a2e7677f2b7e109e77617effc1c1c5f0c98514` · 426 |

## 실제 입력과 독립 Print 신원

### issue7905

- 원본 `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx`, SHA-256 `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1`.
- 독립 Print `mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf`, 287952 bytes, SHA-256 `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511`.
- RHWP 조판 compat `2024`.

### issue8040-insert

- 원본 `samples/pr8040/exam-social-insert.hwp`, SHA-256 `3c3ff50b430c3129a4ce06575b1a398d01fbd4ec17d9dd895f467a945f656e82`.
- 독립 Print `pdf/pr8040/exam-social-insert-2020.pdf`, 641866 bytes, SHA-256 `d368c0ded6f0d9309784e9165aad7360b326a86c01d9938beb95c768ae0d4e74`.
- RHWP 조판 compat `2022`.

### issue8040-insert-delete

- 원본 `samples/pr8040/exam-social-insert-delete.hwp`, SHA-256 `fe0953499311b5be82c85560a92a35c26ea6be3ee0b9f64460d111b3313dc8ee`.
- 독립 Print `pdf/pr8040/exam-social-insert-delete-2020.pdf`, 641867 bytes, SHA-256 `84704f8c2cccf955379c648ea7d9dbf04242e6c4fb2620e37d87ac004b7a330d`.
- RHWP 조판 compat `2022`.

### sample16-hwpx

- 원본 `samples/hwp3-sample16-hwp5.hwpx`, SHA-256 `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`.
- 독립 Print `pdf/hwp3-sample16-hwp5-hwpx-2024.pdf`, 2234018 bytes, SHA-256 `be4a0286d3ad5209a1a0787fd4ff691e110ddbcf7bde3a52a4547958e452a1ee`.
- RHWP 조판 compat `2024`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2024-font-environment.json`, SHA-256 `53a7cff101203c61080a59769730f6f7050e5af22870b8a9d233bff7a95ec64c`.

### sample16-hwp2024

- 원본 `samples/hwp3-sample16-hwp5-2024.hwp`, SHA-256 `f737ee22888b0a11171622b3e1c12d71d0f42cf066f729d3c98dde9ca6cc122d`.
- 독립 Print `pdf/hwp3-sample16-hwp5-2024-hwp-2024.pdf`, 2234015 bytes, SHA-256 `62745aebf90af452bf278f8cfb97c4d523e854496a998279d0e88c7a9a3d9c9b`.
- RHWP 조판 compat `2024`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2024-font-environment.json`, SHA-256 `53a7cff101203c61080a59769730f6f7050e5af22870b8a9d233bff7a95ec64c`.

### sample16-hwp2022

- 원본 `samples/hwp3-sample16-hwp5-2022.hwp`, SHA-256 `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`.
- 독립 Print `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf`, 1830277 bytes, SHA-256 `c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33`.
- RHWP 조판 compat `2022`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2020-font-environment.json`, SHA-256 `9d42f0162206f4b829c603092c8db445ff5979d1d79f4d9a6b0155918459b43f`.

### handbook-hwp

- 원본 `samples/2025 행정업무운영 편람(최종).hwp`, SHA-256 `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`.
- 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`, 4692840 bytes, SHA-256 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`.
- RHWP 조판 compat `2024`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json`, SHA-256 `b365798897121252598cb18e16d01799024caaad8fe7ccf509b983f10090ed25`.

### handbook-hwpx

- 원본 `samples/2025 행정업무운영 편람(최종).hwpx`, SHA-256 `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`, 4692840 bytes, SHA-256 `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
- RHWP 조판 compat `2024`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json`, SHA-256 `b365798897121252598cb18e16d01799024caaad8fe7ccf509b983f10090ed25`.

### seo-body-list-typed

- 원본 `tests/fixtures/issue_8109_saved_fonts/body-list-typed-hwp.hwp`, SHA-256 `ebe04aa214d85ca3f472083ae749f534de446ecf6f58eec532a6c2149839e687`.
- 독립 Print `pdf/issue8109/print/body-list-typed-hwp-2020.pdf`, 19110 bytes, SHA-256 `d4c80383b977a0f61e057adadb9f302d591bbb26e0917c61128ee36d37aed5b3`.
- RHWP 조판 compat `2022`.

### seo-helvetica-styles

- 원본 `tests/fixtures/issue_8109_saved_fonts/control-styles-hwp.hwp`, SHA-256 `2095908091871ec5ff78c4bd225779029646258bfd543e4e6afde7f36550f7da`.
- 독립 Print `pdf/issue8109/print/control-styles-hwp-2020.pdf`, 43664 bytes, SHA-256 `fd8672dcb24dd7bb1d50983b0545033e7c2513bbb2ecfc48edde1cb4e77058ba`.
- RHWP 조판 compat `2022`.

### seo-pic-crop-01

- 원본 `samples/pic-crop-01.hwp`, SHA-256 `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c`.
- 독립 Print `pdf/pic-crop-01-2022.pdf`, 19211 bytes, SHA-256 `4ae5f6f8bc308bedf16fbca139de448c42d29b42bbba22221f4a1acc9694c4d3`.
- RHWP 조판 compat `2022`.

### seo-shape-group-02

- 원본 `samples/shape-group-02.hwp`, SHA-256 `12a4b4962a57b4d292d23a1809255ddb7e8400af4dc7884c5bcb7b4767a6628f`.
- 독립 Print `pdf/shape-group-02-hwp-2020.pdf`, 89651 bytes, SHA-256 `ab759c9c2031f4a6fe64c75381a2c5dbf215777a98b1b0b7cd0243f801662dc7`.
- RHWP 조판 compat `2022`.

### seo-re-03-latin

- 원본 `samples/re-03-latin-only-hancom.hwp`, SHA-256 `52b8da3a7cd5625a5e9624aaa532be5faa29d9fc0fc96cc982109729d53cb09f`.
- 독립 Print `pdf/re-03-latin-only-hancom-hwp-2020.pdf`, 11752 bytes, SHA-256 `d82ea49f681c651d090a11f58ffb9db1a0ae2e37ff71b016400870432f7e53e3`.
- RHWP 조판 compat `2022`.

### sample10

- 원본 `samples/hwp3-sample10.hwp`, SHA-256 `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48`.
- 독립 Print `pdf/issue8371/hwp3-sample10-2020.pdf`, 96185197 bytes, SHA-256 `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918`.
- RHWP 조판 compat `2022`.
- 공급한 외부 참조 그림 `samples/oracle.gif`, SHA-256 `464e863dd2c1650fc6997b03a5d96c9413e61bbabaf7337c783db27203cc2761`.
- 공급한 외부 참조 그림 `samples/rdb02.gif`, SHA-256 `bfadf4cdbbeeb5f3d8632cb54c8c3696977f405204b651f99a7a24c8f39532cf`.
- 공급한 외부 참조 그림 `samples/s1.jpg`, SHA-256 `77dea18ce7f8f93b0931e133dec222aec642eef0ed87b8f2031b94dcbea5c514`.
- 기존 원 PR의 50 MiB 이상 기준 PDF 로컬 보관 계약을 유지한다. 실제 보관 `/home/tsjang/rhwp/pdf/issue8371/hwp3-sample10-2020.pdf`, 전 763쪽. 새 PDF를 재출력하거나 PR에 넣지 않았다.

### yeongwol

- 원본 `samples/issue5699/20099369_yeongwol_forms.hwp`, SHA-256 `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09`.
- 독립 Print `pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf`, 91696 bytes, SHA-256 `a04da624238d77da4832dd302017c7d2f559a01da02e4c1b4ae12cfd3f77b123`.
- RHWP 조판 compat `2022`.
- 승인 대체 설정 `mydocs/pr/assets/planet6897_20261103/yeongwol-font-environment.json`, SHA-256 `d8bffde39aac561051991fe06b83e453439376acff84814439934864589b40e4`.

### outline

- 원본 `samples/task3307/issue3307_outline_number.hwpx`, SHA-256 `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d`.
- 독립 Print `pdf/task3307/issue3307_outline_number-2020.pdf`, 237518 bytes, SHA-256 `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb`.
- RHWP 조판 compat `2022`.

### nested

- 원본 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, SHA-256 `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626`.
- 독립 Print `pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf`, 625577 bytes, SHA-256 `bb486ef1215e79c2ca0c891b2b194bdf988691b0e91b8a07ebdbffc6d749473b`.
- RHWP 조판 compat `2022`.
