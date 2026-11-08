//! 저장 줄의 실제 흐름 예약과 점유 결과로 미주의 fit을 확인한다.

use crate::renderer::typeset::{
    ColumnBreakType, FormattedParagraph, MeasuredTable, PageItem, Paragraph, ResolvedStyleSet,
    TypesetEngine, TypesetState,
};

/// 실제 점유 하한과 명시적 묶음 속성을 구분한 새 미주 제목의 fit 결과.
#[derive(Default)]
pub(super) struct NewHeadFit {
    pub(super) natural_overflows: bool,
    pub(super) can_stay: bool,
}

/// 같은 단에서 실제로 배치한 글줄의 수용 결과.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReflowedEndnoteFit {
    Whole,
    Split(usize),
    Overflow,
}

impl ReflowedEndnoteFit {
    pub(super) fn can_stay(self) -> bool {
        matches!(self, Self::Whole | Self::Split(_))
    }
}

/// 측정 불가와 실제 수용 불가를 구분한 앞부분의 수용 결과.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EndnotePrefixFit {
    Split(usize),
    Overflow,
}

impl EndnotePrefixFit {
    pub(super) fn split_line(self) -> Option<usize> {
        match self {
            Self::Split(line) => Some(line),
            Self::Overflow => None,
        }
    }
}

