//! 펼친 표의 부모 상자와 자식 개체가 각각 소유하는 원점·점유 높이.

use crate::model::paragraph::LineSeg;
use crate::model::shape::{HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use crate::model::table::{Table, TablePageBreak};
use crate::renderer::float_placement::signed_hwpunit;
use crate::renderer::hwpunit_to_px;

/// 흐름 제한이 없는 문단 기준 표는 셀 안여백·세로 정렬과 독립된 개체다.
/// 저장된 빈 앵커 줄은 부모의 높이를 소유하며, 자식은 그 부모 원점을 참조한다.
pub(crate) fn unrestricted_wrapper_frame(table: &Table, nested: &Table, dpi: f64) -> Option<f64> {
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    let [para] = cell.paragraphs.as_slice() else {
        return None;
    };
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    let common = &nested.common;
    if table.row_count != 1
        || table.col_count != 1
        || table.page_break != TablePageBreak::None
        || nested.page_break != TablePageBreak::None
        || common.treat_as_char
        || common.flow_with_text
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || common.horz_rel_to != HorzRelTo::Para
        || common.horz_align != HorzAlign::Left
        || nested.caption.is_some()
        || !para.text.trim().is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos < 0
        || line.line_height <= 0
        || line.text_height <= 0
        || line.text_height > line.line_height
        || line.baseline_distance < 0
        || line.baseline_distance > line.line_height
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    let anchor_bottom = i64::from(line.vertical_pos)
        + i64::from(line.line_height)
        + i64::from(padding.top)
        + i64::from(padding.bottom);
    let height = anchor_bottom.max(i64::from(signed_hwpunit(cell.height)));
    i32::try_from(height)
        .ok()
        .filter(|&height| height > 0)
        .map(|height| hwpunit_to_px(height, dpi))
}

/// 부모의 테두리 높이와 뒤 흐름의 점유 끝은 자식의 행 높이와 구별한다.
pub(crate) struct TableWrapperVerticalFrame {
    pub(crate) child_top: f64,
    child_bottom: f64,
    minimum: f64,
    independent_frame_height: Option<f64>,
}

impl TableWrapperVerticalFrame {
    pub(crate) fn new(table: &Table, nested: &Table, dpi: f64, native_hwp5: bool) -> Self {
        if let Some(height) = unrestricted_wrapper_frame(table, nested, dpi) {
            return Self {
                child_top: hwpunit_to_px(signed_hwpunit(nested.common.vertical_offset), dpi),
                child_bottom: 0.0,
                minimum: height,
                independent_frame_height: Some(height),
            };
        }
        let cell = &table.cells[0];
        let padding = cell.effective_padding(&table.padding);
        let mut top = hwpunit_to_px(i32::from(padding.top), dpi);
        let mut bottom = hwpunit_to_px(i32::from(padding.bottom), dpi);
        let guard_height = cell.vertical_padding_guard_height_hu(table);
        if guard_height < 0x80000000 {
            let height = hwpunit_to_px(guard_height as i32, dpi);
            if crate::model::table::Cell::vertical_padding_is_abnormal(height, top + bottom) {
                let scale = height * 0.5 / (top + bottom);
                top *= scale;
                bottom *= scale;
            }
        }
        Self {
            child_top: top + hwpunit_to_px(i32::from(nested.outer_margin_top), dpi),
            child_bottom: bottom + hwpunit_to_px(i32::from(nested.outer_margin_bottom), dpi),
            minimum: if native_hwp5 {
                hwpunit_to_px(signed_hwpunit(table.common.height).max(0), dpi)
            } else {
                0.0
            },
            independent_frame_height: None,
        }
    }

    pub(crate) fn unrestricted(&self) -> bool {
        self.independent_frame_height.is_some()
    }

    pub(crate) fn height(&self, child_height: f64) -> f64 {
        (self.child_top + child_height + self.child_bottom).max(self.minimum)
    }

    pub(crate) fn border_height(&self, child_height: f64) -> f64 {
        self.independent_frame_height
            .unwrap_or_else(|| self.height(child_height))
    }
}
