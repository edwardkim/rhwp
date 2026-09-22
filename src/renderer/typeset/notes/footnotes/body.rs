//! 본문 각주 등록 조정. 앵커/현재/다음 쪽 선택 뒤 원래 예약 순서로 확정한다.

use crate::renderer::typeset::notes::footnotes::boundary::{
    native_hwp5_body_footnote_tail_reset, native_hwp5_final_marker_footnote_uses_next_reset_page,
    native_hwp5_footnote_fragment_height, native_hwp5_footnote_reset_fragments,
    native_hwp5_two_line_footnote_fragments, NativeHwp5FootnoteFragmentSplit,
};
use crate::renderer::typeset::notes::footnotes::measure::composed_footnote_content_height;
use crate::renderer::typeset::{
    estimate_footnote_note_height, hwpunit_to_px,
    native_hwp5_rowbreak_host_precedes_first_fragment, Control, Footnote, FootnoteRef,
    FootnoteSource, Paragraph, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn register_body_footnote(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        ctrl_idx: usize,
        fn_ctrl: &Footnote,
        has_table: bool,
        native_hwp5_footnote_break: Option<super::boundary::NativeHwp5FootnoteBreak>,
    ) {
        let source = FootnoteSource::Body {
            para_index: para_idx,
            control_index: ctrl_idx,
        };
        let native_table_host_footnote = if st.profile.hwp5_stored_pagination_layout()
            && st.col_count == 1
            && para
                .controls
                .iter()
                .filter(|control| matches!(control, Control::Footnote(_)))
                .count()
                == 1
        {
            let mut top_level_tables =
                para.controls
                    .iter()
                    .enumerate()
                    .filter_map(|(index, control)| {
                        if let Control::Table(table) = control {
                            Some((index, table))
                        } else {
                            None
                        }
                    });
            match (top_level_tables.next(), top_level_tables.next()) {
                (Some((table_control_index, table)), None)
                    if table_control_index + 1 == ctrl_idx
                        && native_hwp5_rowbreak_host_precedes_first_fragment(para, table) =>
                {
                    let full_content_height = composed_footnote_content_height(fn_ctrl, self.dpi);
                    st.native_table_host_terminal_fragment_placement(
                        para_idx,
                        table_control_index,
                        table.row_count,
                    )
                    .map(|terminal_fragment_is_current| {
                        let split = native_hwp5_footnote_reset_fragments(fn_ctrl, self.dpi)
                            .filter(|split| split.force_next_page && st.col_count == 1);
                        let (content_height, draw_separator) = split
                            .map(|split| (split.prefix_height, split.prefix.draw_separator))
                            .unwrap_or_else(|| (full_content_height, true));
                        (
                            terminal_fragment_is_current,
                            content_height,
                            draw_separator,
                            full_content_height,
                        )
                    })
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some((
            terminal_fragment_is_current,
            content_height,
            draw_separator,
            full_content_height,
        )) = native_table_host_footnote
        {
            let overlap_guard = if terminal_fragment_is_current {
                32.0
            } else {
                0.0
            };
            if !st.footnote_fragment_fits_current_page(
                content_height,
                draw_separator,
                true,
                overlap_guard,
            ) {
                // terminal 표가 이미 확정된 page의 body/각주 lane을
                // 소급 축소하지 않는다. 각주가 들어갈 공간이 없으면 표
                // owner를 먼저 flush하고 fresh page에 보존한다.
                st.force_new_page();
            }
            // 표 셀 내부 각주는 table fragment queue가 별도로 등록하지만,
            // 넘버링 caption 뒤의 최상위 형제 각주는 Body source다.
            // 일반 Body marker-page 소급 heuristic을 타지 않는다
            // (정책연구 p87/p91/p95의 note 138/142/147).
            self.register_unqueued_table_footnote_with_content_height(
                st,
                fn_ctrl,
                source,
                full_content_height,
            );
        } else if !has_table {
            let fn_height = estimate_footnote_note_height(fn_ctrl, self.dpi);
            let move_to_next_reset_page = native_hwp5_final_marker_footnote_uses_next_reset_page(
                st, para_idx, para, paragraphs, ctrl_idx, fn_ctrl, fn_height,
            );
            let body_tail_reset =
                native_hwp5_body_footnote_tail_reset(st, para_idx, para, ctrl_idx);
            // 두 줄 각주는 full-note owner route와 다른 계약이다. marker
            // anchor에 첫 line을 소급하고 tail current page에 두 번째 line을
            // 두는 기존 fallback을 보존한다(p31 note 30).
            let collision_routed = native_hwp5_footnote_break.map(|_| {
                crate::renderer::pagination::find_inline_control_target_page(
                    &st.pages,
                    &st.current_items,
                    para_idx,
                    ctrl_idx,
                    para,
                )
            });
            // 본문 문단이 이미 physical page로 나뉜 뒤 처리되는 Body
            // Footnote는, anchor page가 기존 각주를 이미 가진 경우에는 marker가
            // 든 PartialParagraph의 owner를 따라야 한다. 그렇지 않으면 p52의
            // note 60처럼 marker는 p52에 남고 각주는 tail page p53에 등록된다.
            // 첫 각주가 없는 page까지 일반화하면 p62 note 74처럼 기존 흐름에서
            // reserve하던 각주가 빠져 후속 page break를 바꾼다. native HWP5의
            // multi-note stored LINE_SEG ownership만 대상으로 하여 다른 profile과
            // first-note 흐름은 바꾸지 않는다.
            let multi_note_routed = if st.profile.hwp5_stored_pagination_layout() {
                crate::renderer::pagination::find_inline_control_target_page(
                    &st.pages,
                    &st.current_items,
                    para_idx,
                    ctrl_idx,
                    para,
                )
                .filter(|(page_idx, _)| {
                    st.pages
                        .get(*page_idx)
                        .is_some_and(|page| !page.footnotes.is_empty())
                })
            } else {
                None
            };
            // marker 본문이 이미 두 physical page로 나뉜 경우, 각주 자체의
            // stored reset도 일대일이면 prefix는 marker page, suffix는 현재
            // tail page가 소유한다(p129 note 176). 표 셀용으로 검증된 같은
            // 보수적 판정을 재사용하되 Body tail과 기존 marker-page 각주를
            // 모두 확인해 일반 각주를 임의 capacity로 나누지 않는다.
            let source_reset_fragments = st
                .profile
                .hwp5_stored_pagination_layout()
                .then(|| native_hwp5_footnote_reset_fragments(fn_ctrl, self.dpi))
                .flatten();
            let stored_reset_fragments = body_tail_reset
                .filter(|_| multi_note_routed.is_some())
                .and(source_reset_fragments);
            // body marker와 문단은 현재 page에 끝났지만 각주 stored line이
            // `0 -> 0`으로 다시 시작하면 suffix의 물리 owner만 다음 page다
            // (p178 note 240). p30처럼 본문 자체가 이미 split된 두 줄 각주는
            // 기존 collision route가 맡는다.
            let current_page_reset_fragments = source_reset_fragments.filter(|split| {
                split.force_next_page
                    && body_tail_reset.is_none()
                    && native_hwp5_footnote_break.is_none()
                    && crate::renderer::pagination::find_inline_control_target_page(
                        &st.pages,
                        &st.current_items,
                        para_idx,
                        ctrl_idx,
                        para,
                    )
                    .is_none()
            });
            let fragments = native_hwp5_footnote_break
                .filter(|footnote_break| footnote_break.split_footnote)
                .and_then(|_| native_hwp5_two_line_footnote_fragments(fn_ctrl))
                .map(|(prefix, suffix)| NativeHwp5FootnoteFragmentSplit {
                    prefix_height: native_hwp5_footnote_fragment_height(fn_ctrl, prefix, self.dpi),
                    suffix_height: native_hwp5_footnote_fragment_height(fn_ctrl, suffix, self.dpi),
                    prefix,
                    suffix,
                    // 이 fallback은 두 줄 각주의 일반 분할이며,
                    // stored page-top reset을 판독한 경로가 아니다.
                    force_next_page: false,
                })
                .or(stored_reset_fragments);
            // 기존 각주가 있는 marker page의 마지막 prefix line에서 바로
            // body reset이 시작되고 각주 자체에는 fragment reset이 없으면,
            // 한컴은 새 각주 전체를 tail page로 보낸다(p131 note 180).
            // marker가 reset보다 더 앞선 일반 split과 내부 reset 각주는 각각
            // 기존 marker owner/fragment 경로를 유지한다.
            let whole_note_owned_by_tail_page = stored_reset_fragments.is_none()
                && multi_note_routed.is_some()
                && body_tail_reset.is_some_and(|(marker_line, reset_line)| {
                    marker_line + 1 == reset_line
                        && para.line_segs.get(marker_line).is_some_and(|line| {
                            hwpunit_to_px(
                                line.vertical_pos.saturating_add(line.line_height),
                                self.dpi,
                            ) >= st.layout.body_area.height * 0.90
                        })
                });
            // first-footnote collision은 기존에 `Some(None)` 자체도
            // “현재 page에 둔다”는 결정이었다. 그 결정을 multi-note
            // filter로 덮으면 p30 note 29가 tail p31에 떨어진다.
            let routed = if whole_note_owned_by_tail_page {
                None
            } else {
                match collision_routed {
                    Some(route) => route,
                    None => multi_note_routed,
                }
            };
            let fragment_routed_page = fragments.as_ref().and_then(|_| {
                collision_routed
                    .flatten()
                    .or(multi_note_routed)
                    .or_else(|| {
                        // reset 직전 줄 안의 marker라도 current item의 char 범위를
                        // 찾지 못하면 일반 라우터는 tail page를 가리킨다. 이 좁은
                        // HWP5 형상은 저장 reset 전 page가 첫 footnote line owner다.
                        st.pages.len().checked_sub(2).map(|page_idx| (page_idx, 0))
                    })
            });
            if let Some((page_idx, _)) = fragment_routed_page {
                if let Some(split) = fragments {
                    if st.pages.len() > page_idx + 1
                        && st.add_footnote_fragment_to_completed_page(
                            page_idx,
                            fn_ctrl.number,
                            source.clone(),
                            split.prefix,
                            split.prefix_height,
                        )
                    {
                        if let Some(page) = st.pages.last_mut() {
                            page.footnotes.push(FootnoteRef {
                                number: fn_ctrl.number,
                                source: source.clone(),
                                fragment: Some(split.suffix),
                            });
                        }
                        st.add_footnote_fragment_height(
                            split.suffix_height,
                            split.suffix.draw_separator,
                        );
                        return;
                    }
                }
            }
            if let Some(split) = current_page_reset_fragments {
                if let Some(page) = st.pages.last_mut() {
                    page.footnotes.push(FootnoteRef {
                        number: fn_ctrl.number,
                        source: source.clone(),
                        fragment: Some(split.prefix),
                    });
                }
                st.add_footnote_fragment_height(split.prefix_height, split.prefix.draw_separator);
                st.force_new_page();
                if let Some(page) = st.pages.last_mut() {
                    page.footnotes.push(FootnoteRef {
                        number: fn_ctrl.number,
                        source: source.clone(),
                        fragment: Some(split.suffix),
                    });
                }
                st.add_footnote_fragment_height(split.suffix_height, split.suffix.draw_separator);
                return;
            }
            if let Some((page_idx, _)) = routed {
                if st.add_footnote_to_completed_page(
                    page_idx,
                    fn_ctrl.number,
                    source.clone(),
                    fn_height,
                ) {
                    return;
                }
            }
            if move_to_next_reset_page {
                // marker/body는 방금 확정한 current page에 그대로 두고,
                // 다음 paragraph의 stored vpos reset이 시작하는 fresh page에
                // 이 단일 각주만 등록한다. p31 two-line fragment와 p43의
                // existing-note reset은 helper guard 밖이므로 영향이 없다.
                st.force_new_page();
            }
            if let Some(page) = st.pages.last_mut() {
                page.footnotes.push(FootnoteRef {
                    number: fn_ctrl.number,
                    source,
                    fragment: None,
                });
            }
            st.add_footnote_height(fn_height);
        }
    }
}
