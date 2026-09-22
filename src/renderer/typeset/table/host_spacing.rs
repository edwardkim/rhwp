//! 표 호스트 문단 간격 Query. 흐름용 after와 fit용 after를 구분해 보존한다.
//! 기존 호환 조건을 이동할 뿐 새 조판 규칙이나 페이지 상태 변경을 도입하지 않는다.

use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile, table::Table};
use crate::renderer::composer::ComposedParagraph;
use crate::renderer::float_placement::{
    is_para_topbottom_float, native_empty_host_rowbreak_line_advance_hu, signed_hwpunit,
    stored_empty_anchor_band_host_line_advance_hu,
};
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::typeset::{
    para_has_non_whitespace_text, para_is_empty_tac_table_anchor,
    para_is_empty_topbottom_table_anchor, HostSpacing,
};

pub(super) struct HostSpacingInput<'a> {
    pub(super) para: &'a Paragraph,
    pub(super) ctrl_idx: usize,
    pub(super) table: &'a Table,
    pub(super) styles: &'a ResolvedStyleSet,
    pub(super) composed: Option<&'a ComposedParagraph>,
    pub(super) next_para: Option<&'a Paragraph>,
    pub(super) is_column_top: bool,
    pub(super) is_tac: bool,
}

pub(super) struct HostSpacingResult {
    pub(super) host_spacing: HostSpacing,
    pub(super) strict_following_plain_text_fit: bool,
}

