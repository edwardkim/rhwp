# PR #4747 리뷰 — WASM32 Clippy gate 정합

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
current head: 518b0d7d5 (작성 시점 참고값)
```

PR [#4747](https://github.com/edwardkim/rhwp/pull/4747)은 collaborator `jangster77`의 `devel` 대상
PR이며 #4630을 참조한다. 작업지시자의 self-review 지시에 따라 외부 reviewer를 요청하지 않았다.

WASM32 Canvas 경로의 Clippy 16건을 동작 보존 형태로 정리하고 CI Lint에 같은 `--lib` scope의
WASM32 Clippy gate를 추가한다. #4631 CLI target 설계, #4089 Docker 구성, fixture와 renderer output은
변경하지 않으므로 시각 증적 경로는 적용하지 않는다.

- PowerShell `target\\pr-review`에서 `cargo clippy -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings`는 보정 후 성공했다.
- `python scripts\\tests\\test_ci_impact_workflow.py`는 27/27 통과했고 `git diff --check`도 통과했다.

**권고: 보류.** 최신 PR head의 CI·CodeQL과 trailing review-only 기록의 aggregate 성공, 작업지시자
승인 전에는 merge나 #4630 종료를 하지 않는다.
