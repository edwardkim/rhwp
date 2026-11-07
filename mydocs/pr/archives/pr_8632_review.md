# PR #8632 검토 — 다른 쪽으로 끌어 놓은 그림의 앵커 이동

## 최종 판정

**승인 — 메인테이너 보정을 포함한 통합 head.** 승인된 충돌·경계 보정과 필수 로컬 검증을 완료했다. 최신 devel `a7feb849e1`을 별도로 fresh 빌드해 같은 조건으로 대조했으며 기능과 정상 출력을 보존했다. 원 기여 head의 CI, 보정 후보의 로컬 검증, 수용한 원격 제출 head의 CI를 구분한다. 작업지시자가 원격 처리를 승인했으며, 통합 PR #8725를 exact-head CI·self-review 후 merge했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·기여자 | [#8632](https://github.com/edwardkim/rhwp/pull/8632), Kang-T |
| 연결 이슈 | [#8557](https://github.com/edwardkim/rhwp/issues/8557) |
| 원 fork·branch | `Kang-T/rhwp`, `task/8557-relocate-dragged-picture` |
| 고정한 원 head | `a1146f76a0cbafcb11691f73b576dcb428bd62e0` |
| 최초 기준 devel | `45684640be5eddcf5972b063260250345b34d928` |
| 갱신한 devel | `a7feb849e11642b1438b7e9ee460ff4c8a7c946e` (#8696) |
| Rust 빌드 source / Studio 보정 | `226f0d487cc8d72305979ceb8ab46832017634d0` / `63236b9cc5` |
| 검토 branch | `review/integration-20261106-kang-t-picture-editing` |
| reviewer | edwardkim, API 재확인 완료 |
| 경로 | maintainer_general + intake_and_review/local_validation/visual_fixture_evidence/multi_pr_update_branch/first_time_contributor/post_merge |

기존 merged PR이 없는 첫 기여자다. 원 SHA와 원 fork branch를 보존하며 PR별 판정·증거를 분리한다. 최초 승인은 로컬 접수·검토였고 이후 작업지시자가 통합 PR의 push·CI·self-review·merge·후속 처리를 승인했다. workflow 취소·재실행은 수행하지 않았다.

## 변경과 검토 범위

원 PR의 다른 쪽으로 끌어 놓은 그림의 앵커 이동 주장과 연결 이슈를 검증한다. 원 제공물과 동일한 HWP·Mac 한컴 Print PDF를 재사용한다. 소스·sink·선행 해결 PR, Storage 자원 예산·공통 편집/Undo 계약과 현재 devel의 배치·그림 기능을 함께 대조한다.

## 검증 입력과 결과

원 head의 CI는 모두 success였다. 별도로 최신 통합 후보에서 전체 회귀 12,778건, Native 공식 3종, Studio 2,745건·타입·build, 원 13쪽의 Native/fresh optimized WASM과 실제 CDP를 실행했다. 후보의 실제 수치와 출처는 [공통 검증 증거](../assets/kang-t-20261106/README.md)에 연결한다. workspace build·all-target Clippy를 포함한 필수 검사를 모두 통과했고, 최신 base와의 같은 조건 대조도 완료했다. [최종 집계](../assets/kang-t-20261106/latest/validation-summary.json)에 수치·실행 출처·보완 후 재실행을 구분했다. 승인된 원격 절차로 PR #8725의 최신 head CI·최종 정책·self-review를 완료한 뒤 merge했다.

## Merge 후 contributor PR comment 계획

원격 승인·실제 merge·운영 기록·최종 devel 동기화 뒤에만 게시한다. 첫 기여에 대한 감사와 환영, 원 기여와 메인테이너 보정의 구분, 실제 CI/Native/fresh WASM 전쪽·CDP 검증, 각 원 주장 입력/쪽의 review·overlay를 실제 merge SHA에 고정해 준비한다. 원 사례의 4쪽 전체 TSV와 3쪽 Native review·fresh WASM overlay를 준비했다. 게시 초안은 로컬에 있으며 실제 통합 PR·성공 CI·merge SHA·archive 경로를 확정하고 devel의 asset 존재를 확인한 뒤 사용한다. 원 fork branch는 삭제하지 않는다.


## 누적 통합 상태

[공통 검토·보정 계획](../kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-완료)에 따라 작업지시자가 두 기능 보존·메인테이너 보정과 필수 로컬 검증을 승인했다. 누적 통합 `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에서 WASM API와 mutation registry에 두 API를 모두 보존했고 세 원 head의 ancestry를 확인했다. 같은 Rust source digest의 Native/fresh optimized WASM과 이후 Studio-only 보정을 실제 검증했다. 원 기여 ancestry를 보존했고 승인된 원격 절차로 수용했다.

## 공통 편집 규칙 검토 — 실행 전 추적

`실제 mouse drag → updatePictureMoveDrag(시작 쪽 좌표/현재 놓을 쪽 분리) → finishPictureMoveDrag → snapshot relocateFloatingPicture → 컨트롤 제거/목적 경로 재번호 → 앵커 재삽입/재조판 → 실제 배치/선택/Undo`를 확인한다. BinData를 복제하지 않고 소유권을 옮기며 표 칸은 Para/Column/flow_with_text를 적용한다. 같은 본문 문단의 그림 제거로 목적 표의 번호가 바뀌는 경계에서 Native 소유권과 Studio 선택 ref가 함께 맞는지도 실제 실행할 예정이다. 함수 존재나 원 CI 성공으로 해당 경계를 통과 처리하지 않는다.

## 메인테이너 실행에서 확인한 경계와 보정

최초 기준과 최신 devel `a7feb849e1`의 별도 optimized WASM에서 CDP 19222 실제 mouse down/move/up을 실행하면 4쪽 그림을 2쪽에 놓아도 4쪽에 남았다. 원 기여를 통합한 `dc8d869db1`은 그림을 2쪽으로 옮겼지만, 같은 바깥 문단에서 원본 그림(control 0)을 빼며 표 번호가 1→0으로 바뀐 뒤 Studio의 선택 ref에는 1이 남았다. 실제 소유 `cellPath=[0,119,0]`와 선택 `[1,119,0]`의 불일치를 실행·스크린샷으로 확인했다. 독립 PNG 점수로 이 편집 경계를 대신하지 않았다.

또 공개 WASM에서 원본 그림의 캡션을 목적 경로로 주면 오류를 반환하면서 그림을 삭제했다. 삭제 전에 목적 문단뿐 아니라 허용하는 컨테이너 종류도 확인하고, Native가 실제로 계산한 도착 경로를 반환해 Studio 위치 보정·선택에서 함께 소비하도록 보정한다. 동일 문단 재번호·오류 원자성·중첩 칸 소유의 구조 회귀를 추가했다. 보정 뒤 공개 WASM 실패 원자성과 실제 도착 경로, Undo/Redo·저장·최종 전쪽 검증을 통과했다.

## 2026-11-07 추가 시각 판독과 포인터 보정

최초 CDP 검사는 도착 페이지·소유·선택·이력·저장을 보호했지만 칸→본문의 위치까지 충분히 보호하지 못했다. 확대 재판독에서 실제 그림 y=961.3과 놓은 위치 y≈110의 큰 차이를 확인했다. 칸의 제한을 받은 이동 미리보기 위치를 도착점으로 변환하던 것이 원인이다. `mousedown의 실제 bbox/포인터 → grabOffsetX/Y → dropX/Y에서 grab offset 차감 → 도착 컨테이너 원점의 실제 배치 → 공통 setter/선택/이력`으로 연결했다. 원 기여 Native API의 계약과 기존 칸 containment는 유지한다.

보정 `63236b9cc5`와 실제 제품 종료 함수를 실행하는 단위 검사 3건을 기록했다. 옛 hit 경로와 preview 위치를 쓰는 최소 잘못된 구현은 각각 실패한다. 뒤에 기록한 최종 fresh WASM의 실제 포인터·소유·이력 재실행을 완료 판정에 사용했다. Native/WASM 빌드의 Rust source `226f0d487c`와 이후 Studio-only 보정을 구분한다.

최종 optimized WASM `46ef070e9e5b656598a42bdfe9e8ddcb81955cbff77623ccaf572238ec9f160e`에서 포인터 보정 뒤 실제 앞/뒤 페이지·같은 쪽·칸→본문 이동, Undo/Redo·저장 재열기를 다시 확인했다. 본문 도착은 처음 잡은 점과 놓은 점이 일치하고, 칸 도착은 source 제거에 따른 본문 재조판 뒤에도 선택한 칸의 정체성과 실제 render tree의 내부 포함을 지킨다. [실행 결과](../assets/kang-t-20261106/latest/cdp/scope.json).

최신 devel #8696의 글꼴 수정은 원 입력 3종·Native/WASM 6쌍의 PNG가 같으며, 그림 배치 정상 대조군 10쪽씩도 20쌍이 같다. 필수 검증의 미해결 항목이나 시각 예외는 없다. [최신 글꼴 보존](../assets/kang-t-20261106/latest/latest-issue8687-preservation.json)·[배치 보존](../assets/kang-t-20261106/latest/latest-control-preservation.json).

최신 기준의 실제 mouse 음성 대조에서도 4쪽 그림은 원 앵커에 남았다. [최신 devel 재현](../assets/kang-t-20261106/latest/cdp/base-latest-negative.json)과 [후보의 실제 앞/뒤 이동·이력](../assets/kang-t-20261106/latest/cdp/scope.json)을 분리해 보존했다.

## 원격 수용 결과 — 2026-11-07

[통합 PR #8725](https://github.com/edwardkim/rhwp/pull/8725)의 검증 head `07510f602a6081d90c94397fe29613420f3e7f63`, merge `331521a3b1fb5de5dfa39b0707c0b295b90f9dc4`를 확인했다. 원 PR도 자동 MERGED이며 원 기여 SHA와 fork branch를 보존했다. [실제 CI·운영 기록](../assets/kang-t-20261106/remote/completion.md)에 전체 결과와 순서를 연결한다. 기존 CodeQL #478·#483·#484는 생성일과 불변 blob을 대조했고 분류를 유지했다.

이 운영 기록 뒤 최종 devel sync·연결 이슈 후속 근거 댓글·원 PR별 감사 댓글·종료 정리를 수행한다. 실제 댓글과 정리 결과는 아래 원격 기록의 최종 영수증으로 보완한다.
