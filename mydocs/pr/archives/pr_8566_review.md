# PR #8566 검토 — 편집 재조판의 문단 세로 정렬 기준선

## 최종 판정

**승인 — 메인테이너 보정을 포함한 통합 head.** 승인된 충돌·경계 보정과 필수 로컬 검증을 완료했다. 최신 devel `a7feb849e1`을 별도로 fresh 빌드해 같은 조건으로 대조했으며 기능과 정상 출력을 보존했다. 원 기여 head의 CI, 보정 후보의 로컬 검증, 수용한 원격 제출 head의 CI를 구분한다. 작업지시자가 원격 처리를 승인했으며, 통합 PR #8725를 exact-head CI·self-review 후 merge했다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·기여자 | [#8566](https://github.com/edwardkim/rhwp/pull/8566), Kang-T |
| 연결 이슈 | [#8555](https://github.com/edwardkim/rhwp/issues/8555) |
| 원 fork·branch | `Kang-T/rhwp`, `task/8555-vertical-align-baseline` |
| 고정한 원 head | `ec64701aca2c20ab2ad17bcac6238e14a96f210b` |
| 최초 기준 devel | `45684640be5eddcf5972b063260250345b34d928` |
| 갱신한 devel | `a7feb849e11642b1438b7e9ee460ff4c8a7c946e` (#8696) |
| Rust 빌드 source / Studio 보정 | `226f0d487cc8d72305979ceb8ab46832017634d0` / `63236b9cc5` |
| 검토 branch | `review/integration-20261106-kang-t-picture-editing` |
| reviewer | edwardkim, API 재확인 완료 |
| 경로 | maintainer_general + intake_and_review/local_validation/visual_fixture_evidence/multi_pr_update_branch/first_time_contributor/post_merge |

기존 merged PR이 없는 첫 기여자다. 원 SHA와 원 fork branch를 보존하며 PR별 판정·증거를 분리한다. 최초 승인은 로컬 접수·검토였고 이후 작업지시자가 통합 PR의 push·CI·self-review·merge·후속 처리를 승인했다. workflow 취소·재실행은 수행하지 않았다.

## 변경과 검토 범위

원 PR의 편집 재조판의 문단 세로 정렬 기준선 주장과 연결 이슈를 검증한다. 원 제공물과 동일한 HWP·Mac 한컴 Print PDF를 재사용한다. 소스·sink·선행 해결 PR, Storage 자원 예산·공통 편집/Undo 계약과 현재 devel의 배치·그림 기능을 함께 대조한다.

## 검증 입력과 결과

원 head의 CI는 모두 success였다. 별도로 최신 통합 후보에서 전체 회귀 12,778건, Native 공식 3종, Studio 2,745건·타입·build, 원 13쪽의 Native/fresh optimized WASM과 실제 CDP를 실행했다. 후보의 실제 수치와 출처는 [공통 검증 증거](../assets/kang-t-20261106/README.md)에 연결한다. workspace build·all-target Clippy를 포함한 필수 검사를 모두 통과했고, 최신 base와의 같은 조건 대조도 완료했다. [최종 집계](../assets/kang-t-20261106/latest/validation-summary.json)에 수치·실행 출처·보완 후 재실행을 구분했다. 승인된 원격 절차로 PR #8725의 최신 head CI·최종 정책·self-review를 완료한 뒤 merge했다.

## Merge 후 contributor PR comment 계획

원격 승인·실제 merge·운영 기록·최종 devel 동기화 뒤에만 게시한다. 첫 기여에 대한 감사와 환영, 원 기여와 메인테이너 보정의 구분, 실제 CI/Native/fresh WASM 전쪽·CDP 검증, 각 원 주장 입력/쪽의 review·overlay를 실제 merge SHA에 고정해 준비한다. 원 사례의 4쪽 전체 TSV와 1쪽 Native review·fresh WASM overlay를 준비했다. 게시 초안은 로컬에 있으며 실제 통합 PR·성공 CI·merge SHA·archive 경로를 확정하고 devel의 asset 존재를 확인한 뒤 사용한다. 원 fork branch는 삭제하지 않는다.


## 누적 통합 상태

[공통 검토·보정 계획](../kang_t_20261106_review_plan.md#누적-통합에서-확인한-보정-방침--승인-완료)에 따라 작업지시자가 두 기능 보존·메인테이너 보정과 필수 로컬 검증을 승인했다. 누적 통합 `46af1d5d22e4fd2fd567a893bd36ce43f12d2ab6`에서 WASM API와 mutation registry에 두 API를 모두 보존했고 세 원 head의 ancestry를 확인했다. 같은 Rust source digest의 Native/fresh optimized WASM과 이후 Studio-only 보정을 실제 검증했다. 원 기여 ancestry를 보존했고 승인된 원격 절차로 수용했다.

## 공통 조판 규칙 검토 — 실행 전 추적

`ParaShape.attr1[20..21] → ResolvedParaStyle.vertical_alignment → publish_regenerated_rows → publish_vertical_alignment_baselines → LineSeg.baseline_distance`를 확인했다. 먼저 기존 수식 줄 구성·메트릭을 발행한 뒤 정렬점만 저장한다. 정렬 0은 기존 글꼴 기준선을 유지한다. 저장 정보 재사용과 실제 편집 후 재조판을 각각 실행하고 원본·편집본의 페이지/내용 보존 및 Native/Canvas/SVG 소비를 대조한다. 코퍼스 집계는 기여자 관측이며 메인테이너의 실제 검증으로 표시하지 않는다.

## 정상 저장본과 편집 재조판의 독립 대조

원본 공개 HWP의 가운데 정렬 줄 111개는 모두 실제 저장 `baseline_distance=text_height/2`였으며 첫 셀 5/8/11의 값은 600/1200 HWPUNIT이다. [원본 관측](../assets/kang-t-20261106/stored-baseline-observation.json)은 기여자의 전체 corpus 수치와 구분한다. 같은 실제 셀에 글자를 넣었다 지워 저장한 뒤 전 4쪽을 다시 비교했고 후보 Native/WASM 최저는 각각 99.13472%/99.09683%다. 원 문제는 화면에서 작게 보이는 저장 LINE_SEG의 잘못된 값이며, 새 기본 API 계약은 정렬 0/1/2/3을 함께 검사한다.

최신 devel에서 같은 셀 5를 삽입→삭제해 원문을 유지한 채 저장하면 가운데 정렬 기준선이 원 저장값 600에서 1020으로 바뀐다. 후보는 600을 보존하며, 가운데 정렬 111줄 모두 원 저장 계약에 맞는다. 양쪽 모두 전체 4,442자를 보존했다. 렌더러가 기준선을 제한하므로 화면 점수만으로 저장값 수정을 주장하지 않는다. [실제 저장 대조](../assets/kang-t-20261106/latest/center-edit-contract.json)·[본문 보존](../assets/kang-t-20261106/latest/center-edit-text-preservation.json).

최신 devel #8696의 글꼴 수정은 원 입력 3종·Native/WASM 6쌍의 PNG가 같으며, 그림 배치 정상 대조군 10쪽씩도 20쌍이 같다. 필수 검증의 미해결 항목이나 시각 예외는 없다. [최신 글꼴 보존](../assets/kang-t-20261106/latest/latest-issue8687-preservation.json)·[배치 보존](../assets/kang-t-20261106/latest/latest-control-preservation.json).

## 원격 수용 결과 — 2026-11-07

[통합 PR #8725](https://github.com/edwardkim/rhwp/pull/8725)의 검증 head `07510f602a6081d90c94397fe29613420f3e7f63`, merge `331521a3b1fb5de5dfa39b0707c0b295b90f9dc4`를 확인했다. 원 PR도 자동 MERGED이며 원 기여 SHA와 fork branch를 보존했다. [실제 CI·운영 기록](../assets/kang-t-20261106/remote/completion.md)에 전체 결과와 순서를 연결한다. 기존 CodeQL #478·#483·#484는 생성일과 불변 blob을 대조했고 분류를 유지했다.

이 운영 기록 뒤 최종 devel sync·연결 이슈 후속 근거 댓글·원 PR별 감사 댓글·종료 정리를 수행한다. 실제 댓글과 정리 결과는 아래 원격 기록의 최종 영수증으로 보완한다.
