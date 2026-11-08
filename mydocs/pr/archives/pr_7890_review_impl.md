# PR #7890 통합 검토 구현 계획

- 원 head `79942a3807d4066dd7cb44abda7697c19ffb614e`, 기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- [공통 실행 계획](../../plans/pr_review_seo_rii_20261017.md)의 순차 gate를 따른다.
- 고유 적용 대상 24개. 아래 SHA는 원 author를 보존해 오래된 순서로 적용한다.
- merge commit과 다른 PR에 이미 속한 commit은 중복 적용하지 않는다.
- 충돌·보정·검증 결과는 [개별 review](pr_7890_review.md)에 연결하며 검증 전 완료를 단정하지 않는다.
- rollback은 통합 PR 제출 전 이 작업 branch에서만 판단한다. 사용자 source/fork/history를 재작성하지 않는다.

## 원 commit 순서

- `9df25c8f22985486fb0e045f693b5663e8f90609`
- `447cb32e896a380f9c61e328b89a35e67dfbdba3`
- `7cf3f14ddd80d991bf75d6fbe006d4451547c3b4`
- `88cff3560f5052a446fb33b358b3c3f2e32f9f93`
- `084964c5dc54884834d56105ff4429a74c5ac13e`
- `b827287a5a7fdb9bf1968e880148763e48d13772`
- `65adfe356305eca0eac59b73e5a696c5a52758f5`
- `fe3cbd62ce2b49bc4ba10e27af132c338133f3b8`
- `7c4ae0e5383a55168b9cb689f35d247222bba24e`
- `dc6ce6fee23caebf03605496b49084ab47232795`
- `4e2741f70c4e5fe232b6fe4b5985a1b05a50e76a`
- `266117ee941cdccb48d4c039ac372b9a75ed1c6b`
- `493819d8b67a9adac3f881635aa8fabf545e5dc0`
- `ebb1bf918f9006ea7ef4b086018f6a4a988f1dc5`
- `e0604f0d206d84aceebc600ac555704feeda6edb`
- `f5892664440aa3e5a6d940882f3c1250cfc990af`
- `7f93f337c6a79719bdb14c261bc7eb2ae43a3719`
- `f10c215725926f1550cd92ef4eededb3a668e42e`
- `4a53eacd8a77fc18b92470bf32b6809aa22ca6dd`
- `e743bbda044290bac21421c3252479560009b4c1`
- `994e8a7835f22b6c0433d826c490dfb827532e42`
- `c533d85d03499faab411fe41eb64cd7d6a3b34f1`
- `1eade8b0dc6a82d8d4d1874868389bf5ec89276d`
- `6086d76e1e00ad078c769fc6e9b5d5faadf769b3`

## 실제 적용 결과

