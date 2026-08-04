# Task #2905 처리 결과 — group_shapes_native char_offsets 전체 시프트 버그

## 이슈
https://github.com/edwardkim/rhwp/issues/2905

## 문제

`src/document_core/commands/object_ops/shape.rs`의 `group_shapes_native`가 새
`GroupShape` 컨트롤을 문단에 삽입한 뒤, `para.char_offsets` 배열 전체에 무조건
`+8`을 적용했다. `char_offsets`는 컨트롤 인덱스가 아니라 문단 텍스트의 각 문자가
바이트 스트림 상 어디에 위치하는지 기록하는 배열이므로, 새 컨트롤이 삽입되는
지점 **이후**의 항목만 시프트해야 한다. 삽입 지점보다 앞에 위치한 텍스트/다른
컨트롤은 실제 위치가 변하지 않았음에도 오프셋이 잘못 밀렸다.

같은 파일의 `create_shape_control_native`(shape.rs:1519-1556)와
`insert_equation_native`(equation.rs, `Paragraph::shift_for_inline_control_insert`
경유)는 모두 "삽입 지점 이후만 시프트" 규약을 지키고 있어, `group_shapes_native`만
예외였다.

## 수정

컨트롤 삽입 *이전* 상태에서 `find_control_text_positions(para)`로 삽입 지점의
텍스트 char-index(`safe_offset`)를 구하고, `para.char_offsets[safe_offset..]`
구간만 `+8` 하도록 변경했다(`create_shape_control_native`와 동일 패턴).

## 재현/회귀 테스트

`src/document_core/commands/object_ops/shape.rs`
`resize_clamp_tests::group_shapes_only_shifts_char_offsets_after_insertion_point`

- 문단 텍스트 "A"(char_offsets=[0]) 뒤에 사각형 3개(S0,S1,S2)를 순서대로 삽입.
- S1, S2만 그룹으로 묶음(S0는 문단에 그대로 남음).
- 수정 전: `char_offsets`가 `[8]`로 잘못 변경됨(수동으로 fix 라인을 제거해
  회귀를 재현·확인함, red).
- 수정 후: `char_offsets`가 `[0]`으로 유지됨(green).

## 검증

- `cargo build --lib` — 통과
- `cargo test --lib object_ops::shape::` — 6개 전부 통과(신규 테스트 포함)
- `cargo clippy --all-targets --profile release-test -- -D warnings` — 경고 없음
- `rustfmt --edition 2021 src/document_core/commands/object_ops/shape.rs` — 적용

## 변경 파일

- `src/document_core/commands/object_ops/shape.rs` (수정 + 테스트 1건 추가)

수정 범위는 `shape.rs`로 한정했으며, 동일 도메인의 `equation.rs`는 조사
과정에서 다른 가설(page_tree_cache 무효화 누락)을 검토했으나 `recompose_section`이
이미 무조건 `invalidate_page_tree_cache()`를 호출함을 확인해 오탐으로 판정,
해당 파일은 변경 없이 원본 상태로 되돌렸다.
