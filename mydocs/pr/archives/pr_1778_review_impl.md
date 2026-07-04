# PR #1778 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1778
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `76087c78d`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 76087c78d
```

## Stage 3. 변경 내용 검토

완료.

- 배포용 문서를 HWP 저장 시 일반 문서로 강하하는 경로 확인.
- 저장 후 재로드 가능성을 직접 확인하는 회귀 테스트 확인.
- visual sweep은 PR 목적상 필수 검증이 아니며 Poppler 추출 실패를 blocker로 보지 않음.

## Stage 4. 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1768_distribution_doc_save`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
