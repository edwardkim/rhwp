---
kind: pr-review
pr: 5500
issue: 5497
status: merged
merged_at: 2026-08-18T12:08:25Z
merge_commit: 2f757ad73147e99b0feb4ef7faf5e27750602c32
---

# PR #5500 검증 기록 - 문서 전용 Proptest fast-pass

## 범위

- `devel` 대상 pull request의 파일이 모두 `mydocs/**`일 때만 Proptest preflight가 fast-pass를 선택한다.
- `prop roundtrip` job 이름은 유지하고 worker만 명시적으로 skip하므로 required check가 사라지지 않는다.
- 코드·테스트·workflow 변경, `push`, `workflow_dispatch`, PR 파일 목록 조회 실패는 full Proptest 실행으로 fail-closed 한다.
- 권한은 checkout에 필요한 `contents: read`와 PR 파일 목록 조회에 필요한 `pull-requests: read`로 한정했다.

## 검증 근거

- 로컬: `python -m unittest scripts.tests.test_proptest_roundtrip_workflow scripts.tests.test_workflow_contract_wiring` 11건 통과.
- 로컬: `cargo fmt --all`, `cargo fmt --all -- --check`, `node scripts/rust-test-suite-manifest.mjs --generate`, manifest·unit-tier `--check`, `git diff --check`를 통과했다. 생성 harness는 Git에 포함하지 않았다.
- 원격: workflow 변경 PR인 최신 head `3390f401d57d8d40a9d3d8a02c680dde948593a9`에서는 fast-pass 없이 `Proptest preflight`와 `prop roundtrip`이 각각 성공했다. `prop roundtrip`은 2분 11초에 실제 실행되어 통과했다.
- 원격: Build & Test aggregate, Lint, Native Skia, test archive, regular 1/3·2/3·3/3, slow shard, JavaScript·Python·Rust CodeQL이 모두 성공했다. 변경 영향이 없는 WASM Build·frontend unit gate는 명시적으로 skipped됐다.

## 결론 및 후속 처리

PR은 `2f757ad73147e99b0feb4ef7faf5e27750602c32`로 병합됐다. `Closes #5497`에 따른 이슈 종료 상태와, 이 검토 기록만 포함한 docs-only PR의 최신 head에서 `mydocs-only-devel-pr` 및 `prop roundtrip` skip을 확인한다.
