# task-m100-3143: 글자겹침(tcps) 직렬화 미검증 캐스팅 방어

## 이슈

#3143 HWP5 글자겹침(CTRL_TCPS) 직렬화기의 미검증 캐스팅 2건:

1. **비BMP 문자 절단** — `ch as u16` 캐스팅으로 하위 16비트만 기록.
   파서(`parse_char_overlap`)는 서로게이트 쌍을 디코딩하므로 왕복이 깨진다.
2. **u8 카운트 wraparound** — charshape ID 카운트를 `len() as u8` 로 기록.
   HWPX `<hp:compose>` 는 `<hp:charPr>` 자식 수 제한이 없어 256개 이상 입력 시
   카운트 필드(256→0)와 실제 기록 데이터가 어긋난 손상 레코드가 만들어진다.

## 원인 분석

`src/serializer/control.rs` `serialize_char_overlap`:

- chars 를 `ch as u16` 으로 단순 캐스팅 → U+10000 이상에서 절단.
- 카운트 필드(u16/u8)에 상한 검사 없음 → wraparound 시 카운트-데이터 불일치.

## 수정 내용

- chars 를 `encode_utf16()` 으로 서로게이트 쌍 포함 인코딩 (파서와 대칭).
- 카운트 필드(u16/u8)를 타입 한계로 상한 절단하고, 기록 배열도 함께 절단해
  카운트와 실제 기록 데이터가 항상 일치하도록 방어 (손상 레코드 방지).

## red→green 결과

회귀 테스트 2건 추가 (`src/serializer/control/tests.rs`), 수정 전 FAIL 확인:

- `char_overlap_non_bmp_char_roundtrip` — U+1D400(𝐀) 왕복 시
  (수정 전: `비BMP 문자가 왕복 보존되어야 함` assert 실패 — 하위 16비트 문자로 변형)
- `char_overlap_256_char_shape_ids_no_wraparound` — ID 256개 입력 시
  (수정 전: `카운트 필드(0)와 실제 기록된 ID 바이트(1024)가 어긋남 — u8 wraparound`)

수정 후 2건 전부 PASS.

## 검증 명령

```
cargo test char_overlap
```

전량 PASS. `cargo fmt --check` 통과(후속 style 커밋 `5527c211` 로 정렬).

## 관련 코드

- `src/serializer/control.rs` — encode_utf16 인코딩·카운트 상한 절단 (+32/-4)
- `src/serializer/control/tests.rs` — 회귀 테스트 2건 (+59)
- 기준 커밋: upstream/devel `6f34e9b2`
