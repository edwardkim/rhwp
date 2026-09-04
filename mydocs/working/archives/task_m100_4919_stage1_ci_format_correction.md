---
kind: working-note
status: completed
issue: 4919
pr: 4919
stage: 1
last_verified: 2026-08-16
---

# PR #4919 CI formatter 보정

## 원인

GitHub Actions의 `Lint (fmt, clippy, WASM check)`가 `src/lib.rs`의 공개 모듈 선언 순서를
`rustfmt` 정렬 규칙과 다르게 두어 실패했다. 이 실패로 Rust archive와 test shard가 모두
skip되었고, 집계 `Build & Test`도 실패했다.

## 보정

`service` 공개 모듈 선언을 `serializer` 뒤로 옮겨 `cargo fmt --all -- --check`의 요구와
일치시켰다. API, 구현, 테스트 동작은 바꾸지 않는다.

## 검증

- `cargo fmt --all -- --check`
- PR #4919의 최신 contributor head와 maintainer 보정 시작 SHA 일치 확인

## 후속

보정 commit을 contributor의 `feat/service-layer` 브랜치에 push한 뒤 새 Full CI를 확인한다.
