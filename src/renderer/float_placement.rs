//! Flow reservation helpers for non-inline floating objects.

use crate::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertRelTo};
use crate::model::HwpUnit;

use super::hwpunit_to_px;
use super::page_layout::LayoutRect;

/// Interpret an HWPUNIT value that may have been stored through a signed field.
pub(crate) fn signed_hwpunit(value: HwpUnit) -> i32 {
    value as i32
}

/// A non-TAC `TopAndBottom` object positioned from its host paragraph.
pub(crate) fn is_para_topbottom_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Para)
}

/// Horizontal reference data used by float placement and table layout.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FloatPlacementContext {
    pub col_area: LayoutRect,
    pub body_area: Option<LayoutRect>,
    pub paper_width: Option<f64>,
    pub host_margin_left: f64,
    pub host_margin_right: f64,
}

impl FloatPlacementContext {
    pub(crate) fn new(col_area: LayoutRect) -> Self {
        Self {
            col_area,
            body_area: None,
            paper_width: None,
            host_margin_left: 0.0,
            host_margin_right: 0.0,
        }
    }

    pub(crate) fn with_body_area(mut self, body_area: LayoutRect) -> Self {
        self.body_area = Some(body_area);
        self
    }

    pub(crate) fn with_paper_width(mut self, paper_width: f64) -> Self {
        self.paper_width = Some(paper_width);
        self
    }

    pub(crate) fn with_host_margins(mut self, left: f64, right: f64) -> Self {
        self.host_margin_left = left;
        self.host_margin_right = right;
        self
    }
}

/// Compute the same depth-0 horizontal range used by table layout.
pub(crate) fn horizontal_range(
    common: &CommonObjAttr,
    width_px: f64,
    ctx: FloatPlacementContext,
    dpi: f64,
) -> (f64, f64) {
    let h_offset = hwpunit_to_px(signed_hwpunit(common.horizontal_offset), dpi);
    let col_area = ctx.col_area;
    let (ref_x, ref_w) = match common.horz_rel_to {
        HorzRelTo::Paper => {
            let fallback_paper_w = if width_px > col_area.width {
                col_area.x * 2.0 + width_px
            } else {
                col_area.x * 2.0 + col_area.width
            };
            let paper_w = ctx.paper_width.unwrap_or(fallback_paper_w);
            (0.0, paper_w)
        }
        HorzRelTo::Page => ctx
            .body_area
            .filter(|body| body.width > 0.0)
            .map(|body| (body.x, body.width))
            .unwrap_or((col_area.x, col_area.width)),
        HorzRelTo::Para => (
            col_area.x + ctx.host_margin_left,
            col_area.width - ctx.host_margin_left,
        ),
        HorzRelTo::Column => (col_area.x, col_area.width),
    };

    let x = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
        HorzAlign::Center => ref_x + (ref_w - width_px).max(0.0) / 2.0 + h_offset,
        HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - width_px).max(0.0) - h_offset,
    };
    (x, x + width_px.max(0.0))
}

/// A placed float lane in page/column-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatLane {
    pub x_start: f64,
    pub x_end: f64,
    pub bottom: f64,
}

impl FloatLane {
    fn overlaps_x(&self, x_start: f64, x_end: f64) -> bool {
        ranges_overlap(self.x_start, self.x_end, x_start, x_end)
    }
}

/// Tracks bottom reservations for horizontally independent float lanes.
#[derive(Debug, Default, Clone)]
pub(crate) struct FloatLaneSet {
    lanes: Vec<FloatLane>,
}

impl FloatLaneSet {
    pub(crate) fn new() -> Self {
        Self { lanes: Vec::new() }
    }

    pub(crate) fn clear(&mut self) {
        self.lanes.clear();
    }

    pub(crate) fn lanes(&self) -> &[FloatLane] {
        &self.lanes
    }

    pub(crate) fn pushed_top(&self, x_start: f64, x_end: f64, raw_top: f64) -> f64 {
        self.lanes
            .iter()
            .filter(|lane| lane.overlaps_x(x_start, x_end))
            .fold(raw_top, |top, lane| top.max(lane.bottom))
    }

    pub(crate) fn place(
        &mut self,
        x_start: f64,
        x_end: f64,
        raw_top: f64,
        height: f64,
    ) -> FloatLane {
        let top = self.pushed_top(x_start, x_end, raw_top);
        let lane = FloatLane {
            x_start,
            x_end,
            bottom: top + height.max(0.0),
        };
        self.lanes.push(lane);
        lane
    }

    pub(crate) fn max_bottom(&self) -> f64 {
        self.lanes
            .iter()
            .map(|lane| lane.bottom)
            .fold(0.0, f64::max)
    }
}

pub(crate) fn ranges_overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> bool {
    let a0 = a_start.min(a_end);
    let a1 = a_start.max(a_end);
    let b0 = b_start.min(b_end);
    let b1 = b_start.max(b_end);
    a0 < b1 && b0 < a1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::shape::{HorzAlign, HorzRelTo};

    fn base_common() -> CommonObjAttr {
        CommonObjAttr {
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Column,
            horz_align: HorzAlign::Left,
            ..Default::default()
        }
    }

    #[test]
    fn signed_hwpunit_preserves_negative_offsets() {
        assert_eq!(signed_hwpunit((-43892i32) as u32), -43892);
        assert_eq!(signed_hwpunit(51100), 51100);
    }

    #[test]
    fn para_topbottom_float_predicate_requires_non_tac_para_topbottom() {
        let mut common = base_common();
        assert!(is_para_topbottom_float(&common));

        common.treat_as_char = true;
        assert!(!is_para_topbottom_float(&common));

        common.treat_as_char = false;
        common.text_wrap = TextWrap::Square;
        assert!(!is_para_topbottom_float(&common));

        common.text_wrap = TextWrap::TopAndBottom;
        common.vert_rel_to = VertRelTo::Page;
        assert!(!is_para_topbottom_float(&common));
    }

    #[test]
    fn lane_set_does_not_push_non_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        let first = lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(120.0, 200.0, 10.0, 20.0);

        assert_eq!(first.bottom, 50.0);
        assert_eq!(second.bottom, 30.0);
        assert_eq!(lanes.max_bottom(), 50.0);
    }

    #[test]
    fn lane_set_pushes_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(90.0, 160.0, 10.0, 20.0);

        assert_eq!(second.bottom, 70.0);
        assert_eq!(lanes.max_bottom(), 70.0);
    }

    #[test]
    fn horizontal_range_matches_column_right_offset_rule() {
        let mut common = base_common();
        common.horz_align = HorzAlign::Right;
        common.horizontal_offset = 10;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 50.0, ctx, 7200.0);

        assert_eq!(x0, 160.0);
        assert_eq!(x1, 210.0);
    }

    #[test]
    fn horizontal_range_uses_body_area_for_page_relative_objects() {
        let mut common = base_common();
        common.horz_rel_to = HorzRelTo::Page;
        common.horz_align = HorzAlign::Center;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        })
        .with_body_area(LayoutRect {
            x: 40.0,
            y: 0.0,
            width: 300.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 100.0, ctx, 7200.0);

        assert_eq!(x0, 140.0);
        assert_eq!(x1, 240.0);
    }
}
