# PR #8209 적용·보정 실행 기록

- 원 head: `01588f1e997b37795d2dbafecdf1f015341d553c`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8209_review.md](pr_8209_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `ef089bc668d64e02b10a596d927d9d3b0bc347a6` | `b3e02b4657ca534dc7d0980ffd429b9cffe465cc` |
| 2 | `1091fb933df18e3171b34f048d0de24d4a054562` | `8b09ea80ab2e1094b5d437672cb7708a5702629b` |
| 3 | `ca45a7162a9af5102b10ff941d92ab6625a4cfeb` | `6a9b72e2b24cd71557825926f8f6e93e7bc79bae` |
| 4 | `65f281399bae5457f021585532eaab2276e6540c` | `4e1b4a3b7135ed2681046e52e96f08abf2fde830` |
| 5 | `92c64aef7f4971a5b1fb563c7396c284f1e61a06` | `c073ea740bf284005d437ea89e08f36bfad501f4` |
| 6 | `01588f1e997b37795d2dbafecdf1f015341d553c` | `993d9a5e82491a7136a1d3549a7862dff9ba959f` |

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
