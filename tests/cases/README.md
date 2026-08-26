# Rust integration test 원본

새 회귀·계약 테스트의 원본 `.rs` 파일은 이 디렉터리에 둔다. 하위 디렉터리도 허용하며
`scripts/rust-test-suite-manifest.mjs`가 재귀적으로 수집해 기존 generated suite에 자동 배정한다.

```bash
node scripts/rust-test-suite-manifest.mjs --generate
node scripts/run-rust-test.mjs <확장자를_뺀_test_source_이름>
```

`tests/generated/`, `tests/suites/manifest.json`, `Cargo.toml`의 generated target 블록은 직접 수정하지
않는다. 기존 `tests/` 최상위 원본은 점진적으로 이동하며, 새 원본부터 이 경로를 사용한다.
