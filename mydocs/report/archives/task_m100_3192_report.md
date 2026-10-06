# Task M100 #3192 구현 보고서

Issue: #3192

## 목표

HWP5(.hwp) → HWPX 저장 경로에서 문단 번호(numbering) `paraHead` 폴백 스켈레톤이
`textOffset`/`charPrIDRef` 를 하드코딩해 원본 값을 유실하는 문제를 수정한다.

## 원인

- `src/serializer/hwpx/header.rs::write_numbering()` 은 원본 HWPX `raw_para_heads` splice가
  없을 때(HWP5 경유 등) 10개 레벨에 대해 하드코딩 뼈대를 방출한다.
- 이 폴백이 `("textOffset", "50")`, `("charPrIDRef", &u32::MAX.to_string())` 를 무조건
  써서, `NumberingHead.text_distance`/`char_shape_id` 값(HWP5 DocInfo 파서가 표 43 규격
  12바이트 레코드에서 실제로 채운 값, `src/parser/doc_info.rs`)을 전혀 참조하지 않았다.
- 같은 파일의 대응 함수 `write_bullet()` 은 이미 `b.text_distance`/`b.char_shape_id` 를
  값으로 채워 방출하고 있어(989~993번째 줄 부근), numbering 쪽만 비대칭적으로
  하드코딩되어 있었다.
- `numFormat="DIGIT"` 하드코딩은 별도 이슈(#2947/#3097)에서 이미 수정되었다 —
  이번 수정은 그 외 나머지 속성(textOffset, charPrIDRef)에 한정한다.

## 변경

- `write_numbering()` 폴백 스켈레톤에서 `textOffset` 을 `h.text_distance`, `charPrIDRef` 를
  `h.char_shape_id` 값으로 방출하도록 수정. `numFormat="DIGIT"` 하드코딩은 이번 이슈 범위
  밖이라 손대지 않았다.

## 검증

1. Red: `write_numbering_skeleton_preserves_text_distance_and_char_shape_id` 테스트를
   먼저 작성 — `NumberingHead.text_distance = 130`, `char_shape_id = 7` 을 넣고 직렬화한
   결과에 `textOffset="130"`, `charPrIDRef="7"` 이 있는지 확인. 수정 전 코드에서 FAIL
   확인(실제 출력은 `textOffset="50"`, `charPrIDRef="4294967295"`).
2. 최소 수정 적용 후 같은 테스트 PASS.
3. 기존 `write_numbering_falls_back_to_skeleton_when_no_raw`,
   `write_numbering_splices_raw_para_heads_verbatim` 등 주변 테스트 회귀 없음.
4. `RUSTFLAGS="-C linker=rust-lld" cargo test --lib` 전체 스윕: 2554 passed, 0 failed,
   7 ignored.

## 참고

- 이 PC 환경은 Norton TLS 검사로 인한 dbghelp 링커 오류가 있어
  `RUSTFLAGS="-C linker=rust-lld"` 로 우회했다(기존 알려진 이슈, 코드와 무관).
