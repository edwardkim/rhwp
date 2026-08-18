# HARD GATE — `cargo fmt --all -- --check`

PR 을 열기 전에 이 명령이 종료 코드 0 이어야 한다. 실패하면
`gh pr create` 하지 않는다.

## 정본 명령

```bash
cargo fmt --all -- --check
```

- `--all` : 워크스페이스 멤버 전부
- `--` : 이후는 rustfmt 플래그
- `--check` : 파일을 고치지 않고 위반만 보고

로컬에서 고칠 때는 검사가 아니라 적용을 쓴다.

```bash
cargo fmt --all
cargo fmt --all -- --check
```

## 낡은 표기

`cargo fmt --check` 는 **부족하다.** 예전 문서·에이전트 메모에 남아 있다.
CI Lint 의 Format check 는 `cargo fmt --all -- --check` 다.
이 스킬은 낡은 표기를 게이트로 인정하지 않는다.

`local_validation.md` 일부 예시에 아직 `cargo fmt --check` 가 보이면
정본은 `CONTRIBUTING.md` 의 `cargo fmt --all -- --check` 와 이 절이다.

## crates/ 가 있을 때

스파스 체크아웃을 풀어 `crates/` 가 워크트리에 있으면 fmt 는
그 트리 기준으로 **반드시** 통과해야 한다. 빠져 있으면
[exceptions.md](exceptions.md) 스파스 경로를 먼저 닫는다.

## PR 템플릿

PR 템플릿 **첫 체크박스** 가 이 게이트다.
[pr-template-checkboxes.md](pr-template-checkboxes.md).

## 닫는 증거

명령 종료 코드 0. PR 본문 첫 칸을 `[x]` 로 표시.

예제: [09_fmt_all_check.md](../examples/09_fmt_all_check.md),
[10_fmt_stale_check_rejected.md](../examples/10_fmt_stale_check_rejected.md).
