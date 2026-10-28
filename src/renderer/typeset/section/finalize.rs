//! 구역 finalize 책임. 기존 조건과 호출 순서를 보존한다.
use crate::renderer::typeset::{HeaderFooterApply, HeaderFooterRef, Paragraph, TypesetEngine};
impl TypesetEngine {
    pub(in crate::renderer::typeset) fn collect_header_footer_controls(
        paragraphs: &[Paragraph],
        section_index: usize,
    ) -> (
        Vec<(usize, HeaderFooterRef, bool, HeaderFooterApply)>,
        Option<crate::model::control::PageNumberPos>,
    ) {
        match crate::renderer::pagination::collect_header_footer_controls_with_storage(
            paragraphs,
            section_index,
            &crate::storage::Unbounded,
        ) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
}
