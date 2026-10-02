//! 표 옆에 흡수할 문단/접두 줄과 저장 좌표 끝점의 읽기 전용 계산.
//! 포맷·예산 판정·밴드 종료·쪽 전환은 호출자가 기존 순서대로 수행한다.
use super::wrap_match::WrapBand;
use crate::model::paragraph::Paragraph;
use crate::renderer::hwpunit_to_px;
use crate::renderer::pagination::WrapAroundPara;
use crate::renderer::typeset::is_synthetic_line_seg;

/// 상태 적용 전 확정된 기록과 저장 좌표계의 상대 끝점.
pub(in crate::renderer::typeset) struct WrapAbsorption {
    pub source_offset_px: Option<f64>,
    pub paragraph: WrapAroundPara,
}

/// 매칭된 표 anchor의 전체 문단 흡수 후보. 빈 문단은 저장 끝점으로 밴드를 늘리지 않는다.
pub(in crate::renderer::typeset) fn whole_paragraph(
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    is_empty_para: bool,
    band: WrapBand,
    dpi: f64,
) -> Option<WrapAbsorption> {
    let last_seg_match = para
        .line_segs
        .last()
        .map(|s| s.column_start == band.cs && s.segment_width as i32 == band.sw)
        .unwrap_or(false);
    if last_seg_match || is_empty_para {
        let source_offset_to_text_bottom = (!is_empty_para)
            .then(|| {
                let anchor_top = paragraphs
                    .get(band.anchor_para)?
                    .line_segs
                    .iter()
                    .find(|seg| !is_synthetic_line_seg(seg))?
                    .vertical_pos;
                let text_bottom = para
                    .line_segs
                    .iter()
                    .filter(|seg| !is_synthetic_line_seg(seg))
                    .map(|seg| {
                        seg.vertical_pos
                            .saturating_add(seg.line_height)
                            .saturating_add(seg.line_spacing)
                    })
                    .max()?;
                (text_bottom > anchor_top).then(|| hwpunit_to_px(text_bottom - anchor_top, dpi))
            })
            .flatten();
        return Some(WrapAbsorption {
            source_offset_px: source_offset_to_text_bottom,
            paragraph: WrapAroundPara {
                para_index: para_idx,
                table_para_index: band.anchor_para,
                has_text: !is_empty_para,
                start_line: 0,
                end_line: usize::MAX,
            },
        });
    }
    None
}

/// 전폭 꼬리 fit이 확인된 뒤에만 호출한다. 끝 줄은 배타 인덱스다.
pub(in crate::renderer::typeset) fn prefix(
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    wrap_prefix_len: usize,
    band: WrapBand,
    dpi: f64,
) -> WrapAbsorption {
    let source_offset_to_prefix_bottom = paragraphs.get(band.anchor_para).and_then(|anchor| {
        let anchor_top = anchor
            .line_segs
            .iter()
            .find(|seg| !is_synthetic_line_seg(seg))?
            .vertical_pos;
        let prefix_bottom = para
            .line_segs
            .iter()
            .take(wrap_prefix_len)
            .filter(|seg| !is_synthetic_line_seg(seg))
            .map(|seg| {
                seg.vertical_pos
                    .saturating_add(seg.line_height)
                    .saturating_add(seg.line_spacing)
            })
            .max()?;
        (prefix_bottom > anchor_top).then(|| hwpunit_to_px(prefix_bottom - anchor_top, dpi))
    });
    WrapAbsorption {
        source_offset_px: source_offset_to_prefix_bottom,
        paragraph: WrapAroundPara {
            para_index: para_idx,
            table_para_index: band.anchor_para,
            has_text: true,
            start_line: 0,
            end_line: wrap_prefix_len,
        },
    }
}
