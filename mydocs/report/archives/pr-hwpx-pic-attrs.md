# hp:pic 저장 시 numberingType·groupLevel 속성 유실 수정

- **Issue**: #2745
- **Branch**: `pr/fix-issue-2745-hwpx-pic-attrs`
- **Type**: fix
- **Component**: serializer/hwpx, parser/hwpx

## 배경

HWPX 그림(`<hp:pic>`) 직렬화에서 `numberingType`과 `groupLevel` 두 속성이 IR 값을 사용하지 않고 하드코딩되어 방출되어, 저장 시 원본 속성이 유실되는 문제가 있었다. 도형(`<hp:rect>`, `<hp:line>`) 및 표(`<hp:tbl>`)는 이미 #1379/#2697에서 IR 보존으로 정리되었으나 그림 경로는 누락되었다.

## 수정 내용

### 1. Parser: `numberingType` 속성 파싱 추가

**파일**: `src/parser/hwpx/section.rs` - `parse_picture` 함수

기존 `parse_picture`는 `<hp:pic>` 요소의 `numberingType` 속성을 파싱하지 않아 IR의 `common.numbering_type`이 항상 기본값(`None`)으로 남았다. 도형/표 파서와 동일한 매핑 로직을 추가하여 `PICTURE`/`TABLE`/`EQUATION`/`NONE` 값을 IR에 저장한다.

```rust
b"numberingType" => {
    common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
        "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
        "TABLE" => crate::model::shape::ObjectNumberingType::Table,
        "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
        _ => crate::model::shape::ObjectNumberingType::None,
    };
}
```

### 2. Serializer: IR 기반 속성 방출

**파일**: `src/serializer/hwpx/picture.rs` - `write_picture` 함수

- `numberingType`: 하드코딩 `"PICTURE"` → `numbering_type_str(pic.common.numbering_type)` (shape.rs의 기존 헬퍼 함수 사용)
- `groupLevel`: 하드코딩 `"0"` → `pic.shape_attr.group_level.to_string()` (도형 rect/line 경로와 동일)
- `numbering_type_str` 함수 import 추가

## 영향

- `groupLevel` 속성: 그룹(`<hp:container>`) 내 그림 개체의 중첩 레벨이 저장 시 보존된다.
- `numberingType` 속성: 캡션 번호 범주 설정(NONE/PICTURE/TABLE/EQUATION)이 저장 시 보존된다.
- roundtrip 충실도 향상: 이전에는 저장 후 재파싱 시 두 속성이 항상 기본값으로 되돌아갔다.
- 신규 문서(Default::default)는 `common.numbering_type=ObjectNumberingType::None`이므로 `numberingType="NONE"`으로 방출되어, 종전 `"PICTURE"` 하드코딩과 차이가 있다. 이는 올바른 동작이다(도형 경로와 일관됨).
