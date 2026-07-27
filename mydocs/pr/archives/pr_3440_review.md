# PR #3440 검토 기록 — 무변경 undo 기록·redo 파괴 차단

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3440](https://github.com/edwardkim/rhwp/pull/3440) — `Task #3434: [undo] 무변경 연산을 기록하지 않는 공용 no-op 장치` |
| 작성자·검토자 | `@lpaiu-cs` (external contributor) · `@jangster77` (collaborator) |
| base / source head | `devel` / `afa8e26a86ed981b12c5704ad7cc80c35c7f418c` (`fix/2370-noop-skip`) |
| 원 변경 규모 | 10 files, +270 / -35, 3 commits(기능 2 + devel merge 1) |
| 통합 검토 | `review/lpaiu-cs-20260727`; `upstream/devel` `7779e737ac5c5df3428d1a06f1099be16375be49` 기준 |
| 원 변경 적용 | `a2167e040126bc85c25888564b97d9975e0c439c`→`efe9c6d7b`; `a275805e22e8ec1070c7f83d7b9ea2150deefe87`→`64bac3b20`; devel merge 제외 |
| collaborator 보정 | `011702107f2bb9b7c744c29536264e223801e19e`, `0b58a0d4497d2154b37e797ce49b8eca79357fd2` 중 no-op/source-guard 범위 |
| 관련 이슈 | [#3434](https://github.com/edwardkim/rhwp/issues/3434); 통합 PR에서 `Closes #3434` |
| 작성 시점 source 상태 | `MERGEABLE` / `CLEAN`, draft 아님; source Build & Test [성공](https://github.com/edwardkim/rhwp/actions/runs/30228829327/job/89865253206) |
| 라우팅 | base: `collaborator_external_pr`; modifiers: `intake_and_review`, `local_validation`, `multi_pr_update_branch` |

Loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`.

## 원 변경 범위와 판정

원 PR은 snapshot operation이 `null`을 반환하면 before snapshot을 폐기하고 `isNoOp`로 표시해,
`CommandHistory`가 undo push·redo 파기·예산 강제를 모두 생략하는 공용 계약을 추가한다. 경계 z-order와
값이 바뀌지 않은 미주 모양 [확인]을 이 신호에 연결하고, 개체 선택 해제 뒤 중복 `afterEdit`도 같은 호출부에서
제거한다. 아무 일도 하지 않은 조작이 redo와 snapshot 예산을 소비하지 않아야 한다는 방향은 타당하다.

source의 두 번째 기능 commit에 로컬 절대 경로를 가리키는 `rhwp-studio/node_modules` symlink가 함께
들어왔다. 이는 기능과 무관하고 다른 환경에서 재현 불가능하므로 `011702107`에서 제거했다.
`rhwp-studio/.gitignore`도 repository root의 정확한 `/node_modules`만 제외하도록 보정했다.

`0b58a0d44`에서는 fixed-length slice에 의존하던 no-op 가드를 balanced function-body/call 추출로 바꾸고,
삭제·묶기·풀기 경로가 `DEFER_REFRESH_TO_EXIT`를 실제 같은 호출에 전달하는지 단언하도록 했다. runtime
동작은 contributor 설계를 유지했다.

## 실제 browser 검증

Google Chrome `150.0.7871.186`, Node `v24.15.0`, Vite `127.0.0.1:7700`에서 실제 context menu
`insert:arrange-front`를 사용했다.

- 실제 정렬: z-order `[0,1]`→`[2,1]`, undo 0→1, live snapshot 0→2.
- undo 뒤: z-order `[0,1]`, undo 0, redo 1.
- 이미 경계인 개체에 같은 명령을 두 번 적용: z-order·undo·live snapshot 불변, redo 1 유지.
- 이어서 redo: z-order `[2,1]`로 정상 재적용.
- 실제 메뉴 삭제 뒤 undo에서 모델의 개체와 canvas ink가 모두 복원됐다.
- console error: 0.

caret·selection overlay는 비동기 타이밍이 달라 전체 canvas pixel equality를 판정 기준으로 쓰지 않고,
모델 상태·history 수·개체 ROI 복원을 함께 확인했다.

## 검증

- Studio focused no-op·개체 메뉴·source guard 포함 Node focused 30 tests: 통과.
- fresh WASM 뒤 TypeScript 검사: 통과.
- 최종 Studio full test: 670 passed / 0 failed.
- Studio production build: 통과.
- 통합 후보 공통 Rust 게이트: release build; release lib 2949/0/7; release-test 전체와 IR sweep 2/2;
  Native Skia 57/0, 2/0, 4/0; fmt·diff check·clippy·doc test; wasm-pack 모두 통과.
- renderer/layout/fixture를 바꾸지 않아 별도 PDF visual sweep과 baseline 갱신은 대상이 아니다.

source head CI는 녹색이지만, #3443 등 의존 PR과 보정 test가 누적된 최신 통합 head의 full CI를 최종
게이트로 둔다.

## Risk와 최종 권고

가장 큰 위험은 no-op 판정이 실제 mutation을 무변경으로 오인해 undo를 잃는 것이다. 현재는 z-order의
호출 전·후 값과 미주 설정 전체 키 비교로 범위를 좁혔고, 실제 변경·undo·redo와 경계 no-op을 한 세션에서
모두 확인했다. **불필요한 symlink 제거와 가드 보정 후 기술적으로 수용 가능**하다. 통합 PR 본문에는
`Closes #3434`를 사용한다. 최종 merge 조건은 최신 통합 head full CI, mergeable 상태와 작업지시자 승인이다.
