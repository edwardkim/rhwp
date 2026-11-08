//! 각주 예약의 Query와 현재/완료 쪽 반영 Command. 전역 state 캡슐화는 R5에서 연결한다.

use crate::renderer::typeset::{FootnoteFragment, FootnoteRef, FootnoteSource, TypesetState};

impl TypesetState {
    pub(in crate::renderer::typeset) fn has_deferred_body_footnotes(&self) -> bool {
        !self.data.deferred_body_footnotes.is_empty()
    }

    pub(in crate::renderer::typeset) fn defer_body_footnote_for_next_page(
        &mut self,
        note: FootnoteRef,
        height: f64,
    ) {
        self.data.deferred_body_footnotes.push((note, height));
    }

    /// 쪽 전환을 확정한 뒤 다음 각주 lane의 같은 조각 높이를 fit과 paint에 예약한다.
    /// 앞 쪽에 남은 본문을 각주 suffix의 쪽으로 먼저 옮기지 않는다.
    pub(in crate::renderer::typeset) fn drain_deferred_body_footnotes(&mut self) {
        match self.drain_deferred_body_footnotes_with_storage(&crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }

    pub(in crate::renderer::typeset) fn drain_deferred_body_footnotes_with_storage<
        S: crate::storage::Storage,
    >(
        &mut self,
        storage: &S,
    ) -> Result<(), S::Error> {
        for (note, height) in std::mem::take(&mut self.data.deferred_body_footnotes) {
            let draw_separator = note
                .fragment
                .map_or(true, |fragment| fragment.draw_separator);
            self.record_current_footnote_with_storage(note, storage)?;
            self.add_footnote_fragment_height(height, draw_separator);
        }
        Ok(())
    }

    /// 각주 높이 추가
    pub(in crate::renderer::typeset) fn add_footnote_height(&mut self, height: f64) {
        self.add_footnote_fragment_height(height, true);
    }

    /// 각주 조각 높이 추가. 물리 쪽의 구분선은 표시 플래그에 따라 한 번만 예약한다.
    pub(in crate::renderer::typeset) fn add_footnote_fragment_height(
        &mut self,
        height: f64,
        draw_separator: bool,
    ) {
        if self.add_scoped_footnote_height(height, draw_separator) {
            return;
        }
        // [#2097 진단] 각주 예약 시점 — 동작 불변.
        if std::env::var("RHWP_DIAG_FN").is_ok() {
            eprintln!(
                "DIAG_FN add h={:.1} cur_total={:.1} page={} cur_h={:.1} first={}",
                height,
                self.data.current_footnote_height,
                self.data.pages.len() + 1,
                self.data.current_height,
                self.data.is_first_footnote_on_page
            );
        }
        if self.data.is_first_footnote_on_page {
            if draw_separator {
                self.data.current_footnote_height += self.data.footnote_separator_overhead;
                self.data.current_page_has_footnote_separator = true;
            }
            self.data.is_first_footnote_on_page = false;
        } else {
            // 번호 없는 tail이 page의 첫 각주였고, 뒤의 일반 note가 처음으로
            // separator를 요구하는 경우다. layout은 footnote 전체 중 하나라도
            // draw_separator이면 선을 그리므로, reservation도 같은 시점에 한 번
            // 보충해야 본문/FootnoteArea 충돌이 생기지 않는다.
            if draw_separator && !self.data.current_page_has_footnote_separator {
                self.data.current_footnote_height += self.data.footnote_separator_overhead;
                self.data.current_page_has_footnote_separator = true;
            }
            self.data.current_footnote_height += self.data.footnote_between_notes_margin;
        }
        self.data.current_footnote_height += height;
        self.sync_current_page_footnote_area();
    }

    /// 하나의 각주 fragment를 현재 page에 더했을 때의 높이.
    ///
    /// 일반 note와 달리 continuation tail은 separator 없이 시작할 수 있으므로,
    /// queue fit 판정은 `projected_footnote_height`의 "첫 note면 separator" 가정이
    /// 아니라 실제 fragment flag를 사용해야 한다.
    pub(in crate::renderer::typeset) fn projected_footnote_fragment_height(
        &self,
        content_height: f64,
        draw_separator: bool,
    ) -> f64 {
        self.data.current_footnote_height
            + if self.data.is_first_footnote_on_page {
                0.0
            } else {
                self.data.footnote_between_notes_margin
            }
            + if draw_separator && !self.data.current_page_has_footnote_separator {
                self.data.footnote_separator_overhead
            } else {
                0.0
            }
            + content_height
    }

    /// 현재 page의 body/각주 lane에 새 각주 fragment가 들어가는지 판정한다.
    ///
    /// `overlap_guard`는 table fragment처럼 pagination의 flow origin과 실제 paint
    /// 하단이 어긋날 수 있는 경로에서만 사용한다. 후보 식별과 fit을 분리해 caller가
    /// 현재 page에 각주를 추가해도 되는지를 같은 식으로 판정하게 한다.
    pub(in crate::renderer::typeset) fn footnote_fragment_fits_current_page(
        &self,
        content_height: f64,
        draw_separator: bool,
        overlap_guard: f64,
    ) -> bool {
        let projected = self.projected_footnote_fragment_height(content_height, draw_separator);
        let page_available = (self.base_available_height()
            - projected.max(0.0)
            - self.data.current_zone_y_offset
            - self.data.current_bottom_fixed_exclusion)
            .max(0.0);
        let footnote_only_capacity = (self.base_available_height()
            - self.data.current_zone_y_offset
            - self.data.current_bottom_fixed_exclusion)
            .max(0.0);

        projected.max(0.0) <= footnote_only_capacity + 0.5
            && self.data.current_height + overlap_guard <= page_available + 0.5
    }

    pub(in crate::renderer::typeset) fn reserve_painted_footnote_area(&mut self, height: f64) {
        self.data.current_footnote_height = height;
        self.data.current_footnote_body_bottom_reserved = true;
        self.sync_current_page_footnote_area();
    }

    /// 조회와 확정이 같은 구분선·각주 사이 간격을 예약한다.
    fn completed_page_note_added_height(
        &self,
        page_idx: usize,
        content_height: f64,
    ) -> Option<f64> {
        let page = self.data.pages.get(page_idx)?;
        let has_separator = page.footnotes.iter().any(|note| {
            note.fragment
                .map(|fragment| fragment.draw_separator)
                .unwrap_or(true)
        });
        Some(
            content_height
                + if has_separator {
                    0.0
                } else {
                    self.data.footnote_separator_overhead
                }
                + if page.footnotes.is_empty() {
                    0.0
                } else {
                    self.data.footnote_between_notes_margin
                },
        )
    }

    /// 완료된 표시 쪽의 마지막 본문 조각과 새 각주가 같은 물리 예산에 들어야 한다.
    /// 저장 좌표나 줄간격을 다시 추측하지 않고 수용한 조각의 확정 점유 끝을 쓴다.
    pub(in crate::renderer::typeset) fn completed_body_fragment_note_fits(
        &self,
        page_idx: usize,
        col_idx: usize,
        para_idx: usize,
        content_height: f64,
    ) -> bool {
        let Some(page) = self.data.pages.get(page_idx) else {
            return false;
        };
        let Some(column) = page.column_contents.get(col_idx) else {
            return false;
        };
        let Some(crate::renderer::pagination::PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        }) = column.items.last()
        else {
            return false;
        };
        if *para_index != para_idx {
            return false;
        }
        let Some(bottom) = self
            .data
            .paragraph_fragment_content_bottoms
            .get(&(*para_index, *end_line))
        else {
            return false;
        };
        let Some(added) = self.completed_page_note_added_height(page_idx, content_height) else {
            return false;
        };
        let projected = page.layout.footnote_area.height.max(0.0) + added;
        bottom.content_bottom + column.zone_y_offset
            <= page.layout.body_area.height - projected + 0.5
    }

