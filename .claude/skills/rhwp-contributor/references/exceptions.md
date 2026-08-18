# 예외 경로

정상 8단을 닫을 수 없을 때의 분기. 예외는 "건너뛰기"가 아니라
**다른 닫는 증거**다.

## 1. 스파스 체크아웃에 `crates/` 없음

증상: `cargo fmt --all -- --check` 가 워크스페이스 멤버를 못 찾거나
부분만 검사한다. `Test-Path crates` 가 false.

닫는 법:

```bash
git sparse-checkout add crates
# crates/ 가 생긴 뒤
cargo fmt --all -- --check
```

`crates/` 가 있으면 이 명령은 반드시 통과해야 한다. 빠진 채로
"mydocs 만이라 fmt 생략"이라고 쓰지 마라. 이 스킬 파동은 Rust
계약 시험이 있어 fmt 생략 대상이 아니다.

픽스처: `fixtures/layouts/sparse-missing-crates/`.
예제: [18_sparse_missing_crates.md](../examples/18_sparse_missing_crates.md).

## 2. Windows autocrlf vs rustfmt Unix

증상: rustfmt 가 줄끝만 다르다고 실패. `core.autocrlf=true`.

닫는 법: [rustfmt-unix.md](rustfmt-unix.md). 로컬 `core.autocrlf=false`,
생성기 `newline="\n"`, 다시 `cargo fmt --all -- --check`.

예제: [19_windows_autocrlf_unix.md](../examples/19_windows_autocrlf_unix.md).

## 3. 같은 주제의 열린 PR

증상: `gh pr list --search "<키워드>" --state open` 에 이미 head 가 있다.

닫는 법: 새 PR 을 만들지 않는다. 그 PR 에 리뷰로 합류하거나, 이슈에
중복이라고 적고 멈춘다. 이 스킬 고도화는 `#5322` 전용 브랜치
`feat/agent-contributor` 가 없을 때만 연다.

예제: [02_duplicate_open_pr.md](../examples/02_duplicate_open_pr.md).

## 4. CI noci vs FAILURE

두 가지는 다르다.

| 상태 | 뜻 | 에이전트 행동 |
|------|-----|----------------|
| `noci` | 문서 전용 등으로 workflow 가 안 뜨거나 required check 가 안 생긴다 (`paths-ignore`, docs-only) | 없는 검사를 빨간 줄로 보고하지 않는다. push 는 이미 나갔다 |
| `FAILURE` | 실제로 뜬 검사가 빨갛다 (Lint fmt, Build & Test, clippy) | PR 을 완주로 부르지 않는다. 로그를 읽고 고친다 |

문서만 바꾼 `devel` push 에 "Build & Test is expected" 가 보여도
문서 전용 예외다 (`mydocs/manual/memory/feedback_docs_only_ci_exempt.md`).
소스·시험이 섞이면 예외가 아니다.

이 스킬 파동은 `.claude/skills/` 와 `tests/cases/` 가 있어 noci 대상이
아니다. Lint 가 뜨면 fmt 게이트가 그 검사다.

예제: [20_ci_noci_vs_failure.md](../examples/20_ci_noci_vs_failure.md).

## 5. 워크트리 이름 충돌

증상: 쓰려던 경로가 `git worktree list` 에 있다.

닫는 법: 그 경로를 checkout 하지 않는다. 접미를 바꿔 새 경로를 만든다.
금지 경로는 접미만 바꿔도 안 되는 것이 있다 (`rhwp-desk*` 전체).

예제: [06_never_steal_named_worktree.md](../examples/06_never_steal_named_worktree.md).
