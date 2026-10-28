//! 셀 문단의 흐름 앞 간격과 저장 프레임의 물리 앞 띠를 구분한다.

use crate::model::paragraph::Paragraph;

/// 저장 첫 줄은 앞 간격을 이미 원점에 품는다. 흐름에서 다시 더하지 않아도
/// 그 띠가 차지하는 공간은 셀 높이와 첫 분할 유닛에 포함해야 한다.
pub(crate) struct CellParagraphBefore {
    pub flow_before: f64,
    pub stored_lead: f64,
}

impl CellParagraphBefore {
    pub fn occupied_before(&self) -> f64 {
        self.flow_before + self.stored_lead
    }

    /// A floating paragraph anchor precedes its saved text lead. The same lead
    /// is reserved by text layout; adding it to the floating frame counts it twice.
    pub fn stored_anchor_top(&self, first_text_top: f64) -> f64 {
        first_text_top - self.stored_lead
    }
}

pub(crate) fn paragraph_before(
    para: &Paragraph,
    para_index: usize,
    spacing_before: f64,
    dpi: f64,
) -> CellParagraphBefore {
    let flow_before = super::cell_paragraph_spacing_before(para, para_index, spacing_before);
    let stored_lead = if para_index == 0 && !super::para_has_no_stored_line_segs(para) {
        super::cell_first_para_stored_lead(para, spacing_before, dpi)
    } else {
        0.0
    };
    CellParagraphBefore {
        flow_before,
        stored_lead,
    }
}

/// 저장 줄이 없는 순수 텍스트 셀은 앞 간격을 흐름에서 소유한다.
/// 자체 배치 상자가 없는 메타데이터도 같은 텍스트 흐름에 속한다.
/// 그림/표 등 별도 배치 상자를 소유하는 호스트는 이 텍스트 경로에 합치지 않는다.
/// 수용·측정의 paragraph_before와 실제 첫 글줄이 동일한 값을 소비한다.
pub(crate) fn plain_text_before(
    para: &Paragraph,
    para_index: usize,
    spacing_before: f64,
    dpi: f64,
) -> Option<CellParagraphBefore> {
    (para
        .controls
        .iter()
        .all(super::composer::control_is_width_neutral_marker)
        && super::para_has_no_stored_line_segs(para))
    .then(|| paragraph_before(para, para_index, spacing_before, dpi))
}
