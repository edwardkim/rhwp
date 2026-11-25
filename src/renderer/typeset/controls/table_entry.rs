//! 표 컨트롤 진입 시 저장 줄 이월과 데코레이션 경로 선택 조회.
//! 페이지 전환·항목 발행·표 측정/분할은 수행하지 않는다.
use super::super::{
    para_has_non_whitespace_text, para_has_visible_text, para_is_non_tac_overlay_table_anchor,
};
use crate::model::{paragraph::Paragraph, table::Table};
use crate::renderer::{
    float_placement::{original_hwpx_infront_para_flow_paginates, signed_hwpunit},
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    page_layout::LayoutRect,
};

pub(in crate::renderer::typeset) struct TableControlPage {
    pub has_items: bool,
    pub col_count: u16,
    pub profile: crate::model::provenance::LayoutCompatibilityProfile,
}

pub(super) fn needs_stored_line_advance(
    para: &Paragraph,
    table: &Table,
    order_pos: usize,
    has_items: bool,
    dpi: f64,
) -> bool {
    // [#2287 후속/1.hwpx p58] 문단 **내부** 저장 vpos 리셋 경계:
    // 한 문단에 표 여러 개가 서로 다른 저장 줄(ls)에 앉고, TAC 표의
    // 소속 줄 ls[k](k>=1)가 쪽 리셋(vpos<=0, 직전 줄 vpos>5000 —
    // #1920 임계 동일)이면 한글은 그 표를 **다음 쪽 상단**에 둔다.
    // 기존 vpos-reset 처리는 문단 간(next_para first)만 다뤄, 이
    // 형상(1.hwpx pi=322: ls[0] 23676 자리차지 표 + ls[1] vpos=0
    // TAC 표 917px)에서 두 표가 같은 쪽 같은 y 대역에 겹쳐 렌더되고
    // (PMR-004/PM-005) 후속 조각이 페이지 밖(+865.8px)으로 밀렸다.
    // TAC 표의 소속 줄 판정은 place_table_with_text 의 표 줄 매칭
    // (lh ≈ 표높이+outer margins)과 동일식.
    if order_pos > 0 && table.common.treat_as_char && has_items {
        let tbl_line_h = hwpunit_to_px(
            table.common.height as i32
                + table.outer_margin_top as i32
                + table.outer_margin_bottom as i32,
            dpi,
        );
        let reset_line = para.line_segs.windows(2).any(|w| {
            crate::renderer::typeset::stored_line_restarts_page(&w[0], &w[1])
                && (hwpunit_to_px(w[1].line_height, dpi) - tbl_line_h).abs() < 2.0
        });
        return reset_line;
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub(super) fn uses_decoration_placement(
    para: &Paragraph,
    table: &Table,
    para_idx: usize,
    ctrl_idx: usize,
    next_para: Option<&Paragraph>,
    measured_tables: &[MeasuredTable],
    col_count: u16,
    has_tac: bool,
    host_col_w: f64,
    body: LayoutRect,
    paper_height: f64,
    dpi: f64,
    flow_height: f64,
    following: &[Paragraph],
    base_available_height: impl FnOnce() -> f64,
    original_hwpx: impl FnOnce() -> bool,
) -> bool {
    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
    // current_height 누적에 영향을 주지 않는다.
    //
    // [Issue #775] 단일 컬럼 한정. 다단(col_count>=2) 영역에서는 InFrontOfText/BehindText
    // 표라도 cur_h 누적이 컬럼 분배에 필요 (exam_eng.hwp p4 27번 보기 그림 위
    // 데코레이션 표 회귀 차단).
    //
    // [Task #992] 페이지 본문보다 큰 다행(多行) 표는 대개 데코레이션이 아니라
    // 쪽 분할이 필요한 본문 표다. 데코레이션 단축 분기에서 제외해 정상
    // 페이지네이션(format_table → typeset_block_table)을 타게 한다.
    // 제외하지 않으면 페이지보다 큰 표가 한 페이지에 통째로 그려져
    // 본문 영역을 넘는다.
    //
    // [Issue #1271] 단, HWPX paper-anchored BehindText/InFrontOfText 표는
    // rowBreak/repeatHeader 가 있어도 본문 흐름을 밀지 않는 페이지 배경/전경
    // 개체일 수 있다. 특히 cover/background 라벨 표처럼 종이 기준 절대좌표인
    // 표를 oversized_multirow 로 본문 분할하면 PDF에 없는 PartialTable 쪽이
    // 생겨 이후 바탕쪽 홀짝까지 한 쪽씩 밀린다.
    // 워터마크/배경 데코레이션(글뒤로 1×1 래퍼 등, Issue #703)은
    // 본문보다 작아 단축 분기를 그대로 탄다 — page_break/repeat_header
    // 만으로는 구분 불가(calendar_year.hwp 1×1 래퍼도 RowBreak +
    // repeat_header 비트를 가짐).
    let paper_anchored_overlay_table = !table.common.treat_as_char
        && matches!(
            table.common.vert_rel_to,
            crate::model::shape::VertRelTo::Paper
        )
        && matches!(
            table.common.horz_rel_to,
            crate::model::shape::HorzRelTo::Paper
        );
    let table_measured_h = measured_tables
        .iter()
        .find(|mt| mt.para_index == para_idx && mt.control_index == ctrl_idx)
        .map(|mt| mt.total_height)
        .unwrap_or(0.0);
    // 용지 기준 개체의 외곽 상자가 본문 밖에 있으면 자리차지 띠도
    // 본문과 만나지 않는다. 표는 그 위치에 그리고 호스트 글줄만 소비한다.
    // paint와 같은 가장자리 정렬식을 사용하며 본문 안의 고정 표는 제외한다.
    let paper_outside_body =
        paper_table_is_outside_body(table, table_measured_h, body, paper_height, dpi);
    let base_available = base_available_height();
    let oversized_multirow = table.row_count > 1
        && table_measured_h > base_available
        && !paper_anchored_overlay_table
        && !line_flow_continues_under_overlay(
            para,
            table,
            next_para,
            following,
            table_measured_h,
            flow_height,
            base_available,
            dpi,
        );
    let followed_by_empty_overlay_guide = next_para.is_some_and(|p| {
        p.controls.is_empty()
            && !para_has_visible_text(p)
            && p.line_segs.first().is_some_and(|seg| seg.vertical_pos > 0)
    });
    let multicol_empty_overlay_anchor = col_count > 1
        && !oversized_multirow
        && followed_by_empty_overlay_guide
        && para_is_non_tac_overlay_table_anchor(para);
    let multicol_tac_host_overlay_anchor = col_count > 1
        && !oversized_multirow
        && has_tac
        && para_is_non_tac_overlay_table_anchor(para);
    // [#5798] 단(그리고 용지) 밖에 **통째로** 놓인 자리차지(T&B) 표는
    // 한글이 흐름 밴드를 예약하지 않는다 — 가로로 글과 겹칠 수 없어
    // 위/아래로 밀어낼 대상이 없기 때문이다. 2401225 근무일지: 결재란
    // 표 2개(horz=단 offset 64328HU, 단 폭 ~47500HU — x=933px, 용지
    // 793px)가 각 ~101.7px 씩 본문을 밀어 +203px 하강·결재란 겹침을
    // 만들었다(한글은 두 표를 안 그리고 본문을 제자리에 둔다). 그림의
    // #959 가드(단 우측 초과 시 advance skip)와 동일 시멘틱 — 실측이
    // Left/Inside 정렬·Column/Para 기준뿐이라 그 조합에 한정한다.
    let horz_fully_outside_column = !table.common.treat_as_char
        && matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::TopAndBottom
        )
        && matches!(
            table.common.horz_rel_to,
            crate::model::shape::HorzRelTo::Column | crate::model::shape::HorzRelTo::Para
        )
        && matches!(
            table.common.horz_align,
            crate::model::shape::HorzAlign::Left | crate::model::shape::HorzAlign::Inside
        )
        && {
            let left = hwpunit_to_px(signed_hwpunit(table.common.horizontal_offset), dpi);
            let right = left + hwpunit_to_px(table.common.width as i32, dpi);
            left >= host_col_w - 0.5 || right <= 0.5
        };
    // #703: 글앞으로/글뒤로 데코레이션(비-TAC) 표만 본문 흐름에서 제외.
    // treat_as_char(글자처럼 취급) 표는 wrap 설정과 무관하게 인라인이므로
    // 흐름 높이를 예약해야 한다(한컴 의미론). #1995: 전체폭 단일셀 콜아웃
    // 박스가 글앞으로로 저장돼도 zero-height Shape 로 빠지면 후속 문단이
    // 박스 위로 겹치고 문서가 과소 페이지로 압축된다. 단일컬럼 케이스만
    // 가드하고, multicol overlay anchor 경로(자체 TAC 판정 보유)는 유지.
    // A fixed paper rectangle with THROUGH wrapping and no page splitting
    // paints at its paper coordinates. Its geometry is not a body advance;
    // the host's text line, if any, keeps its own flow ownership.
    let paper_through = paper_anchored_overlay_table
        && matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::Through
        )
        && table.layout_page_break() == crate::model::table::TablePageBreak::None;
    paper_outside_body
        || paper_through
        || (matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::InFrontOfText
                | crate::model::shape::TextWrap::BehindText
        ) && ((col_count == 1
    && !oversized_multirow
    && !table.common.treat_as_char
    // [#6366] 원본 HWPX 문단 기준 글앞으로 다행·다열
    // flowWithText 표만 데코레이션 단축(#703)에서 뺀다.
    // 모든 flowWithText 글앞으로 표에 열면 #5918 쪽수와
    // text-overlap 기준선이 깨진다.
    && !original_hwpx_infront_para_flow_paginates(
        original_hwpx(),
        table,
    )) || multicol_empty_overlay_anchor
            || multicol_tac_host_overlay_anchor))
        || horz_fully_outside_column
}

