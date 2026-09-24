//! 빈 호스트 문단의 float 표 lane 수용·예약 좌표 조회.
//! 기존 저장 앵커와 형제 표 판별을 보존하며 페이지·lane 상태는 변경하지 않는다.

use super::super::{
    is_single_rowbreak_table_with_trustworthy_declared_height, is_stored_anchor_picture_table,
    is_synthetic_line_seg, para_has_visible_text, stored_single_topbottom_top_px, FormattedTable,
};
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{
    composer::ComposedParagraph,
    float_placement::{
        horizontal_range, is_para_topbottom_float, signed_hwpunit, FloatLaneSet,
        FloatPlacementContext,
    },
    hwpunit_to_px,
    page_layout::PageLayoutInfo,
    pagination::PageItem,
    style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) struct EmptyFloatPage<'a> {
    pub layout: &'a PageLayoutInfo,
    pub current_column: u16,
    pub profile: LayoutCompatibilityProfile,
    pub current_height: f64,
    pub current_items: &'a [PageItem],
}

pub(in crate::renderer::typeset) struct EmptyFloatPlacement {
    pub x_start: f64,
    pub x_end: f64,
    pub raw_top: f64,
    pub reserved_height: f64,
}

/// 예산 조회는 기존 수평 범위 계산 뒤에만 실행한다. 거절된 후보에는 상태 효과가 없다.
#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    para_idx: usize,
    ctrl_idx: usize,
    para: &Paragraph,
    table: &crate::model::table::Table,
    ft: &FormattedTable,
    composed: Option<&ComposedParagraph>,
    next_para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    para_start_height: f64,
    lanes: &FloatLaneSet,
    page: EmptyFloatPage<'_>,
    available_height: impl FnOnce() -> f64,
    dpi: f64,
) -> Option<EmptyFloatPlacement> {
    use crate::model::shape::{TextWrap, VertRelTo};

    let is_topbottom_para_float = is_para_topbottom_float(&table.common);
    // 한컴 native HWP는 빈 host paragraph에 나란히 놓인 복수 어울림(Square) 표를
    // 본문 block의 세로 합으로 소비하지 않는다. 저장 LINE_SEG의 vpos가 두 표의
    // 공통 anchor를 가리키고, 표는 가로 lane을 나눠 같은 페이지에 떠 있다
    // (1351000 pi=260: 표 2개가 p14의 표 2/그림 9로 함께 배치). 일반 block
    // 경로는 300px + 287px을 차례로 더해 p15를 하나 더 만들었다.
    let is_square_sibling_float = !table.common.treat_as_char
        && matches!(table.common.text_wrap, TextWrap::Square)
        && matches!(table.common.vert_rel_to, VertRelTo::Para);
    if !(is_topbottom_para_float || is_square_sibling_float) || para_has_visible_text(para) {
        return None;
    }
    let topbottom_float_count = para
        .controls
        .iter()
        .filter(|ctrl| {
            matches!(ctrl,
                Control::Table(t)
                    if is_para_topbottom_float(&t.common)
            )
        })
        .take(2)
        .count();
    let square_sibling_count = para
        .controls
        .iter()
        .filter(|ctrl| {
            matches!(ctrl,
                Control::Table(t)
                    if !t.common.treat_as_char
                        && matches!(t.common.text_wrap, TextWrap::Square)
                        && matches!(t.common.vert_rel_to, VertRelTo::Para)
            )
        })
        .take(2)
        .count();
    if is_square_sibling_float && square_sibling_count < 2 {
        return None;
    }

    let column_area = page
        .layout
        .column_areas
        .get(page.current_column as usize)
        .copied()
        .unwrap_or(page.layout.body_area);
    let width_px = hwpunit_to_px(signed_hwpunit(table.common.width), dpi);
    if width_px <= 0.0 || ft.effective_height <= 0.0 {
        return None;
    }

    let para_style_id = composed
        .map(|c| c.para_style_id as usize)
        .unwrap_or(para.para_shape_id as usize);
    let para_style = styles.para_styles.get(para_style_id);
    let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
    let indent = para_style.map(|s| s.indent).unwrap_or(0.0);
    let effective_margin = if indent > 0.0 {
        margin_left + indent
    } else {
        margin_left
    };
    let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);

    let placement_ctx = FloatPlacementContext::new(column_area)
        .with_body_area(page.layout.body_area)
        .with_paper_width(page.layout.page_width)
        .with_host_margins(effective_margin, margin_right);
    let (x_start, x_end) = horizontal_range(&table.common, width_px, placement_ctx, dpi);

    let available = available_height();

    let v_offset_px = hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    // Square sibling 표는 HWP 페이지 좌표계의 저장 vpos를 공유한다. 이 값을 현재
    // 흐름 cursor에 맞춰 page base만큼 빼면 p14의 두 표가 본문 하단으로 밀리고,
    // 렌더 단계의 절대 lane과도 달라진다. 현재 쪽 본문 안에 드는 저장값만 그대로
    // 쓰고, synthetic/no-page vpos는 기존 para_start 기준으로 폴백한다.
    let saved_page_top = is_square_sibling_float
        .then(|| {
            para.line_segs
                .iter()
                .find(|seg| !is_synthetic_line_seg(seg))
                .map(|seg| hwpunit_to_px(seg.vertical_pos, dpi))
                .filter(|top| *top >= 0.0 && *top <= available)
        })
        .flatten();
    // empty-host TopAndBottom 그림 표(두 프로필) 또는 선언 높이를 신뢰할 수 있는
    // 1×1 표(native HWP5)는 다음 문단의 저장 vpos까지 증가할 때만
    // raw vpos를 물리 page anchor로 해석한다. 다음 vpos가 되감기는 p14 그림 8
    // 같은 page-boundary 형상은 이전 anchor가 남아 있을 수 있으므로 기존 flow를
    // 보존한다. native HWP5와 원본 HWPX가 모두 이 저장 형상을 제공한다. HWPX를
    // 일반 block fit으로 보내면 각주 안전 여유 수 px 때문에 실제로는 들어가는
    // 그림 11이 다음 쪽으로 이월된다 (#3738).
    let single_rowbreak_declared_height_is_trustworthy =
        is_single_rowbreak_table_with_trustworthy_declared_height(table, ft.effective_height, dpi);
    // HWPX 원본의 일반 1×1 텍스트 표까지 raw anchor로 보내면 뒤의 다행 표
    // fragment가 흔들린다(#1891). 그림 구조는 두 원본에서 같은 저장 계약을
    // 따르지만, 선언 높이 신뢰 특례는 native HWP5에서만 허용한다.
    let is_stored_anchor_table = is_stored_anchor_picture_table(table)
        || (page.profile.hwp5_stored_pagination_layout()
            && single_rowbreak_declared_height_is_trustworthy);
    // [#3820 Stage 11/#3925] raw anchor는 원본 종류가 아니라 저장 사다리가 표 선언
    // 높이를 실제로 비울 때만 물리 flow anchor다. 비우지 않는 native HWP5 host
    // (pi=1797)도 raw vpos를 쓰면 논리 flow가 300px 이상 부풀어 뒤 본문이 다음 쪽으로
    // 밀린다. HWPX의 같은 형상(36324768)과 동일한 저장 계약으로 묶는다.
    //
    // [#7203] 판정과 원점은 `stored_float_anchor` 가 정본이다. 종전에는 이 자리가
    // `need = 높이 + 위여백 + 아래여백`, 윗변 = raw `vpos` 였고 렌더는 각각
    // `높이 + 아래여백 − 위여백`, `vpos − 위여백` 이라 같은 표에서 두 경로가 갈렸다.
    let stored_single_topbottom_top =
        (is_topbottom_para_float && topbottom_float_count == 1 && is_stored_anchor_table)
            .then(|| stored_single_topbottom_top_px(para, next_para, table, available, dpi))
            .flatten();
    if is_topbottom_para_float && topbottom_float_count < 2 && stored_single_topbottom_top.is_none()
    {
        return None;
    }
    // 저장 LINE_SEG가 가리키는 선언 객체 높이만으로 단일 빈 앵커 표를 lane에
    // 예약하는 경로는, 셀 내용 측정치가 선언 크기와 같은 범위일 때만 안전하다.
    // 1행 1열 RowBreak 표의 실측이 선언의 1.5배를 넘으면 폰트 대체 드리프트가
    // 아니라 실제 셀 본문이 큰 경우다. 이때 선언 높이로 통째 배치하면 렌더러는
    // 확장된 행(예: 363.8px 선언 / 1163.8px 실측)을 그려 page frame 밖으로
    // 내보내므로, 아래 일반 block 경로가 cell-unit fragment를 만들도록 우회한다.
    // `single_row_object_declared_fits_current`와 같은 한도를 공유해 두 특례의
    // 판정이 엇갈리지 않게 한다.
    let stored_single_rowbreak_declared_height_is_trustworthy = stored_single_topbottom_top
        .is_none()
        || table.common.treat_as_char
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        )
        || single_rowbreak_declared_height_is_trustworthy;
    if !stored_single_rowbreak_declared_height_is_trustworthy {
        return None;
    }
    let raw_top = saved_page_top
        .or(stored_single_topbottom_top)
        .unwrap_or_else(|| (para_start_height + v_offset_px).max(para_start_height));
    // [#6795] 같은 문단의 **앞 자리차지 표가 쪽에 걸쳐 쪼개져** 이 쪽을 이미 차지한
    // 경우, 그 조각은 `PageItem::PartialTable` 로 나가고 lane 에는 등록되지 않는다.
    // `para_start_height` 는 문단이 시작한 쪽의 값이라 이어지는 쪽에서는 거의 0 이고,
    // 빈 lane 을 그대로 믿으면 두 표가 같은 앵커에 겹쳐 놓인다
    // (1341000-201100013 31쪽: 548.0 × 401.9px — 아래 표 341.9px 가 안 보인다).
    // 조각이 소비한 흐름 바닥을 raw top 으로 삼으면 lane 이 available 을 넘어
    // 아래 block 경로로 되돌아가고, 한/글처럼 표가 제 쪽을 받는다(45쪽 중 28쪽).
    //
    // [#6946] 앞 형제가 **쪼개지지 않고 통째로** block 경로에 앉은 경우도 같다. 그
    // 표는 lane 예약(`reserved_height`)이 available 을 넘어 여기서 거절됐고
    // `typeset_block_table` 이 저장 사다리 fit 으로 받았으므로 `PageItem::Table` 로
    // 나가지만 lane 에는 없다. 뒤 형제는 빈 lane 을 믿고 문단 앵커(0)에 앉아 두 표가
    // 한 쪽에 겹친다(44529 7쪽: 903.3px + 894.5px 인데 used=924.2, 한/글은 7·8쪽).
    // lane 예약의 control 소유로 배치 경로를 구별한다. 가로 교차만 확인하면 앞의
    // 작은 표 A가 남긴 lane을 block 표 B의 예약으로 오인해 뒤 표 C가 B와 겹친다.
    // 실제 lane 형제는 pushed_top이 밀어 주고, block 형제는 흐름 바닥을 소비한다.
    let raw_top = {
        let blocked_by_sibling = page.current_items.iter().any(|item| {
            let (previous_ctrl, whole) = match item {
                PageItem::PartialTable {
                    para_index,
                    control_index,
                    ..
                } if *para_index == para_idx => (*control_index, false),
                PageItem::Table {
                    para_index,
                    control_index,
                } if *para_index == para_idx && *control_index != ctrl_idx => {
                    (*control_index, true)
                }
                _ => return false,
            };
            let Some(Control::Table(previous)) = para.controls.get(previous_ctrl) else {
                return false;
            };
            let previous_width = hwpunit_to_px(signed_hwpunit(previous.common.width), dpi);
            let (previous_start, previous_end) =
                horizontal_range(&previous.common, previous_width, placement_ctx, dpi);
            let overlaps = crate::renderer::float_placement::ranges_overlap(
                x_start,
                x_end,
                previous_start,
                previous_end,
            );
            let lane_registered = whole
                && lanes
                    .lanes()
                    .iter()
                    .any(|lane| lane.control_index == Some(previous_ctrl));
            overlaps && !lane_registered
        });
        if blocked_by_sibling {
            raw_top.max(page.current_height)
        } else {
            raw_top
        }
    };
    // Square float의 native HWP LINE_SEG는 도형 선언 높이를 anchor와 함께 보존한다.
    // 셀 내용 재측정/host trailing spacing은 block flow에서만 쓰며, lane 예약에 더하면
    // p14처럼 실제로 들어가는 pair를 1~수십 px 초과로 오판한다.
    let reserved_height = if is_square_sibling_float || stored_single_topbottom_top.is_some() {
        hwpunit_to_px(table.common.height as i32, dpi).max(0.0)
    } else {
        ft.effective_height + ft.host_spacing.after_for_fit
    };
    let lane_top = lanes.pushed_top(x_start, x_end, raw_top);
    let lane_bottom = lane_top + reserved_height;

    if lane_bottom > available + 0.5 {
        return None;
    }

    Some(EmptyFloatPlacement {
        x_start,
        x_end,
        raw_top,
        reserved_height,
    })
}
