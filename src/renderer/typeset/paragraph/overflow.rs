//! 일반 fit 실패 뒤의 기존 넘침 허용 판정. 상태 변경과 높이 반영은 하지 않는다.

use super::super::{
    paragraph_forces_page_boundary_after, stored_vpos_overflow_defers_paragraph,
    stored_zero_vpos_after_near_full_line,
};
use super::metrics::FormattedParagraph;
use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
};
use crate::renderer::hwpunit_to_px;

/// 두 판정 사이 상태 변경이 없을 때만 공유하는 값 관측. 각주 차감 가용 높이는 담지 않는다.
pub(in crate::renderer::typeset) struct OverflowPage {
    pub col_count: u16,
    pub current_height: f64,
    pub has_items: bool,
    pub body_height: f64,
    pub body_area_height: f64,
    pub hwp3_layout: bool,
}

pub(in crate::renderer::typeset) fn atomic_overflow_fits(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    page: &OverflowPage,
    available: f64,
    dpi: f64,
) -> bool {
    // [Task #409 v3] atomic TAC top-fit:
    // 단일 라인 + TAC Picture/Shape (분할 불가능) 항목은 시작점이 본문 안이면
    // 현재 페이지에 배치하고 하단 일부는 하단 여백 (15mm) 으로 흘림 허용.
    // HWP 시멘틱 — atomic 항목은 strict bottom-fit 대신 top-fit 으로 판정.
    // (대상 샘플 23페이지 차트 pi=208: lh=316px, 시작 y=721.4 < 1028(본문 끝),
    //  끝 y=1037.4 가 9.4px 초과하지만 하단 여백 56.7px 안이므로 HWP 가 23페이지 배치.)
    // [Task #1027 Stage E2] atomic top-fit 스필은 진짜 인라인 atomic 개체(차트/그림 등,
    // #409)에만 적용한다. 위아래(TopAndBottom) 글상자(Shape)는 한컴이 본문 항목처럼
    // 다음 페이지로 넘기므로(예: AI 184p box pi=142 → 10쪽) 스필 대상에서 제외 —
    // 그렇지 않으면 하드코딩 60px 허용폭으로 페이지 하단에 잘못 스필되어 overflow.
    let is_atomic_tac_singleton = fmt.line_heights.len() == 1
        && para.controls.iter().any(|c| match c {
            Control::Picture(p) => p.common.treat_as_char,
            Control::Shape(s) => {
                s.common().treat_as_char
                    && !matches!(
                        s.common().text_wrap,
                        crate::model::shape::TextWrap::TopAndBottom
                    )
            }
            _ => false,
        });
    let stored_next_page_atomic = page.col_count == 1
        && stored_zero_vpos_after_near_full_line(
            paragraphs,
            para_idx,
            crate::renderer::px_to_hwpunit(page.body_height, dpi),
        );
    if is_atomic_tac_singleton
        && !stored_next_page_atomic
        && page.current_height < available
        && page.has_items
    {
        // 추가 가드: 본문 + 하단 여백 안에 들어가야 함 (footer 침범 금지)
        let bottom_margin_px = hwpunit_to_px(
            page.body_area_height as i32, // body_area.height 는 이미 px
            dpi,
        );
        // 보수적 tolerance: 1mm (약 3.78px) 이상 ~ 하단 여백 끝까지 허용
        // body_area.height 가 px 이므로 직접 비교 — base_available_height 와의
        // 차이는 footnote_area 만 (본 케이스 0). bottom_margin 은 PageDef 에서
        // 가져와야 하나 직접 접근 어려우므로 1mm 이상 ~ 60px 정도까지 허용.
        let _ = bottom_margin_px; // (위 변수는 향후 정밀화용 — 현재 사용 안 함)
        let overflow = page.current_height + fmt.height_for_fit - available;
        // 60px 이내 초과 (대략 하단 여백 1.6cm 까지 허용; HWP 표준 15mm 여백 안)
        if overflow <= 60.0 {
            return true;
        }
    }

    false
}

