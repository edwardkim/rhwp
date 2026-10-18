# PR #7892 통합 검토 구현 계획

- 원 head `6abf22be0cb0fa10f0a5b9dfe57bdd2539e5df4f`, 기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- [공통 실행 계획](../../plans/pr_review_seo_rii_20261017.md)의 순차 gate를 따른다.
- 고유 적용 대상 30개. 아래 SHA는 원 author를 보존해 오래된 순서로 적용한다.
- merge commit과 다른 PR에 이미 속한 commit은 중복 적용하지 않는다.
- 충돌·보정·검증 결과는 [개별 review](pr_7892_review.md)에 연결하며 검증 전 완료를 단정하지 않는다.
- rollback은 통합 PR 제출 전 이 작업 branch에서만 판단한다. 사용자 source/fork/history를 재작성하지 않는다.

## 원 commit 순서

- `5e92cc590df9912f12785f877edf008c8927c344`
- `73a20886174be7be999d1e8ac6f5bf6437c06d01`
- `6ef8c29ab6e91d29c75c5a1b710cd012c0aff653`
- `5486c3d18f14c864a5eb1080b630ba37c3f22536`
- `17702bc78382f86c5a44b89f88634429d218a204`
- `2b9e7d646228094ba0738a5ae1e3c4f91b31a62a`
- `11e0ab0ca4b9e6f7fb17060f41cb94a6fafb0970`
- `2155dae0eb5940108e84ffce866a01cb9c4eb8c0`
- `3325e67e572e6b5a064800df15628119cc9b613a`
- `ab896205e10f53a5fe077da587eaa23d61fc28c8`
- `5b9f3383f104d48d56122aedc65760d4bda40b42`
- `1af3cb865997cc74346af1882a0566f7d8403179`
- `09e53a528cee6da89eba0e197155d08121941b49`
- `4acab69184ad703728f2e78532d3572ba20d7ee6`
- `ace448b4fb726c378ad3d8b849d428b6ad0715d2`
- `b344cac53a20d73cf575ce34a705c6cabd72ecf6`
- `e8d3ea804c5204f5cbc44c000590cb75e4baac34`
- `40f17906614703bfe6ab15194d1548f89d1321cc`
- `d10f419c6b10caf3f7022ed4f4bf50d46b83a534`
- `369c2efa70fc6ce90d8b5a05ada1b3fd835f6b85`
- `592d6aa4f86f54038eb5c0ca833d5b184ecf7b82`
- `83d136f59b7e6221e5e776913a37cc4bad6c6a49`
- `68f7f2ab01b72ba98b20d57e1c20a14857014ae7`
- `189fe1fe9725b4a63059987937a59fffeeeeff47`
- `1916c847d5fd4f6ba9a6737530884b87adba4094`
- `0cb7f9daad6f3137b2df1214c1d58a12c02ff178`
- `b2235e75ee154e3deccc2ebf811c0e4d5eff8c83`
- `36f7d36e728ea9b9a68ec2ad07d353b21ff565ac`
- `eb95f78925f3d29430d51868b862f2679a6fc6ac`
- `a4b2f5ef082554c549f76d1e51e08a95b2d623d2`

## 실제 적용 결과

