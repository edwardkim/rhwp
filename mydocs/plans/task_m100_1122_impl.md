# 구현 계획서: Task M100-1122

## 1. 수정 대상

수식 토큰화와 문 26) 그림 표시 크기 경로를 수정한다.

- `src/renderer/equation/tokenizer.rs`
- `src/renderer/layout/utils.rs`
- `src/renderer/layout/picture_footnote.rs`
- `src/renderer/layout.rs`

테스트는 기존 수식 파서/토크나이저 테스트 위치에 추가한다.

- `src/renderer/equation/tokenizer.rs` 테스트 모듈
- `src/renderer/equation/parser.rs` 테스트 모듈
- `src/renderer/layout/utils.rs` 테스트 모듈

## 2. 구현 방침

### 2.1 `over`/`atop` + 숫자 결합 분리

현재 `read_command()`는 ASCII 알파벳/숫자를 연속으로 읽기 때문에 `over20`을 하나의 `Command("over20")`으로 만든다. 이 결과 파서는 `OVER` 중위 연산자를 보지 못하고, 렌더러는 `over20`을 일반 텍스트처럼 출력한다.

정정 방향:

- `read_command()`의 prefix 분리 대상에 `over`, `OVER`, `atop`, `ATOP`을 추가한다.
- 단, 뒤 문자가 숫자인 경우에만 분리한다.
- `overline`, `overset`, `alphabet` 같은 명령어/식별자는 영향을 받지 않도록 한다.

예상 토큰:

```text
"11 over20" => Number("11"), Command("over"), Number("20")
"3 over5"  => Number("3"),  Command("over"), Number("5")
"a atop2"  => Command("a"),  Command("atop"), Number("2")
```

### 2.2 테스트

토크나이저 테스트:

- `tokenize("11 over20")` 값이 `["11", "over", "20"]`.
- `tokenize("7 over10")` 값이 `["7", "over", "10"]`.
- `tokenize(r"\overline{AB}")`는 기존 LaTeX 명령어 경로로 `overline`을 유지.
- `tokenize("overlap")`는 `overlap` 단일 명령어로 유지.

파서 테스트:

- `parse("11 over20")`가 `EqNode::Fraction { numer: 11, denom: 20 }`.
- `parse("3 over5")`가 `EqNode::Fraction { numer: 3, denom: 5 }`.
- `parse("{8} over {13}")` 기존 동작 유지.

### 2.3 렌더 확인

### 2.3 문 26) 그림 표시 크기

문 26) 주머니 그림은 HWP 레코드에 `common.width=3365 HU`, `shape_attr.current_width=9014 HU`가 함께 존재한다. 기존 SVG 경로는 `common.width`만 사용해 한컴오피스보다 좁게 표시한다.

정정 방향:

- `picture_display_size_hu()` 유틸을 추가한다.
- `current_width/current_height`가 양수이고 `common.width/height`보다 큰 경우 해당 축은 `current` 값을 사용한다.
- `layout_picture_full()`, `layout_body_picture()`와 column 오른쪽 overflow skip 계산에서 같은 표시 크기를 사용한다.
- `current`가 `common`보다 작거나 0인 경우 기존 `common` 크기를 유지한다.

테스트:

- `picture_display_size_uses_larger_current_axis`
- `picture_display_size_keeps_common_when_current_is_smaller`

### 2.4 렌더 확인

구현 후 다음을 확인한다.

```bash
cargo test --lib renderer::equation::tokenizer
cargo test --lib renderer::equation::parser
cargo test --lib renderer::layout::utils::tests::picture_display_size
cargo test --test issue_505
./target/debug/rhwp export-svg samples/3-11월_실전_통합_2022.hwp -p 3 --debug-overlay -o output/task1121_after
rg -n "over20|over5|over10|over4" output/task1121_after/3-11월_실전_통합_2022_004.svg
```

기대:

- `over20`, `over5`, `over10`, `over4` 문자열이 SVG에 남지 않는다.
- 보기 ①~⑤에 분수선 `<line ...>`과 분모 텍스트가 생성된다.
- 문 26) 주머니 그림이 `current_width/current_height` 기준인 약 `120.19px x 125.95px`로 출력된다.

## 3. 완료 조건

- page 4의 `pi=223~225` 보기 수식이 한컴처럼 분수 형태로 표시된다.
- 문 26) 그림 폭이 한컴처럼 현재 표시 크기 기준으로 표시된다.
- 기존 공백 포함 `OVER` 분수와 Task #505의 CASES/PILE/EQALIGN 분수 처리가 유지된다.
- 단계 보고서 `mydocs/working/task_m100_1122_stage1.md`와 최종 보고서 `mydocs/report/task_m100_1122_report.md`를 작성한다.

## 4. 승인 대기

작업지시자의 "새로운 이슈로 등록하고 개선 진행" 지시에 따라 `tokenizer.rs`와 테스트만 좁게 수정한다.
