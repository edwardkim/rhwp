# PR #8133 적용·보정 실행 기록

- 원 head: `465d85f13cab961628ec07247ea6ade82e0cbc46`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8133_review.md](pr_8133_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `5fdd263ee9f05cc977d5bee5cabe03849f6d995c` | `1ff1c0c6e802ee4a7a3982dee095eb5260f04722` |
| 2 | `d39696d63cd6c7ff03c65b0ecb429827b91ce252` | `ca3187260c975c30ba791fdbc49e5ac9a84fbf43` |
| 3 | `68e11eb4b78a837e9c935a5ce04d6e9539110618` | `700c31a35191bda702cfd8c2b73d4484d4423954` |
| 4 | `c0b1451409d1e5e529fa5009c3849105440b75a1` | `79423873e25fea03b08244f7206c1850b859a753` |
| 5 | `b733b47ec089f72a2a1786635f25dcb9032f6c67` | `0c967a25b520c90fcad764a4628f453487e108f6` |
| 6 | `d40fa79b0124c2fc254b466819ad5aaeb5664bac` | `92254fc828acba08f7a211acd533e455a53b6c65` |
| 7 | `57d2b6d4f56cf9c0d9baf0978f3a65aafc63b7dd` | `16437f345a0d55cbbd4c441b027b452f2f1f1ee8` |
| 8 | `c273dcecb5e5c475caeed769a77d75829176a61e` | `b33071aacdb4534c884f210e987b4d5c1dadb3fe` |
| 9 | `3c31f51f9c656d0070f29fd17b0f669ff763554c` | `a21da2b8d9a4cbfd6fdab63dc0f7e266ad929592` |
| 10 | `561f849625c17f63eda6ca84b60ac59fa5dd7025` | `6279d1fc4ddb41a5e81af745062335eb9ea31635` |
| 11 | `9af0aeceb2331fe5d29bedcfd9535aa6f91d8071` | `0ba0b1e1f5cf47212fdac332ad1da1c87befaa62` |
| 12 | `465d85f13cab961628ec07247ea6ade82e0cbc46` | `a7a7ce53ef0f88202b1d8ffcfb43e4fe6f7ff0bb` |

적층 선행 commit은 patch-id로 대조했다. merge commit을 다시 체리픽하지 않았고 원 저자·`-x` 출처를 보존했다.
#8227·#8237의 사전 가상 충돌은 적층 부모를 명시한 예상 tree와 실제 결과가 일치했다. 실제 체리픽 충돌은 없었다. 이후 #8116 리베이스의 line_breaking.rs 한 충돌은 두 수용 조건을 모두 유지해 해소했다. 리베이스 전후 SHA·원 저자 보존은 커밋 대조 원장에 기록했다.

## 보정·검증·후속 경계

1. 고정 원 head 적용과 예상 tree·중복을 원장으로 확인했다.
2. 승인된 기존 미달 페이지를 공통 로직에서 보정하고 단계별 일반 커밋으로 고정했다.
3. 관련 전쪽 Native/fresh WASM 선행 검증·직접 판독 뒤 정식 소유 회귀를 보강하고, 당시 후보의 최적화 WASM 전쪽과 기준 대비 하락을 검증했다. 이후 후보 보정에 대한 비교는 별도로 진행한다.
4. 이전 전체 회귀와 이후 선택 회귀는 실행 당시 후보를 구분해 기록한다. 작업지시자 지시에 따라 전체 회귀는 재실행하지 않으며, 현재 남은 검증과 미해결 기준은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)에서 추적한다.
5. 최종 로컬 커밋·본문을 준비하고 작업지시자 승인 후 원격 push·통합 PR 제출을 진행한다. 이후 CI·머지·원 PR 종료는 별도 후속 조건이다.

보정 SHA: `7e617f4e9d0a9964285a525cd848dafaa09cd8ad`, `033dbdf205c7adee7011070b64c4b2891360dd23`, `a75bd1c867aa10e24338cd4cb4ac6674f7526dac`.

원 contributor head를 rewrite하거나 원격 브랜치를 변경하지 않았다. 되돌림은 필요 시 이 원장의 적용 commit과
관련 공통 보정을 검토해 별도 revert로 처리하며 다른 PR 변경까지 한꺼번에 롤백하지 않는다.

## 2026-10-28 최신 원 PR 대조

가는 획 Skia 보정과 기존 회귀 검사는 유지된다. 통합 후보는 선언 굵기로 raster 계획을 세우는 후속 보정과 실측 쪽 테두리 검사를 추가했다. CI의 차이는 Visual Sweep Brotli 의존성 추가이며 classifier와 해당 검사는 최신 원 PR과 같다. [커밋별 대조 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md).
