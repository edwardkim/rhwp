# Task #3147 처리 결과

## 문제

`src/parser/hwp3/mod.rs`의 `parse_simple_control_char` 메일머지 표시(ch==22,
spec §10.16 표 57) arm 이 필드 이름을 `buf[2..22]`에서 읽었다. 여는 코드
(파일 오프셋 0..2)는 문자 스캔 루프에서 이미 소비되므로 추가로 읽는 22바이트
버퍼에서 필드 이름은 `buf[0..20]`, 닫는 코드(0x0016)는 `buf[20..22]`다.
종전 코드는 이름 앞 2바이트를 유실하고 닫는 코드를 이름 뒤에 혼입시켰다.

바이트 소비량(24바이트 = 12 hchar)은 스펙과 일치해 스트림 정렬은 정상이며,
IR `Field.command` 내용만 훼손되는 결함이다.

## 수정

`buf[2..22]` → `buf[0..20]` 1줄 수정 (스펙 근거 주석 추가).

## 검증 (red → green)

- 합성 문단(문단 정보 43바이트 + ch=22 컨트롤 24바이트, 필드 이름
  `MERGEFIELD` 0 패딩)으로 `parse_paragraph_list`를 호출하는 회귀 테스트
  `hwp3_mail_merge_field_name_starts_at_offset_zero` 추가.
- 수정 전(red): `left: "RGEFIELD" / right: "MERGEFIELD"` FAIL 확인.
- 수정 후(green): 해당 테스트 PASS, `cargo test --lib` 전체 2553건 통과,
  `cargo fmt --check`·`cargo clippy --lib` 무경고.

## 범위

메일머지 arm 의 이름 오프셋만 수정. 같은 arm 의 바이트 소비량·hchar 카운트는
스펙과 일치해 무변경. 글자겹침(ch=23) 겹칠 글자 미추출은 별도 이슈(#3148)로
분리.
