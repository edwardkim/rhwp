# PR #8206 적용·보정 실행 기록

- 원 head: `dd3ad0b29bb55893caf8d618e42a24776d64673f`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8206_review.md](pr_8206_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `69fca8f5003e3ec5a6fd89219e9474ade2638074` | `468d15fd9cda237a690eb7a63ae0c1231b185111` |
| 2 | `251c0b53d2aa22da7764bf32f77e3e5e6165d27f` | `e0f7f2676c30baeed27daa0c0f3b4b24acedea5b` |
| 3 | `6e59ef78f459e1039d2c5fd586df32893965e73d` | `13532fab1ced5b0a30bedaff5f7e141c36c29218` |
| 4 | `dd3ad0b29bb55893caf8d618e42a24776d64673f` | `39c80fc912244ad0a3e89284a44520d90eca5b23` |

적층 선행 commit은 patch-id로 대조했다. merge commit을 다시 체리픽하지 않았고 원 저자·`-x` 출처를 보존했다.
#8227·#8237의 사전 가상 충돌은 적층 부모를 명시한 예상 tree와 실제 결과가 일치했다. 실제 체리픽 충돌은 없었다. 이후 #8116 리베이스의 line_breaking.rs 한 충돌은 두 수용 조건을 모두 유지해 해소했다. 리베이스 전후 SHA·원 저자 보존은 커밋 대조 원장에 기록했다.

## 보정·검증·후속 경계

1. 고정 원 head 적용과 예상 tree·중복을 원장으로 확인했다.
2. 승인된 기존 미달 페이지를 공통 로직에서 보정하고 단계별 일반 커밋으로 고정했다.
3. 관련 전쪽 Native/fresh WASM 선행 검증·직접 판독 뒤 정식 소유 회귀를 보강하고, 당시 후보의 최적화 WASM 전쪽과 기준 대비 하락을 검증했다. 이후 후보 보정에 대한 비교는 별도로 진행한다.
4. 이전 전체 회귀와 이후 선택 회귀는 실행 당시 후보를 구분해 기록한다. 작업지시자 지시에 따라 전체 회귀는 재실행하지 않으며, 현재 남은 검증과 미해결 기준은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)에서 추적한다.
5. 최종 로컬 커밋·본문을 준비하고 작업지시자 승인 후 원격 push·통합 PR 제출을 진행한다. 이후 CI·머지·원 PR 종료는 별도 후속 조건이다.

보정 SHA: `ffba46071517860ca31a43c45cb320a116370cc6`, `1237577af2026127686c2d19f4d1187d5c80e795`, `57d362f43cc14dc92454e95ceb2615f2e23995f6`.

원 contributor head를 rewrite하거나 원격 브랜치를 변경하지 않았다. 되돌림은 필요 시 이 원장의 적용 commit과
관련 공통 보정을 검토해 별도 revert로 처리하며 다른 PR 변경까지 한꺼번에 롤백하지 않는다.
