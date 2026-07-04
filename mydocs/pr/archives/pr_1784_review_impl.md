# PR #1784 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1784
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `8fe1d667d`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 8fe1d667d
```

## Stage 3. 변경 내용 검토

완료.

- HWPX parser의 `outMargin` → `common.margin` 동기화 확인.
- 회귀 테스트가 파싱 모델 값을 직접 검증하는지 확인.
- #1788과 같은 샘플/visual target을 공유해 후속 padding 규칙과 함께 검증.

## Stage 4. 검증

완료.

- visual sweep: 1/1쪽, 자동 후보 0
- `cargo test --test issue_1772_table_outer_margin_sync`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
