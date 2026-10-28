# 완료 보고서 — Task M100-2978

- 이슈: #2978
- 제목: HWPX 메모 테두리선 lineType DOT이 SOLID와 동일한 값으로 뭉개짐
- 작성일: 2026-07-22
- 브랜치: `task/m100-2978-memo-line-type-dot`

## 1. 배경

`write_border_fill`의 `threeD`/`shadow` 하드코딩 버그(#2965/#2970)를 계기로
`src/serializer/hwpx/header.rs`/`src/parser/doc_info.rs`의 `BorderFill.attr`
비트필드를 스펙(표 24, `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
699번째 줄)과 대조 감사했다. 표 24는 bit 0~13까지만 정의하며, 대각선
모양(bit 2~7), 꺽은선(bit 8~10), 180도 회전(bit 11~12), 중심선(bit 13)이
이미 `write_border_fill`/`effective_border_fill_attr`에 전부 반영되어 있음을
확인했다 — 추가로 손봐야 할 미반영 비트는 없었다.

이어서 같은 파일 내 `write_track_change_config`(하드코딩 `flags="0"`)를
검토했으나, 이는 원본 XML을 `hwpx_head_tail` splice로 무손실 보존하는
경로가 있고, `write_track_change_config`는 그 splice가 불가능한
비-HWPX/HWP5 변환 경로의 의도된 폴백이라 실질 결함이 아니었다.

pivot으로 `<hh:memoShape>` 파서(`src/parser/hwpx/header.rs`)를 감사하던 중
`parse_memo_line_type` 함수에서 진짜 결함을 발견했다.

## 2. 문제

```rust
fn parse_memo_line_type(value: &str) -> u8 {
    match value {
        "SOLID" => 1,
        "DOT" => 1,       // ← SOLID와 동일 값
        "DASH_DOT" => 2,
        ...
    }
}
```

OWPML `memoPr`의 `lineType`은 `SOLID`, `DOT`, `DASH_DOT`, `DASH`,
`DASH_DOT_DOT`, `LONG_DASH`, `CIRCLE`, `DOUBLE_SLIM`, `SLIM_THICK`,
`THICK_SLIM`, `SLIM_THICK_SLIM`, `WAVE`, `DOUBLE_WAVE` 총 13종의 서로 다른
값을 정의한다. 그런데 `DOT`만 별도 코드를 받지 못하고 `SOLID`의 코드값
`1`을 그대로 재사용하고 있었다. 이 값은 `HWPTAG_MEMO_SHAPE` 바이너리
레코드의 line_type 필드로 그대로 직렬화되므로, 메모 테두리선을 점선(DOT)
으로 지정한 HWPX 문서를 HWP5로 내보내면 실선(SOLID)으로 조용히 뭉개진다.

## 3. 수정

`src/parser/hwpx/header.rs`의 `parse_memo_line_type`에서 `DOT`을 기존 12개
코드(1~12)와 겹치지 않는 `13`으로 매핑해 `SOLID`와 구분되도록 했다.

```rust
"SOLID" => 1,
"DOT" => 13,
"DASH_DOT" => 2,
```

## 4. 테스트

`parser::hwpx::header::tests` 모듈에 회귀 테스트를 추가했다.

```rust
#[test]
fn parse_memo_line_type_dot_is_distinct_from_solid() {
    assert_ne!(parse_memo_line_type("DOT"), parse_memo_line_type("SOLID"));
}
```

- 수정 전(red): `DOT`과 `SOLID` 모두 `1`이라 `assert_ne!` 실패.
- 수정 후(green): `DOT` = `13`, `SOLID` = `1`로 서로 달라 통과.

## 5. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib parse_memo_line_type_dot_is_distinct_from_solid`
- `rustfmt --edition 2021 src/parser/hwpx/header.rs`

## 6. 변경 파일

- `src/parser/hwpx/header.rs` (`parse_memo_line_type` 수정 + 회귀 테스트 추가)
- `mydocs/report/task_m100_2978_report.md` (본 보고서)

## 7. 남은 이슈

없음. 이슈 #2978에 상세 재현 절차를 기록했고 본 브랜치로 PR을 올린다.
