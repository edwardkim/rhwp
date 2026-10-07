//! 확정 페이지의 머리말·꼬리말·쪽 번호를 마무리한다.
use crate::renderer::typeset::{
    Control, HeaderFooterApply, HeaderFooterRef, PageContent, PageItem, Paragraph,
};
/// 페이지 번호 + 머리말/꼬리말 최종 할당 (기존 Paginator::finalize_pages와 동일)
pub(in crate::renderer::typeset) fn finalize_pages(
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