    /// 분할 문단의 표시가 든 완료 쪽에 통째 각주를 소급 등록한다.
    /// 소유·공존 가능 여부는 호출자가 입증하고 이 명령은 같은 예약량을 확정한다.
    pub(in crate::renderer::typeset) fn add_footnote_to_completed_page(
        &mut self,
        page_idx: usize,
        number: u16,
        source: FootnoteSource,
        content_height: f64,
    ) -> bool {
        let Some(added_height) = self.completed_page_note_added_height(page_idx, content_height)
        else {
            return false;
        };
        let Some(page) = self.data.pages.get_mut(page_idx) else {
            return false;
        };
        let existing_height = page.layout.footnote_area.height.max(0.0);
        page.footnotes.push(FootnoteRef {
            column_index: 0,
            number,
            source,
            fragment: None,
        });
        page.layout
            .update_footnote_area(existing_height + added_height);
        true
    }

    /// 이미 완료된 page에 두 줄 native HWP5 각주의 첫 fragment를 소급 등록한다.
    pub(in crate::renderer::typeset) fn add_footnote_fragment_to_completed_page(
        &mut self,
        page_idx: usize,
        number: u16,
        source: FootnoteSource,
        fragment: FootnoteFragment,
        content_height: f64,
    ) -> bool {
        let data = &mut *self.data;
        let Some(page) = data.pages.get_mut(page_idx) else {
            return false;
        };
        let first = page.footnotes.is_empty();
        let has_separator = page.footnotes.iter().any(|footnote| {
            footnote
                .fragment
                .map(|existing| existing.draw_separator)
                .unwrap_or(true)
        });
        let existing_height = page.layout.footnote_area.height.max(0.0);
        page.footnotes.push(FootnoteRef {
            column_index: 0,
            number,
            source,
            fragment: Some(fragment),
        });
        let added_height = content_height
            + if fragment.draw_separator && !has_separator {
                data.footnote_separator_overhead
            } else {
                0.0
            }
            + if !first {
                data.footnote_between_notes_margin
            } else {
                0.0
            };
        page.layout
            .update_footnote_area(existing_height + added_height);
        true
    }

