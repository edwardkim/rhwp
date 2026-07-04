# PR #1777 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1777
- reviewer assign: `@jangster77`
- PR head 에 update-branch merge commit 이 포함되어 있어 실제 변경 커밋 `b9168064db58e49edef92d456a9803e708972fe6`만 적용했다.

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick b9168064db58e49edef92d456a9803e708972fe6
```

충돌 없음.

## Stage 3. 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

## Stage 4. 판단

테스트 코드만 수정하며, Windows path separator 차이만 비교 단계에서 흡수한다. merge 후보.
