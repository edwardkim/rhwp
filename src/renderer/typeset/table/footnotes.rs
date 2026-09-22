//! 표 도메인 조정. 기존 평가·상태 쓰기 순서를 보존한다.

use crate::renderer::typeset::{
    estimate_footnote_note_height, native_hwp5_footnote_reset_fragments, notes, Footnote,
    FootnoteFragment, FootnoteRef, FootnoteSource, PendingTableFootnoteFragment, TableCellFootnote,
    TableContinuationCursor, TypesetEngine, TypesetState,
};

impl TypesetEngine {
    /// fragment queue를 거치지 않는 표 셀 각주 또는 표 host의 Body 형제 각주를
    /// 현재 owner page에 등록한다. 첫 두 stored line이 `vpos=0 -> 0`으로 명시적으로
    /// 재시작하면 prefix만 owner page에 두고 suffix는 다음 physical page에 둔다.
    pub(in crate::renderer::typeset) fn register_unqueued_table_footnote(
        &self,
        st: &mut TypesetState,
        footnote: &Footnote,
        source: FootnoteSource,
    ) {
        self.register_unqueued_table_footnote_with_content_height(
            st,
            footnote,
            source,
            estimate_footnote_note_height(footnote, self.dpi),
        );
    }

    /// `register_unqueued_table_footnote`와 같은 owner/fragment 계약을 사용하되,
    /// caller가 layout과 같은 composed-line metric으로 잰 content 높이를 예약한다.
    pub(in crate::renderer::typeset) fn register_unqueued_table_footnote_with_content_height(
        &self,
        st: &mut TypesetState,
        footnote: &Footnote,
        source: FootnoteSource,
        content_height: f64,
    ) {
        let split = st
            .profile
            .hwp5_stored_pagination_layout()
            .then(|| native_hwp5_footnote_reset_fragments(footnote, self.dpi))
            .flatten()
            .filter(|split| split.force_next_page && st.col_count == 1);
        if let Some(split) = split {
            st.record_current_footnote(FootnoteRef {
                number: footnote.number,
                source: source.clone(),
                fragment: Some(split.prefix),
            });
            st.add_footnote_fragment_height(split.prefix_height, split.prefix.draw_separator);
            st.force_new_page();
            st.record_current_footnote(FootnoteRef {
                number: footnote.number,
                source,
                fragment: Some(split.suffix),
            });
            st.add_footnote_fragment_height(split.suffix_height, split.suffix.draw_separator);
            st.request_vpos_reset_after_queued_footnote();
            return;
        }

        st.record_current_footnote(FootnoteRef {
            number: footnote.number,
            source,
            fragment: None,
        });
        st.add_footnote_height(content_height);
    }

