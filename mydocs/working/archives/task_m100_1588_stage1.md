# Task #1588 — Stage 1 완료보고서

**단계**: 선 도형 shapeComment 드롭 재현 (RED)
**브랜치**: `local/task1588`

## 작업 내용

`src/serializer/hwpx/shape.rs` 테스트 모듈에 가드 2건 추가:
- `task1588_line_shape_comment_emitted`: description 있는 선 도형 → `<hp:shapeComment>` 방출 검증.
- `task1588_line_shape_no_comment_when_empty`: 빈 설명 → 미방출 검증(빈 태그 금지).

## 결과 (RED 확인)

```
task1588_line_shape_comment_emitted ... FAILED  (shapeComment 미방출 — 드롭)
task1588_line_shape_no_comment_when_empty ... ok
```

- 선 도형 XML 에 `<hp:shapeComment>` 없음 → 근본원인(write_line 의 write_shape_comment 누락) 일치.

## 다음 단계

Stage 2 — `write_line` 에 `write_shape_comment(w, c)?;` 1줄 추가로 GREEN.
