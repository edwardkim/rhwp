# 단계별 완료 보고서 — Task M100-1305 Stage 1~3 (통합)

이슈: [#1305](https://github.com/edwardkim/rhwp/issues/1305) · 브랜치: `local/task1305` (base `local/task1304`)

## 1. 구현 (`src/renderer/equation/parser.rs`)

- `paren_then_script()`: 현재 LParen 의 매칭 RParen 다음 토큰이 `^`/`_` 인지 (paren depth 추적).
- `parse_paren_group()`: `(...)` 를 `EqNode::Paren{ "(", ")", body }` 로 파싱.
- `parse_element` LParen 분기 교체: trailing-script 가 있으면 `parse_paren_group` + `try_parse_scripts`, 없으면 기존 `Symbol("(")` 유지. RParen/LBracket/RBracket 은 무변경.

`(k+1)^2` → `Superscript{ base: Paren, sup: 2 }` (orphan 제거).

## 2. 테스트 (4개 추가)

- `task1305_paren_superscript_binds_to_group`: `(k+1)^2` base=Paren, sup=2.
- `task1305_paren_subscript_binds_to_group`: `(k+1)_i` base=Paren.
- `task1305_plain_paren_not_grouped`: `(k+1)`, `a(b)` Paren 미생성 (회귀 가드).
- `task1305_regression_number_and_leftright_scripts`: `7^2`, `left(x)right^2`(#1226) 정상.

## 3. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib equation::parser` | 78 passed (신규 4 포함) |
| `cargo test --lib` | 0 failed |
| `cargo test --tests` | 0 failed |
| `cargo fmt --check` / clippy | clean |
| 시각 (문18) | `(k+1)^2`,`(k-1)^2` 지수 정상 상승, 권위 PDF 일치 |

## 4. 결론

괄호 그룹 뒤 첨자가 그룹에 결합되어 지수가 정상 상승한다. 첨자 없는 일반 괄호는 무변경(회귀 0).
#1304(시그마 상·하한) + #1305(괄호 지수)로 문18 해설이 권위 PDF와 완전 정합.
