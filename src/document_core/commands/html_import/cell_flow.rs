//! 셀 HTML 편집 후 인라인 표를 소유한 본문 줄도 현재 개체 점유로 발행한다.

use crate::document_core::DocumentCore;
use crate::model::control::Control;

impl DocumentCore {
    pub(super) fn reflow_html_cell_host(
        &mut self,
        section: usize,
        paragraph: usize,
        control: usize,
    ) {
        let host = &self.document.sections[section].paragraphs[paragraph];
        if !matches!(host.controls.get(control), Some(Control::Table(table)) if table.common.treat_as_char)
        {
            return;
        }
        let previous_end = crate::renderer::composer::paragraph_flow_end(host);
        let styles = self.resolve_render_styles();
        if let Some(Control::Table(table)) = self.document.sections[section].paragraphs[paragraph]
            .controls
            .get_mut(control)
        {
            crate::document_core::html_table_width::reserve_content_heights(
                table, &styles, self.dpi,
            );
        }
        // 동일 reflow는 실제 표 측정·바깥 여백을 줄 폭/높이에 함께 소비한다.
        // 저장의 작은 선언 높이로 재조판 결과를 덮어쓰지 않는다.
        self.reflow_paragraph(section, paragraph);
        let hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section].paragraphs,
            paragraph,
            None,
            previous_end,
            &self.styles,
            self.dpi,
            hwp3_layout,
        );
    }
}
