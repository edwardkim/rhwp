//! Read-only occupied-band constraint shared by whole and split placement.
use crate::renderer::{
    float_placement::ParagraphFloatPlacement,
    typeset::{hwpunit_to_px, TypesetState},
};

pub(super) struct HostPlacementConstraint<'a> {
    pub(super) table: &'a crate::model::table::Table,
    pub(super) has_preceding_coanchored_float: bool,
    pub(super) dpi: f64,
}
impl HostPlacementConstraint<'_> {
    pub(super) fn constrain(
        &self,
        mut placement: ParagraphFloatPlacement,
        st: &TypesetState,
    ) -> ParagraphFloatPlacement {
        let table = self.table;
        let has_preceding_coanchored_float = self.has_preceding_coanchored_float;
        if table.common.allow_overlap {
            return placement;
        }
        let outer_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        // A zero-offset co-anchored float consumes flow without adding a
        // visible exclusion. Preserve that occupied floor before publishing
        // a final placement; consulting only the exclusion list loses it.
        if has_preceding_coanchored_float {
            let shift = (st.current_height + outer_top - placement.table_top).max(0.0);
            placement.table_top += shift;
            placement.occupied_bottom += shift;
        }
        placement.clear_occupied_bands(
            st.visible_float_exclusions
                .iter()
                .map(|zone| zone.top..zone.bottom + outer_top),
        )
    }
}
