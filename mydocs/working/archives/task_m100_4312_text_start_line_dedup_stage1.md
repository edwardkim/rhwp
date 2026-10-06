---
kind: working
status: completed
issue: 4312
last_verified: 2026-08-09
---

# Task #4312 Stage 1

## 구현·검증

- `composer::first_text_line(&ComposedParagraph) -> Option<usize>` 신설. `layout.rs:6568`과
  `typeset.rs:13519`(`measure_endnote_para_advance` scratch 경로)의 동일 클로저를 그대로 이동 —
  새 로직 없음. 사이트별 게이트(`has_real_text`, `is_wrap_host`, end_line 계산)는 변경 없이 유지.
  범위 밖(`layout.rs:8458`/`9723` 등 술어가 다른 변형)은 이번 슬라이스에서 건드리지 않았다.
- 검증: `cargo test --profile release-test --lib lineseg_compare`(9/9) +
  `issue_1082_endnote_multicolumn_drift`·`issue_1375_endnote_rewind_column_overflow`
  (sep20/sep2020 fixture 포함, 7/7) → `cargo test --profile release-test --tests` 전체
  (495 바이너리, 5,486 passed, 0 failed) → `cargo fmt --check` → `cargo clippy --all-targets
  -- -D warnings` → Native Skia 3종(58+2+4 passed) → `wasm-pack build --target web` 전부 통과.
- 순수 추출(behavior-preserving) — 클로저 본문 무변경, 호출 지점만 통합했으므로 회귀 위험은
  각 사이트의 기존 회귀 스위트(sep20/sep2020 fixture)로 충분히 커버된다.
