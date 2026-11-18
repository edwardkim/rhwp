//! 캡션 문단은 자기 TAC 표를 직접 출력하며 본문의 별도 PageItem을 만들지 않는다.

use super::{ComposedParagraph, Paragraph, ResolvedStyleSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockTableOwner {
    PageItems,
    Paragraph,
}

pub(crate) fn compose_caption_paragraph(
    paragraph: &Paragraph,
    styles: &ResolvedStyleSet,
) -> ComposedParagraph {
    match compose_caption_paragraph_with_storage(
        paragraph,
        Some(styles),
        &crate::storage::Unbounded,
    ) {
        Ok(composed) => composed,
        Err(never) => match never {},
    }
}

pub(crate) fn compose_caption_paragraph_with_storage<S: crate::storage::Storage>(
    paragraph: &Paragraph,
    styles: Option<&ResolvedStyleSet>,
    storage: &S,
) -> Result<ComposedParagraph, S::Error> {
    super::try_compose_paragraph_with_table_owner(
        paragraph,
        styles.is_some_and(|styles| styles.supplemental_metrics.is_some()),
        styles,
        BlockTableOwner::Paragraph,
        storage,
    )
}
