---
kind: working
status: active
issue: 6197
stage: 1
---

# #6197 Stage 1 - 보조 CI의 기준선 병합 문서 tail 재사용

## 문제

PR #6188은 이미 성공한 코드 후보 뒤에 문서 기록을 추가하고 최신 `devel`을 병합했다. CI 본체는
source parent의 녹색 `Build & Test`와 현재 기준선 병합 tree를 확인해 fast-pass했지만,
`adapter-diff.yml`과 `proptest-roundtrip.yml`은 병합 commit을 코드 변경으로 분류해 worker를 다시 실행했다.

## 변경

- 두 workflow가 현재 PR base를 부모로 갖는 2-parent merge를 한 개의 기준선 bridge로만 인식한다.
- source 계보의 기존 성공 workflow run을 재사용 후보로 확인하면 preflight는 즉시 skip하지 않고
  `pending-base-merge-tree` 상태를 낸다.
- checkout 뒤 `git merge-tree --write-tree`가 일치하거나, 충돌 해소가
  `scripts/verify_review_only_merge_resolution.py`로 `mydocs/` 한정임을 검증한 경우에만 worker를 skip한다.
- workflow와 runner, suite prepare, adapter/proptest harness 변경은 execution surface로 분류해
  fast-pass를 허용하지 않는다.
- 모의 preflight 회귀 테스트는 #6188과 같은 source candidate + current-base merge 계보가
  `pending-base-merge-tree`가 되는지 고정한다.

## 로컬 검증

2026-08-27에 아래 명령을 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
python3 -m unittest \
  scripts/tests/test_adapter_diff_workflow.py \
  scripts/tests/test_proptest_roundtrip_workflow.py \
  scripts/tests/test_review_only_fast_pass_workflows.py
git diff --check
```

- 결과: Python workflow 계약 테스트 38건 통과.
- `cargo fmt --all -- --check` 및 `git diff --check` 통과.
- 제품 Rust, WASM, Studio 회귀는 workflow 정책 변경의 직접 검증 범위가 아니므로 실행하지 않았다.

## 후속 확인

PR의 최신 head GitHub Actions에서 `adapter inter-diff`와 `prop roundtrip`이
`current-base-update-merge-tree-green` 또는 `current-base-update-merge-resolution-mydocs-only-green`으로
skip되는지 확인한다. 계보, 후보 run, merge tree, 실행 경로 중 하나라도 검증하지 못하면 full worker가
실행되어야 한다.
