# PR #7891 통합 검토 구현 계획

- 원 head `8273ece22860b5793767b9888dd320b3bdc414b8`, 기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- [공통 실행 계획](../../plans/pr_review_seo_rii_20261017.md)의 순차 gate를 따른다.
- 고유 적용 대상 29개. 아래 SHA는 원 author를 보존해 오래된 순서로 적용한다.
- merge commit과 다른 PR에 이미 속한 commit은 중복 적용하지 않는다.
- 충돌·보정·검증 결과는 [개별 review](pr_7891_review.md)에 연결하며 검증 전 완료를 단정하지 않는다.
- rollback은 통합 PR 제출 전 이 작업 branch에서만 판단한다. 사용자 source/fork/history를 재작성하지 않는다.

## 원 commit 순서

- `9c20ab17c08806fea8327eafdfc53775f18ee9e7`
- `8360eed403f14c0bc271643edc512adf9e2705ff`
- `8adbbac59cbadd02e49afcba6f416ff251308509`
- `2743e925a211ea8753e82d88896b57203f8fbd41`
- `1b5b5a5213ec9333033e6241301b8c899df0337f`
- `5458b4d7f9281449bf23e879cb7848f17afcffbe`
- `3c14e8ca9f6ea3f75f63a1e7f55a3a4b12b2d70f`
- `bc8ab82e8e5a060ebee67bea4d25146ff4ce37ec`
- `8b3debe2950f89c7bf8e2c40532b1b4db7ed4650`
- `65ac4f709af96b35a127bb8ed0689620bf2f9e91`
- `abd57190925e99cd12d5570c24af5bb1fb55f68c`
- `f47d6fd9e0de0eeb27f135456902385078f0bae8`
- `a8d60bb85df67bd74528277273cf7edd19ba46b0`
- `987b47874a40487f2b6e531e472f4f28d39aef83`
- `c6c6d43a1ad38bdce4793787a6934e8873a5dcc1`
- `f559227abc6f8818a356ffd995fc5727e3e30dac`
- `95857610ca11acde429a0b031d9d11e9845005b3`
- `5cb8bf46451abda43f031b4934e07fa22b76004c`
- `a4d61aa1758570052824e64946e0fc0b0be512d1`
- `60db4c7a47f3a17af324a81b645cd260e1fd86d7`
- `d4b39efd31808524a19043b3fe6e73acb0ef1182`
- `59c6fd56d2ee25826bd4cd78374727c36074529c`
- `a22d902ae8081ae6ec58d8d833527f79b7fa014f`
- `6f4de1630156fb6c772bb6a8aa94d938838dfa27`
- `3a025062849a80115ab0e1dd1e0d9b9a70f8eb96`
- `2525591a822c472eecc19dbda53e817730c6e7dc`
- `342b17371c3e98b8f87fa0d98ccaf7f26d687fe5`
- `f58eb3ee4998dabcd7373a2b54f06b2bde2c57b9`
- `8273ece22860b5793767b9888dd320b3bdc414b8`

## 실제 적용 결과

