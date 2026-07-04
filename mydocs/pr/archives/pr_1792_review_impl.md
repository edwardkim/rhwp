# PR #1792 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1792
- reviewer assign: `@jangster77`
- 실제 변경 커밋: `7a1a730d5`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 7a1a730d5
```

`tests/fixtures/opengov_snapshot.tsv`는 #1799 샘플과 함께 누적되도록 충돌 해결.

## Stage 3. 변경 내용 검토

완료.

- HWPX serializer hidden slot / fieldEnd / bookmark 순서 보존 확인.
- `roundtrip_control_compare.py` 도구 추가 확인.
- `task1591` unit test로 hoist 방지와 hidden slot char shape 위치 확인.
- visual sweep은 serializer PR 범위 밖 참고 자료로만 분리 기록.

## Stage 4. 검증

완료.

- `cargo test --lib task1591`
- `cargo test --profile release-test --tests`
- `cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
