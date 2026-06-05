# 단계별 완료 보고서 — Task M100-1304 Stage 1~3 (통합)

이슈: [#1304](https://github.com/edwardkim/rhwp/issues/1304) · 브랜치: `local/task1304`

변경 범위가 좁아 Stage 1(테스트)·2(구현)·3(검증)을 한 보고서로 통합한다.

## 1. 구현 내용

### 1.1 토크나이저 (`src/renderer/equation/tokenizer.rs`)

`Token` 에 `space_before: bool` 필드 1개 추가. `Tokenizer` 에 `last_had_space` 보조 상태 추가.
`next_token` 진입 시 `skip_spaces` 전후 pos 델타로 공백 유무를 기록하고, `tokenize()` 루프에서 반환 토큰에 스탬프한다.

- 경계 토큰을 새로 만들지 않으므로 파서의 기존 공백 루프에 영향 없음.
- glue-split 꼬리 토큰(`sinx`→`sin`,`x`)은 contiguous 라 `space_before=false`.

### 1.2 파서 (`src/renderer/equation/parser.rs`)

신규 `parse_script_operand()` + 보조 `is_tight_relational()`.

- `is_tight_relational`: 현재 토큰이 **공백 없는 관계연산자**(`= < > <= >= != == ->`)인지.
- `parse_script_operand`: `원자 (공백없는 관계연산자 원자)*` 로 무브레이스 하한 operand 를 묶는다. `->` 는 `→` 로 변환. 브레이스(`{...}`)는 첫 atom 의 `parse_single_or_group` 이 그대로 처리.

적용(아래첨자/하한만):

| 위치 | 변경 |
|------|------|
| `try_parse_scripts` sub | `parse_single_or_group` → `parse_script_operand` |
| `parse_big_op` sub | `parse_single_or_group` → `parse_script_operand` |
| `parse_limit` sub | `parse_single_or_group` → `parse_script_operand` |

위첨자(`^`)·명령 body 는 무변경 → `x^2=4` 류 위첨자 등식 미접촉.

### 1.3 테스트 (`parser.rs` 테스트 모듈, 6개 추가)

- `task1304_unbraced_sum_lower_limit_full`: `sum_k=1 ^6` → ∑ 하한 `Row[k,=,1]`, 상한 `6`.
- `task1304_unbraced_lim_lower_limit_full`: `lim_x->0` → 하한 `Row[x,→,0]`.
- `task1304_braced_sum_unchanged`: 브레이스 `sum _{k=1} ^{6}` 정상 유지.
- `task1304_spaced_equation_not_merged`: `x^2 = 4` 위첨자에 `=4` 미흡수.
- `task1304_adjacent_identifier_not_merged`: `a_n b` 하한 = `n`.
- `task1304_arithmetic_subscript_not_merged`: `a_n+1` 하한 = `n`.

## 2. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib equation::parser` | 74 passed (신규 6 포함) |
| `cargo test --lib equation` | 145 passed |
| `cargo test --lib` | 1582 passed, 0 failed |
| `cargo test --tests` | 0 failed (115 묶음 ok) |
| `cargo fmt --check` (변경 파일) | clean |
| `cargo clippy --lib` (equation) | 경고 없음 |

## 3. 시각 검증

`rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 10` (문18 해설):

- 4줄의 모든 ∑ 가 **아래 `k=1`(하한 전체) / 위 `6`·`5`(상한)** 로 정상 배치.
- 권위 PDF `pdf/3-10월_교육_통합_2022.pdf` 11쪽과 상·하한 일치.
- `7^2` 등 숫자 위첨자도 정상.

## 4. 잔존 사항 (본 task 비범위)

- `(k+1)^2` 의 지수 `2` 가 위첨자가 아니라 낮게 표시됨 → **괄호 그룹 뒤 위첨자 orphan(base 빈 Superscript)** 사전 결함. 브레이스 표기에서도 동일 발생하며 본 이슈와 별개. `7^2`(숫자 base)는 정상이므로 괄호 한정 문제로 확인. **후속 이슈로 분리**한다.

## 5. 결론

사용자 신고(미주 시그마 상·하한 미표시)의 근본 원인(무브레이스 첨자 공백 구분 미지원)을 해소했다. 전 테스트 통과, 회귀 없음.
