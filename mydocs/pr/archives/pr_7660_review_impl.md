# PR #7660 통합 검토 구현 계획

- 원 head `860f8c22ddd892e74030fb13bae8c8af07ef2aff`, 기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- [공통 실행 계획](../../plans/pr_review_seo_rii_20261017.md)의 순차 gate를 따른다.
- 고유 적용 대상 15개. 아래 SHA는 원 author를 보존해 오래된 순서로 적용한다.
- merge commit과 다른 PR에 이미 속한 commit은 중복 적용하지 않는다.
- 충돌·보정·검증 결과는 [개별 review](pr_7660_review.md)에 연결하며 검증 전 완료를 단정하지 않는다.
- rollback은 통합 PR 제출 전 이 작업 branch에서만 판단한다. 사용자 source/fork/history를 재작성하지 않는다.

## 원 commit 순서

- `2766cf6aeeddf4171c974d795fe579b9db141f64`
- `e619ac08e0dbb70642543bba15aa5a1c0392c198`
- `1580d078c387b55eebf211a04c95242bfea6d508`
- `02f39d0fc294fe4d1b0027cb97a6942fe0c7aec2`
- `0c167cf6cf1aa1bfff7222e1ce98c52b7adf9126`
- `90a407b393c32372dbaada1b69f103e04bfa2e76`
- `77cf734cec6464e3e36ac8980ae88dd169ffc9b9`
- `d7ae3de95bfcc85d5deec1c773b8ceb5d703cf22`
- `7bf446974c06cef72966f15afc9b42f48c034f65`
- `2ea7173b420809e36dcbef455c39fdea0ff21e03`
- `c6864b49c1d2134a55f45cca1756d0ef8d0d11f8`
- `24a453f889a7d28ac8fddb59dec87603bc1d27cb`
- `5a91f5ff67c12ac51e1e9eca3e56c82b7c3bbce5`
- `a860adef06c679ff49a31f8a0c4a2c8011098788`
- `5476fcea7b46ee7ad190f103b681ead3c3badc9c`

## 실제 적용 결과

- 원 `2766cf6aeeddf4171c974d795fe579b9db141f64` → 통합 `2f4d36449e5a39f2547638bb312641c15077015f` (applied).
- 원 `e619ac08e0dbb70642543bba15aa5a1c0392c198` → 통합 `dccf3524cebebbf3833fdd1457044f6fbd56a4fb` (applied).
- 원 `1580d078c387b55eebf211a04c95242bfea6d508` → 통합 `eb2ccded4ee2f2d9447786f809db6006df76a1c7` (applied).
- 원 `02f39d0fc294fe4d1b0027cb97a6942fe0c7aec2` → 통합 `283e579b3cfe50c33e60992e5aa7d04d1e16f2f8` (applied).
- 원 `0c167cf6cf1aa1bfff7222e1ce98c52b7adf9126` → 통합 `e64f2dc04c93d935a3a381bdafb8c218380bab2e` (applied).
- 원 `90a407b393c32372dbaada1b69f103e04bfa2e76` → 통합 `5d18a568e4e39004d31d1098628b96068f16315c` (applied).
- 원 `77cf734cec6464e3e36ac8980ae88dd169ffc9b9` → 통합 `37e4ecb82dd725ecb6358876f723f70d25e89c07` (applied).
- 원 `d7ae3de95bfcc85d5deec1c773b8ceb5d703cf22` → 통합 `820f6747a53a978e2bef18c8f92df133f2c84154` (applied).
- 원 `7bf446974c06cef72966f15afc9b42f48c034f65` → 통합 `e19892d48c75091efd9814e7adc1251267a2290a` (applied).
- 원 `2ea7173b420809e36dcbef455c39fdea0ff21e03` → 통합 `c37bc966845e8a297202aef5425fdc91873cc93b` (applied).
- 원 `c6864b49c1d2134a55f45cca1756d0ef8d0d11f8` → 통합 `1a92c9dc539d0de1dee19f85f6aaaa80b4709a79` (applied).
- 원 `24a453f889a7d28ac8fddb59dec87603bc1d27cb` → 통합 `fe69d6af72376ac1da0fe7c94cfe0306fbc53092` (applied).
- 원 `5a91f5ff67c12ac51e1e9eca3e56c82b7c3bbce5` → 통합 `5eab754737a8a80a4f4a7680338a398c8fba49ed` (applied).
- 원 `a860adef06c679ff49a31f8a0c4a2c8011098788` → 통합 `978d7e3dd7f68fe7ece6adde8096cb72bf497cf8` (applied).
- 원 `5476fcea7b46ee7ad190f103b681ead3c3badc9c` → 통합 `1c41caaa96bd4d580940783c55a467c63bb4959a` (applied).

