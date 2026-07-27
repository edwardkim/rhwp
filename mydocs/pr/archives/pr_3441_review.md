# PR #3441 검토 기록 — 중복 refresh·event 래핑 제거

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3441](https://github.com/edwardkim/rhwp/pull/3441) — `Task #3436: [undo] 스냅샷 라우팅과 중복되는 리프레시·emit 한 겹 제거` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `d3c30c41dfb64592c0c4f6ee9f4bdd7b85893508` (`fix/2370-duplicate-refresh`) |
| 원 변경 규모 | 4 files, +75 / -8, 2 commits(기능 1 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `b7391102b4c684b7f4678ab19030be9cb531bf9b`→`c222606b4075d89302edeeb26d4d90e00fc2a0dd`; devel merge 제외 |
| collaborator 보정 | 기능 보정 없음; 통합 후보에서 실제 browser·full gate 재검증 |
| 관련 이슈 | [#3436](https://github.com/edwardkim/rhwp/issues/3436); 통합 PR에서 `Closes #3436` |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30229196163/job/89866131157) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 변경 범위와 판정

snapshot `executeOperation`은 기본 refresh에서 `afterEdit()`을 호출하고, 그 경로가
`document-mutated`와 `document-changed`를 방출한다. 원 PR은 누름틀 insert/edit가 다시 수동으로
`document-mutated`를 emit하던 중복을 제거하고, 셀 숫자 서식 세 명령에서 바깥 `try/catch` 안의
`safeTableOp` 중복 래핑을 제거한다. 라우터가 이미 소유한 부수효과와 오류 처리를 호출부가 되풀이하지 않는
방향은 맞다.

통합 검토에서 contributor runtime 변경을 다시 고칠 차단 결함은 발견하지 않았다. 대신 정적 가드뿐 아니라
실제 메뉴·dialog 경로에서 event 수와 undo/redo를 확인했다.

## 실제 browser 검증

Google Chrome `150.0.7871.186`, Node `v24.15.0`, Vite `127.0.0.1:7700`에서 실제 메뉴를 사용했다.

- `insert:field`: `document-mutated` 1회. marker/boundary 갱신 때문에 `document-changed`는 총 3회일 수
  있으므로, 중복 회귀 판정은 mutated 1회를 기준으로 했다. 생성된 field의 이름은 `browser_field`, 안내는
  `브라우저 안내`, 진입 상태는 `inField=true`였다.
- `field:edit`: `document-mutated` 1회, `document-changed` 1회. 안내·memo·editable 변경이 반영됐고
  누적 history는 undo 2, redo 0이었다.
- `table:thousand-sep`: `1234567`→`1,234,567`; undo→`1234567`; redo→`1,234,567`.
- 이어서 `table:decimal-add`: `1,234,567.0`.
- console error: 0.

따라서 원 PR의 핵심인 수동 `document-mutated` 중복 제거와 숫자 서식의 외부 오류 래퍼 유지가 실제 UI에서
모두 성립한다. field insert의 `document-changed` 3회를 "모든 event 1회"로 잘못 일반화하지 않는다.

## 검증

- `undo-no-duplicate-refresh`, field, cell-number-format focused tests 포함 Node focused 30 tests: 통과.
- fresh WASM 뒤 TypeScript 검사: 통과.
- 최종 Studio full test: 670 passed / 0 failed.
- Studio production build: 통과.
- 통합 후보 공통 Rust 게이트: release build; release lib 2949/0/7; release-test 전체와 IR sweep 2/2;
  Native Skia 57/0, 2/0, 4/0; fmt·diff check·clippy·doc test; wasm-pack 모두 통과.
- renderer·fixture·golden을 바꾸지 않아 visual sweep과 IR baseline 신규 등록은 대상이 아니다.

## Risk와 최종 권고

수동 emit을 지울 때 dirty/autosave 신호까지 잃는 것이 주 위험이다. `afterEdit` 전제를 test로 고정했고 실제
browser에서 mutated·changed, undo/redo와 숫자 문자열을 확인했다. **원 변경을 기술적으로 수용 가능**하다.
통합 PR 본문에는 `Closes #3436`을 사용한다. 최종 merge 조건은 최신 통합 head full CI, mergeable 상태와
작업지시자 승인이다.
