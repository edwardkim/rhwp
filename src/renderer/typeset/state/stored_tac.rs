//! 저장 TAC 진입에 필요한 확정 문단 내용 하단·소유 조회.
use super::*;

impl TypesetState {
    /// 최초 완전한 표의 source 축은 paint의 단 초기화와 같은 규칙으로 조회한다.
    /// 여기서는 후보만 만들고 실제 pen과의 일치는 수용 계획이 확인한다.
    pub(in crate::renderer::typeset) fn stored_tac_line_frame(
        &self,
        para: &Paragraph,
        next: Option<&Paragraph>,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) -> Option<crate::renderer::float_placement::StoredColumnFrame> {
        use crate::renderer::float_placement::StoredColumnFrame;
        if let Some(first) = self.data.current_items.first() {
            // 단의 첫 완전한 표만 이 축의 소유자다. 글줄로 시작한 단의
            // 중간 표가 한 점에서 맞더라도, 그 뒤 재조판 글줄의 저장 끝까지
            // 유효하다고 인증하면 기존 본문의 source 축을 가로채게 된다.
            let PageItem::Table {
                para_index,
                control_index,
            } = first
            else {
                return None;
            };
            let (source_origin_hu, column_anchor_y) = self.data.vpos_stored_column_origin?;
            return self
                .data
                .inline_placements
                .get(&(*para_index, *control_index))?
                .stored_column_frame
                .filter(|frame| {
                    frame.source_origin_hu == source_origin_hu
                        && frame.column_anchor_y == column_anchor_y
                });
        }
        let dpi = self.data.layout.dpi;
        let source_origin_hu =
            if crate::renderer::stored_page_origin::stored_first_margin_is_page_relative(
                para, next, styles, dpi,
            ) {
                0
            } else {
                crate::renderer::float_placement::stored_table_column_origin(
                    para, next, false, None, dpi,
                )?
            };
        Some(StoredColumnFrame {
            source_origin_hu,
            column_anchor_y: self.data.vpos_col_anchor,
        })
    }

    /// 각주 등록 뒤 단일 단의 vpos 관측값을 기존 순서로 확정한다.
    pub(in crate::renderer::typeset) fn commit_deferred_table_anchor(
        &mut self,
        para_index: usize,
        paragraphs: &[Paragraph],
    ) {
        if self.data.col_count == 1 {
            self.data.vpos_prev_layout_para = Some(para_index);
            // 저장 줄 수용이 발행한 축은 표 뒤의 일반 앵커 초기화로 지우지 않는다.
            // 다른 줄·합성·분할 경로는 이 인증을 갖지 않는다.
            let accepted_frame = match self.data.current_items.last() {
                Some(PageItem::Table {
                    para_index: owner,
                    control_index,
                }) if *owner == para_index => self
                    .data
                    .inline_placements
                    .get(&(*owner, *control_index))
                    .and_then(|placement| placement.stored_column_frame),
                _ => None,
            };
            if let Some(frame) = accepted_frame {
                self.record_vpos_page_origin(Some(frame.source_origin_hu));
                self.record_vpos_lazy_origin(None);
                self.record_vpos_origin_provenance(true);
                self.record_vpos_column_anchor(frame.column_anchor_y);
                self.data.vpos_prev_partial_table = false;
                return;
            }
            if matches!(
                self.data.current_items.last(),
                Some(PageItem::Table { .. } | PageItem::PartialTable { .. })
            ) {
                let preserves_paragraph_frame = match self.data.current_items.last() {
                    Some(PageItem::Table { para_index, control_index }) => paragraphs
                        .get(*para_index)
                        .and_then(|para| para.controls.get(*control_index))
                        .is_some_and(|control| matches!(control, Control::Table(table)
                            if crate::renderer::float_placement::is_para_topbottom_float(&table.common))),
                    _ => false,
                };
                if preserves_paragraph_frame {
                    // Whole paragraph-following tables keep the column's
                    // source axis in paint. A table opening the column owns
                    // the same initial origin as build_single_column.
                    self.seed_whole_table_column_origin(para_index, paragraphs);
                } else {
                    self.data.vpos_page_base = None;
                    self.data.vpos_lazy_base = None;
                }
                self.data.vpos_prev_partial_table = matches!(
                    self.data.current_items.last(),
                    Some(PageItem::PartialTable { .. })
                );
            }
        }
    }