pub(in crate::renderer::typeset) fn tail_overflow_candidate(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    page: &OverflowPage,
    forced_page_break_line: Option<usize>,
    dpi: f64,
) -> bool {
    // [Task #1537] 폰트 치환 drift 로 인한 "tail 1줄 spill 후 강제 쪽나누기 고아 페이지" 차단.
    //
    // 증상: 본문 문단 N 이 페이지 하단을 ~한 줄 미만으로 미세 초과(폰트 치환으로 부피가
    // 한컴 대비 커짐)하여 마지막 줄만 새 페이지로 split → 그 직후 문단 N+1 이 명시적
    // 쪽나누기(column_type==Page/Section)를 가지면 또 새 페이지를 강제 → spill 한 1줄이
    // 거의 빈 페이지에 고립된다(2025 행정업무운영 편람: 0-idx page 11/13/17, 본문 1줄+빈공간).
    //
    // 한컴은 폰트 drift 가 없어 문단 N 전체를 현재 페이지에 담고 N+1 의 쪽나누기로 깔끔히
    // 다음 페이지를 시작한다. 우리도 "초과량이 한 줄 미만(=drift)이고 다음 문단이 어차피
    // 쪽나누기로 페이지를 끝낸다"는 두 조건이 모두 맞을 때만 문단 N 을 통째로 현재 페이지에
    // 배치(하단 여백으로 소량 bleed 허용)해 고아 페이지를 제거한다. 일반 본문 흐름(다음
    // 문단이 쪽나누기가 아님)이나 초과량이 한 줄 이상(진짜 split 필요)인 경우는 불변.

    // 다음 문단이 쪽/구역 나누기인가? 사이에 빈 문단(텍스트·컨트롤 없음)이 끼어 있으면
    // 건너뛴다 — 빈 문단은 높이를 거의 차지하지 않고 hide_empty_line 로 흡수되므로,
    // "tail spill → 빈 문단 → 강제 쪽나누기" 패턴에서도 spill 한 줄이 동일하게 고립된다.
    // 단, 텍스트/컨트롤이 있는 일반 문단을 만나면 즉시 중단(false) — 그 문단이
    // 현재 페이지를 마저 채우므로 고아 페이지가 생기지 않는다.
    let next_para_forces_break = {
        let mut idx = para_idx + 1;
        let mut prior_para = para;
        let mut forced = false;
        while let Some(next_para) = paragraphs.get(idx) {
            if paragraph_forces_page_boundary_after(
                prior_para,
                next_para,
                page.col_count,
                page.hwp3_layout,
            ) {
                forced = true;
                break;
            }
            let is_empty = next_para.text.trim().is_empty() && next_para.controls.is_empty();
            if !is_empty {
                break;
            }
            prior_para = next_para;
            idx += 1;
        }
        forced
    };
    // [#6854] 같은 걸음을 **문서가 스스로 선언한 쪽나누기**로만 다시 판정한다.
    // `paragraph_forces_page_boundary_after` 는 저장 사다리에서 **추론한** 경계도
    // 참으로 보는데, 그 추론이 맞아도 흐름이 실제로 거기서 끊기지는 않는 문서가
    // 있다 — 그런 곳에서 아래 완화를 걸면 고아 쪽은 그대로 두고 넘침만 하나 는다
    // (코퍼스 실측 7건). 선언된 `column_type` 은 그런 어긋남이 없다.
    let next_para_declares_page_break = {
        let mut idx = para_idx + 1;
        let mut declared = false;
        while let Some(next_para) = paragraphs.get(idx) {
            if matches!(
                next_para.column_type,
                ColumnBreakType::Page | ColumnBreakType::Section
            ) {
                declared = true;
                break;
            }
            let is_empty = next_para.text.trim().is_empty() && next_para.controls.is_empty();
            if !is_empty {
                break;
            }
            idx += 1;
        }
        declared
    };
    // [#7288] 다음 문단의 **저장 vpos 가 본문 높이를 넘으면** 그 문단은 이 쪽에 있을 수
    // 없다고 **문서가 적은** 것이다 — `#6132` 가 쓰는 바로 그 사실이고, 사다리 패턴에서
    // 추론한 경계가 아니다. 위 경고("추론 경계까지 받으면 쪽 이득 없이 넘침만 는다")는
    // 추론에 대한 것이라 여기에는 해당하지 않는다.
    //
    // 156482639: 표 조각 뒤 빈 문단 `pi=101` 이 잔여를 조금 넘치는데, 다음 `pi=102`
    // ('참고3' 표)의 저장 vpos 73760 = 983.5px 가 본문 977.8px 를 넘어 어차피 새 쪽에서
    // 시작한다. 그때 `pi=101` 을 밀면 그 쪽에는 아무것도 안 들어와 **쪽번호만 남은 빈
    // 쪽**이 된다 — `#6854` 와 같은 고아 쪽이다.
    let next_para_stored_vpos_exceeds_body = paragraphs.get(para_idx + 1).is_some_and(|next| {
        stored_vpos_overflow_defers_paragraph(
            next,
            paragraphs.get(para_idx + 2),
            page.body_height,
            dpi,
        )
    });
    // 본문 높이를 바꾸지 않는 컨트롤(각주/미주)만 허용 — 표/그림/글상자가 있으면
    // 줄 단위 split/배치 규칙이 달라지므로 제외.
    let only_note_controls = para
        .controls
        .iter()
        .all(|c| matches!(c, Control::Footnote(_) | Control::Endnote(_)));
    // [Task #1537] 원래 대상 — 폰트 치환 drift 로 꼬리 한 줄이 흘러넘치는 **글자 있는**
    // 여러 줄 문단.
    let font_drift_tail = !para.text.trim().is_empty() && fmt.line_heights.len() >= 2;
    // [#6854] 같은 고아 쪽이 **잉크 없는 빈 문단**으로도 생긴다. 78494 `pi=86` 은
    // 글자가 없는 한 줄짜리 문단인데 8쪽을 **2.3px** 넘겨(953.6+20.0 vs 971.3) 혼자
    // 9쪽을 열고, 바로 다음 `pi=87` 이 명시적 쪽나누기라 그 쪽에 더는 아무것도
    // 안 들어온다 — 꼬리말 `- 9 -` 만 있는 빈 쪽이 되고 이후 전 쪽이 +1 밀린다.
    //
    // 빈 문단은 하단 여백으로 흘려도 **그려지는 것이 없다** — 넘침이 잉크가 되지
    // 않으므로 `#1537` 이 걱정하던 bleed 가 성립하지 않는다. 초과 상한은 그대로
    // "한 줄 미만"을 쓴다(새 문턱을 만들지 않는다).
    //
    // ⚠ 여기서는 **선언된** 쪽나누기만 인정한다 — 사다리에서 추론한 경계까지 받으면
    // 쪽 이득 없이 넘침만 는다.
    let inkless_tail = para.text.trim().is_empty()
        && fmt.line_heights.len() == 1
        && (next_para_declares_page_break || next_para_stored_vpos_exceeds_body);
    // ⚠ 새 사실은 **잉크 없는 갈래만** 연다. 바깥 관문까지 넓히면 글자 있는
    // `font_drift_tail` 도 함께 열려 다른 문서의 쪽 귀속이 바뀐다 — 실측으로
    // `issue_1749`·`issue_2470` 핀과 본문 넘침 래칫 3구획이 깨졌다.
    page.col_count == 1
        && forced_page_break_line.is_none()
        && (next_para_forces_break || inkless_tail)
        && only_note_controls
        && page.has_items
        && (font_drift_tail || inkless_tail)
}
