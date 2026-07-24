# Task #3148 처리 결과

## 문제

`src/parser/hwp3/mod.rs`의 `parse_simple_control_char` 글자겹침(ch==23,
spec §10.17 표 58) arm 이 추가로 읽은 8바이트 버퍼를 전혀 해석하지 않고
`CharOverlap::default()`(빈 `chars`)를 IR 에 push 했다. 스펙상 버퍼의
`[0..6]`은 겹칠 글자 hchar array[3](최대 3자, 남는 부분 0 패딩),
`[6..8]`은 닫는 코드다. 바이트 소비량(10바이트 = 5 hchar)은 정확해 스트림은
어긋나지 않지만, 겹침 대상 글자가 전량 유실됐다.

IR `CharOverlap.chars`는 HWP5 직렬화(`serialize_char_overlap`)·HWPX 경로가
이미 소비하는 필드이므로 파서만 채우면 전 경로가 연결된다.

## 수정

ch==23 arm 에서 `buf[0..6]`의 hchar 3개를 LE 로 읽어 0 이 아닌 값만
`johab::decode_johab`로 디코딩해 `overlap.chars`에 push (스펙 근거 주석 포함).

## 검증 (red → green)

- 합성 문단(문단 정보 43바이트 + ch=23 컨트롤 10바이트: 겹칠 글자
  'A','B',0 패딩)으로 `parse_paragraph_list`를 호출하는 회귀 테스트
  `hwp3_char_overlap_extracts_overlap_chars` 추가.
- 수정 전(red): `left: [] / right: ['A', 'B']` FAIL 확인.
- 수정 후(green): 해당 테스트 PASS, `cargo test --lib` 전체 2552건 통과,
  `cargo clippy --lib` 무경고 (fmt --check 는 Windows CRLF 체크아웃 노이즈만).

## 범위

겹칠 글자 추출만 수정. 바이트 소비량·hchar 카운트는 스펙과 일치해 무변경.
테두리 종류 등 표 58 외 확장 속성은 HWP3 스펙에 없어 범위 밖. 메일머지
(ch=22) 이름 오프셋 어긋남은 별도 이슈(#3147)로 분리.