[충돌·원 merge 해소 대조](../../working/task_m100_536_integration_review_stage1.md). 로컬 필수 검증을 완료했다. 원격 CI는 실제 통합 PR 생성 뒤 확인한다.

## 통합 후보 검증 연결

- 원 commit 적용과 merge 해소는 위 실제 SHA mapping 및 [1단계](../../working/task_m100_536_integration_review_stage1.md)를 따른다. 누적 source diff를 같은 PR의 고유 변경으로 반복 적용하지 않았다.
- 런타임/검사 분리는 `456ac3585`, 독립 Print 근거의 공통 극한 이름은 `d9a0b1c79`, 정식 원본 회귀는 `1ed02ebff`, font-request 반례·Print 보존은 `068769e9d`에 연결한다. 이 보정들을 원 contributor가 작성한 것으로 표시하지 않는다.
- 실제 실행 결과·source/binary/font/PDF hash와 직접 판독은 [5단계](../../working/task_m100_536_integration_review_stage5.md)·[6단계](../../working/task_m100_536_integration_review_stage6.md)에 있다. 기본 전체 nextest는 10,643 PASS / 0 FAIL / 45 skipped로 끝났다. Native 집중 277 PASS, 공식 Skia lib 4,043 PASS와 focused 2+4 PASS, format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 완료했다. 원격 exact-head CI는 미실행이며 이후 실제 결과로 trailing 기록을 갱신한다.
- 최종 판정과 merge 후 원 PR comment 계획은 같은 번호의 개별 review에 유지한다. 통합 PR 번호의 별도 review를 만들거나 contributor branch를 rewrite하지 않는다.

