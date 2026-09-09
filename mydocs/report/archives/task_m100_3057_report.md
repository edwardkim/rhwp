# Task #3057 처리 결과

## 문제

`src/parser/hwp3/mod.rs`의 `parse_hwp3_object_dispatch` 중 HWP3 필드 코드
컨트롤(ch==5, spec §10.1 표33) 분기는 `field_data`를 읽기만 하고 버렸다.
같은 함수의 책갈피(ch==6) 분기는 파싱 결과를 `Control::Field`로 배선하는데,
필드 코드는 상위 함수 `parse_object_control_char`의 catch-all에 의해
`Control::Unknown`으로만 남아 원본 데이터가 소실됐다.

## 수정

`src/parser/hwp3/mod.rs` ch==5 분기에서 책갈피와 동일한 패턴으로
`field_data`를 디코딩해 `Control::Field`(command에 원본 문자열)로 배선.

## 검증

- 합성 바이트(헤더 8바이트 + payload)로 `parse_object_control_char`를
  직접 호출해 `Control::Field`가 생성되는지 검증하는 단위 테스트 추가.
- `cargo check --lib`, `cargo test --lib field_code_ch5` 통과.

## 범위

책갈피/상호참조는 이미 정상 배선되어 있음을 재확인. 필드 코드 세부
subcommand 파싱(spec 상세 포맷)은 범위 밖 — 원본 바이트를 보존하는
최소 수정만 수행.