/// 쪽 하단을 넘는 문단 기준 글 앞/뒤 표라도 줄 좌표상 다음 문단이 표 띠 안에서
/// 이어지면, 한글은 표만 다음 쪽으로 나누고 본문 흐름은 밀지 않는다(뒤 문단은 표와
/// 겹친 제자리에 놓인다). 이 표는 #992 본문 분할 대신 #703 장식 배치와 #4568 잔여 행
/// 이월을 탄다. 잔여 행 이월은 한 번뿐이라 두 쪽 안에 끝나는 표로 좁힌다.
fn line_flow_continues_under_overlay(
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
    following: &[Paragraph],
    measured_height: f64,
    flow_height: f64,
    base_available: f64,
    dpi: f64,
) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    use crate::model::table::TablePageBreak;
    if table.common.treat_as_char
        || !matches!(
            table.common.text_wrap,
            TextWrap::InFrontOfText | TextWrap::BehindText
        )
        || table.common.vert_rel_to != VertRelTo::Para
        || table.layout_page_break() == TablePageBreak::None
        || !measured_height.is_finite()
        || base_available <= 0.0
    {
        return false;
    }
    // 줄 정보가 없는 원본은 불러올 때 채운 줄(합성 표시)로 같은 판정을 한다. 저장 줄과
    // 합성 줄이 섞이면 두 좌표계가 달라 비교하지 않는다.
    let (Some(anchor), Some(next)) = (
        para.line_segs.first(),
        next_para.and_then(|p| p.line_segs.first()),
    ) else {
        return false;
    };
    if (anchor.tag ^ next.tag) & 0x8000_0000 != 0 {
        return false;
    }
    let (anchor_vpos, next_vpos) = (anchor.vertical_pos, next.vertical_pos);
    let anchor_y = flow_height + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    let gap = next_vpos - anchor_vpos;
    gap > 0
        && hwpunit_to_px(gap, dpi) < measured_height
        && anchor_y >= 0.0
        && anchor_y + measured_height <= 2.0 * base_available
        && first_content_overlaps_overlay(
            anchor,
            following,
            measured_height,
            anchor_y + measured_height - base_available,
            dpi,
        )
}

