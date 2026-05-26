# 구현 계획서 — Task #1139

## 진단 우선순위

1. `src/renderer/equation/tokenizer.rs`
   - 한컴 수식 스크립트의 backtick spacing, style command, 구조 명령 토큰화 결과를 확인한다.
2. `src/renderer/equation/parser.rs`
   - 알 수 없는 명령이 `EqNode::Text`로 떨어지는 지점을 추적한다.
   - `rm`, `it`, `ANGLE`, `bar`, `LEFT/RIGHT`, `lim`, `int` 조합에서 명령 문자열이 표시 문자로 새는지 확인한다.
3. `src/renderer/equation/svg_render.rs` 및 canvas render 경로
   - AST가 정상인데 출력 계층에서 fallback glyph가 생기는지 확인한다.
4. TAC 배치 경로
   - `project_equation_always_tac`에 따라 수식/그림/미주 컨트롤의 인라인 위치가 문단 텍스트에 잘못 삽입되는지 확인한다.

## 예상 수정 후보

### 후보 A: 수식 명령 fallback 정정

조건:

- AST에 `EqNode::Text("it")`, `EqNode::Text("ANGLE")` 등 명령어 자체가 남는다.

수정:

- 한컴 수식 문법에서 스타일/장식/기하 기호로 쓰이는 명령을 `symbols.rs` 또는 parser 전용 처리에 추가한다.
- 명령이 body 없이 쓰이는 경우 표시 텍스트가 아니라 스타일 상태 또는 빈 노드로 처리한다.

검증:

- 해당 script를 직접 파싱하는 단위 테스트 추가.
- SVG에 명령 문자열이 직접 출력되지 않는지 확인.

### 후보 B: TAC 그림/개체 마커 출력 정정

조건:

- 이상 문자가 Equation이 아니라 문단 내 TAC 그림 또는 개체 placeholder에서 나온다.

수정:

- `composer.rs` 또는 paragraph layout의 inline control marker 주입 경로를 확인하고, 실제 렌더 대상인 TAC control만 좌표 예약에 쓰이도록 제한한다.
- 원본에 의도된 TAC 그림은 유지하고, object replacement marker 자체는 화면 텍스트로 표시하지 않는다.

검증:

- 5쪽 문27의 작은 도형/기호가 한컴과 같이 표시되는지 SVG와 rhwp-studio에서 확인.

### 후보 C: 폰트 fallback glyph 정정

조건:

- AST와 출력 문자열은 정상인데 브라우저 폰트 fallback에서 의도와 다른 글리프가 보인다.

수정:

- 수식 렌더 폰트 체인 또는 특정 Unicode 기호 매핑을 한컴 출력과 맞춘다.
- 폰트 추가가 필요하면 `style_resolver`와 `font_metrics_data` 동기화 규칙을 따른다.

검증:

- SVG/Canvas 양쪽에서 동일한 glyph가 보이는지 확인.

## 회귀 테스트 계획

- 신규 테스트 파일 후보: `tests/issue_1139_equation_render.rs`
- 최소 fixture:
  - 문23 `lim _{x\` -> \`0} ...`
  - 문24 `int _{0} ^{pi } ...`
  - 문27 `bar{rm ... it}`, `ANGLE ...`
- 테스트 기대값:
  - parser AST 또는 SVG fragment에 명령 문자열 누출이 없어야 한다.
  - 수식 컨트롤 수와 TAC 속성은 보존되어야 한다.

## 검증 명령

```bash
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_after --debug-overlay
cargo test issue_1139
cargo test --lib
wasm-pack build --target web --out-dir pkg
```

`wasm-pack`은 Rust/WASM 또는 rhwp-studio 표시 경로가 변경될 때 수행한다.

## 산출물

- `mydocs/working/task_m100_1139_stage1.md`
- 필요 시 `mydocs/working/task_m100_1139_stage2.md`
- `mydocs/report/task_m100_1139_report.md`

## 승인 대기

소스 수정은 작업지시자 승인 후 시작한다.