    /// The whole table that opens this column owns paint's initial source axis.
    pub(in crate::renderer::typeset) fn seed_whole_table_column_origin(
        &mut self,
        para_index: usize,
        paragraphs: &[Paragraph],
    ) {
        if let Some(PageItem::Table {
            para_index: first,
            control_index,
        }) = self.data.current_items.first()
        {
            if *first == para_index {
                let has_host_line = self.data.current_items.iter().any(|item| {
                    matches!(
                        item,
                        PageItem::FullParagraph { para_index: owner }
                            | PageItem::PartialParagraph { para_index: owner, start_line: 0, .. }
                            if owner == first
                    )
                }) || paragraphs
                    .get(*first)
                    .is_some_and(crate::renderer::typeset::para_has_visible_text);
                self.data.vpos_page_base = paragraphs.get(*first).and_then(|para| {
                    crate::renderer::float_placement::stored_table_column_origin(
                        para,
                        paragraphs.get(*first + 1),
                        has_host_line,
                        self.data
                            .paragraph_float_placements
                            .get(&(*first, *control_index)),
                        self.data.layout.dpi,
                    )
                });
                self.data.vpos_lazy_base = None;
                self.data.vpos_page_base_stored = self.data.vpos_page_base.is_some()
                    && paragraphs
                        .get(*first)
                        .and_then(|para| para.line_segs.first())
                        .is_some_and(|line| !crate::renderer::typeset::is_synthetic_line_seg(line));
                self.data.vpos_stored_column_origin = self
                    .data
                    .vpos_page_base
                    .map(|base| (base, self.data.vpos_col_anchor));
            }
        }
    }

    /// 저장 TAC 줄 수용에 필요한 읽기 전용 페이지 관측값.
    pub(in crate::renderer::typeset) fn stored_tac_page(
        &self,
        paragraphs: &[crate::model::paragraph::Paragraph],
    ) -> StoredTacPage {
        StoredTacPage {
            original_object_column_origin: self.data.current_items.first().is_some_and(|item| {
                let PageItem::Table {
                    para_index,
                    control_index,
                } = item
                else {
                    return false;
                };
                paragraphs.get(*para_index).is_some_and(|first| {
                    first.source_line_seg_vertical_pos.is_none()
                        && first.line_segs.first().is_some_and(|line| {
                            line.vertical_pos == 0
                                && line.tag
                                    & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                    == 0
                        })
                        && self
                            .data
                            .inline_placements
                            .get(&(*para_index, *control_index))
                            .is_some_and(|placed| placed.y == 0.0 && placed.advance_end.is_some())
                })
            }),
            profile: self.data.profile,
            rejected_stored_forward_origin: self.data.rejected_stored_forward_origin,
            current_height: self.data.current_height,
            empty_column: self.data.current_items.is_empty(),
            vpos_col_anchor: self.data.vpos_col_anchor,
            vpos_page_base: self.data.vpos_page_base,
            vpos_lazy_base: self.data.vpos_lazy_base,
            side_wrap_empty: self.data.side_wrap_exclusions.is_empty(),
            previous_content_bottom: self.previous_text_content_bottom(paragraphs),
            previous_page_tail: (self.data.col_count == 1 && self.data.current_items.is_empty())
                .then(|| {
                    let page = self.data.pages.iter().rev().nth(1)?;
                    if page.section_index != self.data.section_index {
                        return None;
                    }
                    let item = page.column_contents.last()?.items.last()?;
                    let (owner, end) = match item {
                        PageItem::FullParagraph { para_index }
                        | PageItem::Table { para_index, .. } => (
                            *para_index,
                            paragraphs.get(*para_index)?.line_segs.len(),
                        ),
                        PageItem::PartialParagraph { para_index, end_line, .. } =>
                            (*para_index, *end_line),
                        _ => return None,
                    };
                    Some((owner, end))
                })
                .flatten(),
            stored_zero_column_origin: !self.data.current_endnote_flow
                && !self.data.vpos_ladder_dirty
                && self.data.current_start_height == 0.0
                && self.data.vpos_col_anchor == 0.0
                && self.data.current_items.first().is_some_and(|item| {
                    let PageItem::FullParagraph { para_index } = item else {
                        return false;
                    };
                    paragraphs.get(*para_index).is_some_and(|para| {
                        !para.stored_text_partition_is_dirty()
                            && !para.cell_format_vpos_dirty
                            && para.line_segs.first().is_some_and(|seg| {
                                seg.vertical_pos == 0
                                    && seg.tag
                                        & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                        == 0
                            })
                    })
                }),
            stored_text_column_frame: self.data.current_items.first().and_then(|item| {
                let (owner, start) = match item {
                    PageItem::FullParagraph { para_index } => (*para_index, 0),
                    PageItem::PartialParagraph { para_index, start_line, .. } =>
                        (*para_index, *start_line),
                    _ => return None,
                };
                let paragraph = paragraphs.get(owner)?;
                let line = paragraph.line_segs.get(start)?;
                if paragraph.stored_text_partition_is_dirty()
                    || paragraph.cell_format_vpos_dirty
                    || crate::renderer::typeset::is_synthetic_line_seg(line)
                {
                    return None;
                }
                // 다단은 HeightCursor의 단일 단 초기화를 거치지 않을 수 있다.
                // 그때도 첫 수용 글줄의 원문 좌표와 같은 물리 앵커가 단 프레임을 정한다.
                let (source_origin_hu, column_anchor_y) = self
                    .data
                    .vpos_stored_column_origin
                    .unwrap_or((line.vertical_pos, self.data.vpos_col_anchor));
                Some(crate::renderer::float_placement::StoredColumnFrame {
                    source_origin_hu,
                    column_anchor_y,
                })
            }),
            // layout의 단 초기 base와 같은 첫 완전한 표의 저장 줄을 읽는다.
            // PartialTable에는 소비된 컷이 있고, 문단 시작의 들여쓰기·앞 간격은
            // 별도 계약이므로 이 표 원점 조회에 섞지 않는다.
            stored_table_column_base: self.data.current_items.first().and_then(|item| {
                let PageItem::Table { para_index, .. } = item else {
                    return None;
                };
                paragraphs
                    .get(*para_index)
                    .and_then(|para| para.line_segs.first())
                    .filter(|seg| {
                        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    })
                    .map(|seg| seg.vertical_pos)
            }),
        }
    }

