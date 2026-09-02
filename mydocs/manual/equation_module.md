---
kind: guide
status: active
canonical: mydocs/manual/equation_module.md
last_verified: 2026-08-18
---

# 수식 모듈 매뉴얼

한컴 수식 스크립트를 파싱·배치·그리는 코드는 `src/renderer/equation/` 이다.
이 문서는 그 모듈의 진입점과 명령 디스패치 규약이다. 구현 정본은
[`dispatch.rs`](../../src/renderer/equation/dispatch.rs) 와
[`README.md`](../../src/renderer/equation/README.md).

## 언제 이 문서를 보나

- 수식 명령을 추가하거나 분기 순서를 손대려 할 때
- 파서 if 연쇄가 어디로 갔는지 찾을 때
- M09-1 골든이 무엇을 잠그는지 확인할 때

렌더링 엔진 전체는 [렌더링 엔진 설계](../tech/rendering_engine_design.md),
이슈별 수식 조사는 [issue-139](../tech/investigations/issue-139/README.md).

## 파이프라인

```
script
  → tokenizer::tokenize
  → EqParser::parse          (명령은 classify_command → 핸들러)
  → EqLayout::layout
  → svg_render / canvas_render
```

문서 컨트롤에서 크기가 필요할 때는 `equation::intrinsic_size_hwp(script, font_size)`.

## 명령 디스패치

`parse_command` 는 깊이 가드만 하고, 실제 분기는
`classify_command(cmd) -> EqCommandClass` 한 곳이다.

| 가족 | 대표 명령 | 핸들러 |
| --- | --- | --- |
| `InfixDiscard` | `OVER`, `ATOP` | 단독이면 `Empty`. 결합은 `parse_expression` |
| `LatexFraction` | `FRAC`, `DFRAC`, `TFRAC` | `parse_latex_fraction` |
| `RomanText` | `TEXT`, `OPERATORNAME` | 로만체 `FontStyle` |
| `Phantom` | `PHANTOM`, `VPHANTOM`, `HPHANTOM` | 인자 소비 후 공백 |
| `LatexSpacing` | `QUAD`, `QQUAD`, … | `Text` 간격 |
| `Overset` / `Underset` | `OVERSET`, `STACKREL`, `UNDERSET` | 첨자 AST |
| `BeginEnv` / `EndEnv` | `BEGIN`, `END` | LaTeX 환경 |
| `Sqrt` | `SQRT`, `ROOT` | `parse_sqrt` (`ROOT` 도 `Sqrt`) |
| `IntegralNolimits` | `INT`, `DINT`, `OINT`, … | `MathSymbol` + 일반 첨자 |
| `BigOperator` | `SUM`, `PROD`, … | `parse_big_op` |
| `Limit` | `lim`, `Lim` | 원문 대소문자 |
| `Matrix` | `MATRIX`, `PMATRIX`, `BMATRIX`, `DMATRIX` | `parse_matrix` |
| `Cases` / `EqAlign` / `Pile` | `CASES`, `EQALIGN`, `PILE`… | 각 전용 파서 |
| `LeftDelim` / `RightDiscard` | `LEFT`, `RIGHT` | LEFT 는 그룹 뒤 첨자 결합 |
| `Rel` | `REL`, `BUILDREL` | 화살표 위/아래 |
| `LongDiv` / `Ladder` / `Benzene` / `Bigg` | 자리표시·fallback | 현행 그대로 |
| `Choose` / `Binom` / `Color` | 조합·색 | 현행 그대로 |
| `LeftScript` / `Sup` / `Sub` | `LSUB`, `SUP`, `SUB` | 첨자 동의어 |
| `Fallback` | 그 외 | 장식 → 글꼴 → 기호 → 함수 → `Text` |

적분은 `is_big_operator` 보다 먼저, `lim`/`Lim` 은 그보다 뒤에 분류한다.
`VMATRIX` 는 행렬 가족이 아니다.

## 명령을 추가할 때

1. 새 이름이 기존 가족에 들어가면 `classify_command` 해당 `match` 팔만 고친다.
2. 새 가족이면 `EqCommandClass` 변이 + 분류 + `parse_command_inner` 팔을 같이 추가한다.
3. Fallback 표(`DECORATIONS`/`FONT_STYLES`/`lookup_symbol`/`is_function`)에만 넣을 이름은
   분류표에 올리지 않는다. `BENZENE` 처럼 첨자 결합이 달라지는 명령은 Fallback 으로 내리면 안 된다.
4. M09-1 골든(PR #5412)이 깨지면 동작이 바뀐 것이다. 구조만 손댈 때는 골든을 갱신하지 않는다.

## 하지 않는 것

- 골든을 한컴 정답으로 바꾸지 않는다. 현행 엔진 잠금이다.
- `#4056`(HWPX 내보내기 쪽수), `#4865`(깊은 중첩·괄호 O(n²)) 를 이 매뉴얼 범위에서 고치지 않는다.
- gym / `scripts/visual_sweep.py` 를 수식 디스패치와 묶지 않는다.
