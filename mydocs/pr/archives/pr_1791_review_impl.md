# PR #1791 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1791
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `edf0e9a2a`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick edf0e9a2a
```

## Stage 3. 변경 내용 검토

완료.

- exclusion probe에서 line spacing 중복 반영 제거 확인.
- 테스트가 line spacing 제외 계약을 직접 검증하는지 확인.
- #1805와 같은 샘플로 visual sweep 수행.

## Stage 4. 검증

완료.

- visual sweep: 2/2쪽, 자동 후보 0
- `cargo test --test issue_1789_exclusion_probe_line_spacing`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
