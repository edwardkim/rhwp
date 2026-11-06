# PR #8632 검토 — 다른 쪽으로 끌어 놓은 그림의 앵커 이동

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel의 원 증상·선행 해결, 누적 통합·관계형 회귀·Native/fresh WASM 전쪽·실제 Studio 검증을 완료한 뒤 판정한다. 기여자 원 head의 녹색 CI는 통합 후보의 새 검증으로 대신하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·기여자 | [#8632](https://github.com/edwardkim/rhwp/pull/8632), Kang-T |
| 연결 이슈 | [#8557](https://github.com/edwardkim/rhwp/issues/8557) |
| 원 fork·branch | `Kang-T/rhwp`, `task/8557-relocate-dragged-picture` |
| 고정한 원 head | `a1146f76a0cbafcb11691f73b576dcb428bd62e0` |
| 고정한 최신 devel | `45684640be5eddcf5972b063260250345b34d928` |
| 검토 branch | `review/integration-20261106-kang-t-picture-editing` |
| reviewer | edwardkim, API 재확인 완료 |
| 경로 | maintainer_general + intake_and_review/local_validation/visual_fixture_evidence/multi_pr_update_branch/first_time_contributor/post_merge |

기존 merged PR이 없는 첫 기여자다. 원 SHA와 원 fork branch를 보존하며 PR별 판정·증거를 분리한다. 이 세션의 승인 범위는 로컬 접수·검토다. 원격 push·댓글·새 PR·merge와 workflow 취소·재실행은 수행하지 않는다.

## 변경과 검토 범위

원 PR의 다른 쪽으로 끌어 놓은 그림의 앵커 이동 주장과 연결 이슈를 검증한다. 원 제공물과 동일한 HWP·Mac 한컴 Print PDF를 재사용한다. 소스·sink·선행 해결 PR, Storage 자원 예산·공통 편집/Undo 계약과 현재 devel의 배치·그림 기능을 함께 대조한다.

## 검증 입력과 결과

현재 상세 검증 전이다. 통합 순서 #8566 → #8595 → #8632, 적용 SHA·충돌·의존성·검사와 시각 결과는 이 PR 기준으로 갱신한다. 원 PR의 실제 CI는 전부 success이고 미해결 실패 check는 없었다. 해당 값은 원 head의 결과이며 로컬 통합 완료 판정이 아니다.

## Merge 후 contributor PR comment 계획

원격 승인·실제 merge·운영 기록·최종 devel 동기화 뒤에만 게시한다. 첫 기여에 대한 감사와 환영, 원 기여와 메인테이너 보정의 구분, 실제 CI/Native/fresh WASM 전쪽·CDP 검증, 각 원 주장 입력/쪽의 review·overlay를 실제 merge SHA에 고정해 준비한다. 현재는 게시 전이며 실제 결과·이미지가 준비된 뒤 계획을 구체화한다. 원 fork branch는 삭제하지 않는다.


## 누적 통합 상태

[공통 검토·보정 계획](kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-대기)을 따른다. #8566·#8595는 clean merge, #8632 누적 적용은 mutation registry와 WASM API의 두 기능 추가 위치가 겹쳐 승인 대기다. 현재 코드는 앞선 두 원 기여의 통합 상태이며 #8632 source는 fetch ref에 보존했다. 최종 동작·시각 검증은 아직 진행 전이다.
