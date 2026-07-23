# Task m100 #3147 처리결과 — HWP3 메일머지 표시(ch=22) 필드 이름 오프셋 +2 어긋남 수정

## 대상 이슈

- #3147 — HWP3 메일머지 표시(ch=22) 필드 이름이 오프셋 +2로 어긋나게 읽혀 앞 2바이트 유실

## 원인 분석

`src/parser/hwp3/mod.rs` `parse_simple_control_char`의 `22 =>` arm이 필드 이름을
`buf[2..22]`에서 읽었다. 스펙(한글문서파일구조 3.0 §10.16 표 57) 기준 필드 이름은
파일 오프셋 2..22(kchar array[20])인데, 여는 특수 문자 코드(오프셋 0..2)는 문자
스캔 루프에서 이미 소비되므로 이어 읽는 22바이트 버퍼에서 이름은 `buf[0..20]`,
닫는 코드는 `buf[20..22]`다. 종전 코드는:

- 필드 이름 앞 2바이트 유실 (`MERGEFIELD` → `RGEFIELD`)
- 이름이 정확히 20바이트인 경우 닫는 코드 0x0016이 이름 뒤에 혼입

바이트 소비량(24바이트 = 12 hchar)은 스펙과 일치해 스트림 자체는 어긋나지 않는다.

## 수정 내용

- `src/parser/hwp3/mod.rs` — `let name_buf = &buf[2..22];` → `&buf[0..20]` (스펙 근거 주석 포함)

## red → green

재현 테스트 `parser::hwp3::tests::hwp3_mail_merge_field_name_starts_at_offset_zero`
(합성 22바이트 body: 이름 `MERGEFIELD` + 0 패딩 + 닫는 코드 0x0016):

- 수정 전 FAIL: `left: "RGEFIELD" / right: "MERGEFIELD"`
- 수정 후 PASS. 바이트 소비량(22) 불변 검증 포함.

## 검증

- `cargo test --profile release-test --lib -- --test-threads=1`: 2553 passed / 0 failed
  (병렬 실행 시 `renderer::font_paths::tests::env_font_paths_parses_and_filters` 1건이
  간헐 실패하나, 단독·단일 스레드 실행 모두 통과하는 기존 env-var 경합으로 본 수정과 무관)
- `cargo clippy --profile release-test --lib`: 경고 없음
- `rustfmt` (변경 파일) 적용
