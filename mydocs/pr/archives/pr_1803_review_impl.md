# PR #1803 리뷰 구현 메모

## Stage 1. 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1803
- reviewer assign: `@jangster77`
- 실제 커밋: `9813f5e1d23ee524c54f1a6d55583d1d94ebb2f7`

## Stage 2. 로컬 적용

완료.

```bash
git cherry-pick 9813f5e1d23ee524c54f1a6d55583d1d94ebb2f7
```

`mydocs/orders/20260702.md` 자동 병합. 충돌 없음.

## Stage 3. 코드 검토

완료.

- `Bullet` 모델에 `char_shape_id` 추가.
- `parse_bullet` 가 `char_shape_id`를 버리지 않고 보존.
- `serialize_bullet` 이 12바이트 문단 머리 정보를 맞춤.
- NBSP 직렬화 코드를 0x18 에서 0x1E 로 정정.
- layout/roundtrip 및 NBSP unit test 추가.

## Stage 4. 검증

완료.

- `git diff --check upstream/devel..HEAD`
- `cargo fmt --check`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`

## Stage 5. 판단

merge 후보.
