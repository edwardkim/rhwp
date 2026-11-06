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

[공통 검토·보정 계획](kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-완료)에 따라 작업지시자가 두 기능 보존·메인테이너 보정과 필수 로컬 검증을 승인했다. 누적 통합 `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에서 WASM API와 mutation registry에 두 API를 모두 보존했고 세 원 head의 ancestry를 확인했다. 실제 동작·시각 검증은 진행 중이며 원격 작업은 아직 승인 전이다.

## 공통 편집 규칙 검토 — 실행 전 추적

`실제 mouse drag → updatePictureMoveDrag(시작 쪽 좌표/현재 놓을 쪽 분리) → finishPictureMoveDrag → snapshot relocateFloatingPicture → 컨트롤 제거/목적 경로 재번호 → 앵커 재삽입/재조판 → 실제 배치/선택/Undo`를 확인한다. BinData를 복제하지 않고 소유권을 옮기며 표 칸은 Para/Column/flow_with_text를 적용한다. 같은 본문 문단의 그림 제거로 목적 표의 번호가 바뀌는 경계에서 Native 소유권과 Studio 선택 ref가 함께 맞는지도 실제 실행할 예정이다. 함수 존재나 원 CI 성공으로 해당 경계를 통과 처리하지 않는다.

## 메인테이너 실행에서 확인한 경계와 보정

최신 devel의 optimized WASM에서 CDP 19222 실제 mouse down/move/up을 실행하면 4쪽 그림을 2쪽에 놓아도 4쪽에 남았다. 원 기여를 통합한 `dc8d869db1`은 그림을 2쪽으로 옮겼지만, 같은 바깥 문단에서 원본 그림(control 0)을 빼며 표 번호가 1→0으로 바뀐 뒤 Studio의 선택 ref에는 1이 남았다. 실제 소유 `cellPath=[0,119,0]`와 선택 `[1,119,0]`의 불일치를 실행·스크린샷으로 확인했다. 독립 PNG 점수로 이 편집 경계를 대신하지 않았다.

또 공개 WASM에서 원본 그림의 캡션을 목적 경로로 주면 오류를 반환하면서 그림을 삭제했다. 삭제 전에 목적 문단뿐 아니라 허용하는 컨테이너 종류도 확인하고, Native가 실제로 계산한 도착 경로를 반환해 Studio 위치 보정·선택에서 함께 소비하도록 보정한다. 동일 문단 재번호·오류 원자성·중첩 칸 소유의 구조 회귀를 추가했다. 현재는 보정 후 검증 전이며 Undo/Redo·저장·최종 전쪽 결과로 완료를 판정한다.
