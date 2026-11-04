# PR #8227 적용·보정 실행 기록

- 원 head: `a34d1fb70dcdc2e1f692efaa63bfd2d77ee0e74c`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8227_review.md](pr_8227_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `b8a0e876d5fad426108ea7173f9c6ab8d7f47e17` | 동일 patch 선행 반영으로 중복 제외 |
| 2 | `2b3982b028b5b80398d6d0b75bf96716762995a3` | 동일 patch 선행 반영으로 중복 제외 |
| 3 | `1a69db15352e171d2a16b4195d78cc445af2e9e6` | 동일 patch 선행 반영으로 중복 제외 |
| 4 | `462c3dd380da3e3f617c39082f341efcb8fc6afe` | `27387a82999c5569f5106fbad9925485915607ac` |
| 5 | `a19b8bc9359d8b80d86c7b497d9d1e7f40c72774` | `b82f02dbbd7925c4c01f52041d7295a57c4ea2f8` |
| 6 | `6d1ad105f8b56d4639855200e1b6ccf9b7ddc618` | `b93e68f34c9664c2fb5ced0c3443f0ef4a010902` |
| 7 | `a34d1fb70dcdc2e1f692efaa63bfd2d77ee0e74c` | `24c1e2c630d30f586426c2518bf0526a5a5eb863` |

적층 선행 commit은 patch-id로 대조했다. merge commit을 다시 체리픽하지 않았고 원 저자·`-x` 출처를 보존했다.
#8227·#8237의 사전 가상 충돌은 적층 부모를 명시한 예상 tree와 실제 결과가 일치했다. 실제 체리픽 충돌은 없었다. 이후 #8116 리베이스의 line_breaking.rs 한 충돌은 두 수용 조건을 모두 유지해 해소했다. 리베이스 전후 SHA·원 저자 보존은 커밋 대조 원장에 기록했다.

2026-10-28 원 PR head `fe855d85935c47b3ff5fd1b93839e5926f468828`을 다시 가져와 7개 커밋의 stable patch-id를 위 체리픽 당시 원 커밋과 각각 비교했다. 7개 모두 같으며, `src/parser/pdf/editable.rs`도 현재 통합 브랜치와 바이트 단위로 같다. `body_regions.rs`의 차이는 통합한 #8237의 머리·꼬리 띠 보정이다. 원 PR head 변경만으로 추가 코드가 누락됐다고 판정하지 않는다.

## 보정·검증·후속 경계

1. 고정 원 head 적용과 예상 tree·중복을 원장으로 확인했다.
2. 승인된 기존 미달 페이지를 공통 로직에서 보정하고 단계별 일반 커밋으로 고정했다.
3. 관련 전쪽 Native/fresh WASM 선행 검증·직접 판독 뒤 정식 소유 회귀를 보강하고, 당시 후보의 최적화 WASM 전쪽과 기준 대비 하락을 검증했다. 이후 후보 보정에 대한 비교는 별도로 진행한다.
4. 이전 전체 회귀와 이후 선택 회귀는 실행 당시 후보를 구분해 기록한다. 작업지시자 지시에 따라 전체 회귀는 재실행하지 않으며, 현재 남은 검증과 미해결 기준은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)에서 추적한다.
5. 최종 로컬 커밋·본문을 준비하고 작업지시자 승인 후 원격 push·통합 PR 제출을 진행한다. 이후 CI·머지·원 PR 종료는 별도 후속 조건이다.

보정 SHA: 이 PR 단독 보정 없음; 공통 통합 보정은 상위 계획 참조.

원 contributor head를 rewrite하거나 원격 브랜치를 변경하지 않았다. 되돌림은 필요 시 이 원장의 적용 commit과
관련 공통 보정을 검토해 별도 revert로 처리하며 다른 PR 변경까지 한꺼번에 롤백하지 않는다.
