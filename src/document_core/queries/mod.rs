mod bookmark_query;
mod cursor_nav;
mod cursor_rect;
pub(crate) mod doc_tree_nav;
pub(crate) mod field_query;
mod form_query;
pub mod rendering;
// [#3283] `grep` 이 같은 매칭 규칙(find_matches)을 쓰도록 크레이트 내부 공개.
/// 주소(구역·문단·페이지)를 가진 검색 — 조판 엔진이 있어야만 가능한 질의.
pub mod grep;
pub(crate) mod search_query;
pub mod structure;
