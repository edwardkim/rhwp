# Stage 2 완료보고서 — Task #1343

## 구현 내용

### `src/renderer/equation/symbols.rs`
- `EQUATION_PUA: LazyLock<HashMap<char, &'static str>>` 추가 — `U+E04D → "|"`
- `pub fn lookup_equation_pua(ch: char) -> Option<&'static str>` 추가

### `src/renderer/equation/tokenizer.rs`
- `next_token()`의 non-ASCII `Text` 분기 직전에 PUA 조회 삽입
  - 매핑 성공 시 `Token::new(TokenType::Symbol, sym, start)` 반환
  - 단일 `|`(442행)와 동일 토큰 생성 → 파서/레이아웃 경로 동일
- 단위 테스트 `test_pua_conditional_bar` 추가
  - `rm P LEFT ( it A \u{E04D} B RIGHT )` → Symbol 토큰 `["|"]`
  - 모든 토큰에 PUA 원형 코드포인트 잔존하지 않음 검증

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ Finished |
| `cargo test --lib` | ✅ 1617 passed, 0 failed |
| `cargo clippy --lib` | ✅ 경고 없음 |
| `cargo fmt`(변경 파일) | ✅ 변경분 없음(이미 정렬) |

매핑 미존재 PUA 문자는 기존 `Text` 폴백 유지 — 회귀 없음.
