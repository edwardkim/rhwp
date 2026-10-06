# Task m100-2884: HWPX fieldBegin/fieldEnd `fieldid` 속성 왕복 손실 수정

## 이슈

edwardkim/rhwp#2884

## 문제

`<hp:fieldBegin>` 의 `fieldid` 속성(문서 내 고유 `id` 와 별개인 필드 인스턴스 ID)을
파서가 `field_id` 계산의 폴백(`id_attr.or(fieldid_attr)`)으로만 사용하고 별도로
보존하지 않았다. `id` 속성이 거의 항상 존재하므로 `fieldid_attr` 값은 사실상 항상
버려졌고, 직렬화기(`field_begin_open_tag`, `write_field_begin`, `emit_field_end` →
`write_field_end`)는 애초에 `fieldid` 속성을 방출하는 코드 경로 자체가 없었다.

기존 테스트 픽스처(`task1556_multipara_field_parse_serialize_parse_roundtrip`)가
`id="1878228493"` / `fieldid="627272811"` 처럼 두 값이 서로 다른 실물 사례를 이미
담고 있었는데도, 이 값의 재직렬화 보존은 그동안 한 번도 검증되지 않았다.

#2830 (MEMO subList `textDirection` 하드코딩 수정)과 동일한 패턴 — 파서 미독해 +
직렬화기 하드코딩/누락 조합 — 이다.

## 수정

1. `src/model/control.rs`: `Field` 구조체에 `instance_id: Option<u32>` 필드 추가.
   원본 `fieldid` 속성 값을 `field_id` 계산과 무관하게 별도 보존.
2. `src/parser/hwpx/section.rs`: `parse_field_begin_attrs` 에서
   `f.instance_id = fieldid_attr;` 로 원본 값을 그대로 저장 (기존 `field_id` 폴백
   로직은 그대로 유지 — 필드 고유 ID 계약(#1512)에 영향 없음).
3. `src/serializer/hwpx/field.rs`:
   - `field_begin_open_tag` — `instance_id` 가 `Some` 이면 `fieldid="{}"` 속성을
     추가 방출, `None` 이면 기존과 동일하게 생략(자기닫힘 태그 케이스 호환).
   - `write_field_begin` — 동일한 분기 추가.
4. `src/serializer/hwpx/section.rs`: `emit_field_end` 가 `write_field_end` 대신
   `write_field_end_full(w, f.field_id, f.instance_id.unwrap_or(0))` 을 사용하도록
   변경 — 짝이 되는 `fieldEnd` 에도 `fieldid` 가 있으면 동일하게 반영, 0 이면 속성
   생략(#1556 이 정의한 기존 규약 재사용).
5. `Field` 구조체 리터럴을 명시적으로 나열하던 4개 호출부
   (`src/document_core/queries/field_query.rs` 2곳, `src/parser/control.rs` 1곳)에
   `instance_id: None,` 추가.

## 테스트 (red → green)

`src/serializer/hwpx/field.rs::tests::field_begin_preserves_distinct_fieldid_attr`

- `field_id = 1_878_228_493`, `instance_id = Some(627_272_811)` (실물 fixture 값)인
  `Field` 로 `write_field_begin` 호출.
- 수정 전: 출력 XML 에 `fieldid` 속성이 전혀 없어 `assert!(xml.contains(r#"fieldid="627272811""#))` 실패(red).
- 수정 후: `fieldid="627272811"` 이 방출되어 통과(green).

```
test serializer::hwpx::field::tests::field_begin_preserves_distinct_fieldid_attr ... ok
```

## 검증

- `cargo build --lib` — 통과
- `cargo test --lib field_begin_preserves_distinct_fieldid_attr` — 통과 (1 passed)
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음
- `rustfmt --edition 2021` — 변경 파일만 적용

## 변경 파일

- `src/model/control.rs`
- `src/parser/hwpx/section.rs`
- `src/parser/control.rs`
- `src/serializer/hwpx/field.rs`
- `src/serializer/hwpx/section.rs`
- `src/document_core/queries/field_query.rs`
