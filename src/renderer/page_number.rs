//! 쪽번호 할당 (Issue #353)
//!
//! NewNumber 컨트롤은 그 컨트롤의 소유 문단이 페이지에서 **처음 등장**할 때
//! 1회만 page_number 를 갱신해야 한다. 그 외 페이지는 직전 page_number + 1.
//!
//! "처음 등장" 판정 — PartialParagraph/PartialTable 의 분할은 첫 분할만 인정:
//! - FullParagraph                                : 항상 인정
//! - PartialParagraph { start_line == 0 }         : 첫 분할
//! - Table                                        : 항상 인정
//! - PartialTable    { is_continuation == false } : 첫 분할
//! - Shape                                        : 항상 인정

use std::collections::HashSet;

use crate::renderer::pagination::{PageContent, PageItem};

/// 쪽번호를 1회성 NewNumber 적용 + 단조 증가로 계산하는 어시스턴트.
pub(crate) struct PageNumberAssigner<'a> {
    new_page_numbers: &'a [(usize, u16)],
    consumed: HashSet<usize>,
    counter: u32,
    /// NewNumber 컨트롤이 1건 이상 소비되었는지 여부.
    /// 한컴 호환: NewNumber가 존재하면 첫 발화 전 페이지에는 쪽번호 미표시.
    numbering_started: bool,
}

impl<'a> PageNumberAssigner<'a> {
    /// `initial`: 페이지 카운터 시작값 (보통 1; 구역 carry 시 이전 구역 마지막 +1).
    pub fn new(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
        Self {
            new_page_numbers,
            consumed: HashSet::new(),
            counter: initial,
            numbering_started: false,
        }
    }

    /// 페이지에 쪽번호를 할당하고, 다음 페이지를 위해 카운터를 1 증가시킨다.
    ///
    /// 한 페이지에 적용 가능한 NewNumber 가 여러 개 있어도 **마지막 1개만** 적용한다
    /// (소유 문단 인덱스 오름차순 — Vec 순서대로 평가하면 자연히 마지막이 우선).
    pub fn assign(&mut self, page: &PageContent) -> u32 {
        for (idx, &(nn_pi, nn_num)) in self.new_page_numbers.iter().enumerate() {
            if self.consumed.contains(&idx) {
                continue;
            }
            if Self::para_first_appears(page, nn_pi) {
                self.counter = nn_num as u32;
                self.consumed.insert(idx);
                self.numbering_started = true;
            }
        }
        let assigned = self.counter;
        self.counter += 1;
        assigned
    }

    /// 다음 페이지에 적용될 카운터 값 (구역 carry 용).
    pub fn next_counter(&self) -> u32 {
        self.counter
    }

    /// NewNumber가 존재하지만 아직 발화되지 않은 상태인지 판별한다.
    /// true이면 이 페이지에 쪽번호를 표시하지 않아야 한다 (한컴 호환).
    pub fn should_hide_page_number(&self) -> bool {
        !self.new_page_numbers.is_empty() && !self.numbering_started
    }

