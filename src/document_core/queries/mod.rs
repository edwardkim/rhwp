mod bookmark_query;
mod cursor_nav;
mod cursor_rect;
pub(crate) mod doc_tree_nav;
// [#3281] `fields` CLI 가 필드 위치(NestedEntry)를 읽어야 하므로 공개한다.
// 읽기 전용 질의 모듈이며 `structure`·`rendering` 과 같은 가시성이다.
pub mod field_query;
mod form_query;
pub mod rendering;
// [#3283] `grep` 이 같은 매칭 규칙(find_matches)을 쓰도록 크레이트 내부 공개.
/// 주소(구역·문단·페이지)를 가진 검색 — 조판 엔진이 있어야만 가능한 질의.
pub mod changed_pages;
pub mod grep;
/// [#3719 §6-11] 공개 전 개인정보 탐지 — 읽기 전용 판정(마스킹은 CLI 의 치환 경로).
pub mod pii_scan;
pub(crate) mod search_query;
pub mod structure;
pub mod table_extract;
