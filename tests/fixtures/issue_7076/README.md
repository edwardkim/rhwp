# Issue #7076 fixture — 값 없는 누름틀 안내문이 PDF 에 인쇄된다

`ship-collision-analysis-form.hwp`
(sha256 `9c7952549166ec9b20efc8f32521fd3d1673f367599e6e82652b65c782e6bc44`, 54,784 B)

공개 행정규칙 별지 서식이다. 누름틀 22개 중 값이 채워지지 않은 12개가 안내문을 달고
있어, 렌더 프로필이 `Screen` 이면 `#3375` 의 `editor_only` 억제가 꺼진 채 그 안내문이
서식 빈칸에 그려진다. PDF 는 인쇄 등가 출력이므로 프로필을 안 준 갈래도 `Print` 여야
한다는 것이 `#7076` 의 계약이다.

이 파일은 `samples/` 가 아니라 `tests/fixtures/` 에 둔다 — `samples/` 신규 항목은
코퍼스 래칫 6종(clipping · ir_field_sweep · off_canvas · oracle_page_count ·
overflow_cell · text_overlap)의 모수를 흔든다.

런타임 분류기는 이 파일 이름으로 분기하지 않는다.
