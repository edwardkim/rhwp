//! 재조판한 개체 뒤 앵커 줄의 수용·방출·전진을 같은 줄 메트릭으로 수행한다.

use super::{FormattedParagraph, TypesetState};
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;

pub(super) fn place_with_storage<S: crate::storage::Storage>(
    st: &mut TypesetState,
    owner: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    dpi: f64,
    storage: &S,
) -> Result<(), S::Error> {
    let Some(tail) = crate::renderer::composer::composed_object_tail(para)
        .filter(|tail| tail.end == fmt.line_count())
    else {
        return Ok(());
    };
    for line in tail {
        let content_height = fmt.line_content_height(line..line + 1, 0.0);
        if st.current_height + content_height > st.available_height() + 0.5
            && !st.current_items.is_empty()
        {
            st.advance_column_or_new_page_with_storage(storage)?;
        }
        st.record_host_text_fragment(owner, line..line + 1, para, fmt, dpi);
        st.append_item_with_storage(
            PageItem::PartialParagraph {
                para_index: owner,
                start_line: line,
                end_line: line + 1,
            },
            storage,
        )?;
        st.advance_flow_by(fmt.line_advance(line));
    }
    st.advance_flow_by(fmt.spacing_after);
    Ok(())
}
