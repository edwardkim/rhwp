# PR #8210 적용·보정 실행 기록

- 원 head: `276d569c2f5abce5b77bfb89aed016951e314bd3`
- 통합 branch: `review/integration-20261026-document-updates`
- base: `faa885e8b54ffc7cdf4196cd43a3103f362499a2`
- 완료한 로컬 code/test 검증 head: `2e902abacf328bccdd7780606666ac3b1c5c1ce9`
- 최종 판정·입력·명령·제한: [pr_8210_review.md](pr_8210_review.md)
- 단계 계획·독립 근거: [통합 계획·실패별 선행 분석·실제 소비 경로](<../../../mydocs/working/review_integration_20261026_document_updates_plan.md>)

## 실제 체리픽 순서

| 순서 | 원 SHA | 실제 적용 SHA / 처리 |
| ---: | --- | --- |
| 1 | `ad6a918fa7e872d84f13f925c4da35e8f0b49819` | 동일 patch 선행 반영으로 중복 제외 |
| 2 | `b360b1cd3437327a81304b70cbb32a0af32ff7bf` | 동일 patch 선행 반영으로 중복 제외 |
| 3 | `ab52479a8e6abd7291c134005356f8a7d775a1c6` | 동일 patch 선행 반영으로 중복 제외 |
| 4 | `3920830577969a6bfbe256d6970a21f3216dc64e` | 동일 patch 선행 반영으로 중복 제외 |
| 5 | `e405b57d2a648db40993b74dbeb9bb40fd8a7df9` | 동일 patch 선행 반영으로 중복 제외 |
| 6 | `df4b51036e736bf626032c8afe23240987ef3cd1` | 동일 patch 선행 반영으로 중복 제외 |
| 7 | `e0355691ab52253e4c6b5b4714b5b64a85804869` | `f7638637720bd559c2e61f12fb7ca35124588add` |
| 8 | `ef90aed188c899fb5d67d33725523d057d925765` | `7bf0cb93e702390c3bb4587e83adf7c42677143c` |
| 9 | `a6060b598c785cc5a7c1d63a1e3f4ff5e46b62dd` | `53f4072d1429b3c92d8b3869954d2b32e802b875` |
| 10 | `276d569c2f5abce5b77bfb89aed016951e314bd3` | `b6cd8e2f28c5d013a04e8e224fa6311e7731097a` |

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

## 2026-10-28 최신 원 PR 대조

최신 원 PR과 후보는 저장 첫 행 프레임 fallback 평가 순서가 다르다. 전체 행 프레임이 존재하면 행 수는 원본 표의 행 수와 같고 첫 행 프레임은 그 이하이므로 이 차이는 선택 높이를 바꾸지 않는다. [커밋별 대조 근거](../../working/assets/review_integration_20261026/validation/post-rebase-8328/source-pr-paired-range-diff-review.md).
