//! 후속 본문 문단의 어울림 조회와 상태 명령을 기존 순서대로 연결한다.
//! 매칭/흡수/꼬리 정책은 각 Query 모듈, 상태 확정은 state가 소유한다.
//! 밴드 종료 전후의 높이를 합쳐 관측하지 않으며 가용 높이의 단락 평가를 보존한다.

use super::super::{TypesetEngine, TypesetState};
use super::{wrap_absorption, wrap_match, wrap_tail};
use crate::model::{page::PageDef, paragraph::Paragraph};
use crate::renderer::{composer::ComposedParagraph, style_resolver::ResolvedStyleSet};

/// true이면 호출자는 해당 문단의 일반 배치를 건너뛴다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn place(
    engine: &TypesetEngine,
    st: &mut TypesetState,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    has_table: bool,
    page_def: &PageDef,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
) -> bool {
    if st.following_wrap_active() && !has_table {
        let band = st.following_wrap_band();
        let wrap_match::WrapMatch {
            matched,
            is_empty_para,
        } = wrap_match::classify(para, paragraphs, page_def, band);
        if matched {
            // [Task #604 R3] wrap_around 매칭 분기를 anchor 종류 기반으로 본질화.
            //
            // - Picture (그림 Square wrap) anchor: wrap text 가 LineSeg cs/sw 로
            //   사전 인코딩됨 → wrap_anchors 등록 + FullParagraph 통과
            //   (layout 이 LineSeg cs/sw 정합 렌더)
            // - Table (표 Square wrap) anchor: wrap text 는 표 옆 빈 ↵ 표시용
            //   → 흡수 (current_column_wrap_around_paras)
            //
            // Stage 2b: Paragraph.wrap_precomputed (HWP3 휴리스틱 IR 누설) 제거.
            // anchor paragraph 의 controls 검사로 본질 정합 대체.
            let anchor_is_picture = wrap_match::anchor_is_picture(paragraphs, band);
            // [#6175] 유도 밴드의 앵커는 묶음(GroupShape)일 수 있다 — 개체
            // 종류로 흡수/통과를 가르는 이 판정에서 묶음 그림을 표로 오인하면
            // 밴드 옆 본문 문단이 통째로 흡수된다.
            if anchor_is_picture || st.following_wrap_is_derived() {
                let anchor = wrap_match::picture_anchor(paragraphs, band);
                st.register_following_wrap_anchor(para_idx, anchor);
            } else {
                // Table anchor: 어울림 문단을 표 옆에 기록 + height 소비 없음.
                // [Task #855] 단, 첫 줄만 표 옆이고 나머지 줄이 본문 전체 폭으로
                // 흐르는 문단(= 마지막 LINE_SEG 가 wrap zone cs/sw 와 불일치)은
                // 0-높이 흡수 대상이 아니다. 첫 LINE_SEG 만 보고 흡수하면 그런 문단이
                // 통째로 페이지 흐름에서 누락된다. 이 경우 wrap zone 을 종료하고
                // 일반 텍스트 배치로 폴백한다 (LINE_SEG cs/sw 가 이미 wrap 형상을
                // 인코딩하므로 layout 이 첫 줄을 표 옆에, 나머지를 표 아래에 렌더).
                if let Some(absorption) = wrap_absorption::whole_paragraph(
                    para,
                    paragraphs,
                    para_idx,
                    is_empty_para,
                    band,
                    engine.dpi,
                ) {
                    st.commit_wrap_absorption(absorption);
                    return true;
                }

                // [#4090] 빈 표 호스트 뒤의 문단은 처음 몇 줄만 표 왼쪽 띠에
                // 놓이고 마지막 한 줄은 표 아래 전폭으로 돌아올 수 있다. 이 경우
                // 문단 전체를 일반 흐름으로 두면 띠와 전폭 줄을 함께 다시 소비해
                // 이후 페이지가 과도하게 늘어난다. 저장 LINE_SEG와 조판 줄이 1:1이고
                // 전폭 꼬리가 정확히 한 줄인 안정적인 형상만 분리한다.
                let prefix = wrap_tail::classify_prefix(para, band, st.following_wrap_layout());
                let wrap_prefix_len = prefix.len;
                let col_width = st.following_wrap_column_width();
                let formatted = engine.format_paragraph(para, composed, styles, Some(col_width));
                if let Some(suffix_height) = prefix.suffix_height(para, &formatted, is_empty_para) {
                    if suffix_height <= st.available_height() + 0.5 {
                        let absorption = wrap_absorption::prefix(
                            para,
                            paragraphs,
                            para_idx,
                            wrap_prefix_len,
                            band,
                            engine.dpi,
                        );
                        st.commit_wrap_absorption(absorption);
                        st.end_following_wrap();
                        if st.wrap_tail_needs_advance(suffix_height) {
                            st.advance_column_or_new_page();
                        }
                        st.commit_wrap_tail(
                            para_idx,
                            wrap_prefix_len,
                            formatted.line_count(),
                            suffix_height,
                        );
                        return true;
                    }
                }
                // 이 문단은 첫 줄만 Square 띠에 있고 나머지는 표 아래 전폭으로
                // 복귀한다. 일반 fit 전에 띠 바닥을 흐름 하한으로 반영하지 않으면
                // 아래 줄이 표와 겹치는 높이를 아직 사용할 수 있다고 오판한다.
                st.end_following_wrap();
                // fall through → 일반 paragraph 배치
            }
        } else {
            // 매칭 실패 → wrap zone 종료, 정상 처리 진행
            st.end_following_wrap();
            // [Task #741 Stage 4] 매칭 실패 paragraph 의 vpos=0 hint (page break 의도)
            // 발견 시 advance_column_or_new_page. wrap_around active 종료 후 추가 가드.
            // hwp3-sample10-hwp5.hwp paragraph 26 ("● 제목차례 ●") case —
            // paragraph 22 anchor (cs=11084) active 유지로 line 419 vpos-reset 가드
            // 미발현 → 매칭 실패 후 추가 vpos-reset 가드로 페이지 break 정합.
            if wrap_tail::mismatch_starts_new_page(
                para,
                paragraphs,
                para_idx,
                st.following_wrap_has_items(),
                st.following_wrap_column_count(),
            ) {
                st.advance_column_or_new_page();
            }
        }
    }
    false
}