- 원 `5e92cc590df9912f12785f877edf008c8927c344` → 통합 `946e0ffcbbaba5c8370b15f6beb366ca8405f087` (applied).
- 원 `73a20886174be7be999d1e8ac6f5bf6437c06d01` → 통합 `8be2c108b422bd493b2e865c4292f43f34017ba6` (applied).
- 원 `6ef8c29ab6e91d29c75c5a1b710cd012c0aff653` → 통합 `32eba39c2539a957a43afc1ee355733af257de83` (applied).
- 원 `5486c3d18f14c864a5eb1080b630ba37c3f22536` → 통합 `469f6ad5f3e22ef6e23cd9ffc230a327d576460f` (applied).
- 원 `17702bc78382f86c5a44b89f88634429d218a204` → 통합 `6bf5b46f8723c138d0c72ca897ac6e486400471e` (applied).
- 원 `2b9e7d646228094ba0738a5ae1e3c4f91b31a62a` → 통합 `adcfc163a857344acd0a81bde5b79781a7882cc4` (applied).
- 원 `11e0ab0ca4b9e6f7fb17060f41cb94a6fafb0970` → 통합 `611ba1048275559c9eebaba9e7d8a8ac1c7a1b81` (applied).
- 원 `2155dae0eb5940108e84ffce866a01cb9c4eb8c0` → 통합 `9562c08d35d5bbd3ca99a1c5864d4e46f3276856` (applied).
- 원 `3325e67e572e6b5a064800df15628119cc9b613a` → 통합 `1c49be70f8827fee666bbe70801d1b4feb785212` (applied).
- 원 `ab896205e10f53a5fe077da587eaa23d61fc28c8` → 통합 `5894336c2f116089c29076eb4a684cd2303bbdb0` (applied).
- 원 `5b9f3383f104d48d56122aedc65760d4bda40b42` → 통합 `a68ed94fc197831c6d70e8b546bafaa6f924b6a4` (applied).
- 원 `1af3cb865997cc74346af1882a0566f7d8403179` → 통합 `e99f38c53b27777422d299591ce8566b51f79a5c` (applied).
- 원 `09e53a528cee6da89eba0e197155d08121941b49` → 통합 `2c8a3efd51585daca8d77b0b8430cf688c43ba3e` (applied).
- 원 `4acab69184ad703728f2e78532d3572ba20d7ee6` → 통합 `6dcaa2738a22ebbcd4935e8e9f33e0fb53ea5501` (applied).
- 원 `ace448b4fb726c378ad3d8b849d428b6ad0715d2` → 통합 `737d73ec2e85b083a4012d4fd9963f95f4f51500` (applied).
- 원 `b344cac53a20d73cf575ce34a705c6cabd72ecf6` → 통합 `a073d85f51b25868e48ef0d68d76b0d5abc633ff` (applied).
- 원 `e8d3ea804c5204f5cbc44c000590cb75e4baac34` → 통합 `73ca862db5d41fd4d43805cfefe56fb099b59869` (applied).
- 원 `40f17906614703bfe6ab15194d1548f89d1321cc` → 통합 `38439d1b0c3ea5949fb94272610d864b5b3f95c4` (applied).
- 원 `d10f419c6b10caf3f7022ed4f4bf50d46b83a534` → 통합 `3acf33dde2f392bfeeff0ae13a0f467749de89a0` (applied).
- 원 `369c2efa70fc6ce90d8b5a05ada1b3fd835f6b85` → 통합 `3f5f6ad3b7f32780da33b6980f10ee105412475e` (applied).
- 원 `592d6aa4f86f54038eb5c0ca833d5b184ecf7b82` → 통합 `84351bd351323e4ea8ab2a95a61ed3430b863b68` (applied).
- 원 `83d136f59b7e6221e5e776913a37cc4bad6c6a49` → 통합 `e9289352a434a0f6f2ed0deb9012e4c11597d31f` (applied).
- 원 `68f7f2ab01b72ba98b20d57e1c20a14857014ae7` → 통합 `454b95e14825a8583310ad42cdcfcdcbfd222bcb` (applied).
- 원 `189fe1fe9725b4a63059987937a59fffeeeeff47` → 통합 `9d7ff57aa0373682ca898f80ce89a0881d6ea7b2` (applied).
- 원 `1916c847d5fd4f6ba9a6737530884b87adba4094` → 통합 `3e6ba71275aa8a49398f8f7799fd38047ef64461` (applied).
- 원 `0cb7f9daad6f3137b2df1214c1d58a12c02ff178` → 통합 `899ec68474b6f3fa19ecad40ab2d3682a0020e05` (applied).
- 원 `b2235e75ee154e3deccc2ebf811c0e4d5eff8c83` → 통합 `7130347f4bfe4761d0b76e9022f7fd8a1953cea6` (applied).
- 원 `36f7d36e728ea9b9a68ec2ad07d353b21ff565ac` → 통합 `a9c364df2d4d641ebff0886b74ed64c59df1cb8a` (applied).
- 원 `eb95f78925f3d29430d51868b862f2679a6fc6ac` → 통합 `2b9b1cecaf5d969bace858ba5e7f6bb9263b4d6b` (applied).
- 원 `a4b2f5ef082554c549f76d1e51e08a95b2d623d2` → 통합 `6e18d0ec0ab252ce536b9c3b95a3faa6daaf53a9` (applied).

