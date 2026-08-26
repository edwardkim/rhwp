# 19 — Windows autocrlf vs rustfmt Unix

예외. newline_style=Unix. CRLF 가 들어가면 게이트 실패.

권위: [references/rustfmt-unix.md](../references/rustfmt-unix.md).
픽스처: [fixtures/envelopes/fmt_fail_crlf.json](../fixtures/envelopes/fmt_fail_crlf.json).

## 0. 하지 않는 것

- `cargo fmt --check` 를 게이트로 쓰지 않는다. 정본은 `cargo fmt --all -- --check`.
- `git add -A` 를 쓰지 않는다.
- named worktree 를 훔치지 않는다.
- DocumentCore 편집 로직을 발명하지 않는다.
- 새 rhwp CLI 를 만들지 않는다.
- `gym/` 과 `rhwp-work-receipt` 본문을 고치지 않는다.

## 1. 절차

1. `rustfmt.toml` 의 `newline_style = Unix`.
2. `core.autocrlf=true` 면 로컬에서 false 로 끈다.
3. 생성기는 `newline="\n"` 으로 저장한다.

## 2. 명령

```bash
git config --local core.autocrlf false
git config --local core.eol lf
cargo fmt --all -- --check
```

## 3. 체크리스트

- [ ] autocrlf 가 false 다
- [ ] HARD GATE 가 줄끝 때문에 실패하지 않는다

## 4. 실측 메모

- 이슈는 #5322 (`agent: 기여 절차(contributor) 스킬 고도화`).
- 브랜치는 `feat/agent-contributor`, remotes 는 origin=kevin9327 fork, upstream=edwardkim/rhwp.
- HARD GATE 실패 시 `gh pr create` 를 호출하지 않는다.
- `crates/` 가 이 워크트리에 있으면 `cargo fmt --all -- --check` 는 반드시 0 이어야 한다.
- rustfmt `newline_style=Unix`. Windows 에서 CRLF 가 섞이면 게이트가 실패한다.
- 작업 영수증이 필요하면 `.claude/skills/rhwp-work-receipt/` 를 읽고
  `rhwp replay --capsule` / `rhwp audit` / `rhwp lineage` 만 호출한다.
- 이 레시피는 gym 과제가 아니다. Maker seat 로 실 PR 을 닫는다.

## 5. 다음

레시피 색인 [references/recipe-index.md](../references/recipe-index.md).
전체 순서는 [24_full_procedure_walkthrough.md](24_full_procedure_walkthrough.md).
이슈 #5322. 브랜치 `feat/agent-contributor`.
