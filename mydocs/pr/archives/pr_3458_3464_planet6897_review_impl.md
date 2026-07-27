# planet6897 PR 6건 통합 검토 구현 기록 — 2026-07-27

## 통합 범위

최신 `upstream/devel` `6e9d0821889c3a5bd64da37ff17bcde49e684633` 위의 사용자 가시 primary
worktree branch `review/planet6897-20260727`에 contributor source head를 fetch·누적했다.

| PR | source head | 통합 commit | 판정 |
| --- | --- | --- | --- |
| #3458 | `d8849ba0` | `81d208159`, `f3f2d9bd0` | 수용 가능 |
| #3459 | `6a217340` | `c9d67544f` | #3458과 함께 수용 가능 — r25 모집단 검증 보고 |
| #3461 | `b8d1ab08` | `2d2163526` | 수용 가능 |
| #3462 | `0672b982` | `e3839344b`, `242ec43fd` | 수용 가능 |
| #3463 | `74448394` | `08fdad18a` | 수용 가능 |
| #3464 | `edb7116c` | `e1cc64bbf` | 수용 가능 |

각 source는 `devel`의 오래된 head를 base로 하므로 작성 시점 GitHub 상태는 모두 `MERGEABLE`이나
`BEHIND`다. source branch에는 push하지 않았고 maintainer hold/comment도 없었다.

## 검증 묶음

전용 target `CARGO_TARGET_DIR=target/review-planet6897-20260727`, `CARGO_INCREMENTAL=0`으로
다음 순차 검증을 완료했다.

- Rust release-test 전체: 성공, IR field sweep 2/2 포함.
- Native Skia 공식 3종: 57/0, 2/0, 4/0.
- `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`,
  `cargo check --target wasm32-unknown-unknown --lib`: 성공.
- Studio focused 32/0, 전체 `npm test` 670/0, TypeScript no-emit 및 production build 성공.
- #3461 Chrome headless E2E, #3462 HWP/HWPX 115-step E2E 성공.
- #3463은 Windows `win10-ted` PowerShell에서 source SHA 전용 임시 worktree로 17/0 성공; 기본
  worktree는 변경하지 않았고 임시 worktree·junction도 제거했다.

#3458와 #3464의 기준 PDF/actual review PNG는 각 개별 review에 실제 이미지로 포함했다. #3464는
새 HWPX fixture이나 sweep dump가 existing `tests/fixtures/ir_field_sweep_baseline.tsv`와 완전 동일해
baseline TSV 변경이 없다.

## 통합 PR 범위와 후속 조건

현재 branch의 6개 PR을 하나의 통합 PR 후보로 유지한다. #3459는 #3458 A/C/D의 r25 모집단 검증
보고이므로 분리하지 않는다. `20260728` run label과 source commit/CI 날짜의 차이는 기록의 개선점으로
남기되, #3458의 코드·기준 PDF·focused regression·전체 회귀 검증은 별도로 통과해 merge blocker로
분류하지 않는다.

PR을 만들거나 source PR에 comment/close/merge하기 전에는 작업지시자의 별도 승인을 받고, 그 뒤
통합 head full CI와 mergeable을 재확인한다. 통합 merge 뒤에만 원 PR 연결·상태 후속 처리를 수행한다.
