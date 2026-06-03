# Task 240 - 1단계 완료 보고서: Rust WASM API — 책갈피 CRUD + F11 지원

## 완료 항목

### bookmark_query.rs (신규)
- `get_bookmarks_native()` — 문서 전체 순회하여 책갈피 목록 JSON 반환
- `add_bookmark_native()` — 커서 위치에 Bookmark 컨트롤 삽입 (중복 이름 거부)
- `delete_bookmark_native()` — sec/para/ctrl_idx로 책갈피 제거
- `rename_bookmark_native()` — 이름 변경 (중복 검사 포함)
- `collect_bookmarks()` — 내부 헬퍼, 전체 문단 순회

### wasm_api.rs
- `getBookmarks()`, `addBookmark()`, `deleteBookmark()`, `renameBookmark()` 4종 WASM 바인딩

### text_editing.rs
- `classify_control()`에 `Control::Bookmark` → `"bookmark"` 추가
- F11이 책갈피 컨트롤을 선택 대상으로 인식

### queries/mod.rs
- `mod bookmark_query` 등록

## 검증
- `cargo build` 성공
- 716 테스트 통과 (0 실패)
