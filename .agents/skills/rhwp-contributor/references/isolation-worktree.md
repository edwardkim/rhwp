# 격리 워크트리 — named 를 훔치지 않는다

에이전트는 본진 작업트리를 더럽히지 않는다. 기여 1건 = 워크트리 1개.
이미 누가 쓰고 있는 named worktree 를 재사용·강제 checkout 하지 않는다.

## 만드는 법

```bash
git -C <본진> fetch upstream devel
git -C <본진> worktree list
git -C <본진> worktree add -b feat/<주제> <빈경로> upstream/devel
```

`<빈경로>` 는 존재하지 않아야 한다. 존재하면 그 자리를 덮어쓰지 말고
다른 이름을 고른다.

## 금지 경로 (이 환경)

다음 경로는 **절대** 작업 디렉터리로 쓰지 않는다.

| 경로 | 이유 |
|------|------|
| `C:\Users\swsz9\rhwp` | 본진 |
| `C:\Users\swsz9\rhwp-desk` | named, 사용 중 |
| `C:\Users\swsz9\rhwp-desk-*` | named desk 계열 |
| `C:\Users\swsz9\rhwp-handoff` | named |
| `C:\Users\swsz9\rhwp-scaffold-final` | named |
| `C:\Users\swsz9\rhwp-doc-repro` | named |

`git worktree list` 에 이미 올라간 이름도 전부 금지다. 새 이름은
`rhwp-agent-contributor` 처럼 이번 작업 전용이어야 한다.

픽스처: `fixtures/layouts/forbidden-worktrees/registry.json`.

## 스파스 체크아웃

본진이 sparse 이면 새 워크트리도 그 규칙을 물려받을 수 있다.
`crates/` 가 빠지면 `cargo fmt --all` 이 워크스페이스 멤버를 못 찾거나
부분만 검사한다.

```bash
git sparse-checkout add crates
```

`crates/` 가 있으면 `cargo fmt --all -- --check` 는 **반드시** 통과해야 한다.

예외 레시피: [18_sparse_missing_crates.md](../examples/18_sparse_missing_crates.md).

## 닫는 증거

- `git worktree list` 에 새 경로가 있고, 금지 경로가 아니다
- 다른 worktree 의 브랜치를 checkout 하지 않았다

예제: [05_isolation_worktree.md](../examples/05_isolation_worktree.md),
[06_never_steal_named_worktree.md](../examples/06_never_steal_named_worktree.md).
