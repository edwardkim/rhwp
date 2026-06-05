# 최종 결과 보고서 — Task M100-1305: 괄호 그룹 뒤 위첨자 orphan

- 이슈: [#1305](https://github.com/edwardkim/rhwp/issues/1305)
- 브랜치: `local/task1305` (base: `local/task1304`)
- 관련: [#1304](https://github.com/edwardkim/rhwp/issues/1304), [#1226](https://github.com/edwardkim/rhwp/pull/1226)

## 1. 증상

`(k+1)^2` 등 괄호 그룹 뒤 위첨자의 지수 `2` 가 위로 올라가지 못하고 낮게 표시. 브레이스 표기 `(k+1) ^{2}` 에서도 동일. `7^2`(숫자 base)는 정상.

## 2. 근본 원인

리터럴 `(...)` 가 `EqNode::Paren` 그룹이 아니라 느슨한 `Symbol("(")`…`Symbol(")")` 로 파싱되어, `)` 뒤 `^2` 가 결합 base 없이 `Superscript{base:Empty}` orphan 이 됐다(지수 미상승).

## 3. 해결

`(...)` 뒤에 `^`/`_` 가 올 때만 `EqNode::Paren` 그룹으로 묶어 `try_parse_scripts` 로 첨자를 결합. 첨자 없는 괄호는 기존 느슨한 렌더 유지 → 일반 괄호 회귀 0. 라운드 괄호 한정(대괄호는 비범위).

변경 파일: `src/renderer/equation/parser.rs` (`paren_then_script`, `parse_paren_group`, LParen 분기, 테스트 4개).

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` / `--tests` | 0 failed |
| fmt / clippy | clean |
| 시각 (문18 4줄) | 모든 `(k±1)^2` 지수 정상 상승, 권위 PDF 일치 |

#1304 + #1305 로 `3-10월_교육_통합_2022.hwp` 11쪽 문18 해설 시그마·지수가 권위 PDF(`pdf/3-10월_교육_통합_2022.pdf`)와 완전 정합.

## 5. 비범위

- 대괄호 `[...]^n` (텍스트 높이에서도 path 렌더 → 외형 변화 위험). 필요 시 별도 이슈.

## 6. 결론

괄호 지수 orphan 해소. 회귀 없음.
