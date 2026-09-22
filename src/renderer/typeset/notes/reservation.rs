//! 각주 예약의 Query와 현재/완료 쪽 반영 Command. 전역 state 캡슐화는 R5에서 연결한다.

use crate::renderer::typeset::{FootnoteFragment, FootnoteRef, FootnoteSource, TypesetState};

impl TypesetState {
    /// [#2559] 각주가 사용할 수 있는 빈 꼬리말 밴드 높이.
    pub(in crate::renderer::typeset) fn footer_band_reclaim(&self) -> f64 {
        self.footer_band_reclaim_for_height(self.current_footnote_height)
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
        if self.section_has_no_footer && footnote_height > 0.0 {
            self.layout.footer_area.height.max(0.0)
        } else {
            0.0
        }
    }

    /// 각주 높이 추가
    pub(in crate::renderer::typeset) fn add_footnote_height(&mut self, height: f64) {
        self.add_footnote_fragment_height(height, true);
    }

    /// 각주 fragment 높이 추가. 연속 tail은 다음 page에서 separator를 반복하지 않는다.
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
                self.current_footnote_height,
                self.pages.len() + 1,
                self.current_height,
                self.is_first_footnote_on_page
            );
        }
        if self.is_first_footnote_on_page {
            if draw_separator {
                self.current_footnote_height += self.footnote_separator_overhead;
                self.current_page_has_footnote_separator = true;
            }
            self.is_first_footnote_on_page = false;
        } else {
            // 번호 없는 tail이 page의 첫 각주였고, 뒤의 일반 note가 처음으로
            // separator를 요구하는 경우다. layout은 footnote 전체 중 하나라도
            // draw_separator이면 선을 그리므로, reservation도 같은 시점에 한 번
            // 보충해야 본문/FootnoteArea 충돌이 생기지 않는다.
            if draw_separator && !self.current_page_has_footnote_separator {
                self.current_footnote_height += self.footnote_separator_overhead;
                self.current_page_has_footnote_separator = true;
            }
            self.current_footnote_height += self.footnote_between_notes_margin;
        }
        self.current_footnote_height += height;
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
        self.current_footnote_height
            + if self.is_first_footnote_on_page {
                0.0
            } else {
                self.footnote_between_notes_margin
            }
            + if draw_separator && !self.current_page_has_footnote_separator {
                self.footnote_separator_overhead
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
            self.footnote_safety_margin
        } else {
            0.0
        };
        let reclaim = if self.section_has_no_footer {
            self.layout.footer_area.height.max(0.0)
        } else {
            0.0
        };
        let page_available = (self.base_available_height()
            - (projected - reclaim).max(0.0)
            - projected_margin
            - self.current_zone_y_offset
            - self.current_bottom_fixed_exclusion)
            .max(0.0);
        let footnote_only_capacity = (self.base_available_height()
            - self.current_zone_y_offset
            - self.current_bottom_fixed_exclusion)
            .max(0.0);

        (projected - reclaim).max(0.0) + projected_margin <= footnote_only_capacity + 0.5
            && self.current_height + overlap_guard <= page_available + 0.5
    }

    /// 이미 flush된 분할 문단의 앵커 page에 첫 native-HWP5 각주를 소급 등록한다.
    ///
    /// 이 경로는 `native_hwp5_first_footnote_overlap_break_line`가 현재 page의
    /// 겹침을 피하려고 문단 tail을 다음 page로 나눈 경우에만 쓴다. 그 시점에는
    /// `current page`가 tail page를 가리키므로, 일반 `add_footnote_height`를 쓰면
    /// 각주 표식이 남은 anchor line과 각주 본문이 서로 다른 page로 갈라진다.
    pub(in crate::renderer::typeset) fn add_footnote_to_completed_page(
        &mut self,
        page_idx: usize,
        number: u16,
        source: FootnoteSource,
        content_height: f64,
    ) -> bool {
        let Some(page) = self.pages.get_mut(page_idx) else {
            return false;
        };
        let first = page.footnotes.is_empty();
        let has_separator = page.footnotes.iter().any(|footnote| {
            footnote
                .fragment
                .map(|fragment| fragment.draw_separator)
                .unwrap_or(true)
        });
        let existing_height = page.layout.footnote_area.height.max(0.0);
        page.footnotes.push(FootnoteRef {
            number,
            source,
            fragment: None,
        });
        let added_height = content_height
            + if has_separator {
                0.0
            } else {
                self.footnote_separator_overhead
            }
            + if first {
                0.0
            } else {
                self.footnote_between_notes_margin
            };
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
        let Some(page) = self.pages.get_mut(page_idx) else {
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
                self.footnote_separator_overhead
            } else {
                0.0
            }
            + if !first {
                self.footnote_between_notes_margin
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
            return self.current_footnote_height;
        }
        let separator = if self.current_page_has_footnote_separator {
            0.0
        } else {
            self.footnote_separator_overhead
        };
        let between_count = if self.is_first_footnote_on_page {
            note_count.saturating_sub(1)
        } else {
            note_count
        };
        self.current_footnote_height
            + separator
            + self.footnote_between_notes_margin * between_count as f64
            + note_content_height
    }

    pub(in crate::renderer::typeset) fn sync_current_page_footnote_area(&mut self) {
        if self.current_footnote_height <= 0.0 {
            return;
        }
        if let Some(page) = self.pages.last_mut() {
            page.layout
                .update_footnote_area(self.current_footnote_height);
        }
    }
}