- 원 `9df25c8f22985486fb0e045f693b5663e8f90609` → 통합 `924da30d8848a33a28c361f349f97bb2a3285c2a` (applied).
- 원 `447cb32e896a380f9c61e328b89a35e67dfbdba3` → 통합 `1c311b8ac03ef9ff3d91d66941af6a9123f7cada` (applied).
- 원 `7cf3f14ddd80d991bf75d6fbe006d4451547c3b4` → 통합 `bd47a160c6e2425f93b41cbd0f378e2e044ecaf9` (applied).
- 원 `88cff3560f5052a446fb33b358b3c3f2e32f9f93` → 통합 `66f2c9da3bd1e0f605c8e42fa077596d9b641901` (applied).
- 원 `084964c5dc54884834d56105ff4429a74c5ac13e` → 통합 `ae4ad65828a720eca2a51726bcbb0d6e73c53147` (applied).
- 원 `b827287a5a7fdb9bf1968e880148763e48d13772` → 통합 `8684212b359e538eb6d16b8ba0ad5bca02a61df3` (applied).
- 원 `65adfe356305eca0eac59b73e5a696c5a52758f5` → 통합 `53fad1ef327aed56e18ed4c009a31db67a0f6385` (applied).
- 원 `fe3cbd62ce2b49bc4ba10e27af132c338133f3b8` → 통합 `6d4a146e6e9b9a10d0975e90f022fc25e49bcfc7` (applied).
- 원 `7c4ae0e5383a55168b9cb689f35d247222bba24e` → 통합 `f8aa165d8245ff4e548869e4b362f3a34d77c851` (applied).
- 원 `dc6ce6fee23caebf03605496b49084ab47232795` → 통합 `c857fffa6c27d06ecd111d685bcd0b6e03c9dac1` (applied).
- 원 `4e2741f70c4e5fe232b6fe4b5985a1b05a50e76a` → 통합 `c69e4df4322a5560d773151841f56f69f24744cc` (applied).
- 원 `266117ee941cdccb48d4c039ac372b9a75ed1c6b` → 통합 `8887676a193c5ee9aec8166fed8320e5213874e0` (applied).
- 원 `493819d8b67a9adac3f881635aa8fabf545e5dc0` → 통합 `6f0523df1ebef65d0e8ddfc5ecab687de6e93660` (applied).
- 원 `ebb1bf918f9006ea7ef4b086018f6a4a988f1dc5` → 통합 `696ab603f4930ec1ac899ad6e0047e09a98420a4` (applied).
- 원 `e0604f0d206d84aceebc600ac555704feeda6edb` → 통합 `b55e1396580047166dfc5ae9fbd92cad15753dbc` (applied).
- 원 `f5892664440aa3e5a6d940882f3c1250cfc990af` → 통합 `cfce101094507e6c85ef60af51c8100a646b3600` (applied).
- 원 `7f93f337c6a79719bdb14c261bc7eb2ae43a3719` → 통합 `8394eaf645d861427094c55e9f8d6a6aecdd3342` (applied).
- 원 `f10c215725926f1550cd92ef4eededb3a668e42e` → 통합 `37dcf275ebf71ae920cd780ccd3bec459dac86c2` (applied).
- 원 `4a53eacd8a77fc18b92470bf32b6809aa22ca6dd` → 통합 `f1850e72c78ab8eecfedb0061505f63dc9c901f9` (applied).
- 원 `e743bbda044290bac21421c3252479560009b4c1` → 통합 `7daad4d49599f62a78016d133346d29fcaabcaed` (applied).
- 원 `994e8a7835f22b6c0433d826c490dfb827532e42` → 통합 `d20c87bbdaa64b2939399504704d0d21dd61ced1` (applied).
- 원 `c533d85d03499faab411fe41eb64cd7d6a3b34f1` → 통합 `56115aeeb9a6a400906eb20aa4222e2e99528c48` (applied).
- 원 `1eade8b0dc6a82d8d4d1874868389bf5ec89276d` → 통합 `e2b8613de26cc65814f3fc5c0f81c1455671a0c3` (applied).
- 원 `6086d76e1e00ad078c769fc6e9b5d5faadf769b3` → 통합 `76e6070a3f08cad10476e9b577dcdb1baf58eff8` (applied).

[충돌·원 merge 해소 대조](../../working/task_m100_536_integration_review_stage1.md). 로컬 필수 검증을 완료했다. 원격 CI는 실제 통합 PR 생성 뒤 확인한다.

## 통합 후보 검증 연결

- 원 commit 적용과 merge 해소는 위 실제 SHA mapping 및 [1단계](../../working/task_m100_536_integration_review_stage1.md)를 따른다. 누적 source diff를 같은 PR의 고유 변경으로 반복 적용하지 않았다.
- 런타임/검사 분리는 `456ac3585`, 독립 Print 근거의 공통 극한 이름은 `d9a0b1c79`, 정식 원본 회귀는 `1ed02ebff`, font-request 반례·Print 보존은 `068769e9d`에 연결한다. 이 보정들을 원 contributor가 작성한 것으로 표시하지 않는다.
- 실제 실행 결과·source/binary/font/PDF hash와 직접 판독은 [5단계](../../working/task_m100_536_integration_review_stage5.md)·[6단계](../../working/task_m100_536_integration_review_stage6.md)에 있다. 기본 전체 nextest는 10,643 PASS / 0 FAIL / 45 skipped로 끝났다. Native 집중 277 PASS, 공식 Skia lib 4,043 PASS와 focused 2+4 PASS, format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 완료했다. 원격 exact-head CI는 미실행이며 이후 실제 결과로 trailing 기록을 갱신한다.
- 최종 판정과 merge 후 원 PR comment 계획은 같은 번호의 개별 review에 유지한다. 통합 PR 번호의 별도 review를 만들거나 contributor branch를 rewrite하지 않는다.