pub(super) fn resolve(
    input: HostSpacingInput<'_>,
    dpi: f64,
    profile: impl Fn() -> LayoutCompatibilityProfile,
) -> HostSpacingResult {
    let HostSpacingInput {
        para,
        ctrl_idx,
        table,
        styles,
        composed,
        next_para,
        is_column_top,
        is_tac,
    } = input;
    // [#1880] 자리차지(TopAndBottom) 판정: 종전 원시 attr 비트((attr>>21)&7==1)는
    // HWPX 파스가 table.attr 를 미채움(bit0 만 미러, section.rs:1831)이라 항상
    // false, HWP5 재파스는 원시 attr 전체(control.rs:153)라 true — 같은 IR 의
    // convert-HWP(is_hwpx_variant) 재파스에서 host_spacing.before 가 sb 를 잃어
    // defer 가드가 플립됐다(2780073 pi=2/4/6 host_before 6.7↔0.0px, 3075729
    // oracle p13→p12). 의미 필드 + 소스 게이트로 교체: native 는 비트⇔열거형
    // 전단사(shape.rs:394)로 불변, 순수 HWPX 는 종전에도 미진입으로 불변,
    // convert-HWP 만 HWPX 렌더 경로로 정합된다(#1886 origin 전달의 연장).
    let table_wrap_take_place = !profile().hwpx_stored_layout()
        && matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::TopAndBottom
        );

    // host_spacing 계산 — layout과 동일한 규칙
    let para_style_id = composed
        .map(|c| c.para_style_id as usize)
        .unwrap_or(para.para_shape_id as usize);
    let para_style = styles.para_styles.get(para_style_id);
    let sb = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
    let sa = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

    // [#2195 stage50 실험] 자리차지(TopAndBottom) 표도 outer_margin_top 계상 —
    // 86712 구분선 표(566HU) 한글 PDF 괘선 실측: 상단 마진 7.55px 포함.
    let outer_top = if is_tac || is_para_topbottom_float(&table.common) {
        hwpunit_to_px(table.outer_margin_top as i32, dpi)
    } else {
        0.0
    };
    // [Task #1841] visible-host 자리차지(TopAndBottom) 표는 outer_margin_bottom 을
    // 후속 재개 간격에 포함한다 (layout 재개 y 가산과 대칭 — 렌더/pagination 정합).
    // 한글 실측: 표 하단→첫 줄 gap 한글 18.7pt = rhwp 10.2pt + outer_bottom 8.5pt
    // (결재문서 헤더 표 852HU 계열, 동작소방서 36385142/관악소방서 36389312).
    // 전면(모든 비-TAC) 적용은 한컴 핀 테스트(1086/1156/1692 쪽수·분할)와 충돌 —
    // 렌더 좌표가 실제로 바뀌는 visible-host float 형상으로 한정한다.
    let is_visible_host_float =
        is_para_topbottom_float(&table.common) && para_has_non_whitespace_text(para);
    // [#2195 stage58] 빈 호스트 자리차지 표의 bottom margin 은 **흐름 전용** —
    // 다음 항목 간격에는 계상(86712 구분선 7.55px, 한글 괘선 실측)하되 표 자체의
    // 페이지 적합 판정에서는 제외(trailing 간격 면제 — hwpspec 178쪽 핀 #1086).
    let is_empty_host_float =
        is_para_topbottom_float(&table.common) && !para_has_non_whitespace_text(para);
    let outer_bottom = if is_tac || is_visible_host_float {
        hwpunit_to_px(table.outer_margin_bottom as i32, dpi)
    } else if is_empty_host_float {
        hwpunit_to_px(table.outer_margin_bottom as i32, dpi)
    } else {
        0.0
    };
    // #2439: native HWP can encode a single positive-offset empty-host RowBreak table as
    // `painted table + stored empty host line advance`, followed by an ordinary signature.
    // The generic empty-anchor rule intentionally suppresses the host line spacing
    // (#1147/#1836). This narrower native signature proves that the full stored line
    // advance plus the positive visual offset is part of the flow boundary. Keep the gate
    // narrow: the broad `v_off + margin` experiment is disproven by #2097.
    let positive_empty_host_rowbreak_tail = native_empty_host_rowbreak_line_advance_hu(
        profile().hwp5_stored_pagination_layout(),
        para,
        table,
        next_para,
    )
    .map(|line_advance| {
        hwpunit_to_px(line_advance, dpi)
            + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset).max(0), dpi)
    })
    .unwrap_or(0.0);
    let strict_following_plain_text_fit = positive_empty_host_rowbreak_tail > 0.0;
    // [#2195 stage59] 빈 호스트 자리차지 표의 bottom margin fit 계상 판별:
    // 호스트가 **저장 lineseg 보유**(한컴 기계산 문서, hwpspec #1086 178쪽 핀)면
    // 저장 기하 신뢰 - fit 제외(흐름 전용). **NO_LS**(생성계, 86712 표182)는
    // 재계산 - fit 포함. #2195 의 저장 보존 vs NO_LS 재계산 원칙과 동일 축.
    let outer_bottom_flow_only = if !is_tac
        && is_empty_host_float
        && !para.line_segs.is_empty()
        && positive_empty_host_rowbreak_tail <= 0.0
    {
        outer_bottom
    } else {
        0.0
    };

    // 비-TAC 표: 호스트 문단의 trailing line_spacing도 포함
    // [Task #874 #7] 비-TAC 1×1 placeholder 표 (paras=1 text-only) 는 host
    // line_spacing 을 더하지 않는다. 한컴은 표 outer_margin_bottom 만 사용 (호스트
    // 문단 line_spacing 은 본문 라인 간 간격 의미). aift.hwp p21 표 pi=268
    // ("협업 시스템 구성도 이미지") 직후 pi=284 ("코멘트 스레드 관리...") 가
    // 9.6 px 만큼 다음 페이지로 밀려나는 문제 해결.
    let is_single_cell_placeholder = !is_tac
        && table.row_count == 1
        && table.col_count == 1
        && table.cells.len() == 1
        && table
            .cells
            .first()
            .map(|c| {
                c.paragraphs
                    .iter()
                    .all(|p| p.controls.is_empty() && p.line_segs.len() <= 1)
            })
            .unwrap_or(false);
    // [Task #1147] 빈 앵커 wrap=TopAndBottom 비-TAC 표 + 다음이 일반 문단:
    //   host_line_spacing 을 0 으로 억제한다 (빈 앵커 vpos 가 이미 갭을 인코딩해
    //   별도 가산 시 page overflow).
    // [Task #1133] 단, 다음도 빈 앵커 TopAndBottom 표이면 host_line_spacing 이
    //   표-표 사이 간격이므로 보존.
    // [Task #1836] 종전 `is_hwpx_source` 게이트 제거 — 이 억제는 렌더 경로
    //   (layout.rs suppress_empty_anchor_spacing = is_current_empty_para_float,
    //   소스 무관)와 대칭이어야 한다. HWPX 만 억제하면 typeset pagination(HWP5
    //   재파스는 미억제, host_sp +12px phantom)이 layout 렌더(소스 무관 억제,
    //   가시 위치 동일)와 어긋나 라운드트립 쪽나눔이 뒤집힌다 (seoul_0776 p2→p3
    //   1줄 이월; #1763 clamp 가 누적을 razor-thin 경계로 옮겨 노출). #1809/#1841
    //   과 동일 소스 무관화. #1147 원 케이스(HWPX)는 억제 유지되어 불변.
    // [Task #1863] 소스 무관화가 native HWP5 표-표 스택을 깨뜨리는 케이스 보정:
    //   다음 문단이 빈 TAC-표 앵커여도 #1133 과 같은 표 스택이므로 보존한다
    //   (rowbreak-problem-pages.hwp sec1 pi=2→pi=3, 1200HU 가 한컴 페이지 채움에
    //   실제 계상되는 간격 — 억제 시 p12 PartialTable 42.5px overflow).
    let is_topbottom_empty_anchor = !is_tac
        && matches!(
            table.common.text_wrap,
            crate::model::shape::TextWrap::TopAndBottom
        )
        && para.text.is_empty();
    let next_is_empty_table_anchor = next_para
        .map(|p| para_is_empty_topbottom_table_anchor(p) || para_is_empty_tac_table_anchor(p))
        .unwrap_or(false);
    let suppress_empty_anchor_spacing = is_topbottom_empty_anchor && !next_is_empty_table_anchor;

    let host_line_spacing = if suppress_empty_anchor_spacing {
        0.0
    } else if !is_tac && !is_single_cell_placeholder {
        para.line_segs
            .last()
            .filter(|seg| seg.line_spacing > 0)
            .map(|seg| hwpunit_to_px(seg.line_spacing, dpi))
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // spacing_before 조건부 적용
    // - 자리차지(text_wrap=1) 비-TAC 표: spacing_before 제외
    //   (layout에서 v_offset 기반 절대 위치로 배치)
    // - 단 상단: spacing_before 제외
    // - [Task #1147] HWPX 빈 앵커 TopAndBottom 비-TAC 표: 다음 항목이 일반 문단이면
    //   spacing_before 제외 (위 주석). 다음 항목도 표 앵커이면 HWP처럼 보존한다.
    // - [#1880] 빈 앵커 스택(다음도 표 앵커)은 자리차지 제외 분기에서도 sb 를
    //   보존한다 — 아래 #1863 보존 규칙과 동일 근거(스택 사이 간격은 한컴이
    //   실제 계상, 한글 2022 오라클 3075729 p13 = sb 보존). 판정은 #1927 에서
    //   의미 필드+소스 게이트(table_wrap_take_place)로 교체되었고, 본 예외는
    //   그 판정이 발동하는 경로(HWP5-native 파스)에 적용된다 (메인테이너 통합:
    //   PR #1927 × #1928 동일 라인 충돌 해소).
    let before = if !is_tac
        && table_wrap_take_place
        && !(is_topbottom_empty_anchor && next_is_empty_table_anchor)
    {
        outer_top
    } else if suppress_empty_anchor_spacing && !is_column_top {
        outer_top
    } else {
        (if !is_column_top { sb } else { 0.0 }) + outer_top
    };
    // [#6147] layout `stored_empty_anchor_band_host_tail_px` 와 대칭 — 저장 사다리가
    // host 줄 advance 만 증언하는 빈 앵커 밴드는 그 줄을 흐름에 계상한다. `outer_bottom`
    // 은 위에서 이미 더해지므로 여기서는 줄 advance 만 얹는다.
    let stored_empty_anchor_host_line_tail = if positive_empty_host_rowbreak_tail > 0.0 {
        0.0
    } else {
        stored_empty_anchor_band_host_line_advance_hu(
            profile().hwp5_stored_pagination_layout() || profile().hwpx_stored_layout(),
            para,
            ctrl_idx,
            next_para,
        )
        .map(|line_advance| hwpunit_to_px(line_advance, dpi))
        .unwrap_or(0.0)
    };
    let after = sa
        + outer_bottom
        + host_line_spacing
        + positive_empty_host_rowbreak_tail
        + stored_empty_anchor_host_line_tail;
    let host_spacing = HostSpacing {
        before,
        after,
        spacing_after_only: sa,
        after_for_fit: after - outer_bottom_flow_only,
    };

    HostSpacingResult {
        host_spacing,
        strict_following_plain_text_fit,
    }
}
