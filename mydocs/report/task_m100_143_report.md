# Task #143: LaTeX 분수·제곱근 입력 최소 지원 — 결과보고서

## 요약

수식 편집기 미리보기에서 LaTeX 기본 수식이 문자 그대로 렌더링되던 문제를 최소 범위로 정정했다.

이번 PR은 Issue #143 전체 구현을 닫지 않는다. `\frac`, `\sqrt`, `\sqrt[n]` 같은 기본 구조만 기존 AST에 연결하는 1차 지원이다.

## 변경 파일

- `src/renderer/equation/tokenizer.rs`
- `src/renderer/equation/parser.rs`
- `mydocs/orders/20260504.md`
- `mydocs/plans/task_m100_143.md`
- `mydocs/plans/task_m100_143_impl.md`
- `mydocs/working/task_m100_143_stage1.md`
- `mydocs/working/task_m100_143_stage1/latex_equation_preview.png`

## 해결 내용

- `\frac{1}{2}` 가 `Row([Text("\\"), Text("frac"), ...])` 로 파싱되던 문제를 `EqNode::Fraction` 으로 정정
- `x=\frac{-b \pm \sqrt{b^2}}{2a}` 에서 분수, `±`, 제곱근이 한 번에 AST에 반영되도록 검증
- `\sqrt[3]{x}` 를 index 포함 제곱근으로 파싱

## 검증

```text
cargo test --lib test_latex
4 passed; 0 failed

cargo test --lib equation
76 passed; 0 failed

cargo test --lib
1106 passed; 0 failed; 1 ignored
```

시각 증거:

- `mydocs/working/task_m100_143_stage1/latex_equation_preview.png`

## 비고

전체 LaTeX 파서, 환경 블록, 수식 편집 UI 모드 토글은 미구현이다. 본 PR은 `Refs #143` 로 연결하고 issue를 close 하지 않는 것이 맞다.
