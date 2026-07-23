# Issue #3170 처리 결과 보고서

## 목표

HWPX `header.xml`의 `<hh:charPr id="N">`/`<hh:paraPr id="N">` 파싱이 `id` 속성을 무시하고 XML 문서
등장 순서를 그대로 배열 인덱스로 쓰는 결함을 수정한다. 본문 파싱(`section.rs`)과 렌더러/문서
코어는 `charPrIDRef`/`paraPrIDRef` 값을 그대로 `char_shapes`/`para_shapes` 배열 인덱스로 쓰므로,
`id` 가 등장 순서와 다르거나 중간이 비어 있으면 스타일 참조가 조용히 뒤바뀐다.

## 원인

- `src/parser/hwpx/header.rs`의 `parse_char_shape`/`parse_para_shape`가 `<hh:charPr>`/`<hh:paraPr>`
  요소의 `id` 속성을 전혀 읽지 않고, 매 호출마다 `doc_info.char_shapes.push(cs)` /
  `doc_info.para_shapes.push(ps)` 로 등장 순서에만 의존해 배열에 추가했다.
- 반면 `src/parser/hwpx/section.rs`는 `<hp:run charPrIDRef="N">` / `<hp:p paraPrIDRef="N">` 의 `N`
  값을 그대로 `para.char_shape_id`/`para.para_shape_id` 에 저장하고, 렌더러(`style_resolver.rs`,
  `layout.rs`, `typeset.rs` 등)와 `document_core` 서식 커맨드는 이 값을 `char_shapes[N]` /
  `para_shapes[N]` 인덱스로 그대로 조회한다.
- 즉 "XML 등장 순서 == id 값" 이라는 암묵적 가정이 파싱 단계에서 강제되지 않아, id 가 비순차적인
  (스타일 편집기 재정렬, 스타일 삭제로 인한 gap 등) 문서에서 `charPrIDRef`/`paraPrIDRef` 참조가
  다른 CharShape/ParaShape 로 오해석되고, 글꼴·크기·정렬·줄간격 등 서식이 조용히 바뀐다.

## 재현 (red)

`src/parser/hwpx/header.rs` 테스트 모듈에 두 개의 red 테스트를 추가해 재현했다.

- `test_char_pr_id_out_of_order_resolves_by_id_not_document_order`
  — `<hh:charPr id="1" height="2000">` 가 `<hh:charPr id="0" height="1000">` 보다 먼저 등장하는
  header.xml 조각을 파싱한 뒤, `char_shapes[0]`(= `charPrIDRef="0"` 참조 대상)이 `height=1000`
  (id="0")이어야 함을 검증. 수정 전에는 `char_shapes[0]`에 첫 번째로 등장한 id="1" 항목이
  들어가 `height=2000`으로 어긋났다(FAIL 확인).
- `test_para_pr_id_out_of_order_resolves_by_id_not_document_order`
  — 동일 구조를 `<hh:paraPr id="N">` + `<hh:align horizontal="...">` 로 재현. 수정 전
  `para_shapes[0].alignment` 이 `Justify`(default)로 나와 `paraPrIDRef="0"` 참조가 실제로는
  `id="0"` 항목에 도달하지 못함을 확인(FAIL).

## 수정

`parse_char_shape`/`parse_para_shape`에서 `id` 속성을 읽어, 해당 인덱스에 정확히 배치하도록
변경했다.

- `id` 속성 값을 파싱해 `Option<usize>` 로 보관.
- 배열 길이가 `id` 보다 작으면 `Vec::resize_with` 로 기본값(`CharShape::default`/
  `ParaShape::default`)을 채워 확장한 뒤, `char_shapes[id] = cs` / `para_shapes[id] = ps` 로
  정확한 위치에 배치.
- `id` 속성이 없는 비정상 입력만 기존처럼 등장 순서 `push` fallback 유지(하위 호환).

기존 header.rs 단위 테스트 4건은 `<hh:paraPr id="1">`/`id="2"`/`id="3"` 처럼 1부터 시작하는
샘플을 쓰면서도 결과를 `para_shapes[0]`/`[1]`/`[2]` 로 검증하고 있었다(구 코드의 등장 순서 push를
암묵 전제한 것). 수정 후 올바른 의미(= id 값이 실제 인덱스)에 맞춰 해당 테스트들의 기대 인덱스를
`id` 값과 일치하도록 갱신했다:

- `test_parse_hwpx_para_shape_condense_attr_bits`
- `para_shape_linespacing_between_lines_parses_as_space_only`
- `test_parse_hwpx_para_shape_break_non_latin_word_bit`
- `test_parse_hwpx_para_shape_snap_to_grid_bit`

## 검증

| 항목 | 결과 |
|---|---|
| red 테스트 2건 (수정 전) | FAIL 확인 |
| `cargo test --lib parser::hwpx::` | 116 passed, 0 failed |
| `cargo test --lib` (전체) | 2555 passed, 0 failed, 7 ignored |

전체 회귀 테스트가 모두 통과해 이번 수정이 다른 IR/왕복 계약을 깨지 않음을 확인했다.

## 범위 밖

- `hh:style` 의 `nextStyleIDRef`(다음 스타일 자동 적용)는 편집기 동작(Enter 키 시 다음 문단
  스타일 전환)이 애초에 구현돼 있지 않아 이번 결함과 별개다. 별도 기능 이슈로 다룰 사안이며
  이번 PR 범위에 포함하지 않았다.
- `hh:style` 의 `lockForm` 속성 직렬화 문제(#2839)는 이미 별도로 열린 이슈로, 겹치지 않게
  건드리지 않았다.
