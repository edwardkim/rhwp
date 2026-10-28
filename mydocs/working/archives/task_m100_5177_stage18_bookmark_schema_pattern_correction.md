# Stage 18: 책갈피 MCP schema pattern 보정

## 목적

PR #5185의 regular shard 1에서 실패한 `add_bookmark_contract::mcp_declared`의 정규식 기대값을 MCP schema의 실제 JSON 값과 일치시킨다.

## 원인

`capabilities --mcp`가 반환한 JSON의 `hwp_add_bookmark.name.pattern`은 `.*\\S.*` JSON 표기로 전달되며, `serde_json::Value`로 역직렬화한 문자열은 역슬래시 하나를 포함한 `.*\S.*`다. 테스트가 역슬래시 둘을 포함한 Rust raw string을 기대해 실제 schema와 불일치했다.

## 변경 계약

- `mcp_declared`는 역직렬화 후의 `.*\S.*` pattern을 기대한다.
- 빈 문자열과 공백 전용 책갈피 이름을 거부하는 MCP schema의 의미는 변하지 않는다.

## 검증 결과

- `node scripts/rust-test-suite-manifest.mjs --prepare`: 파생 suite 준비 성공.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review-5185-stage18 node scripts/run-rust-test.mjs --cargo-test add_bookmark_contract -- --profile release-test`: 3건 통과.
