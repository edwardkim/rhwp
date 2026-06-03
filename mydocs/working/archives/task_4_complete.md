# Task #4 — 완료보고서

## 비-TAC 그림(어울림 배치) 높이 미반영 수정 ✅

### 수정 파일

- `src/renderer/layout.rs` — `layout_shape_item()` 반환 타입 변경 및 y_offset 반영

### 변경 내용

1. `layout_shape_item()` 반환 타입: `()` → `f64`
2. 비-TAC 그림의 `layout_body_picture()` 반환값(갱신된 y_offset)을 캡처하여 반환
3. 호출부(`layout_column_item`)에서 `y_offset`에 반영

### 검증 결과

- 21페이지: 그림(y=184.5, h=333.5) 끝=518.0, 표(y=596.3) → 겹침 해소
- `cargo test`: 777 passed, 0 failed
- 67페이지 전체 내보내기: 에러/패닉 없음
