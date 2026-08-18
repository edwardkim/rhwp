# 수식 명령 골든 (M09-1 + M09-g)

현행 엔진 잠금이다. 한컴 정답 오라클이 아니다. 엔진을 고치거나
디스패치를 리팩터하지 않는다(M09-2). #4056 은 이 트리에서 다루지 않는다.

- 카탈로그: [`catalog.tsv`](catalog.tsv) (`id`, `dir`, `command`, `honesty`, `script`)
- M09-1 원본 31종: [`m09_1/`](m09_1/)
- 변형·예외·미구현 정직 표: [`m09_expand/`](m09_expand/)
- 시험: `tests/cases/equation_command_goldens.rs`

갱신:

```bash
UPDATE_EQ_GOLDENS=1 cargo test --test <regression_suite> equation_command_goldens
```

의도된 엔진 변경이 아니면 골든을 다시 찍지 않는다.

## honesty 태그

| 태그 | 의미 (현행 동작) |
| --- | --- |
| `implemented` | M09-1 대표 경로. 파서·레이아웃·SVG 가 해당 명령을 처리한다. |
| `implemented-variant` | 같은 명령의 공백·대소문자·크기·중첩·불규칙 격자 변형. |
| `root-aliases-sqrt` | `ROOT` 는 `parse_sqrt` 와 같은 분기라 AST 가 `Sqrt` 이다. |
| `pile-layout-as-row` | `PILE`/`LPILE`/`RPILE` AST 는 `Pile` 이지만 레이아웃은 전용 kind 없이 `Row` 로 세로 쌓는다. |
| `matrix-no-brace-empty` | `parse_matrix`/`parse_eqalign`/`parse_pile` 은 여는 `{` 가 없으면 `Empty`. |
| `missing-operand` | 피연산자·본문·인덱스가 비면 `Empty` 를 끼워 진행한다. |
| `case-insensitive` | hwpeq 명령은 ASCII 대소문자를 가리지 않는다. |
| `atop-vs-over` | `ATOP` 는 분수선 없는 위/아래(`Atop`). `OVER` 와 같은 중위 결합을 쓴다. |
| `latex-frac` | `FRAC`/`DFRAC`/`TFRAC` 는 LaTeX 스타일 두 인자 분수. |
| `vmatrix-unimpl-text` | `VMATRIX` 는 `is_structure_command` 에 있으나 `parse_command` 미분기 → `Text("VMATRIX")` + 뒤 그룹. |
| `smallmatrix-unimpl-text` | `SMALLMATRIX` 동일. 전용 스타일/축소 행렬이 없다. |
| `ladder-fallback-matrix` | `LADDER`/`SLADDER` 는 `parse_matrix(Plain)` 으로 떨어진다. |
| `benzene-placeholder` | `BENZENE` 은 분자 그림 대신 `MathSymbol("⌬")`. |
| `bigg-size-ignored` | `BIGG` 는 크기 변경 없이 다음 요소만 반환한다. |
| `choose-empty-top` | `CHOOSE` 는 앞 요소를 결합하지 못해 `Atop` 윗칸이 `Empty` 다. `BINOM` 은 두 인자를 쓴다. |
| `longdiv-simplified-row` | `LONGDIV` 는 장제법 그림이 아니라 `몫 ÷ 제수 = 본문` 가로 나열. |
| `color-passthrough` | `COLOR` 는 색을 레이아웃 kind 로 남기지 않고 본문만 통과한다. |
| `unknown-command-text` | 미지 명령은 `Text(cmd)` 로 누수한다. |
| `cases-related` | `CASES` 는 행렬 가족이 아니지만 같은 `{ # & }` 수집기다. |
| `rel-buildrel` | `REL`/`BUILDREL` 화살표 위·아래. 한컴 장식 화살표의 현행 근사. |
| `phantom-space` | `PHANTOM` 계열은 본문을 버리고 공백 `Text` 한 칸. |
| `latex-env-partial` | `BEGIN{matrix}` 등 LaTeX 환경은 부분 구현. `END` 단독은 `Empty`. |
| `latex-space` | `QUAD` 등 간격 명령은 심볼 표의 공백 문자. |
| `latex-text` | `TEXT`/`OPERATORNAME` 은 Roman `FontStyle`. |
| `latex-stack` | `OVERSET`/`UNDERSET`/`STACKREL` 은 첨자 노드로 근사. |
| `limit-cmd` | `lim`/`Lim` 전용 극한 분기 (`Lim` 만 대소문자 구분). |
| `left-script` | `LSUB`/`LSUP`/`SUB`/`SUP` 의 현행 첨자 부착. |

## 하지 않은 것

- 수식 디스패치 재작성 (M09-2, PR #5420 이 있으면 유지)
- #4056 수정 (planet PR #5253)
- gym / `scripts/visual_sweep.py`
- 파서·레이아웃·SVG 렌더러 구현 변경
