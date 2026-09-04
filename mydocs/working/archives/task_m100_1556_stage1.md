# 단계 1 완료보고서 — Task #1556

## 목표
고아(다단락) fieldEnd 를 IR 에 기록하도록 모델·파서 확장.

## 변경 사항

### `src/model/paragraph.rs`
- `OrphanFieldEnd { char_idx, begin_id_ref, field_id }` 구조체 추가.
- `Paragraph.orphan_field_ends: Vec<OrphanFieldEnd>` 필드 추가 (Default 빈 벡터).
- 문단 분할(`split_off` 류) 생성부 literal 에 `orphan_field_ends: Vec::new()` 보강.

### `src/parser/hwpx/section.rs`
- `parse_field_end_attrs(ce) -> (u32, u32)` 헬퍼 추가 — `beginIDRef`/`fieldid` 포착.
- `parse_ctrl` 시그니처에 `field_end_attrs: &mut Vec<(u32,u32)>` 추가.
  - Start/Empty 두 fieldEnd 분기 모두 attrs 를 출현 순서대로 push
    (종전 `skip_element` 으로 폐기하던 속성 보존).
- `parse_paragraph`: `field_end_attrs` 선언 + 호출부 전달.
- `visible_char_idx` 루프: `\u{0004}` 처리 시 `field_stack.pop()` 이
  - `Some` → 종전대로 `FieldRange` (동일 문단 필드),
  - `None` → `OrphanFieldEnd { char_idx: visible_char_idx, begin_id_ref, field_id }` 기록.
- `para.orphan_field_ends` 대입.

## 검증
신규 단위 테스트 2건 (`cargo test --lib task1556`) — 통과:
- `task1556_orphan_field_end_recorded_in_end_paragraph`:
  다단락 필드 → 문단0 fieldBegin 보존·고아 0, 문단1 고아 1건
  (`char_idx=2, begin_id_ref=1878228493, field_id=627272811`),
  `char_count=11`(텍스트2+8+끝1), 두번째 char_shape 경계 offsets 축 10.
- `task1556_same_paragraph_field_uses_range_not_orphan`:
  동일 문단 begin+end → `field_ranges` 1, 고아 0 (회귀 가드).

`cargo build` 클린. 직렬화기 미수정 단계이므로 roundtrip 효과는 단계 2 이후.

## 다음 단계
단계 2 — 직렬화기에서 `orphan_field_ends` 방출 + `inferred_control_slot_count` 차감 정합.
