---
kind: guide
status: active
canonical: mydocs/manual/equation_module.md
last_verified: 2026-10-02
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

### 한글 수식의 대체 메트릭과 저장 줄

`HYhwpEQ`의 한글 cmap은 비어 있습니다. 한글 수식은 SVG·Canvas·Native Skia에서
명조 대체 글꼴(`Haansoft Batang` 우선)을 사용합니다. SVG 임베딩도 한글을 실제 대체
face에 수집하여 수학 글꼴의 누락 글리프로 처리하지 않습니다.

보호되지 않은 한글 `HYhwpEQ` 수식은 `flow_metrics_hwp`의 공통 폭·높이·기준선을
사용합니다. 일반 영문 수식과 크기가 보호된 수식은 저장 상자를 유지합니다. 저장 줄이
있는 본문에서는 메트릭이 달라진 최초 수식의 소유 행부터 프레임 채움을 다시 수행하고,
그 앞의 줄은 프레임 수용을 확인한 뒤 원래 줄·문자 축을 유지합니다. 부분 채움의 첫
행을 문자0으로 초기화하지 않으며, 새 행만 재조판 축을 사용합니다. 측정·그림자 배치로
폭만 넓히거나 전체 문단을 다시 조판하여 정상 앞행을 바꾸면 안 됩니다.

이 경로의 한컴 비교 근거는 [#7382 보정330 글꼴 증거](../pr/assets/pr7382_20260926/stage330_equation_font_evidence.json)입니다.
AST의 추정 글자 폭은 실제 한컴 글꼴 조형을 완전히 복원하지 않으므로, 수치 통과와 별개로
수식·뒤 본문·각주와 앞뒤 쪽의 review PNG를 확인합니다. 이 증거는 모든 수식/출력 backend의
전체 시각 검증을 대체하지 않습니다.

한컴2024의 별도 한글 수식 대조군(baseUnit600/800/1000/1200/1600)에서는 한글 전진이
96dpi에서 약8/10/14/16/22px입니다. 이 관측은48dpi 논리 단위에서 반올림한 뒤 출력 DPI로
환산하는 방식과 일치합니다. `EqLayout::for_equation`은 `HYhwpEQ`의 CJK 텍스트에만
이 전진을 적용하고, SVG·Canvas·Skia는 같은 LayoutBox 폭에서 개별 글자 원점을 얻습니다.
글리프 크기·수식 원문·저장 상자를 직접 축소하지 않습니다. 일반 `EqLayout::new`와
OLE 역산 경로는 기존 폭을 유지합니다. 부분 재조판의 이어받는 본문 가용폭은 실제 배치와
동일한 글머리표 문자열·폭을 제외합니다. 진단 대조군은 실제 원본의 대체 기준이 아니며,
원본88쪽과 앞뒤 쪽 비교를 별도로 기록합니다.
수식 부분 재조판 뒤 다음 문단의 저장 시작이 이전 저장 줄끝+줄간격과 연속이면,
공유 `HeightCursor`는 실제 순차 높이로 저장 사다리의 기준점을 갱신합니다.
옛 수식 상자 높이를 후속 저장 vpos로 복원하면 안 됩니다. 저장 단/쪽 리셋·실제 gap,
프레임이 소유하지 않는 float·보호 수식 및 편집 경로는 이 재앵커의 대상이 아닙니다.
이 조건은 페이지네이터와 renderer의 같은 커서에서 실행됩니다.


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
