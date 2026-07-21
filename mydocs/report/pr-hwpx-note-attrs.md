# PR #2730: HWPX 각주/미주 저장 시 prefixChar/suffixChar/instId/flag 속성 유실 수정 (#2716)

## 수정
HWPX 직렬화기 `render_note_sublist`가 number만 방출하고 4개 속성을 버리던 문제 수정.

- 직렬화: `NoteAttrs` 구조체 도입, 한컴 생략 규칙에 따라 flag/number/prefixChar/suffixChar/instId 방출
- 파서: `parse_ctrl_footnote`/`parse_ctrl_endnote`에 `flag` → `number_shape` 매핑 추가
