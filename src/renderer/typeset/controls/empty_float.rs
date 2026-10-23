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
    pub visible_float_exclusions: &'a [super::super::VisibleFloatExclusion],
    pub original_column_origin: bool,
}

pub(in crate::renderer::typeset) struct EmptyFloatPlacement {
    pub x_start: f64,
    pub x_end: f64,
    pub raw_top: f64,
    pub reserved_height: f64,
    pub resolved: Option<crate::renderer::float_placement::ParagraphFloatPlacement>,
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
    prev_para: Option<&Paragraph>,
    next_para: Option<&Paragraph>,
    styles: &ResolvedStyleSet,
    para_start_height: f64,
    lanes: &FloatLaneSet,
    page: EmptyFloatPage<'_>,
    table_reflowed: bool,
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
    let (x_start, mut x_end) = horizontal_range(&table.common, width_px, placement_ctx, dpi);
    let square_outer_frame = !page.profile.session_edited()
        && (page.profile.native_hwp5_layout() || page.profile.hwpx_stored_layout())
        && page.layout.column_areas.len() == 1
        && !table_reflowed
        && crate::renderer::float_placement::stored_square_sibling_outer_frame(para);
    if square_outer_frame {
        // lane은 바깥 상자, 실제 표 원점은 그 안의 왼쪽 여백을 소비한다.
        x_end += hwpunit_to_px(
            i32::from(table.outer_margin_left) + i32::from(table.outer_margin_right),
            dpi,
        );
    }

    let available = available_height();

    // 앞뒤 저장 줄이 새 쪽 원점을 증명하면 닫힌 사다리 상자도 그 원점에서
    // 생산한다. 현재 커서에서 먼저 만든 상자가 복구된 프레임보다 우선하면
    // 앞 표의 paint 여백만 바뀌어도 후속 표의 저장 흐름이 함께 이동한다.
    let rewound_host_origin =
        crate::renderer::float_placement::native_empty_topbottom_rewind_anchor_saved_para_y(
            page.profile.hwp5_stored_pagination_layout(),
            prev_para,
            para,
            next_para,
            table,
            column_area.y + para_start_height,
            &column_area,
            dpi,
        )
        .map(|origin| origin - column_area.y);

