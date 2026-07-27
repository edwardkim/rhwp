# PR #3442 검토 기록 — undo 소스 가드 구간 판정 보강

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3442](https://github.com/edwardkim/rhwp/pull/3442) — `Task #3437: [test] 소스 가드가 우회·리팩터에 무너지지 않게 보강` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `e4d8a6700b0e621ba519c67dbcc828acb2ac583c` (`test/2370-guard-robustness`) |
| 원 변경 규모 | 6 files, +179 / -17, 3 commits(test 2 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `dc89141598f664aa550bf9ca4bbc4bfa591a51d7`→`7c99b235a`; `3065d201c6edcc9f42bc8c11544f936759043533`→`7e628bf2d`; devel merge 제외 |
| collaborator 보정 | `037f4b47ac43f3718c8a97a9ccadf2785734f7a1`, `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 source-guard 범위 |
| 관련 이슈 | [#3437](https://github.com/edwardkim/rhwp/issues/3437); 통합 PR에서 `Closes #3437` |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30229247457/job/89866211330) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 원 변경 범위와 판정

원 PR은 undo 라우팅의 정적 source guard가 별칭, 고정 길이 slice, 주석 문자열과 인자 존재 여부만 보던
취약성을 줄인다. 문자열·주석을 건너뛰며 괄호를 맞추는 `balancedFrom`, 전체 호출을 찾는 `callsOf`, 특정
범위 밖 match를 찾는 `matchesOutside`를 공용 helper로 만들고 다음 계약을 보강한다.

- 개체 mutator 호출은 모두 `recordObjectMutation` 안에 있어야 하며 `services.wasm` 별칭으로 우회할 수 없다.
- 회전·대칭과 formula commit 검사는 고정 문자 창이 아니라 balanced block 전체를 본다.
- `SnapshotCommand('deleteSelection', end, start, ...)`의 cursor 인자 순서와 undo 계약을 함께 고정한다.
- nested-cell merge mock은 실제 JSON 문자열 반환 계약을 사용한다.

이 방향은 테스트가 존재하면서도 실제 회귀를 통과시키는 false confidence를 줄인다. 다만 source의 두 번째
commit에 로컬 절대 경로 `rhwp-studio/node_modules` symlink가 들어와 `037f4b47a`에서 제거했다.

## Collaborator 보정

`0b58a0d44`에서 두 가지 파싱 취약성을 더 고쳤다.

- `functionBodyFrom`은 함수명 뒤 첫 `{`를 바로 찾지 않고, optional parameter type처럼 `{}`가 들어갈 수
  있는 매개변수 목록의 짝을 먼저 찾은 뒤 실제 body block을 추출한다.
- #3440의 `changeZOrder`와 `recordObjectMutation` 가드를 고정 길이 slice가 아닌 실제 body·호출 단위로
  바꿨다.

가드는 TypeScript parser의 대체가 아니며 정규식 literal 내부 괄호까지 일반화하지 않는다. 현재 검사 대상
source의 계약을 정확한 구간으로 좁힌 보조 방어선으로만 평가한다.

## 검증

- source-guard와 관련 undo 가드 focused tests: 통과.
- contributor가 제시한 별칭 우회·cursor 인자 뒤집기 음성 사례를 막는 단언이 유지됨을 확인했다.
- fresh WASM 뒤 TypeScript 검사: 통과.
- 최종 Studio full test: 670 passed / 0 failed.
- Studio production build: 통과.
- 통합 후보 공통 Rust 게이트: release build; release lib 2949/0/7; release-test 전체와 IR sweep 2/2;
  Native Skia 57/0, 2/0, 4/0; fmt·diff check·clippy·doc test; wasm-pack 모두 통과.
- test-only Studio 변경이며 renderer·fixture·golden을 바꾸지 않아 실제 browser 시나리오와 visual sweep,
  IR baseline 신규 등록은 대상이 아니다.

## Risk와 최종 권고

문자열 기반 가드는 완전한 AST 분석이 아니므로 지원 범위를 과장하지 않는다. 현재 repository 패턴에서는
body·call 경계를 정확히 추출하고 실제 우회 두 종류를 막으며, 로컬 환경 symlink도 제거됐다. **보정 후
기술적으로 수용 가능**하다. 통합 PR 본문에는 `Closes #3437`을 사용한다. 최종 merge 조건은 최신 통합
head full CI, mergeable 상태와 작업지시자 승인이다.