    pub(in crate::renderer::typeset) fn projected_footnote_height(
        &self,
        note_content_height: f64,
        note_count: usize,
    ) -> f64 {
        self.projected_footnote_height_in_frame(note_content_height, note_count, false)
    }

    /// 새 쪽의 각주 영역에는 앞 쪽의 본문 각주와 구분선을 이어 붙이지 않는다.
    pub(in crate::renderer::typeset) fn projected_fresh_footnote_height(
        &self,
        note_content_height: f64,
        note_count: usize,
    ) -> f64 {
        self.projected_footnote_height_in_frame(note_content_height, note_count, true)
    }

    fn projected_footnote_height_in_frame(
        &self,
        note_content_height: f64,
        note_count: usize,
        fresh: bool,
    ) -> f64 {
        if let Some(height) =
            self.projected_scoped_footnote_height(note_content_height, note_count, fresh)
        {
            return height;
        }
        let current_height = if fresh {
            0.0
        } else {
            self.data.current_footnote_height
        };
        if note_count == 0 {
            return current_height;
        }
        let separator = if !fresh && self.data.current_page_has_footnote_separator {
            0.0
        } else {
            self.data.footnote_separator_overhead
        };
        let between_count = if fresh || self.data.is_first_footnote_on_page {
            note_count.saturating_sub(1)
        } else {
            note_count
        };
        current_height
            + separator
            + self.data.footnote_between_notes_margin * between_count as f64
            + note_content_height
    }

    pub(in crate::renderer::typeset) fn sync_current_page_footnote_area(&mut self) {
        let data = &mut *self.data;
        if data.current_footnote_height <= 0.0 {
            return;
        }
        if let Some(page) = data.pages.last_mut() {
            page.layout
                .update_footnote_area(data.current_footnote_height);
        }
    }

    pub(in crate::renderer::typeset) fn replace_own_line_notes(
        &mut self,
        notes: Vec<crate::renderer::typeset::OwnLineNote>,
    ) {
        self.data.own_line_notes = notes;
    }

