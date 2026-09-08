//! 쪽번호 할당 (Issue #353)
//!
//! NewNumber 컨트롤은 배치 후 소스 위치로 확정한 페이지에서 1회만
//! page_number 를 갱신한다. 그 외 페이지는 직전 page_number + 1.
//! 문단의 첫 등장 규칙은 소스 줄을 결정할 수 없을 때의 fallback이다.
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
    /// production은 배치 후 확정된 쪽 이벤트를 사용한다.
    targets_are_pages: bool,
    page_ordinal: usize,
    /// NewNumber 컨트롤이 1건 이상 소비되었는지 여부.
    /// 한컴 호환: NewNumber가 존재하면 첫 발화 전 페이지에는 쪽번호 미표시.
    numbering_started: bool,
    /// 직전 [`assign`](Self::assign) 호출에서 NewNumber 가 발화했는지 여부.
    /// 구역 간 carry 를 재시작 지점 앞에서 멈추는 데 쓴다 (Issue #6206).
    last_restarted: bool,
}

impl<'a> PageNumberAssigner<'a> {
    /// `initial`: 페이지 카운터 시작값 (보통 1; 구역 carry 시 이전 구역 마지막 +1).
    pub fn new(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
        Self {
            new_page_numbers,
            consumed: HashSet::new(),
            counter: initial,
            targets_are_pages: false,
            page_ordinal: 0,
            numbering_started: false,
            last_restarted: false,
        }
    }

    /// 배치 후 소스 위치로 해석한 (구역 내 쪽 순번, 새 번호) 이벤트를 사용한다.
    pub fn new_for_pages(new_page_numbers: &'a [(usize, u16)], initial: u32) -> Self {
        let mut assigner = Self::new(new_page_numbers, initial);
        assigner.targets_are_pages = true;
        assigner
    }

    /// 페이지에 쪽번호를 할당하고, 다음 페이지를 위해 카운터를 1 증가시킨다.
    ///
    /// 한 페이지에 적용 가능한 NewNumber 가 여러 개 있어도 **마지막 1개만** 적용한다
    /// (소유 문단 인덱스 오름차순 — Vec 순서대로 평가하면 자연히 마지막이 우선).
    pub fn assign(&mut self, page: &PageContent) -> u32 {
        self.last_restarted = false;
        for (idx, &(nn_pi, nn_num)) in self.new_page_numbers.iter().enumerate() {
            if self.consumed.contains(&idx) {
                continue;
            }
            let applies = if self.targets_are_pages {
                nn_pi == self.page_ordinal
            } else {
                Self::para_first_appears(page, nn_pi)
            };
            if applies {
                self.counter = nn_num as u32;
                self.consumed.insert(idx);
                self.numbering_started = true;
                self.last_restarted = true;
            }
        }
        let assigned = self.counter;
        self.counter += 1;
        self.page_ordinal += 1;
        assigned
    }

    /// 직전 [`assign`](Self::assign) 이 NewNumber 로 카운터를 재설정했는지.
    ///
    /// 재시작 값은 절대값이므로 그 페이지부터는 구역 carry 를 더하면 안 된다 (Issue #6206).
    pub fn last_restarted(&self) -> bool {
        self.last_restarted
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
                PageItem::EndnoteSeparator { .. } => false,
            })
        })
    }
}

/// 소스 컨트롤을 배치된 쪽에 매핑한 1회성 이벤트. 두 조판 경로가 공유한다.
#[derive(Default)]
pub(crate) struct PageControlEvents {
    pub new_numbers: Vec<(usize, u16)>,
    pub hides: Vec<(usize, crate::model::control::PageHide)>,
}

