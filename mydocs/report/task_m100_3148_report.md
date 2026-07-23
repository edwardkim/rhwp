# Task m100 #3148 처리결과 — HWP3 글자겹침(ch=23) 겹칠 글자 array[3] 파싱 배선

## 대상 이슈

- #3148 — HWP3 글자겹침(ch=23)의 겹칠 글자(hchar array[3])가 파싱되지 않아 `CharOverlap.chars`가 항상 빈 Vec

## 원인 분석

`src/parser/hwp3/mod.rs` `parse_simple_control_char`의 `23 =>` arm이 8바이트 버퍼를
읽기만 하고 내용을 전혀 사용하지 않은 채 `CharOverlap::default()`(빈 `chars`)를
IR에 push했다. 스펙(한글문서파일구조 3.0 §10.17 표 58) 기준 글자겹침 10바이트 중
오프셋 2..8이 겹칠 글자 hchar array[3](남는 부분 0 채움)이며, 여는 코드 소비 후
읽는 8바이트 버퍼에서 `buf[0..6]`이 겹칠 글자, `buf[6..8]`이 닫는 코드다.

IR `CharOverlap.chars`는 HWP5 직렬화·HWPX 경로가 이미 소비하므로 파서 배선만으로
전 경로가 연결된다.

## 수정 내용

- `src/parser/hwp3/mod.rs` — `buf[0..6]`에서 hchar 3개를 LE로 읽어 0이 아닌 값만
  `johab::decode_johab`로 디코딩해 `overlap.chars`에 push (스펙 근거 주석 포함)

## red → green

재현 테스트 `parser::hwp3::tests::hwp3_char_overlap_extracts_overlap_chars`
(합성 8바이트 body: 'A', 'B', 0 + 닫는 코드 0x0017):

- 수정 전 FAIL: `left: [] / right: ['A', 'B']`
- 수정 후 PASS. 바이트 소비량(8) 불변 검증 포함.

## 검증

- `cargo test --profile release-test --lib -- --test-threads=1`: 2553 passed / 0 failed
- `cargo clippy --profile release-test --lib`: 경고 없음
- `rustfmt` (변경 파일) 적용

## 비고

#3147과 동일 함수(`parse_simple_control_char`) 인접 arm 수정이므로 메인테이너
지시(같은 파일 축은 한 PR로)에 따라 #3147과 한 PR로 묶어 제출.
