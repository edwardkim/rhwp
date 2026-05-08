//! PaginationState: paginate_with_measured의 가변 상태를 캡슐화

use std::collections::HashMap;
use crate::renderer::page_layout::PageLayoutInfo;
use super::{PageContent, ColumnContent, PageItem, WrapAroundPara};

/// 페이지당 방어 로직 최대 실행 횟수.
/// 정상 문서에서는 절대 도달하지 않는 값. 이 값을 초과하면 무한 루프로 판단하고 강제 배치.
/// TODO: current_height 누적이 정확해지면 방어 로직과 함께 제거 가능.
const DEFENSE_MAX_PER_PAGE: u32 = 100;

/// paginate_with_measured의 12+ 가변 상태 변수를 구조체로 통합
pub(super) struct PaginationState {
    pub pages: Vec<PageContent>,
    pub current_items: Vec<PageItem>,
    pub current_height: f64,
    pub current_column: u16,
    pub current_footnote_height: f64,
    pub is_first_footnote_on_page: bool,
    pub col_count: u16,
    pub layout: PageLayoutInfo,
    pub current_zone_y_offset: f64,
    pub current_zone_layout: Option<PageLayoutInfo>,
    pub on_first_multicolumn_page: bool,
    pub section_index: usize,
    pub footnote_separator_overhead: f64,
    pub footnote_safety_margin: f64,
    /// 현재 단에 축적된 어울림 리턴 문단 목록
    pub current_column_wrap_around_paras: Vec<WrapAroundPara>,
    /// [Task #604 R3] 현재 단의 wrap text 문단 ↔ anchor 메타데이터
    pub current_column_wrap_anchors: std::collections::HashMap<usize, super::WrapAnchorRef>,
    /// 현재 페이지의 vpos 기준점 (첫 문단의 vertical_pos, HWPUNIT)
    /// layout의 vpos 보정과 동기화하기 위해 사용
    pub page_vpos_base: Option<i32>,
    /// 현재 페이지에 비-TAC 블록 표가 존재하는지 (vpos drift 보정용)
    pub page_has_block_table: bool,
    // --- 방어 로직 (TODO: current_height 누적이 정확해지면 제거 가능) ---
    /// 페이지별 방어 로직 실행 횟수 (레이어 1·2 공통).
    /// key: page_index (= pages.len() at time of defense), value: 실행 횟수
    pub defense_counts: HashMap<usize, u32>,
    /// 페이지 전환 시 overflow로 판정된 마지막 항목을 다음 페이지/단으로 이월
    pub overflow_carry: Option<PageItem>,
    /// 레이어 1이 advance_column_or_new_page를 호출 중임을 표시.
    /// 이 플래그가 true이면 레이어 2(check_last_item_overflow)를 스킵하여 이중 이동 방지.
    pub layer1_advancing: bool,
}

impl PaginationState {
    pub fn new(
        layout: PageLayoutInfo,
        col_count: u16,
        section_index: usize,
        footnote_separator_overhead: f64,
        footnote_safety_margin: f64,
    ) -> Self {
        Self {
            pages: Vec::new(),
            current_items: Vec::new(),
            current_height: 0.0,
            current_column: 0,
            current_footnote_height: 0.0,
            is_first_footnote_on_page: true,
            col_count,
            layout,
            current_zone_y_offset: 0.0,
            current_zone_layout: None,
            on_first_multicolumn_page: false,
            section_index,
            footnote_separator_overhead,
            footnote_safety_margin,
            current_column_wrap_around_paras: Vec::new(),
            current_column_wrap_anchors: std::collections::HashMap::new(),
            page_vpos_base: None,
            page_has_block_table: false,
            defense_counts: HashMap::new(),
            overflow_carry: None,
            layer1_advancing: false,
        }
    }