- 원 `9c20ab17c08806fea8327eafdfc53775f18ee9e7` → 통합 `a50c25b3aa332a46a14a1f349d03fe82b8ad5053` (applied).
- 원 `8360eed403f14c0bc271643edc512adf9e2705ff` → 통합 `991701fa59b4575fa8be45a0d0f7a6280d95c122` (applied).
- 원 `8adbbac59cbadd02e49afcba6f416ff251308509` → 통합 `5e550e12b679403ba11fa6d4299791234f1d5758` (applied).
- 원 `2743e925a211ea8753e82d88896b57203f8fbd41` → 통합 `4707a41a973ab272784a5c3f991e1bbf15728fa8` (applied).
- 원 `1b5b5a5213ec9333033e6241301b8c899df0337f` → 통합 `e467f08c4daa7f5db9b0fe73850795180ad698b5` (applied).
- 원 `5458b4d7f9281449bf23e879cb7848f17afcffbe` → 통합 `b72a5f2f67acff72238feadd110857d00eb5ac2c` (applied).
- 원 `3c14e8ca9f6ea3f75f63a1e7f55a3a4b12b2d70f` → 통합 `bda7a209a8b9c2a068f2b78488120042180a3ac9` (applied).
- 원 `bc8ab82e8e5a060ebee67bea4d25146ff4ce37ec` → 통합 `1677240efaf4be79c6bebb75b1358b423220326f` (applied).
- 원 `8b3debe2950f89c7bf8e2c40532b1b4db7ed4650` → 통합 `caa893cd846a38da65cbc906e67bc6d5d3236879` (applied).
- 원 `65ac4f709af96b35a127bb8ed0689620bf2f9e91` → 통합 `fc2335062aba5ebe847a21861121e27968ecd7ef` (applied).
- 원 `abd57190925e99cd12d5570c24af5bb1fb55f68c` → 통합 `b3af1d45f6fa989e43753ab04ce30e72e3da243f` (applied).
- 원 `f47d6fd9e0de0eeb27f135456902385078f0bae8` → 통합 `9060f3d5af21eb3a2fd8f749e7054fddb1a65ade` (applied).
- 원 `a8d60bb85df67bd74528277273cf7edd19ba46b0` → 통합 `a566d272936f7ab52a7d650ee544330761312688` (applied).
- 원 `987b47874a40487f2b6e531e472f4f28d39aef83` → 통합 `5880e26495c31c4bac7fceb93ede75c0632c81f2` (applied).
- 원 `c6c6d43a1ad38bdce4793787a6934e8873a5dcc1` → 통합 `e57e38e1a0c3e18cd6907e2b9defa22f3a32e9d9` (applied).
- 원 `f559227abc6f8818a356ffd995fc5727e3e30dac` → 통합 `b56c132beccf4d14a82b3a0f68ad3a5d631a6729` (applied).
- 원 `95857610ca11acde429a0b031d9d11e9845005b3` → 통합 `3b06b73a0c2ff9392cb4ee34dcdba2bd04829ee0` (applied).
- 원 `5cb8bf46451abda43f031b4934e07fa22b76004c` → 통합 `eb1fc7a32c1728d946d6cd28ac66396e4619187d` (applied).
- 원 `a4d61aa1758570052824e64946e0fc0b0be512d1` → 통합 `56c2f1a984d87d46e4b9b9366b4a74cf38b3a070` (applied).
- 원 `60db4c7a47f3a17af324a81b645cd260e1fd86d7` → 통합 `c840bc89fd85a592f51cee9d79e86ace20f9c8a1` (applied).
- 원 `d4b39efd31808524a19043b3fe6e73acb0ef1182` → 통합 `8c270745c966a0a9c67853fdd790959fa61669d7` (applied).
- 원 `59c6fd56d2ee25826bd4cd78374727c36074529c` → 통합 `e4f63853c0205a1489e832d2f84219ca1e2a33fa` (applied).
- 원 `a22d902ae8081ae6ec58d8d833527f79b7fa014f` → 통합 `420aa5d2a7e33c403a1bc257820217dad8f7223e` (applied).
- 원 `6f4de1630156fb6c772bb6a8aa94d938838dfa27` → 통합 `36ceb87890b4134e5df64890156753c1f86485fa` (applied).
- 원 `3a025062849a80115ab0e1dd1e0d9b9a70f8eb96` → 통합 `dc282d4b42cea652ad5239401c2ecf39b707d1b6` (applied).
- 원 `2525591a822c472eecc19dbda53e817730c6e7dc` → 통합 `cbcdeb7e5a266fdce4399b4c4f78239c3354518d` (applied).
- 원 `342b17371c3e98b8f87fa0d98ccaf7f26d687fe5` → 통합 `3c73244f7a0906d9997bf931d22472df7cc22a58` (applied).
- 원 `f58eb3ee4998dabcd7373a2b54f06b2bde2c57b9` → 통합 `5d6ff65610cc142be8a84a586ae8cff66cf9980b` (applied).
- 원 `8273ece22860b5793767b9888dd320b3bdc414b8` → 통합 `c67a3c2d5e7db31ab934c1c7b8aefeff746d8356` (applied).

[충돌·원 merge 해소 대조](../../working/task_m100_536_integration_review_stage1.md). 로컬 필수 검증을 완료했다. 원격 CI는 실제 통합 PR 생성 뒤 확인한다.

