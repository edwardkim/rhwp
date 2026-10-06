# Task #3016 처리 결과 — HWP3 CharShape "글꼴에 어울리는 빈칸 사용" 매핑 누락 수정

## 이슈

[#3016](https://github.com/edwardkim/rhwp/issues/3016)

HWP3 `Hwp3CharShape` attr 바이트의 bit 0x80("글꼴에 어울리는 빈칸 사용")을
읽는 접근자 `is_font_blank()`가 `src/parser/hwp3/records.rs`에 이미
존재했지만, `src/parser/hwp3/mod.rs`의 `convert_char_shape()`가 이를 호출하지
않아 공통 IR `CharShape.use_font_space` 필드가 HWP3 파서 경로에서 항상
`false`로 고정되는 문제.

## 원인

`convert_char_shape()`는 이탤릭/볼드/밑줄/외곽선/그림자를 모두 IR로
매핑하면서, attr의 마지막 비트(0x80, `is_font_blank()`)에 대한 매핑 한 줄만
빠져 있었다. "접근자는 존재하지만 호출부가 없어 IR 필드가 항상 기본값으로
고정되는" 패턴이다.

## 수정

`src/parser/hwp3/mod.rs`의 `convert_char_shape()`에 한 줄 추가:

```rust
cs.use_font_space = hwp3_cs.is_font_blank();
```

그리고 attr=0x80 케이스를 검증하는 유닛 테스트
`convert_char_shape_maps_font_blank`를 추가했다.

## 검증

- `cargo check --lib` — 통과 (경고 없음).
- `cargo test --lib convert_char_shape` — 관련 테스트 모두 통과
  (`convert_char_shape_maps_font_blank` 포함).
- `rustfmt --edition 2021 src/parser/hwp3/mod.rs` — 적용 (변경 없음).

## 변경 범위

`src/parser/hwp3/mod.rs` 1개 파일 (구현 3줄 + 테스트 8줄). 다른 파일은
건드리지 않았다.
