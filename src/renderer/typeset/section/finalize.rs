//! 구역 finalize 책임. 기존 조건과 호출 순서를 보존한다.
use crate::renderer::typeset::{
    Control, HeaderFooterApply, HeaderFooterRef, Paragraph, TypesetEngine,
};
impl TypesetEngine {
    pub(in crate::renderer::typeset) fn collect_header_footer_controls(
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
}
