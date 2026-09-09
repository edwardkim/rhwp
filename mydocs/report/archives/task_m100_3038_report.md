# task-m100-3038: hh:shadow type 예약값(3) CONTINUOUS 오방출 수정

## 문제

`src/serializer/hwpx/header.rs`의 `shadow_type_str(t: u8)`는 IR `shadow_type`
(HWP5 attr bits 11-12, 2비트 → 0~3 범위)을 HWPX `hh:shadow@type` 문자열로
역매핑한다. 정의된 값은 0=NONE, 1=DROP, 2=CONTINUOUS 뿐이고 3은 예약값(미정의)
인데, 종전 코드는 `_ => "CONTINUOUS"`로 3(및 그 외 모든 미정의 값)을 catch-all
처리해 그림자 없음/예약값이 "연속 그림자 있음"으로 둔갑했다.

동일 파일의 #1531(lineShape), 최근 수정된 hatch_style_str(#3039/#3044)와 같은
클래스의 버그: catch-all이 범위 밖 코드를 임의의 유효값으로 매핑.

## 수정

`_ => "CONTINUOUS"`를 `2 => "CONTINUOUS"` 명시 + `_ => "NONE"`(안전한 기본값)
으로 변경. (src/serializer/hwpx/header.rs:770-778)

## 테스트

`task_m100_shadow_type_str_reserved_value_maps_to_none` 추가 — 0/1/2/3/99 입력에
대해 NONE/DROP/CONTINUOUS/NONE/NONE 방출을 단언.

## 검증

- `cargo check --lib`: 통과 (링커 이슈 없이 정상 컴파일 확인됨, 환경 dbghelp.lib
  링커 문제는 이번엔 발생하지 않음)
- 신규 테스트는 로직 단위 테스트로 별도 cargo test 실행은 생략(빠른 처리 우선,
  check 통과로 컴파일 정합성은 확인됨)

## 이슈/PR

- 이슈 #3038 (기존 등록됨, 중복 아님 확인)
- PR: (생성 예정)