## 최신 base 리베이스

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 직후 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`. [7단계](../../working/task_m100_536_integration_review_stage7.md)의 새 head 검증은 진행 중이다. 위 실제 적용 SHA는 리베이스 전 기록이며 아래는 현재 대응이다.

- 이전 통합 `924da30d8848a33a28c361f349f97bb2a3285c2a` → 현재 `f7b96a99ac5603919d9041fcd699370a33907fc3`.
- 이전 통합 `1c311b8ac03ef9ff3d91d66941af6a9123f7cada` → 현재 `404e7ffa41a2274321c89c396c1349aada1e5f9d`.
- 이전 통합 `bd47a160c6e2425f93b41cbd0f378e2e044ecaf9` → 현재 `b2da7711d348b79eb1cc7b3fb5de3d14d325a82e`.
- 이전 통합 `66f2c9da3bd1e0f605c8e42fa077596d9b641901` → 현재 `c6e005ef297dec705ea6b34b8e68f4442be0ab7f`.
- 이전 통합 `ae4ad65828a720eca2a51726bcbb0d6e73c53147` → 현재 `e91717449ce6bcff54970bfcb4fe9055c1adb498`.
- 이전 통합 `8684212b359e538eb6d16b8ba0ad5bca02a61df3` → 현재 `dd7af54d0437048a6c92715556e41aa98854056a`.
- 이전 통합 `53fad1ef327aed56e18ed4c009a31db67a0f6385` → 현재 `b64eb3e657fffa1257c92d2206d93c85bade92b0`.
- 이전 통합 `6d4a146e6e9b9a10d0975e90f022fc25e49bcfc7` → 현재 `93f89f32d1ecc589595d6390948e133d39ea4a17`.
- 이전 통합 `f8aa165d8245ff4e548869e4b362f3a34d77c851` → 현재 `b982b5b0dd5f7c95e2ea5cb239e99f9cebecf8b1`.
- 이전 통합 `c857fffa6c27d06ecd111d685bcd0b6e03c9dac1` → 현재 `4d0b51a52e5aa9c4c661ab661134cbfa4fdb7ec1`.
- 이전 통합 `c69e4df4322a5560d773151841f56f69f24744cc` → 현재 `94cda59a86100bfbc5ce4288b5cd6c14f25b249a`.
- 이전 통합 `8887676a193c5ee9aec8166fed8320e5213874e0` → 현재 `42398bccff8a64eaaa6bae1d662035132618ac1c`.
- 이전 통합 `6f0523df1ebef65d0e8ddfc5ecab687de6e93660` → 현재 `7248d7f1184408b200d96c0598cadc4a03f7d67d`.
- 이전 통합 `696ab603f4930ec1ac899ad6e0047e09a98420a4` → 현재 `20ad8520d50ac4f5f5138bf8242ae0187de6281b`.
- 이전 통합 `b55e1396580047166dfc5ae9fbd92cad15753dbc` → 현재 `98bd58de34fd4f21e757750cd20cfeafe94b1701`.
- 이전 통합 `cfce101094507e6c85ef60af51c8100a646b3600` → 현재 `cffded203478357c421227a6655ae44a48e77f50`.
- 이전 통합 `8394eaf645d861427094c55e9f8d6a6aecdd3342` → 현재 `fcdcbfb5657e51514e04e1bb0b100d4479d79950`.
- 이전 통합 `37dcf275ebf71ae920cd780ccd3bec459dac86c2` → 현재 `8c9a96aeb2c38908f3ec1716fe0f657383cfcb53`.
- 이전 통합 `f1850e72c78ab8eecfedb0061505f63dc9c901f9` → 현재 `de786549588dd2c355dee3fe4cdd7f809e43a5a0`.
- 이전 통합 `7daad4d49599f62a78016d133346d29fcaabcaed` → 현재 `06ab1030b5245136adbc5cdc87c7a713098592fd`.
- 이전 통합 `d20c87bbdaa64b2939399504704d0d21dd61ced1` → 현재 `cb737464670b3926ba7986fc4f67369c2593915e`.
- 이전 통합 `56115aeeb9a6a400906eb20aa4222e2e99528c48` → 현재 `c08a797c68e263cec5adbd959b5205da83547b69`.
- 이전 통합 `e2b8613de26cc65814f3fc5c0f81c1455671a0c3` → 현재 `47e9fe5681b3668d077909c181e67c35ce92df86`.
- 이전 통합 `76e6070a3f08cad10476e9b577dcdb1baf58eff8` → 현재 `15ff329a5062f1fcd400db744a68434d752f6077`.

## 최종 source의 원 commit 연결

- 원 `9df25c8f22985486fb0e045f693b5663e8f90609` → 현재 `b38a7a6793f2971edab9990729feefcc8095b796`.
- 원 `447cb32e896a380f9c61e328b89a35e67dfbdba3` → 현재 `6d8ec47f9a5d5d92b24a4c07fc3277da64c47566`.
- 원 `7cf3f14ddd80d991bf75d6fbe006d4451547c3b4` → 현재 `a5db4ebcdcc2a6fe866b2eb8f2b5ed993733b21b`.
- 원 `88cff3560f5052a446fb33b358b3c3f2e32f9f93` → 현재 `6c19c62b17a43d62184eb850cc66d430c070ef18`.
- 원 `084964c5dc54884834d56105ff4429a74c5ac13e` → 현재 `0d2be397e009d9fba9ac1a005a785f55f99856b7`.
- 원 `b827287a5a7fdb9bf1968e880148763e48d13772` → 현재 `f203274910b7ae5d68d21916959a9db3099e28c7`.
- 원 `65adfe356305eca0eac59b73e5a696c5a52758f5` → 현재 `9379273b5572cb82d17e36bbd07fb11a757a7f3b`.
- 원 `fe3cbd62ce2b49bc4ba10e27af132c338133f3b8` → 현재 `3b5d02c6241ae31dc580fc95bdd20dec87f337ca`.
- 원 `7c4ae0e5383a55168b9cb689f35d247222bba24e` → 현재 `cf5af67961b0f9734320f3a5b5ccc60d0a1c20b5`.
- 원 `dc6ce6fee23caebf03605496b49084ab47232795` → 현재 `301d3476bb05150ad88877fe525b97d9bcf6bf16`.
- 원 `4e2741f70c4e5fe232b6fe4b5985a1b05a50e76a` → 현재 `5a150f1474d9cea21335ae4ecdbb2fbd013b55bd`.
- 원 `266117ee941cdccb48d4c039ac372b9a75ed1c6b` → 현재 `e0eb172d2ba6e3973bf3e47dcfe11cea941095c1`.
- 원 `493819d8b67a9adac3f881635aa8fabf545e5dc0` → 현재 `e1d47ad55a0d105d9d4946f56d9871e38710924c`.
- 원 `ebb1bf918f9006ea7ef4b086018f6a4a988f1dc5` → 현재 `95895f1ace1bbfb7ffe892b8fdd7b49087214ae2`.
- 원 `e0604f0d206d84aceebc600ac555704feeda6edb` → 현재 `11f623e978f9c1f7b14eddd8e49f03c2947c220e`.
- 원 `f5892664440aa3e5a6d940882f3c1250cfc990af` → 현재 `02998ddede2754e72d98c70a677a107a31b32c79`.
- 원 `7f93f337c6a79719bdb14c261bc7eb2ae43a3719` → 현재 `ffee0950c13441ab825adcc4f64a15061ae285d8`.
- 원 `f10c215725926f1550cd92ef4eededb3a668e42e` → 현재 `668c9db305f82f253b014d382228e7594e72c5cf`.
- 원 `4a53eacd8a77fc18b92470bf32b6809aa22ca6dd` → 현재 `7e0d52635b5fd61b98a35648554b99ddffbded42`.
- 원 `e743bbda044290bac21421c3252479560009b4c1` → 현재 `d74b32006d989a40bb02b28190f811b7f5167aae`.
- 원 `994e8a7835f22b6c0433d826c490dfb827532e42` → 현재 `1334da10a5a110c4dba9e1add44f98e723ea219c`.
- 원 `c533d85d03499faab411fe41eb64cd7d6a3b34f1` → 현재 `319c0e1e6f4b450ba90236db9457e39d8867e7ac`.
- 원 `1eade8b0dc6a82d8d4d1874868389bf5ec89276d` → 현재 `242c70784792a983df9620ecbdf75359197d84a9`.
- 원 `6086d76e1e00ad078c769fc6e9b5d5faadf769b3` → 현재 `6252a459717678fae9ffe4d5c001c7d1379f9c60`.

## 최신 devel 통합 검증

현재 producer `3535106051394672e93ca8401ff9eae8b449ec20` /base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`의 전체 회귀10779PASS와 실제 소비 경로 검증은 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다.원 contributor author와164개 리베이스 commit의 순서·제목을 보존했다.원격 CI·merge 결과는 실제 확인 뒤 같은 PR trailing 기록에 추가한다.

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 review·오늘할일 trailing head CI와 normal merge는 다음 단계다.
