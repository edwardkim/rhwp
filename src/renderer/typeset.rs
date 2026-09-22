//! 단일 패스 조판 엔진 (TypesetEngine)
//!
//! 기존 3단계 파이프라인(height_measurer → pagination → layout)을 대체하는
//! 단일 패스 조판 엔진. 각 요소를 format() → fits() → place/split 순서로
//! 처리하여 측정과 배치를 하나의 흐름으로 통합한다.
//!
//! Phase 2: Break Token 기반 표 조판 구현.
//! Chromium LayoutNG의 Break Token 패턴, LibreOffice Writer의 Master/Follow Chain,
//! MS Word/OOXML의 cantSplit/tblHeader를 참고.

use crate::renderer::typeset::notes::footnotes::boundary::{
    native_hwp5_existing_footnote_reset_overlap_break_line,
    native_hwp5_first_footnote_overlap_break_line, native_hwp5_footnote_reset_fragments,
    NativeHwp5FootnoteFragmentSplit,
};
use crate::renderer::typeset::notes::footnotes::measure::{
    composed_footnote_content_height, queued_table_footnote_content_height,
};

use crate::document_core::queries::rendering::body_pile_stays_on_anchor_page;
use crate::model::control::Control;
use crate::model::footnote::{Footnote, FootnoteShape};
use crate::model::header_footer::HeaderFooterApply;
use crate::model::page::{ColumnDef, ColumnType, PageDef};
use crate::model::paragraph::{ColumnBreakType, LineSeg, Paragraph};
use crate::model::shape::CaptionDirection;
use crate::renderer::composer::{compose_paragraph, first_text_line, ComposedParagraph};
use crate::renderer::float_placement::{
    is_page_bottom_fixed_float, is_para_topbottom_float, signed_hwpunit,
    stored_visible_anchor_band_host_line_advance_from_vpos, FloatLaneSet,
};
use crate::renderer::height_cursor::HeightCursor;
use crate::renderer::height_measurer::{stored_square_picture_has_adjacent_text, MeasuredTable};
use crate::renderer::layout::table_layout::native_terminal_child_host_line_spacing;
use crate::renderer::layout::{
    border_width_to_px, endnote_last_column_tail_overflows_frame,
    ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX, ENDNOTE_LAST_COLUMN_SPLIT_BLEED_PX,
};
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::{
    format_number, hwpunit_to_px, NumberFormat as RenderNumberFormat, DEFAULT_DPI,
};

// [Task #836] 미주 paragraph의 가상 para_index = paragraphs.len() + endnote 내 순번.
// rendering.rs에서 paragraphs + endnote_paragraphs를 합쳐서 전달.
use self::controls::deferred::{DeferredTableControl, DeferredTableFlushPoint};
use self::paragraph::boundary::hwpx_explicit_page_break_tail_line;
use self::paragraph::line_queries::{
    composed_line_char_end, line_has_strict_tac_control, line_has_text_span, line_has_visible_text,
};
use self::paragraph::metrics::FormattedParagraph;
use self::paragraph::stored_lines::{
    next_boundary_reverts_spacing_trim, spacing_trim_restorable,
    stored_ladder_encodes_spacing_before,
};
use super::pagination::{
    estimate_footnote_note_height, footnote_between_notes_margin_px,
    footnote_separator_overhead_px, ColumnContent, EndnoteDeferral, EndnoteParaSource, EndnoteRef,
    FootnoteFragment, FootnoteRef, FootnoteSource, HeaderFooterRef, PageContent, PageItem,
    PaginationResult,
};

/// [#5886] 미주 다단이 본문 하단 24px bleed 를 지나 **용지 밖**까지 그리는
/// 잔여만 단 전환한다. 용지 하단 여백(~30px)+bleed 보다 크고, CI 실측
/// 2022.hwpx 12쪽 잔여 초과(+69.7px, 수정 전 +198px)보다는 작다.
/// 80px 는 663번 문단(69.7px)을 놓쳐 알짜 풀이가 1193px 에 남았다.
const ENDNOTE_PAGE_OFFCANVAS_GUARD_PX: f64 = 56.0;

/// [#4654] 전면 크기 그림 낱장 배치는 문단의 비인라인 그림 중 엄격한 과반일 때만 쓴다.
///
/// 정확히 절반인 문단까지 낱장 정책을 적용하면 기존 pile 문서의 다수 그림 흐름을
/// 불필요하게 페이지 단위로 분리한다. 두 장 이상이라는 #1995의 하한은 유지한다.
fn has_majority_fullpage_images(fullpage_count: usize, noninline_picture_count: usize) -> bool {
    fullpage_count >= 2 && fullpage_count.saturating_mul(2) > noninline_picture_count
}

/// [#6511] #1995 낱장 배치는 본문 흐름에 참여하는 비-TAC 그림만 후보와 과반
/// 분모로 센다. 글뒤로/글앞으로(BehindText/InFrontOfText) 그림은 흐름 공간을
/// 소비하지 않아 한 쪽에 몇 장이든 공존할 수 있으므로(#703 표와 같은 불변식)
/// 전면 크기라도 스캔 이미지가 아니라 배경/워터마크다. 낱장 배치 후보로 세면
/// 배경 프레임을 깐 안내문이 강제 새 쪽을 얻고, 분모로만 남겨도 스캔 그림의
/// 과반을 희석해 정당한 낱장 배치를 억제한다 — 양쪽 모두에서 제외한다.
fn flow_noninline_picture(ctrl: &Control) -> Option<&crate::model::image::Picture> {
    match ctrl {
        Control::Picture(pic)
            if !pic.common.treat_as_char
                && !matches!(
                    pic.common.text_wrap,
                    crate::model::shape::TextWrap::BehindText
                        | crate::model::shape::TextWrap::InFrontOfText
                ) =>
        {
            Some(pic)
        }
        _ => None,
    }
}

use self::table::continuation::{
    BlockTableContinuationContext, BlockTableContinuationPreparedState,
    BlockTableContinuationSource, PendingTableFootnoteFragment, TableContinuationCursor,
    TableContinuationIteration,
};
pub(crate) use self::table::continuation::{ResumablePaginationStep, ResumableTablePaginationJob};
use self::table::scan::{BlockRowScanVars, BlockTableRowScan};

fn empty_paragraph_fallback_line_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
    hwp3_legacy_caps: bool,
) -> Option<(f64, f64)> {
    // 글앞으로/글뒤로/어울림(비자리차지) 앵커 도형·그림만
    // 가진 빈 문단은 한글이 텍스트 관점의 빈 문단과 동일하게 완전한 em 줄박스를
    // 부여한다 (사용안내 실측: Square 그림 앵커 빈 문단 pi1/pi6 이 0~5.3px 로 붕괴
    // → 한글은 27.7px(base 1300 × 160%) 부여 — PrvImage 줄 좌표 대조). 자리차지
    // (TopAndBottom)는 흐름 소비 계약이 별도라 제외를 유지한다.
    let controls_flow_neutral = para.controls.iter().all(|c| {
        let common = match c {
            Control::Picture(p) => &p.common,
            Control::Shape(s) => s.common(),
            _ => return false,
        };
        !common.treat_as_char
            && matches!(
                common.text_wrap,
                crate::model::shape::TextWrap::InFrontOfText
                    | crate::model::shape::TextWrap::BehindText
                    | crate::model::shape::TextWrap::Square
            )
    });
    // char_count == 0 배제는 순수 빈 문단(컨트롤 없음)에만 유지한다 — 글앞/글뒤
    // 도형·그림 앵커 문단은 char_count 0 으로 저장되는 경우가 있고(사용안내 pi1/pi6
    // 실측 0px 붕괴), 한글은 이들에도 완전한 em 줄박스를 부여한다.
    if std::env::var("RHWP_DIAG_EMPTYP").is_ok() && !para.controls.is_empty() {
        eprintln!(
            "DIAG_EMPTYP text={:?} ctrls={} neutral={} segs={} cc={}",
            para.text.chars().take(6).collect::<String>(),
            para.controls.len(),
            controls_flow_neutral,
            para.line_segs.len(),
            para.char_count,
        );
    }
    if !para.text.trim().is_empty()
        || !(para.controls.is_empty() || controls_flow_neutral)
        || !para.line_segs.is_empty()
        || (para.char_count == 0 && para.controls.is_empty())
    {
        return None;
    }
    let char_shape_id =
        para.char_shape_id_at(0)
            .or_else(|| para.char_shapes.first().map(|cs| cs.char_shape_id))? as usize;
    let char_style = styles.char_styles.get(char_shape_id)?;
    let font_size = char_style.font_size;
    if font_size <= 0.0 {
        return None;
    }
    // [#2070 stage12] 폰트 크기 캡(10pt 이하, 비볼드 8pt 미만 제외)은 HWP3 변환본
    // 한정으로 유지한다 — 한글은 NO_LS 빈 문단을 크기와 무관하게 완전한 em 줄박스로
    // 재계산하지만(D-사다리 + 80168 실측), HWP3→HWP5 변환본은 종전 캡이 페이지 수
    // 게이트(sample16-hwp5 = 64)와 정합(캡 전면 제거 시 65 over-split).
    if hwp3_legacy_caps {
        let small_empty_para_max_font = hwpunit_to_px(1000, DEFAULT_DPI);
        if font_size > small_empty_para_max_font + 0.1 {
            return None;
        }
        let meaningful_empty_para_min_font = hwpunit_to_px(800, DEFAULT_DPI);
        if !char_style.bold && font_size < meaningful_empty_para_min_font - 0.1 {
            return None;
        }
    }
    let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
    let ls_type = para_style
        .map(|s| s.line_spacing_type)
        .unwrap_or(crate::model::style::LineSpacingType::Percent);
    Some(crate::renderer::corrected_line_metrics(
        0.0, 0.0, font_size, ls_type, ls_val,
    ))
}

fn raw_table_ctrl_height_px(table: &crate::model::table::Table, dpi: f64) -> Option<f64> {
    let range = crate::model::shape::common_obj_offsets::HEIGHT;
    if table.raw_ctrl_data.len() < range.end {
        return None;
    }
    let height = u32::from_le_bytes(table.raw_ctrl_data[range].try_into().ok()?);
    (height > 0).then(|| hwpunit_to_px(height as i32, dpi))
}

// ========================================================
// Break Token — 조판 분할 지점 (Chromium LayoutNG 참고)
// ========================================================

/// 표 조판의 분할 재개 정보.
/// 다음 페이지에서 이 토큰으로부터 이어서 조판한다.
#[derive(Debug, Clone)]
struct TableBreakToken {
    /// 재개할 시작 행 인덱스
    start_row: usize,
    /// 인트라-로우 분할 시 각 셀의 콘텐츠 오프셋
    cell_content_offsets: Option<Vec<f64>>,
}

/// RowBreak 표가 page boundary를 넘을 때 queue가 source order로 등록할 셀 각주.
///
/// `fragment_split`은 native HWP5 cell-footnote의 실제 stored vpos reset을 그대로
/// 옮긴 경우에만 채운다. 단순 capacity 부족은 일반 각주처럼 원자적으로 유지한다.
#[derive(Debug, Clone, Copy)]
struct TableCellFootnote {
    number: u16,
    cell_index: usize,
    cell_para_index: usize,
    cell_control_index: usize,
    row: usize,
    /// Renderer가 실제 줄바꿈·trailing line-spacing까지 합산한 높이. RowBreak
    /// fragment queue는 이 값을 써야 URL 각주가 본문 위로 침범하지 않는다.
    content_height: f64,
    fragment_split: Option<NativeHwp5FootnoteFragmentSplit>,
}

// ========================================================
// FormattedTable — 표의 format() 결과
// ========================================================

/// 표의 조판 높이 정보 (format 단계 결과).
/// 기존 MeasuredTable + host_spacing을 통합하여 측정-배치 일원화.
#[derive(Debug)]
struct FormattedTable {
    /// 행별 높이 (px)
    row_heights: Vec<f64>,
    /// 행간 간격 (px)
    cell_spacing: f64,
    /// 머리행 수 (repeat_header && has_header_cells일 때 1)
    header_row_count: usize,
    /// 호스트 문단 spacing
    host_spacing: HostSpacing,
    /// 표 자체 높이 (host_spacing 미포함)
    effective_height: f64,
    /// 전체 높이 (host_spacing 포함)
    total_height: f64,
    /// 캡션 높이
    caption_height: f64,
    /// TAC 표 여부
    is_tac: bool,
    /// 누적 행 높이 (cell_spacing 포함)
    cumulative_heights: Vec<f64>,
    /// 표 쪽 나눔 설정
    page_break: crate::model::table::TablePageBreak,
    /// 셀별 측정 데이터 (인트라-로우 분할용)
    cells: Vec<crate::renderer::height_measurer::MeasuredCell>,
    /// 표 셀 내 각주 높이 합계 (가용 높이에서 차감)
    table_footnote_height: f64,
    /// 표 셀 내 각주 수 (separator/between-notes 예약 계산용)
    table_footnote_count: usize,
    /// 표 셀 각주의 source·순서·측정 높이.
    table_footnotes: Vec<TableCellFootnote>,
    /// #2439: 이 표 직후의 일반 문단은 trailing line advance까지 포함해 fit한다.
    /// native HWP의 단일 양수-offset 빈 호스트 RowBreak 표 증거가 있을 때만 true.
    strict_following_plain_text_fit: bool,
}

#[derive(Debug, Clone, Copy)]
struct VisibleFloatExclusion {
    /// 이 밴드를 만든 host 문단. 같은 문단의 co-anchored float 은 밴드를 서로
    /// 넘겨 짚으면 안 되므로(`#1510`) 소유자를 함께 싣는다.
    para_index: usize,
    /// visible host 문단의 자리차지 float 표가 후속 본문을 피하게 만드는 y 구간.
    top: f64,
    bottom: f64,
}

/// 다음 physical page의 본문 시작에 배치할 non-TAC Square picture control.
///
/// 그림은 float라 본문 높이를 소비하지 않지만, native HWP5는 anchor 문단이 page tail에
/// 있고 그림+caption이 각주 영역까지 닿으면 anchor의 남은 본문 줄은 현재 쪽에 유지한
/// 채 그림만 다음 쪽의 wrap band로 이월한다. `PageItem`을 현재 쪽에 즉시 넣으면 layout
/// 단계에서는 그 단의 anchor 좌표밖에 모르므로 본문/각주 위에 겹친다.
#[derive(Debug, Clone)]
struct DeferredSquarePictureControl {
    para_index: usize,
    control_index: usize,
    /// 그림이 다음 physical page를 소유할 때 같은 page에서 narrow line band를 써야 하는
    /// 연속 후속 본문 문단. 그림 PageItem만 이월하면 layout은 이 source contract를 알 수 없어
    /// 첫 문단 뒤의 본문을 전폭으로 그리고 Square 그림과 교차시킨다.
    wrap_target_para_indices: Vec<usize>,
    wrap_anchor: crate::renderer::pagination::WrapAnchorRef,
}

/// 호스트 문단의 spacing (표 전/후)
#[derive(Debug, Clone, Copy)]
struct HostSpacing {
    /// 표 앞 spacing (spacing_before + outer_margin_top)
    before: f64,
    /// 표 뒤 spacing (spacing_after + outer_margin_bottom + host_line_spacing)
    after: f64,
    /// spacing_after만 (마지막 fragment용 — Paginator와 동일)
    spacing_after_only: f64,
    /// 페이지 적합 판정용 after — 빈 호스트 자리차지 표의 outer_margin_bottom 은
    /// 흐름 전진에만 계상하고 fit 에서 제외한다 (#2195 stage58: 86712 구분선 간격
    /// 한글 괘선 실측 vs hwpspec 178쪽 핀(#1086) 동시 충족).
    after_for_fit: f64,
}

/// 단일 패스 조판 엔진
pub struct TypesetEngine {
    dpi: f64,
    /// [#2403] 현재 조판 입력의 레이아웃 호환 프로파일 — typeset 진입 시 set.
    /// (HWPX 저장 시멘틱·HWP3 변환본 판단 등 소스분기의 단일 질의 표면.)
    profile: std::cell::Cell<crate::model::provenance::LayoutCompatibilityProfile>,
    /// [#5854] 현재 구역의 저장 LINE_SEG 사다리가 통짜 합성값인지 — 구역 진입 시 set.
    /// 참이면 줄 metrics 를 저장값이 아니라 글꼴·문단 스타일에서 다시 뽑는다.
    uniform_filler_ladder: std::cell::Cell<bool>,
    /// [#6175] 현재 구역의 용지/쪽 기준 어울림 개체 흐름 증거 — 구역 진입 시 set.
    /// 폭과 세로 band가 모두 맞을 때만 저장 행 admission이 균일한 좁은 행을
    /// 문단 자신의 테두리 inset과 구분한다.
    float_carve_evidence:
        std::cell::RefCell<Vec<crate::renderer::float_placement::FloatCarveEvidence>>,
    render_normalization:
        std::sync::Arc<crate::renderer::render_normalization::RenderNormalizationOverlay>,
}

/// 조판 중 현재 페이지/단 상태
/// [#3236] 1행 1열 RowBreak 표의 선언 높이 신뢰(#1891) 상한 배율. 측정이 선언의
/// 이 배율을 넘으면 폰트 대체 팽창이 아니라 셀 내용이 진짜로 큰 것이므로 특례를
/// 적용하지 않고 인트라-로우 분할 경로에 맡긴다.
const SINGLE_ROW_DECLARED_TRUST_MAX_RATIO: f64 = 1.5;
/// [Task #2085] 표 분할 첫 조각에 남길 최소 상단 높이 / RowBreak 말미 빈 행 허용 오버플로.
const MIN_TOP_KEEP_PX: f64 = 25.0;

/// Row-internal splits normally keep the established content-only orphan threshold.  The narrow
/// native-HWP #2439 contract instead compares the painted fragment, including visible cell
/// padding, because that is the height that actually remains on the page.
#[inline]
fn row_split_meets_min_top_keep(
    content_height: f64,
    painted_height: f64,
    use_painted_height: bool,
) -> bool {
    let keep_height = if use_painted_height {
        painted_height
    } else {
        content_height
    };
    keep_height >= MIN_TOP_KEEP_PX
}

/// [#6035] 행의 셀 문단 저장 사다리에 **비전진(동일 vpos) 연속 seg 쌍**이 있는지 —
/// 저장 시점 한글이 이 행을 쪽 경계에서 줄 단위로 나눈 흔적이다 (2804253 r70:
/// 0/1560/1560, horz 동일이라 좌우분할 아님). 같은 vpos 의 세 의미(좌우분할·쪽
/// 리셋·중복) 중 좌우분할은 `column_start`/폭이 갈리므로 세로 신호만 잡는다.
fn row_has_stored_same_vpos_split_signal(table: &crate::model::table::Table, row: usize) -> bool {
    table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row)
        .any(|cell| {
            cell.paragraphs.iter().any(|paragraph| {
                paragraph.line_segs.windows(2).any(|pair| {
                    pair.iter().all(|seg| {
                        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    }) && pair[0].vertical_pos > 0
                        && pair[1].vertical_pos == pair[0].vertical_pos
                        && pair[1].column_start == pair[0].column_start
                        && pair[1].segment_width == pair[0].segment_width
                })
            })
        })
}

/// [#6860] 셀의 첫 한 줄 뒤에서 다음 문단이 같은 0으로 재개하고, 그 다음
/// 문단부터 정상 전진하는 저장 경계. 모든 문단을 0으로 저장한 입력이나
/// 개체/빈 문단, 서로 다른 열의 줄은 쪽 경계 증거로 사용하지 않는다.
fn row_has_stored_cross_paragraph_zero_reset(
    table: &crate::model::table::Table,
    row: usize,
) -> bool {
    use crate::model::paragraph::LineSeg;

    table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row)
        .any(|cell| {
            let Some(paragraphs) = cell.paragraphs.get(..3) else {
                return false;
            };
            if paragraphs
                .iter()
                .any(|para| !para.controls.is_empty() || para.text.trim().is_empty())
            {
                return false;
            }
            let [first] = paragraphs[0].line_segs.as_slice() else {
                return false;
            };
            let [second] = paragraphs[1].line_segs.as_slice() else {
                return false;
            };
            let Some(third) = paragraphs[2].line_segs.first() else {
                return false;
            };
            [first, second, third].iter().all(|seg| {
                seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && seg.line_height > 0
                    && seg.column_start == first.column_start
                    && seg.segment_width == first.segment_width
            }) && first.vertical_pos == 0
                && second.vertical_pos == 0
                && third.vertical_pos > 0
                && i64::from(third.vertical_pos)
                    == i64::from(second.line_height) + i64::from(second.line_spacing.max(0))
        })
}

/// Native HWP로 저장·재파싱한 뒤에도 남는 "1열 셀을 1×2로 분할"한 표 구조.
///
/// [`crate::model::table::Table::split_cell_into`]는 기존 1열의 다른 셀을 새 2열
/// 전체 span으로 확장하고, 분할 대상 행에는 원문 셀과 템플릿에서 만든 빈 셀을
/// 나란히 둔다. `dirty_flag`는 저장 경계를 넘지 못하므로 이 구조만 #4138의
/// continuation strict-cut 근거로 쓴다. 병합 제목행과 일반 2열 데이터행이 섞인
/// 흔한 표(#2097)는 현재 행의 두 셀이 모두 본문을 가지므로 제외된다.
fn is_reparsed_single_column_cell_split_row(
    table: &crate::model::table::Table,
    row: usize,
) -> bool {
    if table.col_count != 2 || row >= table.row_count as usize {
        return false;
    }

    let mut row_cells = table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row)
        .collect::<Vec<_>>();
    row_cells.sort_by_key(|cell| cell.col);
    let matching_padding = row_cells.len() == 2
        && row_cells[0].padding.left == row_cells[1].padding.left
        && row_cells[0].padding.right == row_cells[1].padding.right
        && row_cells[0].padding.top == row_cells[1].padding.top
        && row_cells[0].padding.bottom == row_cells[1].padding.bottom;
    if row_cells.len() != 2
        || row_cells[0].col != 0
        || row_cells[1].col != 1
        || row_cells
            .iter()
            .any(|cell| cell.col_span != 1 || cell.row_span != 1)
        || row_cells[0].width.abs_diff(row_cells[1].width) > 1
        || row_cells[0].height != row_cells[1].height
        || row_cells[0].border_fill_id != row_cells[1].border_fill_id
        || !matching_padding
    {
        return false;
    }

    let cell_has_content = |cell: &&crate::model::table::Cell| {
        cell.paragraphs
            .iter()
            .any(|para| !para.text.is_empty() || !para.controls.is_empty())
    };
    let cell_is_template_empty = |cell: &&crate::model::table::Cell| {
        cell.paragraphs.len() == 1
            && cell.paragraphs[0].text.is_empty()
            && cell.paragraphs[0].controls.is_empty()
            && !cell.paragraphs[0].has_para_text
            && cell.paragraphs[0].char_count <= 1
    };
    let content_cells = row_cells
        .iter()
        .filter(|cell| cell_has_content(cell))
        .copied()
        .collect::<Vec<_>>();
    let template_cells = row_cells
        .iter()
        .filter(|cell| cell_is_template_empty(cell))
        .copied()
        .collect::<Vec<_>>();
    if content_cells.len() != 1 || template_cells.len() != 1 {
        return false;
    }

    let content_cell = content_cells[0];
    let template_cell = template_cells[0];
    if content_cell.col != 0 || template_cell.col != 1 {
        return false;
    }
    let Some(source_para) = content_cell.paragraphs.first() else {
        return false;
    };
    let template_para = &template_cell.paragraphs[0];

    // `new_from_template`는 문단 header를 그대로 복제하되 instanceId
    // (`raw_header_extra[6..10]`)만 0으로 만든다. 이 차이는 native HWP
    // 저장·재파싱 뒤에도 남는다. 단순 `has_para_text=false`는 자연 작성 빈
    // 셀에도 가능하므로, 나머지 raw header와 셀/문단 서식이 실제 clone인 경우만
    // split provenance로 인정한다.
    let zeroed_instance_clone = source_para.raw_header_extra.len() >= 10
        && source_para.raw_header_extra.len() == template_para.raw_header_extra.len()
        && source_para.raw_header_extra[6..10]
            .iter()
            .any(|byte| *byte != 0)
        && template_para.raw_header_extra[6..10]
            .iter()
            .all(|byte| *byte == 0)
        && source_para.raw_header_extra[..6] == template_para.raw_header_extra[..6]
        && source_para.raw_header_extra[10..] == template_para.raw_header_extra[10..];
    let cloned_first_char_shape = template_para.char_shapes.len()
        == source_para.char_shapes.len().min(1)
        && template_para
            .char_shapes
            .iter()
            .zip(source_para.char_shapes.iter())
            .all(|(template, source)| {
                template.start_pos == source.start_pos
                    && template.char_shape_id == source.char_shape_id
            });
    let cloned_first_line = template_para.line_segs.len() == 1
        && source_para
            .line_segs
            .first()
            .zip(template_para.line_segs.first())
            .is_some_and(|(source, template)| {
                source.text_start == template.text_start
                    && source.vertical_pos == template.vertical_pos
                    && source.line_height == template.line_height
                    && source.text_height == template.text_height
                    && source.baseline_distance == template.baseline_distance
                    && source.line_spacing == template.line_spacing
                    && source.column_start == template.column_start
                    && source.segment_width == template.segment_width
                    && source.tag == template.tag
            });
    if !zeroed_instance_clone
        || content_cell.raw_list_extra != template_cell.raw_list_extra
        || content_cell.list_header_width_ref != template_cell.list_header_width_ref
        || content_cell.text_direction != template_cell.text_direction
        || content_cell.vertical_align != template_cell.vertical_align
        || content_cell.apply_inner_margin != template_cell.apply_inner_margin
        || content_cell.is_header != template_cell.is_header
        || source_para.para_shape_id != template_para.para_shape_id
        || source_para.style_id != template_para.style_id
        || !cloned_first_char_shape
        || !cloned_first_line
    {
        return false;
    }

    let mut other_cells = table
        .cells
        .iter()
        .filter(|cell| cell.row as usize != row)
        .peekable();
    other_cells.peek().is_some()
        && other_cells.all(|cell| cell.col == 0 && cell.col_span == table.col_count)
}

/// Flow spacing repeated around a proven non-TAC RowBreak table fragment.
///
/// The first fragment keeps the formatted host-before value (paragraph spacing + outer top),
/// while a continuation repeats only the table's outer top.  Every fragment reserves the outer
/// bottom.  `repeat_outer_margin` is the narrow native-HWP evidence gate; applying this to every
/// RowBreak table is disproven by the #2097 COM page pins. `vertical_offset` remains a
/// first-fragment-only concern at the call site. `repeat_cellbreak_outer_margin` is the [#5922]
/// native-HWP CellBreak contract: the same reopen for proven empty-host TopAndBottom CellBreak
/// fragments (거대 표의 저장 ladder 는 표 높이를 접어 #2439 증거를 요구할 수 없다).
fn partial_rowbreak_fragment_spacing_px(
    table: &crate::model::table::Table,
    first_fragment_host_before: f64,
    is_continuation: bool,
    repeat_outer_margin: bool,
    repeat_cellbreak_outer_margin: bool,
    dpi: f64,
) -> (f64, f64) {
    let repeats_outer_margin = !table.common.treat_as_char
        && is_para_topbottom_float(&table.common)
        && match table.page_break {
            crate::model::table::TablePageBreak::RowBreak => repeat_outer_margin,
            crate::model::table::TablePageBreak::CellBreak => repeat_cellbreak_outer_margin,
            crate::model::table::TablePageBreak::None => false,
        };
    let before = if is_continuation {
        if repeats_outer_margin {
            hwpunit_to_px(table.outer_margin_top as i32, dpi)
        } else {
            0.0
        }
    } else {
        first_fragment_host_before
    };
    let bottom = if repeats_outer_margin {
        hwpunit_to_px(table.outer_margin_bottom as i32, dpi)
    } else {
        0.0
    };
    (before, bottom)
}
struct TypesetState {
    /// 완성된 페이지 목록
    pages: Vec<PageContent>,
    /// 현재 단에 쌓이는 항목
    current_items: Vec<PageItem>,
    /// 현재 단에서 소비된 높이 (px)
    current_height: f64,
    /// 현재 단 시작 시점의 논리 높이 (px)
    current_start_height: f64,
    /// 현재 단에 미주 흐름 항목이 포함되어 있는지 여부
    current_endnote_flow: bool,
    /// [#5886] 현재 단에 문단-사이 compact 되감김을 넣었으면, 이후 문단도
    /// 렌더 순차 적층이 용지 밖으로 나가는지 시뮬한다.
    column_had_compact_endnote_rewind: bool,
    /// [Task #1082] 현재 단에서 마지막으로 배치된 본문 FullParagraph 의 bottom vpos (HU,
    /// 섹션 절대값). 미주 vpos-delta 누적의 첫 항목 base 시드용. 단 advance 시 None.
    prev_body_bottom_vpos: Option<i32>,
    /// [#2279] 현재 단의 flow 과소 누계 (px) — 문단 place 시 flow_advance_height 가
    /// spacing_after/trailing ls 를 트림한 차액(total_height − advance)의 합.
    /// 렌더러(layout)와 한글은 이 성분을 가산하므로, footer(발신명의) fit 판정의
    /// 렌더-정합 좌표 복원용. 단 advance 시 0.
    flow_underrun: f64,
    /// [compat 2024] 이번 단에서 자리차지 표 앵커 문단의 선행 앵커 줄 세그를
    /// 흐름에서 회수한 양(px 누계). hangul2024_layout 에서만 쌓이며, 저장 vpos
    /// 되감김 쪽-경계 신호를 재적합으로 덮을 자격 판정에 쓴다 — 회수가 없던
    /// 쪽에서는 되감김 신호를 그대로 존중해 여타 문서 동작을 바꾸지 않는다.
    hangul2024_reclaimed: f64,
    /// [compat 2024] 저장 리셋/되감김 신호를 덮은 빈 문단의 인덱스 — 그 문단만
    /// place 적합을 우회해 쪽 하단 여백으로 흘린다(이웃 빈 문단까지 흘리면
    /// 2024 보다 한 문단 과적재, idx22 실측).
    hangul2024_spill_para: Option<usize>,
    /// [#2279 pi78] 이 문서에서 저장 ladder 의 host spacing 누락 서명(OMIT)이
    /// 검출됐는가 — 기계생성 압축 ladder 문서군 판별(문서 단위, 리셋 없음).
    /// 분할 진입 첫 줄 full-advance 요구는 이 문서군에만 적용한다.
    stored_ladder_spacing_omitted: bool,
    /// [#2279 OMIT-eager] 구역 시작 사전 스캔으로 확정한 fresh-재계산 문서군
    /// (spacing-누락 스텝 + 본문 텍스트 쪽 리셋 + segless 본문 문단 동시 보유).
    /// 페이지말 빈 문단 재배치·sa 스냅 보존 규칙은 이 판별에만 발동한다 —
    /// lazy(#2383) 공유 플래그에 얹으면 저장 흐름 신뢰 문서(sample16 #2158 핀)
    /// 까지 번져 +1 회귀.
    omit_fresh_recalc_doc: bool,
    /// [#5699 H1] 이 쪽에서 사다리-미계상 표 밴드 교정으로 확보한 흐름 바닥(px).
    /// 후속 문단의 저장 vpos 후방 스냅이 교정분을 되돌리지 못한다. 쪽 단위 리셋.
    ladder_band_floor: f64,
    /// [#5699 H1] 이번 단에서 교정 판별이 발동한 표 (para, ctrl) — 단 flush 시
    /// 소속 페이지의 `ladder_band_tables` 로 이관해 렌더러와 판정을 공유한다.
    current_ladder_band_tables: Vec<(usize, usize)>,
    /// 현재 단 인덱스
    current_column: u16,
    /// 단 수
    col_count: u16,
    /// 페이지 레이아웃
    layout: PageLayoutInfo,
    /// 구역 인덱스
    section_index: usize,
    /// 각주 높이 누적
    /// [#4090] Square 어울림 개체가 만든 세로 배제 밴드의 바닥(쪽 기준 px).
    /// 밴드를 벗어나거나 쪽이 끝날 때 흐름을 이 값으로 끌어올린다. 0.0 = 없음.
    square_band_bottom: f64,
    /// Square 배제 밴드의 흐름 기준 상단. 옆 문단의 저장 좌표를 현재 흐름 좌표계로
    /// 옮겨 밴드 바닥을 확장할 때만 사용한다.
    square_band_top: Option<f64>,
    current_footnote_height: f64,
    /// [Task #1658 v3] 페이지 하단 고정 표(vert=쪽·valign=Bottom, 결재/서명 틀)의
    /// 하단 배타 영역 높이 — 겹침 허용이므로 합이 아닌 max(union). 본문 텍스트는
    /// 이 영역 위까지만 흐른다 (available_height 차감). 페이지 전환 시 리셋.
    current_bottom_fixed_exclusion: f64,
    /// [Task #1658 v3] 이 페이지에서 하단 고정 표가 소비했을 저장-flow 높이 누계.
    /// 한글 저장 vpos 는 하단 틀도 문서순으로 누적하므로, 후속 틀의 vpos 동기화
    /// (#1611) 시 이 값을 차감해야 본문 텍스트 끝 위치가 복원된다.
    bottom_fixed_consumed_flow: f64,
    /// [#2279 footer-오염] 이 페이지에 PAGE-앵커(vertRelTo=Page, vertAlign=Top)
    /// 절대배치 비-TAC 표가 배치됐는가. 이 표들의 저장 vpos 누적은 절대 위치
    /// 산물이라 본문 흐름과 무관하게 부풀며(36496000 pi3: 저장 스텝 +482px vs
    /// 흐름 +143px), 이후 footer 의 저장 vpos 동기화(target_y)를 오염시킨다.
    /// 켜져 있으면 footer 는 stored 동기화 없이 흐름 좌표로 판정한다.
    page_has_page_abs_top_table: bool,
    /// [#2813] 저장 앵커 줄이 float 스택 아래를 인코딩하는 host 문단: 이 문단의
    /// PartialParagraph(앵커 줄) 아이템을 표들 뒤에 밀어 넣어(한글 문서순) 렌더가
    /// 표를 줄 이후 흐름에 배치하지 않게 한다. 값은 해당 para_idx.
    defer_host_line_item_para: Option<usize>,
    /// 첫 각주 여부
    is_first_footnote_on_page: bool,
    /// 현재 physical page에 실제로 그려질 각주 구분선이 이미 예약됐는가.
    /// 번호 없는 continuation tail은 첫 항목이어도 구분선을 생략할 수 있으므로,
    /// `is_first_footnote_on_page`만으로는 뒤에 오는 일반 note의 separator 비용을
    /// 판정할 수 없다.
    current_page_has_footnote_separator: bool,
    /// 각주 구분선 오버헤드
    footnote_separator_overhead: f64,
    /// 각주 사이 간격
    footnote_between_notes_margin: f64,
    /// 각주 안전 여백
    footnote_safety_margin: f64,
    /// [#2559] 현재 구역에 꼬리말 정의가 없는가. 빈 꼬리말 밴드는 각주가 먼저
    /// 사용하므로, 이 경우에만 각주 높이의 일부를 본문 가용 높이에서 회수한다.
    section_has_no_footer: bool,
    /// 존(zone) y 오프셋 (다단 나누기 시 누적)
    current_zone_y_offset: f64,
    /// 현재 존의 레이아웃 오버라이드
    current_zone_layout: Option<PageLayoutInfo>,
    /// 다단 첫 페이지 여부
    on_first_multicolumn_page: bool,
    /// Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지하는 높이 (px).
    /// col 1 이상으로 advance 시 zone_y_offset에 반영.
    pending_body_wide_top_reserve: f64,
    /// visible text host 의 양수 offset 자리차지 표가 후속 문단을 밀어내는 구간.
    visible_float_exclusions: Vec<VisibleFloatExclusion>,
    /// 현재 쪽에 실제 배치된 그림의 점유 영역(용지 좌표). 다음 단에도 간섭할 수 있다.
    side_wrap_exclusions:
        std::collections::BTreeMap<(usize, usize), super::layout_frame::FrameExclusion>,
    inline_placements:
        std::collections::HashMap<(usize, usize), super::float_placement::InlineBoxPlacement>,
    inline_flow_plans: std::collections::HashMap<usize, super::inline_flow::InlineFlowPlan>,
    paragraph_float_placements:
        std::collections::HashMap<(usize, usize), super::float_placement::ParagraphFloatPlacement>,
    /// 단 상대 TAC 물리 하단. 저장 host 높이와 별개로 다음 어울림 후보 줄을 제한한다.
    inline_box_flow_bottom: f64,
    /// 같은 문단의 선행 RowBreak 표가 continuation 을 만들 때 후행 co-anchored 표를
    /// 후속 섹션 블록 뒤로 잠시 미루기 위한 큐.
    deferred_table_controls: Vec<DeferredTableControl>,
    /// native HWP5 page-tail Square 그림을 다음 physical page의 시작에 넣는 큐.
    /// 단일 컬럼·caption 보유 picture 형상으로 한정한다.
    deferred_next_page_square_pictures: Vec<DeferredSquarePictureControl>,
    /// 다음 physical page의 flush 시점에만 앞에 붙일 Square picture.
    /// `current_items`에 즉시 넣으면 out-of-flow 그림이 문단 fit/vpos 상태를 바꾸어
    /// p1356 뒤 본문을 한 쪽 더 분할한다. layout 순서에는 앞에 있어야 하지만,
    /// typeset 흐름 항목으로는 보이면 안 된다.
    page_start_square_pictures: Vec<DeferredSquarePictureControl>,
    /// 표 조판 중 fragment별로 각주를 등록한 source 키. caller의 기존 표 완료 뒤
    /// 일괄 등록을 건너뛰어 중복을 막는다.
    fragment_queued_table_footnotes: std::collections::HashSet<(usize, usize)>,
    /// RowBreak 표의 남은 cell-footnote를 새 physical page에 먼저 예약했는가.
    /// 호출자 루프가 표 host의 저장 VPOS를 다시 기록하지 않고, 새 page 본문은
    /// 독립한 VPOS cursor로 시작하도록 하는 1회 신호다.
    reset_vpos_after_queued_table_footnote_page: bool,
    /// [Task #1753] 지연 이월되는 visible-host 자리차지 표 직전에 현재 쪽 잔여 공간으로
    /// 선행 배치(prefill)된 후속 문단들 — 메인 루프에서 스킵.
    prefilled_paras: std::collections::HashSet<usize>,
    /// [Task #1755] 이월 전 쪽에 host 텍스트 줄을 PartialParagraph 로 pre-emit 한 문단 —
    /// layout 의 마지막 fragment 뒤 host 렌더 억제 신호(PaginationResult 로 전달).
    pre_emitted_host_paras: std::collections::HashSet<usize>,
    /// [#2015] pre-emit 한 host 텍스트의 실제 높이(px). vert_offset(para_start 기준)를
    /// current_height(=para_start+host_h) 기준으로 환산할 때 감액분으로 쓴다. typeset 예산과
    /// layout(table_partial.rs) 배치가 동일 감액을 적용해 정합한다.
    pre_emitted_host_heights: std::collections::HashMap<usize, f64>,
    /// [Task #359] 다음 pi 가 vpos-reset 가드를 발동할 예정 → 현재 pi 의 fit 안전마진 비활성화.
    /// 단독 항목 페이지 발생 차단용.
    skip_safety_margin_once: bool,
    /// [Task #1725] tail-before-vpos-reset 문단 1회 각주 안전마진(40px) 비활성화.
    /// 각주 있는 페이지에서 한글 LINESEG 는 tail 문단을 본문에 배치(각주는 아래)하는데,
    /// rhwp 각주 예약(+40px 버퍼)이 tail 을 수 px 초과로 밀어 near-empty 페이지 over-pagination.
    skip_footnote_margin_once: bool,
    /// 다음 저장 vpos-reset 직전 tail의 실제 저장 line bounds. 현재 flow와 저장 bottom의
    /// 차이만 다음 문단의 fit allowance로 쓴다.
    tail_saved_bounds_once: Option<(f64, f64)>,
    /// #2439: 단일 양수-offset 빈 호스트 RowBreak 표 뒤 일반 문단의 1회 엄격 fit.
    /// 이 문단은 표의 실제 painted bottom 뒤에서 시작하므로 저장 page-tail 예외와
    /// trailing line-spacing 트림을 적용하지 않는다.
    strict_plain_text_fit_after_empty_host_float_once: bool,
    /// [#2403] 소스분기 질의 표면 — 단일 소유, typeset 진입 시 set. 종전 4필드의
    /// 시멘틱 승계: hwp3_layout()=HWP3→HWP5 변환본(widow 방지 등 variant 분기,
    /// Task #1007) / hwp3_native_layout()=원본 HWP3 (저장 LINE_SEG 계약) /
    /// hwpx_stored_layout()=HWPX 원본 (빈 앵커 TAC 표 host_line_spacing 미가산,
    /// Task #1147) / hwp5_origin_hwpx()=rhwp HWP5→HWPX 산출물 (HWP5 저장 행높이·
    /// pagination marker 보존).
    profile: crate::model::provenance::LayoutCompatibilityProfile,
    /// 문서 전체에 실제 저장 LineSeg가 있는지 여부. HWP5-origin CFB라도 LineSeg가 전부
    /// 비어 있으면 저장 vpos 기반 흐름이 아니라 재조판 흐름으로 취급한다.
    has_stored_line_segs: bool,
    /// [Task #362] 한컴 빈 줄 감추기 옵션 (SectionDef bit 19). true 이면 페이지 시작에서
    /// overflow 유발하는 빈 paragraph 최대 2개까지 height=0 처리.
    hide_empty_line: bool,
    /// [Task #362] 현재 페이지에서 감춘 빈 줄 수 (페이지마다 reset, 최대 2).
    hidden_empty_lines: u32,
    /// [Task #362] 감춘 빈 줄이 적용된 페이지 인덱스 (페이지 변경 감지용).
    hidden_empty_page_idx: usize,
    /// [Task #362] hide_empty_line 으로 감춘 paragraph 인덱스 (PaginationResult 에 포함).
    hidden_empty_paras: std::collections::HashSet<usize>,
    /// [#6146] 저장 vpos 리셋으로 다음 쪽에 넘어가는 문단의 **자리차지 밴드**를 떠나는
    /// 쪽의 흐름 말미에 남긴 (문단, 컨트롤) 집합. 컨트롤 순회에서 다시 배치하지 않는다.
    page_tail_spilled_floats: std::collections::HashSet<(usize, usize)>,
    /// [Task #836] 미주 목록 (섹션별 수집, 문서 끝에 렌더).
    endnotes: Vec<EndnoteRef>,
    endnote_paragraphs: Vec<Paragraph>,
    endnote_para_sources: Vec<EndnoteParaSource>,
    /// [Task #1246] 현재 섹션 미주의 between-notes 마진(HU, 0=미적용). HeightCursor 가 미주 사이
    /// min-gap 보정에 사용. 모든 경계에서 동일한 섹션 설정값이므로 스칼라로 보관.
    endnote_between_notes_hu: i32,
    /// 현재 섹션 미주의 정규화된 "구분선 위" 마진(HU).
    endnote_separator_above_hu: i32,
    /// 현재 섹션 미주의 정규화된 "구분선 아래" 마진(HU).
    endnote_separator_below_hu: i32,
    /// [Task #362] Square wrap 표의 column_start (HU). -1 = 비활성. 후속 같은 cs/sw paragraph 흡수용.
    wrap_around_cs: i32,
    /// NO_LS 문서용 합성 어울림 배제 사각형들.
    /// (x0, x1, flow_top, flow_bottom) — x 는 페이지 px, y 는 단 공통 flow 좌표(px).
    /// Square 그림은 앵커 단 밖(다른 단 위)에 놓일 수 있으므로 페이지 공간으로 기억해
    /// 두고, 각 문단 배치 시 현재 단과의 교차로 텍스트 존(cs/sw)을 산출한다.
    /// 쪽이 바뀌면 비운다.
    wrap_synth_rects: Vec<(f64, f64, f64, f64)>,
    /// [Task #362] Square wrap 표의 segment_width (HU). -1 = 비활성.
    wrap_around_sw: i32,
    /// [Task #362] Square wrap 표가 있는 paragraph 인덱스 (WrapAroundPara 에 기록).
    wrap_around_table_para: usize,
    /// 비-TAC Picture/Shape Square wrap: any_seg_matches만으로 후속 문단 판정 허용.
    /// 그림의 lineseg는 첫 seg cs=0일 수 있어 전체 seg 중 하나라도 일치하면 흡수.
    wrap_around_any_seg: bool,
    /// [#6175] 밴드가 호스트 문단의 저장 사다리가 아니라 어울림 개체의 기하에서
    /// 유도됐다 — 개체 종류(묶음 포함)와 무관하게 뒤따르는 문단을 개체 옆으로 흘린다.
    wrap_around_derived_band: bool,
    /// [#1955] 글뒤로/글앞으로(BehindText/InFrontOfText) 비-TAC 표 anchor 문단.
    /// 이 wrap 은 본문 플로우를 소비하지 않으므로(한글: 후속 문단이 anchor 쪽에
    /// 남음), 직후의 빈 후행 문단들을 anchor 첫 fragment 단에 소급 흡수한다.
    /// 비어있지 않은 문단/새 표/쪽나누기를 만나면 해제.
    behind_float_table_para: Option<usize>,
    /// [#4533 HWP3] 현재 문단 다음 문단의 첫 저장 lineseg vpos — 자리차지
    /// 밴드 비예약(사다리 증거) 판별용. 문단 루프 머리에서 세팅.
    next_para_first_stored_vpos: Option<i32>,
    /// [#5870] 다음 문단이 빈 host 자리차지 표 앵커인가 — 빈-host float 의
    /// 물리-사다리 여분 가산 발동 조건. 문단 루프 머리에서 세팅.
    next_para_is_empty_float_table_anchor: bool,
    /// [#6312] 다음 문단이 개체 없는 실텍스트 본문인가 — 글 있는 자리차지 host
    /// 줄 상자 가산의 사다리 게이트.
    next_para_is_plain_text: bool,
    /// [#1955] 글뒤로 표 후행 빈 문단의 보류 흡수 목록. 표 fragment 는 지연 flush
    /// 되므로 흡수 시점에는 anchor 첫 fragment 단을 찾을 수 없다 — 페이지 확정 후
    /// (최종 flush 뒤) 첫 fragment 단에 일괄 부착한다.
    behind_pending_absorbs: Vec<crate::renderer::pagination::WrapAroundPara>,
    /// [#4514] 이 문단의 overlay 표가 #703 Shape 단축(흐름 소비 0)으로 배치되었음.
    /// 이 앵커는 #1955 흡수를 arming 하지 않는다 — 흡수의 전제("표가 fragment 로
    /// 플로우를 이미 소비")가 성립하지 않아, 후행 빈 문단이 유일한 흐름 공간이다
    /// (sample1-repro 저장 사다리: 필러가 각자 줄 높이만큼 전진해 표 높이를 채움.
    /// 흡수하면 갭이 어느 쪽에도 계상되지 않아 후속 표가 겹침 — 8쪽 555.5px).
    overlay_shape_shortcut_para: Option<usize>,
    /// [#4568] 쪽 하단을 넘는 overlay 표의 **다음 단/쪽으로 넘길 잔여 행** 대기열.
    ///
    /// overlay 표는 흐름 소비가 0 이라 앵커 쪽에서 다음 쪽 항목을 바로 push 할 수 없다
    /// (다음 쪽은 이후 문단이 흐름을 넘길 때 비로소 생긴다). 그래서 앵커 쪽에서
    /// "몇 번째 행부터 잘렸는지"만 적어 두고, 단/쪽이 열릴 때 그 시작에 방출한다.
    /// `(para_index, control_index, start_row, reserve_px)`.
    ///
    /// [#5792] `reserve_px` 는 잔여 행이 새 쪽 흐름에서 예약해야 할 높이다 — 뒤따르는
    /// 흐름이 그 자리를 스스로 만드는 형상(#4514 필러 문단)에서는 0 이다.
    pending_overlay_continuations: Vec<(usize, usize, usize, f64)>,
    /// [#4568] 현재 단에 이어 그릴 overlay 잔여 행 목록. `flush_column` 에서
    /// `ColumnContent::overlay_continuations` 로 옮긴다.
    current_column_overlay_continuations: Vec<crate::renderer::pagination::OverlayContinuation>,
    /// [#4568] 현재 단의 overlay 표 앵커 컷 — `(para, ctrl, end_row)`.
    current_column_overlay_cuts: Vec<(usize, usize, usize)>,
    /// [Task #362] 현재 단에서 표 옆에 배치되는 wrap-around paragraphs.
    /// flush_column 에서 ColumnContent 로 전달.
    current_column_wrap_around_paras: Vec<crate::renderer::pagination::WrapAroundPara>,
    /// [Task #604 R3] 현재 단의 wrap text 문단 ↔ anchor 메타데이터.
    /// wrap_around state machine 매칭 시 등록. flush_column 에서 ColumnContent 로 전달.
    current_column_wrap_anchors:
        std::collections::HashMap<usize, crate::renderer::pagination::WrapAnchorRef>,
    /// [Task #702] 현재 zone 의 ColumnType (Normal/Distribute/Parallel).
    /// process_multicolumn_break 에서 새 ColumnDef 매칭 시 갱신.
    /// Distribute 다단의 짧은 컬럼 vpos-reset 검출 임계값 완화에 사용.
    current_zone_column_type: ColumnType,
    /// [Task #853] 현재 zone 의 "디자인 spacing"(px) — 1단 ColumnDef 의 `간격` 값.
    /// 한컴은 1단 ColumnDef 의 `간격`(가로 단 간격이지만 1단이라 무의미)을 zone 진입
    /// 세로 간격으로 쓴다(shortcut.hwp 1쪽 헤더 띠 = 10mm). zone 전환 시
    /// (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2) 를 zone_y_offset 에
    /// 더한다. 다단(2+) ColumnDef 의 `간격`은 가로 간격이므로 0 으로 둔다.
    current_zone_design_spacing_px: f64,
    /// [Task #1027 Stage D] 컬럼 단위 vpos 스냅 상태 (렌더러 build_single_column 정합).
    /// current_height 상대공간(col_area_y=0)에서 HeightCursor 를 구동한다.
    vpos_page_base: Option<i32>,
    vpos_lazy_base: Option<i32>,
    /// [#2243] page_base 가 **저장**(비합성) lineseg 에서 왔는지. 저장-앵커 사다리는
    /// 저장 lineseg 없는 문단(생성기 누락 — 한글 fresh 재계산으로 성장 가능)을
    /// 만나면 연속성이 깨지므로 dirty 판단에 쓴다.
    vpos_page_base_stored: bool,
    /// [#2243] 저장-앵커 사다리에 저장 lineseg 없는 문단이 끼어 dirty — 이후
    /// 사다리 역스냅(backward)은 fresh 성장분을 뭉갤 수 있어 금지(전방만 허용).
    vpos_ladder_dirty: bool,
    vpos_prev_layout_para: Option<usize>,
    vpos_prev_partial_table: bool,
    /// 컬럼 시작 시점의 current_height (page_path anchor — 렌더러 col_anchor_y 대응).
    vpos_col_anchor: f64,
    /// HWP3-origin 흐름에서는 vpos 보정에서 spacing_before 사전 차감을 생략한다(#1116).
    skip_spacing_before_prededuct: bool,
    /// [#6753] 직전 문단이 `#2279 ①` 트림으로 흐름에서 뺀 `spacing_before`(px).
    /// lazy 기준 역산이 그 트림분을 기준점에 싣지 않도록 `HeightCursor` 로 넘긴다.
    vpos_prev_trimmed_sb_px: f64,
}

/// [Task #853] ColumnDef 의 "디자인 spacing"(px): 1단이면 `간격`, 다단이면 0.
fn column_def_design_spacing_px(cd: &ColumnDef, dpi: f64) -> f64 {
    if cd.column_count.max(1) <= 1 {
        hwpunit_to_px(cd.spacing as i32, dpi)
    } else {
        0.0
    }
}

/// [#5918] 현재 단이 block-table continuation 꼬리 조각(들)과 빈 필러 문단만
/// 담고 있는지 — 저장 vpos 리셋의 이중 쪽 경계 판정용. 꼬리 조각이 저장 경계와
/// 같은 물리 쪽 경계를 이미 열어 놨다면 리셋의 advance는 중복이므로 호출부에서
/// 건너뜀.
///
/// 추가로 꼬리 조각이 쪽의 **소수 부분**(30% 이하)만 차지할 때 한정한다.
/// 드레인이 새로 연 쪽에 조각이 작게 남을 때는 그 쪽이 저장 사다리상 다음
/// 경계(리셋 문단)의 내용을 흡수할 예약 쪽이지만(sample1-repro pi=608:
/// 78px / pi=750: 220px), 조각 자체가 쪽을 대부분 채웠다면 그 쪽은 저장
/// 사다리에서 이미 소진된 독립 경계라 리셋은 별도의 다음 쪽을 가리킨다
/// (task2097/75544 pi=316: 909px·pi=525: 826px, hwpx_sample2 pi=138:
/// 1042px — 한글 COM/PDF 정답지가 전부 존중을 요구한다).
fn page_holds_only_fresh_table_continuation(st: &TypesetState, paragraphs: &[Paragraph]) -> bool {
    const FRESH_CONTINUATION_PAGE_MAX_FILL_RATIO: f64 = 0.30;
    if st.current_items.is_empty() {
        return false;
    }
    if st.current_height > st.available_height() * FRESH_CONTINUATION_PAGE_MAX_FILL_RATIO {
        return false;
    }
    let mut has_continuation = false;
    for item in &st.current_items {
        match item {
            PageItem::PartialTable {
                is_continuation, ..
            } if *is_continuation => has_continuation = true,
            PageItem::FullParagraph { para_index }
            | PageItem::PartialParagraph { para_index, .. } => {
                let empty_only = paragraphs
                    .get(*para_index)
                    .is_some_and(|p| p.text.trim().is_empty() && p.controls.is_empty());
                if !empty_only {
                    return false;
                }
            }
            _ => return false,
        }
    }
    has_continuation
}

fn para_has_visible_text(para: &Paragraph) -> bool {
    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
}

fn para_has_non_whitespace_text(para: &Paragraph) -> bool {
    para.text
        .chars()
        .any(|c| c > '\u{001F}' && c != '\u{FFFC}' && !c.is_whitespace())
}

fn take_strict_plain_text_fit_after_empty_host_float_once(
    pending: &mut bool,
    para: &Paragraph,
) -> bool {
    if !*pending || !para_has_non_whitespace_text(para) || !para.controls.is_empty() {
        return false;
    }
    *pending = false;
    true
}

fn paragraph_page_end_fit_height(
    total_height: f64,
    height_for_fit: f64,
    require_full_advance: bool,
) -> f64 {
    if require_full_advance {
        total_height.max(height_for_fit)
    } else {
        height_for_fit
    }
}

fn para_line_spacing_px(para: &Paragraph, dpi: f64) -> f64 {
    para.line_segs
        .last()
        .filter(|seg| seg.line_spacing > 0)
        .map(|seg| hwpunit_to_px(seg.line_spacing, dpi))
        .unwrap_or(0.0)
}

fn has_following_non_positive_visible_float(para: &Paragraph, control_index: usize) -> bool {
    para.controls
        .iter()
        .skip(control_index + 1)
        .any(|ctrl| match ctrl {
            Control::Table(table) => {
                is_para_topbottom_float(&table.common)
                    && signed_hwpunit(table.common.vertical_offset) <= 0
            }
            _ => false,
        })
}

fn para_is_non_tac_overlay_table_anchor(para: &Paragraph) -> bool {
    !para_has_non_whitespace_text(para)
        && para.controls.iter().any(|ctrl| {
            matches!(
                ctrl,
                Control::Table(table)
                    if !table.common.treat_as_char
                        && matches!(
                            table.common.text_wrap,
                            crate::model::shape::TextWrap::InFrontOfText
                                | crate::model::shape::TextWrap::BehindText
                        )
            )
        })
}

fn para_is_empty_topbottom_table_anchor(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::Table(t) if is_para_topbottom_float(&t.common)))
}

/// native HWP5의 그림 표 뒤에 붙은 빈 문단 중, 저장 vpos가 표의 실제 paint span 안에
/// 들어가는 것은 본문 줄이 아니라 그림 위치를 유지하는 guide line이다.
///
/// 이 fixture의 p182 `그림 67`은 빈 host 2×1 RowBreak 표 뒤에 같은 PS의 빈 line 다섯
/// 개를 남긴다. 표는 이미 선언 높이만큼 흐름을 소비했으므로 guide line까지 다시
/// advance하면 다음 두 본문 문단이 p183으로 밀린다. 일반 빈 줄은 의도된 간격일 수
/// 있으므로, native HWP5·empty host·positive offset·2×1 figure table·동일 PS·표 span
/// 내부라는 저장 계약이 모두 있을 때만 숨긴다.
fn native_hwp5_figure_table_overlay_guide_empty(
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
) -> bool {
    if para.controls.is_empty()
        && !para_has_visible_text(para)
        && para.column_type == ColumnBreakType::None
    {
        let Some(guide_vpos) = para
            .line_segs
            .iter()
            .find(|seg| !is_synthetic_line_seg(seg))
            .map(|seg| seg.vertical_pos)
        else {
            return false;
        };
        let Some((_, host)) = (0..para_idx).rev().find_map(|idx| {
            let candidate = paragraphs.get(idx)?;
            (!candidate.controls.is_empty() || para_has_visible_text(candidate))
                .then_some((idx, candidate))
        }) else {
            return false;
        };
        if host.para_shape_id != para.para_shape_id || para_has_visible_text(host) {
            return false;
        }
        let Some((anchor_vpos, table)) = host
            .line_segs
            .iter()
            .find(|seg| !is_synthetic_line_seg(seg))
            .map(|seg| seg.vertical_pos)
            .and_then(|anchor_vpos| {
                host.controls.iter().find_map(|control| match control {
                    Control::Table(table)
                        if !table.common.treat_as_char
                            && is_para_topbottom_float(&table.common)
                            && matches!(
                                table.page_break,
                                crate::model::table::TablePageBreak::RowBreak
                            )
                            && table.row_count == 2
                            && table.col_count == 1
                            && table.cells.len() == 2
                            && signed_hwpunit(table.common.vertical_offset) > 0
                            && table.cells.first().is_some_and(|cell| {
                                cell.paragraphs.iter().any(|cell_para| {
                                    cell_para.controls.iter().any(|control| {
                                        matches!(control, Control::Picture(_) | Control::Shape(_))
                                    })
                                })
                            }) =>
                    {
                        Some((anchor_vpos, table))
                    }
                    _ => None,
                })
            })
        else {
            return false;
        };
        let table_bottom = anchor_vpos
            .saturating_add(signed_hwpunit(table.common.vertical_offset).max(0))
            .saturating_add(table.common.height.min(i32::MAX as u32) as i32);
        return guide_vpos >= anchor_vpos && guide_vpos < table_bottom;
    }
    false
}

/// [Task #1863] 텍스트 없이 TAC 표만 담은 문단 — 시각적으로 단독 표 줄.
/// 빈 앵커 TopAndBottom 표 뒤에 이런 문단이 오면 표-표 스택이므로 앵커의
/// host_line_spacing 이 표 사이 시각 간격이다 (#1133 과 동일 본질).
fn para_is_empty_tac_table_anchor(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::Table(t) if t.common.treat_as_char))
}

fn para_has_visible_text_or_equation(para: &Paragraph) -> bool {
    para_has_visible_text(para)
        || para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Equation(eq) if eq.common.treat_as_char))
}

fn para_has_visible_text_and_treat_as_char_equation(para: &Paragraph) -> bool {
    para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Equation(eq) if eq.common.treat_as_char))
}

/// HWPX가 자동으로 만드는 미주 제목은 자동번호 컨트롤과 짧은 답 표식만 가진다.
/// 리터럴 문항명이 남은 HWP5 제목과 구별해, 단 하단에서 제목만 고립시키지 않는다.
fn para_is_short_auto_endnote_marker(para: &Paragraph) -> bool {
    para.text
        .chars()
        .filter(|c| *c > '\u{001F}' && *c != '\u{FFFC}' && !c.is_whitespace())
        .count()
        <= 1
        && para.controls.iter().any(|control| {
            matches!(
                control,
                Control::AutoNumber(number)
                    if number.number_type == crate::model::control::AutoNumberType::Endnote
            )
        })
}

fn is_treat_as_char_equation_control(ctrl: Option<&Control>) -> bool {
    matches!(ctrl, Some(Control::Equation(eq)) if eq.common.treat_as_char)
}

fn para_is_treat_as_char_picture_only(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para.controls.iter().any(|ctrl| match ctrl {
            Control::Picture(pic) => pic.common.treat_as_char,
            Control::Shape(shape) => shape.common().treat_as_char,
            _ => false,
        })
}

/// 문단의 가시 payload가 글자처럼 취급하는 그림/도형으로만 이뤄졌는가.
///
/// `para_is_treat_as_char_picture_only`는 기존 조판 경로를 위해 TAC 그림이 **하나라도** 있는
/// 텍스트 없는 문단을 가리킨다. 단일 단 저장 vpos-reset을 강제 분리하는 경우에는 표·수식처럼
/// 다른 컨트롤이 섞이면 안 되므로 더 좁은 판정이 필요하다.
fn para_has_only_treat_as_char_picture_or_shape(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && !para.controls.is_empty()
        && para.controls.iter().all(|ctrl| match ctrl {
            Control::Picture(pic) => pic.common.treat_as_char,
            Control::Shape(shape) => shape.common().treat_as_char,
            _ => false,
        })
}

fn para_has_treat_as_char_picture_or_shape(para: &Paragraph) -> bool {
    para.controls.iter().any(|ctrl| match ctrl {
        Control::Picture(pic) => pic.common.treat_as_char,
        Control::Shape(shape) => shape.common().treat_as_char,
        _ => false,
    })
}

fn non_tac_picture_or_shape_common(ctrl: &Control) -> Option<&crate::model::shape::CommonObjAttr> {
    match ctrl {
        Control::Picture(pic) if !pic.common.treat_as_char => Some(&pic.common),
        Control::Shape(shape) if !shape.common().treat_as_char => Some(shape.common()),
        _ => None,
    }
}

fn para_has_non_tac_picture_or_shape(para: &Paragraph) -> bool {
    para.controls
        .iter()
        .any(|ctrl| non_tac_picture_or_shape_common(ctrl).is_some())
}

fn paper_overlay_object_bottom_abs_px(para: &Paragraph, dpi: f64) -> Option<f64> {
    if !crate::renderer::layout::para_is_floating_overlay_anchor(para) {
        return None;
    }

    let mut max_bottom: Option<f64> = None;
    for ctrl in &para.controls {
        let common = match ctrl {
            Control::Shape(shape) if !shape.common().treat_as_char => shape.common(),
            Control::Picture(pic) if !pic.common.treat_as_char => &pic.common,
            Control::Table(table) if !table.common.treat_as_char => &table.common,
            _ => continue,
        };
        if !matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Paper) {
            continue;
        }

        let top_hu = signed_hwpunit(common.vertical_offset);
        let bottom_hu = top_hu
            .saturating_add(common.height as i32)
            .saturating_add(common.margin.bottom as i32);
        let bottom_px = hwpunit_to_px(bottom_hu, dpi);
        max_bottom = Some(max_bottom.map_or(bottom_px, |prev| prev.max(bottom_px)));
    }

    max_bottom
}

fn para_is_columndef_only_separator(para: &Paragraph) -> bool {
    para.text.trim().is_empty()
        && !para.controls.is_empty()
        && para
            .controls
            .iter()
            .all(|c| matches!(c, Control::ColumnDef(_)))
}

fn para_is_paper_page_square_empty_table_anchor(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para.controls.iter().any(|ctrl| {
            matches!(ctrl, Control::Table(table)
            if !table.common.treat_as_char
                && matches!(
                    table.common.text_wrap,
                    crate::model::shape::TextWrap::Square
                )
                && matches!(
                    table.common.vert_rel_to,
                    crate::model::shape::VertRelTo::Paper
                        | crate::model::shape::VertRelTo::Page
                ))
        })
}

fn para_is_pre_paper_page_square_table_scaffold(para_idx: usize, paragraphs: &[Paragraph]) -> bool {
    let Some(para) = paragraphs.get(para_idx) else {
        return false;
    };
    if para_has_visible_text(para)
        || !matches!(
            para.column_type,
            ColumnBreakType::Page | ColumnBreakType::Section
        )
        || !para.controls.iter().all(|ctrl| {
            matches!(
                ctrl,
                Control::SectionDef(_) | Control::ColumnDef(_) | Control::Bookmark(_)
            )
        })
    {
        return false;
    }

    paragraphs
        .get(para_idx + 1)
        .is_some_and(para_is_paper_page_square_empty_table_anchor)
}

fn para_is_post_paper_page_square_table_scaffold(
    para_idx: usize,
    paragraphs: &[Paragraph],
) -> bool {
    let Some(para) = paragraphs.get(para_idx) else {
        return false;
    };
    if para_has_visible_text(para) {
        return false;
    }
    let empty_no_control_para =
        para.column_type == ColumnBreakType::None && para.controls.is_empty();
    let columndef_only_separator =
        para.column_type == ColumnBreakType::Column && para_is_columndef_only_separator(para);
    if !empty_no_control_para && !columndef_only_separator {
        return false;
    }

    paragraphs
        .get(para_idx.saturating_sub(1))
        .is_some_and(para_is_paper_page_square_empty_table_anchor)
        && paragraphs
            .get(para_idx + 1)
            .is_some_and(para_has_visible_text)
}

fn columndef_separator_between_floating_overlays(
    para_idx: usize,
    paragraphs: &[Paragraph],
) -> bool {
    let prev_overlay = (0..para_idx).rev().find_map(|idx| {
        let para = paragraphs.get(idx)?;
        if para.text.trim().is_empty()
            && (para.controls.is_empty() || para_is_columndef_only_separator(para))
        {
            return None;
        }
        Some(crate::renderer::layout::para_is_floating_overlay_anchor(
            para,
        ))
    });
    let next_overlay = paragraphs.iter().skip(para_idx + 1).find_map(|para| {
        if para.text.trim().is_empty()
            && (para.controls.is_empty() || para_is_columndef_only_separator(para))
        {
            return None;
        }
        Some(crate::renderer::layout::para_is_floating_overlay_anchor(
            para,
        ))
    });

    prev_overlay == Some(true) && next_overlay == Some(true)
}

fn non_tac_picture_or_shape_block_height_px(para: &Paragraph, dpi: f64) -> Option<f64> {
    let mut max_height = 0.0f64;
    let mut found = false;
    for ctrl in &para.controls {
        let Some(common) = non_tac_picture_or_shape_common(ctrl) else {
            continue;
        };
        let block_height_hu =
            common.height as i32 + common.margin.top as i32 + common.margin.bottom as i32;
        max_height = max_height.max(hwpunit_to_px(block_height_hu.max(1), dpi));
        found = true;
    }
    found.then_some(max_height)
}

fn non_tac_picture_or_shape_content_height_px(para: &Paragraph, dpi: f64) -> Option<f64> {
    let mut max_height = 0.0f64;
    let mut found = false;
    for ctrl in &para.controls {
        let Some(common) = non_tac_picture_or_shape_common(ctrl) else {
            continue;
        };
        max_height = max_height.max(hwpunit_to_px((common.height as i32).max(1), dpi));
        found = true;
    }
    found.then_some(max_height)
}

fn non_tac_square_picture_common(ctrl: &Control) -> Option<&crate::model::shape::CommonObjAttr> {
    let common = match ctrl {
        Control::Picture(pic) => Some(&pic.common),
        Control::Shape(shape) => {
            if let crate::model::shape::ShapeObject::Picture(pic) = shape.as_ref() {
                Some(&pic.common)
            } else {
                None
            }
        }
        _ => None,
    }?;
    (!common.treat_as_char && matches!(common.text_wrap, crate::model::shape::TextWrap::Square))
        .then_some(common)
}

/// [#6175] 비-TAC 어울림(Square) 개체의 공통 속성 — 그림뿐 아니라 묶음(GroupShape)
/// 등 모든 개체 종류를 받는다. 묶음 그림도 한컴에서는 같은 배제 밴드를 만든다.
fn non_tac_square_float_common(ctrl: &Control) -> Option<&crate::model::shape::CommonObjAttr> {
    let common = match ctrl {
        Control::Picture(pic) => Some(&pic.common),
        Control::Shape(shape) => Some(shape.common()),
        _ => None,
    }?;
    (!common.treat_as_char && matches!(common.text_wrap, crate::model::shape::TextWrap::Square))
        .then_some(common)
}

/// [#6175] 어울림 개체의 **자기 기하**만으로 유도되는 우측 밴드의 좌측 레인 폭.
///
/// 호스트 문단의 첫 줄이 전폭이면(그림이 그 줄보다 아래에서 시작하는 형상) 기존
/// arming 은 밴드를 못 만든다. 그러나 개체가 단 우단까지 닿는 문단/단 기준 개체라면
/// 옆으로 흐를 레인은 개체 자신의 `horizontal_offset` 이 그대로 규정한다 — 한컴이
/// 뒤따르는 문단의 `segment_width` 로 저장하는 값과 같은 수다(156518601 1쪽:
/// horzOffset 29138 = 저장 사다리 4줄 전부의 horzsize).
fn square_float_left_lane_width(para: &Paragraph, col_w_hu: i32) -> Option<i32> {
    use crate::model::shape::HorzRelTo;
    let mut floats = para.controls.iter().filter_map(non_tac_square_float_common);
    let common = floats.next()?;
    if floats.next().is_some() {
        return None;
    }
    if !matches!(common.horz_rel_to, HorzRelTo::Para | HorzRelTo::Column) {
        return None;
    }
    let lane = common.horizontal_offset as i32;
    let right_edge = lane
        .saturating_add(common.width as i32)
        .saturating_add(common.margin.right as i32);
    // 개체가 단 우단까지 닿아야 좌측 레인 하나로 정의된다. 가운데 놓인 개체는
    // 좌·우 두 레인을 만들므로 이 유도가 성립하지 않는다.
    (lane > 0 && lane < col_w_hu && right_edge >= col_w_hu - 200).then_some(lane)
}

/// NO_LS 문단의 Square 그림들을 페이지 공간 배제
/// 사각형으로 수집한다. 반환 튜플 = (x0, x1, flow_top, flow_bottom) — x 는 페이지 px,
/// y 는 단 공통 flow 좌표(px, 문단 상단 = flow_y 기준). Square 그림은 가로 오프셋으로
/// 앵커 단 밖(다른 단 위)에 놓일 수 있으므로 단이 아니라 페이지 공간으로 기억한다.
fn synth_square_wrap_rects(
    para: &Paragraph,
    anchor_col_x: f64,
    flow_y: f64,
    dpi: f64,
) -> Vec<(f64, f64, f64, f64)> {
    para.controls
        .iter()
        .filter_map(non_tac_square_picture_common)
        .filter_map(|cm| {
            let w = crate::renderer::hwpunit_to_px(cm.width as i32, dpi);
            let h = crate::renderer::hwpunit_to_px(cm.height as i32, dpi);
            if w < 8.0 || h < 8.0 {
                return None;
            }
            let ml = crate::renderer::hwpunit_to_px(cm.margin.left as i32, dpi);
            let mr = crate::renderer::hwpunit_to_px(cm.margin.right as i32, dpi);
            let mb = crate::renderer::hwpunit_to_px(cm.margin.bottom as i32, dpi);
            let x0 = anchor_col_x
                + crate::renderer::hwpunit_to_px(signed_hwpunit(cm.horizontal_offset), dpi)
                - ml;
            let x1 = x0 + ml + w + mr;
            let top = flow_y
                + crate::renderer::hwpunit_to_px(signed_hwpunit(cm.vertical_offset).max(0), dpi);
            let bottom = top + h + mb;
            Some((x0, x1, top, bottom))
        })
        .collect()
}

fn paragraph_by_global_index<'a>(
    body_paragraphs: &'a [Paragraph],
    endnote_paragraphs: &'a [Paragraph],
    para_index: usize,
) -> Option<&'a Paragraph> {
    if para_index < body_paragraphs.len() {
        body_paragraphs.get(para_index)
    } else {
        endnote_paragraphs.get(para_index - body_paragraphs.len())
    }
}

fn page_item_para_index(item: &PageItem) -> Option<usize> {
    match item {
        PageItem::FullParagraph { para_index }
        | PageItem::PartialParagraph { para_index, .. }
        | PageItem::Table { para_index, .. }
        | PageItem::PartialTable { para_index, .. }
        | PageItem::Shape { para_index, .. } => Some(*para_index),
        PageItem::EndnoteSeparator { .. } => None,
    }
}

fn page_item_vpos_base(item: &PageItem, paragraphs: &[Paragraph]) -> Option<i32> {
    match item {
        PageItem::PartialParagraph {
            para_index,
            start_line,
            ..
        } => paragraphs
            .get(*para_index)
            .and_then(|para| para.line_segs.get(*start_line))
            .map(|seg| seg.vertical_pos),
        PageItem::FullParagraph { para_index }
        | PageItem::Table { para_index, .. }
        | PageItem::PartialTable { para_index, .. }
        | PageItem::Shape { para_index, .. } => paragraphs
            .get(*para_index)
            .and_then(|para| para.line_segs.first())
            .map(|seg| seg.vertical_pos),
        PageItem::EndnoteSeparator { .. } => None,
    }
}

fn square_picture_wrap_anchor_for_para(
    st: &TypesetState,
    body_paragraphs: &[Paragraph],
    para: &Paragraph,
    page_def: &PageDef,
) -> Option<crate::renderer::pagination::WrapAnchorRef> {
    if st.wrap_around_cs < 0 {
        return None;
    }

    let para_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
    let para_sw = para
        .line_segs
        .first()
        .map(|s| s.segment_width as i32)
        .unwrap_or(0);
    let is_empty_para = para
        .text
        .chars()
        .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
        && para.controls.is_empty();
    let any_seg_matches = para.line_segs.iter().any(|s| {
        s.column_start == st.wrap_around_cs && s.segment_width as i32 == st.wrap_around_sw
    });
    let body_w =
        (page_def.width as i32) - (page_def.margin_left as i32) - (page_def.margin_right as i32);
    let sw0_match = st.wrap_around_sw == 0 && is_empty_para && para_sw > 0 && para_sw < body_w / 2;

    let anchor_para = paragraph_by_global_index(
        body_paragraphs,
        &st.endnote_paragraphs,
        st.wrap_around_table_para,
    )?;
    let anchor_image_match = if st.wrap_around_cs == 0 {
        let body_left = page_def.margin_left as i32;
        let expected_cs_hu = anchor_para
            .controls
            .iter()
            .find_map(|ctrl| {
                non_tac_square_picture_common(ctrl).map(|common| {
                    common.horizontal_offset as i32
                        + common.width as i32
                        + 2 * common.margin.right as i32
                        - body_left
                })
            })
            .unwrap_or(0);
        expected_cs_hu > 0
            && (para_cs - expected_cs_hu).abs() < 200
            && para_sw > 0
            && para_cs + para_sw <= body_w + 200
    } else {
        false
    };
    let cs_only_match = st.wrap_around_any_seg && para_cs == st.wrap_around_cs && para_sw > 0;
    let matched = (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
        || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
        || sw0_match
        || anchor_image_match
        || cs_only_match;
    if !matched {
        return None;
    }

    let anchor_image_margin_right = anchor_para.controls.iter().find_map(|ctrl| {
        non_tac_square_picture_common(ctrl).map(|common| common.margin.right as i32)
    })?;
    Some(crate::renderer::pagination::WrapAnchorRef {
        anchor_para_index: st.wrap_around_table_para,
        anchor_cs: st.wrap_around_cs,
        anchor_sw: st.wrap_around_sw,
        anchor_image_margin_right,
        band_y_range: None,
    })
}

fn maybe_register_square_picture_wrap_anchor(
    st: &mut TypesetState,
    body_paragraphs: &[Paragraph],
    para: &Paragraph,
    para_index: usize,
    page_def: &PageDef,
) {
    if st.wrap_around_cs < 0 {
        return;
    }
    if let Some(anchor) = square_picture_wrap_anchor_for_para(st, body_paragraphs, para, page_def) {
        st.current_column_wrap_anchors.insert(para_index, anchor);
    } else {
        st.wrap_around_cs = -1;
        st.wrap_around_sw = -1;
        st.wrap_around_any_seg = false;
        st.wrap_around_derived_band = false;
        st.close_square_band();
    }
}

fn activate_square_picture_wrap_for_para(
    st: &mut TypesetState,
    para_index: usize,
    para: &Paragraph,
) {
    if !para
        .controls
        .iter()
        .any(|ctrl| non_tac_square_picture_common(ctrl).is_some())
    {
        return;
    }

    let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
    let anchor_sw = para
        .line_segs
        .first()
        .map(|s| s.segment_width as i32)
        .unwrap_or(0);
    if anchor_cs > 0 || anchor_sw > 0 {
        st.wrap_around_cs = anchor_cs;
        st.wrap_around_sw = anchor_sw;
        st.wrap_around_table_para = para_index;
        st.wrap_around_any_seg = true;
        st.wrap_around_derived_band = false;
    }
}

fn line_has_strict_equation_tac_control(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let start = line.char_start;
    let end = composed_line_char_end(comp, line_idx);
    end > start
        && comp.tac_controls.iter().any(|(pos, _, ci)| {
            *pos >= start && *pos < end && is_treat_as_char_equation_control(para.controls.get(*ci))
        })
}

fn line_is_leading_empty_equation_tac_guide(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    let Some(next) = comp.lines.get(line_idx + 1) else {
        return false;
    };
    line.runs.is_empty()
        && line.char_start == next.char_start
        && !line_has_strict_tac_control(comp, line_idx)
        && line_has_strict_equation_tac_control(para, comp, line_idx + 1)
}

fn equation_only_tac_line_assignment(
    para: &Paragraph,
    comp: &ComposedParagraph,
) -> Option<Vec<usize>> {
    let n_lines = comp.lines.len();
    if n_lines <= 1 || comp.tac_controls.is_empty() {
        return None;
    }
    if !comp.lines.iter().all(|line| line.runs.is_empty()) {
        return None;
    }
    let degenerate = comp
        .lines
        .windows(2)
        .any(|w| w[1].char_start <= w[0].char_start);
    if !degenerate {
        return None;
    }

    let mut assign = vec![n_lines - 1; comp.tac_controls.len()];
    let mut line_idx = 0usize;
    let mut tac_idx = 0usize;
    while tac_idx < comp.tac_controls.len() {
        let pos = comp.tac_controls[tac_idx].0;
        while line_idx < n_lines && comp.lines[line_idx].char_start < pos {
            line_idx += 1;
        }

        let tac_start = tac_idx;
        while tac_idx < comp.tac_controls.len() && comp.tac_controls[tac_idx].0 == pos {
            tac_idx += 1;
        }
        let tac_count = tac_idx - tac_start;

        let line_start = line_idx;
        while line_idx < n_lines && comp.lines[line_idx].char_start == pos {
            line_idx += 1;
        }
        let line_candidates: Vec<usize> = (line_start..line_idx).collect();
        let filtered_candidates: Vec<usize> = line_candidates
            .iter()
            .copied()
            .filter(|idx| !line_is_leading_empty_equation_tac_guide(para, comp, *idx))
            .collect();
        let line_targets = if tac_count > 1 && line_candidates.len() >= tac_count {
            // 같은 char_start에 여러 TAC 수식이 있고 저장 LINE_SEG도 같은 수만큼 있으면
            // 선행 빈 guide 줄도 한컴의 물리 수식 줄로 보존한다.
            &line_candidates
        } else if filtered_candidates.is_empty() {
            &line_candidates
        } else {
            &filtered_candidates
        };

        for offset in 0..tac_count {
            assign[tac_start + offset] = if line_targets.is_empty() {
                line_start.min(n_lines - 1)
            } else {
                line_targets[offset.min(line_targets.len() - 1)]
            };
        }
    }

    Some(assign)
}

fn tac_control_indices_for_line(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> Vec<usize> {
    let Some(line) = comp.lines.get(line_idx) else {
        return Vec::new();
    };
    if comp.tac_controls.is_empty() {
        return Vec::new();
    }

    if let Some(assign) = crate::renderer::composer::stored_tac_line_assignment(para, comp) {
        return assign
            .into_iter()
            .filter_map(|(ci, owner)| (owner == line_idx).then_some(ci))
            .collect();
    }

    if let Some(assign) = equation_only_tac_line_assignment(para, comp) {
        return comp
            .tac_controls
            .iter()
            .enumerate()
            .filter_map(|(idx, (_, _, ci))| {
                (assign.get(idx).copied() == Some(line_idx)).then_some(*ci)
            })
            .collect();
    }

    if line.runs.is_empty() {
        let start = line.char_start;
        let end = comp
            .lines
            .get(line_idx + 1)
            .map(|next| next.char_start)
            .unwrap_or(usize::MAX);
        return comp
            .tac_controls
            .iter()
            .filter_map(|(pos, _, ci)| (*pos >= start && *pos < end).then_some(*ci))
            .collect();
    }

    let next_start = comp.lines.get(line_idx + 1).map(|next| next.char_start);
    let mut hits = Vec::new();
    let mut run_start = line.char_start;
    for (run_idx, run) in line.runs.iter().enumerate() {
        let run_len = run.text.chars().count();
        let run_end = run_start + run_len;
        let next_line_starts_at_run_end = next_start.is_some_and(|start| start == run_end);
        let allow_end_tac = run_idx == line.runs.len() - 1 && !next_line_starts_at_run_end;
        for (pos, _, ci) in &comp.tac_controls {
            if *pos >= run_start && (*pos < run_end || (allow_end_tac && *pos == run_end)) {
                hits.push(*ci);
            }
        }
        run_start = run_end;
    }
    hits
}

fn line_has_tac_equation_control(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    tac_control_indices_for_line(para, comp, line_idx)
        .iter()
        .any(|ci| is_treat_as_char_equation_control(para.controls.get(*ci)))
}

fn line_leading_tac_equation_count(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> usize {
    let Some(line_start) = comp.lines.get(line_idx).map(|line| line.char_start) else {
        return 0;
    };
    let line_controls = tac_control_indices_for_line(para, comp, line_idx);
    comp.tac_controls
        .iter()
        .filter(|(pos, _, ci)| {
            *pos == line_start
                && line_controls.contains(ci)
                && is_treat_as_char_equation_control(para.controls.get(*ci))
        })
        .count()
}

fn line_is_equation_tac_text_run_only(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    if line_has_visible_text(comp, line_idx) {
        return false;
    }

    let line_controls = tac_control_indices_for_line(para, comp, line_idx);
    !line_controls.is_empty()
        && line_controls
            .iter()
            .all(|ci| is_treat_as_char_equation_control(para.controls.get(*ci)))
}

fn line_has_visible_text_or_tac_equation(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
) -> bool {
    line_has_visible_text(comp, line_idx) || line_has_tac_equation_control(para, comp, line_idx)
}

fn line_has_tac_control(para: &Paragraph, comp: &ComposedParagraph, line_idx: usize) -> bool {
    !tac_control_indices_for_line(para, comp, line_idx).is_empty()
}

fn line_tac_picture_or_shape_height(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
    dpi: f64,
) -> Option<f64> {
    tac_control_indices_for_line(para, comp, line_idx)
        .iter()
        .find_map(|ci| {
            para.controls
                .get(*ci)
                .and_then(|ctrl| crate::renderer::tac_object_flow_height_px(ctrl, dpi))
        })
}

fn text_line_is_picture_lead_in(
    para: &Paragraph,
    comp: &ComposedParagraph,
    line_idx: usize,
    raw_lh: f64,
    max_fs: f64,
    dpi: f64,
) -> bool {
    if max_fs <= 0.0 || raw_lh <= max_fs * 2.0 {
        return false;
    }
    let Some(line) = comp.lines.get(line_idx) else {
        return false;
    };
    if line.runs.iter().all(|run| run.text.trim().is_empty())
        || line_tac_picture_or_shape_height(para, comp, line_idx, dpi).is_some()
    {
        return false;
    }
    let Some(next) = comp.lines.get(line_idx + 1) else {
        return false;
    };
    if !next.runs.iter().all(|run| run.text.trim().is_empty()) {
        return false;
    }
    line_tac_picture_or_shape_height(para, comp, line_idx + 1, dpi)
        .map(|height| (raw_lh - height).abs() <= 8.0)
        .unwrap_or(false)
}

/// 큰 inline `TopAndBottom` 그림 바로 앞 native HWP5 본문 문단의 physical-page
/// reset을 찾는다.
///
/// HWP5는 그림을 위한 다음 physical page의 본문 tail을 같은 paragraph의 LINE_SEG에
/// 보관할 수 있다. 일반 text reset은 다단 coordinate drift 또는 저장 시점의 잔재일 수
/// 있으므로 전역 page break 신호가 아니다. 다만 현재 쪽 하단의 visible text가 `vpos=0`
/// 으로 reset되고, 즉시 다음 문단이 page-top 부근에서 시작하는 큰 inline
/// TopAndBottom 그림 하나인 경우는 PDF의 본문-그림 owner 경계를 직접 표현한다.
///
/// 이 경로는 그림 앞 text tail만 PartialParagraph로 나누고 그림 자체는 다음 loop에서
/// normal flow로 배치한다. 따라서 Square/비-TAC float, 표, 각주, 작은 inline object 및
/// successor가 없는 일반 reset에는 적용하지 않는다.
fn native_hwp5_text_reset_before_large_tac_topbottom_picture_break_line(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    dpi: f64,
) -> Option<usize> {
    // [#5128] HWP5-origin HWPX 도 원본 HWP5 와 같은 저장 pagination 을 쓴다.
    // hwp5_stored_pagination_layout() 만 보면 스펙 문서 문단 84 앞 TAC 그림 분할이
    // 빠져 69→68 이 된다.
    if !st.profile.hwp5_stored_pagination_layout()
        || st.col_count != 1
        || st.current_footnote_height > 0.0
        || st.current_items.is_empty()
        || st.current_height < st.layout.body_area.height * 0.60
        || !para.controls.is_empty()
        || !para_has_visible_text(para)
        || fmt.line_heights.len() < 2
        || para.line_segs.len() < fmt.line_heights.len()
    {
        return None;
    }

    let next_para = paragraphs.get(para_idx + 1)?;
    if next_para.controls.len() != 1
        || next_para.line_segs.len() != 1
        || para_has_visible_text(next_para)
        || !para_controls_only_tac_topbottom_objects(next_para)
    {
        return None;
    }
    let next_line = next_para.line_segs.first()?;
    if is_synthetic_line_seg(next_line)
        || !(0..=8_000).contains(&next_line.vertical_pos)
        || hwpunit_to_px(next_line.line_height, dpi) < st.layout.body_area.height * 0.15
    {
        return None;
    }

    para.line_segs[..fmt.line_heights.len()]
        .windows(2)
        .enumerate()
        .find_map(|(prev_idx, pair)| {
            let (prev, next) = (&pair[0], &pair[1]);
            if is_synthetic_line_seg(prev)
                || is_synthetic_line_seg(next)
                || prev.vertical_pos <= 0
                || next.vertical_pos != 0
            {
                return None;
            }
            (hwpunit_to_px(prev.vertical_pos.saturating_add(prev.line_height), dpi)
                >= st.layout.body_area.height * 0.70)
                .then_some(prev_idx + 1)
        })
}

/// native HWP5/HWPX의 2행 그림+caption RowBreak 표인지 판별한다.
///
/// 그림을 첫 행에, caption을 둘째 행에 저장한 표는 현재 페이지의 기존 각주 위에
/// 실제로 들어가도 일반 다행 표와 같은 clean-defer gate를 타기 쉽다. 일반 요구사항
/// 표와 rowspan 표의 이월 계약은 건드리지 않기 위해, 이 구조만 별도 near-fit 보정의
/// 후보로 식별한다.
fn is_two_row_picture_caption_rowbreak_table(table: &crate::model::table::Table) -> bool {
    if table.row_count != 2
        || table.col_count != 1
        || table.cells.len() != 2
        || table
            .cells
            .iter()
            .any(|cell| cell.col != 0 || cell.row > 1 || cell.row_span != 1 || cell.col_span != 1)
    {
        return false;
    }

    let has_picture = table.cells.iter().any(|cell| {
        cell.row == 0
            && cell.paragraphs.iter().any(|para| {
                para.controls
                    .iter()
                    .any(|control| matches!(control, Control::Picture(_)))
            })
    });
    let has_caption_text = table
        .cells
        .iter()
        .any(|cell| cell.row == 1 && cell.paragraphs.iter().any(para_has_visible_text));
    has_picture && has_caption_text
}

/// 빈 host의 저장 좌표를 그대로 쓸 수 있는 단일 그림 표인지 판별한다.
///
/// `TopAndBottom` 자체는 일반 CellBreak/RowBreak 데이터 표에도 사용된다. 선언 높이만
/// 예약하면 그 표의 실제 행·셀 조각을 건너뛰므로, raw page anchor 특례는 non-TAC 그림만
/// 담은 1×1 RowBreak 표로만 제한한다.
fn is_single_noninline_picture_table(table: &crate::model::table::Table) -> bool {
    if table.common.treat_as_char
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
    {
        return false;
    }

    let Some(cell) = table.cells.first() else {
        return false;
    };
    cell.row == 0
        && cell.col == 0
        && cell.row_span == 1
        && cell.col_span == 1
        && cell.paragraphs.len() == 1
        && cell.paragraphs.first().is_some_and(|cell_para| {
            cell_para.text.trim().is_empty()
                && cell_para.controls.len() == 1
                && matches!(
                    cell_para.controls.first(),
                    Some(Control::Picture(picture)) if !picture.common.treat_as_char
                )
        })
}

/// 일반 RowBreak 데이터 표가 raw page anchor 특례를 타지 않도록, 저장 좌표를
/// 신뢰할 수 있는 그림 표 구조만 통과시킨다.
///
/// 1×1 비-TAC 그림 표와 그림·caption을 행으로 분리한 2×1 표는 한컴이 empty host의
/// `LINE_SEG`에 물리 페이지 좌표를 남기는 형상이다. 후자의 내부 그림은 TAC여도
/// 표 자체가 비-TAC float이므로 허용한다. 그림이 아닌 단순 1×1 표는 별도의 선언
/// 높이 신뢰 판정을 통과한 native HWP5에서만 raw anchor를 허용한다.
// [#7203] `stored_ladder_leaves_object_room` 은 `renderer::stored_float_anchor` 가
// 정본이다 — 조판과 렌더가 같은 판정·같은 원점을 쓴다.
use crate::renderer::stored_float_anchor::{
    stored_ladder_leaves_object_room, stored_single_topbottom_top_px,
};

fn is_stored_anchor_picture_table(table: &crate::model::table::Table) -> bool {
    !table.common.treat_as_char
        && matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
        && (is_single_noninline_picture_table(table)
            || is_two_row_picture_caption_rowbreak_table(table))
}

/// 측정기와 렌더러가 풀어 쓰는 빈 1×1 래퍼 표의 실제 행 기하를 돌려준다.
///
/// 래퍼의 page-anchor·caption·PageItem 메타데이터는 원본 표에 남아 있어야 한다.
/// 반면 `MeasuredTable`과 행 컷(`advance_row_cut`)은 내부 표의 행을 기준으로
/// 동작한다. 두 경로가 다른 표를 참조하면 행 수가 1 대 N으로 갈라져, 보이지
/// 않는 래퍼 행 높이가 쪽 소비에 더해진다. `table_layout` 및
/// `HeightMeasurer::measure_table_impl`의 unwrap 조건과 의도적으로 동일하다.
///
/// [Issue #4326] fallback `Paginator`(`pagination/engine.rs`)도 같은 unwrap 여부를
/// `PageItem::PartialTable::row_cursor_is_nested`에 실어야 하므로 crate 내부에 공개한다.
pub(crate) fn row_geometry_table(
    table: &crate::model::table::Table,
) -> &crate::model::table::Table {
    let mut effective = table;
    loop {
        if effective.row_count != 1 || effective.col_count != 1 || effective.cells.len() != 1 {
            return effective;
        }
        let cell = &effective.cells[0];
        if cell.paragraphs.len() != 1 {
            return effective;
        }
        let para = &cell.paragraphs[0];
        let has_visible_text = para
            .text
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n');
        if has_visible_text {
            return effective;
        }
        let Some(nested) = para.controls.iter().find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        }) else {
            return effective;
        };
        effective = nested;
    }
}

/// 1×1 RowBreak 표의 실측 높이가 선언 객체 높이와 같은 범위인지 판별한다.
///
/// 그림 표가 아니더라도 이 형상은 한컴이 empty host의 raw vpos에 직접 배치할 수
/// 있다. 단, 실제 셀 내용이 선언 높이보다 크게 팽창한 표는 이 경로에서 행 fragment가
/// 사라지므로, 선언 높이의 1.5배 이내일 때만 허용한다.
fn is_single_rowbreak_table_with_trustworthy_declared_height(
    table: &crate::model::table::Table,
    effective_height: f64,
    dpi: f64,
) -> bool {
    if table.common.treat_as_char
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
    {
        return false;
    }

    let Some(cell) = table.cells.first() else {
        return false;
    };
    if cell.row != 0 || cell.col != 0 || cell.row_span != 1 || cell.col_span != 1 {
        return false;
    }

    let declared_height = hwpunit_to_px(table.common.height as i32, dpi).max(0.0);
    declared_height > 0.0
        && effective_height <= declared_height * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
}

/// RowBreak 표의 특정 행 셀 안에 저장된 vpos reset이 있는지 판별한다.
///
/// 표 행 경계가 아니라 셀 내부 줄에서 `양수 vpos → 0 이하`로 되감긴 경우는 해당
/// cell tail이 다음 물리 페이지에서 이어진다는 저장 frame 신호다. HWP와 HWPX의
/// 원본 `lineSeg` 모두에 같은 방식으로 적용된다.
fn rowbreak_row_has_internal_saved_vpos_reset(
    table: &crate::model::table::Table,
    row: usize,
) -> bool {
    table
        .cells
        .iter()
        .filter(|cell| cell.row as usize == row)
        .any(|cell| {
            // 저장된 물리 page reset은 cell paragraph 경계에서 시작할 수도 있다. 각
            // paragraph 안의 `windows(2)`만 보면 `<OPTN>` 다음 `간 특수 검사`처럼
            // p[n]의 마지막 LINE_SEG → p[n+1]의 첫 LINE_SEG reset을 놓친다.
            let mut previous_vpos = None;
            for para in &cell.paragraphs {
                for seg in para
                    .line_segs
                    .iter()
                    .filter(|seg| !is_synthetic_line_seg(seg))
                {
                    if previous_vpos.is_some_and(|previous| previous > 0 && seg.vertical_pos <= 0) {
                        return true;
                    }
                    previous_vpos = Some(seg.vertical_pos);
                }
            }
            false
        })
}

/// RowBreak 표 셀 안에 저장된 vpos reset이 있는지 판별한다.
fn rowbreak_table_has_internal_saved_vpos_reset(table: &crate::model::table::Table) -> bool {
    (0..table.row_count as usize).any(|row| rowbreak_row_has_internal_saved_vpos_reset(table, row))
}

/// Whether every text-bearing cell proves, through its stored line segments,
/// that its content belongs inside the declared cell box. This distinguishes a
/// renderer metric expansion from a source-owned row growth without a table
/// size ratio or a pixel cap. Rowspans and controls have independent physical
/// ownership, so they stay on the measured-row path.
fn table_declared_height_has_stored_cell_content_frame(
    table: &crate::model::table::Table,
    dpi: f64,
) -> bool {
    !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            if cell.row_span != 1 || cell.height >= 0x8000_0000 {
                return false;
            }
            let mut has_text = false;
            let mut stored_bottom = None;
            for para in &cell.paragraphs {
                if !para.controls.is_empty() {
                    return false;
                }
                let text = para.text.replace(|c: char| c.is_control(), "");
                if text.trim().is_empty() {
                    continue;
                }
                has_text = true;
                for seg in para
                    .line_segs
                    .iter()
                    .filter(|seg| !is_synthetic_line_seg(seg))
                    .filter(|seg| seg.vertical_pos >= 0 && seg.line_height > 0)
                {
                    let bottom =
                        hwpunit_to_px(seg.vertical_pos.saturating_add(seg.line_height), dpi);
                    stored_bottom =
                        Some(stored_bottom.map_or(bottom, |current: f64| current.max(bottom)));
                }
            }
            !has_text
                || stored_bottom
                    .is_some_and(|bottom| bottom <= hwpunit_to_px(cell.height as i32, dpi))
        })
}

/// A cell-local stored frame proves that the cell's text fits its own box, but
/// not that the table object's declared frame owns every row.  Some native HWP
/// RowBreak tables retain a stale object height while their individual cells
/// carry the full multi-row geometry.  Treating that short object as the whole
/// table collapses a source-owned first fragment into one floating table.
///
/// This checks the other half of the source contract without relying on a
/// measured/declared ratio: each non-rowspan row must have a declared cell box
/// and their geometry, including cell spacing, must fit inside the declared
/// table object frame.
fn table_declared_object_covers_cell_row_frames(
    table: &crate::model::table::Table,
    dpi: f64,
) -> bool {
    let row_count = table.row_count as usize;
    if row_count == 0 || table.cells.is_empty() {
        return false;
    }

    let declared_object_height = raw_table_ctrl_height_px(table, dpi)
        .unwrap_or_else(|| hwpunit_to_px(table.common.height as i32, dpi).max(0.0));
    if declared_object_height <= 0.0 {
        return false;
    }

    let mut row_heights: Vec<Option<f64>> = vec![None; row_count];
    for cell in &table.cells {
        let row = cell.row as usize;
        if row >= row_count || cell.row_span != 1 || cell.height >= 0x8000_0000 {
            return false;
        }
        let height = hwpunit_to_px(cell.height as i32, dpi);
        if height <= 0.0 {
            return false;
        }
        row_heights[row] = Some(row_heights[row].unwrap_or(0.0).max(height));
    }

    let declared_row_geometry =
        row_heights
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .map(|heights| {
                heights.iter().sum::<f64>()
                    + hwpunit_to_px(table.cell_spacing as i32, dpi)
                        * heights.len().saturating_sub(1) as f64
            });
    declared_row_geometry.is_some_and(|height| height <= declared_object_height + 0.5)
}

pub(crate) fn missing_lineseg_trailing_line_break(
    para: &Paragraph,
    line_count: usize,
    current_height: f64,
    available: f64,
    trailing_line_spacing: f64,
    source_uses_inline_field_reset: bool,
    hwp3_converted_missing_lineseg: bool,
) -> Option<usize> {
    crate::renderer::pagination::missing_lineseg_fragment_boundary(
        para,
        line_count,
        current_height,
        available,
        trailing_line_spacing,
        source_uses_inline_field_reset,
        hwp3_converted_missing_lineseg,
    )
}

fn is_synthetic_line_seg(ls: &LineSeg) -> bool {
    ls.tag & 0x80000000 != 0
}

/// [#6409] HWPX 가 글자처럼 취급 표를 쪽높이급 **한 줄**로 저장했으면, leftover
/// 에 행 분할로 끼우지 않고 다음 쪽 상단에서 시작한다.
///
/// 원본 XML 의 vertpos=0 은 typeset 전에 누적 vpos 로 덮인다(3249937 신고서
/// 표 0 → 524475). 남는 신호는 단일 LINE_SEG 높이(vertsize=69344 ≈ 본문 92%).
/// 붙임4 표도 한 줄이지만 잔여에 통째로 들어가(762px < leftover) 그대로 둔다.
fn hwpx_stored_tac_table_starts_at_page_top(
    para: &Paragraph,
    table: &crate::model::table::Table,
    current_items_empty: bool,
    current_height: f64,
    body_available: f64,
    dpi: f64,
) -> bool {
    if current_items_empty || current_height < 1.0 || !table.common.treat_as_char {
        return false;
    }
    let segs: Vec<&LineSeg> = para
        .line_segs
        .iter()
        .filter(|seg| !is_synthetic_line_seg(seg))
        .collect();
    let Some(seg) = segs.first() else {
        return false;
    };
    if segs.len() != 1 {
        return false;
    }
    let stored_h = hwpunit_to_px(seg.line_height, dpi);
    stored_h >= body_available * 0.5 && current_height + stored_h > body_available
}

/// [#5941 축 B] `#5921` 완화를 걸어도 되는 쪽인가 — **지금 쪽이 사실상 비어 있는가**.
///
/// 리셋을 지켰을 때 거의 빈 쪽이 남는 형상에서는 한/글도 쪼개지 않는다. 반대로 쪽이 이미
/// 차 있으면 저장 리셋이 이긴다(아래 `native_near_top_reset` 결합부 주석 참조).
const NEAR_TOP_RESET_EMPTY_PAGE_FILL_RATIO: f64 = 0.10;

fn near_top_reset_page_is_barely_started(current_height: f64, available_height: f64) -> bool {
    available_height > 0.0
        && current_height <= available_height * NEAR_TOP_RESET_EMPTY_PAGE_FILL_RATIO
}

/// [#5921] stored near-top 리셋이 이번 쪽 잔여를 넘는가.
///
/// `native_near_top_reset` 은 저장 vpos≈sb 만 보고 쪽을 가른다. 잔여에
/// 문단(sb+저장 줄 높이)이 들어가면 한글은 같은 쪽에 붙인다
/// (`neartop_reset_sb2500.hwpx`: 잔여 80px > 필요 63px, 한글 2020 1쪽).
/// 과적 케이스(148753276 pi46, used 942>933.6)는 잔여가 없어 리셋이 유지된다.
fn native_near_top_reset_exceeds_remaining(
    para: &Paragraph,
    para_sb_hu: i32,
    current_height: f64,
    available_height: f64,
    dpi: f64,
) -> bool {
    let remaining = (available_height - current_height).max(0.0);
    let real_lines: Vec<&LineSeg> = para
        .line_segs
        .iter()
        .filter(|ls| !is_synthetic_line_seg(ls))
        .collect();
    if real_lines.is_empty() {
        return true;
    }
    let sb_px = hwpunit_to_px(para_sb_hu.max(0), dpi);
    let lines_px: f64 = real_lines
        .iter()
        .map(|ls| hwpunit_to_px(ls.line_height.max(0), dpi))
        .sum();
    sb_px + lines_px > remaining + 0.5
}

/// 어울림(비 `treat_as_char`) 개체를 문단이 품고 있는가.
///
/// 어울림 밴드 옆으로 흐르는 줄은 앞 문단과 같은 세로 위치를 정당하게 다시 쓰므로,
/// 저장된 vpos 충돌을 쪽 경계로 읽으면 안 되는 예외다.
fn para_has_floating_object(para: &Paragraph) -> bool {
    para.controls.iter().any(|c| {
        matches!(
            c,
            Control::Shape(_) | Control::Table(_) | Control::Picture(_) | Control::Equation(_)
        ) && !c.is_treat_as_char_object()
    })
}

/// 앞뒤 문단이 **둘 다** 단 맨 위(stored vpos 0)를 주장하는가 (#5907).
///
/// Task #321 의 단일 단 트리거는 직전 문단의 마지막 줄이 쪽 하단부에 있을 때
/// (`pv > 5000`) 만 저장 리셋을 인정한다. 그런데 한/글은 쪽 하나에 짧은 문단
/// 하나만 올린 뒤 쪽을 넘기기도 하고, 그때 직전 문단의 vpos 는 0 이라 트리거가
/// 침묵한다 — 세 문단이 각각 한 쪽씩 차지하는 `samples/p122.hwp` 가 그 예다.
///
/// 이 규칙은 본문 텍스트의 일반적인 0-vpos 연속에는 적용하지 않는다. `p122`처럼
/// 양쪽 텍스트는 비어 있지만 구역/글자처럼 개체 컨트롤을 단독으로 가진 문단이 다시
/// 0 에서 시작하면, 그 컨트롤 앵커는 앞 쪽의 흐름과 같은 단에 함께 놓일 수 없다.
/// 넘침이 사유가 아니어서 rhwp 자체 흐름으로는 재현되지 않으므로 저장값을 그대로
/// 신뢰한다.
///
/// 오탐을 막기 위해 두 문단이 같은 단 기하(`column_start`/`segment_width`)를 쓰고,
/// 어울림 개체가 없으며, 양쪽 LINE_SEG 가 합성본이 아닌 경우로만 좁힌다.
fn stored_vpos_top_collision(prev: &Paragraph, curr: &Paragraph) -> bool {
    if para_has_floating_object(prev) || para_has_floating_object(curr) {
        return false;
    }
    // 일반 본문/생성 HWPX는 모든 문단의 LINE_SEG vpos를 0으로 저장할 수 있다. 그
    // 경우까지 쪽 경계로 읽으면 매 문단마다 새 쪽이 생긴다. p122의 증거는 양쪽 모두
    // 텍스트 없이 실제 컨트롤/빈 앵커를 단독으로 둔 경우이므로, 그 좁은 경우만
    // 인정한다. 컨트롤도 없는 trailing 빈 문단(#1663) 역시 저장 0이 남아 있을 뿐 쪽
    // 경계가 아니다.
    if para_has_visible_text(prev) || para_has_visible_text(curr) || prev.controls.is_empty() {
        return false;
    }
    let real = |ls: &&LineSeg| !is_synthetic_line_seg(ls);
    let (Some(prev_first), Some(prev_last), Some(curr_first)) = (
        prev.line_segs.iter().find(real),
        prev.line_segs.iter().rev().find(real),
        curr.line_segs.iter().find(real),
    ) else {
        return false;
    };

    // 저장된 조판에서 온 실제 줄 세그먼트여야 한다. 프로그램으로 만든 문단
    // (`LineSeg::default()` + line_height 만 채운 합성 IR)은 vpos·tag·segment_width 가
    // 모두 0 이라 "전부 단 맨 위" 로 보이므로, 그런 IR 에는 이 규칙을 적용하지 않는다.
    let parsed_seg = |ls: &LineSeg| {
        ls.tag & LineSeg::TAG_FIRST_SEGMENT != 0 && ls.segment_width > 0 && ls.line_height > 0
    };
    if !parsed_seg(prev_first) || !parsed_seg(prev_last) || !parsed_seg(curr_first) {
        return false;
    }

    // [#6087] 앞 문단의 저장 **전진이 0**(줄간격 0%: lh + ls ≤ 0)이면 쪽을
    // 점유하지 않으므로, 그 직후의 vpos=0 은 "다시 맨 위 주장"이 아니라 같은
    // 자리다 — 충돌 아님. 30307: pi=0(구역/단 정의, lh 1300 + ls −1300) 직후
    // pi=1 을 충돌로 읽어 완전한 빈 1쪽을 만들었다(한글 13쪽 vs 14쪽). p122
    // 증거(전진 1600/22838 문단들의 연쇄 단독 쪽)는 전진 > 0 이라 불변.
    if prev_last.line_height.saturating_add(prev_last.line_spacing) <= 0 {
        return false;
    }

    // 앞 문단이 통째로 단 맨 위 한 줄에 있었고(첫 줄·마지막 줄 모두 vpos 0),
    // 0 보다 아래에서 끝났는데 다음 문단이 다시 맨 위를 주장한다.
    prev_first.vertical_pos == 0
        && prev_last.vertical_pos == 0
        && curr_first.vertical_pos == 0
        && prev_last.column_start == curr_first.column_start
        && prev_last.segment_width == curr_first.segment_width
}

/// [#6342] 쪽을 거의 채운 TAC 자리차지 표 뒤의 짧은 붙임 두 줄은 잔여 칸에
/// 한 줄만 끼워 넣지 않고 다음 쪽으로 함께 넘긴다.
///
/// `36385445` 결재문서: 4×1 단 기준 TAC 표 899.5px / 본문 952.5px 뒤에 붙임
/// 28.8+28.8px 가 온다. 한글은 둘 다 2쪽에 둔다. 첫 줄만 잔여 53px 에 넣으면
/// used=964.3 으로 넘친다. 모든 원본 HWPX TAC 표에 열면 #3931 편람 쪽수와
/// #6044 상자 간격이 깨지므로, 4×1 단 기준·40px 미만 두 줄만 연다.
fn original_hwpx_tac_filled_page_keeps_short_trail(
    original_hwpx: bool,
    table: &crate::model::table::Table,
    table_height_px: f64,
    body_height_px: f64,
    remaining_px: f64,
    current_h: f64,
    next_h: f64,
) -> bool {
    use crate::model::shape::{HorzRelTo, TextWrap};
    original_hwpx
        && table.common.treat_as_char
        && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        && matches!(table.common.horz_rel_to, HorzRelTo::Column)
        && table.row_count == 4
        && table.col_count == 1
        && body_height_px > 0.0
        && table_height_px >= body_height_px * 0.90
        && current_h > 0.0
        && next_h > 0.0
        && current_h < 40.0
        && next_h < 40.0
        && remaining_px + 0.5 >= current_h
        && remaining_px + 0.5 < current_h + next_h
}

/// Returns only the measured-row slack that remains below a saved native
/// RowBreak first-fragment flow frame. Both bounds are absolute page flow
/// coordinates; host-before spacing and paint-only vertical insets are not
/// part of the stored flow frame.
fn saved_rowbreak_first_fragment_flow_overflow_allowance(
    declared_height: u32,
    outer_table_owns_row_geometry: bool,
    source_flow_bottom: f64,
    fragment_flow_bottom: f64,
) -> f64 {
    const SOURCE_FRAME_EPSILON_PX: f64 = 0.5;
    if declared_height == 0
        || declared_height > i32::MAX as u32
        || !outer_table_owns_row_geometry
        || !source_flow_bottom.is_finite()
        || !fragment_flow_bottom.is_finite()
        || source_flow_bottom <= 0.0
        || fragment_flow_bottom <= 0.0
        || source_flow_bottom > fragment_flow_bottom + SOURCE_FRAME_EPSILON_PX
    {
        return 0.0;
    }

    (fragment_flow_bottom - source_flow_bottom).max(0.0)
}

/// [#6123] 저장 프레임 바닥이 **저장 행 경계**와 같다고 볼 수 있는 허용치.
/// 저장 높이는 HWPUNIT→px 변환에서 행마다 반올림 오차를 남기므로 행 수에
/// 비례하는 몫을 더해 쓴다(호출부).
const SAVED_FRAME_ROW_END_STORED_TOLERANCE_PX: f64 = 1.0;

/// 저장된 첫 RowBreak 조각 높이에 가장 가까운 행 경계를 찾는다.
///
/// 저장 프레임의 남은 물리 공간은 글꼴 측정 drift를 흡수할 수 있지만, 다음 행까지
/// 허용하는 일반 여유값은 아니다. 동률이면 앞 경계를 택해 다음 행을 앞당겨
/// 소유하지 않는다.
fn nearest_saved_rowbreak_frame_row_end(
    frame_height: f64,
    row_heights: &[f64],
    stored_row_heights: &[f64],
    cell_spacing: f64,
) -> Option<usize> {
    if !frame_height.is_finite() || frame_height <= 0.0 {
        return None;
    }

    let mut bottom = 0.0;
    let mut nearest: Option<(usize, f64, f64)> = None;
    for (row, height) in row_heights.iter().enumerate() {
        if row > 0 {
            bottom += cell_spacing;
        }
        bottom += height;
        let distance = (bottom - frame_height).abs();
        if nearest.is_none_or(|(_, best, _)| distance < best) {
            nearest = Some((row + 1, distance, bottom));
        }
    }
    let (end_row, _, row_bottom) = nearest?;

    // [#6123] 프레임이 그 행 경계를 **닿지 못하면** 그 행을 소유하지 않는다.
    //
    // 최근접 스냅에는 거리 제한이 없어, 프레임 바닥이 어떤 행 한복판에 떨어져도
    // 그 행의 끝으로 끌려갔다 — 3112461 7쪽은 프레임 388.0px 이 행 1(측정
    // 36.0~573.1)의 65% 지점인데 행 끝(573.1)으로 스냅돼 그 행을 통째로 앞
    // 쪽에 얹었고, 표가 본문 하단을 174px 넘겼다. 한글은 그 행을 줄 단위로
    // 가른다.
    //
    // 프레임이 경계를 **넘어서는**(frame ≥ 누적) 경우는 종전 그대로다 — 21298295
    // 별표 5 는 프레임이 행 13 경계를 22.6px 지나며, 그 초과는 다음 행의 측정↔
    // 저장 drift 다. 반대로 **모자라는** 쪽은 그 행을 다 담지 못했다는 뜻이므로,
    // 흡수 가능한 drift(관련 행들의 |측정 − 저장| 합 + 행별 px 변환 반올림)
    // 안에서만 허용한다. 저장 높이를 못 읽는 행은 그 행의 측정 높이가 곧 상한이
    // 되어 종전 스냅이 그대로 남는다.
    let shortfall = row_bottom - frame_height;
    if shortfall <= 0.0 {
        return Some(end_row);
    }
    // 모자란 몫이 **조각이 될 수 없는 크기**(`MIN_TOP_KEEP_PX`)면 그 잔여를 다음
    // 쪽으로 옮길 수 없으므로 행을 통째로 두는 것이 맞다 — 1790387 PrEP 보고서는
    // 프레임이 행 3 경계에 16.8px 못 미친다. 그보다 크게 모자라면 흡수 가능한
    // drift(관련 행들의 |측정 − 저장| 합 + 행별 px 변환 반올림) 안에서만 허용한다.
    // 저장 높이를 못 읽는 행은 그 행의 측정 높이가 곧 상한이 되어 종전 스냅이
    // 그대로 남는다.
    let drift_budget: f64 = row_heights
        .iter()
        .take(end_row)
        .enumerate()
        .map(|(row, measured)| {
            let stored = stored_row_heights.get(row).copied().unwrap_or(0.0);
            if stored > 0.0 && stored.is_finite() {
                (measured - stored).abs()
            } else {
                *measured
            }
        })
        .sum::<f64>()
        + SAVED_FRAME_ROW_END_STORED_TOLERANCE_PX * end_row as f64;
    (shortfall <= drift_budget.max(MIN_TOP_KEEP_PX)).then_some(end_row)
}

/// Visible host text that is structurally a numbered table caption.
///
/// A positive table offset alone does not make the host a caption: ordinary
/// section headings such as `1. 편성기준` can share that geometry (#2097).
/// Restrict the native pre-emit path to the explicit `표 N`/`Table N` form
/// observed in Hancom's visible-host captions (policy tables 26--28).
fn visible_host_is_numbered_table_caption(para: &Paragraph) -> bool {
    let has_table_number_control = para.controls.iter().any(|control| {
        matches!(
            control,
            Control::AutoNumber(number)
                if number.number_type == crate::model::control::AutoNumberType::Table
        ) || matches!(
            control,
            Control::NewNumber(number)
                if number.number_type == crate::model::control::AutoNumberType::Table
        )
    });
    if has_table_number_control {
        return true;
    }

    let text = para.text.trim_start();
    let suffix = if let Some(rest) = text.strip_prefix('표') {
        rest
    } else if text
        .get(.."Table".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Table"))
    {
        &text["Table".len()..]
    } else {
        return false;
    };

    suffix
        .trim_start()
        .chars()
        .next()
        .is_some_and(char::is_numeric)
}

/// Native HWP5 can encode a table title as visible text in the table's host
/// paragraph instead of as `Table::caption`.  When the stored host line fits
/// wholly inside the table's positive paragraph-relative offset, Hancom paints
/// that line in the offset lane before the first RowBreak fragment.  Deferring
/// every visible host until the terminal fragment moves such titles to the
/// following page (policy report table 27, p90 -> p91).
///
/// Keep this narrower than the generic visible-host path: a single table,
/// stored (non-synthetic) line geometry, native HWP5, and an offset at least as
/// tall as the complete stored host span.  A shorter offset denotes a genuine
/// post-table host (#1686), and co-anchored table stacks have separate ordering
/// rules (#2439).
fn native_hwp5_rowbreak_host_precedes_first_fragment(
    para: &Paragraph,
    table: &crate::model::table::Table,
) -> bool {
    if !para_has_non_whitespace_text(para)
        || !visible_host_is_numbered_table_caption(para)
        || !is_para_topbottom_float(&table.common)
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return false;
    }

    let vertical_offset = signed_hwpunit(table.common.vertical_offset);
    if vertical_offset <= 0 {
        return false;
    }

    let mut stored = para
        .line_segs
        .iter()
        .filter(|seg| !is_synthetic_line_seg(seg) && seg.line_height > 0);
    let Some(first) = stored.next() else {
        return false;
    };
    let last = stored.next_back().unwrap_or(first);
    let stored_host_span = last
        .vertical_pos
        .saturating_sub(first.vertical_pos)
        .saturating_add(last.line_height)
        .max(0);

    stored_host_span > 0 && stored_host_span <= vertical_offset
}

/// [#2098] 쪽-하단 고정 틀(vert=쪽, valign=Bottom) 표의 빈 앵커 문단 — 저장 vpos=0 은
/// 쪽 기준 절대배치 산물이라 흐름 리셋(새 쪽) 신호가 아니다 (opengov 결재문서 계열
/// 36358528 pi8: 한글은 p1 하단 배치인데 리셋 오독으로 p2 단독 +1쪽). 앵커 배치는
/// page-bottom footer 경로가 배타영역 fit 으로 자체 판정하므로(침범 시 스스로 다음 쪽
/// 이동) 리셋 신호 제외가 과충전을 만들지 않는다.
fn para_is_page_bottom_fixed_table_anchor(para: &Paragraph) -> bool {
    !para_has_visible_text(para)
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(t) if is_page_bottom_fixed_float(&t.common)))
}

/// [#2137] 문단의 컨트롤이 전부 비-TAC 자리차지(TopAndBottom, vert=문단) 그림/도형
/// float 인가. 저장 page-last 증거가 있는 단일 줄 앵커는 개체를 하단 여백으로
/// 스필하는 것이 한컴 정합(156618554: 앵커 저장 경계 70000 ≤ 본문 70018HU, 한글
/// 1쪽 — 소형 글상자 49.8px 를 여백으로 흘림)이라 saved-bounds 신뢰(#2093)에
/// 편입한다. 대형 박스를 한컴이 다음 쪽으로 넘기는 케이스(#1027-E2, AI 184p)는
/// 앵커 저장 vpos 가 다음 쪽을 인코딩해 경계 fit 에서 자연 배제된다. 표 float 는
/// footer 로직 소관이라 제외.
fn para_controls_only_topbottom_floats(para: &Paragraph) -> bool {
    !para.controls.is_empty()
        && para.controls.iter().all(|c| match c {
            Control::Picture(p) => is_para_topbottom_float(&p.common),
            Control::Shape(s) => is_para_topbottom_float(s.common()),
            _ => false,
        })
}

/// [#2137] 단일 줄의 treat_as_char TopAndBottom 그림/도형만 가진 문단.
/// 한컴은 이런 소형 개체 줄을 쪽 하단 여백으로 스필해 현재 쪽에 유지한다
/// (156637323 pi=19 실측). 저장 page-last 증거와 결합해서만 신뢰한다 —
/// 대형 박스는 저장 vpos 가 다음 쪽을 인코딩해 bounds 검사에서 자연 배제
/// (#1027-E2 push 정합 유지).
fn para_controls_only_tac_topbottom_objects(para: &Paragraph) -> bool {
    use crate::model::shape::TextWrap;
    !para.controls.is_empty()
        && para.controls.iter().all(|c| match c {
            Control::Picture(p) => {
                p.common.treat_as_char && matches!(p.common.text_wrap, TextWrap::TopAndBottom)
            }
            Control::Shape(s) => {
                s.common().treat_as_char && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
            }
            _ => false,
        })
}

/// [#6535 잔여] 문단이 **쪽 기준 앵커**(`vert_rel_to == Page`) 자리차지 블록을 품는가.
///
/// 그런 블록의 저장 `vertical_pos` 는 흐름 좌표가 아니라 **절대배치의 산물**이라 대개
/// `0` 이다(`#6535`). 그 `0` 을 "한컴이 여기서 쪽을 끊었다"는 리셋 신호로 읽으면, 남은
/// 자리에 들어가는 표가 혼자 새 쪽으로 밀려 본문 없는 쪽이 생긴다.
///
/// 실측 `36399617_결재문서본문_외부강의신고 결재신청.hwpx` (한/글 1쪽, rhwp 2쪽):
///
/// ```text
/// pi=15 까지 cur_h=639.0 / 본문 990.2  → 잔여 351.2
/// pi=16  text="끝."  Table wrap=TopAndBottom vrel=Page tac=false  h=22234(296.5px)
///        line_segs=[(vpos 0, lh 1200)]
/// → cv==0 && pv=46278>5000 이 리셋으로 읽혀 flush_column, 2쪽의 used=0.0px
/// ```
fn para_hosts_page_anchored_block(para: &Paragraph) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    let is_page_anchored_block = |common: &crate::model::shape::CommonObjAttr| {
        !common.treat_as_char
            && matches!(common.vert_rel_to, VertRelTo::Page)
            && matches!(common.text_wrap, TextWrap::TopAndBottom)
    };
    para.controls.iter().any(|c| match c {
        Control::Table(t) => is_page_anchored_block(&t.common),
        Control::Picture(p) => is_page_anchored_block(&p.common),
        Control::Shape(sh) => is_page_anchored_block(sh.common()),
        _ => false,
    })
}

fn paragraph_saved_vpos_reset_starts_new_page_after(
    current_para: &Paragraph,
    next_para: &Paragraph,
    col_count: u16,
    is_hwp3_variant: bool,
) -> bool {
    if para_is_page_bottom_fixed_table_anchor(next_para) {
        return false;
    }

    let next_first_vpos = next_para.line_segs.first().map(|s| s.vertical_pos);
    let curr_last_vpos = current_para.line_segs.last().map(|s| s.vertical_pos);
    let multi_col = col_count > 1;
    let allowed_top_vpos = if is_hwp3_variant { 1500 } else { 0 };

    matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl))
        if (if multi_col { nv < cl } else { nv <= allowed_top_vpos }) && cl > 5000)
}

/// 저장 `vpos` 되돌아감을 쪽 경계로 인정하려면 쪽이 이만큼 차 있어야 한다 [#3837].
///
/// 쪽 중간에서 인정하면 한 문서 안에서 가드가 연쇄 발동해 쪽수가 늘어난다
/// (`156633519 산업활동동향`: pi 501개 이동 +3쪽). 이 조건 하나로 PI 코호트 회귀가
/// 2건에서 0건이 됐고, 50·75·90% 가 모두 같은 결과라 가장 조인 값을 쓴다.
/// [#6718] 저장 사다리가 지시한 쪽 경계를 "이 문단은 어차피 넘치지 않는다"로 걸러낼 때
/// 쓰는 반올림 여유. 27469 `pi=62` 는 7줄 합 246.4 vs 예산 247.2 로 **0.8px** 차이로
/// 들어가 사다리(@6)를 못 따랐다 — 그 0.8px 은 조판 차이가 아니라 계산 오차 규모다.
const LADDER_FIT_EPSILON_PX: f64 = 1.0;

const STORED_VPOS_REWIND_MIN_FILL: f64 = 0.90;

/// 저장 `vpos` 가 되돌아가는 자리 — 한글이 거기서 쪽을 끊었다는 신호다 [#3837].
///
/// 기존 [`paragraph_saved_vpos_reset_starts_new_page_after`] 는 단일 단에서 `nv == 0`
/// 만 인정하고, 그마저 fit 완화에만 쓰인다(#418 회피). 여기서는 되돌아감 자체를 본다.
///
/// 판별력 실측(r29 `PI_MISMATCH` n=1 코호트 66건): 어긋난 항목의 36% 가 되돌아감인데,
/// 같은 문서 **다른 쪽**의 마지막 항목은 1,134개 중 2개(0.2%)뿐이다 — 180배 농축.
fn stored_vpos_rewinds(prev_vpos: Option<i32>, para: &Paragraph) -> bool {
    let Some(nv) = para
        .line_segs
        .iter()
        .find(|s| !is_synthetic_line_seg(s))
        .map(|s| s.vertical_pos)
    else {
        return false;
    };
    // `cl > 5000` 은 기존 vpos-reset 가드와 같은 하한 — 쪽 상단 근처에서 시작한 항목을
    // 기준으로 삼으면 되돌아감이 아니라 정상 흐름을 끊는다.
    prev_vpos.is_some_and(|cl| nv < cl && cl > 5000)
}

/// 값이 있는 가장 가까운 앞 문단의 마지막 저장 `vpos` — 직전 문단이 비어 line_segs 가
/// 없을 수 있다.
fn preceding_stored_vpos(paragraphs: &[Paragraph], para_idx: usize) -> Option<i32> {
    paragraphs[..para_idx.min(paragraphs.len())]
        .iter()
        .rev()
        .find_map(|p| {
            p.line_segs
                .iter()
                .rev()
                .find(|s| !is_synthetic_line_seg(s))
                .map(|s| s.vertical_pos)
        })
}

/// 저장 줄이 쪽 하단까지 찬 직후 `vpos=0`으로 시작하는 문단은 다음 쪽 소유다.
///
/// 일반 되감김 판정은 직전 줄의 *시작* `vpos > 5000`을 요구한다. 거의 한 쪽 높이인
/// TAC 표처럼 줄 시작은 위쪽이지만 `line_height`가 본문 하단까지 닿는 경우에는 그
/// 조건으로 새 쪽 신호를 놓친다. 이 보조 판정은 atomic TAC의 하단 여백 spill 예외만
/// 제한하며, 저장 줄 하단이 본문의 85%에 못 미치거나 다음 줄이 정확히 0에서 시작하지
/// 않으면 기존 top-fit 동작을 유지한다.
fn stored_zero_vpos_after_near_full_line(
    paragraphs: &[Paragraph],
    para_idx: usize,
    body_height_hu: i32,
) -> bool {
    let current_starts_at_zero = paragraphs
        .get(para_idx)
        .and_then(|para| {
            para.line_segs
                .iter()
                .find(|seg| !is_synthetic_line_seg(seg))
        })
        .is_some_and(|seg| seg.vertical_pos == 0);
    if !current_starts_at_zero || body_height_hu <= 0 {
        return false;
    }

    paragraphs[..para_idx.min(paragraphs.len())]
        .iter()
        .rev()
        .find_map(|para| {
            para.line_segs
                .iter()
                .rev()
                .find(|seg| !is_synthetic_line_seg(seg))
        })
        .is_some_and(|seg| {
            let bottom = seg.vertical_pos.saturating_add(seg.line_height);
            bottom >= body_height_hu.saturating_mul(85) / 100
        })
}

/// Native HWP5의 저장된 다행 RowBreak 프레임을 현재 쪽 소유로 되감아도 되는
/// page-tail 서명인지 판정한다.
///
/// 순차 host-spacing 누적은 저장 top보다 소폭 앞설 수 있지만, 일반적인 과거
/// anchor까지 되감으면 표가 여러 쪽씩 압축된다. 실제 저장본에서 관측한 3.2px
/// drift와 5.1px body-tail 여유만 포괄하도록 양쪽 경계를 제한한다.
fn native_hwp5_saved_rowbreak_tail_frame_matches(
    source_top: f64,
    source_bottom: f64,
    current_height: f64,
    available: f64,
) -> bool {
    const MIN_HOST_SPACING_DRIFT_PX: f64 = 1.0;
    const MAX_HOST_SPACING_DRIFT_PX: f64 = 4.0;
    const MAX_BODY_TAIL_GAP_PX: f64 = 8.0;

    let host_spacing_drift = current_height - source_top;
    let body_tail_gap = available - source_bottom;
    host_spacing_drift >= MIN_HOST_SPACING_DRIFT_PX
        && host_spacing_drift <= MAX_HOST_SPACING_DRIFT_PX
        && body_tail_gap >= 0.0
        && body_tail_gap <= MAX_BODY_TAIL_GAP_PX
}

fn paragraph_forces_page_boundary_after(
    current_para: &Paragraph,
    next_para: &Paragraph,
    col_count: u16,
    is_hwp3_variant: bool,
) -> bool {
    matches!(
        next_para.column_type,
        ColumnBreakType::Page | ColumnBreakType::Section
    ) || paragraph_saved_vpos_reset_starts_new_page_after(
        current_para,
        next_para,
        col_count,
        is_hwp3_variant,
    )
}

/// Native HWP5 regulatory forms commonly encode a circled subheading, one empty
/// carrier line, and its explanatory RowBreak table as three independent
/// paragraphs.  The heading itself can fit in the remaining tail while the table
/// cannot retain even its minimum visible first fragment.  Hancom keeps that
/// heading with the table rather than leaving it alone at the physical page
/// bottom (76076 p55/p70).
///
/// This is intentionally a structural, not a generic heading, rule: it accepts
/// only a circled heading followed by exactly one empty line and a single
/// non-TAC 1x1 TopAndBottom RowBreak table.  Native HWP5 can omit LINE_SEG from
/// that carrier, so its semantic emptiness is the reliable source contract.
/// Normal section titles, explicit keep-with-next styles, and multi-cell tables
/// remain on their normal pagination paths.
fn native_hwp5_circled_rowbreak_table_heading_requires_fresh_page(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    dpi: f64,
) -> bool {
    if !st.profile.hwp5_stored_pagination_layout()
        || st.col_count != 1
        || st.current_items.is_empty()
        || !para.controls.is_empty()
        || fmt.line_heights.len() != 1
        || !para_has_visible_text(para)
        || !matches!(
            para.text.trim_start().chars().next(),
            Some('\u{2460}'..='\u{2473}')
        )
    {
        return false;
    }

    let Some(blank) = paragraphs.get(para_idx + 1) else {
        return false;
    };
    let Some(table_host) = paragraphs.get(para_idx + 2) else {
        return false;
    };
    if !blank.text.trim().is_empty()
        || !blank.controls.is_empty()
        || !table_host.text.trim().is_empty()
    {
        return false;
    }

    let Some(Control::Table(table)) = table_host.controls.first() else {
        return false;
    };
    if table_host.controls.len() != 1
        || table.common.treat_as_char
        || !matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::TopAndBottom
        )
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || table.common.height == 0
    {
        return false;
    }

    let blank_advance = blank
        .line_segs
        .first()
        .filter(|seg| !is_synthetic_line_seg(seg))
        .map(|seg| hwpunit_to_px(seg.line_height.saturating_add(seg.line_spacing), dpi))
        // A missing carrier LINE_SEG is a native HWP5 encoding variant.  Its
        // actual advance is never smaller than the RowBreak orphan minimum, so
        // use that lower bound only for this page-tail decision.
        .unwrap_or(MIN_TOP_KEEP_PX);
    let group_minimum = fmt.height_for_fit + blank_advance + MIN_TOP_KEEP_PX;
    let remaining = (st.available_height() - st.current_height).max(0.0);

    remaining + 0.5 >= fmt.height_for_fit
        && remaining + 0.5 < group_minimum
        && group_minimum <= st.base_available_height() + 0.5
}

/// HWP5-origin 문서는 모든 page ornament를 감추는 빈 PageHide marker 뒤에 같은 쪽의
/// 장식 host를 `Page` break로 한 번 더 기록할 수 있다. 첫 marker가 이미 새 physical
/// page를 열었으므로 host의 break까지 적용하면 빈 page가 materialize된다.
fn hwp5_origin_redundant_pagehide_break_marker(
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    hwpx_stored_layout: bool,
) -> bool {
    if para_idx < 2
        || para.column_type != ColumnBreakType::Page
        || !para.text.trim().is_empty()
        || para.controls.len() != 1
        || !matches!(para.controls.first(), Some(Control::PageHide(_)))
    {
        return false;
    }

    let prior_empty = &paragraphs[para_idx - 1];
    let section_marker = &paragraphs[para_idx - 2];
    let Some(next_para) = paragraphs.get(para_idx + 1) else {
        return false;
    };

    // Stored-layout HWPX section markers that combine a decorative group and
    // PageHide own the blank PageHide page immediately before a page-starting
    // non-inline table. That marker is not the redundant HWP5-origin marker
    // handled here.
    let hwpx_pagehide_blank_page_owner = hwpx_stored_layout
        && section_marker.controls.iter().any(|control| {
            matches!(
                control,
                Control::Shape(shape)
                    if matches!(
                        shape.as_ref(),
                        crate::model::shape::ShapeObject::Group(_)
                    )
            )
        })
        && next_para
            .controls
            .iter()
            .any(|control| matches!(control, Control::Table(table) if !table.common.treat_as_char));

    prior_empty.text.trim().is_empty()
        && prior_empty.controls.is_empty()
        && section_marker.column_type == ColumnBreakType::Section
        && section_marker
            .controls
            .iter()
            .any(|control| matches!(control, Control::PageHide(_)))
        && next_para.column_type == ColumnBreakType::Page
        && next_para
            .controls
            .iter()
            .any(|control| !matches!(control, Control::PageHide(_)))
        && !hwpx_pagehide_blank_page_owner
}

/// 빈 ColumnBreak가 두 non-inline 표 사이에 있고 다음 표가 이미 PageBreak를
/// 소유하면, ColumnBreak는 별도 physical page가 아니라 다음 표의 carrier다.
/// 표의 shape, 크기, 저장 vpos가 아니라 형제 paragraph의 break 소유권만 쓴다.
fn empty_table_carrier_column_break_before_page_table(
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
) -> bool {
    if para_idx == 0
        || para.column_type != ColumnBreakType::Column
        || !para.text.trim().is_empty()
        || !para.controls.is_empty()
    {
        return false;
    }

    let previous = &paragraphs[para_idx - 1];
    let Some(next) = paragraphs.get(para_idx + 1) else {
        return false;
    };

    matches!(previous.controls.as_slice(), [Control::Table(table)]
        if !table.common.treat_as_char)
        && next.column_type == ColumnBreakType::Page
        && matches!(next.controls.as_slice(), [Control::Table(table)]
            if !table.common.treat_as_char)
}

fn single_line_visible_bounds_px(
    para: &Paragraph,
    page_vpos_base: i32,
    dpi: f64,
) -> Option<(f64, f64)> {
    let mut real_lines = para
        .line_segs
        .iter()
        .filter(|ls| !is_synthetic_line_seg(ls));
    let line = real_lines.next()?;
    if real_lines.next().is_some() {
        return None;
    }

    line_seg_visible_bounds_px(line, page_vpos_base, dpi)
}

fn line_seg_visible_bounds_px(seg: &LineSeg, page_vpos_base: i32, dpi: f64) -> Option<(f64, f64)> {
    let top = seg.vertical_pos.saturating_sub(page_vpos_base);
    let bottom = seg
        .vertical_pos
        .saturating_add(seg.line_height)
        .saturating_sub(page_vpos_base);
    (top >= 0 && bottom >= 0).then(|| (hwpunit_to_px(top, dpi), hwpunit_to_px(bottom, dpi)))
}

const SAVED_TAIL_FIT_CHAIN_CAP: usize = 2;

#[derive(Debug, PartialEq, Eq)]
enum SavedTailFitChainDecision {
    NoChange,
    Advance,
    Break,
}

fn saved_tail_fit_chain_decision(
    tail_fit_chain: usize,
    saved_tail_vpos_fit: bool,
    hwp_authoritative: bool,
    native_hwp5_reset_tail_fits_actual_footnote_boundary: bool,
) -> SavedTailFitChainDecision {
    if !saved_tail_vpos_fit
        || hwp_authoritative
        || native_hwp5_reset_tail_fits_actual_footnote_boundary
    {
        SavedTailFitChainDecision::NoChange
    } else if tail_fit_chain >= SAVED_TAIL_FIT_CHAIN_CAP {
        SavedTailFitChainDecision::Break
    } else {
        SavedTailFitChainDecision::Advance
    }
}

const SAVED_LINE_FLOW_ANCHOR_TOLERANCE_PX: f64 = 16.0;
/// [#5822] 누적 흐름이 저장 object frame 안으로 앞서 들어갔을 때 그 frame 을
/// 물리 page owner 로 신뢰하는 최대 드리프트. 실측 42.2px(156634833 p6)를
/// 덮고, frame 깊숙이 지나간 stale anchor(수백 px 뒤처짐)는 기각한다.
const SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX: f64 = 64.0;

/// [#5941] "흐름이 body 바닥에 앉았다"고 볼 여유. 이보다 적게 남았으면 저장 tail 의
/// 쪽 배정은 일반 fit 의 소관이다(`task1725` 242쪽 핀: 여유 0.0).
const BODY_BOTTOM_SEAT_PX: f64 = 10.0;

/// [#2097→#5714] 표를 완결하는 마지막 행의 쪽 하단 압축 수용치 — 한글은 쪽
/// 경계에서 말미 행이 잔여를 이 이내로 초과하면 행 밴드를 잔여로 압축해 쪽을
/// 완결한다 (1741000 실측 초과 6.8px 수용, 삭제 전 #2097 상수 그대로).
const TERMINAL_ROW_BOTTOM_SQUEEZE_TOLERANCE_PX: f64 = 13.0;
/// [#2097→#5714] 압축 수용은 쪽 끝자락(잔여 ≤ 이 값)에서만 — 한글의 압축은 쪽
/// 마무리 동작이다 (1741000 잔여 73.5px 압축 vs kps-ai 잔여 237.3px 이월 실측).
const TERMINAL_ROW_BOTTOM_SQUEEZE_MAX_REST_PX: f64 = 100.0;
/// [#2097→#5714] 압축 수용에 필요한 콘텐츠 여유(잔여-콘텐츠) 하한 — 콘텐츠가
/// 눌릴 공간이 없으면 한글도 이월한다 (1741000 여유 25.9px 압축 실측).
const TERMINAL_ROW_BOTTOM_SQUEEZE_MIN_HEADROOM_PX: f64 = 12.0;

fn saved_bounds_overlap_current_flow(bounds: (f64, f64), current_height: f64) -> bool {
    let (top, bottom) = bounds;
    let line_height = (bottom - top).max(0.0);
    line_height > 0.0 && top <= current_height + line_height && current_height <= bottom
}

fn saved_line_is_anchored_to_current_flow(bounds: (f64, f64), current_height: f64) -> bool {
    let (top, bottom) = bounds;
    bottom > top
        && top <= current_height
        && current_height <= top + SAVED_LINE_FLOW_ANCHOR_TOLERANCE_PX
}

fn saved_line_clears_footnote_area(
    current_footnote_height: f64,
    is_single_column: bool,
    overflow: f64,
    footnote_safety_margin: f64,
    saved_bounds: Option<(f64, f64)>,
    base_available_height: f64,
    current_height: f64,
) -> bool {
    current_footnote_height > 0.0
        && is_single_column
        && overflow <= footnote_safety_margin
        && saved_bounds.is_some_and(|(top, bottom)| {
            let text_limit = base_available_height - current_footnote_height;
            top >= 0.0
                && top <= base_available_height
                && bottom <= text_limit
                && saved_line_is_anchored_to_current_flow((top, bottom), current_height)
        })
}

fn saved_bounds_fit_at_flow_tail(
    bounds: (f64, f64),
    current_height: f64,
    available: f64,
    bottom_spill: f64,
) -> bool {
    saved_bounds_overlap_current_flow(bounds, current_height)
        && bounds.1 <= available + bottom_spill
}

fn saved_table_bounds_fit_at_flow_tail(
    bounds: (f64, f64),
    current_height: f64,
    available: f64,
    table_height: f64,
) -> bool {
    let (top, _) = bounds;
    // 저장 object frame이 현재 누적 흐름보다 조금 앞에서 시작해도, 그 line이 현재
    // 흐름과 겹치고 object bottom이 같은 body 안에 있으면 그 frame이 물리 page
    // owner다. 반대로 현재 흐름보다 뒤처진 쪽 상단 frame은 stale anchor이므로
    // 일반 fit 경로에 맡긴다.
    let source_frame_leads_current_flow =
        current_height <= top && saved_bounds_overlap_current_flow(bounds, current_height);
    // [#5822] 흐름이 저장 frame **안**에 막 들어온 경우(top ≤ 흐름 ≤ top+드리프트
    // 허용, 흐름 ≤ bottom)도 그 frame 이 물리 page owner 다 — 누적 드리프트로
    // 흐름이 사다리보다 수십 px 앞서 달려도, 한글이 기록한 frame 이 body 에
    // 들어가면 표는 그 쪽에 앉는다 (156634833 p6 차트 표: 저장 748.2..929.1 ≤
    // body 933.6 인데 흐름 790.4(드리프트 +42.2px) 기준 적합검사가 밀어 40쪽 vs
    // 한글 39쪽 — 이하 절 전체가 한 쪽씩 밀렸다). 흐름이 frame 깊숙이(허용 초과)
    // 지나간 쪽 상단 stale anchor 는 여전히 일반 fit 경로에 맡긴다.
    //
    // [#5941 3240179] `top == 0` 은 frame 위치 증거가 아니라 "쪽 시작" vpos
    // 센티널이다 — 그 값에 드리프트 허용을 적용하면 흐름이 42~48px 내려간
    // 상태에서도 쪽-말미 크기 표(913~918px)가 현재 쪽에 강제로 앉아 문서가
    // 3쪽 압축된다(16→13, 한글 18). 실제 frame(top>0, 원 케이스 748.2)만
    // 이 갈래의 대상이다. top==0 + 흐름 0 은 기존 anchored 갈래가 그대로 담당.
    let current_flow_inside_source_frame = top > 0.0
        && top <= current_height
        && current_height <= top + SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX
        && current_height <= bounds.1;
    table_height.is_finite()
        && table_height > 0.0
        && (saved_line_is_anchored_to_current_flow(bounds, current_height)
            || source_frame_leads_current_flow
            || current_flow_inside_source_frame)
        && bounds.1 <= available
        && bounds.0 + table_height <= available
}

/// 저장 LineSeg에 앵커된 TAC 표는 `common.height`가 한컴이 기록한 물리 object
/// frame이다. 행 측정 총합에는 host의 outer margin이 더해질 수 있으므로, 저장 frame
/// fit을 판정할 때는 선언 frame을 우선하고 선언값이 없을 때만 측정 높이를 사용한다.
/// [#5699 H1] 저장 사다리가 자리차지(TAC) 표의 밴드 높이를 계상하지 않은
/// 자기모순 판별.
///
/// 저장 th 관례 기반 계상(`charged_px`)이 **선언 표높이와 실측 둘 다**의 1/4
/// 미만이고, 선언과 실측이 서로 정합(2배 이내)일 때만 참이다. 선언·실측이
/// 갈라지는 문서(#2237 측정-저장 발산, #2148 라벨 셀)는 정합 조건에서 빠져
/// 종전 동작 불변. 이 서명의 문서(자치법규/기계생성 균일 사다리)는 한글 2022 가
/// 저장 사다리를 버리고 재조판한다 — 영월군 20099369 오라클 실측: 한글 4쪽
/// 정상 배치 vs 저장 사다리 추종 2쪽 전면 겹침(#5699 H1 코호트 40문서).
pub(crate) fn stored_ladder_omits_tac_band(
    charged_px: f64,
    declared_px: f64,
    measured_px: f64,
) -> bool {
    declared_px > 1.0
        && measured_px > 1.0
        && declared_px.max(measured_px) < declared_px.min(measured_px) * 2.0
        && charged_px < declared_px.min(measured_px) * 0.25
}

fn stored_tac_table_frame_height(
    table: &crate::model::table::Table,
    dpi: f64,
    measured_height: f64,
) -> f64 {
    let declared_height = hwpunit_to_px(table.common.height.min(i32::MAX as u32) as i32, dpi);
    if declared_height.is_finite() && declared_height > 0.0 {
        declared_height
    } else {
        measured_height
    }
}

#[cfg(test)]
mod saved_tac_table_flow_tail_contract {
    use super::saved_table_bounds_fit_at_flow_tail;

    #[test]
    fn requires_the_table_object_not_just_its_anchor_line_to_fit() {
        assert!(saved_table_bounds_fit_at_flow_tail(
            (850.0, 870.0),
            858.0,
            971.0,
            100.0,
        ));
        assert!(
            !saved_table_bounds_fit_at_flow_tail((850.0, 870.0), 858.0, 971.0, 889.0,),
            "a tail anchor line cannot authorize a table that extends into the next page"
        );
    }

    #[test]
    fn accepts_a_forward_source_frame_that_overlaps_the_current_flow() {
        assert!(saved_table_bounds_fit_at_flow_tail(
            (120.0, 500.0),
            100.0,
            700.0,
            350.0,
        ));
        assert!(!saved_table_bounds_fit_at_flow_tail(
            (0.0, 500.0),
            400.0,
            700.0,
            350.0,
        ));
    }
}

fn paragraph_saved_visible_bounds(
    para: &Paragraph,
    page_vpos_base: i32,
    dpi: f64,
) -> Option<(f64, f64)> {
    let mut bounds: Option<(f64, f64)> = None;
    for seg in para
        .line_segs
        .iter()
        .filter(|seg| !is_synthetic_line_seg(seg))
    {
        let (top, bottom) = line_seg_visible_bounds_px(seg, page_vpos_base, dpi)?;
        bounds = Some(match bounds {
            Some((min_top, max_bottom)) => (min_top.min(top), max_bottom.max(bottom)),
            None => (top, bottom),
        });
    }
    bounds
}

fn saved_line_range_fits_body_tail(
    para: &Paragraph,
    start_line: usize,
    end_line: usize,
    body_height_px: f64,
    dpi: f64,
) -> bool {
    if start_line >= end_line || end_line > para.line_segs.len() {
        return false;
    }

    let mut prev_vpos: Option<i32> = None;
    for seg in &para.line_segs[start_line..end_line] {
        if is_synthetic_line_seg(seg) || seg.vertical_pos <= 0 {
            return false;
        }
        if prev_vpos.is_some_and(|prev| seg.vertical_pos < prev) {
            return false;
        }
        let bottom_px = hwpunit_to_px(seg.vertical_pos.saturating_add(seg.line_height), dpi);
        if bottom_px > body_height_px + 0.5 {
            return false;
        }
        prev_vpos = Some(seg.vertical_pos);
    }

    true
}

/// HWPX의 문단 내부 `vpos=0` reset은, reset 직전 fragment가 현재 flow 앵커에서
/// 시작할 때에만 다음 물리 쪽의 시작을 뜻한다. 이 경우 reset 전 줄들은 저장된
/// 현재 쪽 fragment의 owner이므로, 일반 줄 높이 예산만으로 중간 쪽으로 분리하지
/// 않는다. 표·개체·다단·local cursor rewind는 이 계약 밖에 둔다.
fn hwpx_saved_reset_fragment_matches_current_flow(
    st: &TypesetState,
    para: &Paragraph,
    start_line: usize,
    break_line: usize,
    current_page_vpos_base: i32,
    dpi: f64,
) -> bool {
    paragraph::scan::hwpx_saved_reset_fragment_matches_current_flow(
        &st.paragraph_line_scan_page(),
        para,
        start_line,
        break_line,
        current_page_vpos_base,
        dpi,
    )
}

fn paragraph_text_looks_like_list_continuation_tail(para: &Paragraph) -> bool {
    let text = para.text.trim_start();
    text.starts_with('.') || text.starts_with('-') || text.starts_with('·') || text.starts_with('•')
}

/// [Task #1749] 저장 flow 가 이 문단을 "페이지 마지막 줄"로 인코딩했는가.
///
/// saved bounds 신뢰(`saved_single_line_bottom_fits`)는 저장 LINE_SEG vpos 가 페이지
/// 배정을 인코딩한다는 전제 위에 있다. 페이지-마지막 증거는 세 가지다:
/// ① 다음 문단의 첫 실줄이 없음(문서/구역 끝 — fe6de3ef 합성 테스트 보호 케이스),
/// ② 다음 실줄이 현재 줄보다 작은 vpos 로 리셋(다음 줄이 새 쪽),
/// ③ 다음 실줄에 도달하기 전에 명시적 쪽/구역나누기 문단이 있음 — 누적좌표 문서는
///    쪽 경계에서도 vpos 가 리셋되지 않으므로(예: 결재문서 36375752 pi26→pi27
///    [쪽나누기]) 리셋 검사만으로는 이 증거를 볼 수 없다. 신뢰 경로는 vpos 를
///    페이지 기준점 상대값으로 쓰므로 누적좌표라도 페이지 내 위치는 유효하다.
/// 어느 증거도 없이 다음 vpos 가 증가 지속하면(예: 결재문서 36371084 pi18→pi19)
/// 저장 vpos 로 페이지를 알 수 없으므로 신뢰하지 않는다 — 누적높이 판정으로 복귀해
/// 쪽 경계 overfill 을 막는다.
fn saved_flow_marks_page_last(paragraphs: &[Paragraph], para_idx: usize) -> bool {
    let curr_vpos = match paragraphs
        .get(para_idx)
        .and_then(|p| p.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)))
    {
        Some(ls) => ls.vertical_pos,
        None => return false,
    };
    for next_para in paragraphs.iter().skip(para_idx + 1) {
        if matches!(
            next_para.column_type,
            ColumnBreakType::Page | ColumnBreakType::Section
        ) {
            return true;
        }
        if let Some(next) = next_para
            .line_segs
            .iter()
            .find(|ls| !is_synthetic_line_seg(ls))
        {
            return next.vertical_pos < curr_vpos;
        }
    }
    true
}

fn positive_vpos_end_before_negative_wrap(para: &Paragraph) -> Option<i32> {
    let last_real = para
        .line_segs
        .iter()
        .rev()
        .find(|ls| !is_synthetic_line_seg(ls))?;
    if last_real.vertical_pos >= 0 {
        return None;
    }

    para.line_segs
        .iter()
        .filter(|ls| !is_synthetic_line_seg(ls) && ls.vertical_pos > 0)
        .map(|ls| ls.vertical_pos.saturating_add(ls.line_height))
        .max()
}

fn para_near_rowbreak_table(paragraphs: &[Paragraph], para_idx: usize) -> bool {
    let start = para_idx.saturating_sub(1);
    let end = (para_idx + 3).min(paragraphs.len());
    paragraphs[start..end].iter().any(|para| {
        para.controls.iter().any(|control| {
            matches!(
                control,
                Control::Table(table)
                    if matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
            )
        })
    })
}

/// #1672 행정업무 편람 계열: raw TABLE attr 상위 바이트가 비어 있는 RowBreak 표는
/// 기존 4px 안전마진만으로도 페이지가 누적 과분할된다.
fn section_has_zero_high_attr_rowbreak_table(paragraphs: &[Paragraph]) -> bool {
    paragraphs.iter().any(|para| {
        para.controls.iter().any(|control| {
            matches!(
                control,
                Control::Table(table)
                    if matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    ) && (table.raw_table_record_attr & 0xff00_0000) == 0
            )
        })
    })
}

impl TypesetState {
    fn new(
        layout: PageLayoutInfo,
        col_count: u16,
        section_index: usize,
        footnote_separator_overhead: f64,
        footnote_between_notes_margin: f64,
        footnote_safety_margin: f64,
        column_type: ColumnType,
    ) -> Self {
        Self {
            pages: Vec::new(),
            current_items: Vec::new(),
            current_height: 0.0,
            current_start_height: 0.0,
            current_endnote_flow: false,
            column_had_compact_endnote_rewind: false,
            prev_body_bottom_vpos: None,
            flow_underrun: 0.0,
            hangul2024_reclaimed: 0.0,
            hangul2024_spill_para: None,
            stored_ladder_spacing_omitted: false,
            omit_fresh_recalc_doc: false,
            ladder_band_floor: 0.0,
            current_ladder_band_tables: Vec::new(),
            current_column: 0,
            col_count,
            layout,
            section_index,
            square_band_bottom: 0.0,
            square_band_top: None,
            current_footnote_height: 0.0,
            current_bottom_fixed_exclusion: 0.0,
            bottom_fixed_consumed_flow: 0.0,
            page_has_page_abs_top_table: false,
            defer_host_line_item_para: None,
            is_first_footnote_on_page: true,
            current_page_has_footnote_separator: false,
            footnote_separator_overhead,
            footnote_between_notes_margin,
            footnote_safety_margin,
            section_has_no_footer: false,
            current_zone_y_offset: 0.0,
            current_zone_layout: None,
            on_first_multicolumn_page: false,
            pending_body_wide_top_reserve: 0.0,
            visible_float_exclusions: Vec::new(),
            side_wrap_exclusions: std::collections::BTreeMap::new(),
            inline_placements: std::collections::HashMap::new(),
            inline_flow_plans: std::collections::HashMap::new(),
            paragraph_float_placements: std::collections::HashMap::new(),
            inline_box_flow_bottom: 0.0,
            deferred_table_controls: Vec::new(),
            deferred_next_page_square_pictures: Vec::new(),
            page_start_square_pictures: Vec::new(),
            fragment_queued_table_footnotes: std::collections::HashSet::new(),
            reset_vpos_after_queued_table_footnote_page: false,
            prefilled_paras: std::collections::HashSet::new(),
            pre_emitted_host_paras: std::collections::HashSet::new(),
            pre_emitted_host_heights: std::collections::HashMap::new(),
            skip_safety_margin_once: false,
            skip_footnote_margin_once: false,
            tail_saved_bounds_once: None,
            strict_plain_text_fit_after_empty_host_float_once: false,
            profile: Default::default(),
            has_stored_line_segs: false,
            hide_empty_line: false,
            hidden_empty_lines: 0,
            hidden_empty_page_idx: usize::MAX,
            hidden_empty_paras: std::collections::HashSet::new(),
            page_tail_spilled_floats: std::collections::HashSet::new(),
            endnotes: Vec::new(),
            endnote_paragraphs: Vec::new(),
            endnote_para_sources: Vec::new(),
            endnote_between_notes_hu: 0,
            endnote_separator_above_hu: 0,
            endnote_separator_below_hu: 0,
            wrap_around_cs: -1,
            wrap_synth_rects: Vec::new(),
            wrap_around_sw: -1,
            wrap_around_table_para: 0,
            wrap_around_any_seg: false,
            wrap_around_derived_band: false,
            behind_float_table_para: None,
            next_para_first_stored_vpos: None,
            next_para_is_empty_float_table_anchor: false,
            next_para_is_plain_text: false,
            behind_pending_absorbs: Vec::new(),
            overlay_shape_shortcut_para: None,
            pending_overlay_continuations: Vec::new(),
            current_column_overlay_continuations: Vec::new(),
            current_column_overlay_cuts: Vec::new(),
            current_column_wrap_around_paras: Vec::new(),
            current_column_wrap_anchors: std::collections::HashMap::new(),
            current_zone_column_type: column_type,
            current_zone_design_spacing_px: 0.0,
            vpos_page_base: None,
            vpos_lazy_base: None,
            vpos_page_base_stored: false,
            vpos_ladder_dirty: false,
            vpos_prev_layout_para: None,
            vpos_prev_partial_table: false,
            vpos_col_anchor: 0.0,
            skip_spacing_before_prededuct: false,
            vpos_prev_trimmed_sb_px: 0.0,
        }
    }

    /// [#2424] page-flow state를 continuation context로 이동할 때 잠시 남겨둘 빈 자리.
    /// context drain 직후 원래 state로 교체되며 placeholder 자체는 조판에 사용되지 않는다.
    fn transfer_placeholder(&self) -> Self {
        Self::new(
            self.layout.clone(),
            self.col_count,
            self.section_index,
            self.footnote_separator_overhead,
            self.footnote_between_notes_margin,
            self.footnote_safety_margin,
            self.current_zone_column_type,
        )
    }

    /// [Task #1027 Stage D] 컬럼 경계에서 vpos 스냅 상태 초기화.
    /// 렌더러 build_single_column 진입 정합: page/lazy base·prev 초기화,
    /// anchor 를 현 current_height(컬럼 시작값)로 설정.
    fn reset_vpos_cursor(&mut self) {
        self.vpos_prev_trimmed_sb_px = 0.0;
        self.vpos_page_base = None;
        self.vpos_lazy_base = None;
        self.vpos_page_base_stored = false;
        self.vpos_ladder_dirty = false;
        self.vpos_prev_layout_para = None;
        self.vpos_prev_partial_table = false;
        self.vpos_col_anchor = self.current_height;
    }

    /// 사용 가능한 본문 높이 (각주, 존 오프셋 차감)
    fn available_height(&self) -> f64 {
        let base = self.base_available_height();
        let fn_margin = if self.current_footnote_height > 0.0 {
            self.footnote_safety_margin
        } else {
            0.0
        };
        // [#2559] 한글은 꼬리말 콘텐츠가 없는 구역에서 각주가 빈 꼬리말 밴드까지
        // 사용하도록 배치한다. 밴드를 초과하는 각주 높이만 본문을 줄여야 조기
        // 개행이 누적되지 않는다. 실제 꼬리말이 있으면 점유 가능성이 있으므로
        // 보수적으로 밴드를 회수하지 않는다.
        let footnote_penalty = (self.current_footnote_height - self.footer_band_reclaim()).max(0.0);
        // [#3707 진단] 가용 높이의 차감 내역. 두 문서에서 avail 이 21.4px 다른데
        // 본문 영역은 같으므로, 어느 항목이 그 차이를 만드는지 가른다. 동작 불변.
        if std::env::var("RHWP_DIAG_AVAIL").is_ok() {
            eprintln!(
                "DIAG_AVAIL base={:.1} fn_h={:.1} reclaim={:.1} penalty={:.1} fn_margin={:.1} zone_y={:.1} bottom_fixed={:.1} → {:.1}",
                base,
                self.current_footnote_height,
                self.footer_band_reclaim(),
                footnote_penalty,
                fn_margin,
                self.current_zone_y_offset,
                self.current_bottom_fixed_exclusion,
                (base - footnote_penalty - fn_margin - self.current_zone_y_offset
                    - self.current_bottom_fixed_exclusion)
                    .max(0.0),
            );
        }
        (base
            - footnote_penalty
            - fn_margin
            - self.current_zone_y_offset
            - self.current_bottom_fixed_exclusion)
            .max(0.0)
    }

    /// 기본 가용 높이 (각주/존 미차감)
    fn base_available_height(&self) -> f64 {
        self.layout.available_body_height()
    }

    /// 동일 표의 terminal fragment가 아직 current column에 있는지, 직전에 flush된
    /// page에 있는지 구분한다. `Some(true)`는 current, `Some(false)`는 flushed다.
    fn native_table_host_terminal_fragment_placement(
        &self,
        para_index: usize,
        control_index: usize,
        row_count: u16,
    ) -> Option<bool> {
        let is_terminal = |item: &PageItem| match item {
            PageItem::Table {
                para_index: item_para_index,
                control_index: item_control_index,
            } => *item_para_index == para_index && *item_control_index == control_index,
            PageItem::PartialTable {
                para_index: item_para_index,
                control_index: item_control_index,
                end_row,
                end_cut,
                ..
            } => {
                *item_para_index == para_index
                    && *item_control_index == control_index
                    && *end_row == usize::from(row_count)
                    && end_cut.is_empty()
            }
            _ => false,
        };

        if self.current_items.iter().any(&is_terminal) {
            return Some(true);
        }
        self.pages
            .iter()
            .rev()
            .flat_map(|page| page.column_contents.iter().rev())
            .flat_map(|column| column.items.iter().rev())
            .any(is_terminal)
            .then_some(false)
    }

    /// [#4090] Square 어울림 밴드 종료 — 흐름을 밴드 바닥으로 스냅한다.
    ///
    /// 한글은 어울림 개체 옆으로 글을 흘리되 **개체 바닥까지만** 흘리고 그 아래는
    /// 전폭으로 복귀한다. 밴드 안에서는 흐름이 글줄만큼만 전진하므로, 밴드를 벗어날
    /// 때(또는 쪽이 끝날 때) 개체 높이를 반영해 끌어올려야 후속 내용이 개체 안으로
    /// 들어가지 않는다.
    fn close_square_band(&mut self) {
        if self.square_band_bottom > 0.0 {
            self.current_height = self.current_height.max(self.square_band_bottom);
            self.square_band_bottom = 0.0;
            self.square_band_top = None;
        }
    }

    /// 실제로 현재 단에 방출한 그림만 등록한다. 미래/다른 쪽의 그림은 예약하지 않는다.
    fn register_side_wrap_picture(
        &mut self,
        para_index: usize,
        control_index: usize,
        para: &Paragraph,
        paragraph_top: Option<f64>,
        styles: &ResolvedStyleSet,
    ) {
        let Some(Control::Picture(picture)) = para.controls.get(control_index) else {
            return;
        };
        if picture.common.vert_rel_to == crate::model::shape::VertRelTo::Para
            && paragraph_top.is_none()
        {
            // 배치 소유자가 아직 문단 원점을 전달하지 않은 경로는 원시 vpos로 추정하지 않는다.
            return;
        }
        if !self.current_items.iter().any(|item| {
            matches!(item, PageItem::Shape { para_index: pi, control_index: ci }
            if *pi == para_index && *ci == control_index)
        }) {
            return;
        }
        let column = self.inline_flow_column();
        let style = styles.para_styles.get(para.para_shape_id as usize);
        let left = style.map_or(0.0, |s| s.margin_left);
        let right = style.map_or(0.0, |s| s.margin_right);
        let container = super::page_layout::LayoutRect {
            x: column.x + left,
            width: (column.width - left - right).max(0.0),
            ..column
        };
        let paper = super::page_layout::LayoutRect {
            x: 0.0,
            y: 0.0,
            width: self.layout.page_width,
            height: self.layout.page_height,
        };
        let frame = super::float_placement::ObjectPlacementFrame {
            container: &container,
            column: &column,
            body: &self.layout.body_area,
            paper: &paper,
            paragraph_y: column.y + paragraph_top.unwrap_or(0.0),
            alignment: style.map_or(crate::model::style::Alignment::Left, |s| s.alignment),
            dpi: self.layout.dpi,
        };
        if let Some(exclusion) = frame.picture_exclusion(picture) {
            self.side_wrap_exclusions
                .insert((para_index, control_index), exclusion);
        }
    }

    /// 현재 항목을 ColumnContent로 만들어 마지막 페이지에 push
    fn flush_column(&mut self) {
        // [#4090] 쪽이 끝나면 어울림 밴드도 끝난다 — 개체 높이를 used 에 반영한다.
        self.close_square_band();
        self.inline_box_flow_bottom = 0.0;
        if self.current_items.is_empty()
            && self.current_column_wrap_around_paras.is_empty()
            && self.page_start_square_pictures.is_empty()
        {
            return;
        }
        let col_content = ColumnContent {
            column_index: self.current_column,
            start_height: self.current_start_height,
            endnote_flow: self.current_endnote_flow,
            items: self.take_current_page_items(),
            zone_layout: self.current_zone_layout.clone(),
            zone_y_offset: self.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.current_column_wrap_around_paras),
            used_height: self.current_height,
            wrap_anchors: std::mem::take(&mut self.current_column_wrap_anchors),
            overlay_continuations: std::mem::take(&mut self.current_column_overlay_continuations),
            overlay_cuts: std::mem::take(&mut self.current_column_overlay_cuts),
            inline_placements: std::mem::take(&mut self.inline_placements),
            inline_flow_plans: std::mem::take(&mut self.inline_flow_plans),
            paragraph_float_placements: std::mem::take(&mut self.paragraph_float_placements),
        };
        if let Some(page) = self.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.pages.push(self.new_page_content(vec![col_content]));
        }
        // [#5699 H1] 이번 단에서 발동한 사다리-미계상 표를 소속 쪽에 기록.
        if !self.current_ladder_band_tables.is_empty() {
            if let Some(page) = self.pages.last_mut() {
                page.ladder_band_tables
                    .append(&mut self.current_ladder_band_tables);
            } else {
                self.current_ladder_band_tables.clear();
            }
        }
        // [Task #1082] 단 flush 시 본문 last bottom vpos 리셋(미주 vpos-delta 시드 정합).
        self.prev_body_bottom_vpos = None;
        // [#2279] flow 과소 누계도 단 단위 — 리셋.
        self.flow_underrun = 0.0;
        // [compat 2024] hangul2024_reclaimed 는 여기서 리셋하지 않는다 —
        // flush_column 은 Square 밴드 마감 등 쪽/단 전환 없이도 불리므로,
        // 리셋은 실제 전환 지점(advance_column_or_new_page/reset_for_new_page)에서.
    }

    /// deferred Square picture는 layout의 z/order상 이 column의 첫 item이어야 하지만,
    /// typeset 중에는 height·vpos·fit에 관여하면 안 된다. 실제 column을 flush할 때만
    /// 앞에 materialize해 두 성질을 함께 보존한다.
    fn take_current_page_items(&mut self) -> Vec<PageItem> {
        let mut items = std::mem::take(&mut self.page_start_square_pictures)
            .into_iter()
            .map(|deferred| PageItem::Shape {
                para_index: deferred.para_index,
                control_index: deferred.control_index,
            })
            .collect::<Vec<_>>();
        items.append(&mut self.current_items);
        items
    }

    /// 비어있어도 flush
    fn flush_column_always(&mut self) {
        self.inline_box_flow_bottom = 0.0;
        let col_content = ColumnContent {
            column_index: self.current_column,
            start_height: self.current_start_height,
            endnote_flow: self.current_endnote_flow,
            items: self.take_current_page_items(),
            zone_layout: self.current_zone_layout.clone(),
            zone_y_offset: self.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.current_column_wrap_around_paras),
            used_height: self.current_height,
            wrap_anchors: std::mem::take(&mut self.current_column_wrap_anchors),
            overlay_continuations: std::mem::take(&mut self.current_column_overlay_continuations),
            overlay_cuts: std::mem::take(&mut self.current_column_overlay_cuts),
            inline_placements: std::mem::take(&mut self.inline_placements),
            inline_flow_plans: std::mem::take(&mut self.inline_flow_plans),
            paragraph_float_placements: std::mem::take(&mut self.paragraph_float_placements),
        };
        if let Some(page) = self.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.pages.push(self.new_page_content(vec![col_content]));
        }
        // [#5699 H1] 이번 단에서 발동한 사다리-미계상 표를 소속 쪽에 기록.
        if !self.current_ladder_band_tables.is_empty() {
            if let Some(page) = self.pages.last_mut() {
                page.ladder_band_tables
                    .append(&mut self.current_ladder_band_tables);
            } else {
                self.current_ladder_band_tables.clear();
            }
        }
    }

    /// 다음 단 또는 새 페이지
    /// [#6146] 저장 vpos 리셋으로 다음 쪽에 넘어가는 문단의 **자리차지 밴드**를 떠나는
    /// 쪽의 흐름 말미에 남긴다.
    ///
    /// 한글은 보도자료 꼬리의 로고 글상자처럼 쪽 말미에 걸린 비-TAC 자리차지
    /// (TopAndBottom, vert=문단) 개체를 다음 쪽으로 옮기지 않고 본문 아래 여백으로
    /// 흘려 그 쪽에 남긴다 — 한글 2024 PDF 실측 4/4 (156583583 1쪽 y 1029.4..1079.5
    /// vs 본문 하단 1039.3, 156597957·156535759·156742932 동형). 옮기면 다음 쪽
    /// 상단의 제목 표와 겹쳐 글자가 가려진다(#6146).
    ///
    /// 판별은 물리적으로 — **개체 아래끝이 용지 안에 남을 때만** 흘린다. 쪽을 넘겨야
    /// 하는 큰 개체(#1156 의 80mm 차트 OLE 계열)는 아래끝이 용지를 벗어나 제외된다.
    /// 밴드 뒤의 줄·표는 리셋대로 다음 쪽에 놓이므로 쪽수는 바뀌지 않는다.
    fn spill_page_tail_floats(&mut self, para_idx: usize, para: &Paragraph) {
        use crate::model::shape::{TextWrap, VertRelTo};
        if self.current_items.is_empty() || self.col_count > 1 {
            return;
        }
        let dpi = self.layout.dpi;
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            let common = match ctrl {
                Control::Picture(picture) => &picture.common,
                Control::Shape(shape) => shape.common(),
                // 표는 자기 배치 경로(place_table_with_text)가 있으므로 제외한다.
                _ => continue,
            };
            if common.treat_as_char
                || !matches!(common.text_wrap, TextWrap::TopAndBottom)
                || !matches!(common.vert_rel_to, VertRelTo::Para)
            {
                continue;
            }
            let height = hwpunit_to_px(common.height as i32, dpi)
                + hwpunit_to_px(i32::from(common.margin.bottom), dpi);
            if height <= 0.0
                || self.layout.body_area.y + self.current_height + height
                    > self.layout.page_height + 0.5
            {
                continue;
            }
            self.current_items.push(PageItem::Shape {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            self.current_height += height;
            self.page_tail_spilled_floats.insert((para_idx, ctrl_idx));
        }
    }

    fn advance_column_or_new_page(&mut self) {
        self.flush_column();
        self.visible_float_exclusions.clear();
        if self.current_column + 1 < self.col_count {
            self.current_column += 1;
            // Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지한 높이를
            // current_height의 시작값으로 사용 (가용 공간만 줄임, zone_y_offset은 건드리지 않음).
            // layout은 body_wide_reserved로 별도 처리하므로 여기서 zone_y_offset에
            // 넣으면 double-shift가 발생.
            self.current_height = self.pending_body_wide_top_reserve;
            self.current_start_height = self.current_height;
            self.current_endnote_flow = false;
            // [#5699 H1] 밴드 교정 바닥은 단 흐름 좌표 — 새 단에서 리셋.
            self.ladder_band_floor = 0.0;
            // [compat 2024] 앵커 줄 회수 누계는 단 단위 — 새 단에서 리셋.
            if self.hangul2024_reclaimed > 0.0 && std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
                eprintln!(
                    "DIAG_COMPAT24 clear@column-advance reclaimed={:.1}",
                    self.hangul2024_reclaimed
                );
            }
            self.hangul2024_reclaimed = 0.0;
            self.column_had_compact_endnote_rewind = false;
            self.reset_vpos_cursor();
        } else {
            self.push_new_page();
        }
    }

    /// 강제 새 페이지
    fn force_new_page(&mut self) {
        self.flush_column();
        self.push_new_page();
    }

    /// 페이지 보장
    fn ensure_page(&mut self) {
        if self.pages.is_empty() {
            self.pages.push(self.new_page_content(Vec::new()));
        }
    }

    /// 새 페이지 push + 상태 리셋
    fn push_new_page(&mut self) {
        self.layout
            .apply_column_page_number(self.pages.len() as u32 + 1);
        self.pages.push(self.new_page_content(Vec::new()));
        // 합성 어울림 배제 사각형은 쪽 단위 — 새 쪽에서 비운다.
        self.wrap_synth_rects.clear();
        self.reset_for_new_page();
        // [#3738 Stage 22] current page의 본문/각주 흐름을 먼저 확정한 뒤, tail
        // Square picture만 새 physical page의 layout item 앞에 둔다. 즉시
        // `current_items`에 넣지 않아 다음 paragraph의 height/vpos fit은 기존과
        // 같게 유지하고, flush 때만 Shape로 materialize한다.
        for deferred in std::mem::take(&mut self.deferred_next_page_square_pictures) {
            // typeset_wrap_around_paragraph는 next paragraph가 page break를 일으키기
            // 전에 호출된다. 따라서 이 지점에서 다음 page column에 직접 anchor를
            // 등록해야 p1356처럼 whole-narrow paragraph도 stored cs/sw를 보존한다.
            for wrap_target_para_index in &deferred.wrap_target_para_indices {
                self.current_column_wrap_anchors
                    .insert(*wrap_target_para_index, deferred.wrap_anchor.clone());
            }
            self.page_start_square_pictures.push(deferred);
        }
        // Task #321: 새 페이지에서는 body-wide top reserve 초기화
        self.pending_body_wide_top_reserve = 0.0;
        // [#4568] 앞 쪽에서 잘린 overlay 표의 잔여 행을 이 쪽 최상단에 이어 그린다.
        // `current_items` 가 아니라 단 전용 목록으로 넘긴다 — 흐름 항목이 아니라
        // z-layer 장식이고, 항목으로 섞으면 이 조각이 단의 첫 항목이 되어
        // `items.first()` 를 보는 휴리스틱이 조각을 본문으로 읽는다.
        let mut overlay_top_reserve = 0.0f64;
        self.current_column_overlay_continuations.extend(
            std::mem::take(&mut self.pending_overlay_continuations)
                .into_iter()
                .map(|(para_index, control_index, start_row, remaining_px)| {
                    overlay_top_reserve = overlay_top_reserve.max(remaining_px);
                    crate::renderer::pagination::OverlayContinuation {
                        para_index,
                        control_index,
                        start_row,
                        reserve_px: remaining_px,
                    }
                }),
        );
        // 잔여 높이를 새 쪽 흐름에 무조건 예약하면 안 된다 — #4514 기제에서 필러
        // 문단들이 이미 표 높이만큼 흐름 공간을 만들므로 이중 계상이 된다(실측:
        // 예약 시 48 → 56쪽, 한컴 46쪽에서 더 멀어짐). 그래서 대기열 등록부(#5792
        // 게이트)가 "뒤따르는 흐름이 자리를 만들지 못한다"고 판정한 조각만 0 아닌
        // 값을 싣는다 — 필러 형상은 종전대로 0 이라 불변이다.
        if overlay_top_reserve > 0.0 {
            self.current_height = self.current_height.max(overlay_top_reserve);
            self.current_start_height = self.current_height;
        }
    }

    fn reset_for_new_page(&mut self) {
        self.side_wrap_exclusions.clear();
        self.current_column = 0;
        self.current_height = 0.0;
        self.current_start_height = 0.0;
        // [#5699 H1] 밴드 교정 바닥은 쪽 단위.
        self.ladder_band_floor = 0.0;
        // [compat 2024] 앵커 줄 회수 누계는 쪽 단위 — 새 쪽에서 리셋.
        if self.hangul2024_reclaimed > 0.0 && std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
            eprintln!(
                "DIAG_COMPAT24 clear@new-page reclaimed={:.1}",
                self.hangul2024_reclaimed
            );
        }
        self.hangul2024_reclaimed = 0.0;
        self.current_endnote_flow = false;
        self.column_had_compact_endnote_rewind = false;
        self.current_footnote_height = 0.0;
        self.current_bottom_fixed_exclusion = 0.0;
        self.bottom_fixed_consumed_flow = 0.0;
        self.page_has_page_abs_top_table = false;
        self.is_first_footnote_on_page = true;
        self.current_page_has_footnote_separator = false;
        self.current_zone_y_offset = 0.0;
        self.current_zone_layout = None;
        self.on_first_multicolumn_page = false;
        self.visible_float_exclusions.clear();
        self.reset_vpos_cursor();
    }

    fn apply_visible_float_exclusions(&mut self, probe_height: f64) {
        if self.visible_float_exclusions.is_empty() {
            return;
        }

        let use_overlap_probe = self.profile.hwpx_stored_layout() && probe_height > 0.0;
        self.visible_float_exclusions
            .retain(|zone| self.current_height < zone.bottom - 0.5);

        let mut jump_to = self.current_height;
        for zone in &self.visible_float_exclusions {
            let starts_in_zone = jump_to + 0.5 >= zone.top && jump_to < zone.bottom;
            let overlaps_zone =
                use_overlap_probe && jump_to < zone.top && jump_to + probe_height > zone.top + 0.5;
            if starts_in_zone || overlaps_zone {
                jump_to = jump_to.max(zone.bottom);
            }
        }

        if jump_to > self.current_height + 0.5 {
            self.current_height = jump_to;
        }
    }

    /// #2439: layout resolves an empty para-relative float's natural top against the visible
    /// float lane before painting it. Native pagination must consume the same lane when the
    /// upcoming natural top (flow + positive offset/outer lead) intersects it; the generic
    /// paragraph probe above intentionally does not enable native overlap probing.
    fn apply_visible_float_exclusions_for_para_float(&mut self, natural_top_lead: f64) {
        if self.visible_float_exclusions.is_empty() || natural_top_lead <= 0.0 {
            return;
        }

        self.visible_float_exclusions
            .retain(|zone| self.current_height < zone.bottom - 0.5);
        let mut jump_to = self.current_height;
        for zone in &self.visible_float_exclusions {
            let natural_top = jump_to + natural_top_lead;
            if natural_top + 0.5 >= zone.top && natural_top < zone.bottom {
                jump_to = jump_to.max(zone.bottom);
            }
        }
        if jump_to > self.current_height + 0.5 {
            self.current_height = jump_to;
        }
    }

    /// [#6764] 블록 표를 이 쪽에 앉히기 전에, **다른 문단**이 남긴 자리차지 밴드를
    /// 표 높이로 한 번 짚는다.
    ///
    /// 문단 흐름용 `apply_visible_float_exclusions` 의 겹침 프로브는 HWPX 저장
    /// 프로파일 전용이라, 네이티브 HWP5 에서 `vertical_offset` 이 앵커와 표 상단
    /// 사이를 벌려 놓으면 그 틈에서 시작하는 표가 밴드를 **그냥 통과한다.**
    /// 계상은 틈 위에 남고 페인트는 밴드 아래로 가므로 분할 예산이 통째로 어긋난다
    /// (1613000-202200037 182쪽: 예산 817.6px 로 23행을 잘라 넣었는데 페인트는
    /// 용지 밖 885.6px). 표는 한 덩어리라 어긋남이 쪽 규모로 드러난다.
    ///
    /// 같은 문단이 만든 밴드는 건드리지 않는다 — co-anchored float 스택은 자기
    /// 밴드를 넘겨 짚으면 안 된다(`#1510`).
    fn apply_float_band_before_block_table(&mut self, para_index: usize, probe_height: f64) {
        if self.visible_float_exclusions.is_empty() || probe_height <= 0.5 {
            return;
        }
        self.visible_float_exclusions
            .retain(|zone| self.current_height < zone.bottom - 0.5);

        let mut jump_to = self.current_height;
        for zone in &self.visible_float_exclusions {
            if zone.para_index == para_index {
                continue;
            }
            let starts_in_zone = jump_to + 0.5 >= zone.top && jump_to < zone.bottom;
            let crosses_zone = jump_to < zone.top && jump_to + probe_height > zone.top + 0.5;
            if starts_in_zone || crosses_zone {
                jump_to = jump_to.max(zone.bottom);
            }
        }
        if jump_to > self.current_height + 0.5 {
            self.current_height = jump_to;
        }
    }

    fn new_page_content(&self, column_contents: Vec<ColumnContent>) -> PageContent {
        PageContent {
            page_index: self.pages.len() as u32,
            page_number: 0,
            page_number_restarted: false,
            section_index: self.section_index,
            layout: self.layout.clone(),
            column_contents,
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
            ladder_band_tables: Vec::new(),
        }
    }
}

/// dump-pages가 프로덕션 `format_paragraph` 높이를 그대로 말하기 위한 분해 (#4628).
#[derive(Debug, Clone, Copy)]
pub(crate) struct DumpFormattedParagraphHeight {
    pub total: f64,
    pub spacing_before: f64,
    pub line_height_sum: f64,
    pub spacing_after: f64,
    pub line_spacing_sum: f64,
}

mod controls;
#[path = "typeset/inline_flow.rs"]
mod inline_flow;
mod notes;
mod paragraph;
mod state;
mod table;

/// [#2279 OMIT-eager] 저장 ladder 의 spacing-누락(OMIT) 서명 사전 판별.
///
/// #2383 의 lazy 판별(빈 host 문단의 표 성장 시점)은 첫 검출 지점 이전의
/// 페이지말 fit 에 문서군 규칙을 적용하지 못한다(36392557: 판별 pi27, 실제
/// 경계 pi14). 같은 서명 — 연속 문단쌍의 저장 스텝이 bare lh+ls 와 일치
/// (±2px)하면서 그 경계의 paraPr spacing(cur.sa + next.sb)을 반영하지 않음 —
/// 을 구역 시작 시점에 전방 스캔으로 검출한다. 2쌍 이상 요구(단발 우연 일치
/// 배제). 정상(스텝이 spacing 포함) 문서는 스텝이 lh+ls 보다 spacing 만큼
/// 길어 불일치한다.
fn ladder_spacing_omitted_signature(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> bool {
    use crate::model::paragraph::LineSeg;
    // [#2279 OMIT-eager 판별 3요건] lineseg 가 통째로 드롭된 본문 문단(마스킹·
    // 생성기 드롭)이 **다수 비율**로 존재 — 한글이 저장 흐름을 신뢰하지 못하고
    // fresh 재계산하는 문서군의 본질 신호다. 실측 분리: 재계산 문서군은
    // 0.39~0.80(36475730/36360680/36392557), 저장 흐름 신뢰 문서는 0(보도자료
    // 156652332)~0.10(sample16 #2158 핀) — 임계 25% + 4문단, 이하는 국소
    // 드롭으로 보고 저장 흐름 신뢰를 유지한다(+1 회귀 실측).
    let mut text_paras = 0usize;
    let mut segless_text_paras = 0usize;
    for p in paragraphs {
        if p.text.trim().is_empty() {
            continue;
        }
        text_paras += 1;
        if !p
            .line_segs
            .iter()
            .any(|s| s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        {
            segless_text_paras += 1;
        }
    }
    if segless_text_paras < 4 || segless_text_paras * 4 < text_paras {
        return false;
    }
    let mut hits = 0usize;
    let mut spacing_omitted = false;
    // [#2279 OMIT-eager 판별 2요건] 누적좌표 ladder 문서(리셋 없음, 예: 36386907)
    // 는 저장 좌표가 한글 fresh 와 일치하는 신뢰 문서군이라 spacing-누락 스텝이
    // 있어도 페이지말 fit 신뢰 철회 대상이 아니다(트림 수용이 정답 — 92셋 실측
    // 2건 +1 반증). 페이지별 vpos 리셋(재배치 사다리, 예: 36392557 pi14→pi15)
    // 을 함께 요구한다.
    let mut has_page_reset = false;
    for pair in paragraphs.windows(2) {
        let (cur, next) = (&pair[0], &pair[1]);
        let (Some(cur_last), Some(next_first)) = (cur.line_segs.last(), next.line_segs.first())
        else {
            continue;
        };
        if cur_last.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || next_first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        {
            continue;
        }
        if next_first.vertical_pos <= cur_last.vertical_pos {
            // vpos 리셋/역행 경계 — 스텝 판정 불가. 쪽 하단→상단 리셋만 인정.
            // 개체(표/그림) host 문단의 v=0 은 TAC 장식·footer 표의 좌표 관례라
            // 쪽 리셋이 아니다(156652332 담당부서 표 오인 → +1 회귀 실측) —
            // 개체 없는 본문 텍스트 문단으로의 리셋만 페이지 재배치 신호로 본다.
            if cur_last.vertical_pos > 5000
                && next_first.vertical_pos < 5000
                && next.controls.is_empty()
                && para_has_visible_text(next)
            {
                has_page_reset = true;
            }
            continue;
        }
        if spacing_omitted {
            if has_page_reset {
                return true;
            }
            continue;
        }
        let sa = styles
            .para_styles
            .get(cur.para_shape_id as usize)
            .map(|s| s.spacing_after)
            .unwrap_or(0.0);
        let sb = styles
            .para_styles
            .get(next.para_shape_id as usize)
            .map(|s| s.spacing_before)
            .unwrap_or(0.0);
        let spacing_around = sa + sb;
        if spacing_around <= 2.5 {
            continue;
        }
        let step = hwpunit_to_px(next_first.vertical_pos - cur_last.vertical_pos, dpi);
        let bare = hwpunit_to_px(
            cur_last.line_height.saturating_add(cur_last.line_spacing),
            dpi,
        );
        if step <= 0.0 || bare <= 0.0 {
            continue;
        }
        if (step - bare).abs() < 2.0 && step + 1.0 < bare + spacing_around {
            hits += 1;
            if std::env::var("RHWP_DIAG_OMITSIG").is_ok() {
                eprintln!(
                    "DIAG_OMITSIG hit step={:.1} bare={:.1} sa={:.1} sb={:.1} cur_v={} next_v={}",
                    step, bare, sa, sb, cur_last.vertical_pos, next_first.vertical_pos
                );
            }
            if hits >= 2 {
                spacing_omitted = true;
                if has_page_reset {
                    return true;
                }
            }
        }
    }
    spacing_omitted && has_page_reset
}

fn debug_brief_line_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(max_chars) {
        match ch {
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\u{FFFC}' => out.push_str("<TAC>"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    if text.chars().count() > max_chars {
        out.push('…');
    }
    out
}

/// [#2424 프로파일] `typeset_section_with_variant` 하위 단계 누적 실측 — 동작 불변.
/// `RHWP_2424_PROFILE=1`(native 전용)일 때만 채워지고 구역당 한 줄로 출력한다.
/// `(Duration, u32)` 는 (누적 시간, 호출 수). wasm 에서는 enabled=false 로 고정되어
/// `Instant::now` 가 호출되지 않는다 (`paginate_pass` 의 게이트 패턴과 동일).
#[derive(Default)]
struct Issue2424TypesetProfile {
    setup: std::time::Duration,
    wrap_around: (std::time::Duration, u32),
    text_para: (std::time::Duration, u32),
    table_para: (std::time::Duration, u32),
    deferred_flush: (std::time::Duration, u32),
    para_tail: (std::time::Duration, u32),
    endnotes: std::time::Duration,
}

impl Issue2424TypesetProfile {
    fn add(bucket: &mut (std::time::Duration, u32), started: Option<std::time::Instant>) {
        if let Some(started) = started {
            bucket.0 += started.elapsed();
            bucket.1 += 1;
        }
    }

    fn ms(duration: std::time::Duration) -> f64 {
        duration.as_secs_f64() * 1000.0
    }
}

impl TypesetEngine {
    pub fn new(dpi: f64) -> Self {
        Self {
            dpi,
            profile: std::cell::Cell::new(Default::default()),
            uniform_filler_ladder: std::cell::Cell::new(false),
            float_carve_evidence: std::cell::RefCell::new(Vec::new()),
            render_normalization: std::sync::Arc::new(
                crate::renderer::render_normalization::RenderNormalizationOverlay::default(),
            ),
        }
    }

    pub(crate) fn with_render_normalization(
        mut self,
        overlay: std::sync::Arc<crate::renderer::render_normalization::RenderNormalizationOverlay>,
    ) -> Self {
        self.render_normalization = overlay;
        self
    }

    pub fn with_default_dpi() -> Self {
        Self::new(DEFAULT_DPI)
    }

    /// 구역 조판과 같은 format 문맥을 연다. dump-pages 진단이 pagination과
    /// 다른 높이를 말하지 않도록 `typeset_section_with_variant` 와 공유한다 (#4628).
    pub(crate) fn apply_section_format_context(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
    ) {
        self.profile.set(profile);
        self.uniform_filler_ladder
            .set(crate::renderer::stored_line_ladder_is_uniform_filler(
                paragraphs, styles,
            ));
        *self.float_carve_evidence.borrow_mut() =
            crate::renderer::float_placement::paper_or_page_float_carve_evidence(paragraphs);
    }

    /// 프로덕션 문단 높이 분해. pagination이 `format_paragraph` 로 쓰는 값과 같다.
    pub(crate) fn dump_formatted_paragraph_height(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
        endnote: bool,
    ) -> DumpFormattedParagraphHeight {
        let fmt = if endnote {
            self.format_endnote_paragraph(para, composed, styles, column_width_px)
        } else {
            self.format_paragraph(para, composed, styles, column_width_px)
        };
        DumpFormattedParagraphHeight {
            total: fmt.total_height,
            spacing_before: fmt.spacing_before,
            line_height_sum: fmt.line_heights.iter().sum(),
            line_spacing_sum: fmt.line_spacings.iter().sum(),
            spacing_after: fmt.spacing_after,
        }
    }

    fn predict_current_column_para_y(
        &self,
        st: &TypesetState,
        target_para_idx: usize,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        column_width: Option<f64>,
    ) -> Option<f64> {
        let mut local_paras: Vec<Paragraph> = Vec::new();
        let mut local_indices: Vec<(usize, usize)> = Vec::new();
        for pi in st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .chain(std::iter::once(target_para_idx))
        {
            if local_indices.iter().any(|(global, _)| *global == pi) {
                continue;
            }
            if let Some(p) = paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi) {
                let local = local_paras.len();
                local_paras.push(p.clone());
                local_indices.push((pi, local));
            }
        }
        let lookup_local = |pi: usize, indices: &[(usize, usize)]| {
            indices
                .iter()
                .find_map(|(global, local)| (*global == pi).then_some(*local))
        };
        let first_vpos = st
            .current_items
            .iter()
            .filter_map(page_item_para_index)
            .find_map(|pi| {
                paragraph_by_global_index(paragraphs, &st.endnote_paragraphs, pi)
                    .and_then(|p| p.line_segs.first())
                    .map(|seg| seg.vertical_pos)
            })?;

        let available = st.available_height();
        let mut hc = HeightCursor::new(
            self.dpi,
            0.0,
            available,
            st.current_start_height,
            Some(first_vpos),
            st.skip_spacing_before_prededuct,
            false,
            st.current_endnote_flow && st.current_start_height < -0.5,
            st.current_endnote_flow,
        );
        hc.endnote_between_notes_hu = st.endnote_between_notes_hu;
        let mut y = st.current_start_height;
        for item in &st.current_items {
            let Some(pi) = page_item_para_index(item) else {
                continue;
            };
            let Some(local) = lookup_local(pi, &local_indices) else {
                continue;
            };
            y = hc.vpos_adjust(y, local, &local_paras, styles);
            let item_para = &local_paras[local];
            let item_composed =
                crate::renderer::composer::compose_paragraph_in_context(item_para, styles);
            let item_fmt =
                self.format_paragraph(item_para, Some(&item_composed), styles, column_width);
            y += match item {
                PageItem::PartialParagraph {
                    start_line,
                    end_line,
                    ..
                } => item_fmt.line_advances_sum(*start_line..*end_line),
                PageItem::FullParagraph { .. } => item_fmt.total_height,
                PageItem::Table {
                    para_index,
                    control_index,
                } => measured_tables
                    .iter()
                    .find(|mt| mt.para_index == *para_index && mt.control_index == *control_index)
                    .map(|mt| mt.total_height)
                    .unwrap_or(0.0),
                PageItem::PartialTable {
                    para_index,
                    control_index,
                    start_row,
                    end_row,
                    ..
                } => measured_tables
                    .iter()
                    .find(|mt| mt.para_index == *para_index && mt.control_index == *control_index)
                    .map(|mt| {
                        let start = mt
                            .cumulative_heights
                            .get(*start_row)
                            .copied()
                            .unwrap_or(0.0);
                        let end = mt
                            .cumulative_heights
                            .get(*end_row)
                            .copied()
                            .unwrap_or(mt.total_height);
                        (end - start).max(0.0)
                    })
                    .unwrap_or(0.0),
                _ => 0.0,
            };
            let current_vpos_rewinds_from_prev = hc
                .prev_layout_para
                .and_then(|prev_local| {
                    let prev_first = local_paras
                        .get(prev_local)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    let curr_first = local_paras
                        .get(local)
                        .and_then(|p| p.line_segs.first())
                        .map(|seg| seg.vertical_pos)?;
                    Some(curr_first < prev_first)
                })
                .unwrap_or(false);
            if matches!(
                item,
                PageItem::PartialParagraph { start_line, .. } if *start_line > 0
            ) || current_vpos_rewinds_from_prev
            {
                hc.prev_layout_para = None;
                hc.vpos_page_base = None;
                hc.vpos_lazy_base = None;
            } else {
                hc.prev_layout_para = Some(local);
            }
            hc.prev_item_was_partial_table = matches!(item, PageItem::PartialTable { .. });
        }

        let local = lookup_local(target_para_idx, &local_indices)?;
        Some(hc.vpos_adjust(y, local, &local_paras, styles))
    }

    /// 구역의 문단 목록을 조판한다 (단일 패스).
    ///
    /// 기존 paginate()와 동일한 PaginationResult를 반환하므로
    /// 기존 layout/render 파이프라인과 호환된다.
    /// [Task #1046] 비-variant 단축 호출 — `is_hwp3_variant=false` 로 delegate.
    /// 기존 PR/tests 가 사용. force_break_before 는 사후 reflow 이월 hint.
    #[allow(clippy::too_many_arguments)]
    pub fn typeset_section(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        measured_tables: &[MeasuredTable],
        hide_empty_line: bool,
        force_break_before: &std::collections::HashSet<usize>,
    ) -> PaginationResult {
        self.typeset_section_with_variant(
            paragraphs,
            composed,
            styles,
            page_def,
            column_def,
            section_index,
            measured_tables,
            hide_empty_line,
            Default::default(),
            false,
            false,
            None,
            None,
            force_break_before,
            EndnoteDeferral::None,
        )
    }

    /// [Task #2094] HWP3 변환본 vpos 리셋 쪽나눔 판정 — 소스분기(is_hwp3_variant)는
    /// caller 유지, 본 함수는 판정 본체만 담당한다 (원본 무변경 이동).
    #[allow(clippy::too_many_arguments)]
    fn judge_hwp3_variant_vpos_reset_break(
        &self,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        para_idx: usize,
        body_height_hu_for_variant: i32,
        variant_prev_para_idx: Option<usize>,
    ) -> bool {
        let mut variant_vpos_reset_break = false;
        if body_height_hu_for_variant > 0 && !para.text.is_empty() {
            let para_sb = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|ps| ps.spacing_before)
                .unwrap_or(0.0);
            let para_sb_hu = (para_sb * 7200.0 / 96.0) as i32;
            let prev_real_idx_and_ls = variant_prev_para_idx.and_then(|prev_pi| {
                (0..=prev_pi).rev().find_map(|i| {
                    paragraphs
                        .get(i)
                        .and_then(|p| p.line_segs.last())
                        .filter(|ls| !is_synthetic_line_seg(ls))
                        .map(|ls| (i, ls))
                })
            });
            let curr_real = para
                .line_segs
                .first()
                .filter(|ls| !is_synthetic_line_seg(ls));
            if let Some((prev_real_idx, prev_last)) = prev_real_idx_and_ls {
                let prev_end_vpos = prev_last.vertical_pos.saturating_add(prev_last.line_height);
                let prev_positive_wrap_end = paragraphs
                    .get(prev_real_idx)
                    .and_then(positive_vpos_end_before_negative_wrap);
                let prev_prev_end_vpos = if prev_real_idx > 0 {
                    (0..prev_real_idx).rev().find_map(|i| {
                        paragraphs.get(i).and_then(|p| {
                            p.line_segs
                                .last()
                                .filter(|ls| !is_synthetic_line_seg(ls))
                                .map(|ls| ls.vertical_pos.saturating_add(ls.line_height))
                        })
                    })
                } else {
                    None
                };
                let prev_top_content_reset = paragraphs.get(prev_real_idx).is_some_and(|p| {
                    let prev_sb_hu = styles
                        .para_styles
                        .get(p.para_shape_id as usize)
                        .map(|ps| (ps.spacing_before * 7200.0 / 96.0) as i32)
                        .unwrap_or(0);
                    p.line_segs.len() == 1
                        && p.line_segs
                            .first()
                            .is_some_and(|ls| !is_synthetic_line_seg(ls) && ls.vertical_pos == 0)
                        && p.controls.is_empty()
                        && para_has_visible_text(p)
                        && prev_sb_hu < 250
                });
                let next_first_real_vpos = paragraphs
                    .get(para_idx + 1)
                    .and_then(|next_para| next_para.line_segs.first())
                    .filter(|ls| !is_synthetic_line_seg(ls))
                    .map(|ls| ls.vertical_pos);
                let bridge_missing_count = (prev_real_idx + 1..para_idx)
                    .filter(|&i| {
                        paragraphs.get(i).is_some_and(|p| {
                            p.line_segs.is_empty()
                                && p.controls.is_empty()
                                && para_has_visible_text(p)
                        })
                    })
                    .count();
                let high_threshold = body_height_hu_for_variant * 95 / 100;
                let table_heading_reset = prev_real_idx + 1 == para_idx
                    && para.line_segs.is_empty()
                    && para.controls.is_empty()
                    && para_has_visible_text(para)
                    && para_sb_hu >= 500
                    && prev_end_vpos > body_height_hu_for_variant * 85 / 100
                    && paragraphs.get(prev_real_idx).is_some_and(|prev_para| {
                        prev_para
                            .controls
                            .iter()
                            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    })
                    && paragraphs
                        .get(para_idx + 1)
                        .and_then(|next_para| next_para.line_segs.first())
                        .filter(|ls| !is_synthetic_line_seg(ls))
                        .is_some_and(|ls| ls.vertical_pos <= 4000);
                let empty_bridge_heading_reset = para.line_segs.is_empty()
                    && para.controls.is_empty()
                    && para_has_visible_text(para)
                    && para_sb_hu >= 500
                    && bridge_missing_count == 1
                    && prev_end_vpos > body_height_hu_for_variant * 80 / 100
                    && prev_end_vpos <= body_height_hu_for_variant * 85 / 100;

                let real_heading_or_bridge_reset = curr_real.is_some_and(|curr_first| {
                    let curr_first_vpos = curr_first.vertical_pos;
                    let strict_heading_reset = para_sb_hu >= 500
                        && prev_end_vpos > high_threshold
                        && curr_first_vpos < 1500;
                    let delayed_heading_after_top_content_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() >= 2
                        && para_sb_hu >= 500
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos > 0
                        && curr_first_vpos <= 2500
                        && prev_top_content_reset
                        && prev_prev_end_vpos
                            .is_some_and(|end| end > body_height_hu_for_variant * 70 / 100);
                    let bridged_reset = bridge_missing_count >= 2
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos <= 1500
                        && prev_end_vpos > body_height_hu_for_variant * 75 / 100;
                    let negative_wrap_heading_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() == 1
                        && para_sb_hu >= 250
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos < 0
                        && prev_positive_wrap_end
                            .is_some_and(|end| end > body_height_hu_for_variant * 75 / 100);
                    let bottom_heading_before_next_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() == 1
                        && para_sb_hu >= 250
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos > body_height_hu_for_variant * 75 / 100
                        && next_first_real_vpos.is_some_and(|next_vpos| {
                            next_vpos > 0 && next_vpos <= 4000 && curr_first_vpos > next_vpos
                        });
                    strict_heading_reset
                        || delayed_heading_after_top_content_reset
                        || bridged_reset
                        || negative_wrap_heading_reset
                        || bottom_heading_before_next_reset
                });

                if table_heading_reset || empty_bridge_heading_reset || real_heading_or_bridge_reset
                {
                    variant_vpos_reset_break = true;
                }
            }
        }
        variant_vpos_reset_break
    }

    /// [Task #2094] 표 없는 문단의 배치 마무리 국면 — 원본 무변경 통이동 (st 변이만).
    #[allow(clippy::too_many_arguments)]
    fn typeset_no_table_paragraph_tail(
        &self,
        st: &mut TypesetState,
        page_def: &PageDef,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        para_idx: usize,
        has_table: bool,
    ) {
        controls::prepare_no_table_host_wrap(st, page_def, para, para_idx, has_table);
    }

    /// Native HWP5의 page-tail Square 그림은 anchor 본문과 분리되어 다음 physical
    /// page의 wrap band를 소유할 수 있다.
    ///
    /// 단순히 현재 단의 여유가 작다는 이유로 Square 그림을 이월하면, 같은 쪽에 의도된
    /// caption/side-wrap 그림을 넓게 건드린다. 따라서 다음 문단에 “vpos=0 narrow wrap
    /// 줄”이라는 저장 계약이 있고, 현재 쪽의 기존 각주를 고려하면 그림 frame 자체가 더는
    /// 들어가지 않는 native HWP5 Picture에만 적용한다.
    fn native_hwp5_square_picture_next_page_owner(
        &self,
        st: &TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        ctrl: &Control,
        styles: &ResolvedStyleSet,
    ) -> Option<(Vec<usize>, crate::renderer::pagination::WrapAnchorRef)> {
        controls::deferred_picture::next_page_owner(
            st.deferred_picture_page(),
            || st.available_height(),
            self.dpi,
            para_idx,
            para,
            paragraphs,
            ctrl,
            styles,
        )
    }

    /// 후속 어울림 문단 처리는 Query/Command를 연결하는 조정자에 위임한다.
    /// 반환 true = 원본의 `continue`(이 문단은 흡수/기록 완료, 배치 생략) 신호.
    #[allow(clippy::too_many_arguments)]
    fn typeset_wrap_around_paragraph(
        &self,
        st: &mut TypesetState,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        para_idx: usize,
        has_table: bool,
        page_def: &PageDef,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
    ) -> bool {
        controls::wrap_flow::place(
            self, st, para, paragraphs, para_idx, has_table, page_def, composed, styles,
        )
    }

    /// [Task #1007] HWP3 → HWP5 변환본 인지 typeset.
    /// 변환본 시 cross-paragraph vpos reset (이전 last vpos > body/2 + 현재 first vpos < body/4)
    /// 감지하여 page break 트리거 (한컴 인코딩 page break 시그널).
    ///
    /// [Task #1046] `force_break_before`: 사후 reflow 이월 hint — 이 para_idx 들은 현재
    /// 페이지에 이미 항목이 있으면 새 페이지에서 시작한다 (layout overflow 로 판정된 항목
    /// 이월). 빈 셋이면 무동작 → 기존 출력 불변.
    #[allow(clippy::too_many_arguments)]
    pub fn typeset_section_with_variant(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        measured_tables: &[MeasuredTable],
        hide_empty_line: bool,
        profile: crate::model::provenance::LayoutCompatibilityProfile,
        skip_spacing_before_prededuct: bool,
        hwp3_origin_page_tolerance: bool,
        footnote_shape: Option<&FootnoteShape>,
        endnote_shape: Option<&FootnoteShape>,
        force_break_before: &std::collections::HashSet<usize>,
        endnote_deferral: EndnoteDeferral<'_>,
    ) -> PaginationResult {
        // [#2424 프로파일] paginate_pass 와 같은 env var 로 하위 단계 게이트.
        #[cfg(not(target_arch = "wasm32"))]
        let issue2424_ts_enabled =
            std::env::var("RHWP_2424_PROFILE").is_ok_and(|value| !value.is_empty() && value != "0");
        #[cfg(target_arch = "wasm32")]
        let issue2424_ts_enabled = false;
        let issue2424_ts_started = issue2424_ts_enabled.then(std::time::Instant::now);
        let mut issue2424_prof = Issue2424TypesetProfile::default();

        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        // [#2403] 소스분기 프로파일 — 엔진(Cell)과 state 에 한 번에 배선.
        self.profile.set(profile);
        // [#5854] 통짜 합성 LINE_SEG 사다리 판정 — 구역당 한 번.
        self.uniform_filler_ladder
            .set(crate::renderer::stored_line_ladder_is_uniform_filler(
                paragraphs, styles,
            ));
        // [#6175] 용지/쪽 기준 어울림 개체의 폭과 세로 band - 구역당 한 번.
        *self.float_carve_evidence.borrow_mut() =
            crate::renderer::float_placement::paper_or_page_float_carve_evidence(paragraphs);
        let col_count = column_def.column_count.max(1);
        let default_footnote_shape = FootnoteShape::default();
        let footnote_shape = footnote_shape.unwrap_or(&default_footnote_shape);
        let footnote_separator_overhead = footnote_separator_overhead_px(footnote_shape, self.dpi);
        let footnote_between_notes_margin =
            footnote_between_notes_margin_px(footnote_shape, self.dpi);
        let footnote_safety_margin = hwpunit_to_px(3000, self.dpi);
        // [Task #1007] variant cross-paragraph vpos reset THRESHOLD 계산용 body height (HU)
        let body_height_hu_for_variant: i32 = if profile.hwp3_layout() {
            page_def.height.saturating_sub(
                page_def
                    .margin_top
                    .saturating_add(page_def.margin_bottom)
                    .saturating_add(page_def.margin_header)
                    .saturating_add(page_def.margin_footer),
            ) as i32
        } else {
            0
        };
        // [Task #1007] 이전 paragraph 인덱스 (variant vpos reset 감지용)
        let mut variant_prev_para_idx: Option<usize> = None;

        let mut st = TypesetState::new(
            layout,
            col_count,
            section_index,
            footnote_separator_overhead,
            footnote_between_notes_margin,
            footnote_safety_margin,
            column_def.column_type,
        );
        st.hide_empty_line = hide_empty_line;
        st.profile = profile;
        st.has_stored_line_segs = paragraphs
            .iter()
            .any(|p| p.line_segs.iter().any(|ls| !is_synthetic_line_seg(ls)));
        st.skip_spacing_before_prededuct = skip_spacing_before_prededuct;
        // [#2279 OMIT-eager] 기계생성 압축 ladder 문서군 사전 판별 — lazy 판별
        // (#2383, 빈 host 표 성장 시점)보다 앞서 페이지말 fit 규칙에 적용되도록
        // 구역 시작 시점에 확정한다. lazy 경로는 사전 스캔 미검출 형상의
        // fallback 으로 유지. HWP3-origin 계열(변환본·page tolerance 문서)은
        // 변환기 좌표 관례가 달라 저장 흐름 신뢰가 정답 — 대상이 아니다
        // (sample16 64→65 +1 실측, #2158 핀).
        if profile.hwpx_stored_layout()
            && !profile.hwp3_layout()
            && !profile.hwp3_native_layout()
            && !hwp3_origin_page_tolerance
        {
            st.omit_fresh_recalc_doc =
                ladder_spacing_omitted_signature(paragraphs, styles, self.dpi);
            if std::env::var("RHWP_DIAG_OMITSIG").is_ok() {
                eprintln!(
                    "DIAG_OMITSIG sec={} eager={}",
                    section_index, st.omit_fresh_recalc_doc
                );
            }
            // 사전 판별 문서군은 #2391 분할 진입 full-advance 계보도 동일하게
            // 적용 대상 — lazy 판별(#2383)의 상위 집합으로 승격만 한다.
            if st.omit_fresh_recalc_doc {
                st.stored_ladder_spacing_omitted = true;
            }
        }
        st.current_zone_design_spacing_px = column_def_design_spacing_px(column_def, self.dpi);

        // 머리말/꼬리말/쪽 번호/새 번호/감추기 컨트롤 수집
        let (hf_entries, page_number_pos) =
            Self::collect_header_footer_controls(paragraphs, section_index);
        // [#2559] 조건부(Even/Odd)까지 포함해 어떤 꼬리말이라도 정의돼 있으면
        // 밴드가 점유될 수 있다. 완전히 비어 있는 구역에서만 각주 회수를 허용한다.
        st.section_has_no_footer = !hf_entries.iter().any(|(_, _, is_header, _)| !is_header);
        // [#2668 진단] 밴드 회수 판정 관측 — 동작 불변.
        // "꼬리말 위치 쪽 번호(PageNumberPos)가 밴드를 점유하므로 회수 대상에서 빼야 한다"는
        // 판별자 가설을 이 출력으로 반증했다(회귀군·개선군 양쪽에 고루 존재).
        // 경위와 실측은 mydocs/report/task_2668_footnote_band_measurement.md 참조.
        if std::env::var("RHWP_DIAG_FBAND").is_ok() {
            eprintln!(
                "DIAG_FBAND sec={} footer_ctrl={} pnum_pos={:?} reclaim={}",
                section_index,
                hf_entries.iter().any(|(_, _, is_header, _)| !is_header),
                page_number_pos.as_ref().map(|p| p.position),
                st.section_has_no_footer
            );
        }

        // 가시 콘텐츠(텍스트 또는 컨트롤 보유)를 가진 마지막 문단 인덱스. 이 뒤의 빈 문단들은
        // 문서 말미의 trailing 빈 문단이라, co-anchored 자리차지 표가 페이지를 채운 경우에 한해
        // 현재 page 잔여를 초과하면 새 page 를 만들지 않고 흡수한다(아래 trailing-empty 가드).
        let last_content_para_idx: Option<usize> = paragraphs
            .iter()
            .enumerate()
            .rev()
            .find(|(_, p)| !(p.text.is_empty() && p.controls.is_empty()))
            .map(|(i, _)| i);

        if let Some(started) = issue2424_ts_started {
            issue2424_prof.setup = started.elapsed();
        }
        let issue2424_loop_started = issue2424_ts_enabled.then(std::time::Instant::now);

        for (para_idx, para) in paragraphs.iter().enumerate() {
            // [Task #1753] 지연 이월 표 직전에 선행 채움(prefill)으로 이미 배치된 문단 스킵.
            if st.prefilled_paras.contains(&para_idx) {
                continue;
            }
            // 후속 본문이 사이에 없는 새 표 문단은 이전 묶음의 일부가 아니다.
            // 이전 문단의 후행 표를 먼저 완료한다. 새 문단의 명시적 쪽/단 나눔과
            // 저장 vpos를 적용하기 전이어야 이전 표가 새 문단 뒤로 밀리지 않는다.
            // 제목/본문이 사이에 있는 float 흐름은 기존 후행 flush 계약을 유지한다.
            if st.has_deferred_table_controls() && self.paragraph_has_table(para) {
                self.flush_deferred_table_controls(
                    &mut st,
                    paragraphs,
                    composed,
                    styles,
                    measured_tables,
                    DeferredTableFlushPoint::BeforeTableParagraph(para_idx),
                );
            }
            // [#6132] 저장 vpos 가 쪽 본문을 넘고 바로 다음 문단이 되감기면,
            // 한글은 이 문단부터 다음 쪽에 둔 것이다. 다만 그 형상만으로는 부족하다 —
            // 같은 형상이 문단을 쪽 안에 그대로 두는 문서들에도 흔하게 나온다
            // (실측 후보: 2025 행정업무편람 26곳 · 2070 시장구조조사 13곳 ·
            // 2019 벤처투자 3곳 · hwp3-sample16 10곳). 그래서 세 신호를 **함께**
            // 요구한다.
            //
            // 156482639 7쪽: pi=102 '참고3' 표 vpos=73760 은 본문 73,335HU(977.8px)를
            // 넘고 pi=103 이 3790 으로 되감긴다. 한글은 둘 다 8쪽 첫머리에 두는데
            // rhwp 는 7쪽 잔여(974.5px)에 욱여넣어 8쪽이 56.8px 비었다.
            //
            // 기존 #3837 되감김 규칙은 되감긴 **다음** 문단(pi=103)에만 걸리고, 그마저
            // 그 문단이 잔여에 들어가면 분할 루프가 그대로 현재 쪽에 놓는다. 넘긴
            // 주체인 pi=102 자신은 표 경로라 그 판정을 아예 지나지 않는다.
            if st.col_count == 1 && !st.current_items.is_empty() {
                let own_stored_vpos = para
                    .line_segs
                    .iter()
                    .find(|seg| !is_synthetic_line_seg(seg))
                    .map(|seg| seg.vertical_pos);
                let next_stored_vpos = paragraphs
                    .get(para_idx + 1)
                    .and_then(|next| {
                        next.line_segs
                            .iter()
                            .find(|seg| !is_synthetic_line_seg(seg))
                    })
                    .map(|seg| seg.vertical_pos);
                if let (Some(own), Some(next)) = (own_stored_vpos, next_stored_vpos) {
                    // ① **표를 단 문단**만. #3837 되감김 규칙이 닿지 못하는 계보가
                    //    정확히 이것이고(표 경로는 그 판정을 지나지 않는다), 위 네
                    //    문서의 후보 52곳 중 표를 단 문단은 sample16 한 곳뿐이다.
                    let hosts_table = para
                        .controls
                        .iter()
                        .any(|c| matches!(c, crate::model::control::Control::Table(_)));
                    // ② 저장 자리가 본문 바닥을 **근소하게** 넘을 것. 크게 넘는 사다리는
                    //    쪽 리셋이 아니라 구역 누적 좌표계라 판정의 전제가 깨진다
                    //    (sample16 pi=738: 3567.4px / 가용 971.3px).
                    let own_px = hwpunit_to_px(own, self.dpi);
                    let body_bottom = st.base_available_height();
                    let overflows_body_narrowly =
                        own > 0 && own_px > body_bottom && own_px - body_bottom <= MIN_TOP_KEEP_PX;
                    // ③ 이 쪽에 조각이 될 만한 잔여가 남아 있을 것. 잔여가 그보다 작으면
                    //    통상 fit 이 어차피 다음 쪽으로 넘긴다 — 거기서 또 끊으면 빈 쪽이
                    //    하나 더 생긴다(3075729 #1880: 잔여 10.4px, 13쪽 → 14쪽).
                    let page_room_left = body_bottom - st.current_height;
                    if hosts_table
                        && overflows_body_narrowly
                        && next < own
                        && page_room_left > MIN_TOP_KEEP_PX
                    {
                        if std::env::var("RHWP_DIAG_VPOS_OVF").is_ok() {
                            eprintln!(
                                "[VPOS_OVF] pi={} own={} ({:.1}px) next={} avail={:.1} cur_h={:.1} items={}",
                                para_idx,
                                own,
                                own_px,
                                next,
                                body_bottom,
                                st.current_height,
                                st.current_items.len()
                            );
                        }
                        st.advance_column_or_new_page();
                    }
                }
            }
            // [#4533 HWP3] 자리차지 밴드 비예약 판별용 — 표 경로 포함 전 문단 공통.
            st.next_para_first_stored_vpos = paragraphs.get(para_idx + 1).and_then(|p| {
                p.line_segs.first().map(|seg| {
                    p.source_line_seg_vertical_pos
                        .as_ref()
                        .and_then(|source| source.first().copied())
                        .unwrap_or(seg.vertical_pos)
                })
            });
            st.next_para_is_empty_float_table_anchor = paragraphs
                .get(para_idx + 1)
                .is_some_and(|p| para_is_empty_topbottom_table_anchor(p));
            st.next_para_is_plain_text = paragraphs
                .get(para_idx + 1)
                .is_some_and(|p| para_has_non_whitespace_text(p) && p.controls.is_empty());
            if std::env::var("RHWP_FLOW_DBG").is_ok() {
                eprintln!(
                    "FLOW_DBG pi={} page={} cur_h={:.1}",
                    para_idx,
                    st.pages.len(),
                    st.current_height
                );
            }
            // 표 컨트롤 감지
            let has_table = self.paragraph_has_table(para);
            if std::env::var("RHWP_DIAG_FLOW").is_ok() {
                eprintln!("DIAG_ROUTE pi={} has_table={}", para_idx, has_table);
            }

            // [Task #702] 새 ColumnDef 검출. shortcut.hwp p2/p3 파일/미리보기/편집 등은
            // [쪽나누기]+단정의:1단 (header) → [단나누기]+단정의:2단 (content) 패턴 사용.
            // [다단나누기] 외에도 Page/Column break 의 ColumnDef 차이도 zone 재정의 신호로 인식.
            let new_col_def_opt: Option<ColumnDef> = para.controls.iter().find_map(|c| {
                if let Control::ColumnDef(cd) = c {
                    Some(cd.clone())
                } else {
                    None
                }
            });
            let has_diff_col_def = new_col_def_opt
                .as_ref()
                .map(|cd| {
                    cd.column_count.max(1) != st.col_count
                        || cd.column_type != st.current_zone_column_type
                })
                .unwrap_or(false);

            let is_terminal_empty_column_break = para_idx + 1 == paragraphs.len()
                && st.col_count == 1
                && para.column_type == ColumnBreakType::Column
                && !has_diff_col_def
                && para.controls.is_empty()
                && para
                    .text
                    .replace(|c: char| c.is_control(), "")
                    .trim()
                    .is_empty();
            if is_terminal_empty_column_break {
                // 1단 구역 끝의 빈 단나누기 문단은 다음 단/밴드가 없으므로
                // 별도 빈 페이지를 만들지 않는다.
                continue;
            }

            let is_overlay_guide_empty_para = st.col_count > 1
                && para.column_type == ColumnBreakType::None
                && para.controls.is_empty()
                && !para_has_visible_text(para)
                && para
                    .line_segs
                    .first()
                    .is_some_and(|seg| seg.vertical_pos > 0)
                && (0..para_idx)
                    .rev()
                    .find(|&idx| {
                        paragraphs
                            .get(idx)
                            .is_some_and(|p| !p.controls.is_empty() || para_has_visible_text(p))
                    })
                    .and_then(|idx| paragraphs.get(idx))
                    .is_some_and(para_is_non_tac_overlay_table_anchor);
            if is_overlay_guide_empty_para {
                // 글앞/글뒤 표 뒤의 빈 guide 줄은 떠 있는 개체의 위치 보조값이며,
                // 다단 flow 높이를 소비하지 않는다.
                continue;
            }

            let is_native_hwp5_figure_table_overlay_guide_empty = profile
                .hwp5_stored_pagination_layout()
                && native_hwp5_figure_table_overlay_guide_empty(para_idx, para, paragraphs);
            if is_native_hwp5_figure_table_overlay_guide_empty {
                // `그림 67` 같은 2×1 그림 표는 선언한 표 높이로 이미 flow를 예약한다.
                // 표 paint span 내부의 빈 HWP line은 다시 높이를 소비하면 안 되지만,
                // PI↔page 진단에서 문단 자체는 계속 추적 가능해야 하므로 0-height item으로
                // 남긴다.
                st.hidden_empty_paras.insert(para_idx);
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
                continue;
            }

            // 다단 나누기
            if para.column_type == ColumnBreakType::MultiColumn {
                self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
            }

            // 단 나누기
            // [#2019 ②] 부동 개체 전용 빈 앵커 문단(별지 서식)의 단나누기(새 ColumnDef 없음)는
            // 무시한다. 흐름 텍스트가 없고 개체가 절대위치 배치이므로, 단일 단에서 단나누기를
            // 페이지 분할로 변환하면 한글과 달리 폼 요소마다 새 쪽이 생겨 과분할된다(74312).
            // 새 ColumnDef 를 동반하는 단나누기(zone 전환)는 ③(process_multicolumn_break)에서 처리.
            //
            // 추가로, 별지 서식은 폼 사이를 "빈 문단 + 단나누기 + (같은 1단) ColumnDef" 구분자로
            // 나눈다. 이 구분자는 has_diff_col_def=false(단 수 불변)라 위 branch 를 타는데, 단일
            // 단에서 페이지 분할로 변환되어 폼마다 허위 페이지가 생긴다. 텍스트 없이 ColumnDef
            // 만 든 단일 단 단나누기도 억제한다(한글은 이 구분자로 쪽을 나누지 않음).
            let columndef_only_break = para_is_columndef_only_separator(para);
            let empty_columndef_only_break =
                !has_diff_col_def && st.col_count <= 1 && columndef_only_break;
            // [#2019 v3] 별지 서식은 같은 2단 ColumnDef 를 중간중간 반복해 부동
            // overlay 글상자 묶음을 구분한다. 앞뒤가 모두 부동 overlay 앵커이면
            // 실제 텍스트 column advance 로 보지 않는다.
            let overlay_columndef_separator_break = !has_diff_col_def
                && columndef_only_break
                && columndef_separator_between_floating_overlays(para_idx, paragraphs);
            let suppress_floating_anchor_column_break = !has_diff_col_def
                && (crate::renderer::layout::para_is_floating_overlay_anchor(para)
                    || empty_columndef_only_break
                    || overlay_columndef_separator_break
                    || (profile.hwpx_stored_layout()
                        && empty_table_carrier_column_break_before_page_table(
                            para_idx, para, paragraphs,
                        )));
            if para.column_type == ColumnBreakType::Column && !suppress_floating_anchor_column_break
            {
                if has_diff_col_def {
                    // [Task #702] 단나누기 + 새 ColumnDef = zone 재정의 (MultiColumn 등가 처리)
                    self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
                } else if !st.current_items.is_empty() {
                    // [Task #846] 마지막 단에서 명시적 단나누기 → 새 페이지가 아니라 같은
                    // col_count 로 같은 페이지에 새 단-밴드를 시작 (들어갈 공간이 있으면). ≈ #768.
                    // [Task #849] 단, 이는 "배분"(Distribute) 단에서만. "일반"(Normal/신문형)
                    // 단에서 마지막 단의 단나누기는 같은 페이지 새 밴드를 만들지 않는다 (기존 동작).
                    // [Task #866] shortcut.hwp 3쪽 "<편집 화면 분할에서>" pi=94 회귀 수정.
                    let is_last_column = st.current_column + 1 >= st.col_count;
                    if is_last_column
                        && st.col_count > 1
                        && st.current_zone_column_type == ColumnType::Distribute
                    {
                        self.start_new_column_band(&mut st, para_idx, paragraphs);
                    } else {
                        st.advance_column_or_new_page();
                    }
                }
            }

            // 쪽 나누기
            let force_page_break = para.column_type == ColumnBreakType::Page
                || para.column_type == ColumnBreakType::Section;
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let para_style_break = para_style.map(|s| s.page_break_before).unwrap_or(false);

            // [Task #1007/#1035 → #1042 narrow v2] Cross-paragraph vpos reset 감지 —
            // heading paragraph (text 있음 + spacing_before ≥ 500 HU + paragraph local
            // vpos reset) 만 인정. content paragraph (spacing_before < 500) 는 skip.
            // sample16-2024 pi=162 (heading, sb=852, vpos=852) trigger ✓
            // sample16-2022 pi=87 (빈 문단, text_len=0) skip ✓
            // sample16-2022 pi=118 (content, sb=284) skip ✓
            // sample16-2022 pi=316 (content, sb=0) skip ✓
            let variant_vpos_reset_break = profile.hwp3_layout()
                && self.judge_hwp3_variant_vpos_reset_break(
                    para,
                    paragraphs,
                    styles,
                    para_idx,
                    body_height_hu_for_variant,
                    variant_prev_para_idx,
                );

            // [#1956] 명시적 쪽나누기 문단부터는 wrap 밴드 무효 — 새 쪽에는 anchor
            // 개체가 없으므로 후속 문단을 옆에 흡수하면 안 된다. current_items 가 비어
            // 있어 force_new_page 를 생략하는 경우에도 사용자 의도(새 쪽)는 동일하므로
            // 밴드는 해제한다. (법령안 신구조문대비표: 쪽나누기 표제가 흡수되어 pi 6쪽 이탈)
            if (force_page_break || para_style_break) && st.wrap_around_cs >= 0 {
                st.wrap_around_cs = -1;
                st.wrap_around_sw = -1;
                st.wrap_around_any_seg = false;
                st.wrap_around_derived_band = false;
                st.close_square_band();
            }
            // [#1955] 명시적 쪽나누기부터는 글뒤로 표 후행 흡수도 해제 (사용자 의도 새 쪽).
            if force_page_break || para_style_break {
                st.behind_float_table_para = None;
            }

            if (force_page_break || para_style_break || variant_vpos_reset_break)
                && !st.current_items.is_empty()
            {
                st.force_new_page();
                // [Task #702] 쪽나누기 + 새 ColumnDef = 새 페이지에서 col 정의 적용
                if has_diff_col_def {
                    if let Some(cd) = &new_col_def_opt {
                        st.col_count = cd.column_count.max(1);
                        let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                        st.current_zone_layout = Some(new_layout.clone());
                        st.layout = new_layout;
                        st.current_zone_column_type = cd.column_type;
                        // [Task #853] 새 페이지 첫 zone: 디자인 spacing /2 (위쪽 절반)만 추가.
                        // (이전 zone 은 이전 페이지에 있었으므로 아래쪽 절반은 더하지 않음.)
                        let new_ds = column_def_design_spacing_px(cd, self.dpi);
                        st.current_zone_y_offset += new_ds / 2.0;
                        st.current_zone_design_spacing_px = new_ds;
                    }
                }
            }

            // HWP5-origin 문서는 section PageHide를 연 빈 marker와, 같은 쪽 장식 host의
            // Page break를 함께 기록할 수 있다. marker의 break는 적용하되 marker
            // 자체를 배치하지 않아 host가 그 새 쪽을 바로 소유하도록 한다.
            if (profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
                && hwp5_origin_redundant_pagehide_break_marker(
                    para_idx,
                    para,
                    paragraphs,
                    profile.hwpx_stored_layout(),
                )
            {
                st.hidden_empty_paras.insert(para_idx);
                continue;
            }

            // [Task #1046] 사후 reflow 이월: layout 에서 본문 하단 overflow 로 판정된 항목은
            // 현재 페이지에 렌더링하지 않고 다음 페이지로 넘긴다. force_break_before 에 등록된
            // para_idx 가 현재 페이지에 이미 항목이 있으면 새 페이지를 강제 (force_page_break 등가).
            // 빈 셋(reflow hint 없음)이면 무동작 → 기존 출력 불변.
            if force_break_before.contains(&para_idx) && !st.current_items.is_empty() {
                st.force_new_page();
            }

            if para_is_pre_paper_page_square_table_scaffold(para_idx, paragraphs)
                || para_is_post_paper_page_square_table_scaffold(para_idx, paragraphs)
            {
                // [#2019 v3] Paper/Page 기준 Square 표를 둘러싼 빈 스캐폴드 문단.
                // 쪽나누기/ColumnDef 효과는 위에서 적용하되, PDF 기준으로는 별도 빈 줄을
                // 렌더하거나 본문 흐름 높이를 소비하지 않는다.
                st.hidden_empty_paras.insert(para_idx);
                continue;
            }

            // Task #321: 문단간 vpos-reset 기반 강제 분할
            // HWP LINE_SEG의 vertical_pos는 페이지 내 흐름 y 좌표.
            // 현재 문단 first_vpos=0이고 직전 문단이 같은 단에 있으며 last_vpos가 충분히 큰 경우,
            // HWP가 pi 경계에서 페이지/단 분할을 의도한 것 → 강제 분할.
            // [Task #362] wrap-around zone 활성 중에는 vpos-reset 가드 무시 (기존).
            // [Task #724] vpos-reset trigger 발동 시 wrap_around 강제 종료 (신규):
            // HWP5 변환본 case 에서 paragraph 442/443 wrap_around 매칭 후 후속 paragraph
            // (예: 599) vpos=0 시점에도 wrap_around active 유지되어 페이지 분할 위반 →
            // vpos-reset trigger 시 wrap_around 강제 종료 + advance_column_or_new_page.
            if para_idx > 0 && !st.current_items.is_empty() {
                let prev_para = &paragraphs[para_idx - 1];
                let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
                let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
                if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
                    // 현재 문단의 vpos가 직전 문단의 마지막 vpos보다 작은 경우 — 컬럼/페이지 reset 시그널.
                    // - 단일 단: cv == 0 만 인정 (Task #321 보수적 기준 유지).
                    //   단일 단에서 cv != 0 의 cv < pv 는 partial-table split 의 LAYOUT 잔재로
                    //   해석되어야 함 (issue #418 / hwpspec pi=78→pi=79).
                    // - 다단 Normal (NEWSPAPER): cv != 0 도 인정 (Task #470). pv > 5000 임계값 유지.
                    // - 다단 Distribute (BalancedNewspaper): 짧은 컬럼 (3+3 분배 등) 에서 pv 가
                    //   임계값 미달일 수 있어 pv > 0 으로 완화 (Task #702, shortcut 지우기 6항목 정합).
                    //   단일 단/Normal 다단은 영향 없음.
                    let is_distribute = st.col_count > 1
                        && matches!(st.current_zone_column_type, ColumnType::Distribute);
                    // [Task #853] Distribute 다단의 "1줄짜리 컬럼" 케이스: 직전 문단이
                    // 단 1줄(예: vpos=0)이고 현재 문단도 vpos=0 이면 `cv < pv` 가 0<0 으로
                    // 거짓이라 컬럼 전환을 못 잡았다(shortcut.hwp 스타일/속성 섹션). 직전 문단의
                    // vpos+line_height(=콘텐츠 끝)를 기준으로 비교하면 정상 흐름(cv=pv_end+ls≥pv_end)
                    // 은 영향 없고 reset(cv≪pv_end)만 잡힌다.
                    let prev_vpos_end = prev_para
                        .line_segs
                        .last()
                        .map(|s| s.vertical_pos.saturating_add(s.line_height))
                        .unwrap_or(pv);
                    // [Task #1086 Stage 3] HWP3-origin page tolerance 대상 문서는
                    // 새 페이지 첫 문단을 vpos=0 이 아니라 200/500HU 근방으로
                    // 인코딩하기도 한다(hwpspec.hwp s2:pi=89, pi=104). 단일 단에서
                    // 모든 cv<pv 를 reset 으로 보면 일반 직접 작성 HWP(2022년
                    // 국립국어원), partial-table 직후 정상 흐름(hwpspec pi=78→79),
                    // 표 host 문단(hwpspec pi=57)을 깨므로, 비영 near-top reset 은
                    // 직전 문단이 페이지 하단부에 있고 대상이 텍스트/그림-only 문단일
                    // 때만 인정한다.
                    // 그림만 든 빈 문단은 한컴이 조금 더 일찍 새 페이지로 넘기는 케이스
                    // (hwpspec.hwp s3:pi=93)가 있어 표/텍스트보다 낮은 하단 기준을 쓴다.
                    let shape_only_para = para.text.trim().is_empty()
                        && !para.controls.is_empty()
                        && para
                            .controls
                            .iter()
                            .all(|c| matches!(c, Control::Picture(_) | Control::Shape(_)));
                    let has_table_control =
                        para.controls.iter().any(|c| matches!(c, Control::Table(_)));
                    let near_page_top_reset = hwp3_origin_page_tolerance
                        && cv > 0
                        && ((shape_only_para && cv <= 200 && prev_vpos_end > 52_000)
                            || (!shape_only_para
                                && !has_table_control
                                && cv <= 500
                                && prev_vpos_end > 60_000));
                    let para_sb_hu_for_reset = para_style
                        .map(|s| (s.spacing_before * 7200.0 / 96.0) as i32)
                        .unwrap_or(0);
                    // [#1921 d=-1] 네이티브 HWP5/HWPX 의 비영 near-top reset:
                    // 한글은 새 쪽 첫 줄의 stored vpos 를 0 이 아니라 해당 문단의
                    // spacing_before 로 기록하기도 한다 (조문별/규제영향분석서 클래스,
                    // 예: 86034 pi24 vpos=500 = sb 6.7px). cv==0 전용 트리거가 이를
                    // 놓쳐 한 쪽을 과적한다. cv≈sb(±150HU) + 쪽 하단(prev_vpos_end
                    // > 60_000) + 텍스트 전용 문단으로 한정해 partial-table 잔재
                    // (#418)·표 host(#1086 주석) 케이스와 구분한다.
                    let native_near_top_reset = !hwp3_origin_page_tolerance
                        && cv > 0
                        // [#2136] 상한 2000→2500: sb=5000유닛(=2500HU) 문단의 저장
                        // 리셋(cv=2500=sb 정확 일치)이 500HU 차로 배제되어 측정 fit
                        // 과적(148753276 pi46: used 942px > body 933.6px, 한글 p5).
                        // #1750 split-precheck 상한(2500)과 정합. sb 일치 ±150 조건이
                        // 유지되어 오발동 억제.
                        && cv <= 2500
                        && para_sb_hu_for_reset > 0
                        && (cv - para_sb_hu_for_reset).abs() <= 150
                        && !shape_only_para
                        && !has_table_control
                        && para_has_visible_text(para)
                        && prev_vpos_end > 60_000
                        // [#5941 축 B] `#5921` 의 완화("이번 쪽 잔여에 들어가면 저장
                        // near-top 리셋을 버린다")를 **네이티브 HWP5 저장 조판 프로파일의
                        // 이미 찬 쪽**에는 걸지 않는다. 그 프로파일에서 저장 사다리는 작성
                        // 엔진의 쪽 배분 자체이고, "이번 잔여에 들어간다" 는 사실이 그
                        // 기록을 이기지 못한다.
                        //
                        // 실측 — 완화가 필요한 문서는 전부 HWPX 컨테이너이거나, HWP5 라도
                        // **쪽이 사실상 비어 있다**:
                        //
                        // ```text
                        //   neartop_reset_sb2500.hwpx        hwpx   채움  2%   #5921 원 픽스처
                        //   issue1880_anchor_stack_sb…hwpx   hwpx   채움 96%   #6063 핀
                        //   hwp3-sample16-hwp5.hwpx          hwpx   채움 70~100%  off_canvas 래칫
                        //   issue5701 …tac_reset_tail.hwp    hwp5   채움  6%   ← 빈 쪽 갈래
                        //   1480000-201900698.hwp            hwp5   채움 74~100% ← 여기만 완화 금지
                        // ```
                        //
                        // 그 마지막 문서는 완화 때문에 리셋 3개가 지워져 202 → 200 이 됐고
                        // 한/글 2024(205)에서 더 멀어졌다(`#5941` 축 B bisect: `12074fbea`).
                        && ((profile.hwp5_stored_pagination_layout()
                            && !near_top_reset_page_is_barely_started(
                                st.current_height,
                                st.available_height(),
                            ))
                            || native_near_top_reset_exceeds_remaining(
                                para,
                                para_sb_hu_for_reset,
                                st.current_height,
                                st.available_height(),
                                self.dpi,
                            ));
                    let next_heading_after_top_content_reset =
                        paragraphs.get(para_idx + 1).is_some_and(|next_para| {
                            let next_sb_hu = styles
                                .para_styles
                                .get(next_para.para_shape_id as usize)
                                .map(|ps| (ps.spacing_before * 7200.0 / 96.0) as i32)
                                .unwrap_or(0);
                            next_para.line_segs.len() >= 2
                                && next_para
                                    .line_segs
                                    .first()
                                    .filter(|ls| !is_synthetic_line_seg(ls))
                                    .is_some_and(|ls| {
                                        ls.vertical_pos > 0 && ls.vertical_pos <= 2500
                                    })
                                && next_para.controls.is_empty()
                                && para_has_visible_text(next_para)
                                && next_sb_hu >= 500
                        });
                    let hwp3_content_vpos_zero_reset = profile.hwp3_layout()
                        && st.col_count == 1
                        && cv == 0
                        && prev_vpos_end > body_height_hu_for_variant * 70 / 100
                        && para_sb_hu_for_reset < 250
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && next_heading_after_top_content_reset;
                    // [#5907] 앞뒤 문단이 둘 다 stored vpos 0 을 주장하는 충돌.
                    // `pv > 5000` 기준은 "직전 문단이 쪽 하단부에 있었다"를 전제하는데,
                    // 한/글이 짧은 문단 하나만 올리고 쪽을 넘긴 경우 pv 는 0 이라 침묵한다.
                    // 같은 단 기하 + 어울림 개체 없음 + 문단 자체 나누기 없음으로 좁힌다.
                    let stored_top_collision_reset = st.col_count == 1
                        && cv == 0
                        && para.column_type == ColumnBreakType::None
                        && st.wrap_around_cs < 0
                        && !para_is_page_bottom_fixed_table_anchor(para)
                        && stored_vpos_top_collision(prev_para, para);
                    let trigger = if st.col_count > 1 {
                        if is_distribute {
                            cv < prev_vpos_end && prev_vpos_end > 0
                        } else {
                            cv < pv && pv > 5000
                        }
                    } else {
                        (cv == 0
                            && pv > 5000
                            && !hwp3_content_vpos_zero_reset
                            && !para_is_page_bottom_fixed_table_anchor(para)
                            // [#6535 잔여] 쪽-앵커 블록의 vpos=0 은 절대배치 산물이지
                            // 쪽 리셋 신호가 아니다.
                            && !para_hosts_page_anchored_block(para))
                            || near_page_top_reset
                            || native_near_top_reset
                            || stored_top_collision_reset
                    };
                    // [#2279 OMIT-fit] spacing-누락 문서군에서 fresh 재계산이 직전
                    // 쪽에서 밀어낸 빈 문단만 담긴 쪽에는 저장 리셋 경계를 적용하지
                    // 않는다 — 한글 fresh 는 그 빈 문단을 다음 쪽 상단에 두고
                    // 본문을 이어 붙인다 (36392557 pi14+pi15: 한글 PDF p3 상단
                    // 36px = pi14 자리, 단독 쪽 아님).
                    let omit_pushed_empty_page = st.omit_fresh_recalc_doc
                        && st.current_items.iter().all(|item| {
                            page_item_para_index(item)
                                .and_then(|idx| paragraphs.get(idx))
                                .is_some_and(|p| p.controls.is_empty() && p.text.trim().is_empty())
                        });
                    // [compat 2024] (Δ1) 이번 단에서 자리차지 표 앵커 줄을 회수
                    // 했거나 앞선 경계를 이미 덮어 저장 경계가 한 발씩 어긋난
                    // 상태이고, 이 문단의 **첫 줄**이 잔여 공간에 들어가면 저장
                    // vpos 리셋(=2022 조판의 쪽 경계)을 존중하지 않는다 — 문단이
                    // 통째로 못 들어가면 일반 분할 경로가 첫 줄부터 채운다(한글
                    // 2024 의 fresh 흐름과 같은 결정). 회수도 선행 덮음도 없는
                    // 문서/쪽에서는 종전 동작 그대로.
                    let hangul2024_refit =
                        st.profile.hangul2024_layout() && st.hangul2024_reclaimed > 0.0 && {
                            // 빈 문단(텍스트·컨트롤 없음)은 한글이 쪽 하단
                            // 여백으로 흘려도 되는 존재라 need=0, 실문단은 첫 줄
                            // (통째로 못 들어가면 일반 분할이 첫 줄부터 채운다 —
                            // 한글 2024 fresh 와 같은 결정). 회귀를 만들던 것은
                            // 이 완화가 아니라 sticky 연쇄였음(라운드 B/C 회귀값
                            // 동일로 판별).
                            let need: f64 =
                                if !para_has_visible_text(para) && para.controls.is_empty() {
                                    0.0
                                } else {
                                    para.line_segs
                                        .first()
                                        .map(|s| {
                                            hwpunit_to_px(
                                                s.line_height.saturating_add(s.line_spacing),
                                                self.dpi,
                                            )
                                        })
                                        .unwrap_or(0.0)
                                };
                            st.current_height + need
                                <= st.available_height() + st.hangul2024_reclaimed
                        };
                    if std::env::var("RHWP_DIAG_COMPAT24").is_ok() && trigger {
                        eprintln!(
                            "DIAG_COMPAT24 reset-trigger pi={para_idx} refit={hangul2024_refit} \
                             cur={:.1} reclaimed={:.1}",
                            st.current_height, st.hangul2024_reclaimed,
                        );
                    }
                    if trigger
                        && hangul2024_refit
                        && !para_has_visible_text(para)
                        && para.controls.is_empty()
                    {
                        st.hangul2024_spill_para = Some(para_idx);
                    }
                    // [#5919] [#2019 v3] 로 명시 단나누기를 억제한 ColumnDef-only
                    // overlay 구분자의 lineseg vertpos=0 은 쪽/단 경계 신호가 아니라
                    // 마커 문단의 미설정값이다. 이 경계를 저장 vpos 리셋으로 다시
                    // 읽으면 억제했던 단나누기가 되살아나 표 격자와 본문을 서로
                    // 다른 허위 쪽으로 갈라 놓는다(74312 12쪽).
                    // [편집 세션] 저장 vpos 리셋은 저장 시점의 쪽 경계 신호라
                    // 편집으로 앞 내용이 늘거나 줄면 낡은 좌표다. 다만 무조건
                    // 무시하면 이 쪽 잔여가 몇십 px 뿐일 때 다음 쪽에서 시작하던
                    // 표의 머리 행 조각이 잔여에 낑겨 앞 문단과 겹친다(한글
                    // 오라클: 잔여가 작으면 표는 통째로 다음 쪽 유지). 그래서
                    // 표 host 문단은 잔여에 표 머리(첫 두 행)가 실제로 들어갈
                    // 때만 낡은 경계를 무시하고 fresh fit 에 맡긴다.
                    let session_stale_reset_override =
                        self.profile.get().session_edited() && !st.current_items.is_empty() && {
                            let first_table_head_px =
                                para.controls
                                    .iter()
                                    .enumerate()
                                    .find_map(|(ci, c)| match c {
                                        Control::Table(_) => measured_tables
                                            .iter()
                                            .find(|m| {
                                                m.para_index == para_idx && m.control_index == ci
                                            })
                                            .map(|m| m.row_heights.iter().take(2).sum::<f64>()),
                                        _ => None,
                                    });
                            // 표 없는 문단의 저장 리셋은 단/쪽 경계 인코딩
                            // (#2299 shortcut.hwp: 리셋 76곳 = 다단 밴드)일 수
                            // 있어 존중한다 — 무시 대상은 편집으로 성장하는 표
                            // host 문단의 낡은 경계뿐이다.
                            match first_table_head_px {
                                Some(head) => {
                                    st.current_height + head <= st.available_height() + 0.5
                                }
                                None => false,
                            }
                        };
                    let trigger = trigger
                        && !omit_pushed_empty_page
                        && !hangul2024_refit
                        && !overlay_columndef_separator_break
                        && !session_stale_reset_override;
                    if trigger {
                        // [Task #724] wrap_around active 시 강제 종료 — anchor cs=0
                        // (HWP5 변환본 caption-style) 한정. 일반 wrap_around (anchor cs>0)
                        // 는 기존 동작 (Task #362 vpos-reset 무시) 유지.
                        if st.wrap_around_cs == 0 {
                            st.wrap_around_cs = -1;
                            st.wrap_around_sw = -1;
                            st.wrap_around_any_seg = false;
                            st.wrap_around_derived_band = false;
                            st.close_square_band();
                        }
                        if st.wrap_around_cs < 0 {
                            // [#5918] 저장 리셋(cv==0/near-top)이 가리키는 쪽 경계를
                            // block-table continuation 꼬리 조각이 이미 열어 놨으면
                            // 이중으로 쪽을 넘기지 않는다. RowBreak 표의 마지막 조각은
                            // 저장 경계와 무관하게 "앞 조각이 못 들어간" 시점에 새 쪽
                            // 상단에 배출되는데, 그 새 쪽이 곧 저장 사다리의 리셋 지점과
                            // 같은 물리 경계인 경우가 있다(sample1-repro pi=578 꼬리 조각
                            // 쪽 == pi=608 vpos=0 리셋 쪽). 이때 리셋이 한 번 더
                            // advance하면 조각만 남은 근빈 쪽이 생기고 뒤따르는 표가
                            // 남은 공간으로 흐르지 못한다. 현재 쪽이 continuation
                            // 조각(들)과 빈 필러 문단만 담고 있을 때만 건너뛴다 — 실
                            // 내용이 함께 놓인 쪽의 저장 경계는 그대로 존중한다.
                            if !page_holds_only_fresh_table_continuation(&st, paragraphs) {
                                // [#6146] 저장 리셋은 이 문단의 **줄**이 다음 쪽이라는
                                // 신호지 자리차지 밴드까지 옮기라는 신호가 아니다.
                                // 한글은 밴드를 떠나는 쪽의 흐름 말미에 남기고 본문 아래
                                // 여백으로 흘린다.
                                st.spill_page_tail_floats(para_idx, para);
                                st.advance_column_or_new_page();
                            }
                        }
                    }
                }
            }

            // [Task #359] 단독 항목 페이지 차단:
            // 다음 pi 가 vpos-reset 가드를 발동할 예정이고 현재 pi 가 잔여 공간 부족으로
            // 새 페이지를 시작하면 단독 항목 페이지가 발생.
            //   - 현재 pi 가 빈 문단이면: skip (한컴은 표시하지 않음)
            //   - 현재 pi 가 일반 텍스트이면: fit 안전마진 (10px) 1회 비활성화
            //     (kps-ai pi=317 case: 0.x px 차이로 fit 실패하여 단독 페이지 35 발생)
            // 가드 제외 조건:
            //   - 다음 pi 가 force_page_break (column_type==Page/Section) 인 경우 발동 안 함
            //     (정상 쪽나누기 신호 — 단독 페이지 발생 안 함, hwp-multi-001 회귀 차단)
            let next_will_vpos_reset = if (!st.current_items.is_empty() || !st.pages.is_empty())
                && para_idx + 1 < paragraphs.len()
            {
                let next_para = &paragraphs[para_idx + 1];
                let next_force_break = next_para.column_type == ColumnBreakType::Page
                    || next_para.column_type == ColumnBreakType::Section;
                if next_force_break {
                    false
                } else {
                    // [Task #470] 다단 섹션에서는 nv == 0 → nv < cl 로 완화 (컬럼 헤더 오프셋).
                    // 단일 단에서는 partial-table split 회귀 (issue #418) 회피 위해 nv == 0 유지.
                    paragraph_saved_vpos_reset_starts_new_page_after(
                        para,
                        next_para,
                        st.col_count,
                        st.profile.hwp3_layout(),
                    )
                }
            } else {
                false
            };

            // [Task #1725 v2] 현재 일반텍스트 tail 과 새 페이지(vpos-reset) 사이에 빈 문단이 1개
            // 끼어 immediate-next reset 을 놓치는 각주-tail 케이스(국제고속선기준 pi=537/2378).
            // 이 경우도 tail 로 보고 fit 완화 플래그(각주 버퍼/안전마진)만 켠다. 빈 문단 skip
            // 로직(아래 next_will_vpos_reset 분기)에는 영향 없음.
            let tail_before_break_through_empty = !next_will_vpos_reset
                && !st.current_items.is_empty()
                && para_has_visible_text(para)
                && para.controls.is_empty()
                && para_idx + 2 < paragraphs.len()
                && {
                    let mid = &paragraphs[para_idx + 1];
                    let after = &paragraphs[para_idx + 2];
                    mid.text.trim().is_empty()
                        && mid.controls.is_empty()
                        && after.column_type != ColumnBreakType::Page
                        && after.column_type != ColumnBreakType::Section
                        && paragraph_saved_vpos_reset_starts_new_page_after(
                            mid,
                            after,
                            st.col_count,
                            st.profile.hwp3_layout(),
                        )
                };
            if tail_before_break_through_empty {
                st.skip_safety_margin_once = true;
                st.skip_footnote_margin_once = true;
                st.tail_saved_bounds_once = paragraph_saved_visible_bounds(para, 0, self.dpi);
            }

            // [Task #1733] 페이지 하단 빈 줄이 다음 vpos-reset 흐름 앞에 1개 이상 끼는 경우.
            // 기존 가드는 "현재 빈 문단 바로 다음이 reset" 인 경우만 흡수한다. 국제고속선기준은
            // 빈 줄 2개 뒤 본문이 새 쪽 상단으로 reset 되거나, 빈 줄 뒤 하단 제목 1줄이 있고
            // 그 다음 본문이 reset 되는 형태가 있어 near-empty 페이지가 남는다. 현재 빈 문단이
            // 이미 페이지 하단 vpos 를 가지고 있고, 뒤쪽 저장 flow 가 reset 을 명확히 보일 때만
            // 0-높이로 흡수한다.
            let empty_tail_bridge_to_reset = !next_will_vpos_reset
                && !st.current_items.is_empty()
                && !st
                    .current_items
                    .iter()
                    .any(|item| matches!(item, PageItem::PartialTable { .. }))
                && para.text.is_empty()
                && para.controls.is_empty()
                && para.line_segs.first().is_some_and(|seg| {
                    let body_h_hu =
                        crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
                    seg.vertical_pos > body_h_hu * 70 / 100
                })
                && {
                    let mut idx = para_idx + 1;
                    let mut prev = para;
                    let mut found = false;
                    while let Some(next_para) = paragraphs.get(idx) {
                        if next_para.text.is_empty() && next_para.controls.is_empty() {
                            prev = next_para;
                            idx += 1;
                            continue;
                        }

                        let reset_after_empty_run =
                            paragraph_saved_vpos_reset_starts_new_page_after(
                                prev,
                                next_para,
                                st.col_count,
                                st.profile.hwp3_layout(),
                            );
                        let high_tail_heading_then_reset = para_has_visible_text(next_para)
                            && next_para.controls.is_empty()
                            && next_para.line_segs.first().is_some_and(|seg| {
                                let body_h_hu = crate::renderer::px_to_hwpunit(
                                    st.layout.body_area.height,
                                    self.dpi,
                                );
                                seg.vertical_pos > body_h_hu * 70 / 100
                            })
                            && paragraphs.get(idx + 1).is_some_and(|after| {
                                after.column_type != ColumnBreakType::Page
                                    && after.column_type != ColumnBreakType::Section
                                    && paragraph_saved_vpos_reset_starts_new_page_after(
                                        next_para,
                                        after,
                                        st.col_count,
                                        st.profile.hwp3_layout(),
                                    )
                            });

                        found = reset_after_empty_run || high_tail_heading_then_reset;
                        break;
                    }
                    found
                };
            if empty_tail_bridge_to_reset {
                st.hidden_empty_paras.insert(para_idx);
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
                continue;
            }

            if next_will_vpos_reset {
                // [Task #362] 빈 paragraph 가 표/도형/그림 컨트롤을 포함하면 skip 안 함
                // (kps-ai pi=778 case: 빈 텍스트 + 3x3 wrap=Square 표를 가진 paragraph 가
                //  잘못 skip 되어 표 누락).
                let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
                if is_empty_no_ctrl {
                    // 페이지 하단 vpos 를 가진 빈 문단이 다음 vpos=0 본문을 잇는 경우.
                    // fit 가능으로 emit 하더라도 뒤쪽 overflow 방어에서 빈 문단 단독 페이지로
                    // 이동할 수 있으므로, fit 판정 전에 0-높이로 흡수한다.
                    let page_bottom_empty_reset_bridge =
                        para.line_segs.first().is_some_and(|seg| {
                            let body_h_hu = crate::renderer::px_to_hwpunit(
                                st.layout.body_area.height,
                                self.dpi,
                            );
                            seg.vertical_pos > body_h_hu * 70 / 100
                        });
                    if page_bottom_empty_reset_bridge {
                        st.hidden_empty_paras.insert(para_idx);
                        if !st.current_items.is_empty() {
                            st.current_items.push(PageItem::FullParagraph {
                                para_index: para_idx,
                            });
                        }
                        continue;
                    }

                    // [#1648] 빈 문단이 현재 페이지에 들어가면 정상 배치한다(한글 동작 —
                    //   페이지 하단에 빈 줄 1개). 들어가지 않을 때만 skip 하여 단독 빈 페이지를
                    //   차단한다. 종전엔 fit 무검사로 fit 하는 빈 문단까지 드롭하여, 페이지를
                    //   채운 TAC 표 직후의 빈 문단이 누락되고 페이지↔PI 가 한글과 어긋났다
                    //   (#1643 rhwp_pNone).
                    //
                    // 1) height fit: 합산 current_height 기준 (종전 #1648 판정).
                    let empty_h_px = para
                        .line_segs
                        .first()
                        .map(|s| {
                            hwpunit_to_px(
                                (s.line_height.saturating_add(s.line_spacing)) as i32,
                                self.dpi,
                            )
                        })
                        .unwrap_or(0.0);
                    let height_fits = empty_h_px <= st.available_height() - st.current_height;

                    // 2) [#1659] vpos fit: 합산 height 는 음수 줄간격 문단에서 실제 vpos 진행을
                    //   과소평가 → 페이지 하단 빈 문단을 height fit 으로 오판 emit 하지만 placement
                    //   (아래 vpos overflow 가드, ~L2300)는 vpos overflow 로 새 페이지에 단독 배치
                    //   → 단독 빈 페이지 +1 회귀(synam-001 35→36). placement(L2333/L2339)와 동일한
                    //   page_top_vpos 기준 vpos 판정을 AND 로 더해, height·vpos 둘 다 fit 일 때만
                    //   emit. placement 가 height 기반인 다단/wrap 에선 vpos 판정을 생략(true).
                    let vpos_fits = if st.col_count == 1 && st.wrap_around_cs < 0 {
                        // 페이지 첫 실 item 의 top vpos. PartialParagraph continuation 은 원
                        //   문단의 첫 줄이 아니라 fragment 시작 줄(start_line)의 vpos 가 페이지
                        //   상단이다 → line_segs[start_line] 사용. 줄 기준 vpos 가 없는 항목
                        //   (PartialTable continuation)은 None 으로 두어 vpos 판정을 보류(height
                        //   fit 에 위임) — 잘못된 baseline 으로 skip/emit 오판 방지(#1659 리뷰).
                        let page_top_vpos = st
                            .current_items
                            .iter()
                            .find(|item| !matches!(item, PageItem::EndnoteSeparator { .. }))
                            .and_then(|item| match item {
                                PageItem::FullParagraph { para_index }
                                | PageItem::Table { para_index, .. }
                                | PageItem::Shape { para_index, .. } => paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.line_segs.first())
                                    .map(|s| s.vertical_pos),
                                PageItem::PartialParagraph {
                                    para_index,
                                    start_line,
                                    ..
                                } => paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.line_segs.get(*start_line))
                                    .map(|s| s.vertical_pos),
                                // 줄 기준 vpos 없음 → 판정 보류.
                                PageItem::PartialTable { .. }
                                | PageItem::EndnoteSeparator { .. } => None,
                            });
                        match (para.line_segs.last(), page_top_vpos) {
                            (Some(last_seg), Some(top)) => {
                                let body_h_hu = crate::renderer::px_to_hwpunit(
                                    st.layout.body_area.height,
                                    self.dpi,
                                );
                                let vpos_end =
                                    last_seg.vertical_pos.saturating_add(last_seg.line_height);
                                vpos_end <= top + body_h_hu + 283
                            }
                            // vpos 판정 불가 → 제약 없음(height fit 에 위임).
                            _ => true,
                        }
                    } else {
                        true
                    };

                    if !(height_fits && vpos_fits) {
                        // [#1706] 빈 문단이 현재 페이지에 안 들어감.
                        // 종전엔 통째로 drop(continue) → 문단이 모델에서 사라져 한글 대비
                        // 문단→페이지 매핑이 어긋났다(rhwp_pNone; 대형 TAC 표가 페이지를 채운
                        // 직후의 빈 문단). 한글은 이 빈 문단을 현재 페이지 하단의 빈 줄 1개로 유지.
                        // → drop 대신 현재 페이지에 0-높이로 흡수 기록(hide_empty_line 와 동일
                        //   시멘틱). 페이지를 advance 하지 않으므로 단독 빈 페이지 회귀(synam-001
                        //   등)는 발생하지 않는다.
                        st.hidden_empty_paras.insert(para_idx);
                        st.current_items.push(PageItem::FullParagraph {
                            para_index: para_idx,
                        });
                        continue;
                    }
                    // height·vpos 둘 다 fit → 정상 emit (아래로 진행)
                } else {
                    // 일반 텍스트 또는 컨트롤 보유: 안전마진 1회 비활성화 (단독 텍스트 페이지 차단)
                    st.skip_safety_margin_once = true;
                    // [Task #1725] 각주 있는 페이지의 tail 문단: 각주 안전마진(40px 버퍼)도 1회
                    // 비활성화. 한글은 tail 을 본문에 배치하는데 rhwp 각주 예약 버퍼가 tail 을 수 px
                    // 밀어 near-empty 페이지 over-pagination(국제고속선기준 258 vs 242) 을 만든다.
                    st.skip_footnote_margin_once = true;
                    // [Task #1725 v2] 각주 없이 페이지가 수 px over-fill 되어 tail 이 밀리는 경우도
                    // 한글은 저장 tail의 본문 하단 좌표를 유지한다. 다음 fit에서 실제
                    // 저장 bottom까지의 차이만 허용하도록 bounds를 전달한다.
                    st.tail_saved_bounds_once = paragraph_saved_visible_bounds(para, 0, self.dpi);
                }
            } else if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
                // [Task #967] 빈 paragraph 직후 force page break (쪽나누기) case 가드:
                // 빈 paragraph 가 현재 page 잔여 공간 초과 시 별도 page 분기 →
                // +1 page inflate 회귀 (sample18.hwp 의 pi=27, pi=164).
                // 한컴은 빈 paragraph 를 trailing overflow 로 흡수 + 쪽나누기로 새 page 시작.
                // next_will_vpos_reset 가드는 next_force_break 인 경우 발동 안 함
                // (hwp-multi-001 회귀 차단). 본 추가 가드는 빈 paragraph + 다음 쪽나누기
                // case 중에서 **현재 page 잔여 공간 부족 (overflow) 시에만** skip — 빈
                // paragraph 가 page 에 fit 하면 정상 emit (aift.hwp 의 18 case 회귀 방지).
                let next_para = &paragraphs[para_idx + 1];
                let next_force_break = next_para.column_type == ColumnBreakType::Page
                    || next_para.column_type == ColumnBreakType::Section;
                let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
                if next_force_break && is_curr_empty {
                    // empty paragraph 의 예상 height = first line_seg 의 lh + ls
                    let empty_h_px = para
                        .line_segs
                        .first()
                        .map(|s| {
                            hwpunit_to_px(
                                (s.line_height.saturating_add(s.line_spacing)) as i32,
                                self.dpi,
                            )
                        })
                        .unwrap_or(0.0);
                    let avail = st.available_height() - st.current_height;
                    if empty_h_px > avail {
                        // [#1706] 빈 paragraph 가 fit 안 됨.
                        // 종전엔 drop(continue) → 문단이 모델에서 사라져 한글 대비 매핑이
                        // 어긋났다(rhwp_pNone). 한컴은 이 빈 문단을 현재 page 하단의 빈 줄로
                        // 흡수(위 주석)하므로, drop 대신 현재 페이지에 0-높이로 흡수 기록한다.
                        // 페이지를 advance 하지 않으므로 단독 page 회귀(sample18 등)는 없다.
                        st.hidden_empty_paras.insert(para_idx);
                        st.current_items.push(PageItem::FullParagraph {
                            para_index: para_idx,
                        });
                        continue;
                    }
                    // fit 가능 — 정상 emit (기존 동작)
                }
            }

            // co-anchored 자리차지 표가 페이지를 가득 채운 뒤(위 orphan 가드로 통째 이월된
            // 표 등) 오는, 문서 말미의 trailing 빈 문단(텍스트·컨트롤 없음, 뒤에 가시 콘텐츠
            // 없음)이 현재 page 잔여 공간을 초과하면 새 page 를 만들지 않고 skip 한다. 한컴은
            // page 를 채운 자리차지 표 뒤의 빈 문단을 trailing overflow 로 흡수하고 빈 page 를
            // 추가하지 않는다(검증점검표: 결재+점검표 표 뒤 빈 문단 2개로 빈 3쪽이 생기던 회귀).
            // 단독 anchored 표·일반 문단 흐름의 trailing 빈 줄은 정상 페이지네이션을 유지해야
            // 하므로, page 마지막 항목이 co-anchored 자리차지 표일 때로 한정한다(orphan 가드와
            // 동일 신호). 명시적 쪽나누기가 걸린 빈 문단은 사용자 의도이므로 제외한다.
            let last_item_is_coanchored_float_table = match st.current_items.last() {
                Some(PageItem::Table {
                    para_index: tpi,
                    control_index: tci,
                })
                | Some(PageItem::PartialTable {
                    para_index: tpi,
                    control_index: tci,
                    ..
                }) => paragraphs.get(*tpi).is_some_and(|hp| {
                    let this_is_float = hp.controls.get(*tci).is_some_and(
                        |c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)),
                    );
                    let has_preceding_float = hp.controls.iter().take(*tci).any(
                        |c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)),
                    );
                    this_is_float && has_preceding_float
                }),
                _ => false,
            };
            let is_trailing_empty = para.text.is_empty()
                && para.controls.is_empty()
                && last_content_para_idx.is_some_and(|lc| para_idx > lc)
                && last_item_is_coanchored_float_table
                && !force_page_break
                && !para_style_break;
            if is_trailing_empty {
                let empty_h_px = para
                    .line_segs
                    .first()
                    .map(|s| {
                        hwpunit_to_px(
                            (s.line_height.saturating_add(s.line_spacing)) as i32,
                            self.dpi,
                        )
                    })
                    .unwrap_or(0.0);
                let avail = st.available_height() - st.current_height;
                if empty_h_px > avail {
                    continue;
                }
            }

            // [#1955] 글뒤로/글앞으로 표 직후의 빈 후행 문단 흡수.
            // 이 wrap 은 본문 플로우를 소비하지 않아야 하나(한글 시멘틱) 현재
            // 페이지네이션은 fragment 로 플로우를 소비하므로, 최소한 빈 후행 문단은
            // anchor 첫 fragment 단에 소급 기록하여 pi-page 정합을 복원한다.
            // (조례 [별표] 서식: 표 끝 쪽으로 밀리던 후행 빈 문단 — pi 9쪽 이탈)
            if let Some(anchor_pi) = st.behind_float_table_para {
                if !has_table {
                    let is_empty_para = para
                        .text
                        .chars()
                        .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
                        && para.controls.is_empty();
                    if is_empty_para {
                        // 표 fragment 가 지연 flush 라 지금은 anchor 단을 특정할 수
                        // 없다 — 보류 목록에 넣고 페이지 확정 후 일괄 부착.
                        st.behind_pending_absorbs.push(
                            crate::renderer::pagination::WrapAroundPara {
                                para_index: para_idx,
                                table_para_index: anchor_pi,
                                has_text: false,
                                start_line: 0,
                                end_line: usize::MAX,
                            },
                        );
                        continue;
                    }
                    st.behind_float_table_para = None;
                } else {
                    st.behind_float_table_para = None;
                }
            }

            // [Task #362] 어울림(Square wrap) 표 옆 paragraph 흡수.
            // Paginator engine.rs:288-320 동일 시멘틱.
            // 직전에 처리한 Square wrap 표의 (cs, sw) 와 동일한 LINE_SEG 를 가진
            // 후속 paragraph 는 표 옆에 배치되므로 height 소비 없이 wrap_around_paras 에 기록.
            let issue2424_wrap_started = issue2424_ts_enabled.then(std::time::Instant::now);
            let issue2424_wrap_absorbed = self.typeset_wrap_around_paragraph(
                &mut st,
                para,
                paragraphs,
                para_idx,
                has_table,
                page_def,
                composed.get(para_idx),
                styles,
            );
            Issue2424TypesetProfile::add(&mut issue2424_prof.wrap_around, issue2424_wrap_started);
            if issue2424_wrap_absorbed {
                continue;
            }

            st.ensure_page();

            // [Task #404] heading-orphan 패턴 보정 (vpos 기반).
            // 현재 paragraph 가 누적 height 로는 fit 하지만 HWP vpos 기준 페이지 한계를
            // 넘고, 다음 substantial block(Table/Shape/큰 paragraph)이 잔여 영역에 들어
            // 가지 않을 때 → 현재 paragraph 를 다음 페이지로 push 하여 heading + 후속
            // 블록을 같은 페이지에 배치.
            //
            // 조건 (모두 AND):
            //   A) current_items 비어있지 않음 (페이지 첫 item 자기참조 회피)
            //   B) 단일 단 + wrap-around zone 아님 (multi-column / wrap 의미 차이 회피)
            //   C) 누적 height 로 fit
            //   D) vpos overflow > 1mm (283 HU)
            //   E) 다음 paragraph 의 height 가 substantial (>30px ≈ 8mm) AND 잔여 영역에
            //      들어가지 않음
            //
            // Stage 1 진단 로그 분석으로 false positive 41건 → 1건(pi=83)으로 축소.
            // page_top_vpos 는 current_items 의 첫 item para_index 를 통해 즉시 계산
            // (TypesetState 필드 추적은 typeset_paragraph 내부 페이지 flush 와 동기 안 됨).
            if !st.current_items.is_empty() && st.wrap_around_cs < 0 && st.col_count == 1 {
                let page_first_para_idx = st.current_items.iter().find_map(|item| match item {
                    PageItem::FullParagraph { para_index } => Some(*para_index),
                    PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                    PageItem::Table { para_index, .. } => Some(*para_index),
                    PageItem::PartialTable { para_index, .. } => Some(*para_index),
                    PageItem::Shape { para_index, .. } => Some(*para_index),
                    PageItem::EndnoteSeparator { .. } => None,
                });
                let page_top_vpos_opt = page_first_para_idx
                    .and_then(|pi| paragraphs.get(pi))
                    .and_then(|p| p.line_segs.first())
                    .map(|s| s.vertical_pos);
                if let (Some(first_seg), Some(page_top_vpos)) =
                    (para.line_segs.first(), page_top_vpos_opt)
                {
                    let body_h_hu =
                        crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
                    let para_h_px: f64 = para
                        .line_segs
                        .iter()
                        .map(|s| {
                            crate::renderer::hwpunit_to_px(
                                s.line_height.saturating_add(s.line_spacing),
                                self.dpi,
                            )
                        })
                        .sum();
                    let para_h_hu = crate::renderer::px_to_hwpunit(para_h_px, self.dpi);
                    // [Task #643] vpos_end 는 마지막 줄의 bottom (vpos + lh) 기준.
                    // para_h_px 누적은 트레일링 line_spacing 까지 포함하여 ~10-12 HU 과대.
                    // HWP 가 페이지 끝에서 트레일링 ls 를 고려하지 않고 lh 만 fit 검사하는
                    // 시멘틱 정합 (pi=39 page 3 fits 케이스).
                    // 손상 입력의 거대한 vpos/height 로 i32 덧셈이 오버플로(패닉)하지
                    // 않도록 saturating — 정상값에선 동일, 손상값은 i32::MAX 로 포화.
                    let vpos_end = para
                        .line_segs
                        .last()
                        .map(|s| s.vertical_pos.saturating_add(s.line_height))
                        .unwrap_or(first_seg.vertical_pos.saturating_add(para_h_hu));
                    let page_bottom_vpos = page_top_vpos.saturating_add(body_h_hu);

                    let avail = st.available_height();
                    let current_fits = st.current_height + para_h_px <= avail;
                    let vpos_overflow = vpos_end > page_bottom_vpos + 283; // 1mm 안전여유

                    let next_h_px: f64 = paragraphs
                        .get(para_idx + 1)
                        .map(|p| {
                            p.line_segs
                                .iter()
                                .map(|s| {
                                    crate::renderer::hwpunit_to_px(
                                        s.line_height.saturating_add(s.line_spacing),
                                        self.dpi,
                                    )
                                })
                                .sum::<f64>()
                        })
                        .unwrap_or(0.0);
                    let next_substantial = next_h_px > 30.0;
                    let next_doesnt_fit = st.current_height + para_h_px + next_h_px > avail;

                    if current_fits && vpos_overflow && next_substantial && next_doesnt_fit {
                        st.advance_column_or_new_page();
                    }
                }
            }

            let issue2424_branch_started = issue2424_ts_enabled.then(std::time::Instant::now);
            let mut native_hwp5_footnote_break = None;
            let picture_host_origin = (st.pages.len(), st.current_column, st.current_height);
            if !has_table {
                // --- 핵심: format → fits → place/split ---
                let col_w = st
                    .layout
                    .column_areas
                    .get(st.current_column as usize)
                    .map(|a| a.width)
                    .unwrap_or(st.layout.body_area.width);
                // NO_LS 문서의 Square 그림 어울림 합성.
                // 저장 lineseg 가 없으면 어울림 기계(저장 cs/sw 매칭)가 서지 않아 텍스트가
                // 그림 위로 흐른다. 그림 사각형을 페이지 공간으로 기억해 두고(가로
                // 오프셋으로 앵커 단 밖에 놓이는 형상 포함), 현재 단과 교차하는 NO_LS
                // 문단을 감폭으로 측정하고 wrap_anchors 로 성형한다.
                let para_no_ls = !para.line_segs.iter().any(|seg| {
                    seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                });
                let mut col_w = col_w;
                let synth_col_area = st
                    .layout
                    .column_areas
                    .get(st.current_column as usize)
                    .map(|a| (a.x, a.width));
                if st.profile.hwp5_stored_pagination_layout() && para_no_ls {
                    if let Some((cax, _)) = synth_col_area {
                        let rects = synth_square_wrap_rects(para, cax, st.current_height, self.dpi);
                        st.wrap_synth_rects.extend(rects);
                    }
                    if let Some((cax, caw)) = synth_col_area {
                        if !st.wrap_synth_rects.is_empty() {
                            let cur = st.current_height;
                            let mut cs_px: f64 = 0.0;
                            let mut right_cut: f64 = caw;
                            for &(x0, x1, top, bottom) in &st.wrap_synth_rects {
                                // 시작 판정은 한 줄 높이(24px)까지 선행 허용 — 앵커 문단과
                                // 옆 단 텍스트의 y 가 정확히 일치하지 않는 저작 형상에서
                                // 항목 첫 줄이 미성형으로 그림과 겹치던 잔차 완화. 그보다
                                // 아래에서 시작하는 개체는 줄 단위 밴드 배제(아래 post-format
                                // 등록)가 담당한다 — 문단 전체 감폭은 첫 줄까지 밀어낸다.
                                // 소형 아이콘이 문단 첫 줄보다 아래에서 시작하면 문단
                                // 전체 감폭이 첫 줄까지 밀어낸다 — 줄 단위 밴드 배제
                                // (post-format 등록)가 담당하도록 여기서는 제외한다.
                                if bottom - top <= 40.0 && top > cur + 8.0 {
                                    continue;
                                }
                                // 대형 개체는 문단 시작이 개체보다 위면 감폭하지 않는다 —
                                // 개체 위에 있는 제목 줄 문단이 개체 옆으로 밀리는 오폭 방지
                                // (선행 허용 창은 판정/그리기 눈금 오차보다 커서 위험).
                                if bottom - top > 40.0 && top > cur {
                                    continue;
                                }
                                if cur + 40.0 < top || cur >= bottom - 1.0 {
                                    continue;
                                }
                                let ox0 = x0.max(cax);
                                let ox1 = x1.min(cax + caw);
                                if ox1 - ox0 < 8.0 {
                                    continue;
                                }
                                let left_gap = ox0 - cax;
                                let right_gap = cax + caw - ox1;
                                if right_gap >= left_gap {
                                    cs_px = cs_px.max(ox1 - cax + 3.8);
                                } else {
                                    right_cut = right_cut.min(left_gap - 3.8);
                                }
                            }
                            let sw_px = (right_cut - cs_px).max(0.0);
                            if (cs_px > 0.5 || right_cut < caw - 0.5) && sw_px >= caw * 0.25 {
                                st.current_column_wrap_anchors.insert(
                                    para_idx,
                                    crate::renderer::pagination::WrapAnchorRef {
                                        anchor_para_index: para_idx,
                                        anchor_cs: crate::renderer::px_to_hwpunit(cs_px, self.dpi),
                                        anchor_sw: crate::renderer::px_to_hwpunit(sw_px, self.dpi),
                                        anchor_image_margin_right: 0,
                                        band_y_range: None,
                                    },
                                );
                                let margin_left = styles
                                    .para_styles
                                    .get(para.para_shape_id as usize)
                                    .map_or(0.0, |style| style.margin_left);
                                col_w = crate::renderer::synthetic_wrap_column_width(
                                    col_w,
                                    margin_left,
                                    st.current_column_wrap_anchors.get(&para_idx),
                                    self.dpi,
                                );
                            }
                        }
                    }
                }
                // [#6175] devel 의 known-square-band 인자를 그대로 전달한다 —
                // 위에서 좁힌 col_w(합성 어울림 감폭)와 함께 쓴다.
                let formatted = self.format_paragraph_with_known_square_band(
                    para,
                    composed.get(para_idx),
                    styles,
                    Some(col_w),
                    st.wrap_around_derived_band
                        || st.current_column_wrap_anchors.contains_key(&para_idx),
                );
                // 줄 단위 어울림 배제: 문단 시작은 개체 위이지만 뒷줄이 개체 사각형과
                // 교차하는 형상(출석부) — 문단 전체 감폭 대신 교차 y 밴드를 anchor 에
                // 실어 layout 이 교차하는 줄만 감폭한다.
                if st.profile.hwp5_stored_pagination_layout()
                    && para_no_ls
                    && !st.current_column_wrap_anchors.contains_key(&para_idx)
                    && !st.wrap_synth_rects.is_empty()
                {
                    if let Some((cax, caw)) = synth_col_area {
                        let cur = st.current_height;
                        let para_bottom = cur + formatted.height_for_fit;
                        for &(x0, x1, top, bottom) in &st.wrap_synth_rects {
                            // 소형 아이콘이 문단 첫 줄보다 아래에서 시작해 문단 안에서
                            // 끝나는 형상만 — 문단 전체 감폭 판정과 상보.
                            if bottom - top > 40.0 || top <= cur + 8.0 || top >= para_bottom {
                                continue;
                            }
                            let ox0 = x0.max(cax);
                            let ox1 = x1.min(cax + caw);
                            if ox1 - ox0 < 8.0 {
                                continue;
                            }
                            let left_gap = ox0 - cax;
                            let right_gap = cax + caw - ox1;
                            if right_gap < left_gap {
                                continue; // 우측 개체 형상은 미지원(현행 유지)
                            }
                            let cs_px = ox1 - cax + 3.8;
                            let sw_px = caw - cs_px;
                            if sw_px < caw * 0.25 {
                                continue;
                            }
                            st.current_column_wrap_anchors.insert(
                                para_idx,
                                crate::renderer::pagination::WrapAnchorRef {
                                    anchor_para_index: para_idx,
                                    anchor_cs: crate::renderer::px_to_hwpunit(cs_px, self.dpi),
                                    anchor_sw: crate::renderer::px_to_hwpunit(sw_px, self.dpi),
                                    anchor_image_margin_right: 0,
                                    band_y_range: Some((top - cur, bottom - cur)),
                                },
                            );
                            break;
                        }
                    }
                }
                native_hwp5_footnote_break =
                    native_hwp5_first_footnote_overlap_break_line(&st, para, &formatted, self.dpi);
                let is_last_in_section = para_idx + 1 == paragraphs.len();
                if native_hwp5_circled_rowbreak_table_heading_requires_fresh_page(
                    &st, para, &formatted, paragraphs, para_idx, self.dpi,
                ) {
                    st.advance_column_or_new_page();
                }
                // [Task #1027 Stage D] fit 직전 vpos 스냅으로 누적 drift 제거 (렌더러 정합).
                self.vpos_snap_current_height(
                    &mut st,
                    para_idx,
                    paragraphs,
                    styles,
                    formatted.spacing_before,
                );
                if let Some(crate::model::control::Control::Table(prev_table)) =
                    st.current_items.last().and_then(|item| match item {
                        PageItem::Table {
                            para_index,
                            control_index,
                        } => paragraphs
                            .get(*para_index)
                            .and_then(|p| p.controls.get(*control_index)),
                        _ => None,
                    })
                {
                    let next_h = paragraphs.get(para_idx + 1).map(|next| {
                        next.line_segs
                            .iter()
                            .map(|s| {
                                hwpunit_to_px(
                                    s.line_height.saturating_add(s.line_spacing) as i32,
                                    self.dpi,
                                )
                            })
                            .sum::<f64>()
                    });
                    let remaining = st.available_height() - st.current_height;
                    let table_h = hwpunit_to_px(prev_table.common.height as i32, self.dpi);
                    if original_hwpx_tac_filled_page_keeps_short_trail(
                        !st.profile.hwp5_stored_pagination_layout(),
                        prev_table,
                        table_h,
                        st.layout.body_area.height,
                        remaining,
                        formatted.height_for_fit.max(formatted.total_height),
                        next_h.unwrap_or(0.0),
                    ) {
                        st.advance_column_or_new_page();
                    }
                }
                if !self.typeset_inline_flow(&mut st, para_idx, para, styles, measured_tables) {
                    self.typeset_paragraph(
                        &mut st,
                        para_idx,
                        para,
                        &formatted,
                        paragraphs,
                        styles,
                        is_last_in_section,
                    );
                }
                // HWPX가 수식 인라인 개체를 포함한 문단의 line_seg 높이를 실제
                // 조판보다 작게 저장하는 경우, 다음 문단의 양수 VPOS가 같은 물리
                // 쪽의 정확한 흐름 끝을 가리킨다. 일반 문단과 표에는 적용하지 않고,
                // 수식 전용 host가 같은 본문 영역의 다음 앵커를 넘겨 소비했을
                // 때에만 source flow 끝으로 되돌린다.
                let saved_equation_host_next_anchor = st.profile.hwpx_stored_layout()
                    && st.col_count == 1
                    && !para.controls.is_empty()
                    && para
                        .controls
                        .iter()
                        .all(|ctrl| matches!(ctrl, Control::Equation(_)));
                if saved_equation_host_next_anchor {
                    // 수식 host의 조판 높이를 다음 저장 VPOS로 되돌릴 근거는
                    // 수식 control과 같은 위치의 source LineSeg가 서로 다른 높이를
                    // 기록한 경우다. 두 높이가 일치하면 다음 앵커는 일반 본문 흐름의
                    // 정상 간격이므로 이를 압축하면 후속 쪽 owner를 앞당긴다.
                    let saved_equation_line_height_mismatch = para.line_segs.len()
                        == para.controls.len()
                        && para
                            .line_segs
                            .iter()
                            .zip(&para.controls)
                            .any(|(seg, ctrl)| {
                                matches!(ctrl, Control::Equation(equation)
                                if !is_synthetic_line_seg(seg)
                                    && seg.line_height > 0
                                    && equation.common.height > 0
                                    && seg.line_height != equation.common.height as i32)
                            });
                    let source_host = para
                        .line_segs
                        .iter()
                        .rev()
                        .find(|seg| !is_synthetic_line_seg(seg));
                    let source_next = paragraphs.get(para_idx + 1).and_then(|next| {
                        next.line_segs
                            .iter()
                            .find(|seg| !is_synthetic_line_seg(seg))
                    });
                    if let (Some(base), Some(source_host), Some(source_next)) =
                        (st.vpos_page_base, source_host, source_next)
                    {
                        let source_host_y = st.vpos_col_anchor
                            + hwpunit_to_px(
                                (source_host.vertical_pos as i64 - base as i64) as i32,
                                self.dpi,
                            );
                        let source_next_y = st.vpos_col_anchor
                            + hwpunit_to_px(
                                (source_next.vertical_pos as i64 - base as i64) as i32,
                                self.dpi,
                            );
                        if saved_equation_line_height_mismatch
                            && source_next.vertical_pos > source_host.vertical_pos
                            && source_host_y >= 0.0
                            && source_next_y <= st.base_available_height() + 0.5
                            && st.current_height > source_next_y
                        {
                            st.current_height = source_next_y;
                        }
                    }
                }
            } else {
                // 표 문단: Phase 2에서 전환 예정. 현재는 기존 방식 호환용 stub.
                self.typeset_table_paragraph(
                    &mut st,
                    para_idx,
                    para,
                    composed.get(para_idx),
                    paragraphs.get(para_idx + 1),
                    styles,
                    measured_tables,
                    page_def,
                    paragraphs,
                    composed,
                );
            }
            Issue2424TypesetProfile::add(
                if has_table {
                    &mut issue2424_prof.table_para
                } else {
                    &mut issue2424_prof.text_para
                },
                issue2424_branch_started,
            );

            // [Task #1027 Stage D] 항목 배치 후 vpos 커서 prev/base 추적 (렌더러 정합).
            // 렌더러 build_single_column: 매 항목 후 prev_layout_para 갱신, 표/Shape/
            // PartialTable 배치 후 page/lazy base 무효화(LINE_SEG lh 가 개체 높이를
            // 반영 못 해 drift 유발 → 직후 paragraph 는 lazy 역산으로 재산출). 단단 전용.
            if st.col_count == 1 {
                if st.reset_vpos_after_queued_table_footnote_page {
                    // RowBreak 표가 남은 cell-footnote만 든 fresh page를 만든 경우,
                    // 표 host의 저장 VPOS를 이 새 page 본문에 상속하면 빈 첫 문단도
                    // 표 높이만큼 forward-snap 된다. 새 page cursor를 보존하고 다음
                    // 본문 문단이 자체 stored VPOS에서 시작하게 한다.
                    st.reset_vpos_cursor();
                    st.reset_vpos_after_queued_table_footnote_page = false;
                } else {
                    st.vpos_prev_layout_para = Some(para_idx);
                    // [#2243] 저장-앵커 사다리 dirty 태깅: 저장 lineseg 없는 문단(생성기
                    // 누락)은 한글이 fresh 재계산으로 키울 수 있어, 그 뒤 기계 사다리
                    // v0 로의 **역스냅**은 성장분을 뭉갠다 (36398599: p33~36 누락 구간
                    // fresh +4320HU → 한글 4쪽 vs 역스냅 3쪽). dirty 이후 스냅은 전방만
                    // 허용한다. 합성-앵커(전면 NO_LS 문서)는 자기정합이므로 무관.
                    if st.vpos_page_base_stored
                        && paragraphs.get(para_idx).is_some_and(|p| {
                            p.line_segs.first().map_or(true, |s| {
                                s.tag
                                    & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                    != 0
                            })
                        })
                    {
                        st.vpos_ladder_dirty = true;
                    }
                    let last = st.current_items.last();
                    st.vpos_prev_partial_table =
                        matches!(last, Some(PageItem::PartialTable { .. }));
                    if matches!(
                        last,
                        Some(
                            PageItem::Table { .. }
                                | PageItem::PartialTable { .. }
                                | PageItem::Shape { .. }
                        )
                    ) {
                        // [#2243] 예외: 표 호스트의 **저장** lineseg lh 가 개체 높이를 온전히
                        // 포함(기계생성 결재문서: lh = 표 + outMargin)하면 저장 사다리가 표
                        // 라인을 관통해 연속이므로 base 를 유지한다. 무효화하면 후속 문단
                        // 스냅이 드리프트를 역산한 lazy base 로 고착 (36395325 p2 +10.7px,
                        // p4 +16.6px 팬텀 → sliver 쪽 +2). lh 가 개체를 못 담는 일반
                        // 케이스는 종전대로 무효화. HWPX(기계생성 결재문서 계열) 한정 —
                        // HWP5 native 는 종전 핀 보존 (issue_1418 2026_oss_rst 6쪽).
                        let host_line_covers_object = st.profile.hwpx_stored_layout()
                            && matches!(last, Some(PageItem::Table { .. }))
                            && para.line_segs.first().is_some_and(|s| {
                                s.tag
                                    & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
                                    == 0
                            })
                            && {
                                // lh 는 표 + outMargin×2 까지 담아야 사다리 연속이 보장된다
                                // (lh = 표높이만인 기계 사다리는 om 만큼 역스냅 과소 —
                                // 36398599 -1쪽 회귀 차단).
                                let max_tbl_h = para
                                    .controls
                                    .iter()
                                    .filter_map(|c| match c {
                                        Control::Table(t) => Some(
                                            t.common.height as i64
                                                + t.outer_margin_top as i64
                                                + t.outer_margin_bottom as i64,
                                        ),
                                        _ => None,
                                    })
                                    .max()
                                    .unwrap_or(i64::MAX);
                                para.line_segs
                                    .iter()
                                    .map(|s| s.line_height as i64)
                                    .max()
                                    .unwrap_or(0)
                                    >= max_tbl_h
                            };
                        if !host_line_covers_object {
                            // Para-float TopAndBottom 표 예외(렌더러 2513)는 Stage E.
                            st.vpos_page_base = None;
                            st.vpos_lazy_base = None;
                        }
                    }
                }
            }

            // [Task #362] Square wrap 표 처리 후 wrap zone 활성화.
            // Paginator engine.rs:356-372 동일 시멘틱.
            // 후속 paragraph 가 동일 cs/sw 를 가지면 흡수.
            if has_table {
                let has_tac_block = para
                    .controls
                    .iter()
                    .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char));
                let has_non_tac_table = !has_tac_block;
                if has_non_tac_table {
                    let is_wrap_around = para.controls.iter().any(|c| {
                        if let Control::Table(t) = c {
                            matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                        } else {
                            false
                        }
                    });
                    if is_wrap_around {
                        // [Task #1745] 텍스트 혼합 anchor(첫 LINE_SEG 가 전폭 텍스트 줄)면
                        // anchor LINE_SEG 가 wrap 띠를 인코딩하지 않으므로 표 geometry 로
                        // 띠 (cs, sw) 를 도출해 후속 문단 매칭에 사용 (한글: 후속 문단을
                        // 표 옆 잔여 띠에 배치 — samples/task1745).
                        if let Some((strip_cs, strip_sw)) =
                            crate::renderer::text_anchor_square_table_strip(para).or_else(|| {
                                crate::renderer::empty_host_square_table_left_strip(
                                    para,
                                    st.layout.column_width_hu(),
                                )
                            })
                        {
                            st.wrap_around_cs = strip_cs;
                            st.wrap_around_sw = strip_sw;
                            st.wrap_around_table_para = para_idx;
                            st.wrap_around_any_seg = false;
                            st.wrap_around_derived_band = false;
                        } else {
                            let anchor_cs =
                                para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                            let anchor_sw = para
                                .line_segs
                                .first()
                                .map(|s| s.segment_width as i32)
                                .unwrap_or(0);
                            // [#1956] anchor LINE_SEG 가 단 전체 폭이면(표가 본문 폭
                            // 이상 = 옆 공간 없음) 후속 전체 폭 문단들이 전부 오매칭
                            // 되므로 arming 하지 않는다. sw=0 은 기존 sw0_match 담당.
                            let col_w_hu = st.layout.column_width_hu();
                            let band_full_width =
                                anchor_sw > 0 && (anchor_sw - col_w_hu).abs() < 3000;
                            if !band_full_width {
                                st.wrap_around_cs = anchor_cs;
                                st.wrap_around_sw = anchor_sw;
                                st.wrap_around_table_para = para_idx;
                                st.wrap_around_any_seg = false;
                                st.wrap_around_derived_band = false;
                            }
                        }
                    }
                }
            }
            if has_table {
                let issue2424_flush_started = issue2424_ts_enabled.then(std::time::Instant::now);
                self.flush_deferred_table_controls(
                    &mut st,
                    paragraphs,
                    composed,
                    styles,
                    measured_tables,
                    DeferredTableFlushPoint::AfterTableParagraph(para_idx),
                );
                Issue2424TypesetProfile::add(
                    &mut issue2424_prof.deferred_flush,
                    issue2424_flush_started,
                );
                // [#1955] 글뒤로/글앞으로 비-TAC 표 anchor: 후행 빈 문단 흡수 arming.
                let has_behind_float_table = para.controls.iter().any(|c| {
                    matches!(c, Control::Table(t)
                        if !t.common.treat_as_char
                            && matches!(t.common.text_wrap,
                                crate::model::shape::TextWrap::BehindText
                                    | crate::model::shape::TextWrap::InFrontOfText))
                });
                // [#4514] 흡수는 표가 fragment 로 흐름을 이미 소비한 앵커(#1955 원
                // 사례 — oversized [별표] 표)에만 정당하다. #703 Shape 단축(흐름 0)
                // 앵커에서 후행 빈 문단까지 흡수하면 저장 사다리의 갭(≈표 높이)이
                // 어느 쪽에도 계상되지 않아 후속 표가 위로 붕괴한다 (sample1-repro
                // 8쪽: 표 4개 최대 555.5px 겹침). shortcut 앵커는 arming 을 생략해
                // 필러가 자기 줄 높이만큼 정상 흐름으로 전진하게 둔다. 한 문단에
                // shortcut 표와 fragment 표가 공존하는 극단 케이스도 생략 쪽을
                // 택한다(공간 이중 계상보다 유실이 드묾).
                if has_behind_float_table && st.overlay_shape_shortcut_para != Some(para_idx) {
                    st.behind_float_table_para = Some(para_idx);
                }
            }
            // 비-TAC Picture/Shape Square wrap: engine.rs:380-397 동일 시멘틱.
            // 그림의 첫 lineseg cs가 0일 수 있어 any_seg_matches 허용 플래그 활성화.
            let issue2424_tail_started = issue2424_ts_enabled.then(std::time::Instant::now);
            self.typeset_no_table_paragraph_tail(
                &mut st, page_def, para, paragraphs, composed, styles, para_idx, has_table,
            );
            Issue2424TypesetProfile::add(&mut issue2424_prof.para_tail, issue2424_tail_started);

            // Task #321: col 0 처리 중 body-wide TopAndBottom 표/도형이 발견되면
            // col 1+ advance 시 적용할 current_height 시작값을 미리 등록.
            // layout의 body_wide_reserved와 동일 조건으로 detect.
            if st.col_count > 1 && st.current_column == 0 && st.pending_body_wide_top_reserve == 0.0
            {
                let reserve = compute_body_wide_top_reserve_for_para(para, &st.layout, self.dpi);
                if reserve > 0.0 {
                    st.pending_body_wide_top_reserve = reserve;
                }
            }

            // [#1995] 한 문단에 근접-전면(near-full-page) non-TAC 그림이 여러 장이면
            // (임베드 매뉴얼을 페이지 이미지로 삽입한 경우 등) 각 이미지는 공존 불가하므로
            // 각각 한 페이지에 단독 배치해야 한다. 미수정 시 96장이 한 앵커에 스택되어
            // 문서가 과소 페이지가 된다(오라클 268 vs rhwp 174). 한 문단에 본문높이의
            // 60% 이상인 non-TAC 그림이 2장 이상일 때만 발동(정상 단일 그림 문단 불변).
            // 글뒤로/글앞으로 그림은 후보·분모 모두에서 제외한다(#6511, `flow_noninline_picture`).
            let fullpage_img_body_h = st.base_available_height();
            let fullpage_img_ctrls: Vec<usize> = if fullpage_img_body_h > 0.0 && !has_table {
                para.controls
                    .iter()
                    .enumerate()
                    .filter_map(|(ci, c)| {
                        let pic = flow_noninline_picture(c)?;
                        let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                        (h >= fullpage_img_body_h * 0.6).then_some(ci)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            // [#4654] 낱장 배치는 **전면 크기가 과반**인 문단에만 — 디자인
            // 보드형 pile(체육대회 4510000-202300010: 한 문단 그림 210장 중
            // 전면 ~15장, 한글은 쪽당 60여 장 통 적재에 전면 그림도 포함)에서
            // 소수 전면 그림이 낱장으로 탈출해 +12쪽이 됐다. #1995 원 취지
            // (임베드 매뉴얼: 전량 전면 96장, 오라클 268 vs 174 과소)는 과반
            // 조건으로 그대로 보존된다.
            let noninline_pic_count = para
                .controls
                .iter()
                .filter(|c| flow_noninline_picture(c).is_some())
                .count();
            // [#4770] #2004 본문 정규화와 같은 엄격한 저장 스택 계약을 쓴다. 즉 빈
            // 문단의 비-TAC Square·겹침불허 그림/그림-도형이 같은 세로 band에 있고,
            // 저장 첫 줄이 그림 폭 이상 오른쪽에서 시작하며, 그림 하단이 본문 하단 절대
            // 좌표 안에 있을 때만 #1995 낱장 배치를 억제한다. 일반 TopAndBottom·서로
            // 다른 anchor·본문 텍스트 문단은 기존 분산을 유지한다.
            let fullpage_img_min_height_hu =
                (crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi) / 2).max(1);
            let fullpage_img_body_bottom_hu = crate::renderer::px_to_hwpunit(
                st.layout.body_area.y + st.layout.body_area.height,
                self.dpi,
            );
            let stored_line_beside_pile = body_pile_stays_on_anchor_page(
                para,
                fullpage_img_min_height_hu,
                fullpage_img_body_bottom_hu,
            );
            let is_multi_fullpage_img_para = !stored_line_beside_pile
                && has_majority_fullpage_images(fullpage_img_ctrls.len(), noninline_pic_count);

            // [#2097] 이 문단의 TopAndBottom 자리차지 float pushdown 가로 컬럼
            // (h_left, h_right, 스택_높이) px — 가로 겹침으로 스택/나란히 판별.
            let mut topbottom_cols: Vec<(f64, f64, f64)> = Vec::new();
            // [#2814] 이 문단의 pushdown 대상(비-TAC TopAndBottom vert=Para) 그림/도형 수.
            // 3장 이상이 세로로 스택되면 쪽 용량 기반 분배 대상(아래 overflow 이월).
            let pushdown_topbottom_ctrl_count = para
                .controls
                .iter()
                .filter(|c| match c {
                    Control::Picture(pic) => {
                        !pic.common.treat_as_char
                            && matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            )
                    }
                    Control::Shape(s) => {
                        !s.common().treat_as_char
                            && matches!(
                                s.common().text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(
                                s.common().vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            )
                    }
                    _ => false,
                })
                .count();

            // 인라인 컨트롤 처리: 도형/그림/수식/각주 (Paginator engine.rs:509-525 동일)
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                // [#1995] 다수 전면 이미지: 각 전면 그림을 새 페이지에 단독 배치.
                if is_multi_fullpage_img_para && fullpage_img_ctrls.contains(&ctrl_idx) {
                    st.force_new_page();
                    st.current_items.push(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                    // 페이지를 채워 후속 그림/문단이 다음 페이지로 밀리도록.
                    st.current_height = fullpage_img_body_h;
                    continue;
                }
                match ctrl {
                    // [#6266] 비-TAC 양식 개체는 자기 배치(기준·정렬·오프셋)를 갖는
                    // 개체다. 종전에는 IR 에 배치가 없어 무조건 인라인으로 흘렀고,
                    // 쪽 하단 가운데 서식 번호가 제목 줄 안에 그려졌다.
                    Control::Form(form) if !form.common.treat_as_char => {
                        st.current_items.push(PageItem::Shape {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                    }
                    Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                        // [#6146] 저장 리셋 경계에서 떠나는 쪽의 흐름 말미에 이미 흘려
                        // 놓은 자리차지 밴드는 다시 배치하지 않는다.
                        if st.page_tail_spilled_floats.contains(&(para_idx, ctrl_idx)) {
                            continue;
                        }
                        if !has_table {
                            // [#3738 Stage 22] page-tail Square picture는 anchor 본문을
                            // 현재 쪽에 남기되 그림만 다음 physical page의 narrow wrap
                            // band에 배치한다. p155 그림 64처럼 현재 PageItem에 넣으면
                            // caption이 기존 FootnoteArea와 겹친다.
                            if let Some((wrap_target_para_indices, wrap_anchor)) = self
                                .native_hwp5_square_picture_next_page_owner(
                                    &st, para_idx, para, paragraphs, ctrl, styles,
                                )
                            {
                                st.deferred_next_page_square_pictures.push(
                                    DeferredSquarePictureControl {
                                        para_index: para_idx,
                                        control_index: ctrl_idx,
                                        wrap_target_para_indices,
                                        wrap_anchor,
                                    },
                                );
                                continue;
                            }
                            // [Issue #476/#4092] treat_as_char 그림/도형은 박스가 속한 line 이 라우팅된
                            // 페이지/단에 등록. paragraph 가 페이지 분할되면 이 시점의
                            // st.current_items 는 마지막 페이지 상태이므로, 그대로 push 하면
                            // 박스가 잘못된 페이지에 떠 있게 된다.
                            let routed = if crate::renderer::pagination::is_routable_treat_as_char_picture_or_shape(ctrl) {
                                crate::renderer::pagination::find_inline_control_target_page(
                                    &st.pages,
                                    &st.current_items,
                                    para_idx,
                                    ctrl_idx,
                                    para,
                                )
                            } else {
                                None
                            };
                            let item = PageItem::Shape {
                                para_index: para_idx,
                                control_index: ctrl_idx,
                            };
                            match routed {
                                Some((page_idx, col_idx)) => {
                                    if let Some(page) = st.pages.get_mut(page_idx) {
                                        if let Some(col) = page.column_contents.get_mut(col_idx) {
                                            col.items.push(item);
                                        } else {
                                            st.current_items.push(item);
                                        }
                                    } else {
                                        st.current_items.push(item);
                                    }
                                }
                                None => {
                                    st.current_items.push(item);
                                }
                            }
                            // [Task #1052] 글상자 내 각주 수집 (engine.rs:1376-1398 동등)
                            // NO_LS 호스트의 측정 원점만 전달한다. 저장 vpos 소유자는
                            // 기존 저장 배치 경로에 남긴다.
                            let host_top = (para.line_segs.is_empty()
                                && (st.pages.len(), st.current_column)
                                    == (picture_host_origin.0, picture_host_origin.1))
                                .then_some(picture_host_origin.2);
                            st.register_side_wrap_picture(
                                para_idx, ctrl_idx, para, host_top, styles,
                            );
                            if self.profile.get().hwp5_stored_pagination_layout()
                                && !self.profile.get().session_edited()
                                && st.current_items.iter().any(|item| {
                                    matches!(item, PageItem::FullParagraph { para_index } if *para_index == para_idx)
                                })
                            {
                                let saved = paragraphs.get(para_idx + 1).and_then(|next| {
                                    // An earlier picture with measured flow leaves
                                    // the column's painted origin outside this
                                    // stored reservation contract. Do not resume
                                    // absolute saved coordinates midway through
                                    // that chain and paint over preceding text.
                                    let unresolved_picture_flow = st.current_items.iter().any(|item| {
                                        let PageItem::Shape { para_index: owner, control_index } = item else { return false; };
                                        if *owner == para_idx { return false; }
                                        let Some(Control::Picture(picture)) = paragraphs.get(*owner).and_then(|p| p.controls.get(*control_index)) else { return false; };
                                        !picture.common.treat_as_char
                                            && picture.common.text_wrap == crate::model::shape::TextWrap::TopAndBottom
                                            && !st.paragraph_float_placements.get(&(*owner, *control_index)).is_some_and(|placement| matches!(placement.flow, super::float_placement::ParagraphFloatFlow::StoredPicture { .. }))
                                    });
                                    if unresolved_picture_flow { return None; }
                                    let host_style = styles.para_styles.get(para.para_shape_id as usize)?;
                                    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
                                    // The first stored line may retain its paragraph's
                                    // spacing-before at column top. That is an inset,
                                    // not the origin of the source coordinate system.
                                    let first_para = st.current_items.iter().find_map(|item| {
                                        match item {
                                            PageItem::FullParagraph { para_index } => paragraphs.get(*para_index),
                                            _ => None,
                                        }
                                    })?;
                                    let first_before = styles.para_styles.get(first_para.para_shape_id as usize)?.spacing_before;
                                    let base = st.vpos_page_base.unwrap_or(0);
                                    let retained_before = first_before.max(0.0).min(hwpunit_to_px(base.max(0), self.dpi));
                                    let frame_vpos = base - crate::renderer::px_to_hwpunit(retained_before, self.dpi);
                                    super::float_placement::stored_picture_successor_placement(
                                        para, next, host_style.spacing_before,
                                        next_style.spacing_before, frame_vpos, self.dpi,
                                    )
                                });
                                if let Some(placement) = saved.filter(|p| {
                                    p.occupied_bottom <= st.available_height()
                                        && p.anchor_y <= st.current_height
                                }) {
                                    st.paragraph_float_placements.insert((para_idx, ctrl_idx), placement);
                                    st.current_height = placement.paragraph_end(st.current_height, 0.0);
                                    continue;
                                }
                            }
                            // footnote-tbox-01.hwpx 의 글상자 안 각주 본문이 페이지 하단 영역
                            // 에 누락되는 결함 정정. engine.rs (legacy) 는 이미 처리하나
                            // typeset.rs (main, default) 만 누락 — feedback_image_renderer_paths_separate.
                            if let Control::Shape(shape_obj) = ctrl {
                                if let Some(text_box) =
                                    shape_obj.drawing().and_then(|d| d.text_box.as_ref())
                                {
                                    for (tp_idx, tp) in text_box.paragraphs.iter().enumerate() {
                                        for (tc_idx, tc) in tp.controls.iter().enumerate() {
                                            if let Control::Footnote(fn_ctrl) = tc {
                                                if let Some(page) = st.pages.last_mut() {
                                                    page.footnotes.push(FootnoteRef {
                                                        number: fn_ctrl.number,
                                                        source: FootnoteSource::ShapeTextBox {
                                                            para_index: para_idx,
                                                            shape_control_index: ctrl_idx,
                                                            tb_para_index: tp_idx,
                                                            tb_control_index: tc_idx,
                                                        },
                                                        fragment: None,
                                                    });
                                                    let fn_height = estimate_footnote_note_height(
                                                        fn_ctrl, self.dpi,
                                                    );
                                                    st.add_footnote_height(fn_height);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            // Task #409 v2: 비-TAC TopAndBottom + vert=Para Picture/Shape 는
                            // layout 에서 picture_footnote.rs:356 의 `y_offset + total_height`
                            // 패턴으로 후속 콘텐츠를 개체 높이만큼 밀어냄. 하지만 paragraph
                            // line_seg 의 lh 는 텍스트 baseline 만 반영하므로 페이지네이션의
                            // current_height 가 개체 높이만큼 부족하게 누적되어 page packing
                            // 시 layout 실제 y 와 어긋남 (21페이지: pagination used=803px vs
                            // layout y=1275px → pi=192 가 21페이지에 packing 되었다가
                            // overflow 로 잘림). pagination 측에서도 layout 과 동일하게
                            // 개체 높이를 current_height 에 누적.
                            use crate::model::shape::{TextWrap, VertRelTo};
                            // (obj_h, extra=obj_h+margin_bottom, h_left, h_right px)
                            let pushdown_h: Option<(f64, f64, f64, f64)> = match ctrl {
                                Control::Picture(pic)
                                    if !pic.common.treat_as_char
                                        && matches!(
                                            pic.common.text_wrap,
                                            TextWrap::TopAndBottom
                                        )
                                        && matches!(pic.common.vert_rel_to, VertRelTo::Para) =>
                                {
                                    let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    let mb =
                                        hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
                                    let hl = hwpunit_to_px(
                                        pic.common.horizontal_offset as i32,
                                        self.dpi,
                                    );
                                    let hr = hl + hwpunit_to_px(pic.common.width as i32, self.dpi);
                                    Some((h, h + mb, hl, hr))
                                }
                                Control::Shape(s)
                                    if !s.common().treat_as_char
                                        && matches!(
                                            s.common().text_wrap,
                                            TextWrap::TopAndBottom
                                        )
                                        && matches!(s.common().vert_rel_to, VertRelTo::Para) =>
                                {
                                    let cm = s.common();
                                    let h = hwpunit_to_px(cm.height as i32, self.dpi);
                                    let mb = hwpunit_to_px(cm.margin.bottom as i32, self.dpi);
                                    let hl = hwpunit_to_px(cm.horizontal_offset as i32, self.dpi);
                                    let hr = hl + hwpunit_to_px(cm.width as i32, self.dpi);
                                    Some((h, h + mb, hl, hr))
                                }
                                _ => None,
                            };
                            if let Some((obj_h, extra, h_left, h_right)) = pushdown_h {
                                // [Task #1079] 파일 vpos 가 이미 그림 공간을 반영(그림 para 줄
                                // 앞 gap ≥ 그림 높이)하면 VPOS_CORR sync 가 그 공간을 따르므로
                                // pushdown 가산은 이중 계상. gap 이 그림 높이 미만(파일 vpos
                                // 미반영, Task #409 계열)일 때만 가산.
                                const PUSHDOWN_GAP_TOL_PX: f64 = 8.0;
                                let already_accounted = para_idx > 0 && {
                                    let v_cur = para.line_segs.first().map(|s| s.vertical_pos);
                                    let prev_end = paragraphs[para_idx - 1]
                                        .line_segs
                                        .last()
                                        .map(|s| s.vertical_pos.saturating_add(s.line_height));
                                    match (v_cur, prev_end) {
                                        (Some(vc), Some(pe)) if vc > pe => {
                                            hwpunit_to_px((vc - pe) as i32, self.dpi)
                                                >= obj_h - PUSHDOWN_GAP_TOL_PX
                                        }
                                        _ => false,
                                    }
                                };
                                // [#6888] 자기 앵커보다 아래로 떨어진 개체는 뒤따르는
                                // 문단을 밀지 않는다 — 판별은 공용 헬퍼에 둔다(배치와
                                // 같은 답을 써야 `#409` 가 막으려던 desync 가 안 생긴다).
                                let displaced_below_following_flow = match ctrl {
                                    Control::Picture(pic) => {
                                        crate::renderer::topbottom_float_displaced_below_following_flow(
                                            para,
                                            paragraphs.get(para_idx + 1),
                                            &pic.common,
                                            self.dpi,
                                        )
                                    }
                                    Control::Shape(s) => {
                                        crate::renderer::topbottom_float_displaced_below_following_flow(
                                            para,
                                            paragraphs.get(para_idx + 1),
                                            s.common(),
                                            self.dpi,
                                        )
                                    }
                                    _ => false,
                                };
                                if !already_accounted && !displaced_below_following_flow {
                                    // [#2814] 절반쪽급 그림이 한 문단에 여럿 스택되면 한컴은
                                    // 흐름처럼 쪽을 채우며 다음 쪽으로 넘긴다(창조경제 보고서:
                                    // 절반쪽 그림 37장 = 쪽당 2장 × ~19쪽; #1995 의 전면 그림
                                    // 1장/쪽과 별개 축). 이 그림을 더하면 스택 하단이 본문을
                                    // 넘고 현재 쪽에 이미 다른 항목이 있으면, 방금 push 한 이
                                    // Shape 항목을 새 쪽으로 이월하고 스택을 리셋한다.
                                    // 발동은 3장 이상으로 한정 — 2장 스택은 한컴이 razor-full
                                    // 페이지에 압축 유지하는 실측 반례(1051000-201800093 p60,
                                    // 158쪽 정답 유지)가 있어 제외한다.
                                    if pushdown_topbottom_ctrl_count >= 3 {
                                        let ovl_base = topbottom_cols
                                            .iter()
                                            .filter(|c| c.1 > h_left && c.0 < h_right)
                                            .map(|c| c.2)
                                            .fold(0.0_f64, f64::max);
                                        let before_max = topbottom_cols
                                            .iter()
                                            .map(|c| c.2)
                                            .fold(0.0_f64, f64::max);
                                        let delta = (ovl_base + extra - before_max).max(0.0);
                                        let self_is_last = matches!(
                                            st.current_items.last(),
                                            Some(PageItem::Shape { para_index: p, control_index: c })
                                                if *p == para_idx && *c == ctrl_idx
                                        );
                                        if delta > 0.0
                                            && st.current_height + delta
                                                > st.available_height() + 0.5
                                            && self_is_last
                                            && st.current_items.len() > 1
                                        {
                                            st.current_items.pop();
                                            st.advance_column_or_new_page();
                                            st.current_items.push(PageItem::Shape {
                                                para_index: para_idx,
                                                control_index: ctrl_idx,
                                            });
                                            topbottom_cols.clear();
                                        }
                                    }
                                    // [#2097] 같은 문단의 TopAndBottom float pushdown 을 가로
                                    // 컬럼 모델로 예약한다. 판별 축은 세로 offset 이 아니라 가로
                                    // 겹침 — 가로로 겹치는 float 은 세로로 스택되어 합산
                                    // (1342000 취업정책연구: 큰 그림 4장이 같은 단 off≈0 에 스택,
                                    // 세로 offset 은 작아 offset-union 은 오병합), 가로로 분리된
                                    // float 은 나란히라 같은 세로 band 를 공유해 max 예약
                                    // (17809123 자원봉사증: 좌우 그림 2장 h-range 분리). 예약
                                    // 총량 = max(컬럼별 스택 높이). 새 float 이 겹치는 컬럼(들)에
                                    // 스택되면 그 컬럼 높이에 extra 가산, 안 겹치면 새 컬럼. 단일
                                    // float 은 컬럼 1개=extra 라 동작 불변.
                                    let overlapping: Vec<usize> = topbottom_cols
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, c)| c.1 > h_left && c.0 < h_right)
                                        .map(|(i, _)| i)
                                        .collect();
                                    let applied_before =
                                        topbottom_cols.iter().map(|c| c.2).fold(0.0_f64, f64::max);
                                    if overlapping.is_empty() {
                                        topbottom_cols.push((h_left, h_right, extra));
                                    } else {
                                        let base = overlapping
                                            .iter()
                                            .map(|&i| topbottom_cols[i].2)
                                            .fold(0.0_f64, f64::max);
                                        let new_l = overlapping
                                            .iter()
                                            .map(|&i| topbottom_cols[i].0)
                                            .fold(h_left, f64::min);
                                        let new_r = overlapping
                                            .iter()
                                            .map(|&i| topbottom_cols[i].1)
                                            .fold(h_right, f64::max);
                                        for &i in overlapping.iter().rev() {
                                            topbottom_cols.remove(i);
                                        }
                                        topbottom_cols.push((new_l, new_r, base + extra));
                                    }
                                    let applied_after =
                                        topbottom_cols.iter().map(|c| c.2).fold(0.0_f64, f64::max);
                                    st.current_height += (applied_after - applied_before).max(0.0);
                                }
                            }
                        }
                    }
                    Control::Footnote(fn_ctrl) => {
                        self.register_body_footnote(
                            &mut st,
                            para_idx,
                            para,
                            paragraphs,
                            ctrl_idx,
                            fn_ctrl,
                            has_table,
                            native_hwp5_footnote_break,
                        );
                    }
                    Control::Endnote(en_ctrl) => {
                        // [Task #836] 미주 수집 — 문서 끝에 모아서 렌더
                        st.endnotes.push(EndnoteRef {
                            number: en_ctrl.number,
                            section_index,
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                    }
                    _ => {}
                }
            }
            // #6950: text and its tail table may be emitted in paint order rather
            // than logical order. Finish the paragraph after ALL its controls;
            // post-text must not return flow to a position above its own table.
            let spacing_after = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map_or(0.0, |style| style.spacing_after);
            for (&(owner, _), placement) in &st.paragraph_float_placements {
                if owner == para_idx
                    && placement.flow == super::float_placement::ParagraphFloatFlow::NextLine
                {
                    st.current_height = placement.paragraph_end(st.current_height, spacing_after);
                    st.ladder_band_floor = st.ladder_band_floor.max(st.current_height);
                }
            }
            // [Task #1007] variant vpos reset 감지용 prev_para_idx 갱신
            variant_prev_para_idx = Some(para_idx);
        }

        let issue2424_loop_elapsed = issue2424_loop_started
            .map(|started| started.elapsed())
            .unwrap_or_default();
        let issue2424_inloop_flush = issue2424_prof.deferred_flush.0;

        // source tail에 뒤따르는 본문이 없어 자연 page break가 일어나지 않은 경우에도,
        // deferred Square picture는 현재 쪽 FootnoteArea에 겹치지 않고 독립한 다음 physical
        // page를 가져야 한다. 일반 흐름을 되감지 않고 picture PageItem만 drain한다.
        if !st.deferred_next_page_square_pictures.is_empty() {
            st.force_new_page();
        }

        // [미주 배치 — Hancom EndnoteEndOfSection/EndnoteEndOfDocument]
        let issue2424_endnote_started = issue2424_ts_enabled.then(std::time::Instant::now);
        self.typeset_section_endnotes(
            &mut st,
            paragraphs,
            composed,
            styles,
            section_index,
            page_def,
            measured_tables,
            endnote_shape,
            &endnote_deferral,
        );
        if let Some(started) = issue2424_endnote_started {
            issue2424_prof.endnotes = started.elapsed();
        }

        // 마지막 항목 처리
        let issue2424_final_flush_started = issue2424_ts_enabled.then(std::time::Instant::now);
        self.flush_deferred_table_controls(
            &mut st,
            paragraphs,
            composed,
            styles,
            measured_tables,
            DeferredTableFlushPoint::SectionEnd,
        );
        Issue2424TypesetProfile::add(
            &mut issue2424_prof.deferred_flush,
            issue2424_final_flush_started,
        );
        if !st.current_items.is_empty() {
            st.flush_column_always();
        }
        st.ensure_page();

        // [#1955] 글뒤로 표 후행 빈 문단 보류 흡수 부착 — 페이지 확정 후
        // anchor 표의 첫 fragment 가 있는 단에 소급 기록 (한글: 글뒤로 표는
        // 플로우를 소비하지 않으므로 후행 문단이 anchor 쪽에 남음).
        for wrap_para in std::mem::take(&mut st.behind_pending_absorbs) {
            let anchor = wrap_para.table_para_index;
            let is_first_fragment = |it: &PageItem| match it {
                PageItem::Table { para_index, .. } => *para_index == anchor,
                PageItem::PartialTable {
                    para_index,
                    is_continuation,
                    ..
                } => *para_index == anchor && !*is_continuation,
                _ => false,
            };
            let mut attached = false;
            'outer: for page in st.pages.iter_mut() {
                for col in page.column_contents.iter_mut() {
                    if col.items.iter().any(is_first_fragment) {
                        col.wrap_around_paras.push(wrap_para.clone());
                        attached = true;
                        break 'outer;
                    }
                }
            }
            if !attached {
                // anchor 미발견(예: 표가 배치 제외) — 마지막 단에 부착해 문단 누락 방지
                if let Some(col) = st
                    .pages
                    .last_mut()
                    .and_then(|p| p.column_contents.last_mut())
                {
                    col.wrap_around_paras.push(wrap_para);
                }
            }
        }

        // 한컴은 문서의 마지막에 남은 빈 문단 묶음 때문에 별도 빈 쪽을 만들지
        // 않는다. 일반 흐름에서는 이 문단들이 앞 쪽의 tail로 흡수되지만, 큰 표의
        // 마지막 fragment 뒤에서는 저장 vpos가 다음 쪽을 가리킬 수 있다. 그 경우
        // rhwp가 100HU짜리 빈 line-seg만 담은 페이지를 확정하면 실제 출력보다 한
        // 쪽이 늘어난다 (#3637 HWP 2020 oracle 31쪽 → 32쪽). 명시적인 page/section
        // break나 가시 컨트롤은 보존하고, 정말 빈 문단만 있는 마지막 쪽만 버린다.
        Self::discard_terminal_blank_only_page(&mut st.pages, paragraphs);

        // 페이지 번호 + 머리말/꼬리말 할당
        Self::finalize_pages(&mut st.pages, &hf_entries, &page_number_pos, paragraphs);

        if let Some(started) = issue2424_ts_started {
            let total = started.elapsed();
            let loop_other = issue2424_loop_elapsed
                .saturating_sub(issue2424_prof.wrap_around.0)
                .saturating_sub(issue2424_prof.text_para.0)
                .saturating_sub(issue2424_prof.table_para.0)
                .saturating_sub(issue2424_inloop_flush)
                .saturating_sub(issue2424_prof.para_tail.0);
            let post = total
                .saturating_sub(issue2424_prof.setup)
                .saturating_sub(issue2424_loop_elapsed)
                .saturating_sub(issue2424_prof.endnotes);
            eprintln!(
                "RHWP_2424_TYPESET_PROFILE sec={} pages={} paras={} total_ms={:.2} \
                 setup={:.2} loop={:.2} [wrap={:.2}/{} text={:.2}/{} table={:.2}/{} \
                 flush={:.2}/{} tail={:.2}/{} other={:.2}] endnotes={:.2} post={:.2}",
                section_index,
                st.pages.len(),
                paragraphs.len(),
                Issue2424TypesetProfile::ms(total),
                Issue2424TypesetProfile::ms(issue2424_prof.setup),
                Issue2424TypesetProfile::ms(issue2424_loop_elapsed),
                Issue2424TypesetProfile::ms(issue2424_prof.wrap_around.0),
                issue2424_prof.wrap_around.1,
                Issue2424TypesetProfile::ms(issue2424_prof.text_para.0),
                issue2424_prof.text_para.1,
                Issue2424TypesetProfile::ms(issue2424_prof.table_para.0),
                issue2424_prof.table_para.1,
                Issue2424TypesetProfile::ms(issue2424_prof.deferred_flush.0),
                issue2424_prof.deferred_flush.1,
                Issue2424TypesetProfile::ms(issue2424_prof.para_tail.0),
                issue2424_prof.para_tail.1,
                Issue2424TypesetProfile::ms(loop_other),
                Issue2424TypesetProfile::ms(issue2424_prof.endnotes),
                Issue2424TypesetProfile::ms(post),
            );
        }

        PaginationResult {
            pages: st.pages,
            wrap_around_paras: Vec::new(),
            hidden_empty_paras: st.hidden_empty_paras,
            pre_emitted_host_paras: st.pre_emitted_host_paras,
            pre_emitted_host_heights: st.pre_emitted_host_heights,
            endnotes: st.endnotes,
            endnote_paragraphs: st.endnote_paragraphs,
            endnote_para_sources: st.endnote_para_sources,
            endnote_between_notes_hu: st.endnote_between_notes_hu,
            endnote_separator_above_hu: st.endnote_separator_above_hu,
            endnote_separator_below_hu: st.endnote_separator_below_hu,
        }
    }

    // ========================================================
    // format: 문단의 실제 높이를 계산한다
    // ========================================================

    /// 렌더러 `build_single_column` 의 inter-item VPOS_CORR(Stage C `HeightCursor::vpos_adjust`)
    /// 를 페이지네이터에서도 적용해, 단락마다 `+= total_height` 로 누적된 sb·trailing_ls
    /// drift 를 다음 항목 진입 시 제거한다(렌더러와 동일 측정). 단단(col_count==1) 전용 —
    /// 다단/flow-around 는 Stage E.
    ///
    /// HeightCursor 는 `current_height` 상대공간(col_area_y=0)에서 구동한다.
    fn vpos_snap_current_height(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        spacing_before_px: f64,
    ) {
        if st.col_count != 1 {
            return; // 다단은 Stage E
        }
        // 컬럼 첫 항목: anchor + page_base 확립 (렌더러 2186/2216 정합).
        // items.first() 의 vpos 를 page_base 로, 현 current_height 를 anchor 로 둔다.
        if st.current_items.is_empty() {
            st.vpos_col_anchor = st.current_height;
            st.vpos_page_base = paragraphs
                .get(para_idx)
                .and_then(|p| p.line_segs.first())
                .map(|s| s.vertical_pos);
            st.vpos_lazy_base = None;
            // [#2243] 저장 여부 태깅 — dirty 역스냅 금지 판단용.
            st.vpos_page_base_stored = paragraphs
                .get(para_idx)
                .and_then(|p| p.line_segs.first())
                .is_some_and(|s| {
                    s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                });
            st.vpos_ladder_dirty = false;
        }
        let mut hc = HeightCursor {
            dpi: self.dpi,
            col_area_y: 0.0,
            col_area_height: st.base_available_height(),
            col_anchor_y: st.vpos_col_anchor,
            vpos_page_base: st.vpos_page_base,
            vpos_lazy_base: st.vpos_lazy_base,
            prev_layout_para: st.vpos_prev_layout_para,
            prev_item_was_partial_table: st.vpos_prev_partial_table,
            skip_spacing_before_prededuct: st.skip_spacing_before_prededuct,
            trimmed_prev_spacing_before_px: st.vpos_prev_trimmed_sb_px,
            allow_vpos_rewind: false,
            allow_start_height_backtrack: false,
            suppress_large_forward_jump: false,
            suppress_hwpx_stale_forward: st.profile.hwpx_stored_layout(),
            uniform_filler_ladder: self.uniform_filler_ladder.get(),
            endnote_between_notes_hu: 0,
            prev_item_content_bottom_y: None,
            last_compacted_endnote_title_gap: false,
            min_flow_floor: f64::MIN,
            session_edited: self.profile.get().session_edited(),
        };
        let mut y = hc.vpos_adjust(st.current_height, para_idx, paragraphs, styles);
        // [#5699 H1] 저장 사다리가 자리차지 표 밴드를 계상하지 않은 문서: 흐름이
        // 계상 교정으로 확보한 표 밴드 위로 저장 vpos 스냅으로 되감기지 못한다.
        if y < st.ladder_band_floor {
            y = st.ladder_band_floor;
        }
        // 렌더의 min_flow_floor는 floor뿐 아니라 직전 순차 cursor도 보호한다.
        // 회피한 표 이후 분할기만 저장 vpos로 되감으면 쪽 fit와 출력이 갈라진다.
        if !st.inline_placements.is_empty()
            || !st.inline_flow_plans.is_empty()
            || st
                .paragraph_float_placements
                .iter()
                .any(|(&(owner, _), placement)| {
                    owner < para_idx
                        && placement.flow == super::float_placement::ParagraphFloatFlow::NextLine
                })
        {
            y = y.max(st.current_height);
        }
        // [#2243] dirty 저장-앵커 사다리의 역스냅 금지 — 저장 lineseg 누락 문단의
        // fresh 재계산 성장분을 낡은 기계 v0 가 되돌리지 못하게 한다(전방만 허용).
        // [#2279 OMIT-sa] spacing-누락 문서군은 합성(비저장) base 사다리도 동일 —
        // 사다리 자체가 spacing 을 누락하므로 dirty 성장분의 역스냅을 base 저장
        // 여부와 무관하게 금지한다 (36392557 p4: pi61 저장 anchor 로의 역스냅이
        // sa 성장분 +20.1px 를 되감아 페이지 경계를 한 문단 뒤로 밀던 형상).
        if st.vpos_ladder_dirty
            && (st.vpos_page_base_stored || st.omit_fresh_recalc_doc)
            && y < st.current_height
        {
            y = st.current_height;
        }
        // [#2279 OMIT-sa] 합성(비저장) lineseg 경계의 후방 스냅량이 직전 문단의
        // spacing_after 와 일치(±2px)하면 합성 사다리의 sa-누락이다 — 한글
        // fresh 는 sa 를 가산하므로 되감지 않는다 (36392557 p4 pi42/43/44
        // sa 6.7px ×3, 한글 PDF 절대좌표 +20.1px 실측). OMIT 문서군 + 합성
        // seg 직전 문단 한정. 이후 저장 anchor 역스냅이 성장분을 되돌리지
        // 못하게 dirty 처리(위 #2243 확장과 짝).
        if st.omit_fresh_recalc_doc && y < st.current_height && para_idx > 0 {
            let prev = &paragraphs[para_idx - 1];
            let prev_synthetic = !prev.line_segs.is_empty()
                && prev.line_segs.iter().all(|s| {
                    s.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                });
            let prev_sa = styles
                .para_styles
                .get(prev.para_shape_id as usize)
                .map(|s| s.spacing_after)
                .unwrap_or(0.0);
            if prev_synthetic && prev_sa > 0.5 && (st.current_height - y - prev_sa).abs() < 2.0 {
                y = st.current_height;
                st.vpos_ladder_dirty = true;
            }
        }
        // [#2279 ladder-sb] 후방 스냅량이 이 문단의 spacing_before 와 정확히
        // 일치(±2px)하고, **저장 ladder 스텝 자체가 sb 를 누락**했으면 생성기
        // ladder 의 sb-누락이다 — 한글 fresh 는 sb 를 가산하므로(서브픽셀
        // 하니스 실측: 36398709 문단당 −5.0pt 균일 = sb 6.7px) 스냅으로 되감지
        // 않는다. 스텝-검사 없이 ±2px 우연 일치만으로 스킵하면 sb-포함 정상
        // ladder 에서 이중 가산(+1쪽 과다, issue_1853 실측 반증)이 난다.
        if st.profile.hwpx_stored_layout()
            && spacing_before_px > 0.5
            && y < st.current_height
            && (st.current_height - y - spacing_before_px).abs() < 2.0
        {
            y = st.current_height;
        }
        // [#6031] sb-누락 ladder 는 경계 하나가 아니라 **문서 전체 서명**이다 —
        // 위 ±2px 일치 스킵만으로는 누락분이 다음 sb=0 경계의 후방 스냅으로
        // 흘러가 되감긴다(3249937 p3: pi=36 스킵분 −6.7px 가 pi=37 스냅으로
        // 제거, 쪽 말미 누적 +53.3px 를 typeset 만 안 본다). 렌더는 후방 스냅
        // 상한(8px)으로 이 되감김을 거부하므로 판정 좌표와 배치 좌표가 갈라져
        // 쪽-말미 줄이 본문 하단 밖에 그려진다(#5801 코어). 경계 검사
        // (`stored_ladder_encodes_spacing_before`)가 누락을 확정하면 남은 열의
        // 후방 스냅·트림을 dirty 로 철회해 두 좌표계를 일치시킨다 — 한글
        // fresh 도 sb 를 가산한 흐름으로 쪽을 끊는다(p3 말미 810.7px 실측
        // = 한글 809.7pt 정합, 꼬리 '바.' 줄은 한글도 4쪽 첫 줄).
        if st.profile.hwpx_stored_layout()
            && !st.profile.hwp3_layout()
            && spacing_before_px > 5.0
            && !st.vpos_ladder_dirty
            && !stored_ladder_encodes_spacing_before(
                paragraphs,
                para_idx,
                spacing_before_px,
                self.dpi,
            )
        {
            st.vpos_ladder_dirty = true;
            if y < st.current_height {
                y = st.current_height;
            }
        }
        // [#2243 진단] snap 입출력 — 동작 불변.
        if std::env::var("RHWP_DIAG_SNAPALL").is_ok() {
            eprintln!(
                "DIAG_SNAPALL pi={} y_in={:.1} y_out={:.1} base={:?} lazy={:?} anchor={:.1} dirty={}",
                para_idx,
                st.current_height,
                y,
                st.vpos_page_base,
                hc.vpos_lazy_base,
                st.vpos_col_anchor,
                st.vpos_ladder_dirty,
            );
        }
        if std::env::var("RHWP_DIAG_TAC").is_ok() && (y - st.current_height).abs() > 0.05 {
            eprintln!(
                "DIAG_SNAP pi={} y_in={:.1} y_out={:.1} page_base={:?} lazy={:?} anchor={:.1}",
                para_idx,
                st.current_height,
                y,
                st.vpos_page_base,
                hc.vpos_lazy_base,
                st.vpos_col_anchor,
            );
        }
        // lazy_base 는 지연 산출 시 갱신될 수 있으므로 회수.
        st.vpos_lazy_base = hc.vpos_lazy_base;
        st.current_height = y;
    }

    /// 프로덕션 문단 높이. dump-pages 진단도 이 경로만 읽는다 (#4628).
    fn format_paragraph(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
    ) -> FormattedParagraph {
        self.format_paragraph_for_flow(
            para,
            composed,
            styles,
            column_width_px,
            styles.hwp3_variant,
            false,
        )
    }

    fn format_paragraph_with_known_square_band(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
        known_square_band: bool,
    ) -> FormattedParagraph {
        self.format_paragraph_for_flow(
            para,
            composed,
            styles,
            column_width_px,
            styles.hwp3_variant,
            known_square_band,
        )
    }

    fn format_paragraph_for_flow(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        column_width_px: Option<f64>,
        hwp3_body_reflow: bool,
        known_square_band: bool,
    ) -> FormattedParagraph {
        let context = paragraph::context::ParagraphFormatContext::new(
            self.dpi,
            &self.profile,
            &self.uniform_filler_ladder,
            &self.float_carve_evidence,
        );
        paragraph::format::format_paragraph_for_flow(
            &context,
            para,
            composed,
            styles,
            column_width_px,
            hwp3_body_reflow,
            known_square_band,
        )
    }

    // ========================================================
    // fits + place/split: 배치 판단과 실행
    // ========================================================

    /// 문단을 현재 페이지에 배치한다.
    /// fits → place(전체) 또는 split(줄 단위) → move(다음 페이지)
    fn typeset_paragraph(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        fmt: &FormattedParagraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        is_last_in_section: bool,
    ) {
        paragraph::flow::place(
            st,
            paragraph::flow::ParagraphFlowInput {
                para_idx,
                para,
                fmt,
                paragraphs,
                styles,
                is_last_in_section,
            },
            self.dpi,
            || self.profile.get().session_edited(),
        );
    }

    // ========================================================
    // Phase 2: Break Token 기반 표 조판
    // ========================================================

    /// 표의 조판 높이를 계산한다 (format 단계).
    /// MeasuredTable + host_spacing을 통합하여 layout과 동일한 규칙으로 계산.
    #[allow(clippy::too_many_arguments)]
    fn format_table(
        &self,
        para: &Paragraph,
        para_idx: usize,
        ctrl_idx: usize,
        table: &crate::model::table::Table,
        measured_tables: &[MeasuredTable],
        styles: &ResolvedStyleSet,
        composed: Option<&ComposedParagraph>,
        next_para: Option<&Paragraph>,
        is_column_top: bool,
    ) -> FormattedTable {
        table::format(
            table::TableFormatInput {
                para,
                para_idx,
                ctrl_idx,
                table,
                measured_tables,
                styles,
                composed,
                next_para,
                is_column_top,
            },
            self.dpi,
            || self.profile.get(),
            || self.uses_tac_table_flow(table),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn flush_deferred_table_controls(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        flush_point: DeferredTableFlushPoint,
    ) {
        controls::flush_deferred_tables(st, paragraphs, flush_point, |st, deferred| {
            self.place_deferred_table_control(
                st,
                deferred,
                paragraphs,
                composed,
                styles,
                measured_tables,
            );
        });
    }

    /// 지연 큐에서 선택된 표 하나를 현재 열에서 다시 측정하고 배치한다.
    #[allow(clippy::too_many_arguments)]
    fn place_deferred_table_control(
        &self,
        st: &mut TypesetState,
        deferred: DeferredTableControl,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
    ) {
        controls::deferred_placement::place(
            self,
            st,
            deferred,
            paragraphs,
            composed,
            styles,
            measured_tables,
        );
    }

    /// 표가 포함된 문단을 처리한다.
    /// 각 컨트롤(표/도형)에 대해 format → fits → place/split 패턴 적용.
    #[allow(clippy::too_many_arguments)]
    fn typeset_table_paragraph(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        next_para: Option<&Paragraph>,
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        _page_def: &PageDef,
        // [Task #1753] 지연 이월 표의 후속 문단 prefill 용 전체 슬라이스.
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
    ) {
        controls::paragraph_flow::place(
            self,
            st,
            controls::paragraph_flow::TableParagraphInput {
                para_idx,
                para,
                composed,
                next_para,
                styles,
                measured_tables,
                paragraphs_all,
                composed_all,
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn try_typeset_empty_para_float_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        composed: Option<&ComposedParagraph>,
        next_para: Option<&Paragraph>,
        styles: &ResolvedStyleSet,
        para_start_height: f64,
        lanes: &mut FloatLaneSet,
    ) -> bool {
        controls::try_place_empty_para_float_table(
            st,
            para_idx,
            ctrl_idx,
            para,
            table,
            ft,
            composed,
            next_para,
            styles,
            para_start_height,
            lanes,
            self.dpi,
        )
    }

    /// TAC(treat_as_char) 표의 조판.
    #[allow(clippy::too_many_arguments)]
    fn typeset_tac_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        fmt: &FormattedParagraph,
        tac_count: usize,
        is_first_placed: bool,
        is_last_placed: bool,
        styles: &ResolvedStyleSet,
        // [#3837] 값이 있는 가장 가까운 앞 문단의 마지막 저장 vpos.
        prev_stored_vpos: Option<i32>,
    ) {
        // [Task #1152] 호스트 문단의 intra-paragraph vpos-reset 가드.
        // empty-text host paragraph 가 N controls + N line_segs 1:1 매핑이고,
        // 현재 TAC 표의 매핑 line_seg(ctrl_idx>0) 의 vpos==0 이면 HWP 가 "이 표를
        // 새 페이지 상단부터" 라고 명시한 신호. fit 검사는 표 크기가 잔여 영역에
        // 들어가면 통과시키지만 명시 신호를 존중하려면 fit 이전에 advance.
        // 케이스: 2022년 국립국어원 업무계획.hwp pi=586 ci=1 (별첨 박스).
        let tac_table_line_idx = self.tac_table_line_index(para, table, fmt);
        let prior_tac = para
            .controls
            .iter()
            .take(ctrl_idx)
            .filter(|c| matches!(c, Control::Table(t) if self.is_effective_tac_table(para, t, fmt)))
            .count();
        let tac_seg_idx = if tac_count > 1 {
            // [#2322] 텍스트-host 다중 TAC: 선행 텍스트 줄 수만큼 lineseg 매핑을
            // 오프셋한다. 종전 count 기반 매핑은 제목 줄이 있는 문단에서 표1을
            // 텍스트 줄(예: 16px)에 매핑해 851px 표가 16px 로 계상됐다 (20862337
            // r15 재검증 −1 서식 계열). 빈-host 1:1 문서는 오프셋 0 으로 불변.
            let leading_offset = para
                .controls
                .iter()
                .find_map(|c| match c {
                    Control::Table(t) if self.is_effective_tac_table(para, t, fmt) => Some(t),
                    _ => None,
                })
                .and_then(|t| self.tac_table_line_index(para, t, fmt))
                .unwrap_or(0);
            leading_offset + prior_tac
        } else {
            tac_table_line_idx.unwrap_or(0)
        };

        let owned_single_tac_row_height =
            (st.profile.hwpx_stored_layout() && tac_count == 1 && fmt.line_heights.len() == 1)
                .then(|| crate::renderer::composer::owned_rowbreak_tac_height(para, ctrl_idx))
                .flatten()
                .map(|height| hwpunit_to_px(height, self.dpi));
        // HWPX RowBreak TAC 표는 대개 host LINE_SEG가 표의 물리 줄을 보존한다.
        // 다만 단일 빈 host에 큰 다행 표가 있는데 저장 줄이 선언 높이를 전혀
        // 담지 못하면, 그 줄은 표 band의 소유 증거가 아니다. 이 형상을 짧은
        // host 줄로 계상하면 표와 뒤 문단이 한 쪽에 겹친다. 실측 표 높이와
        // trailing line spacing을 써야 한컴의 새 쪽 배치를 재현한다.
        let hwpx_rowbreak_tac_missing_owned_line = st.profile.hwpx_stored_layout()
            && table.common.treat_as_char
            && table.common.flow_with_text
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && tac_count == 1
            && para.controls.len() == 1
            && table.row_count == 3
            && table.col_count == 1
            && table.cells.len() == 3
            && !table.repeat_header
            && !para_has_visible_text(para)
            && fmt.line_heights.len() == 1
            && para.line_segs.len() == 1
            && para.line_segs.first().is_some_and(|seg| {
                !is_synthetic_line_seg(seg)
                    && hwpunit_to_px(seg.line_height, self.dpi) + 0.5 < ft.total_height
            });

        // [Task #1152] 호스트 문단의 intra-paragraph vpos-reset 가드 —
        // (a) 빈-host ctrl 1:1 매핑(원형), (b) [#2322] 텍스트-host 포함 일반형:
        // 표의 매핑 lineseg(tac_seg_idx>0)가 저장 vpos==0 이면 "이 표를 새 쪽
        // 상단부터"라는 명시 신호다. fit 이전에 advance 로 존중한다.
        let ctrl_reset = ctrl_idx > 0
            && para.text.is_empty()
            && para.line_segs.len() == para.controls.len()
            && para
                .line_segs
                .get(ctrl_idx)
                .map(|s| s.vertical_pos)
                .unwrap_or(-1)
                == 0;
        let seg_reset = tac_seg_idx > 0
            && para
                .line_segs
                .get(tac_seg_idx)
                .filter(|s| !is_synthetic_line_seg(s))
                .map(|s| s.vertical_pos)
                == Some(0);
        let intra_para_reset = ctrl_reset || seg_reset;
        if intra_para_reset && !st.current_items.is_empty() {
            st.advance_column_or_new_page();
        }
        // 다중 TAC 표: LINE_SEG 기반 개별 높이 계산
        let table_height = if tac_count > 1 {
            // [#2322] 마지막 TAC 판정은 개수 기반(prior_tac) — tac_seg_idx 는
            // 선행 텍스트 줄 오프셋을 포함하므로 count 비교에 쓰지 않는다.
            let is_last_tac = prior_tac + 1 == tac_count;
            // [compat 2024] 이월 앵커 사다리: 빈 host 의 마지막 저장 seg 가
            // vpos==0(다음 쪽 상단 앵커 줄, 2022 계상)이고 이 호출이 새 단
            // 최상단에서 일어나면, 한글 2024 는 그 이월 계상을 하지 않는다
            // (156609754 pi25 실측: 2022 재저장 ls=3(이월 세그) ↔ 2024 ls=2,
            // 다음 쪽 사다리가 3,597HU 위로). 계상을 0 으로 하고 회수를 적립해
            // 그 쪽의 저장 경계 재적합 자격을 준다.
            let carried_anchor_ladder = st.profile.hangul2024_layout()
                && st.current_items.is_empty()
                && !para_has_visible_text(para)
                && para
                    .line_segs
                    .last()
                    .is_some_and(|s| s.vertical_pos == 0 && !is_synthetic_line_seg(s))
                && para
                    .line_segs
                    .get(tac_seg_idx)
                    .is_some_and(|s| s.vertical_pos > 0);
            let charged = para
                .line_segs
                .get(tac_seg_idx)
                .map(|seg| {
                    let line_h = hwpunit_to_px(seg.line_height, self.dpi);
                    if is_last_tac {
                        line_h
                    } else {
                        line_h + hwpunit_to_px(seg.line_spacing, self.dpi)
                    }
                })
                .unwrap_or(ft.total_height);
            if carried_anchor_ladder {
                st.hangul2024_reclaimed += charged;
                if std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
                    eprintln!(
                        "DIAG_COMPAT24 pi={para_idx} ci={ctrl_idx} carried_reclaim={charged:.1} \
                         total_reclaimed={:.1}",
                        st.hangul2024_reclaimed
                    );
                }
                0.0
            } else {
                charged
            }
        } else if let Some(owned_row_height) = owned_single_tac_row_height {
            owned_row_height
        } else if tac_table_line_idx == Some(0) && fmt.line_heights.len() > 1 {
            // PR #1088 follow-up: hwp-multi-001 pi=46 처럼 TAC 표가 문단의
            // 첫 줄이고 뒤따르는 제목 줄이 같은 문단의 line1(vpos reset)로
            // 인코딩된 경우가 있다. 표 자체는 현재 페이지에 들어가고 post-text
            // 만 다음 페이지로 넘어가야 하는데, 문단 전체 height_for_fit으로
            // fit 판단하면 표까지 다음 페이지로 밀린다.
            //
            // 이때 fit 기준은 line_height만 사용한다. line_spacing까지 포함한
            // line_advance를 쓰면 HWPX lineSeg가 `표줄 + 다음 텍스트줄`로
            // 분리된 문서에서, 표 자체는 남은 영역에 들어가는데도 spacing 때문에
            // 표가 다음 페이지로 밀린다(2025 donations HWPX pi=25).
            fmt.line_heights[0]
        } else if intra_para_reset && tac_seg_idx < fmt.line_heights.len() {
            // [#2311] intra-para reset(저장 vpos==0)으로 새 쪽에 온 표는 자신의
            // 줄(매핑 lineseg)부터만 계상한다. 문단 전체 height_for_fit 을
            // 쓰면 이전 쪽에 남은 선행 줄(전면 tac 그림 등)의 높이가 새 쪽에
            // 유령 계상되어 후속 문단을 한 쪽 더 밀어낸다 (156744475 4쪽→3쪽).
            (tac_seg_idx..fmt.line_heights.len())
                .map(|li| fmt.line_advance(li))
                .sum::<f64>()
        } else if st.profile.hangul2024_layout()
            && tac_seg_idx > 0
            && tac_seg_idx < fmt.line_heights.len()
        {
            // [compat 2024] 한글 2024 는 자리차지 표 앵커 문단의 선행 앵커 줄
            // 세그먼트를 흐름에 계상하지 않는다 — 2022 재저장본은 앵커 문단
            // lineseg 2개(선행 줄 + 표 밴드), 2024 재저장본은 밴드 1개
            // (output/poc/hangul_version_compat_phase0_20260818 Phase 1, Δ1).
            // 표 밴드 세그부터만 계상해 표와 후속 흐름을 그만큼 당기고, 회수량은
            // 저장 vpos 되감김 경계의 재적합 자격으로 적립한다.
            let reclaimed = (0..tac_seg_idx).map(|li| fmt.line_advance(li)).sum::<f64>();
            st.hangul2024_reclaimed += reclaimed;
            if std::env::var("RHWP_DIAG_COMPAT24").is_ok() {
                eprintln!(
                    "DIAG_COMPAT24 pi={para_idx} ci={ctrl_idx} anchor_reclaim={reclaimed:.1} \
                     tac_seg_idx={tac_seg_idx} total_reclaimed={:.1}",
                    st.hangul2024_reclaimed
                );
            }
            (tac_seg_idx..fmt.line_heights.len())
                .map(|li| fmt.line_advance(li))
                .sum::<f64>()
        } else if fmt.total_height > 0.0 {
            // 단일 TAC: 호스트 문단의 height_for_fit 사용
            fmt.height_for_fit
        } else {
            ft.total_height
        };
        // [#2319] 저장 lineseg 없는(기계생성) 문단은 fresh 컴포즈가 tac 표 높이를
        // 줄에 반영하지 못해, 위 fmt 기반 분기가 텍스트 줄높이(예: 17.3px)를
        // 858px 표의 높이로 채택한다 — 서식 문서 과소분할(−1쪽 계열 26건, r15
        // 재검증). 측정 높이보다 작으면 측정 높이로 보정한다. 저장 lineseg 보유
        // 문서는 불변 (#2237 측정-저장 발산 축과 격리).
        let table_height = if para.line_segs.is_empty() && table_height + 0.5 < ft.total_height {
            ft.total_height
        } else {
            table_height
        };
        // [#2279 stale-lh] 마스킹 도구가 셀 내용·cellSz 를 축소하고 표 선언
        // 높이(common.height)와 host lineseg lh 를 갱신하지 않은 생성기 잔존:
        // 저장 lh 기반 table_height 가 실측(ft)의 2배 이상이고 선언 표높이도
        // 같은 배율로 모순이면 실측을 신뢰한다 — 36382471 pi10 현황사진 표:
        // 선언=lh 154.2px vs 셀 실측 25px(재저장 fresh 24.6px 정합), 잔존
        // 154px 소비가 말미 2문단을 밀어 +1쪽. 정상 host 줄박스(표<글자줄,
        // 배율 ~1.1x)와 실측-이상 lh(#2243 커버형, 배율 ~1x)는 불변.
        let table_height = if !para.line_segs.is_empty()
            && ft.total_height > 1.0
            && table_height > ft.total_height * 2.0
            && hwpunit_to_px(table.common.height as i32, self.dpi) > ft.total_height * 2.0
        {
            ft.total_height
        } else {
            table_height
        };
        // [#5699 H1] 위와 대칭인 과소 방향: 저장 lineseg 보유 문서인데 저장 th 관례
        // 기반 계상이 선언·실측 표높이 둘 다의 1/4 미만이면, 생성기 사다리가 표
        // 밴드를 계상하지 않은 자기모순이다(자치법규 서식류 균일 사다리). 실측
        // 높이로 교정해 쪽 나눔이 실기하를 따르게 하고(한글 2022 재조판 정합 —
        // 영월군 20099369 오라클 4쪽 실측), 후속 저장 vpos 후방 스냅이 교정분을
        // 되돌리지 못하게 dirty 처리한다. 선언·실측 발산 문서(#2237/#2148)는
        // 정합 조건에서 배제되어 불변.
        // 게이트: ① HWPX 컨테이너 제외 — 기계 결재문서(36397752 하자검사조서)는
        // 같은 서명이어도 한글 1쪽 유지(오라클 PDF 실측)라 발동 시 +1 회귀(그 축은
        // #2279 OMIT 기계 소관). ② 직파싱 HWP3 는 tac=true "모순 조합" 표만 허용 —
        // 영월군 20099369(HWP3 V3.00, tac=true)는 한글 2022 가 재조판(오라클 4쪽,
        // 표 아래 깨끗한 배치)하는 반면, tac=false TopAndBottom 은 겹침이 한글
        // 정본인 계열(#4533 하동군)이라 기존 no-reserve 규칙에 맡긴다.
        // 밴드-앞 계상 방면: 한글 저장 사다리의 자리차지 표는 밴드가 앵커 줄 **앞**에
        // 계상되는 형(앵커 vpos = 밴드 아래)이 있다 — 직전 문단 저장 vpos 와 앵커
        // 사이 갭이 이미 밴드급이면 사다리는 정상이다(간장 보고서 3738 계열: 갭
        // 음수/대형 — 오라클-잠금 계약 5종이 th-단독 술어를 반증). 갭이 양수이면서
        // 밴드의 절반 미만일 때만 자기모순으로 본다(영월군 실측 27px vs 397px).
        let band_min_px = hwpunit_to_px(table.common.height as i32, self.dpi).min(ft.total_height);
        let anchor_vpos = para.line_segs.get(tac_seg_idx).map(|seg| seg.vertical_pos);
        let anchor_gap_px = anchor_vpos
            .zip(prev_stored_vpos)
            .filter(|(anchor, prev)| anchor > prev)
            .map(|(anchor, prev)| hwpunit_to_px(anchor - prev, self.dpi));
        // 밴드-뒤 계상(관례 A: 앵커 다음 문단 vpos 가 밴드만큼 진행) 방면 —
        // 다음 문단 갭도 함께 소형이어야 진짜 미계상이다(synam-001 pi10·간장
        // 보고서 실측: before-갭은 정상 사다리에서도 자연히 한 줄이라 단독
        // 판별자가 못 된다).
        let next_gap_px = st
            .next_para_first_stored_vpos
            .zip(anchor_vpos)
            .filter(|(next, anchor)| next > anchor)
            .map(|(next, anchor)| hwpunit_to_px(next - anchor, self.dpi));
        let ladder_omits_band = !st.profile.hwpx_container()
            && (!st.profile.hwp3_native_layout() || table.common.treat_as_char)
            && !para.line_segs.is_empty()
            && anchor_gap_px.is_some_and(|gap| gap < band_min_px * 0.5)
            && next_gap_px.is_some_and(|gap| gap < band_min_px * 0.5)
            && stored_ladder_omits_tac_band(
                table_height,
                hwpunit_to_px(table.common.height as i32, self.dpi),
                ft.total_height,
            );
        let owns_tac_band = ladder_omits_band || hwpx_rowbreak_tac_missing_owned_line;
        let table_height = if owns_tac_band {
            if std::env::var("RHWP_5699_DBG").is_ok() {
                eprintln!(
                    "DBG5699_TS pi={} ci={} th_charge={:.1} decl={:.1} meas={:.1} gap={:?} unowned_hwpx={}",
                    para_idx,
                    ctrl_idx,
                    table_height,
                    hwpunit_to_px(table.common.height as i32, self.dpi),
                    ft.total_height,
                    anchor_gap_px,
                    hwpx_rowbreak_tac_missing_owned_line,
                );
            }
            st.vpos_ladder_dirty = true;
            st.current_ladder_band_tables.push((para_idx, ctrl_idx));
            ft.total_height
        } else {
            table_height
        };

        // TAC 표는 분할하지 않고 통째로 배치
        let column = st.inline_flow_column();
        let style = styles.para_styles.get(para.para_shape_id as usize);
        let left = style.map_or(0.0, |s| s.margin_left);
        let right = style.map_or(0.0, |s| s.margin_right);
        let before_text: f64 = (0..tac_table_line_idx.unwrap_or(0))
            .map(|line| fmt.line_advance(line))
            .sum();
        let advance = hwpunit_to_px(
            (table.common.width as i32)
                .saturating_add(i32::from(table.outer_margin_left))
                .saturating_add(i32::from(table.outer_margin_right)),
            self.dpi,
        );
        // total_height에는 host 문단 간격도 들어 있다. 점유 줄 상자는 표의
        // 측정 높이와 바깥 여백만 쓰고, 문단 간격은 기존 회계가 한 번 소비한다.
        let band_height = ft.effective_height
            + hwpunit_to_px(
                i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
                self.dpi,
            );
        let exclusions: Vec<_> = st.side_wrap_exclusions.values().cloned().collect();
        let natural_top = st.current_height + before_text;
        let mut side_wrap_placement = super::float_placement::place_inline_box(
            (column.x + left)..(column.x + column.width - right),
            column.y + natural_top.max(st.inline_box_flow_bottom),
            advance,
            band_height,
            style.map_or(crate::model::style::Alignment::Left, |s| s.alignment),
            &exclusions,
            self.dpi,
        )
        .map(|mut placement| {
            placement.clearance = (placement.y - column.y - natural_top).max(0.0);
            placement
        });
        let clearance = side_wrap_placement.map_or(0.0, |p| p.clearance);
        // 그림 회피로 확정한 물리 줄은 저장 host 줄높이가 작아도 축소되지 않는다.
        // 렌더만 아래로 옮기고 fit에는 짧은 host 높이를 쓰면 쪽 하단을 넘는다.
        let table_height = if side_wrap_placement.is_some() {
            table_height.max(band_height)
        } else {
            table_height
        };
        let available = st.available_height();
        let current_column_has_only_overlay_shapes = st.current_height <= 0.5
            && st
                .current_items
                .iter()
                .all(|item| matches!(item, PageItem::Shape { .. }));
        let fits_after_overlay_shapes =
            current_column_has_only_overlay_shapes && table_height <= available + 12.0;
        let tac_trailing_spacing_for_fit = if hwpx_rowbreak_tac_missing_owned_line {
            (fmt.total_height - fmt.height_for_fit).max(0.0)
        } else {
            0.0
        };
        let saved_tac_table_frame_height =
            stored_tac_table_frame_height(table, self.dpi, table_height);
        let current_page_vpos_base = st.vpos_page_base.unwrap_or(0);
        let saved_tac_table_bottom_fits = !hwpx_rowbreak_tac_missing_owned_line
            && Some(current_page_vpos_base)
                .and_then(|base| {
                    para.line_segs
                        .get(tac_seg_idx)
                        .and_then(|seg| line_seg_visible_bounds_px(seg, base, self.dpi))
                })
                .is_some_and(|bounds| {
                    saved_table_bounds_fit_at_flow_tail(
                        bounds,
                        st.current_height,
                        available,
                        saved_tac_table_frame_height,
                    )
                });
        // [#3837] 저장 vpos 되돌아감은 한글이 이 표를 다음 쪽 맨 위에 뒀다는 신호다
        // (21967401 응시원서: 직전 항목 vpos=41645 인데 이 표는 1000).
        // 이 문단이 이 쪽에서 이미 시작했으면 걸지 않는다 — 되돌아감은 "이 문단이 새 쪽에서
        // 시작한다"는 뜻이라 이미 시작한 뒤에 걸면 문단을 쪼갠다(156483831: 그림은 4쪽,
        // 같은 문단의 표만 5쪽으로 밀려 4->5쪽).
        let same_para_already_placed = st
            .current_items
            .iter()
            .any(|it| page_item_para_index(it) == Some(para_idx));
        let stored_vpos_rewind_break = st.col_count == 1
            && !st.current_items.is_empty()
            && !same_para_already_placed
            && st.current_height >= available * STORED_VPOS_REWIND_MIN_FILL
            && stored_vpos_rewinds(prev_stored_vpos, para);
        if (st.current_height + clearance + table_height + tac_trailing_spacing_for_fit > available
            && (!fits_after_overlay_shapes || side_wrap_placement.is_some())
            && (!saved_tac_table_bottom_fits || side_wrap_placement.is_some())
            && !st.current_items.is_empty())
            || stored_vpos_rewind_break
        {
            st.advance_column_or_new_page();
            // 이전 단의 그림은 새 단을 점유하지 않는다. 이미 확정한 다른 표의 metadata는
            // flush가 전 단에 보존하고 이 표는 새 단의 기존 배치 정책으로 시작한다.
            side_wrap_placement = None;
        }

        if let Some(mut placement) = side_wrap_placement {
            st.current_height += placement.clearance;
            placement.x -= column.x;
            placement.y -= column.y;
            st.inline_placements.insert((para_idx, ctrl_idx), placement);
            st.vpos_ladder_dirty |= placement.clearance > 0.0;
        }
        // place_table_with_text의 후속 텍스트가 새 단으로 넘어가면 flush가 폐기한다.
        // 호출 뒤 갱신하면 이전 단의 표 하단을 새 단에 유출할 수 있다.
        st.inline_box_flow_bottom =
            st.current_height.max(st.inline_box_flow_bottom) + before_text + band_height;
        self.place_table_with_text(
            st,
            para_idx,
            ctrl_idx,
            para,
            table,
            fmt,
            st.current_height,
            table_height,
            is_first_placed,
            is_last_placed,
            // 이 형상은 host LINE_SEG가 표의 물리 하단을 전혀 나타내지 않는다.
            // 표 뒤 일반 문단도 실제 표 하단을 기준으로 trailing spacing까지 포함해
            // 한 번 엄격하게 적합성을 판정해야 다음 쪽으로 올바르게 이월된다.
            ft.strict_following_plain_text_fit || hwpx_rowbreak_tac_missing_owned_line,
            styles,
        );
        // [#5699 H1] 교정 계상으로 확보한 표 밴드 하단을 흐름 바닥으로 고정 —
        // 후속 문단의 저장 vpos 스냅이 밴드 위로 되감지 못한다(쪽/단 단위 리셋).
        if owns_tac_band || side_wrap_placement.is_some() {
            st.ladder_band_floor = st.ladder_band_floor.max(st.current_height);
        }
    }

    /// [#2279 TAC host] 모순 조합(treat_as_char=true + wrap=자리차지) 표의
    /// host 빈 줄박스 높이 (한글 fresh 가 표 위에 별도 배치하는 성분).
    ///
    /// 정상 문서의 TAC 표는 줄박스에 포함되어 별도 가산이 없다 — 본 판정은
    /// 기계생성 결재문서 특유의 모순 조합 + 빈 host 문단에만 발동한다.
    /// 반환 = host 첫 글자모양의 폰트 크기(px) — 한글 PDF 괘선 실측
    /// (36392557 pi27: 진입 22px ≈ om_top 1.9 + 15pt 줄박스 20px)과 정합.
    fn tac_topbottom_conflict_host_line_px(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        if !table.common.treat_as_char
            || !matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            || para_has_non_whitespace_text(para)
        {
            return None;
        }
        let cs_id = para
            .char_shape_id_at(0)
            .or_else(|| para.char_shapes.first().map(|cs| cs.char_shape_id))?
            as usize;
        let font_size = styles.char_styles.get(cs_id)?.font_size;
        if font_size <= 0.0 {
            return None;
        }
        Some(font_size)
    }

    /// 표를 pre-text/table/post-text와 함께 배치한다 (Paginator place_table_fits 동일).
    #[allow(clippy::too_many_arguments)]
    fn place_table_with_text(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
        para_start_height: f64,
        table_total_height: f64,
        is_first_placed: bool,
        is_last_placed: bool,
        strict_following_plain_text_fit: bool,
        styles: &ResolvedStyleSet,
    ) {
        // [#2279 footer-오염] PAGE-앵커 Top 절대배치 표 기록 — 같은 쪽 후속
        // footer 의 저장 vpos 동기화 차단용. 이 표들의 저장 누적은 절대 위치
        // 산물이라 본문 흐름 좌표가 아니다(36496000 pi3 실측).
        if !table.common.treat_as_char
            && matches!(
                table.common.vert_rel_to,
                crate::model::shape::VertRelTo::Page
            )
            && matches!(table.common.vert_align, crate::model::shape::VertAlign::Top)
        {
            st.page_has_page_abs_top_table = true;
        }
        let vertical_offset = Self::get_table_vertical_offset(table);
        let is_visible_para_float =
            is_para_topbottom_float(&table.common) && para_has_non_whitespace_text(para);
        // 단 오른쪽 밖으로 통째로 벗어난 자리차지 개체는 본문 세로 공간을 차지하지
        // 않는다(재현 문서 D: horz=단 227.6mm, A4 폭 210mm — 화면 밖). 이 계약은
        let signed_vertical_offset = vertical_offset as i32;
        let total_lines = fmt.line_heights.len();
        // [#5871] 공백만 있는 host 문단이 두 술어 사이 틈에 빠진다 —
        // `is_visible_para_float` 는 공백을 글자로 안 세는데(`para_has_non_
        // whitespace_text`) 아래 pre-text 판정은 `!para.text.is_empty()` 라 공백
        // 한 칸도 글자로 셌다. 그 결과 저장 줄상자를 표 앞 텍스트로 한 번 방출하고
        // 그 아래에 같은 표를 다시 그려 표가 제 높이만큼 밀렸다(10895 [별표 3]
        // 8쪽 둘째 표 667.7 → 1012.7, 본문 하한 1028.1 초과·쪽번호와 겹침).
        //
        // 다만 "공백=무텍스트" 로만 넓히면 저장 줄상자가 표와 무관한 문서에서
        // 쪽이 늘어난다(19952675 서식 6→7, 한글 6). 발동은 **저장 줄상자가 이미
        // 표를 담고 있다는 증거**가 있을 때로 한정한다 — 첫 저장 줄의 lh 가
        // 표 높이 + 위·아래 바깥여백 이상(10895: lh 24276 = 23710+283+283).
        let whitespace_host_line_covers_table = !para_has_non_whitespace_text(para)
            && !para.text.is_empty()
            && para
                .line_segs
                .iter()
                .find(|ls| !is_synthetic_line_seg(ls))
                .is_some_and(|ls| {
                    i64::from(ls.line_height)
                        >= i64::from(table.common.height.min(i32::MAX as u32))
                            + i64::from(table.outer_margin_top)
                            + i64::from(table.outer_margin_bottom)
                            - 10
                });
        let pre_table_end_line = if !is_visible_para_float
            && signed_vertical_offset > 0
            && !para.text.is_empty()
            && !whitespace_host_line_covers_table
        {
            total_lines
        } else if table.common.treat_as_char
            && total_lines > 1
            && para.text.chars().any(|c| c.is_alphanumeric())
        {
            // 전폭 TAC 표가 자동 줄바꿈으로 자기 줄(line index N)에 놓인 경우(\n 없음):
            // 한컴은 LINE_SEG 순서대로 line0=텍스트 → lineN=표 로 렌더한다.
            // control_text_positions() 는 char_offsets 가 비면 무용하므로, 표 줄의 높이
            // (표 본체 + outer margin top/bottom)와 일치하는 LINE_SEG 인덱스로 판정한다.
            // PUA 필러/공백만 있는 문단(예: 복학원서.hwp pi=16 — 한컴이 표 폭만큼 필러로
            // 줄바꿈시킨 케이스)은 is_alphanumeric() 가 false 라 제외 → compute_tac_leading
            // 경로 유지. (Task #853, Task #842 결함 #2 의 PUA 필러 판정과 정합)
            let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            let om_bot = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
            let tbl_line_h = hwpunit_to_px(table.common.height as i32, self.dpi) + om_top + om_bot;
            para.line_segs
                .iter()
                .enumerate()
                .find(|(_, ls)| (hwpunit_to_px(ls.line_height, self.dpi) - tbl_line_h).abs() < 1.0)
                .map(|(i, _)| i)
                .unwrap_or(0)
        } else {
            0
        };

        // [Task #439] Square wrap (어울림) 표 식별.
        // 어울림 표는 호스트 문단 텍스트와 같은 수직 영역에 배치되므로
        // current_height 누적은 max(host_text, v_off + table) 한 번만.
        // engine.rs:1328 와 동일 시멘틱.
        let is_wrap_around_table = !table.common.treat_as_char
            && matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::Square
            );

        // pre-table 텍스트 (첫 번째 표에서만)
        // [참고2 fix] 배열순서가 아닌 배치순서 기준 (typeset_table_paragraph 산출).
        let is_first_table = is_first_placed;
        let defer_host_line = st.defer_host_line_item_para == Some(para_idx);
        let pre_height: f64 = if pre_table_end_line > 0 && is_first_table {
            let h = fmt.line_advances_sum(0..pre_table_end_line);
            if !defer_host_line {
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: 0,
                    end_line: pre_table_end_line,
                });
            }
            h
        } else {
            0.0
        };

        // 표 배치
        // [#2243] 표가 컬럼 첫 항목이면 vpos 앵커 확립 — 표 경로는
        // vpos_snap_current_height 를 거치지 않아 page_base 가 미확립으로 남고,
        // 후속 문단 스냅이 드리프트 역산 lazy base(-796HU 등)로 고착되던 결함.
        // 저장 lineseg 가 개체 높이를 온전히 포함하는 호스트(기계생성 결재문서
        // lh = 표 + outMargin)로 한정한다.
        if st.profile.hwpx_stored_layout()
            && st.col_count == 1
            && st.current_items.is_empty()
            && pre_height <= 0.5
        {
            if let Some(s0) = para.line_segs.first() {
                let stored =
                    s0.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;
                let covers = s0.line_height as i64
                    >= table.common.height as i64
                        + table.outer_margin_top as i64
                        + table.outer_margin_bottom as i64;
                if stored && covers {
                    st.vpos_col_anchor = st.current_height;
                    st.vpos_page_base = Some(s0.vertical_pos);
                    st.vpos_lazy_base = None;
                    st.vpos_page_base_stored = true;
                    st.vpos_ladder_dirty = false;
                }
            }
        }
        st.current_items.push(PageItem::Table {
            para_index: para_idx,
            control_index: ctrl_idx,
        });

        // [Task #439] 누적 정책:
        // - Square wrap (어울림): max(pre_height, v_off + table_total)
        //     호스트 텍스트와 표가 같은 y 영역을 공유하므로 더 큰 쪽만 누적.
        // - 그 외 (TopAndBottom 등): pre_height + table_total 합산 (기존 동작).
        // 전폭 TAC 표가 자기 줄(line index = pre_table_end_line)에 놓인 split 케이스:
        // table_total_height(=fmt.height_for_fit)는 pre-text 줄까지 포함하므로 pre_height
        // 를 따로 더하면 이중 계산이 된다. 또 표가 차지한 줄은 post-text 에서 제외해야 한다.
        // (Task #853)
        let tac_wrap_split = table.common.treat_as_char
            && pre_table_end_line > 0
            && pre_table_end_line < total_lines;
        let has_preceding_coanchored_float = is_visible_para_float
            && para.controls.iter().take(ctrl_idx).any(|control| {
                matches!(control, Control::Table(previous)
                    if is_para_topbottom_float(&previous.common))
            });
        let issue2439_visible_host_stack = st.profile.hwp5_stored_pagination_layout()
            && st.col_count == 1
            && st.current_zone_y_offset.abs() < 0.5
            && is_visible_para_float
            && signed_vertical_offset > 0
            && has_preceding_coanchored_float
            && para
                .controls
                .iter()
                .filter(|control| matches!(control, Control::Table(_)))
                .count()
                == 2
            && para.line_segs.iter().any(|seg| !is_synthetic_line_seg(seg));

        if is_wrap_around_table && pre_height > 0.0 {
            let v_off_px = crate::renderer::hwpunit_to_px(vertical_offset as i32, self.dpi);
            let table_bottom = v_off_px + table_total_height;
            st.current_height += pre_height.max(table_bottom);
        } else if is_visible_para_float {
            // 통째 배치의 fit 판정에서 확정한 결과를 그대로 소비한다.
            // 이월 뒤에는 이전 단의 원점으로 배치를 재생성하지 않는다.
            let resolved = st
                .paragraph_float_placements
                .get(&(para_idx, ctrl_idx))
                .copied();
            let v_off_px = hwpunit_to_px(signed_vertical_offset, self.dpi);
            let outer_top_px = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            let table_top = if let Some(placement) = resolved {
                placement.table_top
            } else if signed_vertical_offset > 0 {
                // [#6879] 세로 기준점은 앵커 줄이다 — layout 이 같은 값을 더하므로
                // 흐름 예약도 함께 내려야 배치와 어긋나지 않는다. layout 과 **같은**
                // 게이트(TAC 형제 유무)를 써야 배치와 예약이 갈리지 않는다.
                let anchor_offset_px = crate::renderer::layout::tac_sibling_float_anchor_offset_px(
                    para, table, ctrl_idx, self.dpi,
                );
                let stored_top = para_start_height + anchor_offset_px + outer_top_px + v_off_px;
                // [#2439] 같은 visible host 의 첫 표가 offset=0이면 flow 를 전진시키지만
                // exclusion 은 만들지 않는다. 후행 양수-offset 표의 저장 상단이 그 표
                // 내부에 있으면 한컴은 앞 표 아래로 밀어 전체 높이를 보존한다. 저장
                // 상단/하단만 쓰면 후행 exclusion 높이가 겹친 만큼 잘려 후속 본문이
                // 표 안으로 들어가므로, 이미 소비한 co-anchored flow 를 하한으로 둔다.
                if has_preceding_coanchored_float {
                    let stacked_top = if issue2439_visible_host_stack {
                        // #2439: native HWP의 저장된 다중 visible-host 표는 선행 표에
                        // 밀려난 뒤에도 후행 표 자신의 outer-top을 보존한다. 이를
                        // 빼면 표 스택과 후속 안내문이 매 양식마다 약 3.8px 앞당겨진다.
                        st.current_height + outer_top_px
                    } else {
                        st.current_height
                    };
                    stored_top.max(stacked_top)
                } else {
                    stored_top
                }
            } else if st.profile.hwpx_stored_layout() {
                // HWPX visible float 는 같은 문단 안의 앞선 float 뒤에 이어 쌓인다.
                // B/C처럼 둘 다 non-positive offset 이면 문단 시작점이 아니라 현재 흐름
                // 높이를 기준으로 reserve 해야 layout 의 세로 stacking 과 page break 가 맞는다.
                let flow_at_para_start = (st.current_height - para_start_height).abs() < 0.5;
                st.current_height
                    + if flow_at_para_start {
                        outer_top_px
                    } else {
                        0.0
                    }
                    + v_off_px.max(0.0)
            } else {
                para_start_height + outer_top_px + v_off_px
            };
            let table_bottom = table_top + table_total_height.max(0.0);
            let table_bottom = if let Some(mut placement) = resolved {
                placement.occupied_bottom += table_top - placement.table_top;
                placement.table_top = table_top;
                st.paragraph_float_placements
                    .insert((para_idx, ctrl_idx), placement);
                placement.occupied_bottom
            } else {
                table_bottom
            };
            // 단 오른쪽 밖 개체는 배제 영역도 만들지 않는다 — 배제 영역을 만들면
            // 뒤따르는 본문 표가 그만큼 밀려 마지막 블록이 다음 쪽으로 넘어간다.
            if signed_vertical_offset > 0 {
                if table_bottom > table_top + 0.5 {
                    st.visible_float_exclusions.push(VisibleFloatExclusion {
                        para_index: para_idx,
                        top: table_top,
                        bottom: table_bottom,
                    });
                }
                if issue2439_visible_host_stack {
                    // 이 저장 형상은 후행 표가 선행 표 아래로 실제 stacking되어
                    // host 흐름을 소비한다. native 일반 float처럼 현재 높이를
                    // 그대로 두면 post-text probe가 outer-top 틈 앞에서 멈춘다.
                    st.current_height = st.current_height.max(table_bottom);
                } else {
                    st.current_height += pre_height;
                }
            } else {
                let following_non_positive =
                    has_following_non_positive_visible_float(para, ctrl_idx);
                let inter_float_gap = if st.profile.hwpx_stored_layout() && following_non_positive {
                    para_line_spacing_px(para, self.dpi)
                } else {
                    0.0
                };
                st.current_height = st.current_height.max(table_bottom + inter_float_gap);
            }
        } else if tac_wrap_split {
            st.current_height += table_total_height;
        } else if let Some(host_spacing_px) = if st.profile.hwpx_stored_layout()
            && is_last_placed
            && total_lines <= pre_table_end_line + 1
            && fmt.spacing_before + fmt.spacing_after > 0.5
            && self
                .tac_topbottom_conflict_host_line_px(para, table, styles)
                .is_some()
        {
            // [#2279 10k] host spacing 가산은 **문단당 1회** — 다중 표 문단에서
            // 표마다 가산하면 중간 표 배치의 fit 이 과대해져 후속 표가 조기
            // 개행된다 (156767148 보도자료 pi7: 표2개 문단, 한글 1쪽 vs +1쪽,
            // 10k 회귀 +29건 군집의 지배 형상). 마지막 표 배치 시점으로 이연.
            // [#2279 sec0] host 문단에 표 줄 외 추가 줄박스(트레일러 빈 줄)가
            // 이미 있으면 그 줄의 fmt 가 spacing 을 계상하므로 제외(36392557
            // sec0 표지: 표 892.5 + 트레일러 25.6 = 918.1 로 한글 1쪽).
            Some(fmt.spacing_before + fmt.spacing_after)
        } else {
            None
        } {
            // [#2279 sb+α 종결] 기계생성 문서의 TAC-모순(treat_as_char=true +
            // wrap=자리차지) host 표: 한글 fresh 는 host CS 크기의 "빈 줄박스"가
            // 아니라 host paraPr 의 **sb+sa 를 순수 가산**한다(α=0) — 재저장
            // ladder 실측: 36399374 pi4→5 = bare + (pi4.sa 500 + pi5.sb 300)
            // 정확, pi3(sb/sa=0)→4 = bare, 계열X 보도자료 156745609 host 4개
            // 전부 bare. 종전 font_size 근사(#2352 CS-근사)는 2557 pi27
            // (sb 1000)에서 우연히 ±2px 로 맞았던 것. 가산 후에는 생성기 압축
            // anchor 로의 후방 스냅이 성장분을 되돌리지 못하게 dirty 처리.
            st.current_height += host_spacing_px + pre_height + table_total_height;
            st.vpos_ladder_dirty = true;
            // [#2279 sb+α 진단] host spacing 가산 내역. 동작 불변.
            if std::env::var("RHWP_DIAG_TAC").is_ok() {
                eprintln!(
                    "DIAG_HOSTBOX pi={} add={:.1} host_sb={:.1} host_sa={:.1} stored_gap_hu={}",
                    para_idx,
                    host_spacing_px,
                    fmt.spacing_before,
                    fmt.spacing_after,
                    para.line_segs.first().map(|s| s.line_spacing).unwrap_or(-1),
                );
            }
        } else {
            // [#2097 프로브 기록] 빈 host 자리차지 float(v_off>0)의 흐름 전진에
            // v_off + outer_bottom 을 더하는 기하 정합(82802 pi75: 저장 322.6 =
            // v_off 21.6 + outer 3.8 + 표 297.2, rhwp 299.1)은 격리 수정으로
            // 반증됨: 페어 상대측(후속 NO_LS 빈 문단 +12 재계산 축)과 결합되어
            // 순효과가 반전되고, v_off 를 저장이 소비하지 않는 하위 형상
            // (pi114: +20.5→+44.1 악화) 존재. 82802 56→57(hc=51) 악화 실측.
            // vpos-스냅/NO_LS 축과의 동시 정합 없이는 적용 불가.
            //
            // [#4090] 단독 Square 어울림 표는 한글이 옆으로 글을 흘린다. 저장 사다리의 host
            // 줄높이가 표 높이의 1/4 미만이면 한글은 표 높이를 흐름에 예약하지 않은 것이다.
            // 그때는 흐름을 host 줄만큼만 전진시키고 표 높이는 **세로 배제 밴드**로 잡는다 —
            // 밴드를 벗어나거나 쪽이 끝날 때 `close_square_band` 가 흐름을 밴드 바닥으로
            // 끌어올린다. 전진량만 줄이면(밴드 없이) 텍스트가 표를 지나쳐 계속 옆으로 흘러
            // 과소가 된다(실측: 156492236 24쪽 → 14쪽, 정답 17).
            let stored_host_line_px = para
                .line_segs
                .iter()
                .find(|seg| !is_synthetic_line_seg(seg))
                .map(|seg| crate::renderer::hwpunit_to_px(seg.line_height as i32, self.dpi));
            let hangul_flowed_beside_table = is_wrap_around_table
                && table_total_height > 1.0
                && stored_host_line_px.is_some_and(|lh| lh < table_total_height * 0.25);
            // [#4533 HWP3] 비-tac TopAndBottom float 인데 저장 사다리가 표를
            // 예약하지 않은 서식 문서(하동군 21918361: host lh 13.3px·다음 문단
            // 델타 21.3px vs 표 730px — 표·텍스트 겹침이 한글의 정본인데
            // typeset/layout 이 +709 과소비). host lh 와 다음 문단 저장 델타가
            // 둘 다 표 높이의 1/4 미만이면 흐름은 host 줄만 전진한다. HWP5 는
            // 같은 서명이 2중 반증된 환원불가 계열이라 HWP3 계보 한정.
            let next_gap_px = st
                .next_para_first_stored_vpos
                .zip(para.line_segs.first().map(|seg| seg.vertical_pos))
                .filter(|(nv, hv)| nv > hv)
                .map(|(nv, hv)| crate::renderer::hwpunit_to_px(nv - hv, self.dpi));
            // 게이트: 직파싱 HWP3 + rhwp 자신의 HWPX 산출물(마커 계보)만.
            // HWP5 컨테이너 변환본은 한글이 재저장하며 재조판한 것이라 저장
            // lineseg 가 HWP5 계약이다 — 환경정책기본법 1480000-201600147 실측:
            // hwp3_lineage 휴리스틱 HWP5 에 발화하면 52.7→369.8 악화.
            let hwp3_topbottom_no_reserve = (st.profile.hwp3_native_layout()
                || (st.profile.hwp3_layout() && st.profile.hwpx_container()))
                && !table.common.treat_as_char
                && matches!(
                    table.common.text_wrap,
                    crate::model::shape::TextWrap::TopAndBottom
                )
                && table_total_height > 1.0
                && stored_host_line_px.is_some_and(|lh| lh < table_total_height * 0.25)
                && next_gap_px.is_some_and(|g| g < table_total_height * 0.25);
            // [#5870] 빈 host 자리차지 float(vert=문단)의 흐름 전진에 v_off 와
            // 위·아래 바깥여백을 계상한다 — 단, 이 문단의 저장 사다리가 그 물리
            // 공식과 정확히 일치할 때만(`empty_host_physical_ladder_extras_hu`,
            // #2097 반증 회피 근거도 그쪽 주석에). 합성 lineseg(HWP3 계열)는
            // rhwp 가 물리식으로 만들어 자기참조가 되므로 실저장 줄만 증거로
            // 인정하고, 다중 표 host 는 델타가 표 합이라 단일 표 등식에 걸리지
            // 않는다(표 1개 조건으로 명시). layout 의 lane flow-bottom 가산과
            // 대칭이어야 조판·렌더가 어긋나지 않는다.
            let empty_host_physical_extras_hu = ((st.profile.hwp5_stored_pagination_layout()
                || st.profile.hwpx_stored_layout())
                && st.next_para_is_empty_float_table_anchor
                && is_para_topbottom_float(&table.common)
                && !para_has_non_whitespace_text(para)
                && para
                    .controls
                    .iter()
                    .filter(|control| matches!(control, Control::Table(_)))
                    .count()
                    == 1)
                .then(|| {
                    st.next_para_first_stored_vpos
                        .zip(
                            para.line_segs
                                .iter()
                                .find(|seg| !is_synthetic_line_seg(seg))
                                .map(|seg| seg.vertical_pos),
                        )
                        .and_then(|(next_vpos, host_vpos)| {
                            crate::renderer::float_placement::empty_host_physical_ladder_extras_hu(
                                table, host_vpos, next_vpos,
                            )
                        })
                })
                .flatten();
            if hwp3_topbottom_no_reserve {
                st.current_height += pre_height + stored_host_line_px.unwrap_or(0.0);
            } else if hangul_flowed_beside_table {
                let band_top = st.current_height + pre_height;
                st.current_height = band_top + stored_host_line_px.unwrap_or(0.0);
                st.square_band_bottom = st.square_band_bottom.max(band_top + table_total_height);
                st.square_band_top = Some(band_top);
            } else if let Some(extras_hu) = empty_host_physical_extras_hu {
                st.current_height +=
                    pre_height + table_total_height + hwpunit_to_px(extras_hu as i32, self.dpi);
            } else {
                st.current_height += pre_height + table_total_height;
            }
        }
        // [#2243 진단] TAC 표 라인 회계 분해 — 동작 불변.
        if std::env::var("RHWP_DIAG_TAC").is_ok() {
            eprintln!(
                "DIAG_TAC pi={} tac={} pre_h={:.1} table_total={:.1} fmt_total={:.1} fmt_fit={:.1} stored_ls={} cur_h_after={:.1} page={}",
                para_idx,
                table.common.treat_as_char,
                pre_height,
                table_total_height,
                fmt.total_height,
                fmt.height_for_fit,
                !para.line_segs.is_empty(),
                st.current_height,
                st.pages.len(),
            );
        }

        // [#2813] 이연된 host 앵커 줄 아이템 — 마지막 float 뒤에 한글 문서순으로
        // 삽입한다. 렌더는 이 순서대로 표를 흐름 상단부터, 줄을 저장 vpos 로 놓는다.
        if defer_host_line && is_last_placed {
            st.current_items.push(PageItem::PartialParagraph {
                para_index: para_idx,
                start_line: 0,
                end_line: pre_table_end_line.max(1),
            });
            st.defer_host_line_item_para = None;
        }

        // post-table 텍스트
        let is_last_table = is_last_placed;
        let tac_table_count = para
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Table(t) if self.is_effective_tac_table(para, t, fmt)))
            .count();
        let post_table_start = if tac_wrap_split {
            (pre_table_end_line + 1).min(total_lines).max(1)
        } else if self.uses_tac_table_flow(table) {
            pre_table_end_line.max(1)
        } else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
            // HWPX TAC 표(attr 비트0=0): 표줄(pre_table_end_line) 다음에 실제 본문 줄이
            // 있으면 표줄을 post-text 에서 제외(HWP5 attr&0x01 의 pre_end.max(1) 와 정합).
            // 단일 줄(표줄만)은 건드리지 않아 기존 동작 보존.
            pre_table_end_line + 1
        } else if is_last_table && !is_first_table {
            0
        } else {
            pre_table_end_line
        };
        // 중복 방지: 이전 표가 이미 같은 문단의 pre-text(start_line=0)를 추가했으면 건너뜀
        // (engine.rs:1418-1421 와 동일한 가드 — 다중 TopAndBottom 표 문단에서
        //  같은 line 범위가 두 번 emit되어 본문이 두 번 렌더되는 문제 차단)
        // [#6184] 이 가드는 `current_items` 만 보는데, 표가 쪽을 넘긴 뒤에는 그것이
        // **새 쪽**의 항목이라 앞 쪽에 pre-emit 해 둔 host 줄을 못 본다. 그러면 같은
        // 줄이 두 쪽에 모두 그려진다(156489124 pi=324: 12쪽 1030.3 과 13쪽 75.6).
        // pre-emit 기록은 쪽을 넘어 남으므로 함께 본다.
        let pre_text_exists = post_table_start == 0
            && (st.pre_emitted_host_paras.contains(&para_idx)
                || st.current_items.iter().any(|item| {
                    matches!(item, PageItem::PartialParagraph { para_index, start_line, .. }
                    if *para_index == para_idx && *start_line == 0)
                }));
        let has_substantive_text = para_has_non_whitespace_text(para);
        let whitespace_only_single_tac_host_line = !has_substantive_text
            && !para.text.is_empty()
            && table.common.treat_as_char
            && pre_table_end_line == 0
            && total_lines <= 1;
        let has_post_text = !para.text.is_empty()
            && total_lines > post_table_start
            && !whitespace_only_single_tac_host_line;
        let should_add_post_text =
            is_last_table && tac_table_count <= 1 && has_post_text && !pre_text_exists;
        if should_add_post_text {
            let post_height: f64 = fmt.line_advances_sum(post_table_start..total_lines);
            if let Some(origin) = st
                .paragraph_float_placements
                .get(&(para_idx, ctrl_idx))
                .and_then(|placement| placement.stored_host_origin)
            {
                // The host was resolved before fit, even though its text item is
                // emitted after the floating table. Consume that same origin.
                st.current_height = origin;
            }
            // [#2808] 소비 조건을 layout 의 same_owner_table_precedes 와 동일하게
            // 다중 co-anchored float host 로 한정 — 단일 표 host post-text 는 기존
            // 앵커 유지(#1549) 경로로 남긴다.
            if is_visible_para_float && has_preceding_coanchored_float {
                // [#2439] 다중 visible-host float 의 host 텍스트가 마지막 표 뒤에서
                // emit 되는 경우, 같은 문단 소유 exclusion 도 post-text 에는 적용한다.
                // 첫 표 offset=0 뒤의 양수-offset 표가 아래로 밀렸을 때 이 동기화가
                // 없으면 서명란이 두 번째 표의 상단에 겹치고 flow도 한 표 높이만큼
                // 과소 소비된다. 선행 제목은 pre-table 경로라 영향 없다.
                st.apply_visible_float_exclusions(post_height);
                if issue2439_visible_host_stack && is_last_table {
                    // 마지막 표의 exclusion은 표 본체까지만 담는다. 한컴 저장 흐름은
                    // 그 뒤의 outer-bottom과 host LineSeg 간격을 거친 뒤 서명문을
                    // 배치하므로, 해당 두 성분을 post-text 앞에서 한 번만 복원한다.
                    st.current_height += hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
                        + para_line_spacing_px(para, self.dpi);
                }
            }
            // [편집 세션] 자리차지(topbottom) 표가 Enter로 자라 post-text가 본문
            // 하한을 넘으면 문구를 새 쪽으로 보낸다. 저장 형상 열람은 저장 vpos를
            // 신뢰하므로 제외한다(셀 끝 Enter 재현: 하단 문구·로고가 페이지
            // 밖으로 잘리던 구간).
            let session_grown_topbottom_spill = self.profile.get().session_edited()
                && !table.common.treat_as_char
                && is_para_topbottom_float(&table.common);
            if (self.tac_table_line_index(para, table, fmt) == Some(0)
                || session_grown_topbottom_spill)
                && st.current_height + post_height > st.available_height() + 0.5
                && !st.current_items.is_empty()
            {
                st.advance_column_or_new_page();
            }
            st.current_items.push(PageItem::PartialParagraph {
                para_index: para_idx,
                start_line: post_table_start,
                end_line: total_lines,
            });
            st.current_height += post_height;
        } else if is_visible_para_float && is_last_table {
            // [#6312] host 글줄을 post-text 로 방출하지 못한 자리차지 표.
            // 저장 사다리가 lh+ls 만 증언하면 그 줄 상자를 표 밴드 아래에 계상한다.
            let next_vpos = if st.next_para_is_plain_text {
                st.next_para_first_stored_vpos
            } else {
                None
            };
            if let Some(line_advance) = stored_visible_anchor_band_host_line_advance_from_vpos(
                st.profile.hwp5_stored_pagination_layout() || st.profile.hwpx_stored_layout(),
                para,
                ctrl_idx,
                next_vpos,
            ) {
                st.current_height += hwpunit_to_px(line_advance, self.dpi);
            }
        }

        // TAC 표: trailing line_spacing 복원 (Paginator place_table_fits:777-783 동일)
        // has_post_text는 tac_table_count와 무관하게 텍스트 줄 존재 여부만 확인
        let is_tac = self.is_effective_tac_table(para, table, fmt);
        if is_tac && !has_post_text {
            st.current_height += fmt.total_height - fmt.height_for_fit;
        }
        // [#2243 진단] 배치 종료 시 누적 — 동작 불변.
        if std::env::var("RHWP_DIAG_TAC").is_ok() {
            eprintln!(
                "DIAG_TAC_END pi={} cur_h={:.1} trailing_fired={} has_post_text={} delta={:.1}",
                para_idx,
                st.current_height,
                is_tac && !has_post_text,
                has_post_text,
                fmt.total_height - fmt.height_for_fit,
            );
        }
        if strict_following_plain_text_fit && is_last_placed {
            st.strict_plain_text_fit_after_empty_host_float_once = true;
        }
    }

    fn tac_flow_query(&self) -> controls::tac_flow::TacFlowQuery<'_> {
        controls::tac_flow::TacFlowQuery::new(self.dpi, &self.profile)
    }

    fn tac_table_line_index(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> Option<usize> {
        self.tac_flow_query().tac_table_line_index(para, table, fmt)
    }

    fn is_effective_tac_table(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> bool {
        self.tac_flow_query()
            .is_effective_tac_table(para, table, fmt)
    }

    fn uses_tac_table_flow(&self, table: &crate::model::table::Table) -> bool {
        self.tac_flow_query().uses_tac_table_flow(table)
    }

    /// 비-TAC 블록 표의 조판: fits → place / split(Break Token 기반).
    /// 기존 Paginator의 split_table_rows와 동일한 세밀한 분할 로직.
    #[allow(clippy::too_many_arguments)]
    fn pre_emit_visible_rowbreak_host_text(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) -> bool {
        if st.col_count != 1 || st.current_items.is_empty() || !para_has_visible_text(para) {
            return false;
        }
        if st.pre_emitted_host_paras.contains(&para_idx) {
            return true;
        }
        let already_emitted = st.current_items.iter().any(|item| {
            matches!(item, PageItem::FullParagraph { para_index }
                | PageItem::PartialParagraph { para_index, start_line: 0, .. }
                if *para_index == para_idx)
        });
        if already_emitted {
            st.pre_emitted_host_paras.insert(para_idx);
            return true;
        }
        let col_w = st
            .layout
            .column_areas
            .get(st.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(st.layout.body_area.width);
        let host_fmt = self.format_paragraph(para, composed_all.get(para_idx), styles, Some(col_w));
        let host_lines = host_fmt.line_heights.len();
        let host_h = host_fmt.line_advances_sum(0..host_lines);
        // [Task #1763] 저장 flow 로 같은 쪽 후보임이 확인된 경우와 동일하게, host
        // 줄 자체는 추가 안전마진 없이 본문 가용 높이로 판정한다.
        //
        // [#6184] 단 **fit 판정에는 말미 줄간격을 빼고** 잰다 — 쪽 마지막 줄 뒤에는
        // 다음 줄이 없으므로 그 간격은 쪽을 채우지 않는다(#359 의 `height_for_fit`
        // 규약이고, 바로 아래 후속 문단 루프도 이미 그 값을 쓴다). 포함해서 재면
        // 156489124 pi=324 가 24.0px 로 잡혀 잔여 16.6px 를 넘겨 탈락하고, 줄이
        // 표와 함께 다음 쪽으로 밀려 그 쪽 흐름표 위에 겹쳐 그려진다. 한글은 같은
        // 줄을 1031.2..1047.2 로 본문 하단(1046.9)을 0.3px 넘겨 놓는다 — 말미
        // 간격을 요구하지 않는다는 뜻이다. 누적(advance)은 종전대로 전량.
        let host_fit = host_fmt.height_for_fit.min(host_h);
        if host_lines == 0 || st.current_height + host_fit > st.available_height() {
            return false;
        }
        st.current_items.push(PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: 0,
            end_line: host_lines,
        });
        st.current_height += host_h;
        st.pre_emitted_host_paras.insert(para_idx);
        // [#2015] vert_offset 이중계상 보정용 host 높이 기록.
        st.pre_emitted_host_heights.insert(para_idx, host_h);
        true
    }

    /// [Task #1753] 지연 이월되는 visible-host 자리차지 표의 후속 문단 선행 채움.
    ///
    /// 한글은 자리차지(TopAndBottom·vert=Para) RowBreak 표가 현재 쪽 잔여 공간에 안
    /// 들어가 다음 쪽으로 이월될 때, 후속 텍스트를 현재 쪽 잔여 공간에 먼저 채운다
    /// (fill-before-deferred-float — 2814765 pi52/53, 한글 PDF·저장 LINE_SEG 정합).
    /// rhwp 순차 모델에서는 이월 직전에 후속 control-free 문단들을 현재 쪽에 선행
    /// 배치하고 `prefilled_paras` 로 메인 루프에서 스킵한다.
    ///
    /// 가드: 단일 단 + 현재 쪽에 항목 존재 + 텍스트 anchor + RowBreak 자리차지 표
    /// (v_off ≥ 0). 후보 문단은 저장 첫 실줄 vpos 가 (host vpos, 본문높이HU] 구간
    /// (같은 쪽 연속 인코딩 — 누적좌표 문서는 자연 배제)이고 누적높이 fit 일 때만.
    #[allow(clippy::too_many_arguments)]
    fn prefill_before_deferred_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        paragraphs_all: &[Paragraph],
        composed_all: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
    ) {
        const MAX_PREFILL: usize = 8;
        if st.col_count != 1 || st.current_items.is_empty() {
            return;
        }
        if !para_has_visible_text(para)
            || !crate::renderer::float_placement::is_para_topbottom_float(&table.common)
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
        {
            return;
        }
        let Some(host_seg) = para.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)) else {
            return;
        };
        let host_vpos = host_seg.vertical_pos;
        let body_h_hu = crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
        // [#6184] 음수 세로 오프셋은 표를 host 줄 **위**로 들어올리므로 종전에는
        // pre-emit 자체를 막았다. 그러나 저장 사다리가 "이 줄은 이 쪽 본문 안에서
        // 끝난다"고 증언하면 한글은 그 줄을 이 쪽 마지막 줄로 두고 표만 넘긴다
        // (156489124 pi=324: host vpos 71604 + lh 1200 = 72804 ≤ 본문 72847,
        // 다음 문단 vpos 6863 으로 리셋 — 한글 실측도 그 줄이 12쪽 1031.2).
        // 막아 두면 줄이 표와 함께 넘어가 다음 쪽 흐름표 위에 겹쳐 그려진다.
        // 증거 없는 음수(줄이 본문을 넘거나 다음 문단이 이어지는 경우)는 종전대로.
        if signed_hwpunit(table.common.vertical_offset) < 0 {
            let line_ends_inside = host_vpos >= 0
                && i64::from(host_vpos) + i64::from(host_seg.line_height) <= i64::from(body_h_hu);
            let next_resets = paragraphs_all
                .get(para_idx + 1)
                .and_then(|next| next.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)))
                .is_some_and(|seg| seg.vertical_pos < host_vpos);
            if !(line_ends_inside && next_resets) {
                return;
            }
        }
        // [Task #1811] HWPX 의 누적좌표 RowBreak 문서는 host 줄 pre-emit 자체를
        // 저장 vpos 가드보다 먼저 수행한다. 쪽 내부 vpos 를 가진 HWP/HWP3/일반 HWPX
        // 경로는 기존처럼 같은 쪽 저장 flow 확인 뒤에만 pre-emit 한다.
        let pre_emit_before_vpos_check = st.profile.hwpx_stored_layout() && host_vpos > body_h_hu;
        if pre_emit_before_vpos_check
            && !self.pre_emit_visible_rowbreak_host_text(st, para_idx, para, composed_all, styles)
        {
            return;
        }
        if host_vpos < 0 || host_vpos > body_h_hu {
            return; // 누적좌표 등 — 저장 flow 로 같은 쪽 여부를 알 수 없음
        }
        let col_w = st
            .layout
            .column_areas
            .get(st.current_column as usize)
            .map(|a| a.width)
            .unwrap_or(st.layout.body_area.width);
        if !pre_emit_before_vpos_check
            && !self.pre_emit_visible_rowbreak_host_text(st, para_idx, para, composed_all, styles)
        {
            return;
        }
        let end = paragraphs_all.len().min(para_idx + 1 + MAX_PREFILL);
        for next_idx in (para_idx + 1)..end {
            let next = &paragraphs_all[next_idx];
            if !next.controls.is_empty() {
                break;
            }
            let Some(seg) = next.line_segs.iter().find(|ls| !is_synthetic_line_seg(ls)) else {
                break;
            };
            // 저장 flow 가 host 와 같은 쪽 연속임을 인코딩한 경우만.
            if seg.vertical_pos <= host_vpos
                || seg.vertical_pos.saturating_add(seg.line_height) > body_h_hu
            {
                break;
            }
            let fmt_n =
                self.format_paragraph(next, composed_all.get(next_idx), styles, Some(col_w));
            if st.current_height + fmt_n.height_for_fit > st.available_height() {
                break;
            }
            let trim_sb =
                !st.profile.hwp3_layout() && !para_near_rowbreak_table(paragraphs_all, next_idx);
            st.current_items.push(PageItem::FullParagraph {
                para_index: next_idx,
            });
            st.current_height += fmt_n.flow_advance_height(
                next,
                st.col_count,
                trim_sb,
                st.vpos_ladder_dirty
                    || !spacing_trim_restorable(paragraphs_all, next_idx)
                    || next_boundary_reverts_spacing_trim(
                        st.profile.hwpx_stored_layout() && !st.profile.hwp3_layout(),
                        paragraphs_all,
                        styles,
                        next_idx,
                        self.dpi,
                    ),
                false,
            );
            st.prefilled_paras.insert(next_idx);
        }
    }

    // ========================================================
    // 다단 나누기 처리
    // ========================================================

    fn process_multicolumn_break(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        page_def: &PageDef,
    ) {
        st.flush_column();

        // [Task #874 Case 5] leaving zone 의 height 계산 시 마지막 라인의 trailing
        // line_spacing 을 제외한다. zone 간 gap 은 design_spacing/2 + solo_zone_pad 가
        // 이미 담당하므로 vpos_zone_height 에 trailing_ls 까지 더하면 이중 가산.
        // 한컴 PDF 측정 (shortcut.hwp 1쪽): 본문 첫 줄 top 195.3 px (Hancom) vs 210.7 px
        // (rhwp pre) = +15.4 px (≈11.5pt) 넓다. 제목 paragraph 의 trailing_ls 16 px 이
        // vpos_zone_height 에 포함되어 다음 zone(헤더 띠 + 본문)을 일괄 16 px 하향.
        // pi=80 (21_언어_기출_편집가능본 test_544) 회귀 없음 — pi=80 은 zone 내부 box
        // 인접 paragraph 로 trailing_ls 가 layout 의 y_offset 에서 포함됨 (이 변경은 zone
        // 전환 시의 vpos_zone_height 만 수정).
        let vpos_zone_height = if para_idx > 0 {
            let mut max_vpos_end: i32 = 0;
            let mut prev_is_floating_anchor = false;
            for prev_idx in (0..para_idx).rev() {
                if let Some(last_seg) = paragraphs[prev_idx].line_segs.last() {
                    let vpos_end = last_seg.vertical_pos.saturating_add(last_seg.line_height);
                    if vpos_end > max_vpos_end {
                        max_vpos_end = vpos_end;
                    }
                    prev_is_floating_anchor =
                        crate::renderer::layout::para_is_floating_overlay_anchor(
                            &paragraphs[prev_idx],
                        );
                    break;
                }
            }
            // [#2019 부분 완화] leaving zone 높이는 "이 페이지에서 콘텐츠가 내려간 높이"여야
            // 한다. 별지 서식처럼 stored vpos 가 섹션 누적 좌표인 문서에서는 max_vpos_end 가
            // 페이지 높이를 크게 넘어 zone 전환마다 새 페이지가 생기므로, 우선 흐름 누적값
            // (st.current_height, page-상대)을 대신 쓴다. 이 역시 완전한 한글 모델은 아니며,
            // Paper 앵커 object extent 기반 page-local 계산으로 교체되어야 한다.
            let max_vpos_px = hwpunit_to_px(max_vpos_end, self.dpi);
            if max_vpos_end > 0
                && !prev_is_floating_anchor
                && max_vpos_px <= st.layout.available_body_height()
            {
                // 사다리는 **한글의 쪽 경계** 기준이라, 한글이 이미 쪽을 끊은 자리에서는
                // 직전 문단의 vpos 가 다음 쪽 상단 값이 되어 이 쪽이 실제로 소비한 높이를
                // 크게 밑돈다. 그러면 아래 Task #853 가드가 "아직 여유 있다" 로 오판해
                // 새 zone 을 같은 쪽에 얹는다 (2990099 주파수 분배표 115쪽: 사다리 76px
                // vs 실소비 867.4px — zone 두 개가 1657.3px 로 본문 876.9px 에 겹쳐
                // 745.7px 이 쪽 밖으로 나갔다). 흐름 누적은 이 쪽에 실제로 놓인 양이므로
                // 둘 중 큰 값이 이 쪽의 소비량이다.
                max_vpos_px.max(st.current_height)
            } else {
                st.current_height
            }
        } else {
            st.current_height
        };
        // [Task #853] zone 전환 시 디자인 spacing(1단 ColumnDef 의 `간격`)을 세로 간격으로:
        // (이전 zone 디자인 spacing /2) + (새 zone 디자인 spacing /2) 를 더한다.
        // shortcut.hwp 1쪽: 제목 zone(0mm) → 헤더 띠 zone(10mm) → 본문 zone(2단, 0)
        //   → 제목↔헤더 = 5mm, 헤더↔본문 = 5mm (한컴 PDF 정합).
        let new_ds = paragraphs[para_idx]
            .controls
            .iter()
            .find_map(|c| {
                if let Control::ColumnDef(cd) = c {
                    Some(column_def_design_spacing_px(cd, self.dpi))
                } else {
                    None
                }
            })
            .unwrap_or(0.0);
        // [Task #866] 직전 zone 의 마지막 paragraph 가 wrap=위아래 인 글자처럼-취급 표(헤더 띠)를
        // 보유하고 그 zone 의 1단 ColumnDef 간격이 0 이면, 한컴은 표 band 높이(표 본체 +
        // outer_margin top/bottom)만큼을 표 아래에 추가로 비워둔다(한컴 PDF 측정:
        // shortcut.hwp 2·3쪽 헤더 띠 하단↔본문 ~28~33px). ColumnDef 간격>0 인 헤더 띠(1쪽
        // 등)는 그 간격이 이미 zone 사이 여백이 되므로 제외.
        // [Task #874 Stage 2] design_spacing 조건을 ≤ 1mm(=3.8px) 까지 인정. 페이지 break 후
        // current_zone_design_spacing_px 가 stale state 로 1mm 남은 경우 (shortcut.hwp 6쪽
        // pi=210 '도구' 헤더띠 zone cd 가 pi=209 cd=1mm 인 케이스) 도 헤더띠 leaving 으로 식별.
        let tac_band_extra: f64 = if st.current_zone_design_spacing_px < 4.0 {
            (0..para_idx)
                .rev()
                .find(|&i| !paragraphs[i].line_segs.is_empty())
                .and_then(|pi| {
                    paragraphs[pi].controls.iter().find_map(|c| match c {
                        Control::Table(t)
                            if t.common.treat_as_char
                                && matches!(
                                    t.common.text_wrap,
                                    crate::model::shape::TextWrap::TopAndBottom
                                ) =>
                        {
                            Some(
                                hwpunit_to_px(t.common.height as i32, self.dpi)
                                    + hwpunit_to_px(t.outer_margin_top as i32, self.dpi)
                                    + hwpunit_to_px(t.outer_margin_bottom as i32, self.dpi),
                            )
                        }
                        _ => None,
                    })
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        // [Task #866 v2 Stage 2/4] zone 전환 시 추가 세로 여백.
        // (1) 1단/간격=0 zone(헤더 띠 / `<...>` 소제목) 진입·이탈: +1500 HU(=20px).
        //     shortcut.hwp 4쪽 `개체 모양 복사`↔`<스타일에서>`, 6쪽 `도구`↔`맞춤법 검사` 등.
        // (2) [단나누기](ColumnBreakType::Column) 로 시작하는 새 zone: +1500 HU(=20px).
        //     배분 다단 zone 의 마지막 컬럼 [단나누기] = 같은 ColumnDef 로 새 밴드 → 한컴 PDF
        //     상 이전 밴드와 ~한 본문 줄 간격(shortcut.hwp 3쪽 `화면 확대 100%`↔`<편집 화면
        //     분할에서>`). Stage 1 의 Distribute 마지막 컬럼 라우팅과 정합.
        let entering_solo_zero = paragraphs[para_idx].controls.iter().any(|c| {
            matches!(c,
            Control::ColumnDef(cd) if cd.column_count.max(1) <= 1 && cd.spacing == 0)
        });
        let leaving_solo_zero = st.col_count <= 1 && st.current_zone_design_spacing_px < 0.5;
        // [Task #866 v3 Stage 1] 헤더 띠 zone (TAC wrap=TopAndBottom 표) 의 leaving 은
        // `tac_band_extra` 가 이미 표 band 높이만큼 패딩을 추가하므로 `solo_zone_pad` 를 또
        // 더하면 한컴 PDF 대비 본문 첫 줄이 ~13pt 더 아래로 밀려 사용자 "넓다" 피드백 발생.
        // tac_band_extra>0 == 헤더 띠 leaving 케이스 → solo_zone_pad 의 leaving 분기 제외.
        let leaving_is_header_band = leaving_solo_zero && tac_band_extra > 0.5;
        let column_break_new_band = paragraphs[para_idx].column_type == ColumnBreakType::Column;
        let solo_zone_pad = if entering_solo_zero
            || (leaving_solo_zero && !leaving_is_header_band)
            || column_break_new_band
        {
            hwpunit_to_px(1200, self.dpi)
        } else {
            0.0
        };
        let candidate_offset = st.current_zone_y_offset
            + vpos_zone_height
            + tac_band_extra
            + st.current_zone_design_spacing_px / 2.0
            + new_ds / 2.0
            + solo_zone_pad;
        let paper_overlay_tail_overflows_page = paragraphs[para_idx].column_type
            == ColumnBreakType::Column
            && Self::upcoming_paper_overlay_tail_overflows_page(
                para_idx, paragraphs, &st.layout, self.dpi,
            );

        // [Task #853] 새 zone 이 현재 페이지 하단 가까이(여유 ≲ 헤더 띠 1개 높이)에서 시작하면
        // 그 zone 의 콘텐츠(헤더 띠 ~47px 또는 본문 줄들)가 body 하단을 넘어 렌더되므로 다음
        // 페이지로 넘긴다. (shortcut.hwp 3쪽~6쪽 — 다단 zone 다수 누적 시 잔여 콘텐츠가
        // 본문영역을 넘어 바닥 여백에 그려지던 결함)
        let one_line = hwpunit_to_px(1500, self.dpi);
        if paper_overlay_tail_overflows_page
            || candidate_offset > st.layout.available_body_height() - 4.0 * one_line
        {
            st.push_new_page();
            // 새 페이지 첫 zone: 새 zone 디자인 spacing /2 만 (이전 zone 은 이전 페이지).
            st.current_zone_y_offset = new_ds / 2.0;
        } else {
            st.current_zone_y_offset = candidate_offset;
        }
        st.current_zone_design_spacing_px = new_ds;
        st.current_column = 0;
        st.current_height = 0.0;
        st.on_first_multicolumn_page = true;

        for ctrl in &paragraphs[para_idx].controls {
            if let Control::ColumnDef(cd) = ctrl {
                st.col_count = cd.column_count.max(1);
                let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                st.current_zone_layout = Some(new_layout.clone());
                st.layout = new_layout;
                // [Task #702] 새 zone 의 ColumnType 반영. Distribute(배분) 단에서
                // 짧은 컬럼 vpos-reset 검출 임계값 완화용.
                st.current_zone_column_type = cd.column_type;
                break;
            }
        }
    }

    /// [#2019 v3] 명시 단나누기 뒤에 이어지는 Paper-overlay footer band 가
    /// body frame 하단을 넘고, 곧 명시 쪽나누기가 이어지면 그 band 는 현재 쪽
    /// 꼬리에 억지로 붙이지 않고 다음 물리 쪽에 배치한다.
    ///
    /// 74312 별지 서식의 처리절차 하단 꼬리는 Paper 절대좌표(용지 기준)라
    /// 흐름 높이를 거의 소비하지 않지만, 한컴 PDF 는 body 하단 아래쪽 항목
    /// 일부를 다음 쪽 footer 로 넘긴다. 일반 텍스트/표 band 로 확장하지 않도록
    /// "전부 빈 부동 overlay 앵커 + 직후 Page/Section break" 에만 한정한다.
    fn upcoming_paper_overlay_tail_overflows_page(
        para_idx: usize,
        paragraphs: &[Paragraph],
        layout: &PageLayoutInfo,
        dpi: f64,
    ) -> bool {
        let body_bottom_abs = layout.body_area.y + layout.body_area.height;
        let mut saw_overlay = false;
        let mut overflows = false;
        let mut idx = para_idx + 1;

        while let Some(para) = paragraphs.get(idx) {
            if idx > para_idx + 1
                && (para.column_type != ColumnBreakType::None
                    || para
                        .controls
                        .iter()
                        .any(|ctrl| matches!(ctrl, Control::ColumnDef(_))))
            {
                let follows_explicit_page_break = matches!(
                    para.column_type,
                    ColumnBreakType::Page | ColumnBreakType::Section
                );
                return saw_overlay && overflows && follows_explicit_page_break;
            }

            if para.text.trim().is_empty() && para.controls.is_empty() {
                idx += 1;
                continue;
            }
            if !crate::renderer::layout::para_is_floating_overlay_anchor(para) {
                return false;
            }

            saw_overlay = true;
            if paper_overlay_object_bottom_abs_px(para, dpi)
                .is_some_and(|bottom| bottom > body_bottom_abs + 1.0)
            {
                overflows = true;
            }
            idx += 1;
        }

        false
    }

    /// [Task #846] 마지막 단에서 명시적 단나누기(`ColumnBreakType::Column`, 새 ColumnDef 없음)
    /// 를 만났을 때: 새 페이지가 아니라 같은 col_count 로 같은 페이지에 새 단-밴드를 시작한다
    /// (≈ 닫힌 #768). 단, 새 밴드가 본문에 들어갈 공간(이 문단 첫 줄)이 없으면 새 페이지로 넘긴다.
    /// 규칙: `누적_밴드_높이 + 현_밴드_높이(= max(컬럼별 채움)) < 본문_높이` 이면 새 밴드, 아니면 새 페이지.
    fn start_new_column_band(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
    ) {
        st.flush_column();

        // 새 밴드로 들어갈 콘텐츠에 떠다니는(글자처럼 취급이 아닌) 개체가 있으면
        // 같은 페이지에 밴드를 만들지 않고 새 페이지로 넘긴다.
        if Self::upcoming_band_has_floating_object(para_idx, paragraphs) {
            st.push_new_page();
            return;
        }

        // 방금 닫힌 밴드의 높이 = 그 밴드 각 단의 마지막 문단 vpos_end 중 최댓값.
        let zone_off = st.current_zone_y_offset;
        let mut band_height_px = 0.0_f64;
        if let Some(page) = st.pages.last() {
            for cc in page.column_contents.iter().rev() {
                if cc.zone_y_offset != zone_off {
                    break;
                }
                let last_para_idx = cc.items.iter().rev().find_map(|it| match it {
                    PageItem::FullParagraph { para_index }
                    | PageItem::PartialParagraph { para_index, .. }
                    | PageItem::Table { para_index, .. }
                    | PageItem::PartialTable { para_index, .. }
                    | PageItem::Shape { para_index, .. } => Some(*para_index),
                    PageItem::EndnoteSeparator { .. } => None,
                });
                if let Some(pi) = last_para_idx {
                    if let Some(seg) = paragraphs.get(pi).and_then(|p| p.line_segs.last()) {
                        let v = hwpunit_to_px(
                            seg.vertical_pos
                                .saturating_add(seg.line_height)
                                .saturating_add(seg.line_spacing),
                            self.dpi,
                        );
                        if v > band_height_px {
                            band_height_px = v;
                        }
                    }
                }
            }
        }
        if band_height_px <= 0.0 {
            band_height_px = st.current_height;
        }

        let first_line_h = paragraphs
            .get(para_idx)
            .and_then(|p| p.line_segs.first())
            .map(|s| hwpunit_to_px(s.line_height.saturating_add(s.line_spacing), self.dpi))
            .filter(|h| *h > 0.0)
            .unwrap_or(1.0);
        let room_after_band = st.available_height() - band_height_px;

        if room_after_band >= first_line_h {
            st.current_zone_y_offset += band_height_px;
            st.current_column = 0;
            st.current_height = 0.0;
            st.on_first_multicolumn_page = true;
        } else {
            st.push_new_page();
        }
    }

    /// 명시적 단나누기 다음 밴드(= `para_idx` 부터 다음 나누기/새 ColumnDef 직전까지)에
    /// 떠다니는 개체(글자처럼 취급이 아닌 표/그림/그리기 개체)가 있는지.
    fn upcoming_band_has_floating_object(para_idx: usize, paragraphs: &[Paragraph]) -> bool {
        for (offset, p) in paragraphs[para_idx..].iter().enumerate() {
            if offset > 0
                && (p.column_type != ColumnBreakType::None
                    || p.controls
                        .iter()
                        .any(|c| matches!(c, Control::ColumnDef(_))))
            {
                break;
            }
            for ctrl in &p.controls {
                let floating = match ctrl {
                    Control::Table(t) => !t.common.treat_as_char,
                    Control::Shape(s) => !s.common().treat_as_char,
                    Control::Picture(pic) => !pic.common.treat_as_char,
                    _ => false,
                };
                if floating {
                    return true;
                }
            }
        }
        false
    }

    // ========================================================
    // 머리말/꼬리말/쪽 번호 처리
    // ========================================================

    fn collect_header_footer_controls(
        paragraphs: &[Paragraph],
        section_index: usize,
    ) -> (
        Vec<(usize, HeaderFooterRef, bool, HeaderFooterApply)>,
        Option<crate::model::control::PageNumberPos>,
    ) {
        let mut hf_entries = Vec::new();
        let mut page_number_pos = None;
        for (pi, para) in paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                match control {
                    Control::Header(h) => {
                        let r = HeaderFooterRef {
                            para_index: pi,
                            control_index: ci,
                            source_section_index: section_index,
                            table_path: Vec::new(),
                        };
                        hf_entries.push((pi, r, true, h.apply_to));
                    }
                    Control::Footer(f) => {
                        let r = HeaderFooterRef {
                            para_index: pi,
                            control_index: ci,
                            source_section_index: section_index,
                            table_path: Vec::new(),
                        };
                        hf_entries.push((pi, r, false, f.apply_to));
                    }
                    Control::PageNumberPos(pos) => page_number_pos = Some(pos.clone()),
                    Control::Table(table) => {
                        crate::renderer::pagination::collect_nested_header_footer_controls(
                            table,
                            pi,
                            section_index,
                            ci,
                            &[],
                            &mut hf_entries,
                        );
                    }
                    _ => {}
                }
            }
        }
        (hf_entries, page_number_pos)
    }

    /// 끝 페이지가 가시 내용이나 명시적인 쪽/구역 나누기 없이 빈 문단만 가진 경우
    /// 제거한다.
    ///
    /// HWPX는 편집 중 남은 빈 문단에도 저장 vpos를 보존한다. 마지막 표 조각이 앞
    /// 쪽을 거의 채우면 그 vpos가 빈 문단을 새 물리 쪽으로 보낼 수 있지만, 한컴
    /// 출력은 이를 새 빈 쪽으로 인쇄하지 않는다. 여기서 앞 쪽의 실내용/의도적인
    /// 페이지 나누기와 가시 control은 절대 제거하지 않는다.
    ///
    /// [#5907] 앞뒤 문단이 둘 다 stored vpos 0 을 주장해 열린 쪽도 보존한다 —
    /// 넘침 잔재가 아니라 한/글이 저장한 쪽 경계 그 자체이므로, 한/글도 그 빈 쪽을
    /// 인쇄한다 (`samples/p122.hwp` 3쪽, 정본 `pdf/p122-2022.pdf`).
    fn discard_terminal_blank_only_page(pages: &mut Vec<PageContent>, paragraphs: &[Paragraph]) {
        if pages.len() <= 1 {
            return;
        }
        let Some(last_page) = pages.last() else {
            return;
        };
        let mut has_item = false;
        let blank_only = last_page.column_contents.iter().all(|column| {
            if !column.wrap_around_paras.is_empty() {
                return false;
            }
            column.items.iter().all(|item| {
                let PageItem::FullParagraph { para_index } = item else {
                    return false;
                };
                has_item = true;
                let Some(para) = paragraphs.get(*para_index) else {
                    return false;
                };
                let no_visible_text = para
                    .text
                    .replace(|ch: char| ch.is_control(), "")
                    .trim()
                    .is_empty();
                let opened_by_stored_vpos_reset = *para_index > 0
                    && paragraphs
                        .get(*para_index - 1)
                        .is_some_and(|prev| stored_vpos_top_collision(prev, para));
                no_visible_text
                    && para.controls.is_empty()
                    && !opened_by_stored_vpos_reset
                    && !matches!(
                        para.column_type,
                        ColumnBreakType::Page | ColumnBreakType::Section
                    )
            })
        });
        if has_item && blank_only {
            pages.pop();
        }
    }

    /// 페이지 번호 + 머리말/꼬리말 최종 할당 (기존 Paginator::finalize_pages와 동일)
    fn finalize_pages(
        pages: &mut [PageContent],
        hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
        page_number_pos: &Option<crate::model::control::PageNumberPos>,
        paragraphs: &[Paragraph],
    ) {
        // 쪽번호: PageNumberAssigner 가 NewNumber 1회 적용 + 단조 증가를 보장 (Issue #353)
        // 머리말/꼬리말 선택은 engine.rs 와 같은 규칙을 쓴다 — 종류별로 누적하고 쪽 홀짝에
        // 더 구체적인 것을 고른다. 한 변수에 덮어쓰면 등장 순서가 구체성을 이긴다 (#3234).
        let mut active_hf = crate::renderer::pagination::ActiveHeaderFooter::default();
        let events = crate::renderer::page_number::PageControlEvents::collect(pages, paragraphs);
        let mut assigner =
            crate::renderer::page_number::PageNumberAssigner::new_for_pages(&events.new_numbers, 1);

        for (i, page) in pages.iter_mut().enumerate() {
            let page_num = assigner.assign(page);

            // 이 페이지에 속하는 머리말/꼬리말 갱신
            let page_last_para = page
                .column_contents
                .iter()
                .flat_map(|col| col.items.iter())
                .filter_map(|item| match item {
                    PageItem::FullParagraph { para_index } => Some(*para_index),
                    PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                    PageItem::Table { para_index, .. } => Some(*para_index),
                    PageItem::PartialTable { para_index, .. } => Some(*para_index),
                    PageItem::Shape { para_index, .. } => Some(*para_index),
                    PageItem::EndnoteSeparator { .. } => None,
                })
                .max();

            if let Some(last_pi) = page_last_para {
                active_hf.accumulate(hf_entries, last_pi);
            }

            page.page_number = page_num;
            page.page_number_restarted = assigner.last_restarted();
            let (current_header, current_footer) = active_hf.active(page_num);
            page.active_header = current_header;
            page.active_footer = current_footer;
            if !assigner.should_hide_page_number() {
                page.page_number_pos = page_number_pos.clone();
            }

            // 한 컨트롤의 감추기는 소스 위치가 매핑된 한 쪽에만 적용한다.
            if let Some((_, hide)) = events.hides.iter().find(|(target, _)| *target == i) {
                page.page_hide = Some(hide.clone());
            }
        }
    }

    // ========================================================
    // 유틸리티
    // ========================================================

    /// 문단에 블록 표 컨트롤이 있는지 감지
    fn paragraph_has_table(&self, para: &Paragraph) -> bool {
        use crate::renderer::height_measurer::is_tac_table_inline_in_para;
        let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
        para.controls.iter().any(|c| {
            matches!(c, Control::Table(t) if t.attr & 0x01 == 0
                || (t.attr & 0x01 != 0 && !is_tac_table_inline_in_para(t, seg_width, para)))
        })
    }

    /// Text-flow predicate only. A matching row can still paint a declared cell
    /// height, border, fill, or table-level fallback, so it must not by itself
    /// authorize a fragment-height overflow.
    fn row_has_no_text_or_controls(table: &crate::model::table::Table, row: usize) -> bool {
        let mut row_cells = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .peekable();
        if row_cells.peek().is_none() {
            return false;
        }
        row_cells.all(|cell| {
            cell.paragraphs.iter().all(|para| {
                let trimmed = para.text.replace(|c: char| c.is_control(), "");
                trimmed.trim().is_empty() && para.controls.is_empty()
            })
        })
    }

    /// 표의 세로 오프셋 추출 (Paginator와 동일).
    ///
    /// `raw_ctrl_data` 의 첫 4바이트는 `attr` 비트 플래그이고 `vertical_offset` 은
    /// 다음 4바이트 (`raw_ctrl_data[4..8]`) 이지만, IR 의 `common.vertical_offset` 가
    /// 파서가 채운 권위 있는 값이므로 이를 직접 사용한다 (#178).
    fn get_table_vertical_offset(table: &crate::model::table::Table) -> u32 {
        table.common.vertical_offset as u32
    }
}

/// Task #321: 단일 문단의 컨트롤에서 body-wide TopAndBottom 표/도형이 차지하는 높이 계산.
///
/// col 1+ advance 시 current_height 시작값으로 사용하여 layout의 `body_wide_reserved`
/// 와 동일한 가용 공간 축소를 적용한다.
///
/// **Paper(용지) 기준 도형 가드 (v3 정밀화 #326)**: vert_rel_to=Paper 인 도형 중
/// 본문 영역과 겹치지 않는(머리말 영역에만 위치하는) 도형만 제외. body 와 겹치는
/// Paper 도형은 col 1 시작에 영향 → reserve 대상으로 포함.
fn compute_body_wide_top_reserve_for_para(
    para: &Paragraph,
    layout: &PageLayoutInfo,
    dpi: f64,
) -> f64 {
    use crate::model::shape::{TextWrap, VertRelTo};
    let body_w = layout.body_area.width;
    let body_h = layout.available_body_height();
    let body_top = layout.body_area.y;
    let mut max_bottom: f64 = 0.0;
    for ctrl in &para.controls {
        let common = match ctrl {
            Control::Shape(s) => s.common(),
            Control::Table(t) if !t.common.treat_as_char => &t.common,
            Control::Picture(p) if !p.common.treat_as_char => &p.common,
            _ => continue,
        };
        if !matches!(common.text_wrap, TextWrap::TopAndBottom) || common.treat_as_char {
            continue;
        }
        let shape_w = crate::renderer::hwpunit_to_px(common.width as i32, dpi);
        if shape_w < body_w * 0.8 {
            continue;
        }
        let shape_h = crate::renderer::hwpunit_to_px(common.height as i32, dpi);
        let raw_v_offset = crate::renderer::hwpunit_to_px(common.vertical_offset as i32, dpi);

        // body-rel 기준 시작/끝 y 계산.
        // - VertRelTo::Paper: vertical_offset 이 용지 상단(= 0) 기준 → body_top 차감.
        //   본문과 전혀 겹치지 않으면(머리말만 점유) 제외.
        //   본문 위쪽으로 일부 빠져나가면(shape_top_abs < body_top) 본문 침범 영역만 reserve.
        // - VertRelTo::Page / Para: vertical_offset 이 본문/단 top 기준 → body-rel 그대로.
        let (body_y, body_bottom) = if matches!(common.vert_rel_to, VertRelTo::Paper) {
            let shape_top_abs = raw_v_offset;
            let shape_bottom_abs = shape_top_abs + shape_h;
            if shape_bottom_abs <= body_top {
                continue;
            }
            (
                (shape_top_abs - body_top).max(0.0),
                shape_bottom_abs - body_top,
            )
        } else {
            (raw_v_offset, raw_v_offset + shape_h)
        };

        if body_y > body_h / 3.0 {
            continue;
        }
        let outer_bottom = crate::renderer::hwpunit_to_px(common.margin.bottom as i32, dpi);
        let bottom = body_bottom + outer_bottom;
        if bottom > max_bottom {
            max_bottom = bottom;
        }
    }
    max_bottom
}

#[cfg(test)]
mod issue_3820_saved_rowbreak_first_fragment_frame_contract {
    use super::{
        nearest_saved_rowbreak_frame_row_end, saved_rowbreak_first_fragment_flow_overflow_allowance,
    };

    #[test]
    fn rejects_missing_or_unowned_saved_frames() {
        assert_eq!(
            saved_rowbreak_first_fragment_flow_overflow_allowance(0, true, 959.0, 970.0),
            0.0,
            "height=0 is not a saved first-fragment frame"
        );
        assert_eq!(
            saved_rowbreak_first_fragment_flow_overflow_allowance(0x8000_0000, true, 959.0, 970.0,),
            0.0,
            "signed-wrap height is not a usable source frame"
        );
        assert_eq!(
            saved_rowbreak_first_fragment_flow_overflow_allowance(929, false, 959.0, 970.0,),
            0.0,
            "an outer wrapper frame cannot authorize an inner table row cut"
        );
    }

    #[test]
    fn uses_the_absolute_fragment_bottom_not_row_space() {
        assert_eq!(
            saved_rowbreak_first_fragment_flow_overflow_allowance(929, true, 972.0, 970.0,),
            0.0,
            "the source frame may not enter the fragment-reserved bottom lane"
        );
        assert!(
            (saved_rowbreak_first_fragment_flow_overflow_allowance(929, true, 959.0, 971.0,)
                - 12.0)
                .abs()
                < f64::EPSILON,
            "host-before flow spacing does not shrink the saved object's physical slack"
        );
    }

    #[test]
    fn frame_slack_stops_at_its_nearest_row_boundary() {
        assert_eq!(
            nearest_saved_rowbreak_frame_row_end(
                87.0,
                &[30.0, 55.0, 60.0],
                &[30.0, 56.0, 60.0],
                0.0
            ),
            Some(2),
            "saved-frame slack may absorb measurement drift at row 2, but not admit row 3"
        );

        // [#6123] 프레임이 행 경계에 **닿지 못하면** 그 행을 소유하지 않는다 —
        // 행 경계 신호가 아니라 행 안에서 끊으라는 신호다(3112461 7쪽: 프레임 388 이
        // 행 1(36~573)의 65% 지점인데 573 으로 스냅돼 행이 통째로 앞 쪽에 얹혔다).
        assert_eq!(
            nearest_saved_rowbreak_frame_row_end(388.0, &[36.0, 537.0], &[36.0, 472.0], 0.0),
            None,
            "모자란 몫(185.1)이 흡수 가능한 drift(65.0)를 넘으면 행 경계가 아니다"
        );

        // 경계를 **넘어서는** 프레임은 종전대로 그 행 끝을 소유한다 — 초과분은
        // 다음 행의 측정↔저장 drift 다(21298295 별표 5: 행 13 경계를 22.6px 초과).
        assert_eq!(
            nearest_saved_rowbreak_frame_row_end(112.6, &[30.0, 60.0], &[30.0, 60.0], 0.0),
            Some(2),
            "프레임이 경계를 지나면 그 행까지는 확실히 첫 조각 소유다"
        );

        // 조각이 될 수 없는 크기(25px)만큼 모자란 프레임은 그 행을 그대로 소유한다 —
        // 그 잔여는 어차피 다음 쪽으로 옮길 수 없다(1790387 PrEP 보고서: 16.8px).
        assert_eq!(
            nearest_saved_rowbreak_frame_row_end(
                405.8,
                &[28.4, 103.5, 163.5, 127.3],
                &[28.4, 103.5, 163.5, 127.3],
                0.0
            ),
            Some(4),
            "16.8px 잔여는 독립 조각이 될 수 없으므로 행을 통째로 둔다"
        );

        // 저장 행 높이를 못 읽는 표(전부 0)는 종전 최근접 스냅을 유지한다.
        assert_eq!(
            nearest_saved_rowbreak_frame_row_end(87.0, &[30.0, 55.0, 60.0], &[0.0, 0.0, 0.0], 0.0),
            Some(2),
            "저장 높이가 없으면 측정 최근접 스냅이 유일한 신호다"
        );
    }
}

#[cfg(test)]
mod issue_3780_line_advance_oob {
    use super::FormattedParagraph;

    fn fp(lines: usize) -> FormattedParagraph {
        FormattedParagraph {
            tail_line_remaining_width: None,
            computed_host_lines: None,
            total_height: 0.0,
            line_heights: vec![10.0; lines],
            line_spacings: vec![2.0; lines],
            spacing_before: 0.0,
            spacing_after: 0.0,
            height_for_fit: 0.0,
            tac_outer_margin_v_px: 0.0,
        }
    }

    /// red→green: 클램프를 원복하면 이 두 테스트는 index out of bounds 로 패닉한다
    /// (len 31 / index 31 — 실측 사고와 동일한 상태).
    #[test]
    fn out_of_range_line_advance_is_zero_not_panic() {
        let p = fp(31);
        assert_eq!(p.line_advance(31), 0.0);
        assert_eq!(p.line_advance(30), 12.0, "경계 안 동작 불변");
    }

    #[test]
    fn out_of_range_advance_sum_clamps_to_existing_lines() {
        let p = fp(31);
        assert_eq!(p.line_advances_sum(29..33), 24.0, "29,30 두 줄만 합산");
        assert_eq!(p.line_advances_sum(0..31), 31.0 * 12.0, "전 범위 동작 불변");
        assert_eq!(p.line_advances_sum(40..50), 0.0);
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::float_placement::native_empty_host_rowbreak_line_advance_hu;

    #[test]
    fn fullpage_image_single_page_policy_requires_strict_majority() {
        assert!(!super::has_majority_fullpage_images(0, 0));
        assert!(!super::has_majority_fullpage_images(1, 1));
        assert!(!super::has_majority_fullpage_images(2, 4));
        assert!(super::has_majority_fullpage_images(2, 3));
        assert!(super::has_majority_fullpage_images(3, 5));
    }

    use super::*;
    use crate::model::page::{ColumnDef, PageDef};
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::shape::{CommonObjAttr, TextWrap, VertRelTo};
    use crate::model::table::{Cell, Table, TablePageBreak};
    use crate::model::Padding;
    use crate::renderer::composer::ComposedParagraph;
    use crate::renderer::height_measurer::HeightMeasurer;
    use crate::renderer::layout::LayoutEngine;
    use crate::renderer::page_layout::PageLayoutInfo;
    use crate::renderer::pagination::Paginator;
    use crate::renderer::style_resolver::ResolvedStyleSet;

    fn a4_page_def() -> PageDef {
        PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            margin_gutter: 0,
            ..Default::default()
        }
    }

    #[test]
    fn single_column_vpos_reset_gate_requires_exclusive_tac_picture_or_shape() {
        let tac_picture = || {
            let mut picture = crate::model::image::Picture::default();
            picture.common.treat_as_char = true;
            Control::Picture(Box::new(picture))
        };

        let picture_only = Paragraph {
            controls: vec![tac_picture()],
            ..Default::default()
        };
        assert!(para_has_only_treat_as_char_picture_or_shape(&picture_only));

        let mut table = Table::default();
        table.common.treat_as_char = true;
        let mixed_controls = Paragraph {
            controls: vec![tac_picture(), Control::Table(Box::new(table))],
            ..Default::default()
        };
        assert!(
            !para_has_only_treat_as_char_picture_or_shape(&mixed_controls),
            "TAC 그림과 표가 섞인 문단은 단일 단 vpos-reset 강제 분리 대상이 아니다"
        );

        let picture_with_text = Paragraph {
            text: "설명".to_string(),
            controls: vec![tac_picture()],
            ..Default::default()
        };
        assert!(!para_has_only_treat_as_char_picture_or_shape(
            &picture_with_text
        ));
    }

    /// [#4333] 인라인(글자처럼) 도형의 흐름 높이는 조판과 렌더가 같은 정의를 써야 한다.
    ///
    /// 조판은 저장 프레임(`common.height`)만, 렌더는 프레임과 개체 표시 높이
    /// (`shape_attr.current_height`) 중 큰 값을 썼다. samples/ 의 인라인 도형 868개
    /// 가운데 354개(38개 문서)가 둘이 다르므로, 두 정의가 갈리면 그 줄의 예약 높이가
    /// 조판과 렌더에서 달라진다. 두 경로가 `ShapeObject::flow_height_hu` 하나를 보는지
    /// 확인한다 — 매직 넘버가 아니라 **두 정의가 같다**를 단언한다.
    #[test]
    fn typeset_and_render_agree_on_inline_shape_flow_height() {
        use crate::model::shape::{RectangleShape, ShapeObject};

        let dpi = 96.0;
        let shape = ShapeObject::Rectangle(RectangleShape {
            common: CommonObjAttr {
                width: 20000,
                height: 2400,
                treat_as_char: true,
                ..Default::default()
            },
            ..Default::default()
        });
        let mut shape = shape;
        shape.shape_attr_mut().current_height = 4066;
        let expected = crate::renderer::hwpunit_to_px(shape.flow_height_hu(), dpi);
        assert!(
            expected > crate::renderer::hwpunit_to_px(shape.common().height as i32, dpi),
            "표본 전제: 개체 표시 높이가 저장 프레임보다 크다"
        );

        let para = Paragraph {
            text: "\u{FFFC}".to_string(),
            char_count: 1,
            controls: vec![Control::Shape(Box::new(shape))],
            ..Default::default()
        };
        let comp = ComposedParagraph {
            lines: vec![crate::renderer::composer::ComposedLine {
                runs: Vec::new(),
                line_height: 4066,
                baseline_distance: 0,
                segment_width: 20000,
                column_start: 0,
                line_spacing: 0,
                has_line_break: false,
                char_start: 0,
            }],
            para_style_id: 0,
            inline_controls: Vec::new(),
            numbering_text: None,
            tac_controls: vec![(0, 20000, 0)],
            footnote_positions: Vec::new(),
            tab_extended: Vec::new(),
            horizontal_shaping: None,
        };

        // 조판(페이지네이션)이 이 줄에 예약하는 인라인 개체 높이.
        let typeset_reserved = line_tac_picture_or_shape_height(&para, &comp, 0, dpi)
            .expect("조판이 인라인 도형 줄을 인식해야 한다");
        // 렌더가 인라인 도형을 배치할 때 쓰는 높이(같은 단일 정의).
        let render_reserved = crate::renderer::tac_object_flow_height_px(&para.controls[0], dpi)
            .expect("렌더가 인라인 도형 높이를 산출해야 한다");

        assert_eq!(typeset_reserved, render_reserved);
        assert_eq!(typeset_reserved, expected);
    }

    fn make_paragraph_with_height(line_height: i32) -> Paragraph {
        Paragraph {
            line_segs: vec![LineSeg {
                line_height,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn composed_footnote_height_separates_body_exact_metric_from_table_queue_floor() {
        let dpi = 96.0;
        let empty_footnote = Footnote::default();
        assert_eq!(composed_footnote_content_height(&empty_footnote, dpi), 0.0);
        assert_eq!(
            queued_table_footnote_content_height(&empty_footnote, dpi),
            hwpunit_to_px(400, dpi),
        );

        let short_line = Footnote {
            paragraphs: vec![Paragraph {
                text: "짧음".to_string(),
                char_count: 3,
                line_segs: vec![LineSeg {
                    line_height: 200,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(
            composed_footnote_content_height(&short_line, dpi),
            hwpunit_to_px(200, dpi),
        );
        assert_eq!(
            queued_table_footnote_content_height(&short_line, dpi),
            hwpunit_to_px(400, dpi),
        );

        let empty_paragraph = Footnote {
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        };
        assert_eq!(
            composed_footnote_content_height(&empty_paragraph, dpi),
            hwpunit_to_px(400, dpi),
        );
    }

    fn single_column_split_topology() -> Table {
        let mut heading = Cell::new_empty(0, 0, 20_000, 1_000, 1);
        heading.col_span = 2;
        heading.paragraphs[0].text = "머리".to_string();
        heading.paragraphs[0].char_count = 3;

        let mut body = Cell::new_empty(0, 1, 10_000, 2_000, 2);
        body.paragraphs[0].text = "긴 본문".to_string();
        body.paragraphs[0].char_count = 5;
        body.paragraphs[0].raw_header_extra = vec![0; 12];
        body.paragraphs[0].raw_header_extra[6..10].copy_from_slice(&42_u32.to_le_bytes());
        let peer = Cell::new_from_template(1, 1, 10_000, 2_000, &body);

        Table {
            row_count: 2,
            col_count: 2,
            cells: vec![heading, body, peer],
            page_break: TablePageBreak::RowBreak,
            ..Default::default()
        }
    }

    #[test]
    fn visible_host_caption_classifier_excludes_section_headings() {
        use crate::model::control::{AutoNumber, AutoNumberType, NewNumber};

        let mut para = Paragraph {
            text: "표 27. EU 미성년자 생존 장기기증 허용 국가 규정".to_string(),
            ..Default::default()
        };
        assert!(visible_host_is_numbered_table_caption(&para));

        para.text = "Table 27. Minor living donation rules".to_string();
        assert!(visible_host_is_numbered_table_caption(&para));

        para.text = "1. 편성기준".to_string();
        assert!(
            !visible_host_is_numbered_table_caption(&para),
            "#2097 section heading must not be pre-emitted as a table caption"
        );

        para.text = "표 작성 기준".to_string();
        assert!(
            !visible_host_is_numbered_table_caption(&para),
            "a word beginning with 표 is not a numbered caption"
        );

        para.text = "표  ".to_string();
        para.controls = vec![Control::AutoNumber(AutoNumber {
            number_type: AutoNumberType::Table,
            ..Default::default()
        })];
        assert!(
            visible_host_is_numbered_table_caption(&para),
            "literal이 아닌 표 AutoNumber도 visible-host 캡션이다"
        );

        para.text = "번호를 다시 시작하는 표".to_string();
        para.controls = vec![Control::NewNumber(NewNumber {
            number_type: AutoNumberType::Table,
            number: 27,
        })];
        assert!(visible_host_is_numbered_table_caption(&para));

        para.text = "표 작성 기준".to_string();
        para.controls = vec![Control::AutoNumber(AutoNumber {
            number_type: AutoNumberType::Picture,
            ..Default::default()
        })];
        assert!(!visible_host_is_numbered_table_caption(&para));

        para.text = "1. 편성기준".to_string();
        para.controls = vec![Control::NewNumber(NewNumber {
            number_type: AutoNumberType::Page,
            number: 1,
        })];
        assert!(!visible_host_is_numbered_table_caption(&para));
    }

    #[test]
    fn reparsed_single_column_split_topology_is_narrow() {
        let split = single_column_split_topology();
        assert!(
            is_reparsed_single_column_cell_split_row(&split, 1),
            "1열 표의 본문 셀을 1×2로 분할한 저장 구조는 식별한다"
        );
        assert!(
            !is_reparsed_single_column_cell_split_row(&split, 0),
            "전폭 행 자체는 분할 행이 아니다"
        );

        let mut original = split.clone();
        original.col_count = 1;
        original.cells.truncate(2);
        original.cells[0].col_span = 1;
        original.cells[1].width = 20_000;
        assert!(
            !is_reparsed_single_column_cell_split_row(&original, 1),
            "원본 1열 giant-cell에는 strict cut을 적용하지 않는다"
        );

        let mut ordinary_two_column = split.clone();
        ordinary_two_column.cells[2].paragraphs[0].text = "일반 우측 본문".to_string();
        ordinary_two_column.cells[2].paragraphs[0].char_count = 8;
        assert!(
            !is_reparsed_single_column_cell_split_row(&ordinary_two_column, 1),
            "병합 제목행과 일반 2열 데이터행 조합은 #2097 bottom squeeze를 보존한다"
        );

        let mut authored_blank = split.clone();
        authored_blank.cells[2].paragraphs[0].has_para_text = true;
        assert!(
            !is_reparsed_single_column_cell_split_row(&authored_blank, 1),
            "빈 문자열이어도 PARA_TEXT를 가진 자연 작성 셀은 분할 템플릿 빈 셀이 아니다"
        );

        let mut natural_blank = split.clone();
        natural_blank.cells[2].paragraphs[0].raw_header_extra[6..10]
            .copy_from_slice(&7_u32.to_le_bytes());
        assert!(
            !is_reparsed_single_column_cell_split_row(&natural_blank, 1),
            "독립 instanceId를 가진 자연 작성 빈 셀은 zeroed template clone이 아니다"
        );

        let mut blank_left = split.clone();
        blank_left.cells[1].col = 1;
        blank_left.cells[2].col = 0;
        assert!(
            !is_reparsed_single_column_cell_split_row(&blank_left, 1),
            "빈 왼쪽/본문 오른쪽의 자연 2열 행은 split_cell_into(1×2) 형상이 아니다"
        );

        let mut mixed_grid = split;
        mixed_grid
            .cells
            .push(Cell::new_empty(1, 0, 10_000, 1_000, 1));
        mixed_grid.cells[0].col_span = 1;
        assert!(
            !is_reparsed_single_column_cell_split_row(&mixed_grid, 1),
            "다른 행이 원래부터 다열이면 1열→2열 분할 저장 구조가 아니다"
        );
    }

    #[test]
    fn saved_tail_fit_chain_stops_on_third_applicable_line() {
        assert_eq!(
            saved_tail_fit_chain_decision(0, true, false, false),
            SavedTailFitChainDecision::Advance,
            "첫 저장 꼬리줄은 허용한다"
        );
        assert_eq!(
            saved_tail_fit_chain_decision(1, true, false, false),
            SavedTailFitChainDecision::Advance,
            "둘째 저장 꼬리줄은 허용한다"
        );
        assert_eq!(
            saved_tail_fit_chain_decision(2, true, false, false),
            SavedTailFitChainDecision::Break,
            "셋째 저장 꼬리줄에서 예산 초과 연쇄를 끊는다"
        );
    }

    #[test]
    fn saved_tail_fit_chain_keeps_authoritative_boundary_exceptions() {
        assert_eq!(
            saved_tail_fit_chain_decision(2, true, true, false),
            SavedTailFitChainDecision::NoChange,
            "HWP 권위 경계는 저장 꼬리줄 상한으로 중단하지 않는다"
        );
        assert_eq!(
            saved_tail_fit_chain_decision(2, true, false, true),
            SavedTailFitChainDecision::NoChange,
            "native HWP5 실제 각주 경계 예외는 저장 꼬리줄 상한으로 중단하지 않는다"
        );
        assert_eq!(
            saved_tail_fit_chain_decision(2, false, false, false),
            SavedTailFitChainDecision::NoChange,
            "저장 꼬리줄 판정이 아니면 연쇄 상태를 바꾸지 않는다"
        );
    }

    #[test]
    fn saved_line_clears_footnote_area_requires_every_boundary() {
        let allows = |footnote_height, is_single_column, overflow, bounds, current_height| {
            saved_line_clears_footnote_area(
                footnote_height,
                is_single_column,
                overflow,
                40.0,
                bounds,
                1_000.0,
                current_height,
            )
        };

        assert!(
            allows(300.0, true, 40.0, Some((200.0, 700.0)), 216.0),
            "각주 위 저장 좌표, 안전마진 이하 초과, 16px 흐름 허용오차의 경계값은 허용한다"
        );
        assert!(
            !allows(0.0, true, 40.0, Some((200.0, 700.0)), 216.0),
            "각주가 없으면 예외를 적용하지 않는다"
        );
        assert!(
            !allows(300.0, false, 40.0, Some((200.0, 700.0)), 216.0),
            "다단에는 적용하지 않는다"
        );
        assert!(
            !allows(300.0, true, 40.1, Some((200.0, 700.0)), 216.0),
            "안전마진보다 큰 초과는 허용하지 않는다"
        );
        assert!(
            !allows(300.0, true, 40.0, Some((200.0, 700.1)), 216.0),
            "저장 줄 하단이 각주 영역에 닿으면 허용하지 않는다"
        );
        assert!(
            !allows(300.0, true, 40.0, Some((-0.1, 700.0)), 0.0),
            "쪽 밖 음수 저장 좌표는 허용하지 않는다"
        );
        assert!(
            !allows(300.0, true, 40.0, Some((1_000.1, 700.0)), 0.0),
            "본문 높이를 넘는 저장 좌표는 허용하지 않는다"
        );
        assert!(
            !allows(300.0, true, 40.0, Some((200.0, 700.0)), 216.1),
            "흐름 커서가 저장 좌표보다 낮으면 허용하지 않는다"
        );
    }

    /// [#3925] HWPX empty-host 그림 표의 raw anchor는 다음 저장 사다리가 개체 높이를
    /// 실제로 비운 경우에만 쓸 수 있다. 작은 일반 줄의 vpos를 그대로 쓰면
    /// 36324768처럼 표 높이를 이중 소비해 뒤 본문을 다음 쪽으로 민다.
    #[test]
    fn stored_host_anchor_requires_next_ladder_room() {
        let table = Table {
            common: CommonObjAttr {
                height: 18_000,
                ..Default::default()
            },
            outer_margin_top: 500,
            outer_margin_bottom: 300,
            ..Default::default()
        };
        let host = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 1_000,
                line_height: 1_100,
                ..Default::default()
            }],
            ..Default::default()
        };
        let short_ladder = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 2_500,
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(
            !stored_ladder_leaves_object_room(&host, Some(&short_ladder), &table),
            "짧은 다음-vpos 간격은 raw anchor의 저장 증거가 아니다"
        );

        let full_ladder = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 19_800,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            stored_ladder_leaves_object_room(&host, Some(&full_ladder), &table),
            "다음 저장 vpos가 표 높이와 바깥 여백을 모두 비우면 사다리 증거다"
        );

        let covered_host = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 1_000,
                line_height: 18_800,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(
            !stored_ladder_leaves_object_room(&covered_host, None, &table),
            "host 줄 높이는 #3738 표적을 되살릴 수 있어 다음-vpos 사다리 없이 raw anchor의 증거가 아니다"
        );
    }

    /// [#3900] 합성 LineSeg는 serializer가 만든 placeholder라 물리 페이지 경계의
    /// 저장 증거가 아니다. 앞·뒤 양쪽 모두 실제 저장 줄만 보아야 한다.
    #[test]
    fn stored_vpos_rewind_ignores_synthetic_line_segments() {
        let previous = Paragraph {
            line_segs: vec![
                LineSeg {
                    vertical_pos: 41_645,
                    ..Default::default()
                },
                LineSeg {
                    vertical_pos: 0,
                    tag: LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let current = Paragraph {
            line_segs: vec![
                LineSeg {
                    vertical_pos: 0,
                    tag: LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                    ..Default::default()
                },
                LineSeg {
                    vertical_pos: 1_000,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let paragraphs = vec![previous, current.clone()];

        let previous_vpos = preceding_stored_vpos(&paragraphs, 1);
        assert_eq!(previous_vpos, Some(41_645));
        assert!(stored_vpos_rewinds(previous_vpos, &current));
    }

    fn hwpx_tail_page_break_candidate(break_type: ColumnBreakType) -> (Paragraph, Paragraph) {
        let paragraph = Paragraph {
            text: "앞 줄\n마지막 줄".to_string(),
            line_segs: vec![
                LineSeg {
                    vertical_pos: 6_000,
                    line_height: 1_000,
                    ..Default::default()
                },
                LineSeg {
                    vertical_pos: 0,
                    line_height: 1_000,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let next = Paragraph {
            column_type: break_type,
            ..Default::default()
        };
        (paragraph, next)
    }

    #[test]
    fn hwpx_explicit_page_break_tail_splits_only_last_stored_line() {
        for break_type in [ColumnBreakType::Page, ColumnBreakType::Section] {
            let (paragraph, next) = hwpx_tail_page_break_candidate(break_type);
            assert_eq!(
                hwpx_explicit_page_break_tail_line(&paragraph, Some(&next), 2, 100.0, DEFAULT_DPI,),
                Some(1),
                "{break_type:?} 명시 나눔 앞의 vpos=0 tail만 분리해야 한다"
            );
        }
    }

    #[test]
    fn hwpx_explicit_page_break_tail_requires_all_stored_layout_evidence() {
        let rejects = |paragraph: Paragraph, next: Paragraph, line_count| {
            assert_eq!(
                hwpx_explicit_page_break_tail_line(
                    &paragraph,
                    Some(&next),
                    line_count,
                    100.0,
                    DEFAULT_DPI,
                ),
                None,
                "불완전한 저장 증거는 일반 문단 쪽 경계로 승격하면 안 된다"
            );
        };

        let (paragraph, _) = hwpx_tail_page_break_candidate(ColumnBreakType::Page);
        rejects(paragraph, Paragraph::default(), 2);

        let (paragraph, next) = hwpx_tail_page_break_candidate(ColumnBreakType::Column);
        rejects(paragraph, next, 2);

        let (mut paragraph, next) = hwpx_tail_page_break_candidate(ColumnBreakType::Page);
        paragraph.line_segs[1].vertical_pos = 1;
        rejects(paragraph, next, 2);

        let (mut paragraph, next) = hwpx_tail_page_break_candidate(ColumnBreakType::Page);
        paragraph.line_segs.push(LineSeg::default());
        rejects(paragraph, next, 2);

        let (mut paragraph, next) = hwpx_tail_page_break_candidate(ColumnBreakType::Page);
        paragraph.line_segs[0].vertical_pos = 3_000;
        rejects(paragraph, next, 2);

        let (mut paragraph, next) = hwpx_tail_page_break_candidate(ColumnBreakType::Page);
        paragraph.line_segs[0].vertical_pos = 7_000;
        rejects(paragraph, next, 2);
    }

    #[test]
    fn issue2439_native_empty_host_rowbreak_evidence_is_narrow() {
        let table = Table {
            page_break: TablePageBreak::RowBreak,
            common: CommonObjAttr {
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vertical_offset: 350,
                ..Default::default()
            },
            ..Default::default()
        };
        let anchor = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 1200,
                line_spacing: 240,
                ..Default::default()
            }],
            controls: vec![Control::Table(Box::new(table.clone()))],
            ..Default::default()
        };
        // [#2808] 접힌 ladder 증거: next.vpos - anchor.vpos == anchor 줄 advance 일 때만
        // 한컴이 host 줄을 실 흐름에 계상한 것으로 본다 (#2439 재현 문서 서명).
        let signature = Paragraph {
            text: "signature".to_string(),
            line_segs: vec![LineSeg {
                line_height: 1000,
                vertical_pos: 1440,
                ..Default::default()
            }],
            ..Default::default()
        };
        let line_advance =
            native_empty_host_rowbreak_line_advance_hu(true, &anchor, &table, Some(&signature))
                .expect("native evidence line advance");
        assert_eq!(line_advance, 1440);
        let tail = hwpunit_to_px(line_advance + 350, DEFAULT_DPI);
        assert!(
            (tail - hwpunit_to_px(1790, DEFAULT_DPI)).abs() < 0.01,
            "tail must be stored line advance + positive vertical_offset"
        );

        assert!(native_empty_host_rowbreak_line_advance_hu(
            false,
            &anchor,
            &table,
            Some(&signature),
        )
        .is_none());
        assert!(native_empty_host_rowbreak_line_advance_hu(
            true,
            &anchor,
            &table,
            Some(&Paragraph::new_empty()),
        )
        .is_none());

        let mut two_tables = anchor.clone();
        two_tables
            .controls
            .push(Control::Table(Box::new(table.clone())));
        assert!(native_empty_host_rowbreak_line_advance_hu(
            true,
            &two_tables,
            &table,
            Some(&signature),
        )
        .is_none());

        // [#2808] 물리 ladder(다음 문단 vpos 가 표 높이를 이미 포함) 문서는 tail 을
        // 더하면 이중 계상 — 증거 불일치로 억제되어야 한다 (10k r19 회귀 4건).
        let mut physical_ladder_signature = signature.clone();
        physical_ladder_signature.line_segs[0].vertical_pos = 11202;
        assert!(native_empty_host_rowbreak_line_advance_hu(
            true,
            &anchor,
            &table,
            Some(&physical_ladder_signature),
        )
        .is_none());
    }

    fn issue2814_half_page_picture(height_hu: u32) -> Control {
        use crate::model::shape::{TextWrap, VertRelTo};
        let mut pic = crate::model::image::Picture::default();
        pic.common.treat_as_char = false;
        pic.common.text_wrap = TextWrap::TopAndBottom;
        pic.common.vert_rel_to = VertRelTo::Para;
        pic.common.width = 40000;
        pic.common.height = height_hu;
        Control::Picture(Box::new(pic))
    }

    fn issue2814_typeset_pages(host: Paragraph) -> Vec<usize> {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let paras = vec![host];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let (_, measured) = paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );
        result
            .pages
            .iter()
            .map(|page| {
                page.column_contents
                    .iter()
                    .flat_map(|col| col.items.iter())
                    .filter(|item| matches!(item, PageItem::Shape { .. }))
                    .count()
            })
            .collect()
    }

    /// [#2814] 한 문단에 co-anchored 절반쪽 그림이 여럿(≥3)이면 쪽 용량 기반으로
    /// 분배한다 — 한컴은 흐름처럼 쪽을 채우며 넘긴다(창조경제 보고서: 37장 = 2장/쪽).
    #[test]
    fn issue2814_multi_coanchored_half_page_pictures_distribute_across_pages() {
        let mut host = make_paragraph_with_height(900);
        host.controls = (0..6).map(|_| issue2814_half_page_picture(30000)).collect();
        let shapes_per_page = issue2814_typeset_pages(host);
        assert_eq!(
            shapes_per_page,
            vec![2, 2, 2],
            "절반쪽 그림 6장은 쪽당 2장씩 3쪽으로 분배되어야 한다",
        );
    }

    /// [#2814] 2장 스택은 발동하지 않는다 — 한컴이 razor-full 쪽에 압축 유지하는
    /// 실측 반례(1051000-201800093 p60) 보호. 초과분은 기존 overflow 관용에 맡긴다.
    #[test]
    fn issue2814_two_picture_stack_keeps_current_page() {
        let mut host = make_paragraph_with_height(900);
        host.controls = (0..2).map(|_| issue2814_half_page_picture(35000)).collect();
        let shapes_per_page = issue2814_typeset_pages(host);
        assert_eq!(
            shapes_per_page,
            vec![2],
            "2장 스택은 쪽 이월 없이 현재 쪽을 유지해야 한다",
        );
    }

    #[test]
    fn issue2439_strict_following_plain_text_fit_is_consumed_once() {
        let plain = Paragraph {
            text: "signature".to_string(),
            ..Default::default()
        };
        let mut pending = true;
        assert!(take_strict_plain_text_fit_after_empty_host_float_once(
            &mut pending,
            &plain
        ));
        assert!(!pending);
        assert!(!take_strict_plain_text_fit_after_empty_host_float_once(
            &mut pending,
            &plain
        ));

        let mut pending = true;
        assert!(!take_strict_plain_text_fit_after_empty_host_float_once(
            &mut pending,
            &Paragraph::new_empty(),
        ));
        assert!(
            pending,
            "an ineligible paragraph must not consume the one-shot"
        );
        assert!(take_strict_plain_text_fit_after_empty_host_float_once(
            &mut pending,
            &plain,
        ));

        assert_eq!(paragraph_page_end_fit_height(21.3, 13.3, false), 13.3);
        assert_eq!(paragraph_page_end_fit_height(21.3, 13.3, true), 21.3);
    }

    fn issue2439_cut_paragraph(line_count: usize, line_height: i32) -> Paragraph {
        Paragraph {
            text: "가".repeat(line_count),
            char_count: line_count as u32,
            line_segs: (0..line_count)
                .map(|line| LineSeg {
                    vertical_pos: line as i32 * line_height,
                    line_height,
                    text_height: line_height,
                    line_spacing: 0,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn issue2439_row_orphan_guard_uses_padded_visible_fragment_height() {
        // Reproduction row contract: columns 0..31 each finish their one line, while the remarks
        // cell consumes two of three 900-HU lines.  Content-only progress is 24px, just below the
        // 25px orphan threshold, but the painted fragment includes 141-HU top/bottom padding.
        let cells = (0..33u16)
            .map(|col| Cell {
                row: 0,
                col,
                row_span: 1,
                col_span: 1,
                width: if col == 32 { 5421 } else { 1940 },
                height: 2693,
                paragraphs: vec![if col == 32 {
                    issue2439_cut_paragraph(3, 900)
                } else {
                    issue2439_cut_paragraph(1, 1000)
                }],
                ..Default::default()
            })
            .collect();
        let table = Table {
            row_count: 1,
            col_count: 33,
            padding: Padding {
                left: 510,
                right: 510,
                top: 141,
                bottom: 141,
            },
            page_break: TablePageBreak::RowBreak,
            cells,
            ..Default::default()
        };
        let engine = LayoutEngine::new(DEFAULT_DPI);
        let styles = ResolvedStyleSet::default();
        let cut = engine.advance_row_cut(&table, 0, &[], 35.6, &styles);

        let mut expected_cut = vec![1usize; 33];
        expected_cut[32] = 2;
        assert_eq!(
            cut.end_cut, expected_cut,
            "remarks must retain its third line"
        );
        assert!(!cut.fully_consumed);
        assert!(
            cut.consumed_height < MIN_TOP_KEEP_PX,
            "fixture must exercise the former content-only orphan rejection: {}",
            cut.consumed_height,
        );

        let visible_height = engine.row_cut_content_height(&table, 0, &[], &cut.end_cut, &styles);
        assert!(
            visible_height > cut.consumed_height,
            "visible fragment must include vertical padding: content={}, visible={visible_height}",
            cut.consumed_height,
        );
        assert!(
            !row_split_meets_min_top_keep(cut.consumed_height, visible_height, false),
            "ordinary RowBreak tables must keep the content-only orphan guard",
        );
        assert!(
            row_split_meets_min_top_keep(cut.consumed_height, visible_height, true),
            "the strict #2439 padded cut must remain on the current page: {visible_height}",
        );
    }

    #[test]
    fn issue2439_partial_rowbreak_repeats_outer_margins_without_repeating_vertical_offset() {
        let table = Table {
            page_break: TablePageBreak::RowBreak,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            common: CommonObjAttr {
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vertical_offset: 399,
                ..Default::default()
            },
            ..Default::default()
        };
        let outer = hwpunit_to_px(283, DEFAULT_DPI);
        let first_host_before = outer + 2.0;
        let first = partial_rowbreak_fragment_spacing_px(
            &table,
            first_host_before,
            false,
            true,
            false,
            DEFAULT_DPI,
        );
        let continuation = partial_rowbreak_fragment_spacing_px(
            &table,
            first_host_before,
            true,
            true,
            false,
            DEFAULT_DPI,
        );

        assert!((first.0 - first_host_before).abs() < 0.01);
        assert!((first.1 - outer).abs() < 0.01);
        assert!((continuation.0 - outer).abs() < 0.01);
        assert!((continuation.1 - outer).abs() < 0.01);
        assert_eq!(table.common.vertical_offset, 399);
        // The helper deliberately excludes vertical_offset; the caller applies it only when
        // `is_continuation == false`, preventing it from being repeated on later pages.

        let ungated = partial_rowbreak_fragment_spacing_px(
            &table,
            first_host_before,
            true,
            false,
            false,
            DEFAULT_DPI,
        );
        assert_eq!(ungated, (0.0, 0.0));
    }

    fn para_at_vpos(vpos: i32) -> Paragraph {
        Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: vpos,
                line_height: 1300,
                text_height: 1300,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn issue2424_table_continuation_cursor_preserves_break_state() {
        let mut cursor = TableContinuationCursor::default();
        assert_eq!(cursor.row, 0);
        assert!(!cursor.is_continuation);
        assert_eq!(cursor.fragments_emitted, 0);

        cursor.advance(3, None, vec![4, 7], true);
        assert_eq!(
            cursor.row, 2,
            "ordinary intra-row split resumes at end_row-1"
        );
        assert_eq!(cursor.start_cut, vec![4, 7]);
        assert!(!cursor.start_cut_is_block);
        assert!(cursor.is_continuation);
        assert_eq!(cursor.fragments_emitted, 1);

        cursor.advance(3, Some(1), vec![8], true);
        assert_eq!(cursor.row, 1, "rowspan block split resumes at block start");
        assert_eq!(cursor.start_cut, vec![8]);
        assert!(cursor.start_cut_is_block);
        assert_eq!(cursor.fragments_emitted, 2);

        cursor.advance(3, None, vec![99], false);
        assert_eq!(cursor.row, 3, "whole-row consumption resumes at end_row");
        assert!(cursor.start_cut.is_empty());
        assert!(!cursor.start_cut_is_block);
        assert_eq!(cursor.fragments_emitted, 3);

        cursor.skip_consumed_row();
        assert_eq!(cursor.row, 4);
        assert_eq!(cursor.fragments_emitted, 3, "skipping emits no fragment");
        cursor.finish(9, true);
        assert_eq!(cursor.row, 9);
        assert!(cursor.start_cut.is_empty());
        assert_eq!(cursor.fragments_emitted, 4);
    }

    #[test]
    fn issue2424_block_table_context_owns_step_lifecycle() {
        let prepared = BlockTableContinuationPreparedState {
            host_placement: None,
            host_frame: (0, 0, 0.0_f64.to_bits()),
            row_count: 3,
            cell_spacing: 0.0,
            can_intra_split: true,
            base_available: 100.0,
            table_available: 100.0,
            layout_engine: crate::renderer::layout::LayoutEngine::new(DEFAULT_DPI),
            rowspan_touched: vec![false; 3],
            cut_row_heights: vec![10.0; 3],
            whole_row_fit_heights: vec![10.0; 3],
            first_fragment_painted_row_footer_guard: 0.0,
            caption_is_top: false,
            caption_overhead: 0.0,
            total_rows_height: 30.0,
            total_footnote_height: 0.0,
            queue_table_footnotes: false,
            table_footnotes: Vec::new(),
            footnote_margin: 0.0,
            host_spacing_total: 0.0,
            host_spacing_before: 0.0,
            host_spacing_after_only: 0.0,
            terminal_nested_child_host_line_spacing: 0.0,
            strict_following_plain_text_fit: false,
            budget_para_start_height: 0.0,
            first_fragment_actual_footnote_boundary: None,
            source_next_positive_rewind: false,
            first_fragment_saved_offset: None,
            source_cellbreak_row_end: None,
            relax_terminal_table_footnote_fit: false,
        };
        let flow_layout =
            PageLayoutInfo::from_page_def(&a4_page_def(), &ColumnDef::default(), DEFAULT_DPI);
        let flow_state = TypesetState::new(flow_layout, 1, 0, 0.0, 0.0, 0.0, Default::default());
        let mut context = BlockTableContinuationContext::new(0, prepared, flow_state);
        assert_eq!(context.fragment_budget, 1, "zero budget clamps to one");
        assert_eq!(context.steps_completed, 0);
        assert!(!context.is_complete());

        context.step(|_, _, cursor| {
            cursor.advance(2, None, vec![4], true);
            TableContinuationIteration::Emitted
        });
        assert_eq!(context.steps_completed, 1);
        assert_eq!(context.cursor.fragments_emitted, 1);
        assert!(!context.is_complete());

        context.step(|_, _, cursor| {
            cursor.finish(3, true);
            TableContinuationIteration::Complete
        });
        assert!(context.is_complete());
        assert_eq!(context.steps_completed, 2);
        assert_eq!(context.cursor.fragments_emitted, 2);
        let flow_state = context.into_flow_state();
        assert_eq!(flow_state.section_index, 0);
    }

    /// [Task #1749] 저장 flow 페이지-마지막 인코딩 판정
    #[test]
    fn test_saved_flow_marks_page_last() {
        // (a) 다음 실줄 없음(문서 끝) — 신뢰 (fe6de3ef 합성 테스트 보호 케이스)
        let doc_end = vec![para_at_vpos(0), para_at_vpos(72626)];
        assert!(saved_flow_marks_page_last(&doc_end, 1));

        // (b) 다음 줄 vpos 리셋(새 쪽) — 신뢰
        let reset = vec![para_at_vpos(72626), para_at_vpos(700)];
        assert!(saved_flow_marks_page_last(&reset, 0));

        // (c) 누적좌표(다음 vpos 증가 지속) — 불신 (36371084 pi18→pi19)
        let cumulative = vec![para_at_vpos(72626), para_at_vpos(74902)];
        assert!(!saved_flow_marks_page_last(&cumulative, 0));

        // (d) 빈 line_segs 문단 건너뛰고 다음 실줄로 판정
        let with_empty = vec![
            para_at_vpos(72626),
            Paragraph::default(),
            para_at_vpos(74902),
        ];
        assert!(!saved_flow_marks_page_last(&with_empty, 0));

        // (e) 누적좌표라도 다음 문단이 명시적 쪽나누기면 페이지-마지막 증거로 신뢰
        //     (36375752 pi26→pi27 [쪽나누기]: vpos 137484→140204 리셋 없음)
        let mut page_break_next = para_at_vpos(140204);
        page_break_next.column_type = ColumnBreakType::Page;
        let cumulative_with_break = vec![para_at_vpos(137484), page_break_next];
        assert!(saved_flow_marks_page_last(&cumulative_with_break, 0));
    }

    fn page_with_items(items: Vec<PageItem>) -> PageContent {
        PageContent {
            page_index: 0,
            page_number: 0,
            page_number_restarted: false,
            section_index: 0,
            layout: PageLayoutInfo::from_page_def(
                &a4_page_def(),
                &ColumnDef::default(),
                DEFAULT_DPI,
            ),
            column_contents: vec![ColumnContent {
                column_index: 0,
                start_height: 0.0,
                endnote_flow: false,
                items,
                zone_layout: None,
                zone_y_offset: 0.0,
                wrap_around_paras: Vec::new(),
                used_height: 0.0,
                wrap_anchors: std::collections::HashMap::new(),
                overlay_continuations: Vec::new(),
                overlay_cuts: Vec::new(),
                inline_placements: Default::default(),
                inline_flow_plans: Default::default(),
                paragraph_float_placements: Default::default(),
            }],
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
            ladder_band_tables: Vec::new(),
        }
    }

    /// 두 PaginationResult의 페이지 수와 각 페이지의 항목 수가 동일한지 비교
    fn assert_pagination_match(old: &PaginationResult, new: &PaginationResult, label: &str) {
        assert_eq!(
            old.pages.len(),
            new.pages.len(),
            "{}: 페이지 수 불일치 (old={}, new={})",
            label,
            old.pages.len(),
            new.pages.len(),
        );

        for (pi, (old_page, new_page)) in old.pages.iter().zip(new.pages.iter()).enumerate() {
            assert_eq!(
                old_page.column_contents.len(),
                new_page.column_contents.len(),
                "{}: p{} 단 수 불일치",
                label,
                pi,
            );

            for (ci, (old_col, new_col)) in old_page
                .column_contents
                .iter()
                .zip(new_page.column_contents.iter())
                .enumerate()
            {
                assert_eq!(
                    old_col.items.len(),
                    new_col.items.len(),
                    "{}: p{} col{} 항목 수 불일치 (old={}, new={})",
                    label,
                    pi,
                    ci,
                    old_col.items.len(),
                    new_col.items.len(),
                );
            }
        }
    }

    #[test]
    fn test_typeset_engine_creation() {
        let engine = TypesetEngine::new(96.0);
        assert_eq!(engine.dpi, 96.0);
    }

    #[test]
    fn test_typeset_empty_paragraphs() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &[],
            &composed,
            &styles,
            &a4_page_def(),
            &ColumnDef::default(),
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 1, "빈 문서도 최소 1페이지");
    }

    #[test]
    fn table_continuation_does_not_reapply_page_hide() {
        let hide = crate::model::control::PageHide {
            hide_master_page: true,
            hide_page_num: true,
            ..Default::default()
        };
        let mut pages = vec![
            page_with_items(vec![PageItem::PartialTable {
                para_index: 7,
                control_index: 0,
                start_row: 0,
                end_row: 2,
                is_continuation: false,
                start_cut: Vec::new(),
                end_cut: Vec::new(),
                is_block_split: false,
                start_cut_is_block: false,
                row_cursor_is_nested: false,
                end_row_height_override: None,
                start_row_height_override: None,
            }]),
            page_with_items(vec![PageItem::PartialTable {
                para_index: 7,
                control_index: 0,
                start_row: 2,
                end_row: 4,
                is_continuation: true,
                start_cut: Vec::new(),
                end_cut: Vec::new(),
                is_block_split: false,
                start_cut_is_block: false,
                row_cursor_is_nested: false,
                end_row_height_override: None,
                start_row_height_override: None,
            }]),
        ];

        let mut paragraphs = vec![Paragraph::default(); 8];
        paragraphs[7].controls = vec![Control::Table(Box::default()), Control::PageHide(hide)];
        TypesetEngine::finalize_pages(&mut pages, &[], &None, &paragraphs);

        assert!(pages[0].page_hide.is_some());
        assert!(pages[1].page_hide.is_none());
    }

    #[test]
    fn footnote_area_reserve_uses_section_shape_metrics() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let shape = FootnoteShape {
            separator_margin_top: 1000,
            note_spacing: 700,
            raw_unknown: 900,
            separator_line_width: 4,
            ..Default::default()
        };
        let note1 = Paragraph {
            text: "첫 각주".to_string(),
            line_segs: vec![LineSeg {
                line_height: 400,
                ..Default::default()
            }],
            ..Default::default()
        };
        let note2 = Paragraph {
            text: "둘째 각주".to_string(),
            line_segs: vec![LineSeg {
                line_height: 600,
                ..Default::default()
            }],
            ..Default::default()
        };
        let paras = vec![Paragraph {
            text: "본문".to_string(),
            line_segs: vec![LineSeg {
                line_height: 400,
                ..Default::default()
            }],
            controls: vec![
                Control::Footnote(Box::new(crate::model::footnote::Footnote {
                    number: 1,
                    paragraphs: vec![note1],
                    ..Default::default()
                })),
                Control::Footnote(Box::new(crate::model::footnote::Footnote {
                    number: 2,
                    paragraphs: vec![note2],
                    ..Default::default()
                })),
            ],
            ..Default::default()
        }];
        let composed: Vec<ComposedParagraph> = paras
            .iter()
            .map(crate::renderer::composer::compose_paragraph)
            .collect();

        let result = engine.typeset_section_with_variant(
            &paras,
            &composed,
            &styles,
            &a4_page_def(),
            &ColumnDef::default(),
            0,
            &[],
            false,
            Default::default(),
            false,
            false,
            Some(&shape),
            None,
            &std::collections::HashSet::new(),
            EndnoteDeferral::None,
        );

        let expected = footnote_separator_overhead_px(&shape, DEFAULT_DPI)
            + hwpunit_to_px(400, DEFAULT_DPI)
            + footnote_between_notes_margin_px(&shape, DEFAULT_DPI)
            + hwpunit_to_px(600, DEFAULT_DPI);
        let page = result.pages.first().expect("page");
        assert_eq!(page.footnotes.len(), 2);
        assert!((page.layout.footnote_area.height - expected).abs() < 0.01);
    }

    #[test]
    fn test_typeset_single_paragraph() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let paras = vec![make_paragraph_with_height(400)];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let new_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        assert_pagination_match(&old_result, &new_result, "single_paragraph");
    }

    #[test]
    fn test_typeset_page_overflow() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        // 빈 문단만으로는 마지막 blank-only page 제거 경로를 검증하게 된다. 이
        // 테스트의 계약은 overflow 중 가시 문단의 배치이므로, 실제 내용도 넣는다.
        let mut paras: Vec<Paragraph> =
            (0..100).map(|_| make_paragraph_with_height(2000)).collect();
        for (idx, para) in paras.iter_mut().enumerate() {
            para.text = format!("paragraph {idx}");
        }
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let new_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        // `Paginator`의 옛 measured-height 근사는 독립적인 oracle이 아니다. 이
        // synthetic case는 가시 문단 100개의 전량 보존과 body-fit(4쪽)을 직접
        // 고정한다.
        assert_eq!(
            new_result.pages.len(),
            4,
            "100개 2000-HWPUNIT 문단은 A4 4쪽"
        );
        let placed: Vec<_> = new_result
            .pages
            .iter()
            .flat_map(|page| page.column_contents.iter())
            .flat_map(|column| column.items.iter())
            .map(PageItem::para_index)
            .collect();
        assert_eq!(
            placed,
            (0..100).collect::<Vec<_>>(),
            "문단 손실·중복 없이 배치"
        );
    }

    #[test]
    fn saved_single_line_at_body_bottom_stays_on_current_page() {
        let engine = TypesetEngine::with_default_dpi();
        let mut styles = ResolvedStyleSet::default();
        styles
            .para_styles
            .push(crate::renderer::style_resolver::ResolvedParaStyle {
                spacing_before: 9.3,
                ..Default::default()
            });
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let line_height = 1200;
        let line_spacing = 840;
        let spacing_before_hu = crate::renderer::px_to_hwpunit(9.3, DEFAULT_DPI);
        let lead_height = body_height_hu - line_height;
        let lead_measured_height = lead_height - spacing_before_hu + 600;
        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: lead_measured_height,
                    text_height: lead_measured_height,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                para_shape_id: 0,
                text: "tail".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: lead_height,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.pages[0].column_contents[0].items.len(), 2);
    }

    #[test]
    fn two_line_tail_before_vpos_reset_stays_on_current_page_when_visible_bottom_fits() {
        let engine = TypesetEngine::with_default_dpi();
        let mut styles = ResolvedStyleSet::default();
        styles
            .para_styles
            .push(crate::renderer::style_resolver::ResolvedParaStyle::default());
        styles
            .para_styles
            .push(crate::renderer::style_resolver::ResolvedParaStyle {
                spacing_before: hwpunit_to_px(2400, DEFAULT_DPI),
                spacing_after: hwpunit_to_px(1400, DEFAULT_DPI),
                ..Default::default()
            });

        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let line_height = 1200;
        let line_spacing = 840;
        let first_vpos = body_height_hu - 3740;
        let lead_height = first_vpos - 2400;

        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: lead_height,
                    text_height: lead_height,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                para_shape_id: 1,
                text: "line one\nline two".to_string(),
                line_segs: vec![
                    LineSeg {
                        vertical_pos: first_vpos,
                        line_height,
                        text_height: line_height,
                        line_spacing,
                        ..Default::default()
                    },
                    LineSeg {
                        vertical_pos: first_vpos + line_height + line_spacing,
                        line_height,
                        text_height: line_height,
                        line_spacing,
                        text_start: 9,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Paragraph {
                text: "next page".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height,
                    text_height: line_height,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 2);
        assert!(matches!(
            result.pages[0].column_contents[0].items.as_slice(),
            [
                PageItem::FullParagraph { para_index: 0 },
                PageItem::FullParagraph { para_index: 1 }
            ]
        ));
        assert!(matches!(
            result.pages[1].column_contents[0].items.as_slice(),
            [PageItem::FullParagraph { para_index: 2 }]
        ));
    }

    #[test]
    fn multiline_saved_vpos_tail_does_not_split_to_near_empty_page() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let line_height = 1200;
        let line_spacing = 720;
        let first_vpos = body_height_hu - (line_height + line_spacing) * 3 - line_height;
        let lead_height = first_vpos + line_height;

        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: lead_height,
                    text_height: lead_height,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                text: "tail one\ntail two\ntail three\ntail four".to_string(),
                line_segs: (0..4)
                    .map(|line_idx| LineSeg {
                        vertical_pos: first_vpos + (line_height + line_spacing) * line_idx,
                        line_height,
                        text_height: line_height,
                        line_spacing,
                        text_start: line_idx as u32 * 9,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            },
            Paragraph {
                text: "next page".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 2);
        assert!(matches!(
            result.pages[0].column_contents[0].items.as_slice(),
            [
                PageItem::FullParagraph { para_index: 0 },
                PageItem::FullParagraph { para_index: 1 }
            ]
        ));
        assert!(matches!(
            result.pages[1].column_contents[0].items.as_slice(),
            [PageItem::FullParagraph { para_index: 2 }]
        ));
    }

    #[test]
    fn page_bottom_empty_paragraph_before_vpos_reset_does_not_create_blank_page() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let line_height = 1000;
        let line_spacing = 400;

        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: body_height_hu - 100,
                    text_height: body_height_hu - 100,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: body_height_hu - 1200,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                text: "next page".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 2);
        assert!(result.hidden_empty_paras.contains(&1));
        let has_empty_only_page = result.pages.iter().any(|page| {
            page.column_contents.iter().any(|col| {
                matches!(
                    col.items.as_slice(),
                    [PageItem::FullParagraph { para_index: 1 }]
                )
            })
        });
        assert!(!has_empty_only_page);
        assert!(matches!(
            result.pages[1].column_contents[0].items.as_slice(),
            [PageItem::FullParagraph { para_index: 2 }]
        ));
    }

    #[test]
    fn page_bottom_empty_run_before_vpos_reset_does_not_create_blank_page() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let line_height = 1200;
        let line_spacing = 720;

        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: body_height_hu - 5000,
                    text_height: body_height_hu - 5000,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: body_height_hu - 3800,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: body_height_hu - 1880,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                text: "next page".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height,
                    text_height: line_height,
                    line_spacing,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 2);
        assert!(result.hidden_empty_paras.contains(&1));
        assert!(result.hidden_empty_paras.contains(&2));
        let has_empty_only_page = result.pages.iter().any(|page| {
            page.column_contents.iter().any(|col| {
                matches!(
                    col.items.as_slice(),
                    [PageItem::FullParagraph { para_index: 1 }]
                        | [PageItem::FullParagraph { para_index: 2 }]
                        | [
                            PageItem::FullParagraph { para_index: 1 },
                            PageItem::FullParagraph { para_index: 2 }
                        ]
                )
            })
        });
        assert!(!has_empty_only_page);
        assert!(matches!(
            result.pages[1].column_contents[0].items.as_slice(),
            [PageItem::FullParagraph { para_index: 3 }]
        ));
    }

    #[test]
    fn saved_tac_table_line_at_body_bottom_stays_on_current_page() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let body_height_hu =
            crate::renderer::px_to_hwpunit(layout.available_body_height(), DEFAULT_DPI);
        let table_height = 10_072;
        let table_vpos = body_height_hu - table_height;
        let lead_height = table_vpos + 900;
        let paras = vec![
            Paragraph {
                text: "lead".to_string(),
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: lead_height,
                    text_height: lead_height,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Paragraph {
                controls: vec![Control::Table(Box::new(crate::model::table::Table {
                    attr: 1,
                    row_count: 3,
                    col_count: 3,
                    common: crate::model::shape::CommonObjAttr {
                        treat_as_char: true,
                        text_wrap: crate::model::shape::TextWrap::TopAndBottom,
                        height: table_height as u32,
                        ..Default::default()
                    },
                    ..Default::default()
                }))],
                line_segs: vec![LineSeg {
                    vertical_pos: table_vpos,
                    line_height: table_height,
                    text_height: table_height,
                    line_spacing: 120,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(result.pages.len(), 1);
        assert!(matches!(
            result.pages[0].column_contents[0].items.as_slice(),
            [
                PageItem::FullParagraph { para_index: 0 },
                PageItem::Table { para_index: 1, .. }
            ]
        ));
    }

    /// [Task #1363 v3 Stage 2] scratch 측정 부작용 격리 회귀 가드.
    ///
    /// `measure_endnote_para_advance` 는 매 호출 `LayoutEngine::new()` 로 독립 인스턴스를
    /// 쓰므로 (a) 양수·유한, (b) 동일 엔진 반복 호출에 결정적(호출 간 상태 무누적),
    /// (c) 독립 `TypesetEngine` 인스턴스 간 동일(전역/공유 가변 상태 누수 없음)이어야 한다.
    /// scratch 의 numbering/overflow/last_item_content_bottom 변이가 측정에만 머무름을 실증.
    #[test]
    fn test_measure_endnote_advance_side_effect_free() {
        use crate::renderer::composer::compose_paragraph;

        let para = Paragraph {
            text: "각주 측정 격리 회귀 가드 문장".to_string(),
            line_segs: vec![LineSeg {
                line_height: 1000,
                baseline_distance: 850,
                ..Default::default()
            }],
            ..Default::default()
        };
        let composed = compose_paragraph(&para);
        let styles = ResolvedStyleSet::default();
        let item = PageItem::FullParagraph { para_index: 900 };
        let (en_col_w, available, y_start) = (280.0_f64, 900.0_f64, 100.0_f64);

        let engine = TypesetEngine::new(96.0);
        let first = engine.measure_endnote_para_advance(
            &para, &composed, &styles, en_col_w, available, y_start, &item, 0, 900,
        );

        // (a) 양수·유한 — 실제 텍스트 para 는 advance 를 만든다.
        assert!(
            first.is_finite() && first > 0.0,
            "advance must be positive finite: {first}",
        );

        // (b) 동일 엔진 반복 호출 → 결정적 (scratch 호출 간 상태 무누적).
        for _ in 0..5 {
            let v = engine.measure_endnote_para_advance(
                &para, &composed, &styles, en_col_w, available, y_start, &item, 0, 900,
            );
            assert_eq!(v, first, "repeat call drifted — scratch 상태 누적 누수");
        }

        // (c) 독립 TypesetEngine 인스턴스 → 동일 (전역 가변 상태 누수 없음).
        let engine2 = TypesetEngine::new(96.0);
        let other = engine2.measure_endnote_para_advance(
            &para, &composed, &styles, en_col_w, available, y_start, &item, 0, 900,
        );
        assert_eq!(other, first, "independent engine differs — 전역 상태 누수");
    }

    #[test]
    fn test_typeset_line_split() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 여러 줄이 있는 큰 문단 (페이지 경계에서 줄 단위 분할)
        let paras = vec![Paragraph {
            line_segs: (0..50)
                .map(|_| LineSeg {
                    line_height: 1800,
                    line_spacing: 200,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let new_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        assert_pagination_match(&old_result, &new_result, "line_split");
    }

    #[test]
    fn test_typeset_mixed_paragraphs() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 다양한 높이의 문단 혼합
        let paras: Vec<Paragraph> = vec![
            make_paragraph_with_height(400),
            make_paragraph_with_height(10000), // 큰 문단
            make_paragraph_with_height(400),
            make_paragraph_with_height(800),
            make_paragraph_with_height(20000), // 매우 큰 문단
            make_paragraph_with_height(400),
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let new_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        assert_pagination_match(&old_result, &new_result, "mixed_paragraphs");
    }

    #[test]
    fn test_typeset_page_break() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 강제 쪽 나누기가 있는 문단
        let paras = vec![
            make_paragraph_with_height(400),
            {
                let mut p = make_paragraph_with_height(400);
                p.column_type = ColumnBreakType::Page;
                p
            },
            make_paragraph_with_height(400),
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let new_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        assert_pagination_match(&old_result, &new_result, "page_break");
        assert_eq!(new_result.pages.len(), 2, "쪽 나누기로 2페이지");
    }

    // [Task #1046] 사후 reflow force-break hint 메커니즘 검증.
    #[test]
    fn test_typeset_force_break_before_hint() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        // 한 페이지에 충분히 들어가는 3개 문단
        let mut paras = vec![
            make_paragraph_with_height(400),
            make_paragraph_with_height(400),
            make_paragraph_with_height(400),
        ];
        for (idx, para) in paras.iter_mut().enumerate() {
            para.text = format!("paragraph {idx}");
        }
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        // hint 없음 → 1페이지
        let baseline = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &std::collections::HashSet::new(),
        );
        assert_eq!(baseline.pages.len(), 1, "hint 없으면 3문단 모두 1페이지");

        // para_idx=1 에 force-break hint → para 1 이 2페이지에서 시작
        let mut hint = std::collections::HashSet::new();
        hint.insert(1usize);
        let reflowed = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &[],
            false,
            &hint,
        );
        assert_eq!(reflowed.pages.len(), 2, "para1 force-break 로 2페이지");
        let page0_paras: Vec<usize> = reflowed.pages[0]
            .column_contents
            .iter()
            .flat_map(|cc| cc.items.iter().map(|it| it.para_index()))
            .collect();
        let page1_paras: Vec<usize> = reflowed.pages[1]
            .column_contents
            .iter()
            .flat_map(|cc| cc.items.iter().map(|it| it.para_index()))
            .collect();
        assert_eq!(page0_paras, vec![0], "1페이지엔 para0 만");
        assert_eq!(page1_paras, vec![1, 2], "2페이지엔 para1,2");
    }

    // ========================================================
    // 실제 HWP 파일 비교 테스트
    // ========================================================

    /// 실제 HWP 파일로 기존 Paginator와 TypesetEngine 결과 비교
    fn compare_with_hwp_file(path: &str) {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("skip: {} not found", path);
                return;
            }
        };
        let doc = match crate::document_core::DocumentCore::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {} parse error: {}", path, e);
                return;
            }
        };

        let engine = TypesetEngine::with_default_dpi();

        for (sec_idx, section) in doc.document.sections.iter().enumerate() {
            let composed = &doc.composed[sec_idx];
            let measured_tables = &doc.measured_tables[sec_idx];
            let column_def =
                crate::document_core::DocumentCore::find_initial_column_def(&section.paragraphs);

            // 구역에 표가 포함되어 있는지 확인
            let has_tables = section
                .paragraphs
                .iter()
                .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));

            let new_result = engine.typeset_section(
                &section.paragraphs,
                composed,
                &doc.styles,
                &section.section_def.page_def,
                &column_def,
                sec_idx,
                measured_tables,
                section.section_def.hide_empty_line,
                &std::collections::HashSet::new(),
            );

            let old_result = &doc.pagination[sec_idx];
            let label = format!("{} sec{}", path, sec_idx);

            if has_tables {
                // 표가 포함된 구역: Phase 2 전환 전까지 차이 허용 (경고만 출력)
                if old_result.pages.len() != new_result.pages.len() {
                    eprintln!(
                        "WARN {}: 표 포함 구역 페이지 수 차이 (old={}, new={}) — Phase 2에서 해결",
                        label,
                        old_result.pages.len(),
                        new_result.pages.len(),
                    );
                }
            } else {
                // 비-표 구역: 완전 일치 필수
                assert_eq!(
                    old_result.pages.len(),
                    new_result.pages.len(),
                    "{}: 페이지 수 불일치 (old={}, new={})",
                    label,
                    old_result.pages.len(),
                    new_result.pages.len(),
                );

                for (pi, (old_page, new_page)) in old_result
                    .pages
                    .iter()
                    .zip(new_result.pages.iter())
                    .enumerate()
                {
                    assert_eq!(
                        old_page.column_contents.len(),
                        new_page.column_contents.len(),
                        "{}: p{} 단 수 불일치",
                        label,
                        pi,
                    );
                }
            }
        }
    }

    #[test]
    fn test_typeset_vs_paginator_p222() {
        // p222.hwp sec2는 표가 많아 Phase 2 전환 전까지 차이 발생 가능
        // Phase 1에서는 비-표 문단만 검증
        compare_with_hwp_file("samples/p222.hwp");
    }

    #[test]
    fn test_typeset_vs_paginator_hongbo() {
        compare_with_hwp_file("samples/20250130-hongbo.hwp");
    }

    #[test]
    fn test_typeset_vs_paginator_biz_plan() {
        compare_with_hwp_file("samples/biz_plan.hwp");
    }

    /// Issue #703: BehindText/InFrontOfText 표는 본문 흐름에서 제외되어야 한다.
    ///
    /// 글뒤로 (BehindText) / 글앞으로 (InFrontOfText) 표는 시각적으로 본문 텍스트 뒤/앞에
    /// 절대 좌표로 배치되는 데코레이션 (워터마크/배경 등) 이며, 본문 흐름의 vertical advance 에
    /// 영향을 주지 않는다. `pagination/engine.rs:976-981` 와 동일 시멘틱.
    ///
    /// 결함 메커니즘: typeset_block_table → place_table_with_text → `cur_h += table_total_height`
    /// (line 1594) 가 BehindText/InFrontOfText 표에 대해서도 적용되어 본문 흐름 누적이 발생.
    ///
    /// 본 테스트는 BIG BehindText 표 (≈300 mm 높이) 를 1 페이지 본문 안에 넣어두고 후속
    /// paragraph 가 동일 페이지에 들어감을 검증한다. 결함 시 BehindText 표의 거대 height 가
    /// cur_h 에 가산되어 후속 paragraph 가 다음 페이지로 밀림.
    #[test]
    fn test_typeset_703_behind_text_table_no_flow_advance() {
        use crate::model::shape::TextWrap;
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        // BehindText 1×1 표: 본문 높이의 약 80% 차지 (60000 HU ≈ 800 px @96dpi).
        // BehindText 는 데코레이션이므로 본문 흐름 누적 0 이어야 정상.
        // 결함 시 cur_h 에 800 px 가산 → 후속 1 단락도 fit 실패 → 페이지 분할.
        let mut table = crate::model::table::Table {
            row_count: 1,
            col_count: 1,
            cells: vec![crate::model::table::Cell {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                width: 51974,
                height: 60000,
                paragraphs: vec![Paragraph::default()],
                ..Default::default()
            }],
            ..Default::default()
        };
        table.common.text_wrap = TextWrap::BehindText;
        table.common.treat_as_char = false;
        table.common.width = 51974;
        table.common.height = 60000; // ≈800 px @96dpi — 본문 80% 점유 (결함 시 가산되는 양)

        let host_para = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 1000,
                line_spacing: 600,
                ..Default::default()
            }],
            controls: vec![crate::model::control::Control::Table(Box::new(table))],
            ..Default::default()
        };

        // 후속 5 단락 — 본문 정상 흐름이면 호스트(21px) + 5 × 13px = 86 px (1 페이지 여유)
        // 결함 시 호스트(21+800=821px) + 첫 단락(13px) = 834 px 도 fit, 더 추가 시 결국 분할
        // → 단순히 페이지 수 정확히 비교 필요.
        let mut paras = vec![host_para];
        for _ in 0..5 {
            paras.push(make_paragraph_with_height(1000));
        }

        let (paginator_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let typeset_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        // 검증 1: paginator (engine.rs reference) 는 1 페이지에 모두 배치
        assert_eq!(
            paginator_result.pages.len(),
            1,
            "[reference] BehindText 표 + 5 후속 paragraph 는 paginator 에서 1 페이지에 들어가야 함",
        );

        // 검증 2: typeset 결과도 1 페이지 (현재 결함 시 RED — typeset 이 BehindText 표 height 를 누적)
        assert_eq!(
            typeset_result.pages.len(),
            1,
            "[BUG #703] typeset 도 1 페이지여야 함. 결함 시 BehindText 표 height ≈800 px 가 \
             cur_h 에 가산되어 후속 paragraph 가 다음 페이지로 밀림 (RED)",
        );
    }

    /// Issue #1995: 한 문단에 근접-전면(near-full-page) non-TAC 이미지가 여러 장이면
    /// 각 이미지는 공존 불가하므로 각각 한 페이지에 단독 배치되어야 한다.
    ///
    /// 결함(수정 전): 임베드 매뉴얼을 페이지 이미지로 삽입한 문서(한 문단에 전면 이미지
    /// 수십~백장)에서 rhwp 가 전부 한 앵커에 스택 → 문서가 과소 페이지(예: rhwp 174 vs
    /// 한글 268). 수정: typeset 인라인 컨트롤 방출 시 각 전면 이미지를 force_new_page 로
    /// 개별 페이지에 배치.
    #[test]
    fn test_typeset_1995_multi_fullpage_images_split_pages() {
        use crate::model::shape::TextWrap;
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        // 전면 non-TAC 이미지 3장 (각 60000 HU ≈ 800px, 본문 60%+ 점유, wrap=Square)
        let make_pic = || {
            let mut pic = crate::model::image::Picture::default();
            pic.common.treat_as_char = false;
            pic.common.text_wrap = TextWrap::Square;
            pic.common.width = 51974;
            pic.common.height = 60000;
            crate::model::control::Control::Picture(Box::new(pic))
        };
        let host_para = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 1000,
                line_spacing: 600,
                ..Default::default()
            }],
            controls: vec![make_pic(), make_pic(), make_pic()],
            ..Default::default()
        };
        let paras = vec![host_para];

        let (_paginator_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let typeset_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        // 전면 이미지 3장 → 각각 한 페이지 (>= 3). 결함 시 1 페이지에 스택(RED).
        assert!(
            typeset_result.pages.len() >= 3,
            "[#1995] 전면 non-TAC 이미지 3장은 각각 한 페이지에 단독 배치되어야 함(>= 3 페이지). \
             실제 {} 페이지 — 미수정 시 한 앵커에 스택",
            typeset_result.pages.len(),
        );
    }

    /// #4770: 같은 앵커에 겹친 Square 전면 그림 무리라도, 저장 첫 줄이 그림 폭 이상
    /// 오른쪽에서 시작하면(cs ≥ 그림 폭 — 한글이 빈 줄을 그림 옆에 끼운 저장 흔적)
    /// 한글은 스택을 앵커 쪽에 남긴다. 낱장 분산(#1995)을 걸면 안 된다.
    ///
    /// HPV 코호트 s2/pi=1007 실측: 그림 24장(150×212mm) cs=42520=그림 폭·sw=3480,
    /// 한글 1쪽 ↔ 분산 시 24쪽(+24) — 이슈 #4770.
    #[test]
    fn test_typeset_4770_stored_line_beside_pile_keeps_stack_on_anchor_page() {
        use crate::model::shape::TextWrap;
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        // #1995 테스트와 같은 전면 그림 3장 — 유일한 차이는 저장 첫 줄의
        // column_start 가 그림 폭 이상(= 줄이 그림 옆에 끼임)이라는 것.
        let make_pic = || {
            let mut pic = crate::model::image::Picture::default();
            pic.common.treat_as_char = false;
            pic.common.text_wrap = TextWrap::Square;
            pic.common.width = 51974;
            pic.common.height = 60000;
            crate::model::control::Control::Picture(Box::new(pic))
        };
        let host_para = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 1000,
                line_spacing: 600,
                column_start: 51974,
                segment_width: 3480,
                ..Default::default()
            }],
            controls: vec![make_pic(), make_pic(), make_pic()],
            ..Default::default()
        };
        let paras = vec![host_para];

        let (_paginator_result, measured) =
            paginator.paginate(&paras, &composed, &styles, &page_def, &col_def, 0);
        let typeset_result = engine.typeset_section(
            &paras,
            &composed,
            &styles,
            &page_def,
            &col_def,
            0,
            &measured.tables,
            false,
            &std::collections::HashSet::new(),
        );

        assert_eq!(
            typeset_result.pages.len(),
            1,
            "[#4770] 저장 줄이 그림 옆에 끼인(cs ≥ 그림 폭) 스택은 앵커 쪽 1 페이지에 \
             남아야 함. 실제 {} 페이지 — 낱장 분산이 오발동",
            typeset_result.pages.len(),
        );
    }

    #[test]
    fn test_4770_anchor_pile_contract_requires_square_stack_and_page_bottom() {
        use crate::document_core::queries::rendering::body_pile_stays_on_anchor_page;
        use crate::model::page::PageAreas;
        use crate::model::shape::TextWrap;

        let page_def = a4_page_def();
        let body_area = PageAreas::from_page_def(&page_def).body_area;
        let make_para = |text_wrap| {
            let make_pic = || {
                let mut pic = crate::model::image::Picture::default();
                pic.common.treat_as_char = false;
                pic.common.text_wrap = text_wrap;
                pic.common.allow_overlap = false;
                pic.common.width = 51974;
                pic.common.height = 60000;
                crate::model::control::Control::Picture(Box::new(pic))
            };
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: body_area.bottom - 60000,
                    column_start: 51974,
                    segment_width: 3480,
                    ..Default::default()
                }],
                controls: vec![make_pic(), make_pic(), make_pic()],
                ..Default::default()
            }
        };

        assert!(
            body_pile_stays_on_anchor_page(&make_para(TextWrap::Square), 30000, body_area.bottom),
            "페이지 상단 기준 vpos가 본문 하단에 정확히 닿는 Square 스택은 앵커 쪽에 남아야 함"
        );
        assert!(
            !body_pile_stays_on_anchor_page(
                &make_para(TextWrap::TopAndBottom),
                30000,
                body_area.bottom,
            ),
            "TopAndBottom 전면 그림은 cs/vpos가 같아도 #1995 낱장 배치를 억제하면 안 됨"
        );
    }

    /// #3821: page-tail Square 그림은 첫 reset 문단뿐 아니라 그림 높이 안의 연속
    /// guide/visible 문단까지 같은 narrow band를 유지해야 한다. 범위 밖 첫 문단은
    /// 즉시 끊어 다음 일반 본문으로 anchor가 새지 않아야 한다.
    #[test]
    fn issue_3821_square_picture_wrap_band_is_bounded_and_contiguous() {
        let seg = |vertical_pos, column_start, segment_width| LineSeg {
            vertical_pos,
            column_start,
            segment_width,
            ..Default::default()
        };
        let paragraphs = vec![
            // p1693 형상: 앞쪽 full-width tail 뒤에 vpos reset narrow line가 공존.
            Paragraph {
                line_segs: vec![seg(-2000, 0, 45352), seg(0, 0, 25139), seg(2000, 0, 25139)],
                ..Default::default()
            },
            // p1694..1696 형상: 빈 guide 문단도 이후 visible text까지 contract를 잇는다.
            Paragraph {
                line_segs: vec![seg(4000, 0, 25139)],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![seg(4600, 0, 25139)],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![seg(5200, 0, 25139)],
                ..Default::default()
            },
            // p1697 visible text의 마지막 줄 시작점.
            Paragraph {
                line_segs: vec![seg(21800, 0, 25139)],
                ..Default::default()
            },
            // p1698: 그림 실제 bottom(23231 HU) 밖 — 반드시 제외.
            Paragraph {
                line_segs: vec![seg(23800, 0, 25139)],
                ..Default::default()
            },
        ];

        assert_eq!(
            controls::deferred_picture::square_picture_wrap_band_target_paragraphs(
                &paragraphs,
                0,
                0,
                23231,
                0,
                25139,
            ),
            vec![0, 1, 2, 3, 4],
            "actual image band must include p1693..p1697 and stop before p1698",
        );
    }
}