    /// 현재 항목을 ColumnContent로 만들어 마지막 페이지에 push
    pub fn flush_column(&mut self) {
        if self.current_items.is_empty() {
            return;
        }
        let col_content = ColumnContent {
            column_index: self.current_column,
            items: std::mem::take(&mut self.current_items),
            zone_layout: self.current_zone_layout.clone(),
            zone_y_offset: self.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.current_column_wrap_around_paras),
            used_height: self.current_height,
            wrap_anchors: std::mem::take(&mut self.current_column_wrap_anchors),
        };
        if let Some(page) = self.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.pages.push(self.new_page_content(vec![col_content]));
        }
    }

    /// 현재 항목을 ColumnContent로 만들어 (비어있어도) 마지막 페이지에 push
    pub fn flush_column_always(&mut self) {
        let col_content = ColumnContent {
            column_index: self.current_column,
            items: std::mem::take(&mut self.current_items),
            zone_layout: self.current_zone_layout.clone(),
            zone_y_offset: self.current_zone_y_offset,
            wrap_around_paras: std::mem::take(&mut self.current_column_wrap_around_paras),
            used_height: self.current_height,
            wrap_anchors: std::mem::take(&mut self.current_column_wrap_anchors),
        };
        if let Some(page) = self.pages.last_mut() {
            page.column_contents.push(col_content);
        } else {
            self.pages.push(self.new_page_content(vec![col_content]));
        }
    }

    /// 다음 단으로 이동하거나, 마지막 단이면 새 페이지 생성
    pub fn advance_column_or_new_page(&mut self) {
        self.flush_column();
        if self.current_column + 1 < self.col_count {
            self.current_column += 1;
            self.current_height = 0.0;
        } else {
            self.push_new_page();
        }
    }

    /// 페이지 전환 시 마지막 문단 항목이 overflow인지 점검.
    /// overflow이면 해당 항목을 overflow_carry로 이월하여 다음 페이지/단에 재삽입.
    /// FullParagraph / PartialParagraph만 대상 (표·글상자 제외).
    fn check_last_item_overflow(&mut self) {
        // carry가 이미 있으면 중복 방지
        if self.overflow_carry.is_some() {
            return;
        }
        let is_para_item = self.current_items.last().map_or(false, |item| {
            matches!(item, PageItem::FullParagraph { .. } | PageItem::PartialParagraph { .. })
        });
        if !is_para_item {
            return;
        }
        let available = self.available_height();
        if self.current_height <= available + 0.5 {
            return;
        }
        // overflow 감지: 방어 횟수 기록
        let page_idx = self.pages.len();
        let count = self.defense_counts.entry(page_idx).or_insert(0);
        *count += 1;
        if *count > DEFENSE_MAX_PER_PAGE {
            // 상한 초과: 무한 루프 최종 차단, carry 발동 안 함
            return;
        }
        self.overflow_carry = self.current_items.pop();
    }

    /// overflow_carry 항목을 현재 페이지/단에 재삽입하고 current_height를 보정.
    /// page_vpos_base 설정은 호출자(engine.rs)가 para.line_segs 접근 후 수행.
    pub fn reinsert_carry_with_height(&mut self, carry_height: f64) {
        if let Some(carry) = self.overflow_carry.take() {
            self.current_items.push(carry);
            self.current_height += carry_height;
        }
    }

    /// 강제 새 페이지 (쪽 나누기)
    pub fn force_new_page(&mut self) {
        self.flush_column();
        self.push_new_page();
    }

    /// 페이지가 비어있으면 첫 페이지 생성
    pub fn ensure_page(&mut self) {
        if self.pages.is_empty() {
            self.pages.push(self.new_page_content(Vec::new()));
        }
    }

    /// 사용 가능한 본문 높이 (각주 영역, 안전 여백, 존 오프셋 제외)
    pub fn available_height(&self) -> f64 {
        let base = self.layout.available_body_height();
        let fn_margin = if self.current_footnote_height > 0.0 {
            self.footnote_safety_margin
        } else {
            0.0
        };
        (base - self.current_footnote_height - fn_margin - self.current_zone_y_offset).max(0.0)
    }

    /// base_available_height (각주/존 오프셋 미차감)
    pub fn base_available_height(&self) -> f64 {
        self.layout.available_body_height()
    }

    /// 각주 높이 추가
    pub fn add_footnote_height(&mut self, height: f64) {
        if self.is_first_footnote_on_page {
            self.current_footnote_height += self.footnote_separator_overhead;
            self.is_first_footnote_on_page = false;
        }
        self.current_footnote_height += height;
    }

    /// 새 페이지 push + 상태 리셋
    fn push_new_page(&mut self) {
        self.pages.push(self.new_page_content(Vec::new()));
        self.reset_for_new_page();
    }

    /// 새 페이지 상태 리셋
    fn reset_for_new_page(&mut self) {
        self.current_column = 0;
        self.current_height = 0.0;
        self.current_footnote_height = 0.0;
        self.is_first_footnote_on_page = true;
        self.page_vpos_base = None;
        self.page_has_block_table = false;
        self.current_zone_y_offset = 0.0;
        self.current_zone_layout = None;
        self.on_first_multicolumn_page = false;
        self.current_column_wrap_around_paras.clear();
        self.current_column_wrap_anchors.clear();
    }

    /// PageContent 생성 헬퍼
    fn new_page_content(&self, column_contents: Vec<ColumnContent>) -> PageContent {
        PageContent {
            page_index: self.pages.len() as u32,
            page_number: 0,
            section_index: self.section_index,
            layout: self.layout.clone(),
            column_contents,
            active_header: None,
            active_footer: None,
            page_number_pos: None,
            page_hide: None,
            footnotes: Vec::new(),
            active_master_page: None,
            extra_master_pages: Vec::new(),
        }
    }
}
