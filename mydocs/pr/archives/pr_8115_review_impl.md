# PR #8115 적용·보정 실행 기록

- 원 head: `a47a41d3004a0458873cf1a00e90a356f19f2fa5`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8115_review.md](pr_8115_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `d23261a2ce4763092c05e648e8610d9d5c6d4732` | `759db4f377ff985b58992200477aa9c9eb279321` |
| 2 | `afa916e30402fd395d4b2742f2b4a5883ea58a9f` | `4f578fb4f64795e45a92c04dc54414d3d7b49703` |
| 3 | `2d46ea9882fb08aa3f451d0cd72bfc9e0e62fcb0` | `f66f899fabc656830044eb0b240770bc6ac42890` |
| 4 | `bc88ca507fe4256d36f5a6471cc382d539b5c14c` | `c3ddaf20bc7309550955f548420da9f1795128d4` |
| 5 | `93463d0545c294a42e9b891d473302dd978fbbd0` | `e596ba297eadefaaeaf4b1deaadf7d3dcbaa81ec` |
| 6 | `ddecddd9da13ea599c22fc0919614fd203190e29` | `cde916075a4731fb9b4a7f5482e651d9571683ac` |
| 7 | `a7aaa28434b875c8d695337ea4c9b053286bd05a` | `e312f514e68724d7c2e7ed9a43447d875c1d46ac` |
| 8 | `237ed1f5736f714a5878c93a8640bea343b0b163` | `8bc81e65f8f96bf3ded6737249e162d779c378b6` |
| 9 | `a47a41d3004a0458873cf1a00e90a356f19f2fa5` | `eefefaa535cee1434d044ab6c9765b39d51e1299` |

적층 선행 commit은 patch-id로 대조했다. merge commit을 다시 체리픽하지 않았고 원 저자·`-x` 출처를 보존했다.
#8227·#8237의 사전 가상 충돌은 적층 부모를 명시한 예상 tree와 실제 결과가 일치했다. 실제 체리픽 충돌은 없었다. 이후 #8116 리베이스의 line_breaking.rs 한 충돌은 두 수용 조건을 모두 유지해 해소했다. 리베이스 전후 SHA·원 저자 보존은 커밋 대조 원장에 기록했다.

## 보정·검증·후속 경계

1. 고정 원 head 적용과 예상 tree·중복을 원장으로 확인했다.
2. 승인된 기존 미달 페이지를 공통 로직에서 보정하고 단계별 일반 커밋으로 고정했다.
3. 관련 전쪽 Native/fresh WASM 선행 검증·직접 판독 뒤 정식 소유 회귀를 보강하고, 당시 후보의 최적화 WASM 전쪽과 기준 대비 하락을 검증했다. 이후 후보 보정에 대한 비교는 별도로 진행한다.
4. 이전 전체 회귀와 이후 선택 회귀는 실행 당시 후보를 구분해 기록한다. 작업지시자 지시에 따라 전체 회귀는 재실행하지 않으며, 현재 남은 검증과 미해결 기준은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)에서 추적한다.
5. 최종 로컬 커밋·본문을 준비하고 작업지시자 승인 후 원격 push·통합 PR 제출을 진행한다. 이후 CI·머지·원 PR 종료는 별도 후속 조건이다.

보정 SHA: 이 PR 단독 보정 없음; 공통 통합 보정은 상위 계획 참조.

원 contributor head를 rewrite하거나 원격 브랜치를 변경하지 않았다. 되돌림은 필요 시 이 원장의 적용 commit과
관련 공통 보정을 검토해 별도 revert로 처리하며 다른 PR 변경까지 한꺼번에 롤백하지 않는다.