## 통합 후보 검증 연결

- 원 commit 적용과 merge 해소는 위 실제 SHA mapping 및 [1단계](../../working/task_m100_536_integration_review_stage1.md)를 따른다. 누적 source diff를 같은 PR의 고유 변경으로 반복 적용하지 않았다.
- 런타임/검사 분리는 `456ac3585`, 독립 Print 근거의 공통 극한 이름은 `d9a0b1c79`, 정식 원본 회귀는 `1ed02ebff`, font-request 반례·Print 보존은 `068769e9d`에 연결한다. 이 보정들을 원 contributor가 작성한 것으로 표시하지 않는다.
- 실제 실행 결과·source/binary/font/PDF hash와 직접 판독은 [5단계](../../working/task_m100_536_integration_review_stage5.md)·[6단계](../../working/task_m100_536_integration_review_stage6.md)에 있다. 기본 전체 nextest는 10,643 PASS / 0 FAIL / 45 skipped로 끝났다. Native 집중 277 PASS, 공식 Skia lib 4,043 PASS와 focused 2+4 PASS, format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 완료했다. 원격 exact-head CI는 미실행이며 이후 실제 결과로 trailing 기록을 갱신한다.
- 최종 판정과 merge 후 원 PR comment 계획은 같은 번호의 개별 review에 유지한다. 통합 PR 번호의 별도 review를 만들거나 contributor branch를 rewrite하지 않는다.

