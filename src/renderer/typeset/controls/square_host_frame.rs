//! 저장 단의 좌표와 현재 호스트 plan이 공유할 원점을 조회한다.
use crate::renderer::typeset::{
    is_synthetic_line_seg, ComposedParagraph, Control, PageItem, Paragraph, TypesetState,
};

pub(super) fn stored_host_origin(
    st: &TypesetState,
    index: usize,
    paragraphs: &[Paragraph],
    composed: &[ComposedParagraph],
    dpi: f64,
) -> Option<f64> {
    if st.profile.session_edited()
        || st.profile.hwp3_layout()
        || !(st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
    {
        return None;
    }
    let first = match st.current_items.first()? {
        PageItem::FullParagraph { para_index } => *para_index,
        PageItem::Table {
            para_index,
            control_index,
        } => {
            let Control::Table(table) =
                paragraphs.get(*para_index)?.controls.get(*control_index)?
            else {
                return None;
            };
            if !table.common.treat_as_char {
                return None;
            }
            *para_index
        }
        _ => return None,
    };
    let chain = paragraphs.get(first..=index)?;
    if chain.first()?.line_segs.first()?.vertical_pos != 0 {
        return None;
    }
    // dirty 자체가 아니라 실제 이전 plan의 세로 구성이 저장 축을 보존했는지 확인한다.
    // 일반 재조판만으로 dirty가 된 프레임은 이 근거를 제공하지 않는다.
    if st.vpos_ladder_dirty && !st.inline_flow_plans.keys().any(|owner| *owner < index) {
        return None;
    }
    let mut previous = None;
    for (offset, para) in chain.iter().enumerate() {
        let owner = first + offset;
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.line_segs.is_empty()
            || para
                .source_line_seg_vertical_pos
                .as_ref()
                .is_some_and(|source| {
                    source.len() != para.line_segs.len()
                        || source
                            .iter()
                            .zip(&para.line_segs)
                            .any(|(vpos, row)| *vpos != row.vertical_pos)
                })
        {
            return None;
        }
        for row in &para.line_segs {
            if is_synthetic_line_seg(row)
                || previous.is_some_and(|position| row.vertical_pos < position)
            {
                return None;
            }
            previous = Some(row.vertical_pos);
        }
        if owner == index {
            break;
        }
        if let Some(plan) = st.inline_flow_plans.get(&owner) {
            let rows = plan.text_rows.as_ref()?;
            if rows.len() != para.line_segs.len()
                || rows.iter().zip(&para.line_segs).any(|(row, source)| {
                    row.text_start != source.text_start || row.line_height != source.line_height
                })
            {
                return None;
            }
            // 같은 문자 소속의 fresh 피치는 저장 간격과 반올림이 다를 수 있다.
            // 숫자 일치 대신 실제 행 점유가 다음 저장 원점 안에 드는지 확인한다.
            // 줄 수·높이 변경이나 실제 겹침은 저장 축을 재사용할 근거가 아니다.
            let next = paragraphs.get(owner + 1)?.line_segs.first()?;
            let source_advance = next
                .vertical_pos
                .checked_sub(para.line_segs.first()?.vertical_pos)?;
            let start = rows.first()?.vertical_pos;
            if rows.iter().any(|row| {
                i64::from(row.vertical_pos) - i64::from(start) + i64::from(row.line_height)
                    > i64::from(source_advance)
            }) {
                return None;
            }
        } else {
            let actual = composed.get(owner)?;
            if actual.lines.len() != para.line_segs.len()
                || actual
                    .lines
                    .iter()
                    .zip(&para.line_segs)
                    .any(|(row, source)| {
                        row.line_height != source.line_height
                            || row.line_spacing != source.line_spacing
                    })
            {
                return None;
            }
        }
    }
    Some(
        st.current_start_height
            + crate::renderer::hwpunit_to_px(
                paragraphs[index].line_segs.first()?.vertical_pos,
                dpi,
            ),
    )
}