## 최신 base 리베이스

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 직후 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`. [7단계](../../working/task_m100_536_integration_review_stage7.md)의 새 head 검증은 진행 중이다. 위 실제 적용 SHA는 리베이스 전 기록이며 아래는 현재 대응이다.

- 이전 통합 `2f4d36449e5a39f2547638bb312641c15077015f` → 현재 `2a44f66bc026c217bdb2cf400bf847a39c836bf3`.
- 이전 통합 `dccf3524cebebbf3833fdd1457044f6fbd56a4fb` → 현재 `955dfd8a35d3bac4ec88c9fb8b1406cebeba5ae0`.
- 이전 통합 `eb2ccded4ee2f2d9447786f809db6006df76a1c7` → 현재 `4591d8536ad7b305c3834fedbb7407120345d9a6`.
- 이전 통합 `283e579b3cfe50c33e60992e5aa7d04d1e16f2f8` → 현재 `9afd0ce8f927a1f57974398825a1e224e88bd1d8`.
- 이전 통합 `e64f2dc04c93d935a3a381bdafb8c218380bab2e` → 현재 `28c460965c43f1a1358af3640003af7a41a1ce70`.
- 이전 통합 `5d18a568e4e39004d31d1098628b96068f16315c` → 현재 `353e41f8c8bfe1084e34429b9a08b05a9adff785`.
- 이전 통합 `37e4ecb82dd725ecb6358876f723f70d25e89c07` → 현재 `a56dcbcce2141d0fb703352b130972fc2d547ce0`.
- 이전 통합 `820f6747a53a978e2bef18c8f92df133f2c84154` → 현재 `832e4ec93f5e791fa53cee6f5265634327bb3e86`.
- 이전 통합 `e19892d48c75091efd9814e7adc1251267a2290a` → 현재 `8dc55c98a45b6cd18808d1455094228ed85a25eb`.
- 이전 통합 `c37bc966845e8a297202aef5425fdc91873cc93b` → 현재 `1201e3326cd742ec15aceca790f161a0bcdaa209`.
- 이전 통합 `1a92c9dc539d0de1dee19f85f6aaaa80b4709a79` → 현재 `837c425551f96fc0f4c6c98c34576b62708c238a`.
- 이전 통합 `fe69d6af72376ac1da0fe7c94cfe0306fbc53092` → 현재 `12b2184480502147aebc72d59681923eb5a98a4c`.
- 이전 통합 `5eab754737a8a80a4f4a7680338a398c8fba49ed` → 현재 `7d0f672785e9053ba73a09747e39cf93487b0d5b`.
- 이전 통합 `978d7e3dd7f68fe7ece6adde8096cb72bf497cf8` → 현재 `0960547757c5caccbef71749e848bfdef3887744`.
- 이전 통합 `1c41caaa96bd4d580940783c55a467c63bb4959a` → 현재 `c7f928e2aa65bb6706142d30a303c7257ea876da`.

## 최종 source의 원 commit 연결

- 원 `2766cf6aeeddf4171c974d795fe579b9db141f64` → 현재 `3a61a552364196f28c6a026cb7c065e624e586bf`.
- 원 `e619ac08e0dbb70642543bba15aa5a1c0392c198` → 현재 `6e670daa87e208bca4d2a611eaa3c654f2155742`.
- 원 `1580d078c387b55eebf211a04c95242bfea6d508` → 현재 `f63847f36d17046c3648d8c12aebff2de5247dba`.
- 원 `02f39d0fc294fe4d1b0027cb97a6942fe0c7aec2` → 현재 `c5bd505fa461e84fe93cf632a374e8a294e45587`.
- 원 `0c167cf6cf1aa1bfff7222e1ce98c52b7adf9126` → 현재 `3d12162250a9605b10abb80cc1bb25c2ae68fdb9`.
- 원 `90a407b393c32372dbaada1b69f103e04bfa2e76` → 현재 `add60c778715fbf7b128bc23e58c848e95fe84e1`.
- 원 `77cf734cec6464e3e36ac8980ae88dd169ffc9b9` → 현재 `6d48fb0d3a150db70e9038e89fee7a0ace0f6219`.
- 원 `d7ae3de95bfcc85d5deec1c773b8ceb5d703cf22` → 현재 `5ef0bdd6630d298bc3a2ee4306a80121c60442a0`.
- 원 `7bf446974c06cef72966f15afc9b42f48c034f65` → 현재 `89b9b9c67b9bef1eed0b6defb238a59ce92075ba`.
- 원 `2ea7173b420809e36dcbef455c39fdea0ff21e03` → 현재 `ae5f0a4ae9896ecacadec1fcbf4ca683fe7dd9cc`.
- 원 `c6864b49c1d2134a55f45cca1756d0ef8d0d11f8` → 현재 `4c7c00e649aa3f9176125844c468f8ceb07e6915`.
- 원 `24a453f889a7d28ac8fddb59dec87603bc1d27cb` → 현재 `1f0a1bc393f64d59b81e750f7221837f69c19857`.
- 원 `5a91f5ff67c12ac51e1e9eca3e56c82b7c3bbce5` → 현재 `f0f0c78621faadab84079bebe2b5fa998921b148`.
- 원 `a860adef06c679ff49a31f8a0c4a2c8011098788` → 현재 `e9c7ebbed8899adf76178c0d743bbc74bec9fecf`.
- 원 `5476fcea7b46ee7ad190f103b681ead3c3badc9c` → 현재 `804d2b6fc81a7ec5c28d5386333d95c68acf4bef`.

## 최신 devel 통합 검증

현재 producer `3535106051394672e93ca8401ff9eae8b449ec20` /base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`의 전체 회귀10779PASS와 실제 소비 경로 검증은 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다.원 contributor author와164개 리베이스 commit의 순서·제목을 보존했다.원격 CI·merge 결과는 실제 확인 뒤 같은 PR trailing 기록에 추가한다.

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 review·오늘할일 trailing head CI와 normal merge는 다음 단계다.
