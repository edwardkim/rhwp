//! Read-only occupied-band constraint shared by whole and split placement.
use crate::renderer::{
    float_placement::ParagraphFloatPlacement,
    typeset::{hwpunit_to_px, TypesetState},
};

pub(in crate::renderer::typeset) struct HostPlacementConstraint<'a> {
    pub(in crate::renderer::typeset) table: &'a crate::model::table::Table,
    pub(in crate::renderer::typeset) has_preceding_coanchored_float: bool,
    pub(in crate::renderer::typeset) dpi: f64,
}
impl HostPlacementConstraint<'_> {
    pub(in crate::renderer::typeset) fn constrain(
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
        // 흐름이 소유한 같은 앵커의 표 상자는 현재 커서에서 배치한다.
        // 본문 배제 띠는 글줄의 수용 조건이며 앞 문단의 떠 있는 표와
        // 이 흐름 표의 물리 충돌을 입증하지 않는다.
        if matches!(
            placement.flow,
            crate::renderer::float_placement::ParagraphFloatFlow::CoanchoredTable
        ) {
            return placement;
        }
        placement.clear_occupied_bands(
            st.visible_float_exclusions
                .iter()
                .map(|zone| zone.top..zone.bottom + outer_top),
        )
    }
}
