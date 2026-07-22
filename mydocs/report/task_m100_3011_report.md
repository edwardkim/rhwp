# Task m100-3011: 문단 번호 모양(numFormat) CIRCLE_DIGIT 오탈자 호환 누락 수정

## 이슈

- https://github.com/edwardkim/rhwp/issues/3011
- 관련 선행 이슈: #2957/#2964(`autoNumFormat`), #3005/#3007(`pageNum formatType`) — 동일한
  `hc:NumberType1` 스키마를 참조하는 3개 지점 중 앞선 2곳에서 발견된 패턴.

## 원인

`hc:NumberType1`(OWPML `Core XML schema.xml` 5행)은 `autoNumFormat`, `pageNum formatType`,
`hh:paraHead numFormat` 세 곳에서 참조되는 공유 enum이다. 앞의 두 지점(`src/parser/hwpx/section.rs`)은
스펙 철자 `CIRCLED_DIGIT`과 한컴 실물 파일의 오탈자 `CIRCLE_DIGIT`(D 없음)를 모두 인식하도록
처리되어 있었으나, 세 번째 지점인 `src/parser/hwpx/header.rs`의 `parse_numbering_format_code`
(문단 번호 모양 파싱)에는 이 호환 별칭이 누락되어 있었다. 그 결과 `numFormat="CIRCLE_DIGIT"`로
저장된 문서를 읽으면 원문자(circled digit) 서식이 `DIGIT`(0)으로 유실된다.

## 수정

`src/parser/hwpx/header.rs`의 `parse_numbering_format_code` 매치암을 확장:

```rust
"CIRCLED_DIGIT" | "CIRCLE_DIGIT" => 1,
```

## 테스트

- 추가: `test_parse_hwpx_numbering_para_head_accepts_circle_digit_typo_for_hancom_compat`
  (수정 전 실패 → 수정 후 통과 확인)
- `cargo check --lib` 통과
- `rustfmt --edition 2021 src/parser/hwpx/header.rs` 적용

## 범위

`hc:NumberType1` 참조 3개 지점(`autoNumFormat`, `pageNum formatType`, `paraHead numFormat`) 모두
`CIRCLE_DIGIT` 호환 별칭 처리가 일치하게 되어, 이 클래스의 버그는 닫힌 것으로 판단한다.
