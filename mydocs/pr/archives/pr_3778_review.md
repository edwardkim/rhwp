---
kind: review
status: ci-passed
canonical: mydocs/pr/archives/pr_3778_review.md
last_verified: 2026-08-02
---

# PR #3778 검토 기록 — batch stdin 조기 종료 레이스

## 결론

[PR #3778](https://github.com/edwardkim/rhwp/pull/3778)는 batch 인자 검증 거부 경로에서 자식
프로세스가 stdin을 읽기 전에 정상 종료할 때 발생하는 `BrokenPipe` 레이스를 테스트 헬퍼에서만
처리한다. 프로덕션 코드와 CLI 계약은 바꾸지 않는다.

**권고: merge.** review 문서와 오늘할일을 포함한 첫 fast-pass head가 최신 CI를 통과했고, 차단
결함은 발견하지 못했다. `@enigma-jerry72`는 병합된 선행 PR이 없는 첫 기여자이며, #3771과 #3778이
첫 기여 묶음이다. merge 뒤 실제 검증 결과와 함께 감사의 뜻을 PR에 남긴다.

## PR metadata와 라우팅

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#3778](https://github.com/edwardkim/rhwp/pull/3778) / `@enigma-jerry72` |
| base / source head | `devel` / `enigma-jerry72/rhwp:fix/batch-stdin-brokenpipe-flake` |
| contributor code head | `639d6bd921066075f2871cbdf7fd9e9a8173cc1f` |
| 변경 규모 | +23 / -12, `tests/batch_axes_contract.rs` 1개 파일, 1 commit |
| 첫 review-only head 상태 | `7f92ecc55`, `MERGEABLE`, `CLEAN`, fast-pass CI 통과 |
| reviewer | `@edwardkim` 요청 |

base route: `collaborator_external_pr.md`

modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
`post_merge.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`collaborator_external_pr.md`, `intake_and_review.md`, `local_validation.md`,
`review_only_fast_pass.md`, `post_merge.md`

current head: `7f92ecc55ba44f4e3fcb8170cd8dc4000b89625d` (후속 문서 기록 commit 전 참고값; merge 전 재확인 필요)

변경은 renderer, layout, HWP/HWPX fixture, golden, baseline을 포함하지 않는다. 따라서 시각·fixture
증적 경로는 적용하지 않는다.

## 원인과 변경 검토

`batch_axes_contract`의 인자 오류 테스트는 자식 `rhwp`가 stdin을 소비하기 전에 exit 2로 종료하는
것이 정상 계약이다. 종전의 두 헬퍼는 `write_all(...).expect(...)`를 사용해, 자식 종료가 먼저
일어나면 테스트 자체가 `BrokenPipe` panic으로 실패했다.

추가한 `write_stdin_ignoring_early_exit()`는 두 헬퍼가 같은 정책을 사용하게 한다. `ErrorKind::BrokenPipe`
한 종류만 정상으로 허용하고, 다른 stdin I/O 오류는 이전처럼 assertion 실패로 유지한다. 이후에도 각
테스트는 종료 코드와 stdout/stderr를 검증하므로, 입력을 소비하지 않은 정상 인자 거부와 실제 명령
오류를 구분하는 계약은 바뀌지 않는다.

이 결함은 [PR #3785 CI run 30741430161](https://github.com/edwardkim/rhwp/actions/runs/30741430161)의
`Default-feature tests (shard 6/8)`에서 `batch_convert_rejects_flag_as_out_dir_before_any_write`가
`BrokenPipe`로 panic하며 실제로 재현됐다. #3785의 Studio 단축키 변경과는 독립적인 기존 CI 레이스다.

## 검증

| 게이트 | 결과 |
| --- | --- |
| 최신 `devel` 기준 | `639d6bd…`는 `cc3829116…`의 직접 자식, merge conflict 없음 |
| `cargo test --profile release-test --test batch_axes_contract` | 17 passed / 0 failed |
| 실패했던 정확한 테스트 반복 | 컴파일된 release-test 바이너리 100회 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| contributor code head GitHub CI | [CI 30730240090](https://github.com/edwardkim/rhwp/actions/runs/30730240090) 전체 통과; default-feature 8 shards 및 `Build & Test` success |

반복 실행은 현재 Linux 검토 환경에서의 회귀 탐지 근거이며, 모든 프로세스 스케줄을 증명하는 것은 아니다.
그러나 변경은 OS 오류 종류를 좁게 제한하고 기존 CI 실패 원인과 정확히 일치한다.

## Push, CI, merge 후속 처리

이번 commit은 `mydocs/`의 review 문서와 오늘할일만 포함하는 single-parent trailing commit이다.
따라서 contributor code head의 green `Build & Test`를 candidate로 하는 review-only fast-pass A 경로를
적용한다. push 직전 contributor source SHA, fork remote ref, local HEAD를 대조하고 LFS 추적 대상 여부를
판독한다. Markdown-only인 것이 확인되면 `GIT_LFS_SKIP_PUSH=1`로 fork source ref에 dry-run과 실제
push를 수행한다.

fork source ref의 사전 SHA 대조과 LFS 판독 뒤 `7f92ecc55`를 push했다. 해당 head의
[CI 30742380062](https://github.com/edwardkim/rhwp/actions/runs/30742380062)에서 CI preflight와
`Build & Test` aggregate가 success였고, heavy job은 review-only fast-pass로 정상 skipped됐다.
이후 문서 상태를 확정하는 commit도 같은 허용 경로의 single-parent trailing commit이므로, push 뒤
최신 head의 aggregate를 다시 확인한다.

merge 뒤에는 `devel` 반영과 merge SHA를 확인하고, contributor PR에 CI·로컬 검증 결과와 첫 기여에
대한 감사 문구를 실제 줄바꿈 body로 남긴다. #3778에는 close 키워드가 없으므로 별도 issue close는
수행하지 않는다.