impl PageControlEvents {
    pub fn collect(
        pages: &[PageContent],
        paragraphs: &[crate::model::paragraph::Paragraph],
    ) -> Self {
        use crate::model::control::Control;
        let mut by_paragraph: std::collections::HashMap<usize, Vec<(usize, &PageItem)>> =
            std::collections::HashMap::new();
        for (page_index, page) in pages.iter().enumerate() {
            for item in page.column_contents.iter().flat_map(|col| &col.items) {
                if !matches!(item, PageItem::EndnoteSeparator { .. }) {
                    by_paragraph
                        .entry(item.para_index())
                        .or_default()
                        .push((page_index, item));
                }
            }
        }
        let mut events = Self::default();
        for (pi, para) in paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                if matches!(
                    control,
                    Control::PageHide(_) | Control::NewNumber(_) | Control::Table(_)
                ) {
                    let items = by_paragraph.get(&pi).map(Vec::as_slice).unwrap_or(&[]);
                    // 미배치 NewNumber도 남겨 첫 발화 전 번호 숨김 계약을 보존한다.
                    let page = control_page(para, ci, items).unwrap_or(usize::MAX);
                    events.collect_control(control, page);
                }
            }
        }
        events
    }

    fn collect_control(&mut self, control: &crate::model::control::Control, page: usize) {
        use crate::model::control::{AutoNumberType, Control};
        match control {
            Control::PageHide(hide) => self.hides.push((page, hide.clone())),
            Control::NewNumber(number) if number.number_type == AutoNumberType::Page => {
                self.new_numbers.push((page, number.number));
            }
            // #6206: 셀 안 metadata는 외부 표의 첫 배치 쪽에 적용하는 기존 계약 유지.
            Control::Table(table) => {
                for cell in &table.cells {
                    for para in &cell.paragraphs {
                        for control in &para.controls {
                            self.collect_control(control, page);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn item_starts(item: &PageItem) -> bool {
    match item {
        PageItem::FullParagraph { .. } | PageItem::Table { .. } | PageItem::Shape { .. } => true,
        PageItem::PartialParagraph { start_line, .. } => *start_line == 0,
        PageItem::PartialTable {
            is_continuation, ..
        } => !*is_continuation,
        PageItem::EndnoteSeparator { .. } => false,
    }
}

fn control_page(
    para: &crate::model::paragraph::Paragraph,
    control_index: usize,
    items: &[(usize, &PageItem)],
) -> Option<usize> {
    let first = items
        .iter()
        .find(|(_, item)| item_starts(item))
        .map(|(page, _)| *page);
    if para.line_segs.len() < 2 || items.iter().all(|(page, _)| Some(*page) == first) {
        return first;
    }
    // 빈 컨트롤 전용 문단에는 visible char_offsets가 없다. 논리 위치는 PageHide 등을
    // 0칸으로 세므로 원본 char_count가 증명하는 8-unit 스트림 순서를 사용한다.
    let positions = if para.text.is_empty()
        && para.char_offsets.is_empty()
        && para.char_count as usize >= para.controls.len().saturating_mul(8)
    {
        (0..para.controls.len())
            .map(|ci| (ci as u32).saturating_mul(8))
            .collect()
    } else {
        para.control_utf16_positions()
    };
    let source_line = |ci: usize| {
        let position = *positions.get(ci)?;
        (0..para.line_segs.len())
            .rev()
            .find(|&line| para.line_seg_text_start(line) <= position)
    };
    let Some(target_line) = source_line(control_index) else {
        return first;
    };
    // 본문이 분할된 문단은 해당 소스 줄을 가진 fragment가 권위자다.
    if let Some((page, _)) = items.iter().find(|(_, item)| match item {
        PageItem::PartialParagraph {
            start_line,
            end_line,
            ..
        } => (*start_line..*end_line).contains(&target_line),
        PageItem::FullParagraph { .. } => !para.text.is_empty(),
        _ => false,
    }) {
        return Some(*page);
    }
    // 컨트롤 전용 문단에는 본문 fragment가 없으므로 같은 소스 줄에 실제 배치된
    // 표/도형을 따른다. 분할 표의 continuation은 첫 등장으로 취급하지 않는다.
    items
        .iter()
        .filter_map(|(page, item)| {
            let ci = match item {
                PageItem::Table { control_index, .. } | PageItem::Shape { control_index, .. } => {
                    *control_index
                }
                PageItem::PartialTable {
                    control_index,
                    is_continuation: false,
                    ..
                } => *control_index,
                _ => return None,
            };
            (source_line(ci) == Some(target_line)).then_some((ci.abs_diff(control_index), *page))
        })
        .min_by_key(|&(distance, page)| (distance, page))
        .map(|(_, page)| page)
        .or(first)
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
            pagination_tolerance_px: 0.0,
        }
    }

    fn mk_page(items: Vec<PageItem>) -> PageContent {
        PageContent {
            page_index: 0,
            page_number: 0,
            page_number_restarted: false,
            section_index: 0,
            layout: mk_layout(),
            column_contents: vec![ColumnContent {
                column_index: 0,
                start_height: 0.0,
                endnote_flow: false,
                items,
                zone_layout: None,
                zone_y_offset: 0.0,
                wrap_around_paras: Vec::new(),
                used_height: 0.0,
                wrap_anchors: std::collections::HashMap::new(),
                overlay_continuations: Vec::new(),
                overlay_cuts: Vec::new(),
                inline_placements: Default::default(),
                inline_flow_plans: Default::default(),
            }],
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
            ladder_band_tables: Vec::new(),
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
        // [#6206] 발화하지 않은 쪽은 재시작으로 표시되지 않는다 — 구역 carry 대상이다.
        assert!(!a.last_restarted());

        // page 2: para 5 (트리거) — 10
        let p2 = mk_page(vec![PageItem::FullParagraph { para_index: 5 }]);
        assert_eq!(a.assign(&p2), 10);
        // [#6206] 발화한 쪽만 재시작으로 표시된다 — 이 쪽부터 carry 를 멈춘다.
        assert!(a.last_restarted());

        // page 3: para 6 — 11 (NewNumber 재적용 금지)
        let p3 = mk_page(vec![PageItem::FullParagraph { para_index: 6 }]);
        assert_eq!(a.assign(&p3), 11);
        // [#6206] 재시작 표시는 다음 쪽으로 이월되지 않는다.
        assert!(!a.last_restarted());

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
            start_cut: Vec::new(),
            end_cut: Vec::new(),
            is_block_split: false,
            row_cursor_is_nested: false,
            end_row_height_override: None,
            start_row_height_override: None,
        }]);
        assert_eq!(a.assign(&p1), 1);

        // page 2: continuation — 적용 안 됨
        let p2 = mk_page(vec![PageItem::PartialTable {
            para_index: 5,
            control_index: 0,
            start_row: 3,
            end_row: 6,
            is_continuation: true,
            start_cut: Vec::new(),
            end_cut: Vec::new(),
            is_block_split: false,
            row_cursor_is_nested: false,
            end_row_height_override: None,
            start_row_height_override: None,
        }]);
        assert_eq!(a.assign(&p2), 2);
    }

    #[test]
    fn should_hide_before_first_new_number() {
        let nns = vec![(5usize, 1u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);
        assert!(
            a.should_hide_page_number(),
            "NewNumber 존재 + 미발화 → 숨김"
        );

        let p1 = mk_page(vec![PageItem::FullParagraph { para_index: 0 }]);
        a.assign(&p1);
        assert!(
            a.should_hide_page_number(),
            "아직 NewNumber 미트리거 → 숨김"
        );

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

    /// [#4369] 같은 문단에 NewNumber Page 컨트롤이 2개(12, 11 순)면 문서
    /// 순서상 **마지막**(11)이 채택되고 이후 단조 증가한다. HWP5/HWPX 재현
    /// (5쪽, p3 문단에 newNum 12→11 연속)에서 dump-pages 가 1,2,11,12,13 을
    /// 내는 계약의 단위 고정.
    #[test]
    fn same_paragraph_multiple_new_numbers_last_wins() {
        let nns = vec![(5usize, 12u16), (5usize, 11u16)];
        let mut a = PageNumberAssigner::new(&nns, 1);

        let p1 = mk_page(vec![PageItem::FullParagraph { para_index: 0 }]);
        assert_eq!(a.assign(&p1), 1);

        // NewNumber 2개가 같은 문단에서 함께 트리거 — 마지막(11) 채택
        let p2 = mk_page(vec![PageItem::FullParagraph { para_index: 5 }]);
        assert_eq!(a.assign(&p2), 11);

        let p3 = mk_page(vec![PageItem::FullParagraph { para_index: 6 }]);
        assert_eq!(a.assign(&p3), 12);
        let p4 = mk_page(vec![PageItem::FullParagraph { para_index: 7 }]);
        assert_eq!(a.assign(&p4), 13);
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
