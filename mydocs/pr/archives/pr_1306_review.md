# PR #1306 리뷰 - 수식 미주 시그마/괄호 지수 렌더 수정

- PR: https://github.com/edwardkim/rhwp/pull/1306
- 작성일: 2026-06-06
- 작성자: `planet6897`
- 제목: `수식 미주 시그마·괄호 지수 렌더 수정 (#1304, #1305)`
- base: `devel` / `9d3aa212454f5a7d9d7e081ddaec40a804aeda70`
- head: `fix/equation-endnote-sigma-paren` / `0011f3973b4e4a6377e92aa5d44bbfe8283fb1ee`
- 상태: open, draft 아님
- GitHub mergeable: true

## 1. PR 요약

PR #1306은 `3-10월_교육_통합_2022.hwp` 11쪽 문18 해설 미주 영역의 수식 렌더 오류를 수정한다.

핵심 증상:

- 미주 수식의 `sum_k=1 ^6`에서 하한 `k=1` 중 `=1`이 시그마 하한으로 붙지 않음
- 상한 `6`이 정상적인 시그마 상한 위치로 가지 않음
- `(k+1)^2`, `(k-1)^2`처럼 괄호 뒤 위첨자가 base 없는 orphan으로 파싱되어 지수가 올라가지 않음

PR은 두 작업을 포함한다.

| task | 내용 |
|---|---|
| #1304 | 무브레이스 아래첨자에서 공백 없는 관계연산자 run을 operand로 묶음 |
| #1305 | `(...)` 뒤에 첨자가 올 때만 Paren 그룹으로 묶어 첨자를 결합 |

## 2. 변경 범위

PR 커밋 자체 기준:

```text
3d88eac7 Task #1304: 미주 시그마 무브레이스 첨자 공백 구분 + 상·하한 가로 정렬
0011f397 Task #1305: 괄호 그룹 뒤 위첨자 orphan 수정
```

주요 코드 변경:

| file | 변경 |
|---|---|
| `src/renderer/equation/tokenizer.rs` | `Token.space_before` 추가. 일반 공백을 토큰으로 보존하지 않고, 다음 토큰 앞 공백 여부만 기록 |
| `src/renderer/equation/parser.rs` | `parse_script_operand`, `is_tight_relational`, `parse_paren_group`, `paren_then_script` 추가 |
| `src/renderer/equation/layout.rs` | `estimate_text_width`를 render 계층에서 재사용할 수 있도록 `pub(crate)` 처리 |
| `src/renderer/equation/canvas_render.rs` | BigOp 연산자 폭 추정 기준을 layout과 통일 |
| `src/renderer/equation/svg_render.rs` | Canvas와 동일하게 BigOp 중심 정렬 기준 통일 |

문서 변경:

- `mydocs/plans/task_m100_1304*.md`
- `mydocs/plans/task_m100_1305*.md`
- `mydocs/working/task_m100_1304*.md`
- `mydocs/working/task_m100_1305*.md`
- `mydocs/report/task_m100_1304_report.md`
- `mydocs/report/task_m100_1305_report.md`

## 3. GitHub Actions 상태

PR head `0011f3973b4e4a6377e92aa5d44bbfe8283fb1ee` 기준:

| check | conclusion |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze (rust) | pass |
| Analyze (python) | pass |
| Analyze (javascript-typescript) | pass |
| WASM Build | skipping |

주의: PR base는 `9d3aa212`이고 현재 `local/devel`은 `0a5c29e5`이므로, CI 성공은 PR head 자체의 신호로만 봐야 한다. 수용 전 현행 `local/devel` 위에서 maintainer integration 테스트가 필요하다.

## 4. 코드 검토

### 4.1 `Token.space_before`

일반 공백을 별도 토큰으로 남기지 않고 다음 토큰에 `space_before` 플래그만 붙이는 방식이다.

장점:

- 기존 #505 계열의 "일반 공백은 시각 폭을 만들지 않는다" 정책을 유지한다.
- 파서 전체 루프가 공백 토큰을 새로 skip해야 하는 부담이 없다.
- `sum_k=1 ^6`처럼 공백이 operand 경계로 쓰이는 경우만 판정할 수 있다.

주의점:

- `Token` 생성 지점이 늘어나면 `space_before` 기본값이 잘못 들어갈 수 있다.
- EOF에도 `space_before`가 찍힐 수 있으나 현재 파서에서는 EOF의 의미 토큰 소비가 없으므로 실질 영향은 낮다.

### 4.2 무브레이스 아래첨자 operand

`parse_script_operand`는 `원자 (공백 없는 관계연산자 원자)*`만 묶는다.

예상 개선:

- `sum_k=1 ^6` -> 하한 `k=1`, 상한 `6`
- `lim_x->0` -> 하한 `x→0`

회귀 방지 설계:

- 위첨자에는 적용하지 않음
- 산술 연산자 `+`, `-`는 묶지 않음
- 관계연산자 앞에 공백이 있으면 묶지 않음

따라서 `x^2 = 4`, `a_n b`, `a_n+1` 같은 케이스를 건드리지 않으려는 설계가 명확하다.

### 4.3 괄호 뒤 첨자 결합

`(` 토큰을 만났을 때 매칭 `)` 뒤에 `^` 또는 `_`가 있는 경우에만 `EqNode::Paren`으로 묶는다.

