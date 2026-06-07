# PR #1306 처리 보고서 - 수식 미주 시그마/괄호 지수 렌더 수정

- PR: https://github.com/edwardkim/rhwp/pull/1306
- 작성일: 2026-06-06
- 브랜치: `local/pr1306-integration`
- 처리 방식: 전체 PR 브랜치 머지 대신 최신 `local/devel` 위 수식 코드 패치 선별 적용

## 1. 처리 요약

PR #1306의 본질 코드 변경만 최신 `local/devel` 위에 적용했다.

전체 PR 브랜치는 base가 오래되어 최근 `devel` 변경을 되돌릴 위험이 있으므로 사용하지 않았다. 실제 적용 대상은 다음 5개 파일이다.

```text
src/renderer/equation/canvas_render.rs
src/renderer/equation/layout.rs
src/renderer/equation/parser.rs
src/renderer/equation/svg_render.rs
src/renderer/equation/tokenizer.rs
```

## 2. 반영 내용

### #1304 미주 시그마 무브레이스 하한

- `Token.space_before`를 추가해 일반 공백을 시각 토큰으로 만들지 않고 operand 경계 정보로만 보존했다.
- 무브레이스 아래첨자/하한 전용 `parse_script_operand`를 추가했다.
- `sum_k=1 ^6`이 `∑` 하한 `k=1`, 상한 `6`으로 파싱되도록 했다.
- `lim_x->0`도 하한 `x→0`으로 묶는다.
- 위첨자에는 적용하지 않아 `x^2 = 4` 회귀를 피한다.

### #1305 괄호 뒤 첨자 orphan

- `(...)` 뒤에 `^` 또는 `_`가 올 때만 `EqNode::Paren` 그룹으로 묶는다.
- `(k+1)^2`, `(k-1)^2`의 지수가 괄호 그룹 위첨자로 결합된다.
- 첨자 없는 `(k+1)`, `a(b)`는 기존 느슨한 Symbol 흐름을 유지한다.

### BigOp 중심 정렬

- Canvas/SVG BigOp 렌더에서 별도 `estimate_op_width`를 제거하고 layout의 `estimate_text_width`와 동일 기준을 사용한다.
- `∑`와 상·하한의 가로 중심 정렬을 맞춘다.

## 3. 검증

| 항목 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib equation::parser -- --nocapture` | 78 passed |
| `cargo test --lib` | 1597 passed, 0 failed, 6 ignored |
| `cargo test --tests` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |

`cargo test --tests`에서는 `issue_1139_inline_picture_duplicate` 68개, `issue_1285_tac_sequence_right_align`, `issue_505`, `issue_1061_equation_serialize`, `issue_1219_equation_line_hangul_advance` 등 주요 회귀 묶음이 함께 통과했다.

## 4. 시각 판정 산출물

대상:

```text
samples/3-10월_교육_통합_2022.hwp
page: 11쪽 (-p 10)
```

산출물:

```text
output/poc/pr1306-equation-endnote/page11/3-10월_교육_통합_2022_011.svg
output/poc/pr1306-equation-endnote/page11/3-10월_교육_통합_2022_011.png
output/poc/pr1306-equation-endnote/page11-debug/3-10월_교육_통합_2022_011.svg
output/poc/pr1306-equation-endnote/page11-debug/3-10월_교육_통합_2022_011.png
```

판정 포인트:

- 문18 해설 미주 영역의 `sum_k=1 ^6` 계열 수식에서 `∑` 아래 `k=1`, 위 `6`이 모두 보여야 한다.
- `(k+1)^2`, `(k-1)^2`의 지수 `2`가 괄호 그룹 위첨자로 올라가야 한다.
- `7^2` 등 일반 위첨자 회귀가 없어야 한다.

메인테이너 시각 판정:

```text
2026-06-06 통과
```

## 5. 현재 상태

코드 통합, 테스트, SVG/PNG 시각 판정을 모두 통과했다.

남은 절차:

1. 필요 시 WASM 빌드
2. 커밋
3. `local/devel` 병합
4. `devel` 병합 및 push
5. PR #1306 / 이슈 #1304, #1305 종료 처리
