//! 문단 조판의 책임 경계.
//!
//! 구성된 줄 조회, 문단 구성 결과의 높이 조회, 저장 줄 간격 판정을 소유한다.
//! 문단 구성은 필요한 관측값을 읽고 결과만 반환한다.
//! fit 예산의 읽기 전용 계산은 fit에, 1회성 보정 소비는 state에 있다.
//! 줄 후보 계산은 scan, 그 뒤의 경계 보정은 split에 있다.
//! 줄 분할 반복과 페이지 전환은 이 모듈이 조정한다. 진입 fit/표 문단은 아직 상위에 남아 있다.
//! 하위 Query는 원본 IR이나 페이지 상태를 변경하지 않으며, 확정 조각 적용은 state가 맡는다.

pub(super) mod context;
pub(super) mod fit;
pub(super) mod format;
pub(super) mod line_queries;
pub(super) mod metrics;
pub(super) mod placement;
pub(super) mod scan;
pub(super) mod split;
pub(super) mod stored_lines;

use super::TypesetState;
use crate::model::paragraph::Paragraph;
use metrics::FormattedParagraph;

/// 진입 fit 판단 뒤의 줄 분할을 조정한다. 쪽 전환 후 후보를 다시 계산하며,
/// 전체 문단 재시도와 조각 배치 후 이월을 구분한다. 진입 시 고정한 기준 예산은 유지한다.
#[allow(clippy::too_many_arguments)]
pub(super) fn place_split_paragraph(
    st: &mut TypesetState,
    para_idx: usize,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    line_count: usize,
    base_available: f64,
    layout_drift_safety_px: f64,
    forced_page_break_line: Option<usize>,
    native_hwp5_existing_footnote_reset_line: Option<usize>,
    current_page_vpos_base: Option<i32>,
    is_tac_picture_stack: bool,
    dpi: f64,
) {
    // 줄 단위 분할 루프
    let mut cursor_line: usize = 0;
    while cursor_line < line_count {
        let fn_margin = if st.current_footnote_height > 0.0 {
            st.footnote_safety_margin
        } else {
            0.0
        };
        let page_avail = if cursor_line == 0 {
            (base_available
                - st.current_footnote_height
                - fn_margin
                - st.current_height
                - st.current_zone_y_offset)
                .max(0.0)
        } else {
            base_available
        };

        let sp_b = if cursor_line == 0 {
            fmt.spacing_before
        } else {
            0.0
        };
        // Task #332 Stage 4b: partial split 의 줄 단위 fit 검사에도 layout drift 마진 적용
        let avail_for_lines = (page_avail - sp_b - layout_drift_safety_px).max(0.0);

        let scan::LineScanResult {
            end_line,
            cumulative,
            used_saved_tail_vpos_fit,
        } = scan::scan_lines(
            para,
            fmt,
            paragraphs,
            para_idx,
            cursor_line,
            line_count,
            avail_for_lines,
            forced_page_break_line,
            native_hwp5_existing_footnote_reset_line,
            current_page_vpos_base,
            is_tac_picture_stack,
            &st.paragraph_line_scan_page(),
            dpi,
        );

        let split::SplitBoundary {
            end_line,
            cumulative,
        } = split::refine_split_boundary(
            para,
            fmt,
            paragraphs.get(para_idx + 1),
            cursor_line,
            line_count,
            avail_for_lines,
            st.base_available_height(),
            st.profile.hwp5_stored_pagination_layout(),
            dpi,
            split::SplitBoundary {
                end_line,
                cumulative,
            },
        );

        let Some(fragment) = placement::plan_fragment(
            fmt,
            para_idx,
            cursor_line,
            line_count,
            sp_b,
            avail_for_lines,
            split::SplitBoundary {
                end_line,
                cumulative,
            },
            used_saved_tail_vpos_fit,
            &st.current_items,
        ) else {
            st.advance_column_or_new_page();
            continue;
        };
        st.commit_split_paragraph_fragment(fragment);

        if end_line >= line_count {
            break;
        }

        // move: 나머지 줄 → 다음 단/페이지
        st.advance_column_or_new_page();
        cursor_line = end_line;
    }
}
