//! 확정한 공동 호스트 계획의 쪽 소속·개체 원점·흐름 끝을 함께 반영한다.

use super::TypesetState;
use crate::model::paragraph::Paragraph;
use crate::renderer::{height_measurer::MeasuredTable, style_resolver::ResolvedStyleSet};

impl TypesetState {
    pub(in crate::renderer::typeset) fn place_mixed_host_line(
        &mut self,
        para_index: usize,
        para: &Paragraph,
        styles: &ResolvedStyleSet,
        tables: &[MeasuredTable],
        dpi: f64,
    ) -> bool {
        if self.data.col_count != 1
            || !self.data.visible_float_exclusions.is_empty()
            || !self.data.deferred_next_page_stored_frames.is_empty()
        {
            return false;
        }
        let Some(plan) = crate::renderer::typeset::controls::mixed_host_line::prepare(
            para,
            para_index,
            styles,
            tables,
            crate::renderer::hwpunit_to_px(self.data.layout.column_width_hu(), dpi),
            self.data.current_height,
            self.available_height(),
            dpi,
        ) else {
            return false;
        };
        self.ensure_page();
        for page in 0..=plan.table_page {
            if page > 0 {
                self.advance_column_or_new_page();
            }
            // 쪽 소속을 확정한 뒤에도 같은 쪽의 원문 컨트롤 순서는 보존한다.
            for control in 0..para.controls.len() {
                if control == plan.picture_control && plan.picture_page == page {
                    self.data.current_items.push(super::super::PageItem::Shape {
                        para_index,
                        control_index: control,
                    });
                    self.data
                        .paragraph_float_placements
                        .insert((para_index, control), plan.picture);
                    self.data.current_height =
                        self.data.current_height.max(plan.picture.occupied_bottom);
                }
                if control == plan.table_control && plan.table_page == page {
                    self.data
                        .inline_placements
                        .insert((para_index, control), plan.table);
                    self.data.current_items.push(super::super::PageItem::Table {
                        para_index,
                        control_index: control,
                    });
                    self.data.current_height = self.data.current_height.max(plan.table_end);
                }
            }
        }
        if plan.picture_page > plan.table_page {
            self.defer_stored_frame(super::super::DeferredStoredFrameControl {
                kind: super::super::DeferredStoredFrameKind::Picture,
                para_index,
                control_index: plan.picture_control,
                placement: plan.picture,
                following_body_para_index: None,
            });
        }
        self.data.vpos_ladder_dirty = true;
        true
    }
}
