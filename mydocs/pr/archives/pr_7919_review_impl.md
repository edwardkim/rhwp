# PR #7919 통합 검토 구현 계획

- 원 head `25c56fde48d4afc500782f296cb120d5b42b139f`, 기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`.
- [공통 실행 계획](../../plans/pr_review_seo_rii_20261017.md)의 순차 gate를 따른다.
- 고유 적용 대상 4개. 아래 SHA는 원 author를 보존해 오래된 순서로 적용한다.
- merge commit과 다른 PR에 이미 속한 commit은 중복 적용하지 않는다.
- 충돌·보정·검증 결과는 [개별 review](pr_7919_review.md)에 연결하며 검증 전 완료를 단정하지 않는다.
- rollback은 통합 PR 제출 전 이 작업 branch에서만 판단한다. 사용자 source/fork/history를 재작성하지 않는다.

## 원 commit 순서

- `45815cebe617eece7d1cd05f85c6fe2241b60a92`
- `ebe37071fc8cc6e86a6ba4ed9bb8e245d0ee8c3d`
- `538208de33a093eb26a65a55bbe8efef60a8d259`
- `8a560f40aa9a005091b6729de84dacc34e89f501`

## 실제 적용 결과

- 원 `45815cebe617eece7d1cd05f85c6fe2241b60a92` → 통합 `03bac9ffe097f86e6751d45a2e484f0ab4f87fe0` (applied).
- 원 `ebe37071fc8cc6e86a6ba4ed9bb8e245d0ee8c3d` → 통합 `b1bc6b7cd300bcbd75e3d3aaad0b5b8a4224a95e` (applied).
- 원 `538208de33a093eb26a65a55bbe8efef60a8d259` → 통합 `a47578af312a5ffa8724630b7885a12e0bd61d91` (applied).
- 원 `8a560f40aa9a005091b6729de84dacc34e89f501` → 통합 `36e8c8c8613c49bba41e08e10c6243640e1c4871` (applied).

[충돌·원 merge 해소 대조](../../working/task_m100_536_integration_review_stage1.md). 로컬 필수 검증을 완료했다. 원격 CI는 실제 통합 PR 생성 뒤 확인한다.

## 통합 후보 검증 연결

- 원 commit 적용과 merge 해소는 위 실제 SHA mapping 및 [1단계](../../working/task_m100_536_integration_review_stage1.md)를 따른다. 누적 source diff를 같은 PR의 고유 변경으로 반복 적용하지 않았다.
- 런타임/검사 분리는 `456ac3585`, 독립 Print 근거의 공통 극한 이름은 `d9a0b1c79`, 정식 원본 회귀는 `1ed02ebff`, font-request 반례·Print 보존은 `068769e9d`에 연결한다. 이 보정들을 원 contributor가 작성한 것으로 표시하지 않는다.
- 실제 실행 결과·source/binary/font/PDF hash와 직접 판독은 [5단계](../../working/task_m100_536_integration_review_stage5.md)·[6단계](../../working/task_m100_536_integration_review_stage6.md)에 있다. 기본 전체 nextest는 10,643 PASS / 0 FAIL / 45 skipped로 끝났다. Native 집중 277 PASS, 공식 Skia lib 4,043 PASS와 focused 2+4 PASS, format·Native/WASM/workspace lint·workspace build·doc test·실제 feature-off ABI·기존 HFT Native 검사도 완료했다. 원격 exact-head CI는 미실행이며 이후 실제 결과로 trailing 기록을 갱신한다.
- 최종 판정과 merge 후 원 PR comment 계획은 같은 번호의 개별 review에 유지한다. 통합 PR 번호의 별도 review를 만들거나 contributor branch를 rewrite하지 않는다.

## 최신 base 리베이스

새 base `b7a5388034ee66f58a7e1d5363974e83b6374f57`, 직후 head `a7aac1a9ce5b2b902471ecd8f234f7d72834812d`. [7단계](../../working/task_m100_536_integration_review_stage7.md)의 새 head 검증은 진행 중이다. 위 실제 적용 SHA는 리베이스 전 기록이며 아래는 현재 대응이다.

- 이전 통합 `03bac9ffe097f86e6751d45a2e484f0ab4f87fe0` → 현재 `43daad32fb9b0dd17927a267dbf177639bf64833`.
- 이전 통합 `b1bc6b7cd300bcbd75e3d3aaad0b5b8a4224a95e` → 현재 `b379378e005e3cd192fcf480621d6268c70a2b5c`.
- 이전 통합 `a47578af312a5ffa8724630b7885a12e0bd61d91` → 현재 `fde831b30c8d52eacadefe8c362d216723cd9f06`.
- 이전 통합 `36e8c8c8613c49bba41e08e10c6243640e1c4871` → 현재 `04a9833c522617578692ead513d01e279f951c3d`.

## 최종 source의 원 commit 연결

- 원 `45815cebe617eece7d1cd05f85c6fe2241b60a92` → 현재 `af33aa8504d3e472032aad146dfbfa0cc27f31eb`.
- 원 `ebe37071fc8cc6e86a6ba4ed9bb8e245d0ee8c3d` → 현재 `29c6f1ad96a4b9dfc50d960466c40e777e23da40`.
- 원 `538208de33a093eb26a65a55bbe8efef60a8d259` → 현재 `b8a15c61a4b9bcc208142d84b4098d64cf39eb32`.
- 원 `8a560f40aa9a005091b6729de84dacc34e89f501` → 현재 `387141ffa421d9b0e9c6a80939a4a87a7f55106a`.

## 최신 devel 통합 검증

현재 producer `3535106051394672e93ca8401ff9eae8b449ec20` /base `9415b920c8dec299e8ada69b01dbd1c3c044f1f1`의 전체 회귀10779PASS와 실제 소비 경로 검증은 [8단계](../../working/task_m100_536_integration_review_stage8.md)를 따른다.원 contributor author와164개 리베이스 commit의 순서·제목을 보존했다.원격 CI·merge 결과는 실제 확인 뒤 같은 PR trailing 기록에 추가한다.

## 통합 PR #7969 code head CI 완료

검증한 통합 code head `c5b99e2c2825f1c56e9750134fc5722f916a8a8b`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/39007700375) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39007700497) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39007700494) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39007700528) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39007700524) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39008935219)가 실제 성공했다. CodeQL의 JavaScript/TypeScript·Python·Rust·Actions Analyze와 GHAS CodeQL check도 성공했다. Full CI의 Native·lint·frontend, archive A/B/C/D builder·worker가 실제 성공했으며, 별도 WASM Build job의 skip과 frontend fresh WASM 성공을 구분한다.

같은 PR의 review·오늘할일 trailing head CI와 normal merge는 다음 단계다.
