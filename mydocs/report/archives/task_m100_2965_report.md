# Task m100-2965: HWPX borderFill threeD/shadow 라운드트립 소실 수정

## 이슈

https://github.com/edwardkim/rhwp/issues/2965

## 근거

HWP5 바이너리 BORDER_FILL 레코드(스펙 표 23/24)는 속성 UINT16 을 그대로
`BorderFill.attr` 필드에 보존한다(`src/parser/doc_info.rs:318`
`parse_border_fill`). 표 24 에 따르면:

- bit 0: 3D 효과 유무
- bit 1: 그림자 효과 유무
- bit 2~13: slash/backSlash 대각선 모양·꺾은선·회전·중심선

그런데 `src/serializer/hwpx/header.rs` 의 `write_border_fill` (수정 전)은
`<hh:borderFill>` 의 `threeD`/`shadow` 두 속성을 `attr` 비트를 읽지 않고 항상
상수 `"0"` 으로 방출했다.

```rust
// 수정 전
&[
    ("id", &(id + 1).to_string()),
    ("threeD", "0"),
    ("shadow", "0"),
    ("centerLine", center_line_type(bf)),
    ("breakCellSeparateLine", "0"),
]
```

같은 함수 안에서 대각선(bit 2~13)은 `attr` 에서 정확히 역매핑되고 있어
threeD/shadow(bit 0/1) 만 예외적으로 하드코딩되어 있었다. HWP5→HWPX 저장
경로에서 3D 테두리 또는 그림자 효과가 걸린 borderFill 을 저장하면 항상
`threeD="0" shadow="0"` 으로 덮어써져 효과가 소실된다.

동일 클래스 결함이 최근 numbering(`numFormat`, #2947), bullet paraHead(#2828),
style langID/lockForm(#2839) 에서 반복 발견·수정된 "파서는 실값을 필드에
보존하지만 직렬화기가 상수로 되돌린다" 패턴과 같다.

## 재현 (수정 전, red)

```rust
let mut bf = BorderFill::default();
bf.attr = 0b11; // bit0(3D)=1, bit1(shadow)=1
// write_border_fill 재방출 시:
// 수정 전: threeD="0" shadow="0" (attr 값 무시, 유실)
```

## 수정 내용

`src/serializer/hwpx/header.rs` `write_border_fill` — `effective_border_fill_attr(bf)`
로 얻은 유효 attr 값에서 bit0/bit1 을 읽어 `threeD`/`shadow` 속성을 방출하도록
변경. 기존 `bool01()` 헬퍼(같은 파일 내 이미 존재)를 재사용해 "1"/"0" 문자열로
변환.

## 검증 (green)

새 테스트 `write_border_fill_preserves_three_d_and_shadow_bits`
(`src/serializer/hwpx/header.rs`)가 `attr=0b11` 일 때 `threeD="1"`/`shadow="1"`
이 방출되는지 확인한다.

```
test serializer::hwpx::header::tests::write_border_fill_preserves_three_d_and_shadow_bits ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2522 filtered out; finished in 0.00s
```

`cargo check --lib` 통과. `rustfmt --edition 2021` 적용 완료.

## 잔여 범위

`breakCellSeparateLine` 은 스펙 표 24 범위 밖 별도 필드로, 이번 수정에서는
다루지 않는다(별도 조사 필요 시 후속 이슈로 분리).
