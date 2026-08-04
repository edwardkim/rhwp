# Task #2872 처리 결과 — TableOfContents 필드 type 값 라운드트립 소실 수정

## 이슈

https://github.com/edwardkim/rhwp/issues/2872

HWPX `<hp:fieldBegin type="...">`를 파싱할 때 `parse_field_type()`
(`src/parser/hwpx/section.rs:4400`)은 `"TABLE_OF_CONTENTS"` / `"TABLEOFCONTENTS"`를
`FieldType::TableOfContents`로 인식하지만, 직렬화기 `field_type_str()`
(`src/serializer/hwpx/field.rs`)은 이를 `"TOC"`로 emit해왔다. `"TOC"`는 파서의 어느 분기와도
매치되지 않아 `_ => FieldType::Unknown`으로 떨어진다. 즉 HWPX 저장→재로드 1회 왕복만으로
목차 필드 타입이 소실된다.

같은 파일 안에 `ClickHere`(#1595), `DocDate`/`UserInfo`/`Summary`(OWPML 스키마 표기 정합화)에
대해 동일 계열 버그가 이미 수정된 이력이 있었고, `TableOfContents` 분기만 그 정리에서 누락되어
있었다.

## 원인

`src/serializer/hwpx/field.rs`의 `field_type_str()`:

```rust
TableOfContents => "TOC",
```

파서가 인식하는 값과 불일치.

## 수정

```rust
TableOfContents => "TABLE_OF_CONTENTS",
```

파서가 실제로 받아들이는 두 표기(`TABLE_OF_CONTENTS`/`TABLEOFCONTENTS`) 중 하나로 emit하도록
정합화.

## 테스트 (red → green)

- 기존 `field_type_str_covers_main_variants` 테스트가 버그 값(`"TOC"`)을 그대로 검증하고
  있어 이를 올바른 기대값(`"TABLE_OF_CONTENTS"`)으로 교체.
- 신규 테스트 `field_type_str_toc_round_trips_through_hwpx_parser` 추가:
  수정 전 코드에서는 `field_type_str(FieldType::TableOfContents)`가 `"TOC"`를 반환해
  실패(red), 수정 후에는 `"TABLE_OF_CONTENTS"`를 반환해 통과(green).

## 검증

- `cargo build --lib` — 통과
- `cargo test --lib field_type_str` — 3개 테스트 통과
  (`field_type_str_covers_main_variants`, `field_type_str_matches_owpml_schema`,
  `field_type_str_toc_round_trips_through_hwpx_parser`)
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음
- `rustfmt --edition 2021 src/serializer/hwpx/field.rs` — 적용

## 변경 파일

- `src/serializer/hwpx/field.rs` (수정)
- `mydocs/report/task_m100_2872_report.md` (신규, 본 문서)
