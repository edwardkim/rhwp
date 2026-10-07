//! 각주 예약의 Query와 현재/완료 쪽 반영 Command. 전역 state 캡슐화는 R5에서 연결한다.

use crate::renderer::typeset::{FootnoteFragment, FootnoteRef, FootnoteSource, TypesetState};

impl TypesetState {
    /// [#2559] 각주가 사용할 수 있는 빈 꼬리말 밴드 높이.
    pub(in crate::renderer::typeset) fn footer_band_reclaim(&self) -> f64 {
        self.footer_band_reclaim_for_height(self.data.current_footnote_height)
    }

    /// 예약 전 first-footnote collision을 계산할 때도 현재 page와 동일한 footer
    /// band 회수 계약을 적용한다.
    pub(in crate::renderer::typeset) fn footer_band_reclaim_for_height(
        &self,
        footnote_height: f64,
    ) -> f64 {
        // [#2668 실험 토글] 밴드 회수를 끄고 A/B 하기 위한 진단 스위치. 동작 기본값 불변.
        if std::env::var("RHWP_FB_OFF").is_ok() {
            return 0.0;
        }
        if self.data.section_has_no_footer
            && footnote_height > 0.0
            && !self.data.current_footnote_body_bottom_reserved
        {
            self.data.layout.footer_area.height.max(0.0)
        } else {
            0.0
        }
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
        reserve_safety_margin: bool,
        overlap_guard: f64,
    ) -> bool {
        let projected = self.projected_footnote_fragment_height(content_height, draw_separator);
        let projected_margin = if projected > 0.0 && reserve_safety_margin {
            self.data.footnote_safety_margin
        } else {
            0.0
        };
        let reclaim = self.footer_band_reclaim_for_height(projected);
        let page_available = (self.base_available_height()
            - (projected - reclaim).max(0.0)
            - projected_margin
            - self.data.current_zone_y_offset
            - self.data.current_bottom_fixed_exclusion)
            .max(0.0);
        let footnote_only_capacity = (self.base_available_height()
            - self.data.current_zone_y_offset
            - self.data.current_bottom_fixed_exclusion)
            .max(0.0);

        (projected - reclaim).max(0.0) + projected_margin <= footnote_only_capacity + 0.5
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
        bottom + column.zone_y_offset <= page.layout.body_area.height - projected + 0.5
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
        let Some(page) = self.data.pages.get_mut(page_idx) else {
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
            number,
            source,
            fragment: Some(fragment),
        });
        let added_height = content_height
            + if fragment.draw_separator && !has_separator {
                self.data.footnote_separator_overhead
            } else {
                0.0
            }
            + if !first {
                self.data.footnote_between_notes_margin
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
        if note_count == 0 {
            return self.data.current_footnote_height;
        }
        let separator = if self.data.current_page_has_footnote_separator {
            0.0
        } else {
            self.data.footnote_separator_overhead
        };
        let between_count = if self.data.is_first_footnote_on_page {
            note_count.saturating_sub(1)
        } else {
            note_count
        };
        self.data.current_footnote_height
            + separator
            + self.data.footnote_between_notes_margin * between_count as f64
            + note_content_height
    }

    pub(in crate::renderer::typeset) fn sync_current_page_footnote_area(&mut self) {
        if self.data.current_footnote_height <= 0.0 {
            return;
        }
        if let Some(page) = self.data.pages.last_mut() {
            page.layout
                .update_footnote_area(self.data.current_footnote_height);
        }
    }
}
