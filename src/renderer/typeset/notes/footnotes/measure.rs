//! 본문 각주 측정과 표 큐 최소 높이의 서로 다른 계약.

use crate::renderer::typeset::{
    compose_paragraph, hwpunit_to_px, Footnote, FootnoteFragment, ResolvedStyleSet, TypesetEngine,
    TypesetState,
};

/// 본문 예약과 reset 충돌 조회가 최종 각주 배치와 같은 표 높이를 소비한다.
pub(in crate::renderer::typeset) struct BodyFootnoteMeasure<'a> {
    pub engine: &'a TypesetEngine,
    pub styles: &'a ResolvedStyleSet,
}

impl BodyFootnoteMeasure<'_> {
    pub fn block_content_height(
        &self,
        st: &TypesetState,
        footnote: &Footnote,
        fragment: Option<FootnoteFragment>,
    ) -> Option<f64> {
        let layout = crate::renderer::layout::LayoutEngine::new(self.engine.dpi);
        self.engine.prime_section_cell_context(&layout);
        layout.set_layout_profile(st.profile);
        layout.prime_column_layout_env(&st.layout);
        layout.set_render_normalization_overlay(std::sync::Arc::clone(
            &self.engine.render_normalization,
        ));
        layout.footnote_with_block_table_content_height(
            footnote,
            self.styles,
            st.layout.body_area.width,
            fragment,
        )
    }

    pub fn content_height(&self, st: &TypesetState, footnote: &Footnote) -> f64 {
        self.block_content_height(st, footnote, None)
            .unwrap_or_else(|| composed_footnote_content_height(footnote, self.engine.dpi))
    }
}

impl crate::renderer::typeset::TypesetEngine {
    /// 재조판한 단일 단 본문 각주도 실제 FootnoteArea를 예약한다.
    /// 이미 구분선·여백을 포함한 상자에 추정용 안전 여백을 다시 더하지 않는다.
    pub(in crate::renderer::typeset) fn reserve_current_body_note_area(
        &self,
        st: &mut crate::renderer::typeset::TypesetState,
        paragraphs: &[crate::model::paragraph::Paragraph],
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    ) {
        if st.col_count != 1 {
            return;
        }
        let Some(page) = st.pages.last().filter(|page| !page.footnotes.is_empty()) else {
            return;
        };
        let layout = crate::renderer::layout::LayoutEngine::new(self.dpi);
        // 원 HWP5와 그 저장 줄을 보존한 HWPX는 같은 각주 쪽 분할을 따른다.
        // 앞 쪽에 참조가 배치된 이어받기 각주도 재조판 예약값으로 소속을 바꾸지 않는다.
        // 줄 정보가 없거나 합성·편집된 각주는 아래 실제 조판 높이를 예약한다.
        if st.profile.hwp5_stored_pagination_layout()
            && !st.profile.session_edited()
            && layout.footnote_area_has_stored_lines(
                &page.footnotes,
                paragraphs,
                st.layout.body_area.width,
            )
        {
            return;
        }
        layout.set_layout_profile(st.profile);
        layout.set_render_normalization_overlay(self.render_normalization.clone());
        let height = layout.estimate_footnote_area_height_with_metrics(
            &page.footnotes,
            paragraphs,
            styles,
            st.layout.body_area.width,
            st.footnote_separator_overhead,
            st.footnote_between_notes_margin,
        );
        st.reserve_painted_footnote_area(height);
    }
}

/// 각주의 실제 렌더 높이. 일반 paginator의 빠른 저장 LineSeg 추정과 달리
/// `layout_footnote_area`와 같은 composed line/line-spacing 규칙을 쓴다. 긴 URL은
/// renderer에서 두 줄 이상으로 재래핑되므로, 저장 LineSeg 한 줄만 예약하면 본문과
/// FootnoteArea가 겹친다. 문단 자체가 없는 malformed 각주는 실제 layout처럼 0을
/// 반환하고, 비어 있는 문단만 layout의 400HU 안내 줄을 예약한다.
pub(in crate::renderer::typeset) fn composed_footnote_content_height(
    footnote: &Footnote,
    dpi: f64,
) -> f64 {
    let total_lines: usize = footnote
        .paragraphs
        .iter()
        .map(|para| {
            crate::renderer::composer::compose_paragraph(para)
                .lines
                .len()
                .max(1)
        })
        .sum();
    if total_lines == 0 {
        return 0.0;
    }

    let mut height = 0.0;
    let mut line_index = 0usize;
    for para in &footnote.paragraphs {
        let composed = crate::renderer::composer::compose_paragraph(para);
        if composed.lines.is_empty() {
            height += hwpunit_to_px(400, dpi);
            line_index += 1;
            continue;
        }
        for line in &composed.lines {
            height += hwpunit_to_px(line.line_height, dpi);
            line_index += 1;
            if line_index < total_lines {
                height += hwpunit_to_px(line.line_spacing, dpi);
            }
        }
    }
    height
}

/// 표 셀 fragment queue의 기존 최소 예약값은 400HU다. Body 각주의 exact metric과
/// 섞지 않고 queue 경로에서만 이 하한을 유지한다.
pub(in crate::renderer::typeset) fn queued_table_footnote_content_height(
    footnote: &Footnote,
    dpi: f64,
) -> f64 {
    composed_footnote_content_height(footnote, dpi).max(hwpunit_to_px(400, dpi))
}