[충돌·원 merge 해소 대조](../../working/task_m100_536_integration_review_stage1.md). 로컬 필수 검증을 완료했다. 원격 CI는 실제 통합 PR 생성 뒤 확인한다.

## 통합 후보 검증 연결

- 원 commit 적용과 merge 해소는 위 실제 SHA mapping 및 [1단계](../../working/task_m100_536_integration_review_stage1.md)를 따른다. 누적 source diff를 같은 PR의 고유 변경으로 반복 적용하지 않았다.
- 런타임/검사 분리는 `456ac3585`, 독립 Print 근거의 공통 극한 이름은 `d9a0b1c79`, 정식 원본 회귀는 `1ed02ebff`, font-request 반례·Print 보존은 `068769e9d`에 연결한다. 이 보정들을 원 contributor가 작성한 것으로 표시하지 않는다.
- 실제 실행 결과·source/binary/font/PDF hash와 직접 판독은 [5단계](../../working/task_m100_536_integration_review_stage5.md)·[6단계](../../working/task_m100_536_integration_review_stage6.md)에 있다. 기본 전체 nextest는 10,643 PASS / 0 FAIL / 45 skipped로 끝났다. Native 집중 277 PASS, 공식 Skia lib 4,043 PASS와 focused 2+4 PASS, format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 완료했다. 원격 exact-head CI는 미실행이며 이후 실제 결과로 trailing 기록을 갱신한다.
- 최종 판정과 merge 후 원 PR comment 계획은 같은 번호의 개별 review에 유지한다. 통합 PR 번호의 별도 review를 만들거나 contributor branch를 rewrite하지 않는다.

