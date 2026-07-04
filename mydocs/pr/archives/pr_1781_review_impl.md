# PR #1781 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1781
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `befc1f280`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick befc1f280
```

## Stage 3. 변경 내용 검토

완료.

- `control.rs`의 중첩 그룹 serializer 경계 보정 확인.
- roundtrip 후 벡터 보존 테스트 확인.
- 기준 PDF `pdf/nested_group_vectors-2024.pdf`로 15쪽 visual sweep 수행.

## Stage 4. 검증

완료.

- visual sweep: 15/15쪽, 자동 후보 0
- `cargo test --test issue_1771_nested_group_roundtrip`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