    fn para_first_appears(page: &PageContent, target_pi: usize) -> bool {
        page.column_contents.iter().any(|col| {
            col.items.iter().any(|item| match item {
                PageItem::FullParagraph { para_index } => *para_index == target_pi,
                PageItem::PartialParagraph {
                    para_index,
                    start_line,
                    ..
                } => *para_index == target_pi && *start_line == 0,
                PageItem::Table { para_index, .. } => *para_index == target_pi,
                PageItem::PartialTable {
                    para_index,
                    is_continuation,
                    ..
                } => *para_index == target_pi && !*is_continuation,
                PageItem::Shape { para_index, .. } => *para_index == target_pi,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::page_layout::{LayoutRect, PageLayoutInfo};
    use crate::renderer::pagination::{ColumnContent, PageContent, PageItem};

    fn mk_layout() -> PageLayoutInfo {
        PageLayoutInfo {
            page_width: 0.0,
            page_height: 0.0,
            header_area: LayoutRect::default(),
            body_area: LayoutRect::default(),
            column_areas: Vec::new(),
            footnote_area: LayoutRect::default(),
            footer_area: LayoutRect::default(),
            dpi: 96.0,
            separator_type: 0,
            separator_width: 0,
            separator_color: 0,
        }
    }

    fn mk_page(items: Vec<PageItem>) -> PageContent {
        PageContent {
            page_index: 0,
            page_number: 0,
            section_index: 0,
            layout: mk_layout(),
            column_contents: vec![ColumnContent {
                column_index: 0,
                items,
                zone_layout: None,
                zone_y_offset: 0.0,
                wrap_around_paras: Vec::new(),
                used_height: 0.0,
            wrap_anchors: std::collections::HashMap::new(),
            }],
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
        }
    }

    #[test]
    fn no_new_number_means_monotonic_from_initial() {
        let mut a = PageNumberAssigner::new(&[], 1);
        let p = mk_page(vec![PageItem::FullParagraph { para_index: 0 }]);
        assert_eq!(a.assign(&p), 1);
        assert_eq!(a.assign(&p), 2);
        assert_eq!(a.assign(&p), 3);
    }

    #[test]
    fn new_number_applied_once_then_monotonic() {
        // NewNumber Page=10 at para 5
        let nns = vec![(5usize, 10u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        // page 1: paras 0..3 — NewNumber 미트리거
        let p1 = mk_page(vec![
            PageItem::FullParagraph { para_index: 0 },
            PageItem::FullParagraph { para_index: 1 },
        ]);
        assert_eq!(a.assign(&p1), 1);

        // page 2: para 5 (트리거) — 10
        let p2 = mk_page(vec![PageItem::FullParagraph { para_index: 5 }]);
        assert_eq!(a.assign(&p2), 10);

        // page 3: para 6 — 11 (NewNumber 재적용 금지)
        let p3 = mk_page(vec![PageItem::FullParagraph { para_index: 6 }]);
        assert_eq!(a.assign(&p3), 11);

        // page 4: para 7 — 12
        let p4 = mk_page(vec![PageItem::FullParagraph { para_index: 7 }]);
        assert_eq!(a.assign(&p4), 12);
    }

    #[test]
    fn partial_paragraph_first_split_triggers() {
        let nns = vec![(5usize, 1u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        // page 1: PartialParagraph 첫 분할 — 트리거
        let p1 = mk_page(vec![PageItem::PartialParagraph {
            para_index: 5,
            start_line: 0,
            end_line: 3,
        }]);
        assert_eq!(a.assign(&p1), 1);

        // page 2: PartialParagraph 두번째 분할 — 트리거 안 함 (이미 consumed)
        let p2 = mk_page(vec![PageItem::PartialParagraph {
            para_index: 5,
            start_line: 3,
            end_line: 6,
        }]);
        assert_eq!(a.assign(&p2), 2);
    }

    #[test]
    fn partial_paragraph_non_first_split_does_not_trigger() {
        // NewNumber 트리거 문단이 PartialParagraph 의 두번째 분할에만 등장하는 경우
        // (start_line > 0) — 적용 안 됨. 카운터는 그냥 진행.
        let nns = vec![(5usize, 100u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        let p1 = mk_page(vec![PageItem::PartialParagraph {
            para_index: 5,
            start_line: 2,
            end_line: 4,
        }]);
        assert_eq!(a.assign(&p1), 1);
        assert!(a.consumed.is_empty(), "not consumed when start_line>0");
    }

    #[test]
    fn partial_table_continuation_does_not_trigger() {
        let nns = vec![(5usize, 1u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        // page 1: 첫 분할 — 트리거
        let p1 = mk_page(vec![PageItem::PartialTable {
            para_index: 5,
            control_index: 0,
            start_row: 0,
            end_row: 3,
            is_continuation: false,
            split_start_content_offset: 0.0,
            split_end_content_limit: 0.0,
        }]);
        assert_eq!(a.assign(&p1), 1);

        // page 2: continuation — 적용 안 됨
        let p2 = mk_page(vec![PageItem::PartialTable {
            para_index: 5,
            control_index: 0,
            start_row: 3,
            end_row: 6,
            is_continuation: true,
            split_start_content_offset: 0.0,
            split_end_content_limit: 0.0,
        }]);
        assert_eq!(a.assign(&p2), 2);
    }

    #[test]
    fn should_hide_before_first_new_number() {
        let nns = vec![(5usize, 1u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);
        assert!(a.should_hide_page_number(), "NewNumber 존재 + 미발화 → 숨김");

        let p1 = mk_page(vec![PageItem::FullParagraph { para_index: 0 }]);
        a.assign(&p1);
        assert!(a.should_hide_page_number(), "아직 NewNumber 미트리거 → 숨김");

        let p2 = mk_page(vec![PageItem::FullParagraph { para_index: 5 }]);
        a.assign(&p2);
        assert!(!a.should_hide_page_number(), "NewNumber 발화 후 → 표시");

        let p3 = mk_page(vec![PageItem::FullParagraph { para_index: 6 }]);
        a.assign(&p3);
        assert!(!a.should_hide_page_number(), "이후에도 계속 표시");
    }

    #[test]
    fn should_not_hide_when_no_new_numbers() {
        let a = PageNumberAssigner::new(&[], 1);
        assert!(!a.should_hide_page_number(), "NewNumber 없으면 항상 표시");
    }

    #[test]
    fn multiple_new_numbers_each_consumed_once() {
        // 별첨 시작 시점에 NewNumber=1 이 또 한번 등장하는 케이스
        let nns = vec![(5usize, 1u16), (20usize, 1u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        // page 1: 첫 NewNumber 트리거 → 1
        let p1 = mk_page(vec![PageItem::FullParagraph { para_index: 5 }]);
        assert_eq!(a.assign(&p1), 1);
        // page 2: → 2
        let p2 = mk_page(vec![PageItem::FullParagraph { para_index: 6 }]);
        assert_eq!(a.assign(&p2), 2);
        // page 3: 두번째 NewNumber 트리거 → 1
        let p3 = mk_page(vec![PageItem::FullParagraph { para_index: 20 }]);
        assert_eq!(a.assign(&p3), 1);
        // page 4: → 2
        let p4 = mk_page(vec![PageItem::FullParagraph { para_index: 21 }]);
        assert_eq!(a.assign(&p4), 2);
    }
}
