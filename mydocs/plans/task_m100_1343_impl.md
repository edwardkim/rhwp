# 구현계획서 — Task #1343: 수식 조건부 막대 U+E04D(PUA) 두부 렌더 수정

- **이슈**: #1343 (M100)
- **브랜치**: `local/task1343`

## 단계 구성 (3단계)

### Stage 1 — PUA 매핑 구현

`src/renderer/equation/symbols.rs`:
- `EQUATION_PUA: LazyLock<HashMap<char, &'static str>>` 추가
  - `U+E04D → "|"` (조건부 막대)
- `pub fn lookup_equation_pua(ch: char) -> Option<&'static str>` 추가

`src/renderer/equation/tokenizer.rs`:
- `next_token()`의 non-ASCII `Text` 분기 직전에 PUA 조회 삽입
  - 매핑 성공 시 `Token::new(TokenType::Symbol, sym, start)` 반환
  - 단일 `|` 기호와 동일한 토큰을 생성하므로 파서/레이아웃 경로 동일

산출물: 소스 2파일 변경.

### Stage 2 — 빌드·테스트·단위검증

- `cargo build` / `cargo test` 통과
- 토크나이저 단위 테스트 추가: `U+E04D` 입력 → `Symbol "|"` 토큰 검증
- `cargo fmt`(변경 파일 한정), `cargo clippy` 경고 없음

산출물: `_stage2.md` 완료보고서.

### Stage 3 — 시각 검증·최종 보고

- `rhwp export-svg`로 16페이지 출력 → `P(A|B)` 막대 정상(`|`) 렌더 확인
- 두부(▦) 소거 및 기존 `|` 렌더와 동일성 확인
- `pdf/` 권위 자료와 시각 정합 비교
- `_report.md` 최종 보고서 작성

산출물: `_stage3.md`, `_report.md`.

## 커밋 전략

- Stage 1 소스 + 단위테스트 → `Task #1343: 수식 PUA 조건부 막대(U+E04D) → '|' 매핑`
- 단계별/최종 보고서는 해당 단계 커밋에 동봉
