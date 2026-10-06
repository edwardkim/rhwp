# Task #1592 — Stage 1 완료보고서 (RED)

**단계**: 빈 문단 spurious (0,0) 재현
**브랜치**: `local/task1592`

`task1592_empty_paragraph_no_spurious_charshape`: 완전 빈 문단(text="", char_shapes=[],
컨트롤 없음) roundtrip → 재파싱 char_shapes 검증.

결과(RED): `빈 문단은 char_shapes 가 비어야 한다 ...: [(0, 0)]` — 직렬화기가 빈
`<hp:run charPrIDRef="0">` 추가 → (0,0) 발생. 근본원인(RunSplitter::new 규칙3 + close_run 규칙5) 일치.