/// 앵커 뒤 빈 문단(공백·개체 없음)을 건너 처음 나오는 내용 문단이 줄 좌표상 표와 겹치는가.
/// 같은 쪽이면 앵커 줄부터 표 높이 안, 한 번 쪽이 바뀐 뒤면 다음 쪽으로 넘어갈 표 꼬리 안이다.
/// 빈 문단 사다리가 표 높이와 다음 쪽 꼬리까지 덮고 그 뒤에 내용이 오면(#4514 필러 형상)
/// 표가 흐름을 밀든 말든 배치가 같으므로 종전 본문 분할 경로를 둔다.
fn first_content_overlaps_overlay(
    anchor: &crate::model::paragraph::LineSeg,
    following: &[Paragraph],
    measured_height: f64,
    tail_height: f64,
    dpi: f64,
) -> bool {
    let mut previous = anchor.vertical_pos;
    let mut page_changed = false;
    for paragraph in following {
        let (Some(first), Some(last)) = (paragraph.line_segs.first(), paragraph.line_segs.last())
        else {
            return false;
        };
        if (first.tag ^ anchor.tag) & 0x8000_0000 != 0 {
            return false;
        }
        if first.vertical_pos < previous {
            if page_changed {
                return false;
            }
            page_changed = true;
        }
        previous = last.vertical_pos;
        if paragraph.controls.is_empty() && !para_has_non_whitespace_text(paragraph) {
            continue;
        }
        return if page_changed {
            hwpunit_to_px(first.vertical_pos, dpi) < tail_height
        } else {
            first.vertical_pos > anchor.vertical_pos
                && hwpunit_to_px(first.vertical_pos - anchor.vertical_pos, dpi) < measured_height
        };
    }
    false
}

fn paper_table_is_outside_body(
    table: &Table,
    measured_height: f64,
    body: LayoutRect,
    paper_height: f64,
    dpi: f64,
) -> bool {
    use crate::model::shape::{TextWrap, VertRelTo};
    if table.common.treat_as_char
        || table.common.vert_rel_to != VertRelTo::Paper
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.caption.is_some()
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || !measured_height.is_finite()
        || !paper_height.is_finite()
        || paper_height <= 0.0
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return false;
    }
    let height = measured_height.max(hwpunit_to_px(table.common.height as i32, dpi));
    let (top, bottom) = crate::renderer::float_placement::absolute_table_vertical_geometry(
        table,
        0.0,
        paper_height,
        height,
        dpi,
    );
    let outer_top = top - hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    bottom <= body.y || outer_top >= body.y + body.height
}
