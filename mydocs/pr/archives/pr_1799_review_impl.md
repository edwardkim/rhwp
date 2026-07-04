# PR #1799 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1799
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `6669e4aa3`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 6669e4aa3
```

`tests/fixtures/opengov_snapshot.tsv`는 #1792 샘플과 함께 누적되도록 충돌 해결.

## Stage 3. 변경 내용 검토

완료.

- fixed table bottom exclusion 예약 경로 확인.
- typeset/table layout의 page-bottom 처리 변경 확인.
- 두 기준 PDF로 visual sweep 수행.

## Stage 4. 검증

완료.

- visual sweep: 1/1쪽 + 1/1쪽, 자동 후보 0
- `cargo test --test issue_1658_page_bottom_fixed_exclusion`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