    // lane 경로도 block·중첩 표와 같은 로컬 저장 상자를 소비한다.
    // 양수 오프셋은 앞 줄의 높이와 별개로 이 상자에 포함된다.
    let closed_source_placement = (is_topbottom_para_float
        && !page.profile.session_edited()
        && (page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        && !table_reflowed)
        .then(|| {
            crate::renderer::float_placement::ParagraphFloatPlacement::from_empty_stored_ladder_host(
                para,
                next_para?,
                table,
                rewound_host_origin.unwrap_or(para_start_height),
                para_style.map_or(0.0, |style| style.spacing_before),
                ft.effective_height,
                dpi,
            )
        })
        .flatten();
    if std::env::var_os("RHWP_DIAG_FLOAT_FRAME").is_some() {
        eprintln!(
            "DIAG_FLOAT_FRAME pi={para_idx} ci={ctrl_idx} origin={para_start_height} height={} reflowed={table_reflowed} closed={closed_source_placement:?}",
            ft.effective_height,
        );
    }

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
    let stored_outer_box = !page.profile.session_edited()
        && (page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        && !table_reflowed
        && crate::renderer::stored_float_anchor::stored_empty_topbottom_outer_box_is_valid(
            para,
            next_para,
            table,
            Some(ft.effective_height),
            dpi,
        );
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
    let legacy_stored_top =
        (is_topbottom_para_float && topbottom_float_count == 1 && is_stored_anchor_table)
            .then(|| {
                stored_single_topbottom_top_px(
                    para,
                    next_para,
                    table,
                    available,
                    dpi,
                    page.profile.hwpx_stored_layout(),
                )
            })
            .flatten();
    // 다음 저장 위치와의 차이는 높이 증거이며 절대 쪽 원점의 증거가 아니다.
    // 새로 수용하는 바깥 상자는 현재 문단의 흐름 원점에서 배치한다.
    let mut restored_stored_outer_box = false;
    let outer_box_flow_top = stored_outer_box.then(|| {
        let (top_offset, _) = crate::renderer::stored_float_anchor::stored_topbottom_object_span(
            para,
            next_para,
            table,
            page.profile.hwpx_stored_layout(),
        );
        let flow_top = para_start_height + hwpunit_to_px(top_offset as i32, dpi);
        // 전체 상자 뒤의 저장 줄이 높이를 증명해도 현재 커서는 앞 float의
        // 점유 영역 안에 남을 수 있다. 저장 host가 앞 밴드 끝을 벗어나는 경우일
        // 때만 원점을 복구하고, 예약과 출력에 같은 확정 상자를 전달한다.
        let saved_top = (page.layout.column_areas.len() == 1)
            .then(|| {
                stored_single_topbottom_top_px(
                    para,
                    next_para,
                    table,
                    available,
                    dpi,
                    page.profile.hwpx_stored_layout(),
                )
            })
            .flatten();
        saved_top
            .filter(|top| {
                let host = *top - hwpunit_to_px(top_offset as i32, dpi);
                page.visible_float_exclusions.iter().any(|zone| {
                    zone.para_index < para_idx
                        && flow_top < zone.bottom
                        && flow_top + ft.effective_height > zone.top
                        && host + 0.5 >= zone.bottom
                })
            })
            .inspect(|_| restored_stored_outer_box = true)
            .unwrap_or(flow_top)
    });
    let stored_single_topbottom_top = legacy_stored_top.or(outer_box_flow_top);
    // 원본 후속 줄이 본체·캡션·양쪽 여백의 바깥 상자를 정확히 닫으면 그 원점·끝을
    // lane 예약과 실제 출력에 공유한다. 일반 캡션 측정 원점은 증거가 없을 때 쓴다.
    // 직전 문단의 저장 위치보다 되감긴 호스트는 새 쪽의 저장 원점을 가진다.
    // 닫힌 바깥 상자가 확인된 경우에만 같은 원점을 예약과 출력에 전달한다.
    let original_frame_follows_body = page.original_column_origin
        && para.source_line_seg_vertical_pos.is_none()
        && para.column_type == crate::model::paragraph::ColumnBreakType::None
        // 앞뒤 줄이 서로 다른 쪽에 속하면 닫힌 상자는 새 쪽 원점이다.
        // 이전 본문 뒤의 호스트라는 사실만으로 현재 쪽에 수용할 수 없다.
        && para.line_segs.first().is_some_and(|host| {
            next_para.is_some_and(|next| {
                next.source_line_seg_vertical_pos.is_none()
                    && next.line_segs.first().is_some_and(|after| {
                        after.vertical_pos > host.vertical_pos
                    })
            })
        })
        && prev_para.is_some_and(|previous| {
            previous.controls.is_empty()
                && !previous.stored_text_partition_is_dirty()
                && previous.source_line_seg_vertical_pos.is_none()
                && previous
                    .line_segs
                    .last()
                    .zip(para.line_segs.first())
                    .is_some_and(|(before, host)| {
                        !is_synthetic_line_seg(before)
                            && !is_synthetic_line_seg(host)
                            && i64::from(before.vertical_pos) + i64::from(before.line_height)
                                <= i64::from(host.vertical_pos)
                    })
        });
    let closed_control_frame = (!page.profile.session_edited()
        && (page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        && page.layout.column_areas.len() == 1
        && !table_reflowed)
        .then(|| {
            // 스냅샷 부재만으로 현재 저장 좌표가 이 쪽의 물리 원점이라고
            // 판단할 수 없다. 앞 본문과 후속 줄이 같은 원본 축에서 닫히는
            // 경우에만 block 경로의 원본 프레임을 사용한다. 그 외에는
            // 원본 호스트 기준 상자를 현재 흐름에 평행 이동한다.
            // 선행 밴드와 뒤 저장 줄이 복구한 원본 상자를 증명한 경우는
            // 이미 쪽 원점이 확정됐다. 이를 이전 호스트 흐름에 다시 옮기면
            // 표 뒤의 저장 줄·앞 간격도 원본 프레임과 갈라진다.
            let uses_host_flow_origin = table.caption.is_none()
                && para.column_type == crate::model::paragraph::ColumnBreakType::None
                && !original_frame_follows_body
                && !restored_stored_outer_box;
            let frame_vpos = if uses_host_flow_origin {
                para.source_line_seg_vertical_pos
                    .as_ref()
                    .and_then(|source| source.first().copied())
                    .or_else(|| para.line_segs.first().map(|line| line.vertical_pos))?
            } else {
                0
            };
            let mut frame = crate::renderer::float_placement::stored_empty_control_table_frame(
                para,
                next_para?,
                table,
                ft.effective_height,
                para_style.map_or(0.0, |style| style.spacing_before),
                frame_vpos,
                dpi,
                page.profile.hwpx_stored_layout(),
            )?;
            if uses_host_flow_origin {
                let host_origin = rewound_host_origin.unwrap_or(para_start_height);
                frame.anchor_y += host_origin;
                frame.table_top += host_origin;
                frame.occupied_bottom += host_origin;
                frame.stored_successor_line_origin =
                    frame.stored_successor_line_origin.map(|y| y + host_origin);
            }
            Some(frame)
        })
        .flatten();
    if !original_frame_follows_body
        && closed_control_frame.is_some_and(|frame| frame.table_top + 0.5 < page.current_height)
    {
        // 새 쪽의 닫힌 상자를 현재 쪽 위에 겹쳐 놓지 않는다. block의 기존
        // 저장 프레임 이월·통째 fit 경로가 같은 상자로 쪽 전이를 확정한다.
        return None;
    }
    let control_frame = closed_control_frame.or_else(|| {
        crate::renderer::float_placement::captioned_para_table_frame(
            table,
            (para_start_height + v_offset_px).max(para_start_height),
            ft.host_spacing.before,
            ft.effective_height,
            ft.host_spacing.after,
        )
    });
    if is_topbottom_para_float
        && topbottom_float_count < 2
        && stored_single_topbottom_top.is_none()
        && control_frame.is_none()
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
    let saved_page_top = saved_page_top.map(|top| {
        if square_outer_frame && signed_hwpunit(table.common.vertical_offset) > 0 {
            top + v_offset_px
        } else {
            top
        }
    });
    // 닫힌 후속 줄은 프레임의 높이를 증명하며 현재 흐름을 절대 쪽 원점으로
    // 바꾸지는 않는다. 이미 수용한 저장 앵커를 먼저 선택하고, 아래 lane의
    // 확정 이동을 같은 placement의 원점·점유 끝·후속 줄에 함께 반영한다.
    let raw_top = saved_page_top
        .or_else(|| {
            // 닫힌 상자는 원점 앞의 호스트 간격까지 이미 분리했다.
            // raw 호스트+오프셋으로 다시 시작하면 같은 간격이 lane의
            // 예약 끝과 paint에 재가산된다. 별도 ladder 상자는 기존
            // 자신의 원점 계약을 유지한다.
            closed_control_frame
                .filter(|_| closed_source_placement.is_none())
                .map(|frame| frame.anchor_y)
        })
        .or(legacy_stored_top)
        .or_else(|| closed_source_placement.map(|placement| placement.table_top))
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
    let reserved_height = if let Some(frame) = control_frame {
        frame.occupied_bottom - frame.anchor_y
    } else if is_square_sibling_float || stored_single_topbottom_top.is_some() {
        hwpunit_to_px(table.common.height as i32, dpi).max(0.0)
    } else {
        ft.effective_height + ft.host_spacing.after_for_fit
    };
    let lane_top = lanes.pushed_top(x_start, x_end, raw_top);
    if std::env::var_os("RHWP_DIAG_FLOAT_FRAME").is_some() {
        eprintln!(
            "DIAG_FLOAT_LANE pi={para_idx} ci={ctrl_idx} raw={raw_top} lane={lane_top} closed={closed_control_frame:?} before={} after={} reserved={reserved_height}",
            ft.host_spacing.before, ft.host_spacing.after,
        );
    }
    let closed_source_resolved = closed_source_placement.map(|mut placement| {
        let shift = lane_top - placement.table_top;
        placement.anchor_y += shift;
        placement.table_top = lane_top;
        placement.occupied_bottom += shift;
        placement.stored_successor_line_origin = placement
            .stored_successor_line_origin
            .map(|origin| origin + shift);
        placement.flow = crate::renderer::float_placement::ParagraphFloatFlow::NextLine;
        placement
    });
    // HWPX의 빈 호스트 형제 표는 각 개체의 바깥 상자를 차례로 점유한다.
    // 위여백을 누락하고 아래여백을 fit 면제와 함께 버리면 뒤 표가 누적해서
    // 올라간다. 마지막 아래여백의 적합성 면제와 형제의 실제 예약을 구분한다.
    let square_resolved = (square_outer_frame && saved_page_top.is_some()).then(|| {
        let top = lane_top + hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
        crate::renderer::float_placement::ParagraphFloatPlacement {
            flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
            anchor_y: saved_page_top.expect("유효 저장 호스트 프레임"),
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: Some(
                x_start - column_area.x + hwpunit_to_px(i32::from(table.outer_margin_left), dpi),
            ),
            table_top: top,
            occupied_bottom: top
                + ft.effective_height
                + hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi),
        }
    });
    // 저장 상자는 원점·예약 끝을 한 결과로 전달한다. 뒤의 host 줄간격은
    // 이미 증명된 전체 여백 상자 높이에 다시 더하지 않는다.
    let outer_box_resolved =
        (stored_outer_box && stored_single_topbottom_top.is_some()).then(|| {
            let (top_offset, occupied_height) =
                crate::renderer::stored_float_anchor::stored_topbottom_object_span(
                    para,
                    next_para,
                    table,
                    page.profile.hwpx_stored_layout(),
                );
            let anchor_y = lane_top - hwpunit_to_px(top_offset as i32, dpi);
            crate::renderer::float_placement::ParagraphFloatPlacement {
                flow: crate::renderer::float_placement::ParagraphFloatFlow::NextLine,
                anchor_y,
                stored_host_origin: None,
                // The source ladder certifies the complete outer-box height
                // regardless of whether its absolute anchor needed recovery.
                // Project the successor from the accepted anchor; both flow
                // finalization and paint consume this same closed frame.
                stored_successor_line_origin: Some(
                    anchor_y + hwpunit_to_px(occupied_height as i32, dpi),
                ),
                table_left: None,
                table_top: lane_top,
                occupied_bottom: anchor_y + hwpunit_to_px(occupied_height as i32, dpi),
            }
        });
    let control_resolved = control_frame.map(|mut frame| {
        let lane_shift = lane_top - frame.anchor_y;
        frame.anchor_y = if closed_control_frame.is_some() && original_frame_follows_body {
            frame.anchor_y + lane_shift
        } else if closed_control_frame.is_some() {
            rewound_host_origin.unwrap_or(para_start_height)
        } else {
            para_start_height
        };
        frame.table_top += lane_shift;
        frame.occupied_bottom += lane_shift;
        // lane 이동은 원본 후속 줄의 앞 간격까지 소비했다는 증거가 아니다.
        if lane_shift.abs() > frame.anchor_y.abs().max(1.0) * 1e-9 {
            frame.stored_successor_line_origin = None;
        }
        frame
    });
    let resolved = closed_source_resolved
        .or(control_resolved)
        .or(square_resolved)
        .or(outer_box_resolved)
        .or_else(|| {
            crate::renderer::float_placement::ParagraphFloatPlacement::from_empty_sibling_box(
                para,
                table,
                lane_top,
                para_start_height,
                ft.effective_height,
                ft.host_spacing.before,
                ft.host_spacing.after,
                dpi,
            )
        });
    // 다른 문단의 이미 수용한 밴드도 같은 확정 상자가 통과해야 한다.
    // paint만 다시 피한 뒤 확정 원점으로 덮으면 예약과 출력이 갈라진다.
    let resolved = resolved.map(|placement| {
        if !is_topbottom_para_float || table.common.allow_overlap {
            return placement;
        }
        let outer_top = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
        let mut cleared = placement.clear_occupied_bands(
            page.visible_float_exclusions
                .iter()
                .filter(|zone| zone.para_index < para_idx)
                .map(|zone| zone.top..zone.bottom + outer_top),
        );
        if (cleared.table_top - placement.table_top).abs()
            > placement.table_top.abs().max(1.0) * 1e-9
        {
            cleared.stored_successor_line_origin = None;
        }
        cleared
    });
    let lane_bottom = resolved.map_or(lane_top + reserved_height, |p| {
        if square_resolved.is_some()
            || closed_source_resolved.is_some()
            || control_resolved.is_some()
            || outer_box_resolved.is_some()
        {
            p.occupied_bottom
        } else {
            p.table_top + ft.effective_height + ft.host_spacing.after_for_fit
        }
    });

    if lane_bottom > available + 0.5 {
        return None;
    }

    Some(EmptyFloatPlacement {
        x_start,
        x_end,
        raw_top,
        reserved_height: resolved.map_or(reserved_height, |p| p.occupied_bottom - lane_top),
        resolved,
    })
}
