//! 계산 표의 이어받기 뒤 앞 간격을 현재 단의 한 결과로 확정한다.

use super::metrics::FormattedParagraph;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::{pagination::PageItem, typeset::TypesetState};

fn computed_fragment_host<'a>(
    st: &TypesetState,
    paragraphs: &'a [Paragraph],
) -> Option<&'a Paragraph> {
    let PageItem::PartialTable {
        para_index,
        control_index,
        is_continuation: true,
        ..
    } = st.current_items.last()?
    else {
        return None;
    };
    let host = paragraphs.get(*para_index)?;
    let Control::Table(table) = host.controls.get(*control_index)? else {
        return None;
    };
    crate::renderer::float_placement::computed_empty_table_host(host, table).then_some(host)
}

pub(super) fn confirm_split_successor(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
) {
    if st.prefilled_line_prefixes.contains_key(&para_idx) {
        return;
    }
    let Some(before) = computed_fragment_host(st, paragraphs).and_then(|host| {
        crate::renderer::float_placement::split_float_successor_spacing_before(
            host,
            para,
            st.current_start_height,
            fmt.spacing_before,
            st.current_height,
        )
    }) else {
        return;
    };
    st.record_float_pushed_line_spacing(para_idx, before);
}

/// fit·전진·줄 분할은 paint가 받는 같은 현재 단 기록을 소비한다.
/// 단을 넘기면 기록도 함께 비워지므로 원래 앞 간격으로 다시 판단한다.
pub(super) fn confirmed_format(
    st: &TypesetState,
    para_idx: usize,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
) -> Option<FormattedParagraph> {
    computed_fragment_host(st, paragraphs)?;
    let before = st.float_pushed_line_spacings.get(&para_idx)?.spacing_before;
    let mut adjusted = fmt.clone();
    let delta = before - fmt.spacing_before;
    adjusted.spacing_before = before;
    adjusted.total_height += delta;
    adjusted.height_for_fit += delta;
    Some(adjusted)
}
