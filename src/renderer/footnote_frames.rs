//! 각주 소유 단과 통단·오른쪽 단 정책을 예약·측정·배치에서 함께 사용한다.
use super::page_layout::{LayoutRect, PageLayoutInfo};
use crate::model::footnote::FootnotePlacement;

#[derive(Clone, Copy)]
pub(crate) struct FramePolicy {
    pub placement: FootnotePlacement,
    pub columns: u16,
}

impl FramePolicy {
    pub fn target(self, owner: u16) -> u16 {
        match self.placement {
            FootnotePlacement::BelowText => 0,
            FootnotePlacement::RightColumn => self.columns.saturating_sub(1),
            FootnotePlacement::EachColumn => owner.min(self.columns.saturating_sub(1)),
        }
    }

    pub fn spans_page(self) -> bool {
        self.columns <= 1 || self.placement == FootnotePlacement::BelowText
    }

    pub fn charges_column(self, column: u16) -> bool {
        self.placement != FootnotePlacement::RightColumn || column + 1 >= self.columns
    }

    pub fn area(self, layout: &PageLayoutInfo, owner: u16) -> LayoutRect {
        if self.spans_page() {
            layout.body_area
        } else {
            layout
                .column_areas
                .get(usize::from(self.target(owner)))
                .copied()
                .unwrap_or(layout.body_area)
        }
    }
}

/// 구분선과 주석 사이 간격을 포함하는 한 각주 프레임의 누적 예약.
#[derive(Clone, Copy, Default)]
pub(crate) struct Reservation {
    pub height: f64,
    pub count: usize,
    pub separator: bool,
}

impl Reservation {
    pub fn projected(
        self,
        content: f64,
        count: usize,
        separator: bool,
        above: f64,
        between: f64,
    ) -> f64 {
        if count == 0 {
            return self.height;
        }
        self.height
            + content
            + if separator && !self.separator {
                above
            } else {
                0.0
            }
            + between
                * if self.count == 0 {
                    count.saturating_sub(1)
                } else {
                    count
                } as f64
    }

    pub fn add(&mut self, content: f64, separator: bool, above: f64, between: f64) {
        self.height = self.projected(content, 1, separator, above, between);
        self.count += 1;
        self.separator |= separator;
    }
}
