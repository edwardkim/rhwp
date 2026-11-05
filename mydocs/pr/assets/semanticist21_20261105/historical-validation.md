# 이전 단계의 검증 기록

현재 R6 완료 증거는 [최종 전쪽 측정](validation-r6/README.md)과 개별 PR 리뷰다.
이 문서는 이전 제품의 분석·실패·보정과 최종 제품의 검증을 혼동하지 않기 위한 기록이다.
원시 JSON·실행 로그는 제출하지 않으며 아래 실제 결론과 source를 기록한다.

## 이전 적용 기록

아래는 ECE9 시점의 적용 SHA다. 이후 170b 리베이스의 실제 대응 SHA는 원 PR별 리뷰의 체리픽 표를 따른다.
원 기여 44개 커밋을 각각 한 번 누적 적용했다. 메인터너 보정은 별도 일반 커밋이다.

| 원 PR | 원 commit | ECE9 적용 commit |
| --- | --- | --- |
| [PR #8526](https://github.com/edwardkim/rhwp/pull/8526) | `e1fc9e35fb9a923feb0a138627175f1d11d1ab3f` | `ef2e7f0e4b968d57d092a86fe9b1ef6016859878` |
| [PR #8526](https://github.com/edwardkim/rhwp/pull/8526) | `a4ce64e250512153d3db6e27d88bb9448f120ee2` | `80398d7db157516ceed8437d8a3a1445a3c7a0eb` |
| [PR #8527](https://github.com/edwardkim/rhwp/pull/8527) | `1d6c685106306a297cc1fcf2d09b3aaf56732969` | `3db4b1213b5055df886963f6cf67b6dda3318319` |
| [PR #8527](https://github.com/edwardkim/rhwp/pull/8527) | `7243067034a9cb2d64bb0e6f8938f496461b1c34` | `99f6cc16c17c6232315df2346524283cd88c06c8` |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | `b5a228ff54f9307eb7e6c7d8999c5e6f775b0e3e` | `d376d6fcd1a923fb1fe76bd74290c25faca4f144` |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | `06d4928479fe238f9d63948e6d4edaf7e3835fe2` | `6add9f68d9bff732b77195b17e501cde199ae4d1` |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | `6640ee6cf49a694ed7c97cc6c6836c7f20af691e` | `ed11233edd0ff2ee298d5db003b4613c4d387a74` |
| [PR #8528](https://github.com/edwardkim/rhwp/pull/8528) | `6785bddfb1047ea6e3e74b7fc2eec94efd0d7ce6` | `3a2b6093dc5116e9a0bcf617f8038bf090bf033a` |
| [PR #8536](https://github.com/edwardkim/rhwp/pull/8536) | `3c76eabef330286d9c9ec236e808c263a37dccaf` | `1caf817d638fe970b9c0953a68793e40a1491001` |
| [PR #8536](https://github.com/edwardkim/rhwp/pull/8536) | `0dc6e32b87b0097a9f139f8428b6d17470a31a7f` | `9fb66a16331ca540f099e54f05566481f01e8540` |
| [PR #8537](https://github.com/edwardkim/rhwp/pull/8537) | `2469c0b13ad4acf290d20bb721b93320766d73d4` | `0bd0a1c889a0bcbf5c62dc0b948382f4a2f73173` |
| [PR #8537](https://github.com/edwardkim/rhwp/pull/8537) | `6c2369bd25c28456b01e629f66c18c7c60176fc7` | `e31664535187bba6202c3fd1f7ee6256a4a9a674` |
| [PR #8538](https://github.com/edwardkim/rhwp/pull/8538) | `8a516f07240e5901561ebaa08849c6bf26d2422c` | `b3bd9daa10c4cf0ef114d5ee092383adc48d79e8` |
| [PR #8538](https://github.com/edwardkim/rhwp/pull/8538) | `39669dd94237454610fb6ca522258836912396a6` | `7cb31605463efe0f12c41e3fb99ec3722534d948` |
| [PR #8541](https://github.com/edwardkim/rhwp/pull/8541) | `ce799fbbde5eba7bf997705f75c0bc73994f72da` | `4a64ea1d4becdaef42504630511a04251de4afcd` |
| [PR #8541](https://github.com/edwardkim/rhwp/pull/8541) | `b8fefa3de9248ecc0aee0aa2c000a83e84be4d5c` | `28782dab9d326b2eb0c5a8a7768bdb97d860796a` |
| [PR #8541](https://github.com/edwardkim/rhwp/pull/8541) | `b7cc596d61a5cebb9b17fa0c1fa26ac5d2f4a007` | `8cc8113ce8ae9006437181383960e34d993a9d89` |
| [PR #8549](https://github.com/edwardkim/rhwp/pull/8549) | `560014d4554a6798f8524c00d1e261f89fcdca17` | `f92f8469d6a28d21f8610969348af1c8ec7cbeec` |
| [PR #8549](https://github.com/edwardkim/rhwp/pull/8549) | `5ccd6da2adbf628b8a885bf2b2b4fa0e2eb7191b` | `c5a830e55c6f7ef255bc882a20f7584f1b8577a9` |
| [PR #8558](https://github.com/edwardkim/rhwp/pull/8558) | `d86a2a607137170ac6df4c3c908940724a6a447c` | `6882f1c53af5db82b52676718da0c6c033dc551f` |
| [PR #8558](https://github.com/edwardkim/rhwp/pull/8558) | `6e508084575e95681db5dda2794b314f038aa685` | `e27434509160e0388bfef29bc56666c154d2e6bb` |
| [PR #8564](https://github.com/edwardkim/rhwp/pull/8564) | `9cd75cef7f52fe1a9b040a61632d4acc8d5d0212` | `923448e9f179f38f0f3eb9b50a26f634d5a45413` |
| [PR #8564](https://github.com/edwardkim/rhwp/pull/8564) | `5477f2cc6c4daf93ea315828650ae94eaf15b1b7` | `7d660661029da93505bb9d245ded8b99ffc09935` |
| [PR #8570](https://github.com/edwardkim/rhwp/pull/8570) | `8a4c8d8477929c89c84aab5ec6189ba7a7191cb6` | `2dc6bc74b45fb28d3c642d06909c56c98d8c1e06` |
| [PR #8570](https://github.com/edwardkim/rhwp/pull/8570) | `bd781342ce9ea546cabba540d57b74205c5b5448` | `12bfa0ce71b148cb76f1ecb8d33ec57280f983e3` |
| [PR #8571](https://github.com/edwardkim/rhwp/pull/8571) | `5d557479347c6218e4f5d991c1781a5461e751ad` | `c24e511035f3215abbb13f68e6c83be35297b11a` |
| [PR #8571](https://github.com/edwardkim/rhwp/pull/8571) | `dfef7b709773f56535ca1da123bb837eaad85700` | `7838f1322d4a01a62ced917ad03a27d189fa9f36` |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `9ded487923f1fa8d6e49e2ef1ef85fff4145d90c` | `6c6c46361080bfe7326905e520087e12ea0fadeb` |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `ee831a3465fd8d58c589031cc084fda95a28be80` | `7e1432ce70fe6a9652152a6c780eb428527836a2` |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `a9cd16ad09ba6a280a1e7a6456342a9741619f15` | `b01e73e221d9540a2737dafc8baa84a5ad6ae124` |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `410c5dba40d0dbdf27a6da6312e5530003a56bb0` | `6ccab9ad30dfe2cdc32960026c61d92a58a48ffb` |
| [PR #8572](https://github.com/edwardkim/rhwp/pull/8572) | `d9750b5604d3f23fd8645ae3f19dfc0cb293a70e` | `25d514236272a0d879a3b8df5cfce6237f3ba987` |
| [PR #8577](https://github.com/edwardkim/rhwp/pull/8577) | `6a16216a8c44961bff01aa555bed635c93fcbfe5` | `fa31c608dd450118f1ea6ef89d5615a3910dcdcd` |
| [PR #8577](https://github.com/edwardkim/rhwp/pull/8577) | `7af4ed39fbb8c06dec2b202eb29609f11ce4a379` | `21472c786109f82d81247757d538c7fb163d2ca3` |
| [PR #8586](https://github.com/edwardkim/rhwp/pull/8586) | `610f0e55c61fbf28a2f301c45163b76b47d8a406` | `10018283deeb9d9691ab2ef8a3182590961fa07a` |
| [PR #8602](https://github.com/edwardkim/rhwp/pull/8602) | `cad70bbaa4da6edbfb5348aa0fb91c83f1367241` | `ce18e03b8a9a7e23d996033dbfd9a087f4fc09b1` |
| [PR #8602](https://github.com/edwardkim/rhwp/pull/8602) | `338a75a1afd5bbe62843d71784a5c82b89613620` | `03c31d8b09d49d1cdced001f0951ebb6ccca1964` |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `70b3a3a937930e5752a271ed1d18c8efbf7cde14` | `08a7b672e547ad2c32f3c12173b2410f615c0781` |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `d0bddfa86f8f17a2dc2acefcd025484d8e7942d4` | `6f125e3d90b00c5bcb1ca17b27861b0521b9a286` |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `28213394d0df5c1f96d417f46088d8a4dd318350` | `d8565449bee222a7140311858d2c8c9c86e9706c` |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `c469db2ce8128f016b70a8268fb2cb646457e15c` | `edc3ead1a1f5bebcfc207d71c1be952d33808091` |
| [PR #8616](https://github.com/edwardkim/rhwp/pull/8616) | `5768ed5323238c5b1e1027eb0e0dcba6c62e26d8` | `75a989ff98a27a7b71a83e3fe947b0e1132d947d` |
| [PR #8629](https://github.com/edwardkim/rhwp/pull/8629) | `4c3c316ff627b5403f7b7c7b9df4f8cb82b83b84` | `3a86b4b2c98180286df93830c8d49451ed022c0a` |
| [PR #8629](https://github.com/edwardkim/rhwp/pull/8629) | `d38d02f03ffdc5b6461228394c1aa9f72ffff1f0` | `6bb2738bd4f8632535417e99ba0b972a3002fa7e` |

## 제품별 실제 결과의 구분

- 리베이스 전 제품 `809097923a0d8f10c24eb3d42572b2888954a121`: Native 56사례·136쪽의 이전 대조 출처다.
- 각주 선 보정 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`: note-formats HWP/HWPX 10·11쪽의 97.31/96.53% 하락을 양쪽 100.00%로 복구했다.
  원인과 대상 4행·당시 전쪽 TSV는 [각주 선 분석](rebase-ece9-note-repair/README.md)에 있다.
- 머리말 높이 계약 `a314d7a24d3234305abff43149f575853db31105`: 독립 한컴 Print를 확보한 6개 원본의 전쪽 양쪽 100%, 원본 최소 높이·내용 포함 관계 검사 4건 PASS.
- 셀 첫 문단 보정 `2de033b4e42be406a01ea0b9053a4531247e147c`: issue7344 원본에서 R3 71.72% → R4 Native/fresh WASM 100.00%.
  바깥 위 여백과 문단 간격을 원본 HWPUNIT에서 읽는 기존 모듈 4건 PASS. 최초 기대값 작성 오류와 실제 엔진 결함을 구분했다.
- R5 `71e5e0ae2a72a6b8f51682d131077bd4df44994f`: 최소 공개 글꼴 5개, 실제 전체 목록 55 binaries·12,735 PASS·45 ignored, 누락·중복 0.
  성능 1건을 독립 실행하고 나머지 12,734건을 상보 필터로 실행했다. 전체 목록과 실제 PASS 이름을 대조했다.
  이는 현재 R6 전체 검증 결과가 아니다.
- R5 실제 API: 12그룹 중 base 11 FAIL→후보 PASS. PR #8602 재열기는 양쪽 PASS여서 독립 CFB로 결함 검출을 보완했다.
  현재 저장본 bytes 동일성을 확인한 범위에서만 [현재 동작 기록](validation-r6/runtime-checks.md)으로 재사용한다.
- R6 `86709ac16711820e61baa3b87e2c4ddaac23573e`: 빈 번호 머리 거리 보정 뒤 원본 두 형식·양쪽 1쪽 raw 100%, 최소 글꼴 정식 회귀 8 PASS.
  새 Native/fresh WASM 각각 128입력·301쪽과 독립 Print 쪽수 일치, base 대비 하락 자체 0쪽.

## 재현 명령

공유 `target/pr-review`, 컴파일 jobs 8, 최소 공개 글꼴 5개, 테스트 스레드 12를 사용한다.
전체 명령은 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`다.
HML 0.5초 성능 계약은 다른 렌더링/컴파일을 종료한 뒤 단독 실행하며 허용치를 바꾸지 않는다.
두 상보 필터 `test(inflated_row_count_does_not_slow_down_parsing)`와 그 `not` 범위가 전체 실제 목록을 덮는지 확인한다.

## 이전 producer 신원

| 단계 파일 | 제품 source | Native SHA-256 | WASM SHA-256 |
| --- | --- | --- | --- |
| r4-native-producer.json | `2de033b4e42be406a01ea0b9053a4531247e147c` | `a8dd99014df3e6b7e16475d500c724f094bf36cdd99534ed3b4904aa3b99e489` | `해당 없음` |
| r4-wasm-producer.json | `2de033b4e42be406a01ea0b9053a4531247e147c` | `해당 없음` | `e43ac1efd53ce1083cf50ce3f6f506bb5f2bece55b143aa187e8ba817f866b46` |
| rebase-ece9-r2-candidate-wasm-producer.json | `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b` | `해당 없음` | `c63ee387825f6925b20ef584c3bea631e44ef0d2961ac4748deed15bf8a9d85d` |
| rebase-ece9-r2-native-producer.json | `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b` | `f564cb394403e02085eaf1888615671ee4b7468a6a435cc5db3da2adb310f669` | `해당 없음` |
| pr8549-r3-native-producer.json | `a314d7a24d3234305abff43149f575853db31105` | `78a4670707a43433cbf46125e22003ff444840f449c5f708bb74025ee6569626` | `해당 없음` |
| pr8549-r3-wasm-producer.json | `a314d7a24d3234305abff43149f575853db31105` | `해당 없음` | `3166eeefa19ba7e6ffd63dc236b78232cc496a9361f6991f18896249c28a82ee` |
| base-native-producer.json | `170b886f167b705b6e630c5b34bb638d92fae0a9` | `09a3d39f157d38c4a498c023e36a949bed23d2a480918fa54d847967c281bcc4` | `해당 없음` |
| base-wasm-producer.json | `170b886f167b705b6e630c5b34bb638d92fae0a9` | `해당 없음` | `8a420d0b2e7895187cd917eca0e193c245316cf5ccd84e938b431d636f4d7ee6` |
| r5-native-producer.json | `71e5e0ae2a72a6b8f51682d131077bd4df44994f` | `bb13db2a4011356ac678f19958a89a056fc10146e0646192076972e5bceb7d34` | `해당 없음` |
| r5-wasm-producer.json | `71e5e0ae2a72a6b8f51682d131077bd4df44994f` | `해당 없음` | `1d64c3307cf85a5fde220d501cc32c5ff45ded2b3c048b059b769c21f6e46bb9` |
| native-producer.json | `1da15804ebb5d6fc8f63b7ccb8410018bd42c29d` | `e52882bc5acda6cec799e1cf784861bfda514a867f55daf3b5e1f720b35109e1` | `해당 없음` |
| wasm-producer.json | `86709ac16711820e61baa3b87e2c4ddaac23573e` | `해당 없음` | `4755dbfeda5bba99d72191a54e90342ae381ec17f0dd840926bbf1fdd75d1f30` |
