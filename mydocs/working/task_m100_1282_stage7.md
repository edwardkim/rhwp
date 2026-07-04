# Task 1282 Stage 7 - PR clippy 보정

## 목적

PR 준비 필수 검증인 `cargo clippy --all-targets -- -D warnings`에서 기존 회귀 테스트의 문자열 검색 표현이
`clippy::search_is_some` 경고로 실패했다.

## 수정

- `tests/issue_1139_inline_picture_duplicate.rs`
  - `find(...).is_some()`을 같은 의미의 `contains(...)`로 변경한다.

## 검증

- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `git diff --check`

## 판단

Task #1282의 기능 변경은 아니며, PR 필수 검증을 통과시키기 위한 기존 테스트 표현 정리다.