    /// 이미 등록한 각주 위에 `notes` 를 차례로 더했을 때 본문 가용 높이가 줄어드는 양.
    /// 실제 예약(`add_footnote_fragment_height`)과 같은 구분선·각주 사이 간격을 쓴다.
    pub(in crate::renderer::typeset) fn own_line_note_cost<'a>(
        &self,
        notes: impl Iterator<Item = &'a crate::renderer::typeset::OwnLineNote>,
    ) -> f64 {
        let (sum, count) = notes.fold((0.0, 0usize), |(sum, count), note| {
            (sum + note.content_height, count + 1)
        });
        if count == 0 {
            return 0.0;
        }
        let current = self.data.current_footnote_height;
        let projected = self.projected_footnote_height(sum, count);
        let penalty = |height: f64| height.max(0.0);
        (penalty(projected) - penalty(current)).max(0.0)
    }

    /// 참조 줄 순서대로 본문 각주가 이 쪽에 어떻게 놓일지 정하고, 참조 줄마다 그 줄까지의
    /// 각주가 본문에서 빼는 높이를 돌려준다(`notes` 는 줄 순서, `space_after_line` 은 그 줄
    /// 아래끝 뒤에 남는 본문 공간).
    ///
    /// 한/글은 줄을 놓을 때마다 그 줄의 각주를 등록한다. 각주가 다 들어가면 뒤 줄은 그 위에
    /// 들어가야 하고, 앞부분만 들어가면 그 쪽의 나머지를 각주가 차지해 뒤 줄은 다음 쪽으로 간다.
    /// 한 줄도 들어가지 않으면 각주는 통째로 다음 쪽에서 시작하고 뒤 줄을 막지 않는다
    /// (그 뒤 각주도 번호 순서대로 따라간다).
    pub(in crate::renderer::typeset) fn own_line_note_levels(
        &self,
        notes: &[crate::renderer::typeset::OwnLineNote],
        space_after_line: impl Fn(usize) -> f64,
    ) -> Vec<OwnLineNoteLevel> {
        let mut levels: Vec<OwnLineNoteLevel> = Vec::new();
        let mut placed: Vec<crate::renderer::typeset::OwnLineNote> = Vec::new();
        let mut deferred = self.has_deferred_body_footnotes();
        let mut blocked = false;
        for note in notes {
            if !blocked && !deferred {
                let space = space_after_line(note.line) + 0.5;
                let with = |content_height: f64| {
                    let mut probe = note.clone();
                    probe.content_height = content_height;
                    self.own_line_note_cost(placed.iter().chain(std::iter::once(&probe)))
                };
                if with(note.content_height) <= space {
                    placed.push(note.clone());
                } else if note
                    .stored_split
                    .map(|split| split.prefix_height)
                    .or_else(|| note.lines.first().map(|(height, _)| *height))
                    .is_some_and(|first| with(first) <= space)
                {
                    blocked = true;
                } else {
                    deferred = true;
                }
            }
            let cost = self.own_line_note_cost(placed.iter());
            match levels.last_mut() {
                Some(level) if level.line == note.line => {
                    level.cost = cost;
                    level.blocks_later_lines = blocked;
                }
                _ => levels.push(OwnLineNoteLevel {
                    line: note.line,
                    cost,
                    blocks_later_lines: blocked,
                }),
            }
        }
        levels
    }

    /// 확정한 본문 조각의 줄이 참조한 각주를 그 조각의 쪽에 등록한다.
    /// 남은 본문 아래에 각주가 들어가지 않으면 다음 쪽 각주 영역으로 넘긴다
    /// (한/글은 넘친 각주를 다음 쪽에 이어 싣는다).
    pub(in crate::renderer::typeset) fn register_own_line_notes_with_storage<
        S: crate::storage::Storage,
    >(
        &mut self,
        para_idx: usize,
        lines: std::ops::Range<usize>,
        trailing_slack: f64,
        storage: &S,
    ) -> Result<(), S::Error> {
        // 각주 영역은 본문 마지막 줄의 아래끝까지 올라올 수 있다. 그 줄 뒤
        // 줄간격·문단 뒤 간격은 본문과 각주가 함께 쓰는 빈 공간이다.
        let guard = -trailing_slack.max(0.0);
        let notes: Vec<_> = self
            .data
            .own_line_notes
            .iter()
            .filter(|note| {
                note.para_index == para_idx
                    && lines.contains(&note.line)
                    && !self
                        .data
                        .line_registered_notes
                        .contains(&(para_idx, note.control_index))
            })
            .cloned()
            .collect();
        for note in notes {
            if std::env::var_os("RHWP_DIAG_OWN_NOTES").is_some() {
                eprintln!(
                    "DIAG_OWN_NOTE_REG pi={para_idx} lines={lines:?} note_line={} number={} h={:.1} page={} cur_h={:.1} nlines={} stored={:?} deferred={}",
                    note.line, note.number, note.content_height, self.data.pages.len(), self.data.current_height,
                    note.lines.len(), note.stored_split.map(|s| (s.prefix.end_line, s.prefix_height)), self.has_deferred_body_footnotes()
                );
            }
            let source = FootnoteSource::Body {
                para_index: para_idx,
                control_index: note.control_index,
            };
            let whole = FootnoteRef {
                column_index: 0,
                number: note.number,
                source: source.clone(),
                fragment: None,
            };
            if self.has_deferred_body_footnotes() {
                // 앞 번호가 다음 쪽으로 넘어갔으면 뒤 번호도 그 뒤를 잇는다.
                self.defer_body_footnote_for_next_page(whole, note.content_height);
            } else if self.footnote_fragment_fits_current_page(note.content_height, true, guard) {
                self.record_current_footnote_with_storage(whole, storage)?;
                self.add_footnote_fragment_height(note.content_height, true);
            } else if let Some(split) = self.own_line_note_split(&note, guard) {
                self.record_current_footnote_with_storage(
                    FootnoteRef {
                        column_index: 0,
                        number: note.number,
                        source: source.clone(),
                        fragment: Some(split.prefix),
                    },
                    storage,
                )?;
                self.add_footnote_fragment_height(split.prefix_height, split.prefix.draw_separator);
                self.defer_body_footnote_for_next_page(
                    FootnoteRef {
                        column_index: 0,
                        number: note.number,
                        source,
                        fragment: Some(split.suffix),
                    },
                    split.suffix_height,
                );
            } else {
                self.defer_body_footnote_for_next_page(whole, note.content_height);
            }
            self.data
                .line_registered_notes
                .insert((para_idx, note.control_index));
        }
        Ok(())
    }

    /// 현재 쪽에 다 들어가지 않는 각주의 앞 조각. 저장 각주 사다리의 나눔이 들어가면
    /// 그것을, 아니면 남은 공간에 들어가는 가장 많은 줄을 앞 조각으로 쓴다.
    fn own_line_note_split(
        &self,
        note: &crate::renderer::typeset::OwnLineNote,
        guard: f64,
    ) -> Option<crate::renderer::typeset::notes::footnotes::boundary::NativeHwp5FootnoteFragmentSplit>
    {
        let fits = |height: f64| self.footnote_fragment_fits_current_page(height, true, guard);
        if let Some(split) = note.stored_split.filter(|split| fits(split.prefix_height)) {
            return Some(split);
        }
        let line_count = note.lines.len();
        let fragment_height = |start: usize, end: usize| -> f64 {
            note.lines[start..end]
                .iter()
                .enumerate()
                .map(|(index, (height, spacing))| {
                    height
                        + if start + index + 1 < end {
                            *spacing
                        } else {
                            0.0
                        }
                })
                .sum()
        };
        let split_line = (1..line_count)
            .rev()
            .find(|&end| fits(fragment_height(0, end)))?;
        Some(
            crate::renderer::typeset::notes::footnotes::boundary::NativeHwp5FootnoteFragmentSplit {
                prefix: crate::renderer::pagination::FootnoteFragment {
                    start_line: 0,
                    end_line: split_line,
                    draw_separator: true,
                    draw_number: true,
                },
                prefix_height: fragment_height(0, split_line),
                suffix: crate::renderer::pagination::FootnoteFragment {
                    start_line: split_line,
                    end_line: line_count,
                    draw_separator: true,
                    draw_number: false,
                },
                suffix_height: fragment_height(split_line, line_count),
                force_next_page: false,
            },
        )
    }
}

/// 참조 줄 하나까지의 본문 각주가 이 쪽 본문에서 빼는 높이.
#[derive(Debug, Clone, Copy)]
pub(in crate::renderer::typeset) struct OwnLineNoteLevel {
    pub line: usize,
    /// 이 줄까지 이 쪽에 통째로 놓이는 각주들이 본문에서 빼는 높이.
    pub cost: f64,
    /// 이 줄의 각주가 앞부분만 들어가 쪽 나머지를 차지한다(뒤 줄은 다음 쪽).
    pub blocks_later_lines: bool,
}
