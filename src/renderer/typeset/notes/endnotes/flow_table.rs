//! 미주가 소유한 흐름 표도 본문의 측정·행 분할·이어받기 결과를 사용한다.

use crate::model::{control::Control, paragraph::ColumnBreakType};
use crate::renderer::height_measurer::HeightMeasurer;
use crate::renderer::typeset::{
    FormattedParagraph, MeasuredTable, Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    /// 별도 글줄이 없는 문단의 표를 본문과 같은 TAC/블록 표 조판으로 방출한다.
    /// 배경·전경·Square 표와 가시 호스트의 글줄은 기존 개체 흐름을 유지한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn typeset_endnote_flow_tables(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        para_index: usize,
        formatted: &FormattedParagraph,
        future: &[Paragraph],
        endnote_start: i32,
        compact_spacing: bool,
        body_measured_tables: &[MeasuredTable],
    ) -> bool {
        let Some(paragraph) = para_index
            .checked_sub(paragraphs.len())
            .and_then(|index| st.endnote_paragraphs.get(index))
        else {
            return false;
        };
        // 한 줄의 독립 TAC 표는 그 글줄 하나를 원자적으로 수용한다.
        // 같은 줄의 여러 표나 본문 혼재를 세로 블록으로 해석하지 않는다.
        let single_inline_table = paragraph.controls.len() == 1
            && formatted.line_count() == 1
            && matches!(paragraph.controls.first(), Some(Control::Table(table))
                if table.common.treat_as_char);
        if !paragraph.text.is_empty()
            || paragraph.column_type != ColumnBreakType::None
            || paragraph.controls.is_empty()
            || !paragraph.controls.iter().all(|control| {
                single_inline_table
                    || matches!(control, Control::Table(table)
                    if crate::renderer::float_placement::is_para_topbottom_float(&table.common)
                        && table.common.flow_with_text)
            })
        {
            return false;
        }

        // 다음 문단 조회도 실제 가상 인덱스를 사용한다. 표 조판이 아직 방출하지
        // 않은 미주 내용을 본문으로 오인하거나 다음 속성을 잃지 않게 한다.
        let mut paint_paragraphs = paragraphs.to_vec();
        paint_paragraphs.extend_from_slice(&st.endnote_paragraphs);
        paint_paragraphs.extend(future.iter().map(|paragraph| {
            super::row_projection::render_paragraph(paragraph, endnote_start, compact_spacing)
        }));
        let paint_composed: Vec<_> = paint_paragraphs
            .iter()
            .map(|paragraph| {
                crate::renderer::composer::compose_paragraph_in_context(paragraph, styles)
            })
            .collect();
        let paragraph = &paint_paragraphs[para_index];
        let profile = st.profile;
        let measurer = HeightMeasurer::new(self.dpi)
            .with_hwp3_variant(profile.hwp3_layout())
            .with_stored_geometry_policy(profile.stored_geometry_policy())
            .with_native_hwp5(profile.native_hwp5_layout())
            .with_hwpx_stored_layout(profile.hwpx_stored_layout())
            .with_session_edited(profile.session_edited())
            .with_render_normalization(std::sync::Arc::clone(&self.render_normalization));
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                unreachable!("흐름 표 문단의 컨트롤을 먼저 확인했다");
            };
            st.append_endnote_measured_table(measurer.measure_table(
                table,
                para_index,
                control_index,
                styles,
            ));
        }
        let mut measured_tables = body_measured_tables.to_vec();
        measured_tables.extend_from_slice(&st.endnote_measured_tables);
        let width = st.inline_flow_column().width;
        // 저장 높이의 누계가 아니라 기존 항목의 실제 흐름 원점을 이어받는다.
        // 같은 단의 앞 표 조각과 글줄도 위의 공유 측정 원장을 사용한다.
        let origin = self
            .measure_endnote_column_with_item(
                st,
                paragraphs,
                styles,
                st.available_height(),
                width,
                None,
                true,
                None,
                &measured_tables,
            )
            .map_or(st.current_height, |geometry| geometry.flow_bottom);
        st.align_flow_to(origin);
        st.mark_endnote_flow();

        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                unreachable!("흐름 표 문단의 컨트롤을 먼저 확인했다");
            };
            let origin = st.current_height;
            let formatted_table = self.format_table(
                paragraph,
                para_index,
                control_index,
                table,
                &measured_tables,
                styles,
                paint_composed.get(para_index),
                paint_paragraphs.get(para_index + 1),
                st.current_items.is_empty(),
            );
            let measured = measured_tables.iter().find(|measured| {
                measured.para_index == para_index && measured.control_index == control_index
            });
            if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
                eprintln!(
                    "EN_FLOW_TABLE pi={para_index} ci={control_index} origin={origin:.3} rows={:?} available={:.3}",
                    formatted_table.row_heights,
                    st.available_height(),
                );
            }
            if single_inline_table {
                self.typeset_tac_table(
                    st,
                    para_index,
                    control_index,
                    paragraph,
                    table,
                    &formatted_table,
                    formatted,
                    1,
                    true,
                    true,
                    styles,
                    paint_composed.get(para_index),
                    paint_paragraphs.get(para_index + 1),
                    crate::renderer::typeset::preceding_stored_vpos(&paint_paragraphs, para_index),
                );
                continue;
            }
            let normalized_frame = (!st.profile.session_edited())
                .then(|| {
                    crate::renderer::table_row_frame::normalized_empty_control_frame(
                        paragraph,
                        paint_paragraphs.get(para_index + 1)?,
                        table,
                        &formatted_table.row_heights,
                        origin,
                        self.dpi,
                        st.profile.hwpx_stored_layout(),
                    )
                })
                .flatten();
            let owning_column = (st.pages.len(), st.current_column);
            self.typeset_block_table(
                st,
                para_index,
                control_index,
                paragraph,
                table,
                &formatted_table,
                formatted,
                measured,
                styles,
                origin,
                origin,
                control_index == 0,
                control_index + 1 == paragraph.controls.len(),
                &paint_paragraphs,
                &paint_composed,
            );
            // 분할·이월은 기존 표 경로가 소유한다. 현재 단에 통째 방출된 표만
            // 같은 본체·바깥 여백·후속 줄 프레임을 실제 paint에도 전달한다.
            if owning_column == (st.pages.len(), st.current_column)
                && st.current_items.last().is_some_and(|item| {
                    matches!(item, crate::renderer::pagination::PageItem::Table {
                        para_index: owner,
                        control_index: control,
                    } if *owner == para_index && *control == control_index)
                })
            {
                if let Some(frame) = normalized_frame {
                    st.record_paragraph_float_placement((para_index, control_index), frame);
                }
            }
        }
        true
    }
}