    /// RowBreak 표의 cell-footnote queue를 현재 fragment page에 들어가는 만큼만
    /// 등록한다. 마지막 fragment도 capacity를 넘겨 한꺼번에 넣지 않는다. 남은 note는
    /// caller가 다음 physical page로 넘겨 이후 본문과 같은 footnote lane을 공유한다.
    pub(in crate::renderer::typeset) fn register_queued_table_footnotes(
        &self,
        st: &mut TypesetState,
        continuation: &mut TableContinuationCursor,
        notes: &[TableCellFootnote],
        para_idx: usize,
        ctrl_idx: usize,
        fragment_start_row: usize,
        fragment_end_row: usize,
        fragment_has_intra_row_cut: bool,
        terminal_fragment: bool,
        relax_terminal_table_footnote_fit: bool,
        queued_fresh_page: bool,
    ) {
        let note_fits = |st: &TypesetState, content_height: f64, draw_separator: bool| {
            // 단일단의 중간 RowBreak fragment 뒤에는 같은 page에 이어질 본문이 없다.
            // 다음 fragment가 새 page에서 시작하므로, 이 fragment의 table-cell 각주는
            // 일반 본문 후속 배치를 위한 40px safety buffer를 중복 예약하지 않는다.
            // `current_height`는 flow origin 기준이고 FootnoteArea는 page-body 기준으로
            // 내려온다. native HWP5 table fragment에서는 이 두 origin의 차이만큼 table
            // 하단이 separator에 먼저 닿는다. queue 대상에만 32px 물리 guard를 둬
            // 실제 표 bbox와 각주 separator가 겹치지 않게 한다. 일반 footnote 및 HWPX
            // 경로의 기존 pagination 계약은 변경하지 않는다.
            let table_footnote_overlap_guard = if relax_terminal_table_footnote_fit {
                // Table 25의 continuation 페이지는 PDF에서 표 하단과 각주선 사이가
                // 약 한 줄뿐이다. 기존 32px guard는 그 실측 간격보다 커 111번을
                // 다음 page로 잘못 넘겼다.
                12.0
            } else if st.profile.hwp5_stored_pagination_layout() {
                32.0
            } else {
                0.0
            };
            st.footnote_fragment_fits_current_page(
                content_height,
                draw_separator,
                (terminal_fragment || st.col_count != 1) && !relax_terminal_table_footnote_fit,
                table_footnote_overlap_guard,
            )
        };
        let add_note = |st: &mut TypesetState,
                        note: &TableCellFootnote,
                        fragment: Option<FootnoteFragment>,
                        content_height: f64| {
            st.record_current_footnote(FootnoteRef {
                number: note.number,
                source: FootnoteSource::TableCell {
                    para_index: para_idx,
                    table_control_index: ctrl_idx,
                    cell_index: note.cell_index,
                    cell_para_index: note.cell_para_index,
                    cell_control_index: note.cell_control_index,
                },
                fragment,
            });
            let draw_separator = fragment
                .map(|fragment| fragment.draw_separator)
                .unwrap_or(true);
            st.add_footnote_fragment_height(content_height, draw_separator);
        };

        // HWP 저장 reset이 있는 table-cell 각주의 tail은 앞 fragment에서 이미 번호가
        // 출력됐으므로, 다음 table fragment page에서 순서상 가장 먼저 등록한다. tail이
        // 새 page의 첫 각주이고 뒤에 새 note가 이어지면 separator는 새 page에 한 번
        // 필요하다. (p66 note 77 → p67 note 78–85)
        if let Some(pending) = continuation.pending_table_footnote_fragment {
            let Some(note) = notes.get(pending.note_index) else {
                continuation.pending_table_footnote_fragment = None;
                return;
            };
            let Some(split) = note.fragment_split else {
                continuation.pending_table_footnote_fragment = None;
                return;
            };
            let mut tail = pending.fragment;
            if st.is_first_footnote_on_page && continuation.next_table_footnote < notes.len() {
                tail.draw_separator = true;
            }
            if !note_fits(st, split.suffix_height, tail.draw_separator) {
                return;
            }
            add_note(st, note, Some(tail), split.suffix_height);
            continuation.pending_table_footnote_fragment = None;
        }

        while let Some(note) = notes.get(continuation.next_table_footnote) {
            // [#5966] `force_next_page` 는 "이 각주를 다음 물리 쪽에 두라"는 저장
            // 지시다. 큐 소진을 위해 **강제로 연 새 쪽**에서는 이미 충족됐으므로
            // 일반 fit 경로(원자 배치)로 보낸다 — 종전에는 이 단락이 원자 배치를
            // 차단했고, 마커 행이 현재 fragment 밖이면 분할 필터도 기각해 빈 새
            // 쪽에서 진행 불가(디버그 불변식 패닉, 1130000-202100008: note 0
            // h=90.1px 가 avail 876.9px 에 들어가는데도 정지).
            let force_source_page_split = !queued_fresh_page
                && note
                    .fragment_split
                    .is_some_and(|split| split.force_next_page);
            if force_source_page_split || !note_fits(st, note.content_height, true) {
                // p728 note 77처럼 table cell 안의 stored vpos reset이 실제 footnote
                // page boundary를 명시하고, marker row가 지금 확정한 intermediate
                // fragment에 있을 때만 note를 line fragment로 나눈다. 단순 capacity
                // 부족, multi-column, marker가 다른 fragment인 경우는 종전의 원자
                // queue를 그대로 유지한다. 단, 첫 두 stored line의 `0 -> 0`은 source가
                // 명시한 다음 physical page이므로 terminal table에서도 보존한다
                // (p176 note 234).
                let split = note.fragment_split.filter(|split| {
                    (!terminal_fragment || split.force_next_page)
                        && st.col_count == 1
                        && !fragment_has_intra_row_cut
                        && continuation.pending_table_footnote_fragment.is_none()
                        && note.row >= fragment_start_row
                        && note.row < fragment_end_row
                });
                let mut split_registered = false;
                if let Some(split) = split
                    .filter(|split| note_fits(st, split.prefix_height, split.prefix.draw_separator))
                {
                    let note_index = continuation.next_table_footnote;
                    add_note(st, note, Some(split.prefix), split.prefix_height);
                    continuation.next_table_footnote += 1;
                    if terminal_fragment && split.force_next_page {
                        st.force_new_page();
                        add_note(st, note, Some(split.suffix), split.suffix_height);
                    } else {
                        continuation.pending_table_footnote_fragment =
                            Some(PendingTableFootnoteFragment {
                                note_index,
                                fragment: split.suffix,
                            });
                    }
                    split_registered = true;
                }
                if !split_registered || !(terminal_fragment && force_source_page_split) {
                    break;
                }
                continue;
            }
            add_note(st, note, None, note.content_height);
            continuation.next_table_footnote += 1;
        }
    }
}
