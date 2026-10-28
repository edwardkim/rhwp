//! 현재 목록 마커의 가로 줄 구성과 원본의 물리 프레임 슬롯을 함께 보존한다.

use super::{line_breaking, projects_current_marker_geometry, ResolvedStyleSet, SpaceMetric};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::layout_frame::{LayoutFrame, ParagraphBox};

pub(super) fn project(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    if para.line_segs.len() < 2
        // 기존에 인증한 마지막 자동 어절의 캐시 되감김은 실제 쪽 슬롯이 아니다.
        || super::stored_word_tail::has_unmarked_final_word_rewind(para)
        || !para.controls.is_empty()
        || para.text.contains('\n')
        || para.stored_text_partition_is_dirty()
        || frame.models_exclusions()
        || para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return None;
    }
    let last_frame_start = para
        .line_segs
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| {
            pair[1].is_first_line_of_page()
                || pair[1].is_first_line_of_column()
                || pair[1].vertical_pos < pair[0].vertical_pos
        })
        .map(|(index, _)| index + 1)
        .next_back()?;
    let box_ = ParagraphBox::content(frame.horizontal.clone());
    if !projects_current_marker_geometry(para, &box_, styles) {
        return None;
    }
    let mut probe = frame.clone();
    let mut rows = line_breaking::layout_paragraph_in_frame(para, &mut probe, styles, dpi)?;
    let trace = std::env::var_os("RHWP_DIAG_MARKER_FRAMES").is_some();
    if trace {
        eprintln!(
            "MARKER_FRAMES style={} source_rows={} projected_rows={} last_frame_start={}",
            para.para_shape_id,
            para.line_segs.len(),
            rows.len(),
            last_frame_start,
        );
    }
    // 앞 프레임들의 슬롯 수를 그대로 유지하고 마지막 프레임도 비우지 않는다.
    if rows.len() <= last_frame_start || rows.len() >= para.line_segs.len() {
        return None;
    }
    let physical_flags = LineSeg::TAG_FIRST_LINE_OF_PAGE | LineSeg::TAG_FIRST_LINE_OF_COLUMN;
    for (row, source) in rows.iter_mut().zip(&para.line_segs) {
        // 다른 크기의 새 줄을 원래 슬롯에 밀어 넣지 않는다. 세로 조판이 달라지면
        // 현재의 가로 캐시 갱신 계약으로는 프레임 보존을 입증할 수 없다.
        if row.line_height != source.line_height || row.line_spacing != source.line_spacing {
            if trace {
                eprintln!("MARKER_FRAMES metric_mismatch new={row:?} source={source:?}");
            }
            return None;
        }
        row.vertical_pos = source.vertical_pos;
        row.tag = (row.tag & !physical_flags) | (source.tag & physical_flags);
    }
    Some((rows, probe.project_space_metrics()))
}
