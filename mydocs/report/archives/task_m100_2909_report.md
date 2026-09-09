# task_m100_2909 처리 결과 보고

## 이슈

#2909 — HWPX 같은 문단 내 짝(matched) `fieldEnd`(HYPERLINK 등)의 `fieldid` 속성이 직렬화 시
항상 소실됨

## 원인

`<hp:fieldBegin>`/`<hp:fieldEnd>` 필드가 같은 문단 안에서 짝을 이룰 때, 파서
(`src/parser/hwpx/section.rs` `parse_paragraph()`)는 `fieldEnd` 자신의 `fieldid` 속성값을
`parse_field_end_attrs()` 로 이미 읽어오지만, `field_stack.pop()`이 `Some`인(=같은 문단 매칭)
분기에서는 그 값을 어디에도 저장하지 않고 버렸다. `FieldRange`(`src/model/paragraph.rs`)에는
애초에 이 값을 담을 필드가 없었다.

직렬화기(`src/serializer/hwpx/section.rs` `emit_field_end()`)는 `write_field_end(w, f.field_id)`
만 호출해 `beginIDRef` 속성만 쓰고 `fieldid` 는 전혀 방출하지 않았다. 반면 문단 경계를 넘는
"고아(orphan) fieldEnd" 경로(`emit_orphan_field_end()`)는 `write_field_end_full()` 로 이미
`fieldid` 까지 보존하고 있어(#1556), 같은 성격의 값을 다루는 두 경로가 비대칭적이었다.
`<hp:pic lock>`(#2875), `<hp:tbl lock>`(#2855), `hp:equation lock`(#2840) 등에서 반복 확인된
"파싱은 되지만 직렬화 시 하드코딩/누락"류 결함과 동일 클래스다.

## 수정

1. `src/model/paragraph.rs`: `FieldRange`에 `pub end_field_id: u32` 필드 추가(짝 `fieldEnd`
   자신의 `fieldid`, 0이면 없음/생략). `split_at`/`merge_from` 등 기존 `FieldRange` 복제 지점
   에서 값을 그대로 전파하도록 수정.
2. `src/parser/hwpx/section.rs`: `parse_paragraph()`의 매칭(같은 문단) `field_ranges.push()`에
   `end_field_id: field_id`(파싱된 `fieldEnd` 자신의 `fieldid`) 반영.
3. `src/serializer/hwpx/section.rs`: `emit_field_end()` 시그니처를 `control_idx: usize` 대신
   `fr: &FieldRange` 를 받도록 변경. `fr.end_field_id == 0`이면 종전대로 `write_field_end`,
   아니면 `write_field_end_full(f.field_id, fr.end_field_id)`로 `fieldid` 를 되돌려 쓰도록 하여
   고아 경로와 대칭을 맞춤. 8개 호출부(`fr.control_idx` → `fr`)도 함께 수정.
4. 나머지 파일(`document_core/commands/document.rs`, `document_core/queries/field_query.rs`,
   `model/paragraph/tests.rs`, `parser/body_text.rs`, `serializer/body_text.rs`,
   `serializer/hwpx/mod.rs`)은 `FieldRange` 구조체 리터럴 생성부에 신규 필드를 반영하기 위한
   컴파일 정합성 수정(대부분 `..Default::default()` 또는 기존 `fr.end_field_id` 전파, HWP5
   경로는 개념이 없어 `0`).

## 테스트 (red → green)

`src/serializer/hwpx/section.rs::tests::bookmark_hyperlink_matched_field_end_preserves_own_fieldid`
신규 추가: `fieldBegin id=42`, 짝 `fieldEnd fieldid=100`(서로 다른 값)인 HYPERLINK 필드를
직렬화해 `<hp:fieldEnd beginIDRef="42" fieldid="100"/>` 가 나오는지 단언.

- Red: `emit_field_end()`를 임시로 종전 로직(`write_field_end(f.field_id)`만 호출)으로 되돌려
  실행 → 실패 확인(`fieldid` 누락된 XML로 assert 실패, 실제 출력 캡처함).
- Green: 수정 로직 복원 후 재실행 → 통과.

## 검증

- `cargo build --lib`: 성공
- `cargo test --lib`: 2511 passed, 0 failed (전체 lib 유닛 테스트), 관련 테스트
  `bookmark_hyperlink_matched_field_end_preserves_own_fieldid`: 1 passed
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 경고 없음
- `rustfmt --edition 2021`(변경 파일만): 적용, 기능적 diff 없음(CRLF 노이즈 파일은 원상 복구)

## 관련

- 이슈: https://github.com/edwardkim/rhwp/issues/2909
- 전례: #1556(고아 fieldEnd fieldid 보존), #2875(hp:pic lock), #2855(hp:tbl lock),
  #2840(hp:equation lock)
