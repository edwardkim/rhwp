//! 확정 페이지의 빈 꼬리·머리말·꼬리말·쪽 번호를 마무리한다.
use crate::renderer::typeset::paragraph::stored_line_overflows_body;
use crate::renderer::typeset::{
    stored_vpos_top_collision, ColumnBreakType, Control, HeaderFooterApply, HeaderFooterRef,
    PageContent, PageItem, Paragraph,
};
/// 끝 페이지가 가시 내용이나 명시적인 쪽/구역 나누기 없이 빈 문단만 가진 경우
/// 제거한다.
///
/// HWPX는 편집 중 남은 빈 문단에도 저장 vpos를 보존한다. 마지막 표 조각이 앞
/// 쪽을 거의 채우면 그 vpos가 빈 문단을 새 물리 쪽으로 보낼 수 있지만, 한컴
/// 출력은 이를 새 빈 쪽으로 인쇄하지 않는다. 여기서 앞 쪽의 실내용/의도적인
/// 페이지 나누기와 가시 control은 절대 제거하지 않는다.
///
/// [#5907] 앞뒤 문단이 둘 다 stored vpos 0 을 주장해 열린 쪽도 보존한다 —
/// 넘침 잔재가 아니라 한/글이 저장한 쪽 경계 그 자체이므로, 한/글도 그 빈 쪽을
/// 인쇄한다 (`samples/p122.hwp` 3쪽, 정본 `pdf/p122-2022.pdf`).
///
/// 저장 줄이 본문 높이를 넘는 빈 문단의 쪽도 보존한다. 편집기는 Enter 로 만든
/// 문단의 vpos 를 쪽 경계에서 되감지 않고 이어 쓰므로, 저장 조판으로도 앞 쪽에
/// 들어가지 않는다 — 흐름이 넘쳐 연 정당한 쪽이다. 버리면 그 문단이 어느 쪽에도
/// 속하지 않아 캐럿이 갈 곳이 없다.
pub(super) fn discard_terminal_blank_only_page(
    pages: &mut Vec<PageContent>,
    paragraphs: &[Paragraph],
) {
    if pages.len() <= 1 {
        return;
    }
    let Some(last_page) = pages.last() else {
        return;
    };
    // #3637의 저장 표 꼬리 호환 처리는 표 바로 뒤의 종료 문단에만 적용한다.
    // 일반 본문 줄을 거친 반복 Enter의 새 쪽은 그 표의 꼬리가 아니다.
    let Some(PageItem::PartialTable {
        para_index: table_para,
        ..
    }) = pages
        .get(pages.len() - 2)
        .and_then(|page| page.column_contents.last())
        .and_then(|column| column.items.last())
    else {
        return;
    };
    let mut tail_items = last_page
        .column_contents
        .iter()
        .flat_map(|column| &column.items);
    let Some(PageItem::FullParagraph {
        para_index: tail_para,
    }) = tail_items.next()
    else {
        return;
    };
    if tail_items.next().is_some() || table_para.checked_add(1) != Some(*tail_para) {
        return;
    }
    let mut has_item = false;
    let blank_only = last_page.column_contents.iter().all(|column| {
        if !column.wrap_around_paras.is_empty() {
            return false;
        }
        column.items.iter().all(|item| {
            let PageItem::FullParagraph { para_index } = item else {
                return false;
            };
            has_item = true;
            let Some(para) = paragraphs.get(*para_index) else {
                return false;
            };
            let no_visible_text = para
                .text
                .replace(|ch: char| ch.is_control(), "")
                .trim()
                .is_empty();
            let opened_by_stored_vpos_reset = *para_index > 0
                && paragraphs
                    .get(*para_index - 1)
                    .is_some_and(|prev| stored_vpos_top_collision(prev, para));
            let stored_line_overflows_body = stored_line_overflows_body(
                para,
                last_page.layout.body_area.height,
                last_page.layout.dpi,
            );
            no_visible_text
                && para.controls.is_empty()
                && !opened_by_stored_vpos_reset
                && !stored_line_overflows_body
                && !matches!(
                    para.column_type,
                    ColumnBreakType::Page | ColumnBreakType::Section
                )
        })
    });
    if has_item && blank_only {
        pages.pop();
    }
}
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
