# 완료 보고서 — Task M100-3018

- 이슈: #3018
- 제목: 선(직선) 개체 파싱 시 방향 보정 플래그가 항상 소실됨 (INT32 대 UINT16 오독)
- 작성일: 2026-07-22
- 브랜치: `task/m100-3018-line-flag-u16`

## 1. 완료 내용

`src/parser/control/shape.rs`의 `parse_line_shape_data()`에서 "일반 선"
(is_connector == false) 분기가 방향 보정 플래그를 `read_i32()`(4바이트)로
읽던 것을 `read_u16()`(2바이트)로 수정했다.

한글문서파일형식 5.0 revision 1.3 표92("선 개체 속성")는 좌표 4개(4바이트×4)
뒤에 `UINT16` 속성 필드가 오며 전체 길이가 18바이트라고 명시한다. 기존
코드는 좌표 16바이트를 읽은 뒤 남은 2바이트뿐인 자리에서 4바이트를
요구했기 때문에 `ByteReader`가 매번 읽기에 실패해 `unwrap_or(0)`으로
`started_right_or_bottom` 값이 항상 `false`로 소실되고 있었다.

## 2. 주요 변경

- `src/parser/control/shape.rs`
  - `parse_line_shape_data()`: `read_i32()` → `read_u16()`, 스펙 근거 주석 추가
  - `task195_tests` 모듈에 회귀 테스트
    `line_started_right_or_bottom_parsed_from_18byte_record` 추가
    (18바이트 레코드에서 플래그(1)가 정확히 읽히는지 검증)

## 3. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib task195_tests::line_started_right_or_bottom_parsed_from_18byte_record`
  - 1 passed
- `rustfmt --edition 2021 src/parser/control/shape.rs`

## 4. 리스크

- 순수 직선(HWPTAG_SHAPE_COMPONENT_LINE, is_connector == false) 개체에만
  영향을 준다. 연결선(connector)과 사각형/타원/호/다각형/곡선 등 다른
  도형 파서는 이번 조사에서 스펙과 대조해 별도 불일치를 확인하지 못했다.
- 자체 시리얼라이저(`src/serializer/control.rs`)는 여전히 이 필드를
  `write_i32`(4바이트)로 기록한다. 이번 수정은 파서(읽기) 쪽만 스펙에
  맞췄으며, 값 자체는 0/1이라 리틀엔디안 하위 2바이트만 읽어도 왕복
  일관성은 유지된다(자체 저장 파일 재파싱 시 문제 없음). 시리얼라이저를
  18바이트 스펙에 맞추는 것은 별도 범위로 남겨둔다.

## 5. 결론

Task M100-3018 구현과 회귀 테스트를 완료했다. PR 생성 후 리뷰를 기다린다.
