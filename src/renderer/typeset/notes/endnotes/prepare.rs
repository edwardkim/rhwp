//! 미주 시작의 구분자·간격·되감기 준비와 carry 조정.

use crate::renderer::typeset::notes::endnotes::profile::{
    endnote_between_notes_margin, endnote_between_notes_pagination_margin,
    endnote_has_visible_separator, endnote_separator_below_margin, EndnoteFlowProfile,
    ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU,
};
use crate::renderer::typeset::notes::endnotes::types::{EndnoteEmitVars, EndnotePrepCarry};
use crate::renderer::typeset::{
    hwpunit_to_px, line_has_visible_text, line_has_visible_text_or_tac_equation,
    line_tac_picture_or_shape_height, para_has_visible_text, para_has_visible_text_or_equation,
    Control, EndnoteRef, FootnoteShape, PageItem, Paragraph, ResolvedStyleSet, TypesetEngine,
    TypesetState, ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

impl TypesetEngine {
    /// [#2026 추출] 미주-당 프리앰블 — 구분자 방출, 미주-간 간격/되감기 플래그 산출,
    /// `EndnoteEmitVars` 생산. 원본 무변경 이동 (current_… 선언만 캐리 리셋 대입으로 전환).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn prepare_endnote_emit(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        section_index: usize,
        endnote_shape: Option<&FootnoteShape>,
        endnote_flow_profile: Option<EndnoteFlowProfile>,
        compact_endnote_separator_profile: bool,
        emitted_endnote_count: usize,
        last_render_endnote_para_local_idx: Option<usize>,
        cleared_single_line_internal_rewind_split: bool,
        prev_endnote_had_inline_object_vpos_overestimate: bool,
        en_ref: &EndnoteRef,
        en_ctrl: &crate::model::footnote::Endnote,
        carry: EndnotePrepCarry,
    ) -> (EndnoteEmitVars, EndnotePrepCarry) {
        let EndnotePrepCarry {
            mut vpos_offset,
            mut prev_en_bottom_vpos,
            mut prev_en_content_bottom_vpos,
            mut prev_endnote_had_vpos_rewind,
            mut emitted_endnote_separator,
            mut current_endnote_had_inline_object_vpos_overestimate,
        } = carry;
        if !emitted_endnote_separator {
            if let (Some(shape), Some(profile)) = (endnote_shape, endnote_flow_profile) {
                let sep_height = profile.separator_height_px(self.dpi);
                if sep_height > 0.0 {
                    st.append_item(PageItem::EndnoteSeparator {
                        separator_length: shape.separator_length,
                        margin_above: shape.separator_above_margin_hu(),
                        margin_below: endnote_separator_below_margin(shape),
                        line_type: shape.separator_line_type,
                        line_width: shape.separator_line_width,
                        color: shape.separator_color,
                    });
                    st.mark_endnote_flow();
                    if !profile.compact_separator_below {
                        st.advance_flow_by(sep_height);
                        st.record_column_flow_origin(st.current_height);
                    }
                }
            }
            emitted_endnote_separator = true;
        }
        let rewind_group_advance_threshold = if st.current_column + 1 < st.col_count {
            0.85
        } else {
            0.95
        };
        let default_nonzero_between_note_tail_candidate = endnote_flow_profile
            .map(EndnoteFlowProfile::nonzero_default_between_notes)
            .unwrap_or(false)
            && en_ref.number > 0;
        let default_late_question_group_tail = compact_endnote_separator_profile
            && endnote_shape
                .map(|shape| {
                    endnote_between_notes_margin(shape) as i32 <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && default_nonzero_between_note_tail_candidate
            && st.current_column + 1 >= st.col_count;
        let default_question_group_head_tail = compact_endnote_separator_profile
            && prev_endnote_had_inline_object_vpos_overestimate
            && endnote_shape
                .map(|shape| {
                    endnote_between_notes_margin(shape) as i32 <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && {
                let head_h: f64 = en_ctrl
                    .paragraphs
                    .iter()
                    .take(2)
                    .filter_map(|p| {
                        let first = p.line_segs.first()?.vertical_pos;
                        let bottom = p
                            .line_segs
                            .iter()
                            .map(|s| {
                                s.vertical_pos
                                    .saturating_add(s.line_height)
                                    .saturating_add(s.line_spacing)
                            })
                            .max()?;
                        Some(hwpunit_to_px((bottom - first).max(0), self.dpi))
                    })
                    .sum();
                head_h > 0.0 && st.current_height + head_h <= st.available_height() - 8.0
            };
        // 기본 7mm 미주는 제목 한 줄 tail을 허용하되, 빈/TAC 식만
        // 뒤따르는 orphan 제목은 frame overflow로 이어지므로 제외한다.
        let default_question_group_title_tail = compact_endnote_separator_profile
            && endnote_shape
                .map(|shape| {
                    endnote_between_notes_margin(shape) as i32 <= ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                })
                .unwrap_or(false)
            && en_ref.number > 0
            && !st.current_items.is_empty()
            && en_ctrl.paragraphs.first().is_some_and(|head| {
                if head.line_segs.len() != 1 {
                    return false;
                }
                let title_h = hwpunit_to_px(
                    head.line_segs[0].line_height + head.line_segs[0].line_spacing,
                    self.dpi,
                );
                let title_fits = title_h > 0.0
                    && st.current_height + title_h
                        <= st.available_height() + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0;
                if !title_fits {
                    return false;
                }
                if st.current_column + 1 >= st.col_count {
                    return en_ctrl
                        .paragraphs
                        .get(1)
                        .map(para_has_visible_text)
                        .unwrap_or(true);
                }
                if !default_nonzero_between_note_tail_candidate {
                    return false;
                }
                let mut head_h = 0.0;
                let mut head_count = 0usize;
                for para in en_ctrl.paragraphs.iter().take(4) {
                    let Some(first) = para.line_segs.first() else {
                        continue;
                    };
                    let Some(bottom) = para
                        .line_segs
                        .iter()
                        .map(|seg| {
                            seg.vertical_pos
                                .saturating_add(seg.line_height)
                                .saturating_add(seg.line_spacing)
                        })
                        .max()
                    else {
                        continue;
                    };
                    head_h += hwpunit_to_px((bottom - first.vertical_pos).max(0), self.dpi);
                    head_count += 1;
                }
                head_count >= 2
                    && st.current_height + head_h
                        <= st.available_height() + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            });
        let zero_question_group_title_tail = compact_endnote_separator_profile
            && prev_endnote_had_vpos_rewind
            && st.current_column + 1 < st.col_count
            && endnote_shape
                .map(|shape| {
                    shape.separator_above_margin_hu() == 0
                        && endnote_between_notes_margin(shape) == 0
                        && endnote_separator_below_margin(shape) == 0
                        && endnote_has_visible_separator(shape)
                })
                .unwrap_or(false)
            && en_ctrl
                .paragraphs
                .first()
                .map(|p| {
                    let en_col_w = st
                        .layout
                        .column_areas
                        .get(st.current_column as usize)
                        .map(|a| a.width)
                        .unwrap_or(st.layout.body_area.width);
                    let comp = crate::renderer::composer::compose_paragraph_in_context(p, styles);
                    let fmt =
                        self.format_endnote_paragraph(p, Some(&comp), &styles, Some(en_col_w));
                    fmt.line_heights.len() == 1
                        && line_has_visible_text_or_tac_equation(p, &comp, 0)
                        && st.current_height + fmt.line_advance(0)
                            <= st.available_height() + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                })
                .unwrap_or(false);
        let zero_between_question_group_title_tail = compact_endnote_separator_profile
            && prev_endnote_had_vpos_rewind
            && st.current_column + 1 < st.col_count
            && endnote_shape
                .map(|shape| {
                    endnote_has_visible_separator(shape) && endnote_between_notes_margin(shape) == 0
                })
                .unwrap_or(false)
            && en_ref.number > 0
            && !st.current_items.is_empty()
            && en_ctrl.paragraphs.first().is_some_and(|head| {
                head.line_segs.first().is_some_and(|seg| {
                    let title_h = hwpunit_to_px(
                        (seg.line_height.saturating_add(seg.line_spacing)).max(0),
                        self.dpi,
                    );
                    title_h > 0.0
                        && st.current_height + title_h
                            <= st.available_height()
                                + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                                + 2.0
                })
            });
        let visible_large_between_question_group_title_tail = compact_endnote_separator_profile
            && prev_endnote_had_vpos_rewind
            && st.current_column + 1 < st.col_count
            && endnote_flow_profile
                .map(EndnoteFlowProfile::visible_non_default_between_notes)
                .unwrap_or(false)
            && en_ref.number > 0
            && !st.current_items.is_empty()
            && en_ctrl.paragraphs.first().is_some_and(|head| {
                if head.line_segs.len() != 1 {
                    return false;
                }
                let Some(first) = head.line_segs.first() else {
                    return false;
                };
                let title_h = hwpunit_to_px(
                    (first.line_height.saturating_add(first.line_spacing)).max(0),
                    self.dpi,
                );
                title_h > 0.0
                    && st.current_height + title_h
                        <= st.available_height() + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX + 2.0
            });
        if st.col_count > 1
            && compact_endnote_separator_profile
            && !st.current_items.is_empty()
            && prev_endnote_had_vpos_rewind
            && !default_late_question_group_tail
            && !default_question_group_head_tail
            && !default_question_group_title_tail
            && !zero_question_group_title_tail
            && !zero_between_question_group_title_tail
            && !visible_large_between_question_group_title_tail
            && st.current_height > st.available_height() * rewind_group_advance_threshold
        {
            let group_first = en_ctrl
                .paragraphs
                .iter()
                .filter_map(|p| p.line_segs.first().map(|s| s.vertical_pos))
                .min();
            let group_bottom = en_ctrl
                .paragraphs
                .iter()
                .flat_map(|p| {
                    p.line_segs.iter().map(|s| {
                        s.vertical_pos
                            .saturating_add(s.line_height)
                            .saturating_add(s.line_spacing)
                    })
                })
                .max();
            if let (Some(first), Some(bottom)) = (group_first, group_bottom) {
                let group_h = hwpunit_to_px((bottom - first).max(0), self.dpi);
                let available = st.available_height();
                if group_h > 0.0
                    && group_h <= available + 0.5
                    && st.current_height + group_h > available
                {
                    let reclaimed = (available - st.current_height).max(0.0);
                    st.advance_column_or_new_page();
                    st.reclaim_flow_by(reclaimed);
                    st.record_column_flow_origin(st.current_height);
                    st.mark_endnote_flow();
                    st.reset_vpos_cursor();
                    prev_en_bottom_vpos = None;
                    prev_en_content_bottom_vpos = None;
                }
            }
        }
        let boundary_prev_endnote_had_vpos_rewind = prev_endnote_had_vpos_rewind;
        let mut prev_group_bottom: Option<i32> = None;
        let endnote_has_vpos_rewind = en_ctrl.paragraphs.iter().any(|p| {
            let internal_rewind = p
                .line_segs
                .windows(2)
                .any(|w| w[1].vertical_pos < w[0].vertical_pos);
            let first = p.line_segs.first().map(|s| s.vertical_pos);
            let bottom = p
                .line_segs
                .iter()
                .map(|s| {
                    s.vertical_pos
                        .saturating_add(s.line_height)
                        .saturating_add(s.line_spacing)
                })
                .max();
            let group_rewind = matches!(
                (prev_group_bottom, first),
                (Some(prev), Some(cur)) if cur < prev
            );
            if let Some(b) = bottom {
                prev_group_bottom = Some(b);
            }
            internal_rewind || group_rewind
        });
        prev_endnote_had_vpos_rewind = endnote_has_vpos_rewind;
        current_endnote_had_inline_object_vpos_overestimate = false;
        let continued_endnote_tail_before_new_note =
            st.current_endnote_flow && !st.current_items.is_empty();

        // endnote 단위로 시작점 결정
        if emitted_endnote_count > 0 {
            if let (Some(shape), Some(prev_local_idx)) =
                (endnote_shape, last_render_endnote_para_local_idx)
            {
                let between_notes = endnote_between_notes_margin(shape) as i32;
                if between_notes > 0 {
                    // [Task #1246] 섹션 미주 between-notes 마진(HU)을 보관 →
                    // HeightCursor 가 미주 사이 min-gap 보정에 사용. 모든 경계 동일값.
                    st.record_endnote_between_margin(between_notes);
                    let prev_spacing = st
                        .endnote_paragraphs
                        .get(prev_local_idx)
                        .and_then(|p| p.line_segs.last())
                        .map(|s| s.line_spacing.max(0))
                        .unwrap_or(0);
                    let extra_gap = (between_notes - prev_spacing).max(0);
                    let large_rewind_equation_tail_between_notes_boundary = {
                        let visible_large_profile = endnote_flow_profile
                            .map(EndnoteFlowProfile::visible_large_between_notes)
                            .unwrap_or(false);
                        let previous_tail_is_equation_only = st
                            .endnote_paragraphs
                            .get(prev_local_idx)
                            .map(|prev_para| {
                                !para_has_visible_text(prev_para)
                                    && para_has_visible_text_or_equation(prev_para)
                            })
                            .unwrap_or(false);
                        visible_large_profile
                            && boundary_prev_endnote_had_vpos_rewind
                            && continued_endnote_tail_before_new_note
                            && previous_tail_is_equation_only
                            && st.current_height < st.available_height() * 0.35
                    };
                    let large_equation_tail_before_tac_head_boundary = {
                        let visible_large_profile = endnote_flow_profile
                            .map(EndnoteFlowProfile::visible_large_between_notes)
                            .unwrap_or(false);
                        let previous_tail_is_large_equation_only = st
                            .endnote_paragraphs
                            .get(prev_local_idx)
                            .map(|prev_para| {
                                !para_has_visible_text(prev_para)
                                    && prev_para
                                        .line_segs
                                        .last()
                                        .map(|seg| seg.line_height >= 3000)
                                        .unwrap_or(false)
                            })
                            .unwrap_or(false);
                        let current_head_has_large_tac_picture =
                            en_ctrl.paragraphs.iter().take(8).any(|head_para| {
                                let head_comp =
                                    crate::renderer::composer::compose_paragraph_in_context(
                                        head_para, styles,
                                    );
                                (0..head_comp.lines.len()).any(|line_idx| {
                                    !line_has_visible_text(&head_comp, line_idx)
                                        && line_tac_picture_or_shape_height(
                                            head_para, &head_comp, line_idx, self.dpi,
                                        )
                                        .is_some_and(|height| height >= 80.0)
                                })
                            });
                        visible_large_profile
                            && !large_rewind_equation_tail_between_notes_boundary
                            && endnote_has_visible_separator(shape)
                            && continued_endnote_tail_before_new_note
                            && previous_tail_is_large_equation_only
                            && current_head_has_large_tac_picture
                            && st.col_count > 1
                            && st.current_column + 1 >= st.col_count
                            && st.current_height > st.available_height() * 0.45
                            && st.current_height < st.available_height() * 0.65
                    };
                    if std::env::var("RHWP_ENDNOTE_BOUNDARY_DEBUG").is_ok() {
                        eprintln!(
                            "ENDNOTE_BOUNDARY note={} src=s{}:p{}:ci{} emitted={} col={}/{} cur={:.2} avail={:.2} between={} prev_spacing={} extra={} large_rewind={} large_tac_head={} continued={} visible_sep={}",
                            en_ref.number,
                            en_ref.section_index,
                            en_ref.para_index,
                            en_ref.control_index,
                            emitted_endnote_count,
                            st.current_column + 1,
                            st.col_count,
                            st.current_height,
                            st.available_height(),
                            between_notes,
                            prev_spacing,
                            extra_gap,
                            large_rewind_equation_tail_between_notes_boundary,
                            large_equation_tail_before_tac_head_boundary,
                            continued_endnote_tail_before_new_note,
                            endnote_has_visible_separator(shape),
                        );
                    }
                    if extra_gap > 0 {
                        // split=1 내부 rewind를 가짜 단 분할로 보고 해소한 뒤에는
                        // 그 분할이 만들던 암묵적 여백이 사라진다. 큰 미주 사이
                        // 문서에서는 다음 미주 경계부터 전체 between-notes 값을
                        // 예약해 PDF의 24쪽 흐름을 유지한다.
                        // 보이는 구분선이 없는 미주는 renderer가 이전 문단
                        // line_spacing에 전체 "미주 사이"를 반영한다. pagination도
                        // 같은 전체 gap을 써야 첫 단 하단에서 under-count가 생기지 않는다.
                        let visible_separator_tail_continues_current_column =
                            endnote_has_visible_separator(shape)
                                && continued_endnote_tail_before_new_note
                                && st.current_height < st.available_height() * 0.25
                                && between_notes > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU;
                        let pagination_gap = if visible_separator_tail_continues_current_column {
                            // 같은 단/쪽에 직전 미주 tail이 이미 이어져 있으면
                            // 직전 문단 line_spacing이 "미주 사이"를 대표한다.
                            // 여기서 vpos_offset까지 다시 밀면 다음 번호가
                            // 한컴보다 약 미주사이만큼 아래로 내려간다.
                            0
                        } else if large_rewind_equation_tail_between_notes_boundary {
                            // 내부 vpos 되감김으로 현재 쪽 상단에 이어진
                            // 수식 tail은 저장 vpos와 기본 lineSeg 흐름이
                            // 이미 경계를 만든다. 초과 pagination gap까지
                            // 더하면 다음 문항 제목이 한 gap만큼 늦어진다.
                            0
                        } else if between_notes > ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU
                            && (cleared_single_line_internal_rewind_split
                                || !endnote_has_visible_separator(shape))
                        {
                            between_notes
                        } else {
                            endnote_flow_profile
                                .map(EndnoteFlowProfile::pagination_between_notes_margin)
                                .unwrap_or_else(|| endnote_between_notes_pagination_margin(shape))
                        };
                        if pagination_gap > 0 {
                            vpos_offset += pagination_gap;
                        }
                        let skip_default_render_between_notes_trailing = endnote_flow_profile
                            .map(|profile| {
                                profile.visible_nonzero_default_between_notes()
                                    && profile.large_separator_margin()
                            })
                            .unwrap_or(false)
                            && continued_endnote_tail_before_new_note
                            && st.current_height > st.available_height() * 0.70
                            && st.current_height < st.available_height() * 0.75;
                        let skip_default_mid_column_between_notes_trailing = endnote_flow_profile
                            .map(EndnoteFlowProfile::visible_nonzero_default_between_notes)
                            .unwrap_or(false)
                            && boundary_prev_endnote_had_vpos_rewind
                            && continued_endnote_tail_before_new_note
                            && st.current_column + 1 >= st.col_count
                            && st.current_height > st.available_height() * 0.25
                            && st.current_height < st.available_height() * 0.50;
                        let skip_absorbed_render_between_notes_trailing = {
                            let absorbed_visible_profile = endnote_flow_profile
                                .map(|profile| {
                                    profile.visible_separator && profile.absorbed_between_notes_gap
                                })
                                .unwrap_or(false);
                            let absorbed_tail_continues_at_column_top =
                                st.current_height < st.available_height() * 0.25;
                            let absorbed_tail_near_column_bottom =
                                st.current_height > st.available_height() * 0.65;
                            let absorbed_short_tac_tail = st
                                .endnote_paragraphs
                                .get(prev_local_idx)
                                .map(|prev_para| {
                                    let last_line_height = prev_para
                                        .line_segs
                                        .last()
                                        .map(|seg| seg.line_height)
                                        .unwrap_or(0);
                                    !para_has_visible_text(prev_para)
                                        && last_line_height <= 1200
                                        && prev_para.controls.iter().any(|ctrl| {
                                            matches!(
                                                ctrl,
                                                Control::Equation(eq)
                                                    if eq.common.treat_as_char
                                            )
                                        })
                                })
                                .unwrap_or(false);
                            absorbed_visible_profile
                                && boundary_prev_endnote_had_vpos_rewind
                                && continued_endnote_tail_before_new_note
                                && (absorbed_tail_continues_at_column_top
                                    || (st.current_column + 1 >= st.col_count
                                        && absorbed_tail_near_column_bottom)
                                    || (absorbed_short_tac_tail
                                        && st.current_height > st.available_height() * 0.80))
                        };
                        let skip_render_between_notes_trailing =
                            skip_default_render_between_notes_trailing
                                || skip_default_mid_column_between_notes_trailing
                                || skip_absorbed_render_between_notes_trailing;
                        if st
                            .endnote_paragraphs
                            .get(prev_local_idx)
                            .is_some_and(|p| !p.line_segs.is_empty())
                        {
                            if !skip_render_between_notes_trailing {
                                // 내부 vpos 되감김으로 현재 단/쪽 상단에 이어진
                                // 수식 tail은 저장 lineSeg 흐름에 기본 gap이 이미
                                // 포함되어 있다. 20mm 전체를 render tail에 다시
                                // 주입하면 다음 제목이 한 note gap만큼 내려간다.
                                let render_between_notes =
                                    if large_rewind_equation_tail_between_notes_boundary {
                                        ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU.max(prev_spacing)
                                    } else {
                                        between_notes
                                    };
                                st.apply_endnote_render_tail_spacing(
                                    prev_local_idx,
                                    render_between_notes,
                                );
                            }
                        }
                    }
                }
            }
        }
        (
            EndnoteEmitVars {
                boundary_prev_endnote_had_vpos_rewind,
                endnote_has_vpos_rewind,
                continued_endnote_tail_before_new_note,
                default_nonzero_between_note_tail_candidate,
                default_question_group_title_tail,
                compact_endnote_separator_profile,
                prev_endnote_had_inline_object_vpos_overestimate,
                endnote_start: vpos_offset,
            },
            EndnotePrepCarry {
                vpos_offset,
                prev_en_bottom_vpos,
                prev_en_content_bottom_vpos,
                prev_endnote_had_vpos_rewind,
                emitted_endnote_separator,
                current_endnote_had_inline_object_vpos_overestimate,
            },
        )
    }
}
