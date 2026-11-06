//! 예약하는 각주 프레임과 현재 본문 단이 소비하는 예산을 구분한다.
use crate::model::footnote::FootnotePlacement;
use crate::renderer::footnote_frames::{FramePolicy, Reservation};
use crate::renderer::typeset::TypesetState;

impl TypesetState {
    pub(in crate::renderer::typeset) fn configure_footnote_placement(
        &mut self,
        placement: FootnotePlacement,
    ) {
        self.data.footnote_placement = placement;
    }

    pub(in crate::renderer::typeset) fn footnote_frame_policy(&self) -> FramePolicy {
        FramePolicy {
            placement: self.data.footnote_placement,
            columns: self.data.col_count,
        }
    }

    pub(in crate::renderer::typeset) fn footnote_area_width(&self) -> f64 {
        let layout = self
            .data
            .current_zone_layout
            .as_ref()
            .unwrap_or(&self.data.layout);
        self.footnote_frame_policy()
            .area(layout, self.data.current_column)
            .width
    }

    pub(in crate::renderer::typeset) fn select_footnote_frame(&mut self) {
        let policy = self.footnote_frame_policy();
        if policy.spans_page() {
            return;
        }
        let data = &mut *self.data;
        let frame = if policy.charges_column(data.current_column) {
            data.footnote_frames
                .get(usize::from(policy.target(data.current_column)))
                .copied()
                .unwrap_or_default()
        } else {
            Reservation::default()
        };
        data.current_footnote_height = frame.height;
        data.is_first_footnote_on_page = frame.count == 0;
        data.current_page_has_footnote_separator = frame.separator;
        data.current_footnote_body_bottom_reserved = false;
    }

    pub(in crate::renderer::typeset) fn add_scoped_footnote_height(
        &mut self,
        height: f64,
        separator: bool,
    ) -> bool {
        let policy = self.footnote_frame_policy();
        if policy.spans_page() {
            return false;
        }
        let data = &mut *self.data;
        data.footnote_frames
            .resize(usize::from(policy.columns), Reservation::default());
        data.footnote_frames[usize::from(policy.target(data.current_column))].add(
            height,
            separator,
            data.footnote_separator_overhead,
            data.footnote_between_notes_margin,
        );
        let maximum = data
            .footnote_frames
            .iter()
            .map(|frame| frame.height)
            .fold(0.0, f64::max);
        if let Some(page) = data.pages.last_mut() {
            page.layout.update_footnote_area(maximum);
        }
        self.select_footnote_frame();
        true
    }

    pub(in crate::renderer::typeset) fn projected_scoped_footnote_height(
        &self,
        content: f64,
        count: usize,
        fresh: bool,
    ) -> Option<f64> {
        let policy = self.footnote_frame_policy();
        if policy.spans_page() {
            return None;
        }
        let owner = if fresh { 0 } else { self.data.current_column };
        let frame = if fresh {
            Reservation::default()
        } else {
            self.data
                .footnote_frames
                .get(usize::from(policy.target(owner)))
                .copied()
                .unwrap_or_default()
        };
        let projected = frame.projected(
            content,
            count,
            true,
            self.data.footnote_separator_overhead,
            self.data.footnote_between_notes_margin,
        );
        // 오른쪽 단의 각주는 앞 단의 본문 높이에서 빼지 않는다. 다만 목표 프레임 자체가
        // 한 쪽을 넘으면 기존 fit·분할 경로가 새 참조를 이월하도록 초과 예산을 전달한다.
        Some(
            if policy.charges_column(owner) || projected > self.base_available_height() {
                projected
            } else {
                0.0
            },
        )
    }
}
