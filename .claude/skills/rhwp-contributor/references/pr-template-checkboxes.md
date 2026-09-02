# PR 템플릿 — 첫 체크박스 = fmt 게이트

`.github/pull_request_template.md` 의 **테스트** 절 첫 칸이
`cargo fmt --all -- --check` 통과다. 에이전트는 그 칸을 가장 먼저 채운다.

devel 템플릿이 아직 `cargo test` 를 첫 칸에 두고 있어도, 이 스킬은
첫 칸을 fmt 게이트로 해석하고 본문에 명시적으로 적는다.
템플릿을 fmt 첫 칸으로 고치는 열린 PR 이 있으면 그 파일을 가로채지 않는다.

## 본문에 적을 첫 칸

```markdown
## 테스트

- [x] `cargo fmt --all -- --check` 통과 (PR 생성·push 직전 필수. `cargo fmt --check` 만으로는 부족)
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] 관련 `cargo test` 통과 (`agent_contributor_skill_contract` / `test_agent_contributor`)
- [ ] 시각 근거 (렌더/레이아웃이 아니면 N/A)
- [ ] 작업 증빙 `rhwp replay --capsule` (해당 시)
```

첫 칸이 비어 있으면 이 스킬은 PR 을 완주로 보지 않는다.

## 낡은 칸

`- [ ] cargo fmt --check` 를 그대로 복사하지 마라. 명령이 틀렸다.

예제: [17_pr_first_checkbox_fmt.md](../examples/17_pr_first_checkbox_fmt.md).
