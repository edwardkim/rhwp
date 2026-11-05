# PR #8389·#8397의 Native 전쪽 무하락 검증

제품 source `c183a1ac61c2aa4188989fd93032cae68d3538c1`, Native SHA-256 `cbb91433ec4e5b968ea7820dae1af09aa712991ae5b3027c68590fee5f2138d9`.

중복 제외 17문서·1,770쪽을 원 TSV 2,153행과 대조했다. **표시값 하락 0행·누락 0행·오류 0건**이다. 소수 셋째 자리에서 HALF_UP으로 반올림한 소수 둘째 자리 표시값을 비교하며 상승은 허용했다.

원 비교표 SHA-256 `21e3ca2a3035927bfbf5c17710657ea2b2b8691025c6fb55571fdc781690384e`, 현재 비교표 SHA-256 `8eef797f895e085fdacece53109fb71c8808f1619caa8395846c03d96aad2e1b`.

[전체 원행 대조 TSV](native-display-comparison.tsv)와 다음 17개 실제 전쪽 TSV를 제출한다.

| 문서 식별자 | 전체 쪽수 · 최저 raw | 전쪽 TSV | 입력 SHA-256 · 독립 PDF SHA-256 |
| --- | --- | --- | --- |
| `issue7905` | 6쪽 · 98.86% | [Native TSV](<native/issue7905.tsv>) | `63c5669a5fbf99bed259bb6040b3a1ec17b218e7115b867c32bd545dbb623cc1` · `1ce35eb7e2d6f93e41834215d6cf7653e3a09ea2fc544856b3df3a89e6d24511` |
| `issue8040-insert` | 4쪽 · 99.99% | [Native TSV](<native/issue8040-insert.tsv>) | `3c3ff50b430c3129a4ce06575b1a398d01fbd4ec17d9dd895f467a945f656e82` · `d368c0ded6f0d9309784e9165aad7360b326a86c01d9938beb95c768ae0d4e74` |
| `issue8040-insert-delete` | 4쪽 · 99.99% | [Native TSV](<native/issue8040-insert-delete.tsv>) | `fe0953499311b5be82c85560a92a35c26ea6be3ee0b9f64460d111b3313dc8ee` · `84704f8c2cccf955379c648ea7d9dbf04242e6c4fb2620e37d87ac004b7a330d` |
| `sample16-hwpx` | 64쪽 · 99.99% | [Native TSV](<native/sample16-hwpx.tsv>) | `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c` · `be4a0286d3ad5209a1a0787fd4ff691e110ddbcf7bde3a52a4547958e452a1ee` |
| `sample16-hwp2024` | 64쪽 · 99.99% | [Native TSV](<native/sample16-hwp2024.tsv>) | `f737ee22888b0a11171622b3e1c12d71d0f42cf066f729d3c98dde9ca6cc122d` · `62745aebf90af452bf278f8cfb97c4d523e854496a998279d0e88c7a9a3d9c9b` |
| `sample16-hwp2022` | 64쪽 · 100.00% | [Native TSV](<native/sample16-hwp2022.tsv>) | `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe` · `c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33` |
| `handbook-hwp` | 383쪽 · 99.96% | [Native TSV](<native/handbook-hwp.tsv>) | `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` · `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593` |
| `handbook-hwpx` | 383쪽 · 99.96% | [Native TSV](<native/handbook-hwpx.tsv>) | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` · `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e` |
| `seo-body-list-typed` | 1쪽 · 100.00% | [Native TSV](<native/seo-body-list-typed.tsv>) | `ebe04aa214d85ca3f472083ae749f534de446ecf6f58eec532a6c2149839e687` · `d4c80383b977a0f61e057adadb9f302d591bbb26e0917c61128ee36d37aed5b3` |
| `seo-helvetica-styles` | 1쪽 · 100.00% | [Native TSV](<native/seo-helvetica-styles.tsv>) | `2095908091871ec5ff78c4bd225779029646258bfd543e4e6afde7f36550f7da` · `fd8672dcb24dd7bb1d50983b0545033e7c2513bbb2ecfc48edde1cb4e77058ba` |
| `seo-pic-crop-01` | 1쪽 · 100.00% | [Native TSV](<native/seo-pic-crop-01.tsv>) | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` · `4ae5f6f8bc308bedf16fbca139de448c42d29b42bbba22221f4a1acc9694c4d3` |
| `seo-shape-group-02` | 1쪽 · 100.00% | [Native TSV](<native/seo-shape-group-02.tsv>) | `12a4b4962a57b4d292d23a1809255ddb7e8400af4dc7884c5bcb7b4767a6628f` · `ab759c9c2031f4a6fe64c75381a2c5dbf215777a98b1b0b7cd0243f801662dc7` |
| `seo-re-03-latin` | 1쪽 · 100.00% | [Native TSV](<native/seo-re-03-latin.tsv>) | `52b8da3a7cd5625a5e9624aaa532be5faa29d9fc0fc96cc982109729d53cb09f` · `d82ea49f681c651d090a11f58ffb9db1a0ae2e37ff71b016400870432f7e53e3` |
| `sample10` | 763쪽 · 99.41% | [Native TSV](<native/sample10.tsv>) | `d9ceb35d8abfb73e9afbe349bccb2a986cf552d66ae7f3f34dcfe4d9d385cb48` · `310e3251e910ac8c7c6f34d0e50d85c4a0ecbbadd65e69fe4f5071370931a918` |
| `yeongwol` | 4쪽 · 99.97% | [Native TSV](<native/yeongwol.tsv>) | `6d723789148699479983fbcb6fa6368a252c1187fe0a7b3c922b583f6727ec09` · `a04da624238d77da4832dd302017c7d2f559a01da02e4c1b4ae12cfd3f77b123` |
| `outline` | 9쪽 · 99.99% | [Native TSV](<native/outline.tsv>) | `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d` · `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb` |
| `nested` | 17쪽 · 94.96% | [Native TSV](<native/nested.tsv>) | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` · `bb486ef1215e79c2ca0c891b2b194bdf988691b0e91b8a07ebdbffc6d749473b` |

issue7905는 6쪽 모두 98% 이상이고 전쪽 최저 98.86%다. [원본·Print·환경](../../pr8389_8397/README.md)을 재사용했다. fresh WASM은 별도 동일 범위 검증을 완료해야 하며, Native 결과로 대신하지 않는다. 후속 37e 보정이 이 제품에 들어 있다고 표시하지 않는다.
