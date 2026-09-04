# 완료 보고서 — Task M100-2973

- 이슈: #2973
- 제목: borderFill@threeD 속성이 파싱되지 않고 직렬화 시 항상 "0"으로 하드코딩됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-2973-borderfill-threed`

## 1. 배경 및 근본 원인

HWPX 헤더의 `<hh:borderFill>` 요소는 `id`, `threeD`, `shadow`, `centerLine`,
`breakCellSeparateLine` 속성을 갖는다. `src/parser/hwpx/header.rs::parse_border_fill`
은 이 중 `centerLine` 만 읽고 나머지는 전부 버렸고(모델 `BorderFill` 구조체에 대응
필드 자체가 없었음), `src/serializer/hwpx/header.rs`는 `threeD` 를 항상 문자열
`"0"` 으로 고정 출력했다. 결과적으로 원본 HWPX 문서에 `threeD="1"` 이 설정돼 있어도
파싱 단계에서 정보가 소실되고, 재저장 시 무조건 `"0"` 으로 찍혀 3차원 효과 설정이
조용히 사라지는 라운드트립 데이터 손실이 있었다.

이 저장소에는 동일 유형(HWPX 속성은 실존하지만 모델 필드가 없어 파싱이 버려지고
직렬화기가 상수를 하드코딩)의 버그가 여러 차례 확인·수정된 이력이 있다
(`write_diagonal` 함수의 회귀 가드 주석, `<hh:img>` bright/contrast/effect
파싱 누락 이슈였던 [Issue #1156] 등). 이번 `threeD` 문제도 같은 패턴이다.

pageBorderFill@type 슬롯 배정 이슈(#2896/#2906, BOTH/EVEN/ODD 슬롯 배정 문제)와는
무관한, `borderFill` 개별 속성(threeD) 파싱 누락/하드코딩 문제다.

## 2. 수정 내용

- `src/model/style.rs`
  - `BorderFill` 구조체에 `pub three_d: bool` 필드 추가.
- `src/parser/hwpx/header.rs`
  - `parse_border_fill` 의 속성 루프를 `match` 로 바꾸고 `threeD` 속성을
    `parse_bool` 로 읽어 `bf.three_d` 에 반영.
  - 단위 테스트 `test_border_fill_three_d_attr_parsed` 추가
    (`threeD="1"` → `bf.three_d == true` 검증).
- `src/serializer/hwpx/header.rs`
  - 하드코딩된 `("threeD", "0")` 을 `("threeD", if bf.three_d { "1" } else { "0" })`
    로 교체해 실제 모델 값을 직렬화.

`BorderFill` 구조체에 필드를 추가한 파급으로, 기존에 모든 필드를 명시적으로 나열해
구조체 리터럴을 만들던 호출부(`..Default::default()` 를 쓰지 않는 곳)에
`three_d: false` 를 추가해 컴파일을 통과시켰다(`src/document_core/html_table_import.rs`,
`src/document_core/commands/object_ops/table.rs`, `src/parser/doc_info.rs`(HWP5
바이너리 파서 — HWP5 BorderFill 레코드에는 threeD 개념이 없어 `false` 고정),
`src/serializer/doc_info/tests.rs`, `src/wasm_api/tests.rs`). `..Default::default()`
를 쓰던 곳(`src/document_core/builders/exam_paper.rs`, `src/renderer/style_resolver.rs`)
은 자동으로 `three_d: false` 가 적용되어 수정이 필요 없었다.

## 3. Red → Green

- Red: `threeD` 속성을 파서가 읽지 않던 시점 기준으로, `bf.three_d` 필드가 없어
  테스트 자체가 컴파일되지 않거나(필드 추가 전) 필드를 추가해도 파서 match 암을
  넣기 전에는 `bf.three_d` 가 항상 `Default` 값인 `false` 로 남아
  `assert!(bf.three_d)` 가 실패했다.
- Green: `parse_border_fill` 에 `b"threeD" => bf.three_d = parse_bool(&attr)` 암을
  추가한 뒤 통과.

```
test parser::hwpx::header::tests::test_border_fill_three_d_attr_parsed ... ok
```

## 4. 스코프 밖 (후속 이슈 제안)

`shadow`, `breakCellSeparateLine` 속성도 동일 패턴(파싱 안 됨 + 직렬화 시 상수
하드코딩)이지만, 이번 수정은 diff를 최소화하기 위해 `threeD` 단일 속성으로
스코프를 좁혔다. 나머지는 별도 이슈로 분리하는 것을 제안한다.

## 5. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib test_border_fill_three_d_attr_parsed`
- `rustfmt --edition 2021` (변경 파일 전체)

## 6. 변경 파일

- `src/model/style.rs`
- `src/parser/hwpx/header.rs`
- `src/serializer/hwpx/header.rs`
- `src/document_core/html_table_import.rs`
- `src/document_core/commands/object_ops/table.rs`
- `src/parser/doc_info.rs`
- `src/serializer/doc_info/tests.rs`
- `src/wasm_api/tests.rs`
