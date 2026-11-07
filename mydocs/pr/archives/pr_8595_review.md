# PR #8595 검토 — 표 칸 문단에 그림 삽입

## 최종 판정

**승인 — 메인테이너 보정을 포함한 통합 head.** 승인된 충돌·경계 보정과 필수 로컬 검증을 완료했다. 최신 devel `a7feb849e1`을 별도로 fresh 빌드해 같은 조건으로 대조했으며 기능과 정상 출력을 보존했다. 원 기여 head의 CI, 보정 후보의 로컬 검증, 수용한 원격 제출 head의 CI를 구분한다. 작업지시자가 원격 처리를 승인했으며, 통합 PR #8725를 exact-head CI·self-review 후 merge했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·기여자 | [#8595](https://github.com/edwardkim/rhwp/pull/8595), Kang-T |
| 연결 이슈 | [#8556](https://github.com/edwardkim/rhwp/issues/8556) |
| 원 fork·branch | `Kang-T/rhwp`, `task/8556-insert-picture-in-cell` |
| 고정한 원 head | `c2f05a26f38fc314c012d0655067cd664be35bd1` |
| 최초 기준 devel | `45684640be5eddcf5972b063260250345b34d928` |
| 갱신한 devel | `a7feb849e11642b1438b7e9ee460ff4c8a7c946e` (#8696) |
| Rust 빌드 source / Studio 보정 | `226f0d487cc8d72305979ceb8ab46832017634d0` / `63236b9cc5` |
| 검토 branch | `review/integration-20261106-kang-t-picture-editing` |
| reviewer | edwardkim, API 재확인 완료 |
| 경로 | maintainer_general + intake_and_review/local_validation/visual_fixture_evidence/multi_pr_update_branch/first_time_contributor/post_merge |

기존 merged PR이 없는 첫 기여자다. 원 SHA와 원 fork branch를 보존하며 PR별 판정·증거를 분리한다. 최초 승인은 로컬 접수·검토였고 이후 작업지시자가 통합 PR의 push·CI·self-review·merge·후속 처리를 승인했다. workflow 취소·재실행은 수행하지 않았다.

## 변경과 검토 범위

원 PR의 표 칸 문단에 그림 삽입 주장과 연결 이슈를 검증한다. 원 제공물과 동일한 HWP·Mac 한컴 Print PDF를 재사용한다. 소스·sink·선행 해결 PR, Storage 자원 예산·공통 편집/Undo 계약과 현재 devel의 배치·그림 기능을 함께 대조한다.

## 검증 입력과 결과

원 head의 CI는 모두 success였다. 별도로 최신 통합 후보에서 전체 회귀 12,778건, Native 공식 3종, Studio 2,745건·타입·build, 원 13쪽의 Native/fresh optimized WASM과 실제 CDP를 실행했다. 후보의 실제 수치와 출처는 [공통 검증 증거](../assets/kang-t-20261106/README.md)에 연결한다. workspace build·all-target Clippy를 포함한 필수 검사를 모두 통과했고, 최신 base와의 같은 조건 대조도 완료했다. [최종 집계](../assets/kang-t-20261106/latest/validation-summary.json)에 수치·실행 출처·보완 후 재실행을 구분했다. 승인된 원격 절차로 PR #8725의 최신 head CI·최종 정책·self-review를 완료한 뒤 merge했다.

## Merge 후 contributor PR comment 계획

원격 승인·실제 merge·운영 기록·최종 devel 동기화 뒤에만 게시한다. 첫 기여에 대한 감사와 환영, 원 기여와 메인테이너 보정의 구분, 실제 CI/Native/fresh WASM 전쪽·CDP 검증, 각 원 주장 입력/쪽의 review·overlay를 실제 merge SHA에 고정해 준비한다. 원 사례의 5쪽 전체 TSV와 1쪽 Native review·fresh WASM overlay를 준비했다. 게시 초안은 로컬에 있으며 실제 통합 PR·성공 CI·merge SHA·archive 경로를 확정하고 devel의 asset 존재를 확인한 뒤 사용한다. 원 fork branch는 삭제하지 않는다.


## 누적 통합 상태

[공통 검토·보정 계획](../kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-완료)에 따라 작업지시자가 두 기능 보존·메인테이너 보정과 필수 로컬 검증을 승인했다. 누적 통합 `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에서 WASM API와 mutation registry에 두 API를 모두 보존했고 세 원 head의 ancestry를 확인했다. 같은 Rust source digest의 Native/fresh optimized WASM과 이후 Studio-only 보정을 실제 검증했다. 원 기여 ancestry를 보존했고 승인된 원격 절차로 수용했다.

## 공통 조판 규칙 검토 — 실행 전 추적

`insertPictureInCell → insert_picture_in_cell_native → 공통 insert_picture_native_anchored → 셀 문단 Picture(Para/Column/flow_with_text) → cell_units_uncached → cached cell_units → 행 컷/소유/예약 높이 → 셀 배치`를 추적한다. 변경한 비인라인 그림 높이는 텍스트 옆 점유 띠와 겹치는 부분을 한 번만 계상하며 TopAndBottom 별도 띠는 보존한다. 저장·재조판·분할 시작/끝·후속 본문과 최근 #8658의 선언 높이 경계를 대조한다. 신규 API와 기존 UI의 삽입 경로는 구분하며 기존 메뉴가 자동으로 신규 API를 소비한다고 간주하지 않는다.

빈 경로와 마지막 셀 문단 번호 오류의 실패 원자성, 기존 본문/글상자 삽입의 컨트롤 스트림 갭과 저장 재열기, Storage 소비 경계를 실제로 확인한다. 최종 공개 WASM·Native·CDP와 저장 재열기에서 실제 실행을 완료했다.

## 메인테이너 실행에서 확인한 경계와 보정

보정 전 optimized WASM `dc8d869db1`(WASM SHA-256 `d2739886e07842932539a29dcc671ce0f601293257cdd289f27579689082fc19`)의 실제 공개 API를 호출했다. 마지막 셀 문단 999 요청은 오류 뒤에도 BinData를 등록해 저장 해시가 달라졌고, `(1200, 미지정)`·`(미지정, -300)`은 모두 `(0,0)`으로 저장됐다. 같은 PNG·정상 두 오프셋 대조군은 `(1200,-300)`을 유지했다.

공통 그림 삽입에서 실제 도착 문단까지 먼저 확인한 뒤 BinData를 등록하고, 두 선택 인수를 각각 기본값 0으로 해석한다. 기존 셀 밖 본문 sibling 삽입과 글상자 음수 제한은 유지한다. 정상·오류·부분 인수의 합성 상태 회귀를 추가했다. 위 값은 보정 전 실행이며, 보정 뒤 저장 불변과 각 선택 인수의 보존을 최종 공개 WASM에서 확인했다.

## 보정 후 확인

실제 공개 WASM에서 문단 999 오류 뒤 저장 해시는 동일했고 두 선택 오프셋을 독립적으로 보존했다. Native 집중 검사와 저장 재열기를 통과했다. 브라우저의 새 API를 snapshot 명령으로 실행해 사전 가열한 layout cache·칸 소속·Undo/Redo·저장 재열기를 확인했다. 기존 메뉴가 신규 API로 바뀐 것으로 설명하지 않는다. 원 이슈는 별도 칸 문단 API를 제안하며 기존 `insertPicture` 보존을 명시한다.

최신 devel과 후보의 같은 조건 전쪽 비교에서 Print 글꼴 명시 최저는 Native 66.84323%→99.31803%, fresh WASM 66.45711%→99.26090%다. 실제 2쪽에서 기준의 33/34번 이어받기가 Print 및 후보의 34번으로 바로잡혔고 뒤 35~60번 줄·괘선·셀 소속을 직접 대조했다. [전쪽 비교](../assets/kang-t-20261106/latest/latest-base-candidate-primary-comparison.json).

최신 devel #8696의 글꼴 수정은 원 입력 3종·Native/WASM 6쌍의 PNG가 같으며, 그림 배치 정상 대조군 10쪽씩도 20쌍이 같다. 필수 검증의 미해결 항목이나 시각 예외는 없다. [최신 글꼴 보존](../assets/kang-t-20261106/latest/latest-issue8687-preservation.json)·[배치 보존](../assets/kang-t-20261106/latest/latest-control-preservation.json).

## 원격 수용 결과 — 2026-11-07

[통합 PR #8725](https://github.com/edwardkim/rhwp/pull/8725)의 검증 head `07510f602a6081d90c94397fe29613420f3e7f63`, merge `331521a3b1fb5de5dfa39b0707c0b295b90f9dc4`를 확인했다. 원 PR도 자동 MERGED이며 원 기여 SHA와 fork branch를 보존했다. [실제 CI·운영 기록](../assets/kang-t-20261106/remote/completion.md)에 전체 결과와 순서를 연결한다. 기존 CodeQL #478·#483·#484는 생성일과 불변 blob을 대조했고 분류를 유지했다.

이 운영 기록 뒤 최종 devel sync·연결 이슈 후속 근거 댓글·원 PR별 감사 댓글·종료 정리를 수행한다. 실제 댓글과 정리 결과는 아래 원격 기록의 최종 영수증으로 보완한다.
