# PR #3443 검토 기록 — 여섯 다이얼로그 적용·실패 계약 통일

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3443](https://github.com/edwardkim/rhwp/pull/3443) — `Task #3435: [ui] 다이얼로그 [확인]의 라우팅·실패 처리를 공용 헬퍼로 통일` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `641fbc45bec152db16c8e1f4c97c5eb87a89b005` (`fix/2370-dialog-error-handling`) |
| 원 변경 규모 | GitHub 표면 20 files, +471 / -118, 4 commits(#3440 공유 2 + 고유 1 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; #3440 적용 뒤 고유 commit 누적 |
| 원 변경 적용 | 공유 `a2167e04`·`a275805e`는 #3440에서 한 번만 적용; 고유 `6c442c20f4838071c78acddb9f75e9ad2795ea3a`→`4ad7b4d52`; devel merge 제외 |
| collaborator 보정 | `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 실제 throw/fallback 회귀 test |
| 관련 이슈 | [#3435](https://github.com/edwardkim/rhwp/issues/3435); 통합 PR에서 `Closes #3435` |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30229707456/job/89867370342) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 원 변경과 의존성

원 PR은 #3440의 `operation() -> null` no-op 계약 위에 `applyThroughRouter`를 만들고, page setup·section·
column·page border·new number·endnote shape 여섯 dialog의 [확인]을 같은 snapshot routing과 실패 계약으로
통일한다. 성공하면 `true`, router 또는 fallback이 throw하면 warning 후 `false`를 반환해 ModalDialog가
입력을 보존한 채 열린 상태를 유지한다. 자체 inline 오류 UI가 있는 formula dialog는 더 강한 기존 계약을
유지해 제외했다.

source branch가 #3440 commit 두 개를 포함한 stacked PR이므로 통합 branch에는 이를 중복 적용하지 않고,
#3440 뒤에 고유 `4ad7b4d52`만 누적했다.

## Collaborator 보정과 browser 검증

원 정적 test는 helper source의 모양을 주로 확인했다. `0b58a0d44`에서 router와 fallback이 실제로 throw하는
runtime test를 추가해 두 경우 모두 `false`와 warning 1회를 반환하는지 고정했다.

Google Chrome `150.0.7871.186`, Node `v24.15.0`, Vite `127.0.0.1:7700`에서 여섯 실제 메뉴·dialog를
열고 [확인]했다.

- `page:col-settings`, `page:section-settings`, `page:page-border`, `file:page-setup`,
  `page:new-page-num`, `insert:endnote-shape`: 각 적용에서 undo command가 정확히 1개 증가했다.
- 6회 적용 뒤 column count 3, 6회 undo 뒤 1로 복원됐다.
- `setPageDef` 실패를 강제 주입하면 dialog가 열린 채 유지되고 undo 0, live snapshot 12가 그대로였다.
- warning은 `[PageSetupDialog] 적용 실패: Error: browser-probe-setPageDef` 한 건, console error는 0이었다.

따라서 성공·undo·실패 입력 보존을 실제 UI surface에서 함께 확인했다.

## 검증

- `dialog-apply-standard` runtime throw/fallback과 관련 undo dialog focused tests: 통과.
- fresh WASM 뒤 TypeScript 검사: 통과.
- 최종 Studio full test: 670 passed / 0 failed.
- Studio production build: 통과.
- 통합 후보 공통 Rust 게이트: release build; release lib 2949/0/7; release-test 전체와 IR sweep 2/2;
  Native Skia 57/0, 2/0, 4/0; fmt·diff check·clippy·doc test; wasm-pack 모두 통과.
- renderer·fixture를 바꾸지 않아 PDF visual sweep과 IR baseline 신규 등록은 대상이 아니다.

## Risk와 최종 권고

현재 실패 UI는 사용자 화면의 inline 오류가 아니라 console warning뿐이라는 UX 한계가 남는다. 그러나 이번
이슈의 계약은 예외를 삼키지 않되 입력과 history를 손상하지 않는 것이며, 성공·실패 양쪽이 실제 browser와
runtime test로 고정됐다. **#3440 의존성과 보정을 포함해 기술적으로 수용 가능**하다. 통합 PR 본문에는
`Closes #3435`를 사용한다. 최종 merge 조건은 최신 통합 head full CI, mergeable 상태와 작업지시자 승인이다.
