# Task #143 Stage 1: LaTeX 분수·제곱근 최소 지원 구현 완료

## 작업 범위

Issue #143 전체 듀얼 LaTeX 파서 중, 수식 편집기 붙여넣기에서 즉시 체감되는 최소 범위만 구현했다.

- `\frac{a}{b}` → `EqNode::Fraction`
- `\sqrt{x}` → 기존 `SQRT` 경로 재사용
- `\sqrt[n]{x}` → index 포함 `EqNode::Sqrt`
- `\pm` 등 기존 매핑된 기호는 백슬래시 명령 토큰화 후 기존 `lookup_symbol` 경로 재사용

## RED 확인

수정 전 신규 테스트 실패를 확인했다.

```text
renderer::equation::tokenizer::tests::test_latex_command_prefix
left: [Text, Command, LBrace, Number, RBrace, LBrace, Number, RBrace]
right: [Command, LBrace, Number, RBrace, LBrace, Number, RBrace]

renderer::equation::parser::tests::test_latex_frac
Expected Fraction, got Row([Text("\\"), Text("frac"), Number("1"), Number("2")])
```

## 구현

대상 파일:

- `src/renderer/equation/tokenizer.rs`
- `src/renderer/equation/parser.rs`

변경 내용:

- 토크나이저가 `\` 뒤 ASCII 알파벳 명령어를 `Command` 로 반환하도록 추가
- 파서가 `frac` 계열 명령을 두 인자 `EqNode::Fraction` 으로 변환
- 파서가 LaTeX 대괄호 루트 인덱스 `\sqrt[n]{x}` 를 처리

## 검증

```text
cargo test --lib test_latex
4 passed; 0 failed

cargo test --lib equation
76 passed; 0 failed

cargo test --lib
1106 passed; 0 failed; 1 ignored
```

## 시각 검증

렌더러를 `cargo build --lib` 로 다시 빌드한 뒤 Playwright로 실제 PNG를 캡처했다.

스크린샷:

![LaTeX equation preview](task_m100_143_stage1/latex_equation_preview.png)

초기 캡처 1회는 오래된 `target/debug/librhwp.rlib` 에 링크되어 기존 깨진 렌더가 표시되었다. 이후 `cargo build --lib` 재실행 후 같은 생성 스크립트로 재캡처하여 위 스크린샷에서 `\frac{1}{2}` 와 이차방정식 조각이 분수/제곱근으로 렌더되는 것을 확인했다.
