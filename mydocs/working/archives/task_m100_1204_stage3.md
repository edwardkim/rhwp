# Stage 3 완료보고서 — Task M100 #1204

**단계**: 회귀 테스트
**브랜치**: `local/task1204`

## 추가 테스트

`tokenizer.rs`:
- `test_root_sqrt_prefix_split_on_digit` — root3/sqrt5 분리, rootn(letter) 비분리, `root {4} of {x}` 유지.
- `test_prime_prefix_split` — primeF 분리, `f prime` 유지.

`parser.rs`:
- `test_font_style_body_decoration_not_leaked` — `rm bar {F prime F}` → Decoration 포함, `Text("bar")` 없음.
- `test_root_glued_digit_parses_as_sqrt` — `root3 y` → Sqrt 포함, `Text("root3")` 없음.

## 결과

- 수식 모듈 테스트 통과(신규 4건 포함), 회귀 0.
- 전체 스위트 `cargo test --release` 통과.
