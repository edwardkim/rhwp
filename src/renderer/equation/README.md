# 수식 모듈 (`src/renderer/equation`)

한컴 수식 스크립트(버전 6.0)를 토큰 → AST → 레이아웃 → SVG/Canvas 로 그린다.
명령 분기의 정본은 [`dispatch.rs`](dispatch.rs) 이고, 파서는 그 분류만 본다.

매뉴얼: [`mydocs/manual/equation_module.md`](../../../mydocs/manual/equation_module.md).

## 파일

| 파일 | 역할 |
| --- | --- |
| `tokenizer.rs` | 스크립트 → 토큰 |
| `dispatch.rs` | 명령 이름 → `EqCommandClass` (분류만, 파싱 없음) |
| `parser.rs` | 토큰 → `EqNode`. `parse_command` 가 분류표로 핸들러를 고른다 |
| `ast.rs` | `EqNode` / `MatrixStyle` / `PileAlign` |
| `symbols.rs` | 기호·함수·장식·큰 연산자 표 |
| `layout.rs` | `EqNode` → `LayoutBox` |
| `svg_render.rs` | `LayoutBox` → SVG 조각 |
| `canvas_render.rs` | WASM Canvas 경로 |
| `mod.rs` | `intrinsic_size_hwp` 진입점 |

## 디스패치 순서

`classify_command` 의 앞쪽이 이긴다. 순서를 바꾸면 동작이 바뀐다.

1. 중위 폐기 (`OVER`/`ATOP`) — 결합은 `parse_expression`
2. LaTeX 분수·본문·phantom·간격·overset/underset·begin/end
3. `SQRT`/`ROOT` (`ROOT` 도 AST 는 `Sqrt`)
4. 적분 nolimits (`INT`/`DINT`/…) — `is_big_operator` 보다 앞
5. 큰 연산자 (`SUM`/`PROD`/…)
6. `lim`/`Lim` (원문 대소문자)
7. 행렬·cases·eqalign·pile·LEFT/RIGHT·rel·longdiv·ladder·benzene·bigg·choose/binom·color·첨자 동의어
8. Fallback: 장식 → 글꼴 → 기호 → 함수 → `Text`

`VMATRIX`/`SMALLMATRIX` 는 분류표에 없다. 현행처럼 fallback 이다.

## 현행 잠금 (바꾸지 않음)

- `ROOT` 는 `SQRT` 와 같은 파서 분기라 AST 가 `Sqrt` 다.
- `PILE`/`LPILE`/`RPILE` 레이아웃은 전용 kind 없이 `Row` 로 세로 쌓는다.
- 중괄호 없는 `MATRIX` 는 `Empty`.
- 깊이 가드·괄호 짝 계산은 이 모듈이 손대지 않는다.

골든은 M09-1 (PR #5412). 의도된 엔진 변경이 아니면 `UPDATE_EQ_GOLDENS=1` 로 갱신하지 않는다.
