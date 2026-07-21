# PR #26xx: Rust 의존성 18건 일괄 semver 업데이트

## 이슈
- **Issue**: #2604 — Rust 의존성 18건 semver 호환 업데이트

## 변경
`cargo update -p`로 24개 패키지 업데이트 (18개 직접 + 6개 종속):
serde_json, regex+automata+syntax, toml+toml_writer, memchr, quote, proc-macro2, log, libc, once_cell, cc(+shlex), indexmap+hashbrown, smallvec, unicode-ident, pin-project-lite, arrayvec, either, siphasher, zerocopy+zerocopy-derive

## 결과
- Cargo.lock만 변경 (Cargo.toml 수정 없음)
- `cargo build` / `cargo test` 통과

Closes #2604
