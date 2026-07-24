# Task m100-2828: HWPX bullet paraHead 라운드트립 소실 수정

## 이슈

https://github.com/edwardkim/rhwp/issues/2828

## 근거

`src/parser/hwpx/header.rs` 의 `parse_bullet_hwpx` (수정 전)는 `<hh:bullet>` 의
`char`/`useImage` 두 속성만 읽고, 자식 `<hh:paraHead>` (align/useInstWidth/
autoIndent/widthAdjust/textOffsetType/textOffset/numFormat/charPrIDRef/checkable
9개 속성 보유, HWP5 BULLET record 표 44 문단 머리 정보 12바이트에 대응)를 통째로
skip 했다. 그 결과 `src/serializer/hwpx/header.rs` 의 `write_bullet` (수정 전)은
원본 값과 무관하게

```rust
("align", "LEFT"),
("useInstWidth", "0"),
("autoIndent", "1"),
("widthAdjust", &b.width_adjust.to_string()), // 파서가 안 채우므로 항상 "0"
("textOffsetType", "PERCENT"),
("textOffset", "50"),                          // 하드코딩
("numFormat", "DIGIT"),
("charPrIDRef", &u32::MAX.to_string()),         // 하드코딩
("checkable", "0"),
```

를 방출했다. 동일 클래스 결함이 `numbering`(`<hh:numbering>`/`<hh:paraHead>`)에서는
이미 커밋 `3f564107`로 `raw_para_heads` 원본 구간 splice 방식으로 해소됐으나,
`bullet` 에는 이 패턴이 적용돼 있지 않았다.

`Bullet` 모델의 `attr`/`width_adjust`/`text_distance`/`char_shape_id` 필드는
HWP5 바이너리 `parse_bullet`(`src/parser/doc_info.rs:825`)이 파싱하는 12바이트
문단 머리 정보와 동일 의미이지만, HWPX 경로는 이 필드들을 전혀 채우지 않았다.

## 재현 (수정 전, red)

```rust
let xml = r##"<hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head">
  <hh:refList><hh:bullets itemCnt="1">
    <hh:bullet id="1" char="❏" useImage="0">
      <hh:paraHead level="0" align="CENTER" useInstWidth="1" autoIndent="0"
        widthAdjust="120" textOffsetType="PERCENT" textOffset="30"
        numFormat="DIGIT" charPrIDRef="9" checkable="1"/>
    </hh:bullet>
  </hh:bullets></hh:refList>
</hh:head>"##;
let (doc_info, _) = parse_hwpx_header(xml).unwrap();
// 수정 전: bullets[0].width_adjust == 0, char_shape_id == 0 (유실)
// write_bullet 재방출 시 align="CENTER"/useInstWidth="1" 등이 "LEFT"/"0" 으로 덮어써짐
```

## 수정 내용

`numbering.raw_para_heads` 와 동일 패턴으로 `Bullet` 에
`raw_para_head: Option<String>` 필드를 추가:

1. `src/model/style.rs` — `Bullet::raw_para_head` 필드 추가.
2. `src/parser/hwpx/header.rs` — `parse_bullet_hwpx` 가 `xml: &str` 를 추가로 받아
   `<hh:paraHead>` 여는/닫는 태그 사이 원본 구간을 byte-exact 로 캡처해
   `bullet.raw_para_head` 에 저장. 동시에 `apply_bullet_para_head_attrs` 로
   `widthAdjust`/`textOffset`/`charPrIDRef` 를 `Bullet::width_adjust`/
   `text_distance`/`char_shape_id` 필드로도 흡수(HWP5 경로 폴백용).
3. `src/serializer/hwpx/header.rs` — `write_bullet` 이 `raw_para_head` 가 있으면
   원본 그대로 splice, 없으면(HWP5→HWPX 경로 등) 필드값(`width_adjust`/
   `text_distance`/`char_shape_id`)으로 채운 뼈대로 폴백.
4. `src/parser/doc_info.rs`, `src/serializer/doc_info/tests.rs` — 새 필드 추가로 인한
   `Bullet` 구조체 리터럴 2곳에 `raw_para_head: None` 보강(HWP5 바이너리 경로는
   HWPX paraHead 원본이 없으므로 `None`이 올바름).

## 검증 (green)

새 테스트 `bullet_para_head_align_useinstwidth_survive_roundtrip_via_raw_splice`
(`src/parser/hwpx/header.rs`)가 위 재현 XML로:

- `bullet.raw_para_head` 가 원본 `<hh:paraHead .../>` 구간과 byte-exact 일치
- `width_adjust == 120`, `text_distance == 30`, `char_shape_id == 9`

를 확인한다.

```
test parser::hwpx::header::tests::bullet_para_head_align_useinstwidth_survive_roundtrip_via_raw_splice ... ok
test serializer::doc_info::tests::test_serialize_bullet_layout_and_roundtrip ... ok
test renderer::pua_oldhangul::tests::test_no_collision_with_pua_bullet_range ... ok
test renderer::layout::tests::test_square_bullet_with_space_preserves_layout ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2486 filtered out
```

`cargo build --lib` 통과, `cargo clippy --all-targets --profile release-test -- -D warnings`
경고 0건, `rustfmt --edition 2021 --check` 통과.

## 잔여 범위

`checkable`/`align`/`useInstWidth`/`autoIndent`/`textOffsetType` 는 `Bullet` 의
7수준 전용 필드로는 표현하지 않고 `raw_para_head` 원본 splice로만 보존한다
(numbering 과 동일한 설계 선택). 여러 `<hh:paraHead>`/`<hh:img>` 자식을 동시에 갖는
케이스도 splice 구간에 전부 포함되므로 별도 처리 불필요.
