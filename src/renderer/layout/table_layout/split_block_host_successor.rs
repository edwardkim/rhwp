//! 쪽을 넘어 닫히는 빈 블록 표 host 뒤 문단의 저장 프레임 자리.
//!
//! 칸 안의 빈 host 문단이 글 위아래(TopAndBottom) 블록 표 하나로 여러 저장 쪽 프레임을
//! 걸치면, 마지막 프레임에서 표를 닫은 뒤의 문단은 그 프레임 원점(칸 내용 위)에서 잰 저장
//! vpos 에 놓인다. 쪽을 넘지 않는 host 의 `absorbed_block_host_successor_y` 처럼 뒤 문단의
//! 앞 간격은 표 띠에 흡수된다 — KAF 총회 자료 36쪽 칸 r6·c5: 중첩 표 바닥 434.6px, 표 바깥
//! 아래 여백 141HU 뒤 뒤 빈 문단 저장 줄 위 436.65px(내용 위 117.13 + vpos 23964), 문단 앞
//! 간격 150HU 는 쓰지 않음(한/글 2020 Print). 행 컷 높이(`row_cut_height`)와 부분 표 배치가
//! 같은 당김을 쓴다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{TextWrap, VertRelTo};
use crate::model::table::{Cell, Table};
use crate::renderer::hwpunit_to_px;

use super::{LayoutEngine, ResolvedStyleSet};

impl LayoutEngine {
    /// 저장 프레임 줄 위(`frame_line_top`)가 현재 흐름의 줄 위(`flow_line_top`)보다 위면
    /// 당길 양을 돌려준다. 당김은 뒤 문단 앞 간격(흡수되는 띠)을 넘지 않는다 — 저장 줄이 더
    /// 위를 가리켜도(다른 저장 당시 높이) 표 바깥 여백과 표 내용은 그대로 둔다.
    pub(crate) fn split_block_host_successor_pull(
        &self,
        host: &Paragraph,
        next: &Paragraph,
        styles: &ResolvedStyleSet,
        flow_line_top: f64,
        frame_line_top: f64,
    ) -> f64 {
        let profile = self.profile.get();
        if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
            || profile.session_edited()
            || !host.text.trim().is_empty()
            || host.stored_text_partition_is_dirty()
            || next.stored_text_partition_is_dirty()
        {
            return 0.0;
        }
        let block_tables = host
            .controls
            .iter()
            .filter(|control| {
                matches!(control, Control::Table(table)
                    if !table.common.treat_as_char
                        && table.common.text_wrap == TextWrap::TopAndBottom
                        && table.common.vert_rel_to == VertRelTo::Para
                        && !self.nested_table_is_overlay(table))
            })
            .count();
        if block_tables != 1
            || next
                .controls
                .iter()
                .any(|control| matches!(control, Control::Table(t) if !t.common.treat_as_char))
        {
            return 0.0;
        }
        let Some(seg) = next.line_segs.first() else {
            return 0.0;
        };
        if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || seg.vertical_pos < 0 {
            return 0.0;
        }
        let spacing_before = styles
            .para_styles
            .get(next.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_before.max(0.0));
        (flow_line_top - frame_line_top).clamp(0.0, spacing_before)
    }

    /// 이 조각(`cut_units` 칸 유닛 범위)이 host 문단의 중첩 표 쪽 프레임 시작 유닛에서 열리고
    /// host 뒤 문단까지 담는가. 측정 쪽 프레임 원점(그 유닛의 시작)과 배치 쪽 조각 내용 위가
    /// 같은 자리다.
    pub(crate) fn fragment_opens_split_block_host_frame(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        cut_units: (usize, usize),
        host_idx: usize,
    ) -> bool {
        let units = self.cell_units(cell, table, styles);
        let (start, end) = (cut_units.0, cut_units.1.min(units.len()));
        start < end
            && units[start].para_idx == host_idx
            && units[start].hard_break_before
            && units[start].stored_frame_break_before
            && units[start..end]
                .iter()
                .all(|unit| unit.para_idx != host_idx || unit.nested_row.is_some())
            && units[start..end]
                .iter()
                .any(|unit| unit.para_idx == host_idx + 1)
    }

    /// 행 컷 높이용: 칸 유닛 `[su, eu)` 조각이 쪽을 넘은 빈 블록 표 host 의 마지막 프레임을
    /// 열고 뒤 문단을 담으면 그 문단 앞 간격 중 흡수되는 몫. 흐름 줄 위는 이어받는 자식 표의
    /// 위 띠(`nested_continuation_header_height`)와 앞 유닛, 뒤 문단 앞 간격의 합이다.
    pub(super) fn split_block_host_cut_pull(
        &self,
        cell: &Cell,
        table: &Table,
        styles: &ResolvedStyleSet,
        units: &[super::CellUnit],
        su: usize,
        eu: usize,
    ) -> f64 {
        let Some(host_idx) = units.get(su).map(|unit| unit.para_idx) else {
            return 0.0;
        };
        if !self.fragment_opens_split_block_host_frame(cell, table, styles, (su, eu), host_idx) {
            return 0.0;
        }
        let (Some(host), Some(next)) = (
            cell.paragraphs.get(host_idx),
            cell.paragraphs.get(host_idx + 1),
        ) else {
            return 0.0;
        };
        let Some(successor) = units[su..eu.min(units.len())]
            .iter()
            .position(|unit| unit.para_idx == host_idx + 1)
            .map(|offset| su + offset)
        else {
            return 0.0;
        };
        let Some(seg) = next.line_segs.first() else {
            return 0.0;
        };
        let spacing_before = styles
            .para_styles
            .get(next.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_before.max(0.0));
        let flow_line_top = self.nested_continuation_header_height(cell, units, su, styles)
            + units[su..successor]
                .iter()
                .map(|unit| unit.height)
                .sum::<f64>()
            + spacing_before;
        self.split_block_host_successor_pull(
            host,
            next,
            styles,
            flow_line_top,
            hwpunit_to_px(seg.vertical_pos, self.dpi),
        )
    }
}
