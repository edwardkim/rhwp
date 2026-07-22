# task/m100-2855 처리 결과 — 표(hp:tbl) lock 파싱/방출 누락 수정

## 이슈

#2855 — `<hp:tbl>` 의 `lock`(개체 잠금) 속성이 파서(`parse_table`)에 arm 자체가 없어
버려지고, 방출측(`src/serializer/hwpx/table.rs:62`)도 `bool01(false)` 리터럴을 그대로
방출해 IR을 전혀 참조하지 않았다. 결과적으로 잠긴 표를 왕복하면 잠금 상태가 소리 없이
풀린다.

`<hp:rect>`/`<hp:line>`/`<hp:pic>`/`<hp:container>` 는 `parse_object_element_attrs`
(공유 함수, `src/parser/hwpx/section.rs:2858`)를 통해 `lock`을 정상 파싱하지만, 표는
별도 파서(`parse_table`)를 쓰기 때문에 예외였다.

## 근거

- `src/parser/hwpx/section.rs:1600-1661` (`parse_table` 속성 매치) — `lock` arm 부재,
  `_ => {}` 로 조용히 버려짐.
- `src/serializer/hwpx/table.rs:62`(수정 전) — `let lock = bool01(false);` 리터럴.
- 대조: `<hp:equation>` 경로는 IR(`common.locked`)을 참조하도록 되어 있어(§#2840 계열
  패턴), 표만 그 패턴을 따르지 않음을 확인.

이번 작업 시점에 `CommonObjAttr.locked` 필드 자체가 `origin/devel`에 아직 없어(별도
진행 중인 수식 lock 브랜치에만 존재, 미병합), 표 수정을 위해 필드도 함께 추가했다.

## 재현 (RED)

`src/serializer/hwpx/table.rs` 에 `task2855_tbl_lock_survives_xml_ir_xml_roundtrip`
테스트 추가. `lock="1"` 을 가진 `<hp:tbl>` XML을 파싱 → `table.common.locked` 확인
→ 재직렬화 → `lock="1"` 이 방출되는지 확인. 필드 추가 전에는
`no field 'locked' on type CommonObjAttr` 컴파일 에러로 RED, 필드 추가 후에는
파싱 단계에서 `false`(기본값)로 남아 어서션 실패로 RED를 재현했다.

## 수정

1. `src/model/shape.rs` — `CommonObjAttr` 에 `pub locked: bool` 필드 추가.
2. `src/parser/hwpx/section.rs` (`parse_table`) — `b"lock" => table.common.locked =
   attr_str(&attr) == "1",` arm 추가.
3. `src/serializer/hwpx/table.rs:62` — `bool01(false)` → `bool01(table.common.locked)`.
4. `src/document_core/converters/common_obj_attr_writer.rs` — 테스트 헬퍼
   `make_sample()` 에 신규 필드 `locked: false` 추가 (컴파일 오류 해소, 기존 동작
   변경 없음).

## 검증 (GREEN)

- `cargo build --lib` — 성공.
- `cargo test --lib task2855_tbl_lock_survives_xml_ir_xml_roundtrip` — 1 passed.
- `cargo test --lib table::` — 기존 125개 표 관련 테스트 전부 통과(회귀 없음).
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음.
- `rustfmt --edition 2021` 변경 파일 4개 적용 — 포맷 외 diff 변화 없음.

## 영향 범위

`src/model/shape.rs`, `src/parser/hwpx/section.rs`, `src/serializer/hwpx/table.rs`,
`src/document_core/converters/common_obj_attr_writer.rs`. 표(`<hp:tbl>`) 경로에 한정된
수정이며 다른 개체 유형(`rect`/`line`/`pic`/`container`)의 `lock` 처리는 이미 정상
동작함을 별도로 확인했으므로 손대지 않았다.
