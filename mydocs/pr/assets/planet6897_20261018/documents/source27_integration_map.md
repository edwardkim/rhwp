# 원 PR 27개의 현재 통합 출처

- 기준 devel: `a6e9d18fc1061318055b4eb348daec201427ea03`; 검증 head: `3cb5a4c290663343123074be1e2fc0dda18d3db4`.
- 원 PR 본문·댓글·review·reviewer 재조회: `2026-10-21T22:22:01.648050+00:00`. 이전 조회와 변경이 없으며 27개 모두 기능 tip의 현재 통합 commit을 확인했습니다.
- 이력은 최초 작업 계획 `ddbe5da9720d4720cb24613cc491456b09f9f2cd` 한 개 → 원 기여 124개 연속 → 메인터너 보정 순서입니다. 최초 계획 commit은 계획 Markdown 한 파일만 수정합니다. 원 기여 마지막은 `0a8618dc8e6eb4f6b4d3a34655442a9c1816d049`, 첫 메인터너 보정은 `cee69143e564c913c5c039d9ea8c3803baee08fd`입니다.
- 각 원 PR의 실제 head와 기능 tip을 구분합니다. 일부 원 head에는 devel merge가 포함되어 있습니다. #7975는 그 merge에서 새로 작성한 parser·fixture 기능 차이를 일반 commit으로 별도 추출했고, 아래에 함께 기록합니다. 원 기여자의 source branch는 변경하지 않았습니다.
- 실제 source와 공통 보정의 기능 범위·기존 이관·최종 판정은 code `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7`의 CI 성공 뒤 아래 archive에 기록했습니다. 실제 runtime은 `3cb5a4c290663343123074be1e2fc0dda18d3db4`, 새 회귀 등록은 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`입니다.

| 원 PR | 조회 상태 | 원 head | 원 기능 tip | 현재 기능 통합 commit | source merge의 별도 기능 보정 |
| --- | --- | --- | --- | --- | --- |
| [#7934](https://github.com/edwardkim/rhwp/pull/7934) | OPEN | `a60c275e8abbce2c6cb9e4277422b7084b13e830` | `a60c275e8abbce2c6cb9e4277422b7084b13e830` | [`50edc55d03`](https://github.com/edwardkim/rhwp/commit/50edc55d037146c93247c940f5faa50656e0ddb1) | — |
| [#7956](https://github.com/edwardkim/rhwp/pull/7956) | OPEN | `6bcefd1ac368ad81dca81239a1e8ee6e1f2ae092` | `6bcefd1ac368ad81dca81239a1e8ee6e1f2ae092` | [`3cbaabdf06`](https://github.com/edwardkim/rhwp/commit/3cbaabdf06284b220d0b080e11c2a706f62790e6) | — |
| [#7965](https://github.com/edwardkim/rhwp/pull/7965) | OPEN | `690290fa9d06c6f19f18a753900aa3555e1de1c0` | `690290fa9d06c6f19f18a753900aa3555e1de1c0` | [`361c049311`](https://github.com/edwardkim/rhwp/commit/361c049311f828c954f3887b9832ef24b255396d) | — |
| [#7967](https://github.com/edwardkim/rhwp/pull/7967) | OPEN | `2004d296003b0a75c823a00c4ac09486ed672385` | `2004d296003b0a75c823a00c4ac09486ed672385` | [`2ef7840122`](https://github.com/edwardkim/rhwp/commit/2ef784012279c8764bb42653210debfc2fbfede4) | — |
| [#7968](https://github.com/edwardkim/rhwp/pull/7968) | CLOSED | `4fde7ae18c1c5ded4ed7c645e72db682ce58b088` | `4fde7ae18c1c5ded4ed7c645e72db682ce58b088` | [`68be0352ad`](https://github.com/edwardkim/rhwp/commit/68be0352ad95cee4f859704373e918ba794b0343) | — |
| [#7973](https://github.com/edwardkim/rhwp/pull/7973) | OPEN | `ff1d1f5bbeba9db05d069a80e409257fcb97a7c5` | `6df857e2c1b164a6016f6ea50f0299da6bbe5587` | [`495e9c726d`](https://github.com/edwardkim/rhwp/commit/495e9c726d7f19338c927fa9e5138fff799eea67) | — |
| [#7975](https://github.com/edwardkim/rhwp/pull/7975) | OPEN | `31bfbcc003ff051833793404395d859edbad9c9c` | `dbdbdd8b3a7b7510e047be23979b8366b84536fc` | [`9dcaba4d76`](https://github.com/edwardkim/rhwp/commit/9dcaba4d76bc9e5ad8bf58992f79a5f28ed7a883) | `dd599ee8b5c59eb155e42d726d57da546619f2f1` |
| [#7976](https://github.com/edwardkim/rhwp/pull/7976) | CLOSED | `7f85ab9b0198edbd68d848e43b370831407c33bd` | `7f85ab9b0198edbd68d848e43b370831407c33bd` | [`ef7fcf6a47`](https://github.com/edwardkim/rhwp/commit/ef7fcf6a4735bc541504b196caef5aaf119c93f0) | — |
| [#7979](https://github.com/edwardkim/rhwp/pull/7979) | OPEN | `7121c20333af61183ea94ea4be30ba0641b6977b` | `7377b745bce39aa28dca444b1df1b8c90a4598a2` | [`c756b84bd1`](https://github.com/edwardkim/rhwp/commit/c756b84bd18fbe0f35e3f03764f30bc563492537) | — |
| [#7982](https://github.com/edwardkim/rhwp/pull/7982) | OPEN | `7ffd13b64aa90110070d5c47eb6043b1d83af4df` | `7ffd13b64aa90110070d5c47eb6043b1d83af4df` | [`f5db70d67d`](https://github.com/edwardkim/rhwp/commit/f5db70d67d5096f5b4504124ecd1a0abd48a1c49) | — |
| [#7987](https://github.com/edwardkim/rhwp/pull/7987) | OPEN | `fe5d9ebd5260eca29c23e913d8b81cc1c209ccac` | `fe5d9ebd5260eca29c23e913d8b81cc1c209ccac` | [`49b7fa1c1c`](https://github.com/edwardkim/rhwp/commit/49b7fa1c1cab771ea4b9c33a17ccd381a6d8b379) | — |
| [#8005](https://github.com/edwardkim/rhwp/pull/8005) | OPEN | `839af8c92427d834d7042ec540c5e0fbab01473c` | `839af8c92427d834d7042ec540c5e0fbab01473c` | [`7bf6a66139`](https://github.com/edwardkim/rhwp/commit/7bf6a6613992e42cd8bfb41a76edd7894e46d585) | — |
| [#8006](https://github.com/edwardkim/rhwp/pull/8006) | OPEN | `6627c3c07de960c3a5fad75d3a7d02d7ff2562e2` | `279a19415b203b7c074fca32403ea10a3fc0634b` | [`01e2c9905a`](https://github.com/edwardkim/rhwp/commit/01e2c9905a8a9ae5767d20ead9e264229c327e38) | — |
| [#8007](https://github.com/edwardkim/rhwp/pull/8007) | OPEN | `43ffb10633c0c4504a8585a5c2b41dcc79df8a2d` | `43ffb10633c0c4504a8585a5c2b41dcc79df8a2d` | [`09a25fcf57`](https://github.com/edwardkim/rhwp/commit/09a25fcf5733a1fda0cb33191d614d2681488bf7) | — |
| [#8009](https://github.com/edwardkim/rhwp/pull/8009) | OPEN | `ed01693a3890c63a08bdf7a06dca33312206f268` | `44a78241f02a224c778d9f3d2a1f4ee928e97f22` | [`160ee37d77`](https://github.com/edwardkim/rhwp/commit/160ee37d772c1e861309f1f1ee57fca0f3654dfa) | — |
| [#8010](https://github.com/edwardkim/rhwp/pull/8010) | CLOSED | `a9672907e47edc8fafe191711aa4251e66aefe0b` | `a9672907e47edc8fafe191711aa4251e66aefe0b` | [`a2733e980d`](https://github.com/edwardkim/rhwp/commit/a2733e980df445bb4ddc0d6ab9847576549cfabc) | — |
| [#8011](https://github.com/edwardkim/rhwp/pull/8011) | OPEN | `99b9a49a694ba565ff8eb3f45bb78e93e0a8a538` | `99b9a49a694ba565ff8eb3f45bb78e93e0a8a538` | [`ce97b41642`](https://github.com/edwardkim/rhwp/commit/ce97b416423e568e3577b1783488f8f1dc99ab48) | — |
| [#8012](https://github.com/edwardkim/rhwp/pull/8012) | OPEN | `56296c533a792565a83466eacb60e54477180b0b` | `56296c533a792565a83466eacb60e54477180b0b` | [`7632c340cc`](https://github.com/edwardkim/rhwp/commit/7632c340cc0f527acb3fafcda19fe312715d47c7) | — |
| [#8013](https://github.com/edwardkim/rhwp/pull/8013) | OPEN | `f6b42559cec98067f47c0d84abca74d09201e86c` | `f6b42559cec98067f47c0d84abca74d09201e86c` | [`39095f7723`](https://github.com/edwardkim/rhwp/commit/39095f7723841c9f514d14e76f9aa65917951c25) | — |
| [#8015](https://github.com/edwardkim/rhwp/pull/8015) | OPEN | `6aeebaffd730f8958d94bb09bdc461b2861ee647` | `6aeebaffd730f8958d94bb09bdc461b2861ee647` | [`2d08aa389d`](https://github.com/edwardkim/rhwp/commit/2d08aa389d07144c3d5c06aadde009972229c8ca) | — |
| [#8026](https://github.com/edwardkim/rhwp/pull/8026) | OPEN | `3ffe698eb37a245e10be1463fa1355ab0f88a7e0` | `6652f7d0c0686f4a525f0f9e60f1cccd92a1af7f` | [`703a26cea4`](https://github.com/edwardkim/rhwp/commit/703a26cea4f047a1ce4e62d9cd62eebf555e6adb) | — |
| [#8028](https://github.com/edwardkim/rhwp/pull/8028) | CLOSED | `2d0fc147b5bdc42a2b316c1975ddc6fa8535d110` | `2d0fc147b5bdc42a2b316c1975ddc6fa8535d110` | [`0516cf1045`](https://github.com/edwardkim/rhwp/commit/0516cf1045ea6c129f1fda8f7392fc2cba56a33b) | — |
| [#8034](https://github.com/edwardkim/rhwp/pull/8034) | OPEN | `fe9d0e8c3cadaf362d5cf6b38ecef6ac4c0746c1` | `fe9d0e8c3cadaf362d5cf6b38ecef6ac4c0746c1` | [`6aa492f9f2`](https://github.com/edwardkim/rhwp/commit/6aa492f9f27dd4e16233148635fe2dec929d3607) | — |
| [#8059](https://github.com/edwardkim/rhwp/pull/8059) | OPEN | `1367ff84693d4745f40efe5889b56835c2793688` | `1367ff84693d4745f40efe5889b56835c2793688` | [`0a8618dc8e`](https://github.com/edwardkim/rhwp/commit/0a8618dc8e6eb4f6b4d3a34655442a9c1816d049) | — |
| [#8063](https://github.com/edwardkim/rhwp/pull/8063) | OPEN | `6efafb91aa51c720803fb22ca8bc612d7e7d7f8b` | `9e3388e8d775db4139e8af715a511af647963e2d` | [`e26cfeebc7`](https://github.com/edwardkim/rhwp/commit/e26cfeebc7291b6ad1a083af299cb286404fd1b7) | — |
| [#8068](https://github.com/edwardkim/rhwp/pull/8068) | OPEN | `b7505e0ac6850acd1108368d9843c806b65ccf27` | `b7505e0ac6850acd1108368d9843c806b65ccf27` | [`7ebfa0e1e0`](https://github.com/edwardkim/rhwp/commit/7ebfa0e1e03efd815a5d37b45d603a5da72cc1b2) | — |
| [#8082](https://github.com/edwardkim/rhwp/pull/8082) | OPEN | `a44f2432b1d207fd05227eb3258970eecc7292c5` | `a44f2432b1d207fd05227eb3258970eecc7292c5` | [`cf426f02c9`](https://github.com/edwardkim/rhwp/commit/cf426f02c9ead028f3d2d32a2d71ec5ce93ca31e) | — |

## 원 PR별 최종 archive 기록

| 원 PR | 검토 | 구현·통합 |
| --- | --- | --- |
| #7934 | [최종 검토](../../../archives/pr_7934_review.md) | [구현·통합](../../../archives/pr_7934_review_impl.md) |
| #7956 | [최종 검토](../../../archives/pr_7956_review.md) | [구현·통합](../../../archives/pr_7956_review_impl.md) |
| #7965 | [최종 검토](../../../archives/pr_7965_review.md) | [구현·통합](../../../archives/pr_7965_review_impl.md) |
| #7967 | [최종 검토](../../../archives/pr_7967_review.md) | [구현·통합](../../../archives/pr_7967_review_impl.md) |
| #7968 | [최종 검토](../../../archives/pr_7968_review.md) | [구현·통합](../../../archives/pr_7968_review_impl.md) |
| #7973 | [최종 검토](../../../archives/pr_7973_review.md) | [구현·통합](../../../archives/pr_7973_review_impl.md) |
| #7975 | [최종 검토](../../../archives/pr_7975_review.md) | [구현·통합](../../../archives/pr_7975_review_impl.md) |
| #7976 | [최종 검토](../../../archives/pr_7976_review.md) | [구현·통합](../../../archives/pr_7976_review_impl.md) |
| #7979 | [최종 검토](../../../archives/pr_7979_review.md) | [구현·통합](../../../archives/pr_7979_review_impl.md) |
| #7982 | [최종 검토](../../../archives/pr_7982_review.md) | [구현·통합](../../../archives/pr_7982_review_impl.md) |
| #7987 | [최종 검토](../../../archives/pr_7987_review.md) | [구현·통합](../../../archives/pr_7987_review_impl.md) |
| #8005 | [최종 검토](../../../archives/pr_8005_review.md) | [구현·통합](../../../archives/pr_8005_review_impl.md) |
| #8006 | [최종 검토](../../../archives/pr_8006_review.md) | [구현·통합](../../../archives/pr_8006_review_impl.md) |
| #8007 | [최종 검토](../../../archives/pr_8007_review.md) | [구현·통합](../../../archives/pr_8007_review_impl.md) |
| #8009 | [최종 검토](../../../archives/pr_8009_review.md) | [구현·통합](../../../archives/pr_8009_review_impl.md) |
| #8010 | [최종 검토](../../../archives/pr_8010_review.md) | [구현·통합](../../../archives/pr_8010_review_impl.md) |
| #8011 | [최종 검토](../../../archives/pr_8011_review.md) | [구현·통합](../../../archives/pr_8011_review_impl.md) |
| #8012 | [최종 검토](../../../archives/pr_8012_review.md) | [구현·통합](../../../archives/pr_8012_review_impl.md) |
| #8013 | [최종 검토](../../../archives/pr_8013_review.md) | [구현·통합](../../../archives/pr_8013_review_impl.md) |
| #8015 | [최종 검토](../../../archives/pr_8015_review.md) | [구현·통합](../../../archives/pr_8015_review_impl.md) |
| #8026 | [최종 검토](../../../archives/pr_8026_review.md) | [구현·통합](../../../archives/pr_8026_review_impl.md) |
| #8028 | [최종 검토](../../../archives/pr_8028_review.md) | [구현·통합](../../../archives/pr_8028_review_impl.md) |
| #8034 | [최종 검토](../../../archives/pr_8034_review.md) | [구현·통합](../../../archives/pr_8034_review_impl.md) |
| #8059 | [최종 검토](../../../archives/pr_8059_review.md) | [구현·통합](../../../archives/pr_8059_review_impl.md) |
| #8063 | [최종 검토](../../../archives/pr_8063_review.md) | [구현·통합](../../../archives/pr_8063_review_impl.md) |
| #8068 | [최종 검토](../../../archives/pr_8068_review.md) | [구현·통합](../../../archives/pr_8068_review_impl.md) |
| #8082 | [최종 검토](../../../archives/pr_8082_review.md) | [구현·통합](../../../archives/pr_8082_review_impl.md) |

[code head까지의 실제 커밋 순서](integration_commit_order.md)