    /// 같은 문단의 인라인 표시와 흐름을 전진시키지 않은 표 뒤에서도 글줄 소유를 유지한다.
    /// 공간을 소비한 표나 다른 문단을 넘어 앞 내용 하단을 조회하지 않는다.
    fn previous_text_content_bottom(
        &self,
        paragraphs: &[crate::model::paragraph::Paragraph],
    ) -> Option<(usize, usize, f64, Option<f64>)> {
        let mut following_owner = None;
        for item in self.data.current_items.iter().rev() {
            let (owner, end_line) = match item {
                PageItem::Shape {
                    para_index,
                    control_index,
                } => {
                    let control = paragraphs.get(*para_index)?.controls.get(*control_index)?;
                    if !crate::renderer::typeset::controls::stored_tac::inline_text_control(control)
                        || following_owner.is_some_and(|owner| owner != *para_index)
                    {
                        return None;
                    }
                    following_owner = Some(*para_index);
                    continue;
                }
                PageItem::Table {
                    para_index,
                    control_index,
                } => {
                    let para = paragraphs.get(*para_index)?;
                    let bounds = self
                        .data
                        .paragraph_fragment_content_bottoms
                        .get(&(*para_index, para.line_segs.len()))?;
                    // 표가 독립 배치되어도 글줄은 별도 소유다. 같은 완전한 본문 조각의
                    // 실제 전진 끝이 현재 커서와 같을 때만 표가 흐름을 늘리지 않았음을 안다.
                    if !matches!(para.controls.get(*control_index), Some(Control::Table(table))
                        if !table.common.treat_as_char)
                        || bounds.flow_end != self.data.current_height
                        || following_owner.is_some_and(|owner| owner != *para_index)
                    {
                        return None;
                    }
                    following_owner = Some(*para_index);
                    continue;
                }
                PageItem::FullParagraph { para_index } => {
                    (*para_index, paragraphs.get(*para_index)?.line_segs.len())
                }
                PageItem::PartialParagraph {
                    para_index,
                    end_line,
                    ..
                } => (*para_index, *end_line),
                _ => return None,
            };
            if following_owner.is_some_and(|following_owner| following_owner != owner) {
                return None;
            }
            return self
                .data
                .paragraph_fragment_content_bottoms
                .get(&(owner, end_line))
                .map(|bounds| {
                    (
                        owner,
                        end_line,
                        bounds.content_bottom,
                        bounds.last_line_spacing,
                    )
                });
        }
        None
    }
}
