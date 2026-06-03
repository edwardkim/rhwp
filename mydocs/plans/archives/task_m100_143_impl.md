# Task #143: LaTeX 분수·제곱근 입력 최소 지원 — 구현계획서

## 구현 원칙

- 기존 한컴 수식 문법을 바꾸지 않는다.
- 별도 대형 LaTeX 파서를 만들지 않는다.
- 현재 AST와 렌더러가 이미 지원하는 구조만 연결한다.
- 테스트를 먼저 추가하고 실패를 확인한 뒤 최소 구현한다.

## 1단계: 실패 테스트 작성

대상 파일:

- `src/renderer/equation/tokenizer.rs`
- `src/renderer/equation/parser.rs`

테스트:

- `\frac{1}{2}` 토큰이 `Command("frac")`, `{`, `1`, `}`, `{`, `2`, `}` 형태가 되는지 확인
- `\frac{1}{2}` 파싱 결과가 `EqNode::Fraction` 인지 확인
- `x=\frac{-b \pm \sqrt{b^2}}{2a}` 파싱 결과 문자열에 `Fraction`, `MathSymbol("±")`, `Sqrt` 가 모두 포함되는지 확인
- `\sqrt[3]{x}` 가 index 포함 `Sqrt` 인지 확인

검증:

```bash
cargo test --lib equation::tokenizer::tests::test_latex_command_prefix
cargo test --lib equation::parser::tests::test_latex_frac
```

수정 전 실패를 확인한다.

## 2단계: 토크나이저 최소 수정

대상 파일:

- `src/renderer/equation/tokenizer.rs`

변경:

- `\` 뒤에 ASCII alphabetic 문자가 오면 백슬래시는 소비하고 뒤의 명령어 이름만 `Command` 로 반환한다.
- `\` 단독 또는 `\` 뒤 명령어가 아닌 경우는 기존처럼 텍스트/기호에 가깝게 안전 처리한다.

## 3단계: 파서 최소 수정

대상 파일:

- `src/renderer/equation/parser.rs`

변경:

- `parse_command()` 에 `FRAC`/`DFRAC`/`TFRAC` 분기 추가
- `parse_latex_fraction()` 헬퍼 추가: 두 개의 `parse_single_or_group()` 결과를 `EqNode::Fraction` 으로 반환
- `parse_sqrt()` 에 LaTeX 대괄호 인덱스 `\sqrt[n]{x}` 처리 추가

## 4단계: 검증

명령:

```bash
cargo test --lib equation
cargo test --lib
```

시각 검증:

- 임시 HTML 또는 SVG 파일로 `\frac{1}{2}` 와 `x=\frac{-b \pm \sqrt{b^2}}{2a}` 렌더링을 생성
- Playwright로 PNG 스크린샷 저장
- 스크린샷 경로를 최종 보고와 PR 본문에 기록

## PR

- 브랜치: `task143-latex-frac-preview`
- PR 제목: `Task #143: LaTeX fraction preview support`
- Draft PR로 생성
- 본문에는 `Refs #143` 로 연결하고, 전체 Issue #143을 닫지 않는다.