장점:

- `(k+1)^2`의 위첨자 base가 `Paren`이 되어 orphan을 제거한다.
- 첨자가 없는 `(k+1)`, `a(b)`는 기존 느슨한 Symbol 흐름을 유지한다.

주의점:

- round parenthesis 한정이다. 대괄호 `[...]^n`은 PR 본문처럼 비범위로 남는다.
- 중첩 괄호는 depth로 찾기 때문에 기본 구조는 타당하나, 비정상 괄호 입력에 대한 회귀는 테스트가 제한적이다.

### 4.4 BigOp 가로 정렬

Canvas/SVG 렌더에서 기존 `estimate_op_width = char_count * fs * 0.6`을 제거하고, layout의 `estimate_text_width`를 재사용한다.

이 변경은 layout에서 이미 계산한 BigOp 중심과 실제 렌더 중심을 맞추는 방향이라 타당하다. 특히 `∑` 폭 과소추정으로 상·하한과 연산자 중심이 어긋나는 문제를 줄일 수 있다.

## 5. 통합 리스크

### 5.1 전체 PR 브랜치 머지는 위험

`local/devel..local/pr1306-upstream` 전체 diff를 보면 현재 devel에 이미 반영된 문서 archive, 샘플, #1284/#1303 계열 코드가 대량 삭제/되돌림처럼 나타난다.

원인:

```text
PR merge base: 9d3aa212
current local/devel: 0a5c29e5
```

따라서 전체 브랜치 merge 또는 PR head checkout 기반 수용은 금지해야 한다.

### 5.2 코드 패치 선별 적용은 가능

다음 코드 파일만 merge-base 기준 diff로 떼어 현행 `local/devel`에 적용하는 dry-run은 통과했다.

```text
src/renderer/equation/canvas_render.rs
src/renderer/equation/layout.rs
src/renderer/equation/parser.rs
src/renderer/equation/svg_render.rs
src/renderer/equation/tokenizer.rs
```

즉 권장 방식은 `local/devel`에서 integration branch를 만들고, PR #1306의 수식 코드 패치와 필요한 테스트만 선별 적용하는 것이다.

## 6. 권장 처리

권장: **수용 방향으로 진행**.

단, 전체 PR 브랜치 merge가 아니라 **현행 `local/devel` 위에 코드 패치 선별 적용**으로 진행한다.

근거:

- 문제 정의가 명확하고 PR 설계가 좁다.
- PR head 기준 CI/CodeQL/Canvas visual diff가 통과했다.
- `Token.space_before` 방식은 기존 공백 렌더 정책을 크게 흔들지 않는다.
- 괄호 뒤 첨자 결합도 trailing-script 케이스만 건드려 회귀 범위를 줄였다.
- 코드 패치만 현행 `local/devel`에 적용하는 dry-run이 통과했다.

권장 절차:

1. `local/devel`에서 `local/pr1306-integration` 생성
2. merge-base `9d3aa212..local/pr1306-upstream`의 수식 코드 파일 diff만 적용
3. 필요한 단위 테스트를 현행 `parser.rs` 테스트 모듈에 함께 반영
4. `cargo fmt --all -- --check`
5. `cargo test --lib equation::parser`
6. `cargo test --lib`
7. `cargo test --tests`
8. `cargo clippy --lib -- -D warnings`
9. SVG 시각 판정:
   - `samples/3-10월_교육_통합_2022.hwp` 11쪽
   - 문18 미주 4줄 시그마 상·하한
   - `(k+1)^2`, `(k-1)^2` 지수 위치
10. 필요 시 WASM 빌드 후 rhwp-studio 시각 판정
11. 통과 시 `local/devel` 병합, `devel` 병합/push, PR/이슈 종료 처리

## 7. 시각 판정 포인트

대상 샘플:

```text
samples/3-10월_교육_통합_2022.hwp
pdf/3-10월_교육_통합_2022.pdf
```

우선 확인:

- 11쪽 문18 해설 미주 영역
- `sum_k=1 ^6` 계열 4개 수식
- `∑` 위 `6`, 아래 `k=1` 전체가 보여야 함
- `(k+1)^2`, `(k-1)^2`에서 `2`가 괄호 그룹의 위첨자로 올라가야 함
- `7^2`와 일반 단일 위첨자 회귀 없음

## 8. PR 코멘트 초안

```markdown
확인했습니다. 이번 PR은 미주 수식에서 `sum_k=1 ^6`처럼 브레이스 없이 공백으로 구분되는 시그마 하한을 `k=1` 전체로 파싱하고, `(k+1)^2`처럼 괄호 그룹 뒤 위첨자가 orphan으로 떨어지는 문제를 함께 수정합니다.

PR head 기준 Build & Test / CodeQL / Canvas visual diff가 통과한 것도 확인했습니다.

다만 PR base가 현재 `devel`보다 뒤라서 전체 브랜치 머지는 최근 변경을 되돌릴 위험이 있습니다. maintainer integration에서는 수식 관련 코드 패치만 최신 `local/devel` 위에 선별 적용하고, `3-10월_교육_통합_2022.hwp` 11쪽 문18 해설 수식에 대해 SVG/웹 시각 판정을 진행하겠습니다.
```
