# Task #1588 — Stage 2 완료보고서

**단계**: 수정 (write_line 에 shapeComment 방출)
**브랜치**: `local/task1588`

## 변경 내용

`src/serializer/hwpx/shape.rs` `write_line`: caption 방출 직후 `write_shape_comment(w, c)?;`
1줄 추가 (OWPML 순서 outMargin→caption→shapeComment, write_rect 동형).

## 검증 (모두 GREEN)

| 검사 | 결과 |
|------|------|
| `task1588_line_shape_comment_emitted` | PASS |
| `task1588_line_shape_no_comment_when_empty` | PASS |
| `cargo test --lib` | 1963 passed, 0 failed |
| `hwpx_roundtrip_baseline` | 4/4 |
| `cargo clippy --lib` (shape.rs) | 무경고 |

## 다음 단계

Stage 3 — fidelity 전수 통제 비교(3건 해소 + 악화 0) + opengov 가드 편입.