## 최신 base 리베이스

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 직후 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`. [7단계](../../working/task_m100_536_integration_review_stage7.md)의 새 head 검증은 진행 중이다. 위 실제 적용 SHA는 리베이스 전 기록이며 아래는 현재 대응이다.

- 이전 통합 `946e0ffcbbaba5c8370b15f6beb366ca8405f087` → 현재 `da8cfee1f5b432b561f8c262dd0c1f2288c49fd3`.
- 이전 통합 `8be2c108b422bd493b2e865c4292f43f34017ba6` → 현재 `5fb931aea2da508bdcd358d108bbe8b92999bb46`.
- 이전 통합 `32eba39c2539a957a43afc1ee355733af257de83` → 현재 `6d4dad79731f66274f64a76923b1d95b3506246d`.
- 이전 통합 `469f6ad5f3e22ef6e23cd9ffc230a327d576460f` → 현재 `ea3f13401a1a6b6bbe4135d1db1ea1334144bbda`.
- 이전 통합 `6bf5b46f8723c138d0c72ca897ac6e486400471e` → 현재 `a15f30af23a56cfb5005247fbc830d8af66f577c`.
- 이전 통합 `adcfc163a857344acd0a81bde5b79781a7882cc4` → 현재 `08030544d4006599198f9871e9ea14180b7a54dd`.
- 이전 통합 `611ba1048275559c9eebaba9e7d8a8ac1c7a1b81` → 현재 `fb64ee326745a1b4936f7c0d2c9cd1740f317ec2`.
- 이전 통합 `9562c08d35d5bbd3ca99a1c5864d4e46f3276856` → 현재 `2d263012d0dc26c6d225580766acf1fa7d5a8be4`.
- 이전 통합 `1c49be70f8827fee666bbe70801d1b4feb785212` → 현재 `2ea02024f6be73f5d57cf1636ab46d5e2c4bfcd9`.
- 이전 통합 `5894336c2f116089c29076eb4a684cd2303bbdb0` → 현재 `985472b47c68a38618c69eb24eb0ac65e527dfd8`.
- 이전 통합 `a68ed94fc197831c6d70e8b546bafaa6f924b6a4` → 현재 `fad7d5fc5241b41f794fd74749dd11cf61d07d3f`.
- 이전 통합 `e99f38c53b27777422d299591ce8566b51f79a5c` → 현재 `0425f603d0f39d6b6c2e322c51a08da99fd71568`.
- 이전 통합 `2c8a3efd51585daca8d77b0b8430cf688c43ba3e` → 현재 `cc06b86f70205940349a992993bd3f6e45b654af`.
- 이전 통합 `6dcaa2738a22ebbcd4935e8e9f33e0fb53ea5501` → 현재 `84696fe7de5e004f9882f3488a151f57ca45cee2`.
- 이전 통합 `737d73ec2e85b083a4012d4fd9963f95f4f51500` → 현재 `24465ba6538f86c501eb2666c2ee6fd8b5731ab1`.
- 이전 통합 `a073d85f51b25868e48ef0d68d76b0d5abc633ff` → 현재 `a421cbb81b40cf95f958d1150b9e6d35176880ee`.
- 이전 통합 `73ca862db5d41fd4d43805cfefe56fb099b59869` → 현재 `0143d431a5b329b923f599171c11a4daa58f1e10`.
- 이전 통합 `38439d1b0c3ea5949fb94272610d864b5b3f95c4` → 현재 `5711514288cd8cb90c4c89e40c552e02bc5f90ea`.
- 이전 통합 `3acf33dde2f392bfeeff0ae13a0f467749de89a0` → 현재 `01313ece0f60f230c2f001abccb6774f91bede65`.
- 이전 통합 `3f5f6ad3b7f32780da33b6980f10ee105412475e` → 현재 `1cccc265f900e2c6d52ce4391abf5a6db050ff31`.
- 이전 통합 `84351bd351323e4ea8ab2a95a61ed3430b863b68` → 현재 `a03dbb7098df897a0b0c579eb0af796272e36cf0`.
- 이전 통합 `e9289352a434a0f6f2ed0deb9012e4c11597d31f` → 현재 `13ef3c18f1c0603bbc834cd62a93d82e07aa4f2f`.
- 이전 통합 `454b95e14825a8583310ad42cdcfcdcbfd222bcb` → 현재 `b743ccd4ad7ff57172e6d243f256249f740d4460`.
- 이전 통합 `9d7ff57aa0373682ca898f80ce89a0881d6ea7b2` → 현재 `580e2ed340f39df54bfc740f6baaca9167165655`.
- 이전 통합 `3e6ba71275aa8a49398f8f7799fd38047ef64461` → 현재 `e27d166e8b4bd3c98d10663502dc0b587b7727f5`.
- 이전 통합 `899ec68474b6f3fa19ecad40ab2d3682a0020e05` → 현재 `2da5c2ce638fb46c7432f5031013f694d11c9086`.
- 이전 통합 `7130347f4bfe4761d0b76e9022f7fd8a1953cea6` → 현재 `96c327bb9b7ad529f632707ea71ecabba193cadd`.
- 이전 통합 `a9c364df2d4d641ebff0886b74ed64c59df1cb8a` → 현재 `40ef7bfdef6b4ded708cbe285b36c84759c3022d`.
- 이전 통합 `2b9b1cecaf5d969bace858ba5e7f6bb9263b4d6b` → 현재 `9e732b630d1d46b987c2b34d9eab50d98a171398`.
- 이전 통합 `6e18d0ec0ab252ce536b9c3b95a3faa6daaf53a9` → 현재 `b7e2c61ec17df5c37f6adc77f30327767b082fcf`.

## 최종 source의 원 commit 연결

- 원 `5e92cc590df9912f12785f877edf008c8927c344` → 현재 `476c47589b85f1bcc7a52bd2b00bed9ceeb3cff4`.
- 원 `73a20886174be7be999d1e8ac6f5bf6437c06d01` → 현재 `ddeb7e033f6e4c1bfcd8ef0e7136dbc881f47cd6`.
- 원 `6ef8c29ab6e91d29c75c5a1b710cd012c0aff653` → 현재 `688e1833f78940041542888213ada9cf4abfd088`.
- 원 `5486c3d18f14c864a5eb1080b630ba37c3f22536` → 현재 `59848c2649e4486e59987154adf2289146dbdc6e`.
- 원 `17702bc78382f86c5a44b89f88634429d218a204` → 현재 `c6ee58580f6c78ac4ef031df4d3bf9d0a59fb09c`.
- 원 `2b9e7d646228094ba0738a5ae1e3c4f91b31a62a` → 현재 `6f6a5a5d09878b91dce23d25bd088fe8b7cb9d23`.
- 원 `11e0ab0ca4b9e6f7fb17060f41cb94a6fafb0970` → 현재 `ca5b3406e73f2818a39a8b7c6b57f72fdef8c2ac`.
- 원 `2155dae0eb5940108e84ffce866a01cb9c4eb8c0` → 현재 `2aa264f87edf768fa6c3fb8de827b79d735aa29d`.
- 원 `3325e67e572e6b5a064800df15628119cc9b613a` → 현재 `76673578cddd40e27ea5c688f80ef5c5d3710676`.
- 원 `ab896205e10f53a5fe077da587eaa23d61fc28c8` → 현재 `219ffc63151c35ab3018ee3f72ee49bc3cdf631e`.
- 원 `5b9f3383f104d48d56122aedc65760d4bda40b42` → 현재 `26eb4c63bbe53c66618494d0f6c4e29936fef78f`.
- 원 `1af3cb865997cc74346af1882a0566f7d8403179` → 현재 `ab49748020593a574f418bfd076ea09d2100007f`.
- 원 `09e53a528cee6da89eba0e197155d08121941b49` → 현재 `358e853330108e68ad00b5d3fa96d062088fb3da`.
- 원 `4acab69184ad703728f2e78532d3572ba20d7ee6` → 현재 `19e9aae7a834bae5a7337617effeb2fb9edaf0c2`.
- 원 `ace448b4fb726c378ad3d8b849d428b6ad0715d2` → 현재 `ac9875214b2739de14d4687c60e5065216acdf48`.
- 원 `b344cac53a20d73cf575ce34a705c6cabd72ecf6` → 현재 `0dcad27b691d19a46d82f348cb8d8612238aa8e1`.
- 원 `e8d3ea804c5204f5cbc44c000590cb75e4baac34` → 현재 `ba048bfa8d3f00508ee20e8246ea8ab9ed53a6aa`.
- 원 `40f17906614703bfe6ab15194d1548f89d1321cc` → 현재 `391b4c6516d0be3a73c32bd8ed76e9aa2d24d701`.
- 원 `d10f419c6b10caf3f7022ed4f4bf50d46b83a534` → 현재 `ad5112a418556de23ecd72f292208ad4ac2b71fe`.
- 원 `369c2efa70fc6ce90d8b5a05ada1b3fd835f6b85` → 현재 `e6c197ca69cfdf9e5b158fb1ff2d8628e76fc395`.
- 원 `592d6aa4f86f54038eb5c0ca833d5b184ecf7b82` → 현재 `58ac5013c70eec95437189638845f35304005871`.
- 원 `83d136f59b7e6221e5e776913a37cc4bad6c6a49` → 현재 `020bac73afd8b454ceca22ff148870359b5b8740`.
- 원 `68f7f2ab01b72ba98b20d57e1c20a14857014ae7` → 현재 `fe206229299fc739f330438abb6f4b08aaadbc08`.
- 원 `189fe1fe9725b4a63059987937a59fffeeeeff47` → 현재 `ea146db420ec99086c9aa4d1cb28478c994bb6e8`.
- 원 `1916c847d5fd4f6ba9a6737530884b87adba4094` → 현재 `1935dca005e2d8f1d16bad17c28b4ec49cafed40`.
- 원 `0cb7f9daad6f3137b2df1214c1d58a12c02ff178` → 현재 `5538eb3b87757884dceeaf2e24399c7f579f1517`.
- 원 `b2235e75ee154e3deccc2ebf811c0e4d5eff8c83` → 현재 `4251f01cf66d3f35da55e8963e66edbb6a6573cf`.
- 원 `36f7d36e728ea9b9a68ec2ad07d353b21ff565ac` → 현재 `f3462ac4d33ca6ef591be4f2fc6f48bb7ef9c9a5`.
- 원 `eb95f78925f3d29430d51868b862f2679a6fc6ac` → 현재 `0b549e080ecd4b01babdc9e1d9ad6a06449721ad`.
- 원 `a4b2f5ef082554c549f76d1e51e08a95b2d623d2` → 현재 `b4900052de7db8d7c7cee623e4f24949b79838ce`.

## 최신 devel 통합 검증

현재 producer `3535106051394672e93ca8401ff9eae8b449ec20` /base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`의 전체 회귀10779PASS와 실제 소비 경로 검증은 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다.원 contributor author와164개 리베이스 commit의 순서·제목을 보존했다.원격 CI·merge 결과는 실제 확인 뒤 같은 PR trailing 기록에 추가한다.

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 review·오늘할일 trailing head CI와 normal merge는 다음 단계다.
