# PR #8595 검토 — 표 칸 문단에 그림 삽입

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel의 원 증상·선행 해결, 누적 통합·관계형 회귀·Native/fresh WASM 전쪽·실제 Studio 검증을 완료한 뒤 판정한다. 기여자 원 head의 녹색 CI는 통합 후보의 새 검증으로 대신하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·기여자 | [#8595](https://github.com/edwardkim/rhwp/pull/8595), Kang-T |
| 연결 이슈 | [#8556](https://github.com/edwardkim/rhwp/issues/8556) |
| 원 fork·branch | `Kang-T/rhwp`, `task/8556-insert-picture-in-cell` |
| 고정한 원 head | `c2f05a26f38fc314c012d0655067cd664be35bd1` |
| 고정한 최신 devel | `45684640be5eddcf5972b063260250345b34d928` |
| 검토 branch | `review/integration-20261106-kang-t-picture-editing` |
| reviewer | edwardkim, API 재확인 완료 |
| 경로 | maintainer_general + intake_and_review/local_validation/visual_fixture_evidence/multi_pr_update_branch/first_time_contributor/post_merge |

기존 merged PR이 없는 첫 기여자다. 원 SHA와 원 fork branch를 보존하며 PR별 판정·증거를 분리한다. 이 세션의 승인 범위는 로컬 접수·검토다. 원격 push·댓글·새 PR·merge와 workflow 취소·재실행은 수행하지 않는다.

## 변경과 검토 범위

원 PR의 표 칸 문단에 그림 삽입 주장과 연결 이슈를 검증한다. 원 제공물과 동일한 HWP·Mac 한컴 Print PDF를 재사용한다. 소스·sink·선행 해결 PR, Storage 자원 예산·공통 편집/Undo 계약과 현재 devel의 배치·그림 기능을 함께 대조한다.

## 검증 입력과 결과

현재 상세 검증 전이다. 통합 순서 #8566 → #8595 → #8632, 적용 SHA·충돌·의존성·검사와 시각 결과는 이 PR 기준으로 갱신한다. 원 PR의 실제 CI는 전부 success이고 미해결 실패 check는 없었다. 해당 값은 원 head의 결과이며 로컬 통합 완료 판정이 아니다.

## Merge 후 contributor PR comment 계획

원격 승인·실제 merge·운영 기록·최종 devel 동기화 뒤에만 게시한다. 첫 기여에 대한 감사와 환영, 원 기여와 메인테이너 보정의 구분, 실제 CI/Native/fresh WASM 전쪽·CDP 검증, 각 원 주장 입력/쪽의 review·overlay를 실제 merge SHA에 고정해 준비한다. 현재는 게시 전이며 실제 결과·이미지가 준비된 뒤 계획을 구체화한다. 원 fork branch는 삭제하지 않는다.


## 누적 통합 상태

[공통 검토·보정 계획](kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-완료)에 따라 작업지시자가 두 기능 보존·메인테이너 보정과 필수 로컬 검증을 승인했다. 누적 통합 `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에서 WASM API와 mutation registry에 두 API를 모두 보존했고 세 원 head의 ancestry를 확인했다. 실제 동작·시각 검증은 진행 중이며 원격 작업은 아직 승인 전이다.

## 공통 조판 규칙 검토 — 실행 전 추적

`insertPictureInCell → insert_picture_in_cell_native → 공통 insert_picture_native_anchored → 셀 문단 Picture(Para/Column/flow_with_text) → cell_units_uncached → cached cell_units → 행 컷/소유/예약 높이 → 셀 배치`를 추적한다. 변경한 비인라인 그림 높이는 텍스트 옆 점유 띠와 겹치는 부분을 한 번만 계상하며 TopAndBottom 별도 띠는 보존한다. 저장·재조판·분할 시작/끝·후속 본문과 최근 #8658의 선언 높이 경계를 대조한다. 신규 API와 기존 UI의 삽입 경로는 구분하며 기존 메뉴가 자동으로 신규 API를 소비한다고 간주하지 않는다.

빈 경로와 마지막 셀 문단 번호 오류의 실패 원자성, 기존 본문/글상자 삽입의 컨트롤 스트림 갭과 저장 재열기, Storage 소비 경계를 실제로 확인한다. 아직 실행 전이므로 완료 판정이 아니다.
