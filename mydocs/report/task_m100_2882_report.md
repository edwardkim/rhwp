# #2882 처리 결과 — HWPX hp:ole/hp:chart numberingType 파싱만 되고 저장 시 NONE 하드코딩 수정

## 문제

`src/parser/hwpx/section.rs`의 `parse_hp_ole_element`(OLE, fallback 경로)와
`parse_hp_chart_element`(차트) 두 함수가 `numberingType` 속성을 읽긴 하지만,
그 값을 `bool` 지역 변수(`numbering_type_picture`)로만 축약해 HWP5 변환용 비트
플래그(`hwp5_gen_shape_attr_bit28`)에만 반영하고, HWPX 라운드트립 직렬화가 실제로
참조하는 `common.numbering_type: ObjectNumberingType` 필드는 채우지 않는다.
직렬화기(`numbering_type_str`, `src/serializer/hwpx/shape.rs`)는 오직
`common.numbering_type`만 보므로, 저장 시 항상 `NONE`으로 되쓰인다. 같은 파일의
공용 도형 파서(`section.rs:2892` 부근)는 이미 올바른 패턴으로 매핑하고 있어,
OLE/차트 두 파서만 이 매핑에서 빠져 있었다.

## 수정

`src/parser/hwpx/section.rs`의 `parse_hp_chart_element`와 `parse_hp_ole_element`
두 함수의 `b"numberingType"` 핸들러에, 공용 도형 파서(`section.rs:2892`)와 동일한
패턴으로 `common.numbering_type` 매핑을 추가:

```rust
common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
    _ => crate::model::shape::ObjectNumberingType::None,
};
```

기존 `numbering_type_picture` bool 변수와 그것이 채우는
`common.hwp5_gen_shape_attr_bit28`(HWP5 변환 경로용)은 그대로 유지했다 —
이슈 본문이 명시한 대로 HWP5 바이너리 레이아웃(#1283)과는 무관한, HWPX XML
라운드트립 문제만 다룬다.

## 테스트 (red → green)

`src/parser/hwpx/section.rs`의 기존 `mod tests`에 2건 추가:

- `issue2882_ole_numbering_type_picture_is_parsed_into_common_field` —
  `<hp:ole numberingType="PICTURE" .../>`를 파싱해 `OleShape.common.numbering_type`이
  `ObjectNumberingType::Picture`임을 단언
- `issue2882_chart_numbering_type_table_is_parsed_into_common_field` —
  `<hp:chart numberingType="TABLE" .../>`를 파싱해 동일하게 `Table`임을 단언
  (차트는 내부적으로 `OleShape`로 모델링됨)

수정 전 코드(두 곳 모두 되돌린 상태)로 실행한 결과:

```
running 2 tests
test issue2882_chart_numbering_type_table_is_parsed_into_common_field ... FAILED
test issue2882_ole_numbering_type_picture_is_parsed_into_common_field ... FAILED

  left: None
 right: Table   (또는 Picture)

test result: FAILED. 0 passed; 2 failed
```

수정 적용 후 재실행:

```
running 2 tests
test issue2882_ole_numbering_type_picture_is_parsed_into_common_field ... ok
test issue2882_chart_numbering_type_table_is_parsed_into_common_field ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2522 filtered out; finished in 0.00s
```

red → green을 로컬에서 직접 확인했다.

## 검증 (디스크 제약으로 경량 검증만 수행)

- `cargo check --lib` 통과
- 위 테스트 2건 `cargo test --lib issue2882`로 실행, 통과(red 확인 1회 + green
  확인 2회)
- 전체 `cargo test`, `cargo build --lib`, `cargo clippy --profile release-test`는
  로컬 디스크 여유 공간 제약으로 스킵
- `rustfmt --edition 2021 src/parser/hwpx/section.rs` 적용, `git diff --name-only`로
  의도한 파일만 변경됨을 확인

## 변경 파일

- `src/parser/hwpx/section.rs`
