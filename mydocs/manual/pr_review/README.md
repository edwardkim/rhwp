---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# PR review 조건별 가이드 선택표

이 디렉터리는 [PR 리뷰 · 통합 워크플로우 매뉴얼](../pr_review_workflow.md)의 자식 가이드 모음이다.
모 문서를 먼저 읽은 뒤, 아래에서 기본 경로 하나와 필요한 보조 경로를 선택한다.

## 선택 순서

1. **누가 어떤 PR을 처리하는가**로 기본 경로 하나를 선택한다.
2. **무엇을 바꾸고 현재 어느 단계인가**로 보조 경로를 추가한다.
3. 상태를 바꾸기 전에 선택 결과와 읽은 문서를 보고한다.
4. 새 head가 생기거나 merge 단계로 넘어가면 선택표를 다시 확인한다.

| 문서 | 읽는 시점 |
| --- | --- |
| [intake_and_review.md](intake_and_review.md) | 모든 정식 PR review의 접수, reviewer assign, review 문서 작성 |
| [maintainer_general.md](maintainer_general.md) | maintainer가 외부 PR을 일반 경로로 처리 |
| [collaborator_self_merge.md](collaborator_self_merge.md) | collaborator 자신의 PR을 준비·merge |
| [collaborator_external_pr.md](collaborator_external_pr.md) | collaborator가 contributor PR head를 보정하거나 기록을 더함 |
| [local_validation.md](local_validation.md) | local fetch, simulation, Cargo/npm/fixture 검증 |
| [visual_fixture_evidence.md](visual_fixture_evidence.md) | renderer 또는 HWP/HWPX/PDF fixture·페이지·시각 검증 |
| [multi_pr_update_branch.md](multi_pr_update_branch.md) | 대량 유입, 다수 PR 누적 검토, update branch, stale CI |
| [review_only_fast_pass.md](review_only_fast_pass.md) | code PR 뒤 review-only commit 또는 PR 전체가 허용된 review-only 범위 |
| [post_merge.md](post_merge.md) | merge 이후 문서·asset·issue·comment·정리 |
| [rework_and_exceptions.md](rework_and_exceptions.md) | 재작업 요청/close, Dependabot, 오래된 base, 대형 PR |

## 대표 조합

| 상황 | 기본 경로 | 보조 경로 |
| --- | --- | --- |
| 단일 외부 코드 PR을 maintainer가 검토 | maintainer_general | intake_and_review, local_validation, post_merge |
| renderer PR과 신규 HWP fixture | maintainer_general | intake_and_review, local_validation, visual_fixture_evidence, post_merge |
| 한 기여자의 여러 PR을 체리픽 누적 검토 | maintainer_general | intake_and_review, local_validation, multi_pr_update_branch, 필요 시 visual_fixture_evidence, post_merge |
| collaborator 본인 PR | collaborator_self_merge | intake_and_review, local_validation, 필요 시 visual_fixture_evidence |
| collaborator가 contributor PR에 보정 commit을 추가 | collaborator_external_pr | intake_and_review, local_validation, 필요 시 visual_fixture_evidence와 multi_pr_update_branch, post_merge |
| collaborator가 contributor code를 local 검증한 뒤 review·오늘할일만 source head에 추가 | collaborator_external_pr | intake_and_review, local_validation, review_only_fast_pass, post_merge |
| 완료된 원 PR의 review·asset만 별도 PR로 반영 | collaborator_self_merge | intake_and_review, review_only_fast_pass, post_merge |
| base가 잘못되었거나 재작업 요청 | 해당 기본 경로 | intake_and_review, rework_and_exceptions |

다수 PR을 동시에 다루더라도 reviewer assign, review 문서, 최종 CI·merge 판단은 PR 번호별로 분리한다.
CI 관찰과 읽기 전용 조사는 병렬로 할 수 있지만, Cargo 공유 상태와 GitHub·Git 상태 변경은
[모 문서의 순차 게이트](../pr_review_workflow.md#3-병렬-실행과-순차-게이트)를 따른다.