impl TypesetEngine {
    /// 일반 inline 미주 글줄의 전체·앞부분 점유를 한 계약으로 조회한다.
    /// 저장 좌표의 자동 단 컷이 있어도 현재 방출할 줄 범위를 직접 측정한다.
    /// 별도 개체 계획·명시적 경계·아직 조판하지 않은 keep 그룹은 대체하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn query_reflowed_endnote_fit(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        formatted: &FormattedParagraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> Option<ReflowedEndnoteFit> {
        if st.current_items.is_empty()
            || formatted.line_count() == 0
            || formatted.line_count() != paragraph.line_segs.len()
            || paragraph.column_type != ColumnBreakType::None
            || paragraph.controls.iter().any(|control| match control {
                crate::model::control::Control::Table(_)
                | crate::model::control::Control::Picture(_)
                | crate::model::control::Control::Shape(_) => true,
                crate::model::control::Control::Equation(equation) => {
                    !equation.common.treat_as_char
                }
                _ => false,
            })
        {
            return None;
        }
        let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
        if style.keep_with_next || style.page_break_before {
            return None;
        }
        let whole = self
            .measure_endnote_column_with_item(
                st,
                paragraphs,
                styles,
                available,
                column_width,
                Some(PageItem::FullParagraph { para_index }),
                true,
                None,
                measured_tables,
            )?
            .occupied_bottom?;
        if whole <= available {
            return Some(ReflowedEndnoteFit::Whole);
        }
        if style.keep_lines {
            return Some(ReflowedEndnoteFit::Overflow);
        }
        for end_line in (1..formatted.line_count()).rev() {
            if style.widow_orphan && (end_line < 2 || formatted.line_count() - end_line < 2) {
                continue;
            }
            let occupied = self
                .measure_endnote_column_with_item(
                    st,
                    paragraphs,
                    styles,
                    available,
                    column_width,
                    Some(PageItem::PartialParagraph {
                        para_index,
                        start_line: 0,
                        end_line,
                    }),
                    true,
                    None,
                    measured_tables,
                )?
                .occupied_bottom?;
            if occupied <= available {
                return Some(ReflowedEndnoteFit::Split(end_line));
            }
        }
        Some(ReflowedEndnoteFit::Overflow)
    }

    /// 앞 문단에 명시적인 묶음이 있을 때만 분할 불가인 다음 한 줄을 예약한다.
    /// 저장 vpos 되감김이나 내용 종류만으로 제목·수식을 묶지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_kept_successor_overflows(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
        future: &[Paragraph],
        endnote_start: i32,
        compact_spacing: bool,
    ) -> bool {
        if st.current_items.is_empty()
            || paragraph.column_type != ColumnBreakType::None
            || !styles
                .para_styles
                .get(paragraph.para_shape_id as usize)
                .is_some_and(|style| style.keep_with_next)
        {
            return false;
        }
        let Some(successor) = future.first() else {
            return false;
        };
        // 다줄 successor의 부분 컷과 별도 float는 자신의 분할 계약을 따른다.
        // 문단 전체를 임의로 예약해 한 줄 묶음을 확대하지 않는다.
        let composed = crate::renderer::composer::compose_paragraph_in_context(successor, styles);
        if self
            .format_endnote_paragraph(successor, Some(&composed), styles, Some(column_width))
            .line_count()
            != 1
        {
            return false;
        }
        let items = super::super::emit::endnote_paragraph_items(paragraph, para_index, false);
        self.measure_endnote_column_with_items(
            st,
            paragraphs,
            styles,
            available,
            column_width,
            &items,
            true,
            Some((&future[..1], endnote_start, compact_spacing)),
            measured_tables,
        )
        .and_then(|geometry| geometry.occupied_bottom)
        .is_some_and(|bottom| bottom > available)
    }

    /// 자동 분할 후보 전체가 실제로 들어가면 저장 누계의 컷을 취소한다.
    /// 글줄과 별도 개체의 소유·원점 계획을 혼동하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_unsplit_candidate_fits(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        styles
            .para_styles
            .get(paragraph.para_shape_id as usize)
            .is_some_and(|style| !style.keep_with_next && !style.page_break_before)
            && !paragraph.controls.iter().any(|control| {
                matches!(
                    control,
                    crate::model::control::Control::Table(_)
                        | crate::model::control::Control::Picture(_)
                        | crate::model::control::Control::Shape(_)
                )
            })
            && self.reflowed_endnote_whole_paragraph_fits(
                st,
                paragraphs,
                styles,
                paragraph,
                para_index,
                available,
                column_width,
                measured_tables,
            )
    }

    /// 저장 단 경계의 현재 문단을 실제 소유·표 측정 문맥으로 대조한다.
    /// 별도 PageItem/float 예약이 필요한 후보는 그 계획을 발행하기 전이므로
    /// FullParagraph만으로 fit을 선언하지 않는다. 이전 항목의 계획은 유지한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_saved_boundary_fits(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        let candidate_is_inline = paragraph.controls.iter().all(|control| match control {
            crate::model::control::Control::Table(table) => table.common.treat_as_char,
            crate::model::control::Control::Picture(picture) => picture.common.treat_as_char,
            crate::model::control::Control::Shape(shape) => shape.common().treat_as_char,
            _ => true,
        });
        if !candidate_is_inline || paragraph.column_type != ColumnBreakType::None {
            return false;
        }
        let items = super::super::emit::endnote_paragraph_items(paragraph, para_index, false);
        let geometry = self.measure_endnote_column_with_items(
            st,
            paragraphs,
            styles,
            available,
            column_width,
            &items,
            true,
            None,
            measured_tables,
        );
        let occupied = geometry.and_then(|result| result.occupied_bottom);
        if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
            eprintln!("EN_REWIND_OCCUPIED pi={para_index} items={} bottom={occupied:?} available={available:.3}", items.len());
        }
        occupied.is_some_and(|bottom| bottom <= available)
    }

    /// 저장 단의 잔여 높이는 현재 문단뿐 아니라 앞서 방출한 수식 줄에도 의존한다.
    pub(in crate::renderer::typeset) fn endnote_column_has_reissued_rows(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut reissued = false;
        for item in &st.current_items {
            let Some(para_index) = crate::renderer::typeset::page_item_para_index(item) else {
                continue;
            };
            let Some(source) = para_index
                .checked_sub(paragraphs.len())
                .and_then(|local| st.endnote_para_sources.get(local))
            else {
                continue;
            };
            let Some(crate::model::control::Control::Endnote(note)) = paragraphs
                .get(source.para_index)
                .and_then(|parent| parent.controls.get(source.control_index))
            else {
                continue;
            };
            let Some(original) = note.paragraphs.get(source.note_para_index) else {
                continue;
            };
            let composed =
                crate::renderer::composer::compose_paragraph_in_context(original, styles);
            let changed = crate::renderer::composer::stored_row_projection::reissued_equation_rows(
                original, &composed,
            );
            if std::env::var_os("RHWP_EN_ROW_METRIC_DEBUG").is_some() {
                let (first, end) = match item {
                    PageItem::PartialParagraph {
                        start_line,
                        end_line,
                        ..
                    } => (*start_line, *end_line),
                    _ => (0, original.line_segs.len()),
                };
                let mut source_advance = 0_i64;
                let mut actual_advance = 0_i64;
                for (source, actual) in original
                    .line_segs
                    .iter()
                    .zip(&composed.lines)
                    .skip(first)
                    .take(end.saturating_sub(first))
                {
                    source_advance +=
                        i64::from(source.line_height) + i64::from(source.line_spacing);
                    actual_advance +=
                        i64::from(actual.line_height) + i64::from(actual.line_spacing);
                }
                eprintln!("EN_ROW_METRIC pi={para_index} lines={first}..{end} source={source_advance} actual={actual_advance} changed={changed}");
            }
            reissued |= changed;
        }
        reissued
    }

    /// 저장 컷의 뒤 조각을 현재 단에 가져올 만큼 실제 전진이 줄었는지 확인한다.
    /// 재발행했다는 사실만으로 다음 단 소유의 글줄을 현재 단에 합치지 않는다.
    pub(in crate::renderer::typeset) fn endnote_column_reclaims_saved_tail(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        cut: usize,
    ) -> bool {
        let Some(tail) = paragraph.line_segs.get(cut..) else {
            return false;
        };
        let tail_advance: i64 = tail
            .iter()
            .map(|line| i64::from(line.line_height) + i64::from(line.line_spacing))
            .sum();
        if tail.is_empty() || tail_advance <= 0 {
            return false;
        }
        let mut reclaimed = 0_i64;
        for item in &st.current_items {
            let para_index = match item {
                PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, .. } => *para_index,
                _ => continue,
            };
            let Some(source) = para_index
                .checked_sub(paragraphs.len())
                .and_then(|local| st.endnote_para_sources.get(local))
            else {
                continue;
            };
            let Some(crate::model::control::Control::Endnote(note)) = paragraphs
                .get(source.para_index)
                .and_then(|parent| parent.controls.get(source.control_index))
            else {
                return false;
            };
            let Some(original) = note.paragraphs.get(source.note_para_index) else {
                return false;
            };
            let composed =
                crate::renderer::composer::compose_paragraph_in_context(original, styles);
            if original.stored_text_partition_is_dirty()
                || composed.lines.len() != original.line_segs.len()
            {
                return false;
            }
            let (first, end) = match item {
                PageItem::PartialParagraph {
                    start_line,
                    end_line,
                    ..
                } => (*start_line, *end_line),
                _ => (0, original.line_segs.len()),
            };
            for (source, actual) in original
                .line_segs
                .iter()
                .zip(&composed.lines)
                .skip(first)
                .take(end.saturating_sub(first))
            {
                reclaimed += i64::from(source.line_height) + i64::from(source.line_spacing)
                    - i64::from(actual.line_height)
                    - i64::from(actual.line_spacing);
            }
        }
        if std::env::var_os("RHWP_EN_ROW_METRIC_DEBUG").is_some() {
            eprintln!("EN_SAVED_TAIL cut={cut} reclaimed={reclaimed} advance={tail_advance}");
        }
        reclaimed >= tail_advance
    }

    /// 저장 컷의 확대·축소 모두 실제 앞부분의 점유로 확인한다.
    /// 유효한 컷이 없으면 Overflow, 별도 배치 계약이면 None을 반환한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_prefix_split(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> Option<EndnotePrefixFit> {
        let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
        if st.current_items.is_empty()
            || paragraph.column_type != ColumnBreakType::None
            || style.keep_with_next
            || style.page_break_before
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.controls.iter().any(|control| match control {
                crate::model::control::Control::Table(_)
                | crate::model::control::Control::Picture(_)
                | crate::model::control::Control::Shape(_) => true,
                crate::model::control::Control::Equation(equation) => {
                    !equation.common.treat_as_char
                }
                _ => false,
            })
        {
            return None;
        }
        let composed = crate::renderer::composer::compose_paragraph_in_context(paragraph, styles);
        let formatted =
            self.format_endnote_paragraph(paragraph, Some(&composed), styles, Some(column_width));
        if formatted.line_count() <= 1 || formatted.line_count() != paragraph.line_segs.len() {
            return None;
        }
        if style.keep_lines {
            return Some(EndnotePrefixFit::Overflow);
        }
        for end_line in (1..formatted.line_count()).rev() {
            if style.widow_orphan && (end_line < 2 || formatted.line_count() - end_line < 2) {
                continue;
            }
            let occupied = self
                .measure_endnote_column_with_item(
                    st,
                    paragraphs,
                    styles,
                    available,
                    column_width,
                    Some(PageItem::PartialParagraph {
                        para_index,
                        start_line: 0,
                        end_line,
                    }),
                    true,
                    None,
                    measured_tables,
                )?
                .occupied_bottom?;
            if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
                eprintln!("EN_PREFIX_OCCUPIED pi={para_index} end={end_line} bottom={occupied:.3} available={available:.3}");
            }
            if occupied <= available {
                return Some(EndnotePrefixFit::Split(end_line));
            }
        }
        Some(EndnotePrefixFit::Overflow)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_whole_paragraph_fits(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        // 저장 단 구성이 유지되면 컷을 보존한다. 재발행된 앞 줄 때문에 구성이
        // 달라진 경우만 실제 전체 배치를 재검증하며 명시적 나누기는 수용하지 않는다.
        paragraph.column_type == ColumnBreakType::None
            && super::super::measure::stored_endnote_column_cut(st, paragraphs, para_index)
                .is_none_or(|cut| {
                    self.endnote_column_reclaims_saved_tail(st, paragraphs, styles, paragraph, cut)
                })
            && self
                .simulate_endnote_column_with_item(
                    st,
                    paragraphs,
                    styles,
                    available,
                    column_width,
                    Some(PageItem::FullParagraph { para_index }),
                    true,
                    None,
                    measured_tables,
                    true,
                )
                .is_some_and(|occupied_end| occupied_end <= available)
    }

    /// 별도 묶음이나 float 원점 계획이 없는 한 줄의 실제 점유를 확인한다.
    /// 저장 높이 추정이 초과해도 물리 줄이 들어가면 앞 단에 남길 수 있다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_single_line_fits(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        formatted: &FormattedParagraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        if st.current_items.is_empty()
            || formatted.line_count() != 1
            || paragraph.line_segs.len() != 1
            || paragraph.column_type != ColumnBreakType::None
            || !styles
                .para_styles
                .get(paragraph.para_shape_id as usize)
                .is_some_and(|style| !style.keep_with_next && !style.page_break_before)
            || paragraph.controls.iter().any(|control| match control {
                crate::model::control::Control::Table(table) => !table.common.treat_as_char,
                crate::model::control::Control::Picture(picture) => !picture.common.treat_as_char,
                crate::model::control::Control::Shape(shape) => !shape.common().treat_as_char,
                _ => false,
            })
        {
            return false;
        }
        self.reflowed_endnote_whole_paragraph_fits(
            st,
            paragraphs,
            styles,
            paragraph,
            para_index,
            available,
            column_width,
            measured_tables,
        )
    }

    /// 이월 전에 실제 배치의 흐름 끝·점유 끝·간격을 함께 확인한다.
    /// 명시적 묶음은 유지하며, 저장 높이 추정만으로 제목을 넘기지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn query_reflowed_endnote_new_head_fit(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        formatted: &FormattedParagraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> NewHeadFit {
        if st.current_items.is_empty()
            || formatted.line_count() != 1
            || paragraph.column_type != ColumnBreakType::None
        {
            return NewHeadFit::default();
        }
        let Some(previous) = self.measure_endnote_column_with_item(
            st,
            paragraphs,
            styles,
            available,
            column_width,
            None,
            true,
            None,
            measured_tables,
        ) else {
            return NewHeadFit::default();
        };
        let Some(occupied) = previous.occupied_bottom else {
            return NewHeadFit::default();
        };
        let previous_index = st
            .current_items
            .iter()
            .rev()
            .find_map(crate::renderer::typeset::page_item_para_index);
        let current_source = para_index
            .checked_sub(paragraphs.len())
            .and_then(|local| st.endnote_para_sources.get(local));
        let between_complete_notes =
            previous_index
                .zip(current_source)
                .is_some_and(|(previous_index, current_source)| {
                    let previous_source = previous_index
                        .checked_sub(paragraphs.len())
                        .and_then(|local| st.endnote_para_sources.get(local));
                    crate::renderer::typeset::paragraph_by_global_index(
                        paragraphs,
                        &st.endnote_paragraphs,
                        previous_index,
                    )
                    .is_some_and(|previous| {
                        crate::renderer::endnote_spacing::starts_note_after_complete_tail(
                            previous_source,
                            current_source,
                            &PageItem::FullParagraph { para_index },
                            &st.current_items,
                            previous_index,
                            previous.line_segs.len(),
                        )
                    })
                });
        // 첫 미주 앞의 본문·구분선 간격은 실제 흐름에 이미 반영돼 있다.
        // 앞 미주의 마지막 줄을 소유할 때만 다른 미주 사이 간격을 적용한다.
        let gap = if between_complete_notes {
            crate::renderer::hwpunit_to_px(st.endnote_between_notes_hu.max(0), self.dpi)
        } else {
            0.0
        };
        let origin = crate::renderer::endnote_spacing::origin_after_gap(
            previous.flow_bottom,
            Some(occupied),
            gap,
        );
        let natural_overflows = origin + formatted.height_for_fit > available;
        let can_stay = !natural_overflows
            && styles
                .para_styles
                .get(paragraph.para_shape_id as usize)
                .is_some_and(|style| !style.keep_with_next && !style.page_break_before)
            && self.reflowed_endnote_whole_paragraph_fits(
                st,
                paragraphs,
                styles,
                paragraph,
                para_index,
                available,
                column_width,
                measured_tables,
            );
        if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
            eprintln!(
                "EN_HEAD_FIT pi={para_index} flow={:.3} occupied={occupied:.3} origin={origin:.3} head_height={:.3} gap={gap:.3} available={available:.3} overflows={natural_overflows} can_stay={can_stay}",
                previous.flow_bottom,
                formatted.height_for_fit,
            );
        }
        NewHeadFit {
            natural_overflows,
            can_stay,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_single_line_overflows(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        formatted: &FormattedParagraph,
        para_index: usize,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        if formatted.line_heights.len() > 1
            && paragraph.line_segs.len() == formatted.line_heights.len()
            && paragraph.column_type == ColumnBreakType::None
            && !paragraph
                .line_segs
                .windows(2)
                .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
        {
            // 유효한 저장 내부 컷은 별도 경로에서 소비한다. 연속 다줄 문단의
            // 첫 물리 줄조차 들어가지 않으면 뒤 줄이나 bleed 예외로 fit하지 않는다.
            let geometry = self.measure_endnote_column_with_item(
                st,
                paragraphs,
                styles,
                available,
                column_width,
                Some(PageItem::PartialParagraph {
                    para_index,
                    start_line: 0,
                    end_line: 1,
                }),
                true,
                None,
                measured_tables,
            );
            if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
                eprintln!(
                    "EN_FIRST_LINE_FIT pi={para_index} flow={:?} occupied={:?} available={available:.3}",
                    geometry.as_ref().map(|result| result.flow_bottom),
                    geometry.as_ref().and_then(|result| result.occupied_bottom),
                );
            }
            // 첫 줄의 내용과 줄 상자가 들어가면 앞 조각을 현재 단에 남긴다.
            // 흐름 끝의 후행 줄간격은 뒤 조각의 원점이며 첫 줄 점유가 아니다.
            return geometry
                .and_then(|result| result.occupied_bottom)
                .is_some_and(|bottom| bottom > available);
        }
        // 다줄의 분할 컷은 이 단일 줄 계약으로 바꾸지 않는다.
        if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
            eprintln!(
                "EN_SINGLE_FIT pi={para_index} items={} formatted_lines={} stored_lines={} break={:?} available={available:.3}",
                st.current_items.len(), formatted.line_heights.len(), paragraph.line_segs.len(), paragraph.column_type,
            );
        }
        if st.current_items.is_empty()
            || formatted.line_heights.len() != 1
            || paragraph.line_segs.len() != 1
            || paragraph.column_type != ColumnBreakType::None
        {
            return false;
        }
        // 현재 단과 후보만 측정한다. 빈 미래 문단으로 표·그림의 실제 문맥을
        // 거절하거나 측정값을 빈 배열로 바꾸지 않는다.
        let occupied = self.simulate_endnote_column_with_item(
            st,
            paragraphs,
            styles,
            available,
            column_width,
            Some(PageItem::FullParagraph { para_index }),
            true,
            None,
            measured_tables,
            true,
        );
        if std::env::var_os("RHWP_FLOW_BREAK_DEBUG").is_some() {
            eprintln!(
                "EN_SINGLE_OCCUPIED pi={para_index} bottom={occupied:?} available={available:.3}"
            );
        }
        occupied.is_some_and(|occupied_end| occupied_end > available)
    }
    /// 앞 단에 남긴 제목의 뒤 첫 줄도 같은 점유 끝에서 시작해야 한다.
    /// 실제 첫 줄조차 안 들어가면 다줄 전체를 새 단에서 시작한다.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn reflowed_endnote_head_successor_first_line_overflows(
        &self,
        st: &TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        paragraph: &Paragraph,
        formatted: &FormattedParagraph,
        available: f64,
        column_width: f64,
        measured_tables: &[MeasuredTable],
    ) -> bool {
        if paragraph.column_type != ColumnBreakType::None {
            return false;
        }
        let Some(&first_height) = formatted.line_heights.first() else {
            return false;
        };
        let previous = st
            .current_items
            .iter()
            .rev()
            .filter_map(crate::renderer::typeset::page_item_para_index)
            .next()
            .and_then(|index| {
                crate::renderer::typeset::paragraph_by_global_index(
                    paragraphs,
                    &st.endnote_paragraphs,
                    index,
                )
            });
        let Some(last) = previous.and_then(|p| p.line_segs.last()) else {
            return false;
        };
        self.measure_endnote_column_with_item(
            st,
            paragraphs,
            styles,
            available,
            column_width,
            None,
            true,
            None,
            measured_tables,
        )
        .is_some_and(|geometry| {
            let origin = crate::renderer::endnote_spacing::origin_after_gap(
                geometry.flow_bottom,
                geometry.occupied_bottom,
                crate::renderer::hwpunit_to_px(last.line_spacing.max(0), self.dpi),
            );
            origin + formatted.spacing_before + first_height > available
        })
    }
}
