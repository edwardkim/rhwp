# 03 — 정본을 읽고 이슈에 기록

단 2. AGENTS.md 로딩 순서와 local_validation.md §4.3 을 인용한다.

권위: [references/analyze-canonical.md](../references/analyze-canonical.md).
픽스처: [fixtures/checklists/step_02_analyze.json](../fixtures/checklists/step_02_analyze.json).

## 0. 하지 않는 것

- `cargo fmt --check` 를 게이트로 쓰지 않는다. 정본은 `cargo fmt --all -- --check`.
- `git add -A` 를 쓰지 않는다.
- named worktree 를 훔치지 않는다.
- DocumentCore 편집 로직을 발명하지 않는다.
- 새 rhwp CLI 를 만들지 않는다.
- `gym/` 과 `rhwp-work-receipt` 본문을 고치지 않는다.

## 1. 절차

1. CONTRIBUTING.md 의 `cargo fmt --all -- --check` 를 확인한다.
2. 형제 스킬 계약 시험 패턴을 읽는다.
3. DocumentCore 편집 로직을 설계하지 않는다고 이슈에 적는다.

## 2. 명령

```bash
gh issue view 5322 --repo edwardkim/rhwp
```

## 3. 체크리스트

- [ ] 정본 경로가 이슈에 있다
- [ ] 검증 계획에 HARD GATE 가 있다
- [ ] 비범위에 gym · 새 CLI · DocumentCore 가 있다

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
