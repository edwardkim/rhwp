mod clipboard;
// [#5769] 삭제의 참 역연산 — 조각 저장소. DeleteFragment 타입이 DocumentCore
// 필드로 쓰이므로 pub(crate).
pub(crate) mod delete_fragment;
mod document;
mod footnote_ops;
mod formatting;
mod header_footer_ops;
mod html_import;
mod object_ops;
// [#3565] 대형 문서 결함을 이분법으로 좁히기 위한 쪽 범위 추출.
pub mod page_extract;
mod table_ops;
mod text_editing;