## 최신 base 리베이스

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 직후 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`. [7단계](../../working/task_m100_536_integration_review_stage7.md)의 새 head 검증은 진행 중이다. 위 실제 적용 SHA는 리베이스 전 기록이며 아래는 현재 대응이다.

- 이전 통합 `a50c25b3aa332a46a14a1f349d03fe82b8ad5053` → 현재 `d7d9cd163fb8bf7753c710eaf67d67f58466a27b`.
- 이전 통합 `991701fa59b4575fa8be45a0d0f7a6280d95c122` → 현재 `4036fe0621779bc5ca1613665d9dc3dbc8e6eae9`.
- 이전 통합 `5e550e12b679403ba11fa6d4299791234f1d5758` → 현재 `c028112d58bac4dd8c15598a12d58f76a3032d1b`.
- 이전 통합 `4707a41a973ab272784a5c3f991e1bbf15728fa8` → 현재 `4771ca54eb997150eda6da005d72bc0fc194b338`.
- 이전 통합 `e467f08c4daa7f5db9b0fe73850795180ad698b5` → 현재 `cc5580b383f2fcd3035389198410a6d15031c189`.
- 이전 통합 `b72a5f2f67acff72238feadd110857d00eb5ac2c` → 현재 `8f7928b2018b38b87aa595a731d8a88f0173f583`.
- 이전 통합 `bda7a209a8b9c2a068f2b78488120042180a3ac9` → 현재 `11ec3a822a0552b65a8b4584c55c3d6ac95ca2ef`.
- 이전 통합 `1677240efaf4be79c6bebb75b1358b423220326f` → 현재 `c6710f16152e808573d2887c165a5c5983624bfd`.
- 이전 통합 `caa893cd846a38da65cbc906e67bc6d5d3236879` → 현재 `674dcedd33062df9253e85a912bd21964c9be39f`.
- 이전 통합 `fc2335062aba5ebe847a21861121e27968ecd7ef` → 현재 `d121dccfbb4809bb28edc9488a8854dba5702bf0`.
- 이전 통합 `b3af1d45f6fa989e43753ab04ce30e72e3da243f` → 현재 `aad4fa8e2f860aeceb4b8aeb470aaa56b0c52075`.
- 이전 통합 `9060f3d5af21eb3a2fd8f749e7054fddb1a65ade` → 현재 `2e28a1e408bfd4a7cf1ea5f8f7547f534a12df07`.
- 이전 통합 `a566d272936f7ab52a7d650ee544330761312688` → 현재 `ab248b3a5717813286f4f19cdf40f793d8df152b`.
- 이전 통합 `5880e26495c31c4bac7fceb93ede75c0632c81f2` → 현재 `9ddf26f62ab919db2c19552021a24e432062c4a5`.
- 이전 통합 `e57e38e1a0c3e18cd6907e2b9defa22f3a32e9d9` → 현재 `47ad9166df438734d01404a41545cfcf355dfb2c`.
- 이전 통합 `b56c132beccf4d14a82b3a0f68ad3a5d631a6729` → 현재 `ef47e7d0418a00f27b3e42944e03596fd626ce12`.
- 이전 통합 `3b06b73a0c2ff9392cb4ee34dcdba2bd04829ee0` → 현재 `f28c546c6c3b555af66a3e770e24f8222048b435`.
- 이전 통합 `eb1fc7a32c1728d946d6cd28ac66396e4619187d` → 현재 `a7f25b5320aa6819b0065debd9be66bdd1b7d8d5`.
- 이전 통합 `56c2f1a984d87d46e4b9b9366b4a74cf38b3a070` → 현재 `cb728c1ce26ac4a48a4809311a143f25e944bb6c`.
- 이전 통합 `c840bc89fd85a592f51cee9d79e86ace20f9c8a1` → 현재 `5bda16181e4f9754e35d213990125aaa7efc7c6e`.
- 이전 통합 `8c270745c966a0a9c67853fdd790959fa61669d7` → 현재 `efdf4604165ad4581f79fc2d42c1a6c393692eb0`.
- 이전 통합 `e4f63853c0205a1489e832d2f84219ca1e2a33fa` → 현재 `81f72349bd59a2c48414bac4540ac04484de0f57`.
- 이전 통합 `420aa5d2a7e33c403a1bc257820217dad8f7223e` → 현재 `2b3e3a063dff34a7b7b8dbc12161d01ebd7db343`.
- 이전 통합 `36ceb87890b4134e5df64890156753c1f86485fa` → 현재 `396ae6ab0573e7068fd052891d3af331dcce72ba`.
- 이전 통합 `dc282d4b42cea652ad5239401c2ecf39b707d1b6` → 현재 `557e3b767c30d2cfb7f87ae4a0ec684fbca72c41`.
- 이전 통합 `cbcdeb7e5a266fdce4399b4c4f78239c3354518d` → 현재 `ad966934ed79742b5fa6046012e6fcf2fe8e1daf`.
- 이전 통합 `3c73244f7a0906d9997bf931d22472df7cc22a58` → 현재 `3d8dfd9bd1942972751b78d09d10ad104e9c0e68`.
- 이전 통합 `5d6ff65610cc142be8a84a586ae8cff66cf9980b` → 현재 `b78b8904dcacfae984da67abda8fa745ec01b6de`.
- 이전 통합 `c67a3c2d5e7db31ab934c1c7b8aefeff746d8356` → 현재 `b7bcf211f9e204b896a3d42b9ffa6d768a90abcc`.

## 최종 source의 원 commit 연결

- 원 `9c20ab17c08806fea8327eafdfc53775f18ee9e7` → 현재 `27d05a639c210112f6731a8dccdf2f72db3e6953`.
- 원 `8360eed403f14c0bc271643edc512adf9e2705ff` → 현재 `673fc6ef540c50fa92efd7d21474e7fe5be22cd1`.
- 원 `8adbbac59cbadd02e49afcba6f416ff251308509` → 현재 `c9fd7149fd89167a5f42871e690ed524655520e7`.
- 원 `2743e925a211ea8753e82d88896b57203f8fbd41` → 현재 `63d0e5c3ac6a0b697dad052fcb8fe746c8651e9b`.
- 원 `1b5b5a5213ec9333033e6241301b8c899df0337f` → 현재 `466996670b8bd5af0705fb2de5a7045045f892e8`.
- 원 `5458b4d7f9281449bf23e879cb7848f17afcffbe` → 현재 `827291e399146bf4c4863b9b4eb13ba8b28d4f26`.
- 원 `3c14e8ca9f6ea3f75f63a1e7f55a3a4b12b2d70f` → 현재 `14511227fa766df019c78bbd6183cd8ad0971918`.
- 원 `bc8ab82e8e5a060ebee67bea4d25146ff4ce37ec` → 현재 `1aef53b43c9678b2d4b0512cef280370c1ae8abc`.
- 원 `8b3debe2950f89c7bf8e2c40532b1b4db7ed4650` → 현재 `7170a6cf675bcdf1a91844cb599e1066dbc00f9f`.
- 원 `65ac4f709af96b35a127bb8ed0689620bf2f9e91` → 현재 `ac4fcd235f60aaccc359f55cfb3e5c6178dcb6e1`.
- 원 `abd57190925e99cd12d5570c24af5bb1fb55f68c` → 현재 `b29822af292f4ad1ecba9c6d8d67e3d7c54b4eb3`.
- 원 `f47d6fd9e0de0eeb27f135456902385078f0bae8` → 현재 `d150b2abf504b004a066c556999a30d362c5b2db`.
- 원 `a8d60bb85df67bd74528277273cf7edd19ba46b0` → 현재 `1ce32b215a2eecea297e1ee254bc0d74328671c9`.
- 원 `987b47874a40487f2b6e531e472f4f28d39aef83` → 현재 `a152d203c5ec45920e31597249bb5b81009ade66`.
- 원 `c6c6d43a1ad38bdce4793787a6934e8873a5dcc1` → 현재 `8f790685c54b120fd67926a13e583d942972828e`.
- 원 `f559227abc6f8818a356ffd995fc5727e3e30dac` → 현재 `f3aa10a090ca96594f0a29c10f2be4aa0f122438`.
- 원 `95857610ca11acde429a0b031d9d11e9845005b3` → 현재 `5f63eff1e1ef3a523b0293e83d5bdd3ead367f4b`.
- 원 `5cb8bf46451abda43f031b4934e07fa22b76004c` → 현재 `3902bed894d2ab9300983174ca3b5d77a638740c`.
- 원 `a4d61aa1758570052824e64946e0fc0b0be512d1` → 현재 `05f56f55347fcc07feae28dd945f8baaea041e8b`.
- 원 `60db4c7a47f3a17af324a81b645cd260e1fd86d7` → 현재 `2e1f46da0c885e0356ae83a7b26a3eb6c8baad11`.
- 원 `d4b39efd31808524a19043b3fe6e73acb0ef1182` → 현재 `566d34d923c6a41fa82c698792c78e02a1c832fa`.
- 원 `59c6fd56d2ee25826bd4cd78374727c36074529c` → 현재 `d9fb6d874ceb8d955e7b7beac311d1c2147c1ad4`.
- 원 `a22d902ae8081ae6ec58d8d833527f79b7fa014f` → 현재 `dc97369936e646d81f84d50639a8b24b666275e1`.
- 원 `6f4de1630156fb6c772bb6a8aa94d938838dfa27` → 현재 `299b5b67b1eab7b856c2a91056fc1e1e4530766b`.
- 원 `3a025062849a80115ab0e1dd1e0d9b9a70f8eb96` → 현재 `30df5a1c363fa69b1320aab2fe0e05fc5d73b9f9`.
- 원 `2525591a822c472eecc19dbda53e817730c6e7dc` → 현재 `91b203f0d5e12010ab71f340e439afcb11541d92`.
- 원 `342b17371c3e98b8f87fa0d98ccaf7f26d687fe5` → 현재 `2038c3cd88896abfd27ec01643f4b9b57b7686b5`.
- 원 `f58eb3ee4998dabcd7373a2b54f06b2bde2c57b9` → 현재 `4d46e208f410e891dcc3a6c867c4e5f35fe13f29`.
- 원 `8273ece22860b5793767b9888dd320b3bdc414b8` → 현재 `e8882b9b176aad5c376dbdd46799ef033d9770e8`.

## 최신 devel 통합 검증

현재 producer `3535106051394672e93ca8401ff9eae8b449ec20` /base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`의 전체 회귀10779PASS와 실제 소비 경로 검증은 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다.원 contributor author와164개 리베이스 commit의 순서·제목을 보존했다.원격 CI·merge 결과는 실제 확인 뒤 같은 PR trailing 기록에 추가한다.

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 review·오늘할일 trailing head CI와 normal merge는 다음 단계다.
