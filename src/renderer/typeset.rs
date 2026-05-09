//! 단일 패스 조판 엔진 (TypesetEngine)
//!
//! 기존 3단계 파이프라인(height_measurer → pagination → layout)을 대체하는
//! 단일 패스 조판 엔진. 각 요소를 format() → fits() → place/split 순서로
//! 처리하여 측정과 배치를 하나의 흐름으로 통합한다.
//!
//! Phase 2: Break Token 기반 표 조판 구현.
//! Chromium LayoutNG의 Break Token 패턴, LibreOffice Writer의 Master/Follow Chain,
//! MS Word/OOXML의 cantSplit/tblHeader를 참고.

use crate::model::control::Control;
use crate::model::shape::CaptionDirection;
use crate::model::header_footer::HeaderFooterApply;
use crate::model::paragraph::{Paragraph, ColumnBreakType};
use crate::model::page::{PageDef, ColumnDef, ColumnType};
use crate::renderer::composer::ComposedParagraph;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::{hwpunit_to_px, DEFAULT_DPI};
use super::pagination::{
    PaginationResult, PageContent, ColumnContent, PageItem,
    HeaderFooterRef, FootnoteRef, FootnoteSource,
};

// ========================================================
// Break Token — 조판 분할 지점 (Chromium LayoutNG 참고)
// ========================================================

/// 표 조판의 분할 재개 정보.
/// 다음 페이지에서 이 토큰으로부터 이어서 조판한다.
#[derive(Debug, Clone)]
struct TableBreakToken {
    /// 재개할 시작 행 인덱스
    start_row: usize,
    /// 인트라-로우 분할 시 각 셀의 콘텐츠 오프셋
    cell_content_offsets: Option<Vec<f64>>,
}

// ========================================================
// FormattedTable — 표의 format() 결과
// ========================================================

/// 표의 조판 높이 정보 (format 단계 결과).
/// 기존 MeasuredTable + host_spacing을 통합하여 측정-배치 일원화.
#[derive(Debug)]
struct FormattedTable {
    /// 행별 높이 (px)
    row_heights: Vec<f64>,
    /// 행간 간격 (px)
    cell_spacing: f64,
    /// 머리행 수 (repeat_header && has_header_cells일 때 1)
    header_row_count: usize,
    /// 호스트 문단 spacing
    host_spacing: HostSpacing,
    /// 표 자체 높이 (host_spacing 미포함)
    effective_height: f64,
    /// 전체 높이 (host_spacing 포함)
    total_height: f64,
    /// 캡션 높이
    caption_height: f64,
    /// TAC 표 여부
    is_tac: bool,
    /// 누적 행 높이 (cell_spacing 포함)
    cumulative_heights: Vec<f64>,
    /// 표 쪽 나눔 설정
    page_break: crate::model::table::TablePageBreak,
    /// 셀별 측정 데이터 (인트라-로우 분할용)
    cells: Vec<crate::renderer::height_measurer::MeasuredCell>,
    /// 표 셀 내 각주 높이 합계 (가용 높이에서 차감)
    table_footnote_height: f64,
}

/// 호스트 문단의 spacing (표 전/후)
#[derive(Debug, Clone, Copy)]
struct HostSpacing {
    /// 표 앞 spacing (spacing_before + outer_margin_top)
    before: f64,
    /// 표 뒤 spacing (spacing_after + outer_margin_bottom + host_line_spacing)
    after: f64,
    /// spacing_after만 (마지막 fragment용 — Paginator와 동일)
    spacing_after_only: f64,
}

/// 단일 패스 조판 엔진
pub struct TypesetEngine {
    dpi: f64,
}

/// 조판 중 현재 페이지/단 상태
struct TypesetState {
    /// 완성된 페이지 목록
    pages: Vec<PageContent>,
    /// 현재 단에 쌓이는 항목
    current_items: Vec<PageItem>,
    /// 현재 단에서 소비된 높이 (px)
    current_height: f64,
    /// 현재 단 인덱스
    current_column: u16,
    /// 단 수
    col_count: u16,
    /// 페이지 레이아웃
    layout: PageLayoutInfo,
    /// 구역 인덱스
    section_index: usize,
    /// 각주 높이 누적
    current_footnote_height: f64,
    /// 첫 각주 여부
    is_first_footnote_on_page: bool,
    /// 각주 구분선 오버헤드
    footnote_separator_overhead: f64,
    /// 각주 안전 여백
    footnote_safety_margin: f64,
    /// 존(zone) y 오프셋 (다단 나누기 시 누적)
    current_zone_y_offset: f64,
    /// 현재 존의 레이아웃 오버라이드
    current_zone_layout: Option<PageLayoutInfo>,
    /// 다단 첫 페이지 여부
    on_first_multicolumn_page: bool,
    /// Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지하는 높이 (px).
    /// col 1 이상으로 advance 시 zone_y_offset에 반영.
    pending_body_wide_top_reserve: f64,
    /// [Task #359] 다음 pi 가 vpos-reset 가드를 발동할 예정 → 현재 pi 의 fit 안전마진 비활성화.
    /// 단독 항목 페이지 발생 차단용.
    skip_safety_margin_once: bool,
    /// [Task #362] 한컴 빈 줄 감추기 옵션 (SectionDef bit 19). true 이면 페이지 시작에서
    /// overflow 유발하는 빈 paragraph 최대 2개까지 height=0 처리.
    hide_empty_line: bool,
    /// [Task #362] 현재 페이지에서 감춘 빈 줄 수 (페이지마다 reset, 최대 2).
    hidden_empty_lines: u32,
    /// [Task #362] 감춘 빈 줄이 적용된 페이지 인덱스 (페이지 변경 감지용).
    hidden_empty_page_idx: usize,
    /// [Task #362] hide_empty_line 으로 감춘 paragraph 인덱스 (PaginationResult 에 포함).
    hidden_empty_paras: std::collections::HashSet<usize>,
    /// [Task #362] Square wrap 표의 column_start (HU). -1 = 비활성. 후속 같은 cs/sw paragraph 흡수용.
    wrap_around_cs: i32,
    /// [Task #362] Square wrap 표의 segment_width (HU). -1 = 비활성.
    wrap_around_sw: i32,
    /// [Task #362] Square wrap 표가 있는 paragraph 인덱스 (WrapAroundPara 에 기록).
    wrap_around_table_para: usize,
    /// 비-TAC Picture/Shape Square wrap: any_seg_matches만으로 후속 문단 판정 허용.
    /// 그림의 lineseg는 첫 seg cs=0일 수 있어 전체 seg 중 하나라도 일치하면 흡수.
    wrap_around_any_seg: bool,
    /// [Task #362] 현재 단에서 표 옆에 배치되는 wrap-around paragraphs.
    /// flush_column 에서 ColumnContent 로 전달.
    current_column_wrap_around_paras: Vec<crate::renderer::pagination::WrapAroundPara>,
    /// [Task #604 R3] 현재 단의 wrap text 문단 ↔ anchor 메타데이터.
    /// wrap_around state machine 매칭 시 등록. flush_column 에서 ColumnContent 로 전달.
    current_column_wrap_anchors: std::collections::HashMap<usize, crate::renderer::pagination::WrapAnchorRef>,
    /// [Task #702] 현재 zone 의 ColumnType (Normal/Distribute/Parallel).
    /// process_multicolumn_break 에서 새 ColumnDef 매칭 시 갱신.
    /// Distribute 다단의 짧은 컬럼 vpos-reset 검출 임계값 완화에 사용.
    current_zone_column_type: ColumnType,
}

impl TypesetState {
    fn new(
        layout: PageLayoutInfo,
        col_count: u16,
        section_index: usize,
        footnote_separator_overhead: f64,
        footnote_safety_margin: f64,
        column_type: ColumnType,
    ) -> Self {
        Self {
            pages: Vec::new(),
            current_items: Vec::new(),
            current_height: 0.0,
            current_column: 0,
            col_count,
            layout,
            section_index,
            current_footnote_height: 0.0,
            is_first_footnote_on_page: true,
            footnote_separator_overhead,
            footnote_safety_margin,
            current_zone_y_offset: 0.0,
            current_zone_layout: None,
            on_first_multicolumn_page: false,
            pending_body_wide_top_reserve: 0.0,
            skip_safety_margin_once: false,
            hide_empty_line: false,
            hidden_empty_lines: 0,
            hidden_empty_page_idx: usize::MAX,
            hidden_empty_paras: std::collections::HashSet::new(),
            wrap_around_cs: -1,
            wrap_around_sw: -1,
            wrap_around_table_para: 0,
            wrap_around_any_seg: false,
            current_column_wrap_around_paras: Vec::new(),
            current_column_wrap_anchors: std::collections::HashMap::new(),
            current_zone_column_type: column_type,
        }
    }

    /// 사용 가능한 본문 높이 (각주, 존 오프셋 차감)
    fn available_height(&self) -> f64 {
        let base = self.layout.available_body_height();
        let fn_margin = if self.current_footnote_height > 0.0 {
            self.footnote_safety_margin
        } else {
            0.0
        };
        (base - self.current_footnote_height - fn_margin - self.current_zone_y_offset).max(0.0)
    }

    /// 기본 가용 높이 (각주/존 미차감)
    fn base_available_height(&self) -> f64 {
        self.layout.available_body_height()
    }

    /// 각주 높이 추가
    fn add_footnote_height(&mut self, height: f64) {
        if self.is_first_footnote_on_page {
            self.current_footnote_height += self.footnote_separator_overhead;
            self.is_first_footnote_on_page = false;
        }
        self.current_footnote_height += height;
    }

    /// 현재 항목을 ColumnContent로 만들어 마지막 페이지에 push
    fn flush_column(&mut self) {
        if self.current_items.is_empty() && self.current_column_wrap_around_paras.is_empty() {
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

    /// 비어있어도 flush
    fn flush_column_always(&mut self) {
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

    /// 다음 단 또는 새 페이지
    fn advance_column_or_new_page(&mut self) {
        self.flush_column();
        if self.current_column + 1 < self.col_count {
            self.current_column += 1;
            // Task #321: col 0 상단의 body-wide TopAndBottom 표/도형이 차지한 높이를
            // current_height의 시작값으로 사용 (가용 공간만 줄임, zone_y_offset은 건드리지 않음).
            // layout은 body_wide_reserved로 별도 처리하므로 여기서 zone_y_offset에
            // 넣으면 double-shift가 발생.
            self.current_height = self.pending_body_wide_top_reserve;
        } else {
            self.push_new_page();
        }
    }

    /// 강제 새 페이지
    fn force_new_page(&mut self) {
        self.flush_column();
        self.push_new_page();
    }

    /// 페이지 보장
    fn ensure_page(&mut self) {
        if self.pages.is_empty() {
            self.pages.push(self.new_page_content(Vec::new()));
        }
    }

    /// 새 페이지 push + 상태 리셋
    fn push_new_page(&mut self) {
        self.pages.push(self.new_page_content(Vec::new()));
        self.reset_for_new_page();
        // Task #321: 새 페이지에서는 body-wide top reserve 초기화
        self.pending_body_wide_top_reserve = 0.0;
    }

    fn reset_for_new_page(&mut self) {
        self.current_column = 0;
        self.current_height = 0.0;
        self.current_footnote_height = 0.0;
        self.is_first_footnote_on_page = true;
        self.current_zone_y_offset = 0.0;
        self.current_zone_layout = None;
        self.on_first_multicolumn_page = false;
    }

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

/// 문단 format() 결과: 문단의 실제 렌더링 높이 정보
#[derive(Debug)]
struct FormattedParagraph {
    /// 총 높이 (spacing 포함)
    total_height: f64,
    /// 줄별 콘텐츠 높이 (line_height만)
    line_heights: Vec<f64>,
    /// 줄별 줄간격 (line_spacing)
    line_spacings: Vec<f64>,
    /// spacing_before
    spacing_before: f64,
    /// spacing_after
    spacing_after: f64,
    /// trailing line_spacing을 제외한 판단용 높이
    height_for_fit: f64,
}

impl FormattedParagraph {
    /// 특정 줄의 advance 높이 (콘텐츠 + 줄간격)
    #[inline]
    fn line_advance(&self, line_idx: usize) -> f64 {
        self.line_heights[line_idx] + self.line_spacings[line_idx]
    }

    /// 줄 범위의 advance 합계
    fn line_advances_sum(&self, range: std::ops::Range<usize>) -> f64 {
        range.into_iter()
            .map(|i| self.line_heights[i] + self.line_spacings[i])
            .sum()
    }
}

impl TypesetEngine {
    pub fn new(dpi: f64) -> Self {
        Self { dpi }
    }

    pub fn with_default_dpi() -> Self {
        Self::new(DEFAULT_DPI)
    }

    /// 구역의 문단 목록을 조판한다 (단일 패스).
    ///
    /// 기존 paginate()와 동일한 PaginationResult를 반환하므로
    /// 기존 layout/render 파이프라인과 호환된다.
    pub fn typeset_section(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        measured_tables: &[MeasuredTable],
        hide_empty_line: bool,
    ) -> PaginationResult {
        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        let col_count = column_def.column_count.max(1);
        let footnote_separator_overhead = hwpunit_to_px(400, self.dpi);
        let footnote_safety_margin = hwpunit_to_px(3000, self.dpi);

        let mut st = TypesetState::new(
            layout, col_count, section_index,
            footnote_separator_overhead, footnote_safety_margin,
            column_def.column_type,
        );
        st.hide_empty_line = hide_empty_line;

        // 머리말/꼬리말/쪽 번호/새 번호/감추기 컨트롤 수집
        let (hf_entries, page_number_pos, new_page_numbers, page_hides) =
            Self::collect_header_footer_controls(paragraphs, section_index);

        for (para_idx, para) in paragraphs.iter().enumerate() {
            // 표 컨트롤 감지
            let has_table = self.paragraph_has_table(para);

            // [Task #702] 새 ColumnDef 검출. shortcut.hwp p2/p3 파일/미리보기/편집 등은
            // [쪽나누기]+단정의:1단 (header) → [단나누기]+단정의:2단 (content) 패턴 사용.
            // [다단나누기] 외에도 Page/Column break 의 ColumnDef 차이도 zone 재정의 신호로 인식.
            let new_col_def_opt: Option<ColumnDef> = para.controls.iter().find_map(|c| {
                if let Control::ColumnDef(cd) = c { Some(cd.clone()) } else { None }
            });
            let has_diff_col_def = new_col_def_opt.as_ref().map(|cd| {
                cd.column_count.max(1) != st.col_count
                    || cd.column_type != st.current_zone_column_type
            }).unwrap_or(false);

            // 다단 나누기
            if para.column_type == ColumnBreakType::MultiColumn {
                self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
            }

            // 단 나누기
            if para.column_type == ColumnBreakType::Column {
                if has_diff_col_def {
                    // [Task #702] 단나누기 + 새 ColumnDef = zone 재정의 (MultiColumn 등가 처리)
                    self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
                } else if !st.current_items.is_empty() {
                    st.advance_column_or_new_page();
                }
            }

            // 쪽 나누기
            let force_page_break = para.column_type == ColumnBreakType::Page
                || para.column_type == ColumnBreakType::Section;
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let para_style_break = para_style.map(|s| s.page_break_before).unwrap_or(false);

            if (force_page_break || para_style_break) && !st.current_items.is_empty() {
                st.force_new_page();
                // [Task #702] 쪽나누기 + 새 ColumnDef = 새 페이지에서 col 정의 적용
                if has_diff_col_def {
                    if let Some(cd) = &new_col_def_opt {
                        st.col_count = cd.column_count.max(1);
                        let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                        st.current_zone_layout = Some(new_layout.clone());
                        st.layout = new_layout;
                        st.current_zone_column_type = cd.column_type;
                    }
                }
            }

            // Task #321: 문단간 vpos-reset 기반 강제 분할
            // HWP LINE_SEG의 vertical_pos는 페이지 내 흐름 y 좌표.
            // 현재 문단 first_vpos=0이고 직전 문단이 같은 단에 있으며 last_vpos가 충분히 큰 경우,
            // HWP가 pi 경계에서 페이지/단 분할을 의도한 것 → 강제 분할.
            // [Task #362] wrap-around zone 활성 중에는 vpos-reset 가드 무시 (기존).
            // [Task #724] vpos-reset trigger 발동 시 wrap_around 강제 종료 (신규):
            // HWP5 변환본 case 에서 paragraph 442/443 wrap_around 매칭 후 후속 paragraph
            // (예: 599) vpos=0 시점에도 wrap_around active 유지되어 페이지 분할 위반 →
            // vpos-reset trigger 시 wrap_around 강제 종료 + advance_column_or_new_page.
            if para_idx > 0 && !st.current_items.is_empty() {
                let prev_para = &paragraphs[para_idx - 1];
                let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
                let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
                if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
                    // 현재 문단의 vpos가 직전 문단의 마지막 vpos보다 작은 경우 — 컬럼/페이지 reset 시그널.
                    // - 단일 단: cv == 0 만 인정 (Task #321 보수적 기준 유지).
                    //   단일 단에서 cv != 0 의 cv < pv 는 partial-table split 의 LAYOUT 잔재로
                    //   해석되어야 함 (issue #418 / hwpspec pi=78→pi=79).
                    // - 다단 Normal (NEWSPAPER): cv != 0 도 인정 (Task #470). pv > 5000 임계값 유지.
                    // - 다단 Distribute (BalancedNewspaper): 짧은 컬럼 (3+3 분배 등) 에서 pv 가
                    //   임계값 미달일 수 있어 pv > 0 으로 완화 (Task #702, shortcut 지우기 6항목 정합).
                    //   단일 단/Normal 다단은 영향 없음.
                    let is_distribute = st.col_count > 1
                        && matches!(st.current_zone_column_type, ColumnType::Distribute);
                    let trigger = if st.col_count > 1 {
                        if is_distribute {
                            cv < pv && pv > 0
                        } else {
                            cv < pv && pv > 5000
                        }
                    } else {
                        cv == 0 && pv > 5000
                    };
                    if trigger {
                        // [Task #724] wrap_around active 시 강제 종료 — anchor cs=0
                        // (HWP5 변환본 caption-style) 한정. 일반 wrap_around (anchor cs>0)
                        // 는 기존 동작 (Task #362 vpos-reset 무시) 유지.
                        if st.wrap_around_cs == 0 {
                            st.wrap_around_cs = -1;
                            st.wrap_around_sw = -1;
                            st.wrap_around_any_seg = false;
                        }
                        if st.wrap_around_cs < 0 {
                            st.advance_column_or_new_page();
                        }
                    }
                }
            }

            // [Task #359] 단독 항목 페이지 차단:
            // 다음 pi 가 vpos-reset 가드를 발동할 예정이고 현재 pi 가 잔여 공간 부족으로
            // 새 페이지를 시작하면 단독 항목 페이지가 발생.
            //   - 현재 pi 가 빈 문단이면: skip (한컴은 표시하지 않음)
            //   - 현재 pi 가 일반 텍스트이면: fit 안전마진 (10px) 1회 비활성화
            //     (kps-ai pi=317 case: 0.x px 차이로 fit 실패하여 단독 페이지 35 발생)
            // 가드 제외 조건:
            //   - 다음 pi 가 force_page_break (column_type==Page/Section) 인 경우 발동 안 함
            //     (정상 쪽나누기 신호 — 단독 페이지 발생 안 함, hwp-multi-001 회귀 차단)
            let next_will_vpos_reset = if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
                let next_para = &paragraphs[para_idx + 1];
                let next_force_break = next_para.column_type == ColumnBreakType::Page
                    || next_para.column_type == ColumnBreakType::Section;
                if next_force_break {
                    false
                } else {
                    let next_first_vpos = next_para.line_segs.first().map(|s| s.vertical_pos);
                    let curr_last_vpos = para.line_segs.last().map(|s| s.vertical_pos);
                    // [Task #470] 다단 섹션에서는 nv == 0 → nv < cl 로 완화 (컬럼 헤더 오프셋).
                    // 단일 단에서는 partial-table split 회귀 (issue #418) 회피 위해 nv == 0 유지.
                    let multi_col = st.col_count > 1;
                    matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl))
                        if (if multi_col { nv < cl } else { nv == 0 }) && cl > 5000)
                }
            } else { false };

            if next_will_vpos_reset {
                // [Task #362] 빈 paragraph 가 표/도형/그림 컨트롤을 포함하면 skip 안 함
                // (kps-ai pi=778 case: 빈 텍스트 + 3x3 wrap=Square 표를 가진 paragraph 가
                //  잘못 skip 되어 표 누락).
                let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
                if is_empty_no_ctrl {
                    // 빈 문단 skip (단독 빈페이지 차단)
                    continue;
                } else {
                    // 일반 텍스트 또는 컨트롤 보유: 안전마진 1회 비활성화 (단독 텍스트 페이지 차단)
                    st.skip_safety_margin_once = true;
                }
            }
            // [Task #362] 어울림(Square wrap) 표 옆 paragraph 흡수.
            // Paginator engine.rs:288-320 동일 시멘틱.
            // 직전에 처리한 Square wrap 표의 (cs, sw) 와 동일한 LINE_SEG 를 가진
            // 후속 paragraph 는 표 옆에 배치되므로 height 소비 없이 wrap_around_paras 에 기록.
            if st.wrap_around_cs >= 0 && !has_table {
                let para_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                let para_sw = para.line_segs.first().map(|s| s.segment_width as i32).unwrap_or(0);
                let is_empty_para = para.text.chars().all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
                    && para.controls.is_empty();
                let any_seg_matches = para.line_segs.iter().any(|s|
                    s.column_start == st.wrap_around_cs && s.segment_width as i32 == st.wrap_around_sw
                );
                let body_w = (page_def.width as i32) - (page_def.margin_left as i32) - (page_def.margin_right as i32);
                let sw0_match = st.wrap_around_sw == 0 && is_empty_para && para_sw > 0
                    && para_sw < body_w / 2;
                // [Task #724] HWP5 변환본 case: anchor host 의 wrap=Square image 위치/폭/margin
                // 으로 expected_cs 정확 계산 후 para_cs 일치 확인. anchor cs=0 (caption-style)
                // 한정 가드. expected_cs = (image_x_offset + width + 2*margin) - body_left.
                let anchor_image_match = if st.wrap_around_cs == 0 {
                    let body_left = page_def.margin_left as i32;
                    let expected_cs_hu = paragraphs.get(st.wrap_around_table_para)
                        .and_then(|p| p.controls.iter().find_map(|c| {
                            let cm = match c {
                                Control::Picture(pic) => Some(&pic.common),
                                Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                                    Some(&pic.common)
                                } else { None },
                                _ => None,
                            };
                            cm.filter(|cm| !cm.treat_as_char
                                && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
                                .map(|cm| cm.horizontal_offset as i32 + cm.width as i32
                                    + 2 * cm.margin.right as i32 - body_left)
                        }))
                        .unwrap_or(0);
                    expected_cs_hu > 0
                        && (para_cs - expected_cs_hu).abs() < 200
                        && para_sw > 0
                        && para_cs + para_sw <= body_w + 200
                } else { false };
                if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
                    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
                    || sw0_match
                    || anchor_image_match {
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
                    let anchor_is_picture = paragraphs.get(st.wrap_around_table_para)
                        .map(|p| p.controls.iter().any(|c| match c {
                            Control::Picture(pic) => !pic.common.treat_as_char,
                            Control::Shape(s) => {
                                if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                                    !pic.common.treat_as_char
                                } else { false }
                            }
                            _ => false,
                        }))
                        .unwrap_or(false);
                    if anchor_is_picture {
                        // Picture anchor: wrap_anchors 등록 + FullParagraph 통과
                        // [Task #722] anchor image 의 outer margin_right (HU) 추출
                        let anchor_margin_right = paragraphs.get(st.wrap_around_table_para)
                            .and_then(|p| p.controls.iter().find_map(|c| {
                                let cm = match c {
                                    Control::Picture(pic) => Some(&pic.common),
                                    Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                                        Some(&pic.common)
                                    } else { None },
                                    _ => None,
                                };
                                cm.filter(|cm| !cm.treat_as_char
                                    && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
                                    .map(|cm| cm.margin.right as i32)
                            })).unwrap_or(0);
                        st.current_column_wrap_anchors.insert(
                            para_idx,
                            crate::renderer::pagination::WrapAnchorRef {
                                anchor_para_index: st.wrap_around_table_para,
                                anchor_cs: st.wrap_around_cs,
                                anchor_sw: st.wrap_around_sw,
                                anchor_image_margin_right: anchor_margin_right,
                            },
                        );
                    } else {
                        // Table anchor: 어울림 문단을 표 옆에 기록 + height 소비 없음
                        st.current_column_wrap_around_paras.push(
                            crate::renderer::pagination::WrapAroundPara {
                                para_index: para_idx,
                                table_para_index: st.wrap_around_table_para,
                                has_text: !is_empty_para,
                            }
                        );
                        continue;
                    }
                } else {
                    // 매칭 실패 → wrap zone 종료, 정상 처리 진행
                    st.wrap_around_cs = -1;
                    st.wrap_around_sw = -1;
                    st.wrap_around_any_seg = false;
                }
            }

            st.ensure_page();

            // [Task #404] heading-orphan 패턴 보정 (vpos 기반).
            // 현재 paragraph 가 누적 height 로는 fit 하지만 HWP vpos 기준 페이지 한계를
            // 넘고, 다음 substantial block(Table/Shape/큰 paragraph)이 잔여 영역에 들어
            // 가지 않을 때 → 현재 paragraph 를 다음 페이지로 push 하여 heading + 후속
            // 블록을 같은 페이지에 배치.
            //
            // 조건 (모두 AND):
            //   A) current_items 비어있지 않음 (페이지 첫 item 자기참조 회피)
            //   B) 단일 단 + wrap-around zone 아님 (multi-column / wrap 의미 차이 회피)
            //   C) 누적 height 로 fit
            //   D) vpos overflow > 1mm (283 HU)
            //   E) 다음 paragraph 의 height 가 substantial (>30px ≈ 8mm) AND 잔여 영역에
            //      들어가지 않음
            //
            // Stage 1 진단 로그 분석으로 false positive 41건 → 1건(pi=83)으로 축소.
            // page_top_vpos 는 current_items 의 첫 item para_index 를 통해 즉시 계산
            // (TypesetState 필드 추적은 typeset_paragraph 내부 페이지 flush 와 동기 안 됨).
            if !st.current_items.is_empty()
                && st.wrap_around_cs < 0
                && st.col_count == 1
            {
                let page_first_para_idx = st.current_items.first().map(|item| match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph { para_index, .. } => *para_index,
                    PageItem::Table { para_index, .. } => *para_index,
                    PageItem::PartialTable { para_index, .. } => *para_index,
                    PageItem::Shape { para_index, .. } => *para_index,
                });
                let page_top_vpos_opt = page_first_para_idx
                    .and_then(|pi| paragraphs.get(pi))
                    .and_then(|p| p.line_segs.first())
                    .map(|s| s.vertical_pos);
                if let (Some(first_seg), Some(page_top_vpos)) =
                    (para.line_segs.first(), page_top_vpos_opt)
                {
                    let body_h_hu =
                        crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
                    let para_h_px: f64 = para
                        .line_segs
                        .iter()
                        .map(|s| {
                            crate::renderer::hwpunit_to_px(
                                s.line_height + s.line_spacing,
                                self.dpi,
                            )
                        })
                        .sum();
                    let para_h_hu = crate::renderer::px_to_hwpunit(para_h_px, self.dpi);
                    // [Task #643] vpos_end 는 마지막 줄의 bottom (vpos + lh) 기준.
                    // para_h_px 누적은 트레일링 line_spacing 까지 포함하여 ~10-12 HU 과대.
                    // HWP 가 페이지 끝에서 트레일링 ls 를 고려하지 않고 lh 만 fit 검사하는
                    // 시멘틱 정합 (pi=39 page 3 fits 케이스).
                    let vpos_end = para
                        .line_segs
                        .last()
                        .map(|s| s.vertical_pos + s.line_height)
                        .unwrap_or(first_seg.vertical_pos + para_h_hu);
                    let page_bottom_vpos = page_top_vpos + body_h_hu;

                    let avail = st.available_height();
                    let current_fits = st.current_height + para_h_px <= avail;
                    let vpos_overflow = vpos_end > page_bottom_vpos + 283; // 1mm 안전여유

                    let next_h_px: f64 = paragraphs
                        .get(para_idx + 1)
                        .map(|p| {
                            p.line_segs
                                .iter()
                                .map(|s| {
                                    crate::renderer::hwpunit_to_px(
                                        s.line_height + s.line_spacing,
                                        self.dpi,
                                    )
                                })
                                .sum::<f64>()
                        })
                        .unwrap_or(0.0);
                    let next_substantial = next_h_px > 30.0;
                    let next_doesnt_fit =
                        st.current_height + para_h_px + next_h_px > avail;

                    if current_fits && vpos_overflow && next_substantial && next_doesnt_fit {
                        st.advance_column_or_new_page();
                    }
                }
            }

            if !has_table {
                // --- 핵심: format → fits → place/split ---
                let formatted = self.format_paragraph(para, composed.get(para_idx), styles);
                let is_last_in_section = para_idx + 1 == paragraphs.len();
                self.typeset_paragraph(&mut st, para_idx, para, &formatted, is_last_in_section);
            } else {
                // 표 문단: Phase 2에서 전환 예정. 현재는 기존 방식 호환용 stub.
                self.typeset_table_paragraph(
                    &mut st, para_idx, para, composed.get(para_idx),
                    styles, measured_tables, page_def,
                );
            }

            // [Task #362] Square wrap 표 처리 후 wrap zone 활성화.
            // Paginator engine.rs:356-372 동일 시멘틱.
            // 후속 paragraph 가 동일 cs/sw 를 가지면 흡수.
            if has_table {
                let has_tac_block = para.controls.iter().any(|c| {
                    matches!(c, Control::Table(t) if t.common.treat_as_char)
                });
                let has_non_tac_table = !has_tac_block;
                if has_non_tac_table {
                    let is_wrap_around = para.controls.iter().any(|c| {
                        if let Control::Table(t) = c {
                            matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                        } else { false }
                    });
                    if is_wrap_around {
                        st.wrap_around_cs = para.line_segs.first()
                            .map(|s| s.column_start)
                            .unwrap_or(0);
                        st.wrap_around_sw = para.line_segs.first()
                            .map(|s| s.segment_width as i32)
                            .unwrap_or(0);
                        st.wrap_around_table_para = para_idx;
                        st.wrap_around_any_seg = false;
                    }
                }
            }
            // 비-TAC Picture/Shape Square wrap: engine.rs:380-397 동일 시멘틱.
            // 그림의 첫 lineseg cs가 0일 수 있어 any_seg_matches 허용 플래그 활성화.
            if !has_table {
                let has_non_tac_pic_square = para.controls.iter().any(|c| {
                    let cm = match c {
                        Control::Picture(p) => Some(&p.common),
                        Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() { Some(&p.common) } else { None },
                        _ => None,
                    };
                    cm.map(|cm| !cm.treat_as_char && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)).unwrap_or(false)
                });
                if has_non_tac_pic_square {
                    let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                    let anchor_sw = para.line_segs.first().map(|s| s.segment_width as i32).unwrap_or(0);
                    if anchor_cs > 0 || anchor_sw > 0 {
                        st.wrap_around_cs = anchor_cs;
                        st.wrap_around_sw = anchor_sw;
                        st.wrap_around_table_para = para_idx;
                        st.wrap_around_any_seg = true;
                        // [Task #722] anchor host paragraph 자체도 wrap_anchors 등록.
                        // LINE_SEG cs/sw 가 wrap zone 으로 인코딩되어 있으면 host paragraph 의
                        // 줄도 image 우측 wrap zone 에 layout 되어야 한다 (한컴 PDF 권위 정합).
                        // 미등록 시 paragraph_layout 의 wrap_anchor 분기 미진입 → col_area
                        // 전체 폭 layout → image 영역 침범 → image z-order 후 그려져 가려짐.
                        //
                        // Case 가드 (Stage 3~5 진단):
                        //   - LINE_SEG ≥ 2 → wrap zone (multi-line)
                        //   - LINE_SEG 1 + caption_room ≤ line_height → wrap zone (image 가
                        //     body_top 자체에 위치 → image 위 caption 영역 없음, 강제 wrap)
                        //   - LINE_SEG 1 + caption_room > line_height → caption-style (자기
                        //     미등록 → col_area 전체 폭 layout, image 위 자유 영역 표시)
                        let body_top_hu = page_def.margin_top as i32;
                        let line_height_hu = para.line_segs.first()
                            .map(|s| s.line_height as i32).unwrap_or(900);
                        let (image_voff_hu, image_margin_right_hu) = para.controls.iter().find_map(|c| {
                            let cm = match c {
                                Control::Picture(p) => Some(&p.common),
                                Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() {
                                    Some(&p.common)
                                } else { None },
                                _ => None,
                            };
                            cm.filter(|cm| !cm.treat_as_char
                                && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
                                .map(|cm| (cm.vertical_offset as i32, cm.margin.right as i32))
                        }).unwrap_or((0, 0));
                        let caption_room_hu = image_voff_hu - body_top_hu;
                        let is_caption_style = para.line_segs.len() == 1
                            && caption_room_hu > line_height_hu;
                        // [PR #732 후속 — exam_science 회귀 가드] image_mr=0 (margin 부재) 이면
                        // 본 환경 OLD 동작 보존 — Task #722 host_self register skip.
                        // 본질: image_mr > 0 인 경우 (한컴 viewer 가 inter-image-text gap 으로
                        // margin 적용) 만 host_self register 가 의미. exam_science p.21/37/60 의
                        // Square wrap picture 는 image_mr=0 (호스트 margin 부재) 이므로 OLD 의
                        // col_area-full-width layout 정합 (line_seg cs=0/sw=실제 wrap zone 인코딩
                        // 으로 한컴 정합 이미 유지). hwp3-sample5.hwp 의 page 8/27/48 (Task #722
                        // 본질 영역) 은 image_mr > 0 으로 가드 통과 → 정합 유지.
                        if !is_caption_style && image_margin_right_hu > 0 {
                            st.current_column_wrap_anchors.insert(
                                para_idx,
                                crate::renderer::pagination::WrapAnchorRef {
                                    anchor_para_index: para_idx,
                                    anchor_cs,
                                    anchor_sw,
                                    anchor_image_margin_right: image_margin_right_hu,
                                },
                            );
                        }
                    }
                }
            }

            // Task #321: col 0 처리 중 body-wide TopAndBottom 표/도형이 발견되면
            // col 1+ advance 시 적용할 current_height 시작값을 미리 등록.
            // layout의 body_wide_reserved와 동일 조건으로 detect.
            if st.col_count > 1 && st.current_column == 0 && st.pending_body_wide_top_reserve == 0.0 {
                let reserve = compute_body_wide_top_reserve_for_para(
                    para, &st.layout, self.dpi,
                );
                if reserve > 0.0 {
                    st.pending_body_wide_top_reserve = reserve;
                }
            }

            // 인라인 컨트롤 처리: 도형/그림/수식/각주 (Paginator engine.rs:509-525 동일)
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                        if !has_table {
                            // [Issue #476] treat_as_char Shape 는 박스가 속한 line 이 라우팅된
                            // 페이지/단에 등록. paragraph 가 페이지 분할되면 이 시점의
                            // st.current_items 는 마지막 페이지 상태이므로, 그대로 push 하면
                            // 박스가 잘못된 페이지에 떠 있게 된다.
                            let is_tac_shape = matches!(ctrl,
                                Control::Shape(s) if s.common().treat_as_char);
                            let routed = if is_tac_shape {
                                crate::renderer::pagination::find_inline_control_target_page(
                                    &st.pages, &st.current_items, para_idx, ctrl_idx, para,
                                )
                            } else {
                                None
                            };
                            let item = PageItem::Shape {
                                para_index: para_idx,
                                control_index: ctrl_idx,
                            };
                            match routed {
                                Some((page_idx, col_idx)) => {
                                    if let Some(page) = st.pages.get_mut(page_idx) {
                                        if let Some(col) = page.column_contents.get_mut(col_idx) {
                                            col.items.push(item);
                                        } else {
                                            st.current_items.push(item);
                                        }
                                    } else {
                                        st.current_items.push(item);
                                    }
                                }
                                None => {
                                    st.current_items.push(item);
                                }
                            }
                            // Task #409 v2: 비-TAC TopAndBottom + vert=Para Picture/Shape 는
                            // layout 에서 picture_footnote.rs:356 의 `y_offset + total_height`
                            // 패턴으로 후속 콘텐츠를 개체 높이만큼 밀어냄. 하지만 paragraph
                            // line_seg 의 lh 는 텍스트 baseline 만 반영하므로 페이지네이션의
                            // current_height 가 개체 높이만큼 부족하게 누적되어 page packing
                            // 시 layout 실제 y 와 어긋남 (21페이지: pagination used=803px vs
                            // layout y=1275px → pi=192 가 21페이지에 packing 되었다가
                            // overflow 로 잘림). pagination 측에서도 layout 과 동일하게
                            // 개체 높이를 current_height 에 누적.
                            use crate::model::shape::{TextWrap, VertRelTo};
                            let pushdown_h: Option<f64> = match ctrl {
                                Control::Picture(pic) if !pic.common.treat_as_char
                                    && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                                    && matches!(pic.common.vert_rel_to, VertRelTo::Para) => {
                                    let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    let mb = hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
                                    Some(h + mb)
                                }
                                Control::Shape(s) if !s.common().treat_as_char
                                    && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
                                    && matches!(s.common().vert_rel_to, VertRelTo::Para) => {
                                    let cm = s.common();
                                    let h = hwpunit_to_px(cm.height as i32, self.dpi);
                                    let mb = hwpunit_to_px(cm.margin.bottom as i32, self.dpi);
                                    Some(h + mb)
                                }
                                _ => None,
                            };
                            if let Some(extra) = pushdown_h {
                                st.current_height += extra;
                            }
                        }
                    }
                    Control::Footnote(fn_ctrl) => {
                        if !has_table {
                            if let Some(page) = st.pages.last_mut() {
                                page.footnotes.push(FootnoteRef {
                                    number: fn_ctrl.number,
                                    source: FootnoteSource::Body {
                                        para_index: para_idx,
                                        control_index: ctrl_idx,
                                    },
                                });
                            }
                            let fn_height = Self::estimate_footnote_height(fn_ctrl, self.dpi);
                            st.add_footnote_height(fn_height);
                        }
                    }
                    _ => {}
                }
            }
        }

        // 마지막 항목 처리
        if !st.current_items.is_empty() {
            st.flush_column_always();
        }
        st.ensure_page();

        // 페이지 번호 + 머리말/꼬리말 할당
        Self::finalize_pages(
            &mut st.pages, &hf_entries, &page_number_pos,
            &new_page_numbers, &page_hides, section_index,
        );

        PaginationResult { pages: st.pages, wrap_around_paras: Vec::new(), hidden_empty_paras: st.hidden_empty_paras }
    }

    // ========================================================
    // format: 문단의 실제 높이를 계산한다
    // ========================================================

    /// 문단의 렌더링 높이를 계산한다 (format).
    /// 기존 HeightMeasurer::measure_paragraph()와 동일한 로직.
    fn format_paragraph(
        &self,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
    ) -> FormattedParagraph {
        let para_style_id = composed.map(|c| c.para_style_id as usize).unwrap_or(0);
        let para_style = styles.para_styles.get(para_style_id);
        let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
        let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

        let ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
        let ls_type = para_style.map(|s| s.line_spacing_type)
            .unwrap_or(crate::model::style::LineSpacingType::Percent);

        let (line_heights, line_spacings): (Vec<f64>, Vec<f64>) = if let Some(comp) = composed {
            comp.lines.iter()
                .map(|line| {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let max_fs = line.runs.iter()
                        .map(|r| {
                            styles.char_styles.get(r.char_style_id as usize)
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        })
                        .fold(0.0f64, f64::max);
                    let lh = if max_fs > 0.0 && raw_lh < max_fs {
                        use crate::model::style::LineSpacingType;
                        let computed = match ls_type {
                            LineSpacingType::Percent   => max_fs * ls_val / 100.0,
                            LineSpacingType::Fixed     => ls_val.max(max_fs),
                            LineSpacingType::SpaceOnly => max_fs + ls_val,
                            LineSpacingType::Minimum   => ls_val.max(max_fs),
                        };
                        computed.max(max_fs)
                    } else {
                        raw_lh
                    };
                    (lh, hwpunit_to_px(line.line_spacing, self.dpi))
                })
                .unzip()
        } else if !para.line_segs.is_empty() {
            para.line_segs.iter()
                .map(|seg| (
                    hwpunit_to_px(seg.line_height, self.dpi),
                    hwpunit_to_px(seg.line_spacing, self.dpi),
                ))
                .unzip()
        } else {
            (vec![hwpunit_to_px(400, self.dpi)], vec![0.0])
        };

        let lines_total: f64 = line_heights.iter().zip(line_spacings.iter())
            .map(|(h, s)| h + s)
            .sum();
        let total_height = spacing_before + lines_total + spacing_after;

        // 적합성 판단용: trailing line_spacing 제외
        let trailing_ls = line_spacings.last().copied().unwrap_or(0.0);
        let height_for_fit = (total_height - trailing_ls).max(0.0);

        FormattedParagraph {
            total_height,
            line_heights,
            line_spacings,
            spacing_before,
            spacing_after,
            height_for_fit,
        }
    }

    // ========================================================
    // fits + place/split: 배치 판단과 실행
    // ========================================================

    /// 문단을 현재 페이지에 배치한다.
    /// fits → place(전체) 또는 split(줄 단위) → move(다음 페이지)
    fn typeset_paragraph(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        fmt: &FormattedParagraph,
        is_last_in_section: bool,
    ) {
        // Task #332 Stage 4a: layout drift 안전 마진.
        // typeset 의 fit 추정과 layout 의 실측 진행은 폰트 메트릭/표 측정 다중성 등으로
        // 미세하게 어긋날 수 있다 (~수 px). 마진을 빼서 보수적으로 fit 을 판정해
        // layout 시점의 LAYOUT_OVERFLOW (clamp pile 트리거) 를 사전 차단한다.
        // [Task #359] 다음 pi 가 vpos-reset 가드 발동 예정 시 안전마진 1회 비활성화
        // (단독 항목 페이지 차단).
        // [Task #361] 직전 항목이 PartialTable 인 경우 안전마진 비활성화.
        // PartialTable 의 cur_h 는 row 단위로 정확히 누적되므로 안전마진이 과함.
        // (k-water-rfp p15 case: PartialTable 직후 작은 텍스트 (16px) 가 잔여 5.3px 부족으로
        // fit 실패하여 다음 페이지로 밀리는 회귀.)
        // [Task #643] VPOS_CORR 백워드 허용 (8px) 으로 layout drift 누적이 해소됨.
        // 트레일링 ls 누적 fit 산식 정정과 함께 안전마진 10 → 4 축소.
        const LAYOUT_DRIFT_SAFETY_PX: f64 = 4.0;
        let prev_is_partial_table = matches!(
            st.current_items.last(),
            Some(PageItem::PartialTable { .. })
        );
        let safety = if st.skip_safety_margin_once {
            st.skip_safety_margin_once = false;
            0.0
        } else if prev_is_partial_table {
            0.0
        } else {
            LAYOUT_DRIFT_SAFETY_PX
        };
        let available = (st.available_height() - safety).max(0.0);

        // Task #321 Stage 1 진단: 포맷터 총 높이 vs LINE_SEG 실측 총 높이 비교
        // Stage 5a 확장: per-paragraph 카테고리 분해 (sb/sa/lines/line_sum/ls_sum)
        if std::env::var("RHWP_TYPESET_DRIFT").is_ok() {
            let vpos_h: Option<f64> = if let (Some(first), Some(last)) = (para.line_segs.first(), para.line_segs.last()) {
                let span_hu = (last.vertical_pos + last.line_height) - first.vertical_pos;
                if span_hu > 0 { Some(crate::renderer::hwpunit_to_px(span_hu, self.dpi)) } else { None }
            } else { None };
            let first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
            let last_vpos = para.line_segs.last().map(|s| s.vertical_pos).unwrap_or(-1);
            let lh_sum: f64 = fmt.line_heights.iter().sum();
            let ls_sum: f64 = fmt.line_spacings.iter().sum();
            let line_count = fmt.line_heights.len();
            let trailing_ls = fmt.line_spacings.last().copied().unwrap_or(0.0);
            let diff = match vpos_h {
                Some(v) => fmt.total_height - v,
                None => 0.0,
            };
            let vpos_h_str = vpos_h.map(|v| format!("{:.1}", v)).unwrap_or_else(|| "-".to_string());
            eprintln!(
                "TYPESET_DRIFT_PI: pi={} col={} sb={:.1} sa={:.1} lines={} lh_sum={:.1} ls_sum={:.1} trail_ls={:.1} fmt_total={:.1} vpos_h={} diff={:+.1} first_vpos={} last_vpos={} cur_h={:.1} avail={:.1}",
                para_idx, st.current_column, fmt.spacing_before, fmt.spacing_after,
                line_count, lh_sum, ls_sum, trailing_ls,
                fmt.total_height, vpos_h_str, diff,
                first_vpos, last_vpos,
                st.current_height, available,
            );

            // 옵션: per-line 분해 (LINE_SEG 와 비교)
            if std::env::var("RHWP_TYPESET_DRIFT_LINES").is_ok() {
                for (li, (lh, ls)) in fmt.line_heights.iter().zip(fmt.line_spacings.iter()).enumerate() {
                    let seg = para.line_segs.get(li);
                    let seg_lh = seg.map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi)).unwrap_or(-1.0);
                    let seg_ls = seg.map(|s| crate::renderer::hwpunit_to_px(s.line_spacing, self.dpi)).unwrap_or(-1.0);
                    let seg_vpos = seg.map(|s| s.vertical_pos).unwrap_or(-1);
                    eprintln!(
                        "TYPESET_DRIFT_LINE: pi={} li={} fmt_lh={:.1} fmt_ls={:.1} seg_lh={:.1} seg_ls={:.1} vpos={}",
                        para_idx, li, lh, ls, seg_lh, seg_ls, seg_vpos,
                    );
                }
            }
        }

        // 다단 레이아웃에서 문단 내 단 경계 감지
        // [Task #459] on_first_multicolumn_page 가드 제거: 다단 구역이 여러 페이지에 걸칠 때
        // 후속 페이지에서도 LINE_SEG vpos-reset 으로 인코딩된 단 경계를 인식해야 함.
        let col_breaks = if st.col_count > 1 && st.current_column == 0 {
            Self::detect_column_breaks_in_paragraph(para)
        } else {
            vec![0]
        };

        if col_breaks.len() > 1 {
            self.typeset_multicolumn_paragraph(st, para_idx, para, fmt, &col_breaks);
            return;
        }

        // [Task #362] 한컴 빈 줄 감추기 (SectionDef bit 19, hide_empty_line):
        // 빈 paragraph 가 현재 공간을 overflow 시키면 height=0 으로 처리 (페이지 당 최대 2개).
        // Paginator (engine.rs:85-106) 와 동일 시멘틱.
        // (kps-ai p67~70 case: PartialTable 후속 빈 paragraphs 가 다수 발생, 한컴은 표시 안 함.)
        if st.hide_empty_line {
            let current_page_idx = st.pages.len();
            if current_page_idx != st.hidden_empty_page_idx {
                st.hidden_empty_lines = 0;
                st.hidden_empty_page_idx = current_page_idx;
            }
            let trimmed = para.text.replace(|c: char| c.is_control(), "");
            let is_empty_para = trimmed.trim().is_empty() && para.controls.is_empty();
            if is_empty_para
                && !st.current_items.is_empty()
                && st.current_height + fmt.height_for_fit > available
                && st.hidden_empty_lines < 2
            {
                st.hidden_empty_lines += 1;
                st.hidden_empty_paras.insert(para_idx);
                // height=0 으로 page 진행 — fit 분기에서 추가 처리하지 않음
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
                return;
            }
        }

        // [Task #676] trailing empty paragraph 가드 (단단 전용):
        // 섹션 마지막 빈 paragraph 가 LAYOUT_DRIFT_SAFETY_PX(10px) 영역 내 미세 overflow 로
        // fit 실패 시 height=0 흡수 — 단독 빈 페이지 차단. 한컴2022 정합 시멘틱.
        // (통합재정통계 2010.11/2011.10: pi=14 cur_h=751.0 + 16.0 = 767.0 > avail 766.2,
        //  overflow=0.8px ≤ safety_margin 10px → 흡수.)
        // hide_empty_line (Task #362) 분기와 달리 SectionDef bit 무관, 섹션 마지막 1개만 흡수.
        if is_last_in_section
            && st.col_count == 1
            && !st.current_items.is_empty()
        {
            let trimmed = para.text.replace(|c: char| c.is_control(), "");
            let is_empty_para = trimmed.trim().is_empty() && para.controls.is_empty();
            if is_empty_para {
                let total_h = st.current_height + fmt.height_for_fit;
                let fit_fail_within_safety =
                    total_h > available && total_h <= available + LAYOUT_DRIFT_SAFETY_PX;
                if fit_fail_within_safety {
                    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
                    return;
                }
            }
        }

        // fits: 문단 전체가 현재 공간에 들어가는가?
        // [Task #359] fit 판정은 height_for_fit (trailing_ls 제외) 으로,
        // 누적은 total_height (full) 로 분리. 각 항목별 trailing_ls 가
        // 누적에서 빠지면 N items 누적 시 N × trailing_ls 만큼 drift 발생
        // (k-water-rfp p3 case: 36 items × 평균 ~9px = ~311px LAYOUT_OVERFLOW).
        // trailing_ls 는 페이지 마지막 항목의 fit 판정에만 의미가 있음
        // (페이지 끝에는 다음 줄이 없으니 line_spacing 미적용).
        if st.current_height + fmt.height_for_fit <= available {
            // place: 전체 배치
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            // [Task #391] 다단/단단 분기:
            //   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
            //   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
            // 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
            // typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
            st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
            return;
        }

        // [Task #409 v3] atomic TAC top-fit:
        // 단일 라인 + TAC Picture/Shape (분할 불가능) 항목은 시작점이 본문 안이면
        // 현재 페이지에 배치하고 하단 일부는 하단 여백 (15mm) 으로 흘림 허용.
        // HWP 시멘틱 — atomic 항목은 strict bottom-fit 대신 top-fit 으로 판정.
        // (대상 샘플 23페이지 차트 pi=208: lh=316px, 시작 y=721.4 < 1028(본문 끝),
        //  끝 y=1037.4 가 9.4px 초과하지만 하단 여백 56.7px 안이므로 HWP 가 23페이지 배치.)
        let is_atomic_tac_singleton = fmt.line_heights.len() == 1
            && para.controls.iter().any(|c| match c {
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            });
        if is_atomic_tac_singleton
            && st.current_height < available
            && !st.current_items.is_empty()
        {
            // 추가 가드: 본문 + 하단 여백 안에 들어가야 함 (footer 침범 금지)
            let bottom_margin_px = hwpunit_to_px(
                st.layout.body_area.height as i32, // body_area.height 는 이미 px
                self.dpi,
            );
            // 보수적 tolerance: 1mm (약 3.78px) 이상 ~ 하단 여백 끝까지 허용
            // body_area.height 가 px 이므로 직접 비교 — base_available_height 와의
            // 차이는 footnote_area 만 (본 케이스 0). bottom_margin 은 PageDef 에서
            // 가져와야 하나 직접 접근 어려우므로 1mm 이상 ~ 60px 정도까지 허용.
            let _ = bottom_margin_px; // (위 변수는 향후 정밀화용 — 현재 사용 안 함)
            let overflow = st.current_height + fmt.height_for_fit - available;
            // 60px 이내 초과 (대략 하단 여백 1.6cm 까지 허용; HWP 표준 15mm 여백 안)
            if overflow <= 60.0 {
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
                st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
                return;
            }
        }

        // split: 줄 단위 분할
        let line_count = fmt.line_heights.len();
        if line_count == 0 {
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            // [Task #391] 다단/단단 분기:
            //   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
            //   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
            // 다단에서는 layout 이 vpos 기반으로 항목을 단별로 stacking 하므로
            // typeset 누적 시 trailing_ls 인플레이션이 단을 조기 종료시킴.
            st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
            return;
        }

        // Task #332 Stage 4a: partial split 시에도 동일 마진 적용
        let base_available = (st.base_available_height() - LAYOUT_DRIFT_SAFETY_PX).max(0.0);

        // 남은 공간이 없거나 첫 줄도 못 넣으면 먼저 다음 단/페이지로
        let first_line_h = fmt.line_heights[0];
        let remaining = (available - st.current_height).max(0.0);
        if (st.current_height >= available || remaining < first_line_h)
            && !st.current_items.is_empty()
        {
            st.advance_column_or_new_page();
        }

        // 줄 단위 분할 루프
        let mut cursor_line: usize = 0;
        while cursor_line < line_count {
            let fn_margin = if st.current_footnote_height > 0.0 {
                st.footnote_safety_margin
            } else {
                0.0
            };
            let page_avail = if cursor_line == 0 {
                (base_available - st.current_footnote_height - fn_margin
                    - st.current_height - st.current_zone_y_offset).max(0.0)
            } else {
                base_available
            };

            let sp_b = if cursor_line == 0 { fmt.spacing_before } else { 0.0 };
            // Task #332 Stage 4b: partial split 의 줄 단위 fit 검사에도 layout drift 마진 적용
            let avail_for_lines = (page_avail - sp_b - LAYOUT_DRIFT_SAFETY_PX).max(0.0);

            // 현재 페이지에 들어갈 줄 범위 결정
            let mut cumulative = 0.0;
            let mut end_line = cursor_line;
            for li in cursor_line..line_count {
                // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
                // line_segs[li].vertical_pos == 0 (li>0) 은 HWP 가 해당 line 을
                // 다음 단/페이지 최상단에 배치하도록 인코딩한 신호.
                // 다단 한정 적용 — 단일 단은 partial-table split 회귀 (issue #418) 차단 위해 미적용.
                if st.col_count > 1
                    && li > cursor_line
                    && para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)
                {
                    break;
                }
                let content_h = fmt.line_heights[li];
                if cumulative + content_h > avail_for_lines && li > cursor_line {
                    // [Task #631] HWP 권위값 더블체크
                    // 누적 추정으로는 fit 실패하지만 HWP 파일 자체가 다음 줄(li+1)에
                    // vpos-reset(=0) 을 인코딩한 경우, 한컴 엔진이 직접 li 까지를 현재
                    // 페이지에 배치한 것이다. typeset 보수 마진(20px) 으로 인한 콘텐츠
                    // 손실을 차단하기 위해 HWP 신호를 우선한다.
                    // 조건: (1) 다음 줄의 vpos==0 (페이지 경계 신호)
                    //       (2) 현재 줄의 hwp 좌표 vpos+lh 가 body_available 안
                    let hwp_authoritative = para.line_segs.get(li + 1)
                        .map(|next| next.vertical_pos == 0)
                        .unwrap_or(false)
                        && para.line_segs.get(li).map(|cur| {
                            let bottom_px = crate::renderer::hwpunit_to_px(
                                cur.vertical_pos + cur.line_height, self.dpi);
                            bottom_px <= st.base_available_height()
                        }).unwrap_or(false);
                    if !hwp_authoritative {
                        break;
                    }
                }
                cumulative += fmt.line_advance(li);
                end_line = li + 1;
            }

            if end_line <= cursor_line {
                end_line = cursor_line + 1;
            }

            let part_line_height = fmt.line_advances_sum(cursor_line..end_line);
            let part_sp_after = if end_line >= line_count { fmt.spacing_after } else { 0.0 };
            let part_height = sp_b + part_line_height + part_sp_after;

            if cursor_line == 0 && end_line >= line_count {
                // 전체가 배치됨 — overflow 재확인
                let prev_is_table = st.current_items.last().map_or(false, |item| {
                    matches!(item, PageItem::Table { .. } | PageItem::PartialTable { .. })
                });
                let overflow_threshold = if prev_is_table {
                    let trailing_ls = fmt.line_spacings.get(end_line.saturating_sub(1)).copied().unwrap_or(0.0);
                    cumulative - trailing_ls
                } else {
                    cumulative
                };
                if overflow_threshold > avail_for_lines && !st.current_items.is_empty() {
                    st.advance_column_or_new_page();
                    continue;
                }
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
            } else {
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: cursor_line,
                    end_line,
                });
            }
            st.current_height += part_height;

            if end_line >= line_count {
                break;
            }

            // move: 나머지 줄 → 다음 단/페이지
            st.advance_column_or_new_page();
            cursor_line = end_line;
        }
    }

    // ========================================================
    // Phase 2: Break Token 기반 표 조판
    // ========================================================

    /// 단일 각주의 높이를 추정한다 (HeightMeasurer::estimate_single_footnote_height 동일).
    fn estimate_footnote_height(footnote: &crate::model::footnote::Footnote, dpi: f64) -> f64 {
        let mut fn_height = 0.0;
        for para in &footnote.paragraphs {
            if para.line_segs.is_empty() {
                fn_height += hwpunit_to_px(400, dpi);
            } else {
                for seg in &para.line_segs {
                    fn_height += hwpunit_to_px(seg.line_height, dpi);
                }
            }
        }
        if fn_height <= 0.0 {
            fn_height = hwpunit_to_px(400, dpi);
        }
        fn_height
    }

    /// 표의 조판 높이를 계산한다 (format 단계).
    /// MeasuredTable + host_spacing을 통합하여 layout과 동일한 규칙으로 계산.
    fn format_table(
        &self,
        para: &Paragraph,
        para_idx: usize,
        ctrl_idx: usize,
        table: &crate::model::table::Table,
        measured_tables: &[MeasuredTable],
        styles: &ResolvedStyleSet,
        composed: Option<&ComposedParagraph>,
        is_column_top: bool,
    ) -> FormattedTable {
        let mt = measured_tables.iter().find(|mt|
            mt.para_index == para_idx && mt.control_index == ctrl_idx
        );

        let is_tac = table.attr & 0x01 != 0;
        let table_text_wrap = (table.attr >> 21) & 0x07;

        // host_spacing 계산 — layout과 동일한 규칙
        let para_style_id = composed.map(|c| c.para_style_id as usize)
            .unwrap_or(para.para_shape_id as usize);
        let para_style = styles.para_styles.get(para_style_id);
        let sb = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
        let sa = para_style.map(|s| s.spacing_after).unwrap_or(0.0);

        let outer_top = if is_tac {
            hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
        } else {
            0.0
        };
        let outer_bottom = if is_tac {
            hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
        } else {
            0.0
        };

        // 비-TAC 표: 호스트 문단의 trailing line_spacing도 포함
        let host_line_spacing = if !is_tac {
            para.line_segs.last()
                .filter(|seg| seg.line_spacing > 0)
                .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // spacing_before 조건부 적용
        // - 자리차지(text_wrap=1) 비-TAC 표: spacing_before 제외
        //   (layout에서 v_offset 기반 절대 위치로 배치)
        // - 단 상단: spacing_before 제외
        let before = if !is_tac && table_text_wrap == 1 {
            outer_top
        } else {
            (if !is_column_top { sb } else { 0.0 }) + outer_top
        };
        let after = sa + outer_bottom + host_line_spacing;
        let host_spacing = HostSpacing { before, after, spacing_after_only: sa };

        let (row_heights, cell_spacing, effective_height, caption_height,
             cumulative_heights, page_break, cells, header_row_count) = if let Some(mt) = mt {
            let hrc = if mt.repeat_header && mt.has_header_cells { 1 } else { 0 };
            (
                mt.row_heights.clone(),
                mt.cell_spacing,
                mt.total_height,
                mt.caption_height,
                mt.cumulative_heights.clone(),
                mt.page_break,
                mt.cells.clone(),
                hrc,
            )
        } else {
            (Vec::new(), 0.0, 0.0, 0.0, vec![0.0], Default::default(), Vec::new(), 0)
        };

        let total_height = effective_height + host_spacing.before + host_spacing.after;

        // 표 셀 내 각주 높이 사전 계산 (Paginator engine.rs:565-581 동일)
        let mut table_footnote_height = 0.0;
        let mut table_has_footnotes = false;
        for cell in &table.cells {
            for cp in &cell.paragraphs {
                for cc in &cp.controls {
                    if let Control::Footnote(fn_ctrl) = cc {
                        let fn_height = Self::estimate_footnote_height(fn_ctrl, self.dpi);
                        if !table_has_footnotes {
                            // 첫 각주 시 구분선 오버헤드 추가 여부는 호출 시점의 상태에 의존
                            // 여기서는 순수 각주 높이만 누적 (구분선은 typeset_block_table에서 처리)
                        }
                        table_footnote_height += fn_height;
                        table_has_footnotes = true;
                    }
                }
            }
        }

        FormattedTable {
            row_heights,
            cell_spacing,
            header_row_count,
            host_spacing,
            effective_height,
            total_height,
            caption_height,
            is_tac,
            cumulative_heights,
            page_break,
            cells,
            table_footnote_height,
        }
    }

    /// 표가 포함된 문단을 처리한다.
    /// 각 컨트롤(표/도형)에 대해 format → fits → place/split 패턴 적용.
    fn typeset_table_paragraph(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        composed: Option<&ComposedParagraph>,
        styles: &ResolvedStyleSet,
        measured_tables: &[MeasuredTable],
        _page_def: &PageDef,
    ) {
        // 호스트 문단 format (TAC 표의 높이 보정용)
        let fmt = self.format_paragraph(para, composed, styles);

        // TAC 표 카운트 및 플러시 판단
        let tac_count = para.controls.iter()
            .filter(|c| matches!(c, Control::Table(t) if t.attr & 0x01 != 0))
            .count();

        let has_tac = tac_count > 0;
        let height_for_fit = if has_tac { fmt.height_for_fit } else { fmt.total_height };

        // 넘치면 flush (단일 TAC 표만)
        if st.current_height + height_for_fit > st.available_height()
            && !st.current_items.is_empty()
            && has_tac
            && tac_count <= 1
        {
            st.advance_column_or_new_page();
        }

        st.ensure_page();

        let height_before = st.current_height;
        let page_count_before = st.pages.len();

        // 각 컨트롤에 대해 format → fits → place/split
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Table(table) => {
                    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
                    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
                    // current_height 누적에 영향을 주지 않는다.
                    if matches!(
                        table.common.text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText
                    ) {
                        st.current_items.push(PageItem::Shape {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                        continue;
                    }
                    let is_column_top = st.current_height < 1.0;
                    let ft = self.format_table(
                        para, para_idx, ctrl_idx, table,
                        measured_tables, styles, composed, is_column_top,
                    );

                    let mt = measured_tables.iter().find(|mt|
                        mt.para_index == para_idx && mt.control_index == ctrl_idx);
                    if ft.is_tac {
                        self.typeset_tac_table(st, para_idx, ctrl_idx, para, table, &ft, &fmt, tac_count);
                    } else {
                        self.typeset_block_table(st, para_idx, ctrl_idx, para, table, &ft, &fmt, mt);
                    }

                    // 표 셀 내 각주 수집 (Paginator engine.rs:679-701 동일)
                    for (cell_idx, cell) in table.cells.iter().enumerate() {
                        for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                            for (cc_idx, cc) in cp.controls.iter().enumerate() {
                                if let Control::Footnote(fn_ctrl) = cc {
                                    if let Some(page) = st.pages.last_mut() {
                                        page.footnotes.push(FootnoteRef {
                                            number: fn_ctrl.number,
                                            source: FootnoteSource::TableCell {
                                                para_index: para_idx,
                                                table_control_index: ctrl_idx,
                                                cell_index: cell_idx,
                                                cell_para_index: cp_idx,
                                                cell_control_index: cc_idx,
                                            },
                                        });
                                    }
                                    let fn_height = Self::estimate_footnote_height(fn_ctrl, self.dpi);
                                    st.add_footnote_height(fn_height);
                                }
                            }
                        }
                    }
                }
                Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                    // Task #402: 같은 paragraph의 선행 TAC 컨트롤이 있는 TAC 그림은
                    // 자기 line_seg에 위치하므로 그 line의 높이를 페이지 누적에 반영해야 함.
                    // 누락 시 후속 항목이 페이지 끝을 넘어 그려져 겹침/오버플로 발생 (#402).
                    let tac_separate_line_h: Option<f64> = match ctrl {
                        Control::Picture(p) if p.common.treat_as_char => Some(()),
                        Control::Shape(s) if s.common().treat_as_char => Some(()),
                        _ => None,
                    }.and_then(|_| {
                        let prior_tac_count = para.controls.iter().take(ctrl_idx).filter(|c| match c {
                            Control::Table(t) => t.common.treat_as_char,
                            Control::Picture(p) => p.common.treat_as_char,
                            Control::Shape(s) => s.common().treat_as_char,
                            _ => false,
                        }).count();
                        if prior_tac_count == 0 { return None; }
                        para.line_segs.get(prior_tac_count).map(|seg| {
                            let lh = hwpunit_to_px(seg.line_height, self.dpi);
                            let ls_extra = if seg.line_spacing > 0 {
                                hwpunit_to_px(seg.line_spacing, self.dpi)
                            } else { 0.0 };
                            lh + ls_extra
                        })
                    });
                    if let Some(line_h) = tac_separate_line_h {
                        // 자기 line이 현재 페이지에 들어가지 않으면 다음 페이지로 분할
                        if !st.current_items.is_empty()
                            && st.current_height + line_h > st.available_height() + 0.5
                        {
                            st.advance_column_or_new_page();
                        }
                    }
                    st.current_items.push(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                    if let Some(line_h) = tac_separate_line_h {
                        st.current_height += line_h;
                    }
                }
                _ => {}
            }
        }

        // TAC 표 높이 보정 (Paginator engine.rs:123-179 동일)
        if has_tac && fmt.total_height > 0.0 && st.pages.len() == page_count_before {
            let height_added = st.current_height - height_before;
            // tac_seg_total 계산: 각 TAC 표의 max(seg.lh, 실측높이) + ls/2
            let mut tac_seg_total = 0.0;
            let mut tac_idx = 0;
            for (ci, c) in para.controls.iter().enumerate() {
                if let Control::Table(t) = c {
                    if t.attr & 0x01 != 0 {
                        if let Some(seg) = para.line_segs.get(tac_idx) {
                            let seg_lh = hwpunit_to_px(seg.line_height, self.dpi);
                            let mt_h = measured_tables.iter()
                                .find(|mt| mt.para_index == para_idx && mt.control_index == ci)
                                .map(|mt| mt.total_height)
                                .unwrap_or(0.0);
                            let effective_h = seg_lh.max(mt_h);
                            let ls_half = hwpunit_to_px(seg.line_spacing, self.dpi) / 2.0;
                            tac_seg_total += effective_h + ls_half;
                        }
                        tac_idx += 1;
                    }
                }
            }
            let cap = if tac_seg_total > 0.0 {
                let is_col_top = height_before < 1.0;
                let effective_sb = if is_col_top { 0.0 } else { fmt.spacing_before };
                let outer_top: f64 = para.controls.iter()
                    .filter_map(|c| match c {
                        Control::Table(t) if t.attr & 0x01 != 0 =>
                            Some(hwpunit_to_px(t.outer_margin_top as i32, self.dpi)),
                        _ => None,
                    })
                    .sum();
                (effective_sb + outer_top + tac_seg_total).min(fmt.total_height)
            } else {
                fmt.total_height
            };
            if height_added > cap {
                st.current_height = height_before + cap;
            }
        }
    }

    /// TAC(treat_as_char) 표의 조판.
    fn typeset_tac_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        fmt: &FormattedParagraph,
        tac_count: usize,
    ) {
        // 다중 TAC 표: LINE_SEG 기반 개별 높이 계산
        let table_height = if tac_count > 1 {
            let tac_idx = para.controls.iter().take(ctrl_idx)
                .filter(|c| matches!(c, Control::Table(t) if t.attr & 0x01 != 0))
                .count();
            let is_last_tac = tac_idx + 1 == tac_count;
            para.line_segs.get(tac_idx).map(|seg| {
                let line_h = hwpunit_to_px(seg.line_height, self.dpi);
                if is_last_tac {
                    line_h
                } else {
                    line_h + hwpunit_to_px(seg.line_spacing, self.dpi)
                }
            }).unwrap_or(ft.total_height)
        } else if fmt.total_height > 0.0 {
            // 단일 TAC: 호스트 문단의 height_for_fit 사용
            fmt.height_for_fit
        } else {
            ft.total_height
        };

        // TAC 표는 분할하지 않고 통째로 배치
        let available = st.available_height();
        if st.current_height + table_height > available && !st.current_items.is_empty() {
            st.advance_column_or_new_page();
        }

        self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, table_height);
    }

    /// 표를 pre-text/table/post-text와 함께 배치한다 (Paginator place_table_fits 동일).
    fn place_table_with_text(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
        table_total_height: f64,
    ) {
        let vertical_offset = Self::get_table_vertical_offset(table);
        let total_lines = fmt.line_heights.len();
        let pre_table_end_line = if vertical_offset > 0 && !para.text.is_empty() {
            total_lines
        } else {
            0
        };

        // [Task #439] Square wrap (어울림) 표 식별.
        // 어울림 표는 호스트 문단 텍스트와 같은 수직 영역에 배치되므로
        // current_height 누적은 max(host_text, v_off + table) 한 번만.
        // engine.rs:1328 와 동일 시멘틱.
        let is_wrap_around_table = !table.common.treat_as_char
            && matches!(table.common.text_wrap, crate::model::shape::TextWrap::Square);

        // pre-table 텍스트 (첫 번째 표에서만)
        let is_first_table = !para.controls.iter().take(ctrl_idx)
            .any(|c| matches!(c, Control::Table(_)));
        let pre_height: f64 = if pre_table_end_line > 0 && is_first_table {
            let h = fmt.line_advances_sum(0..pre_table_end_line);
            st.current_items.push(PageItem::PartialParagraph {
                para_index: para_idx,
                start_line: 0,
                end_line: pre_table_end_line,
            });
            h
        } else {
            0.0
        };

        // 표 배치
        st.current_items.push(PageItem::Table {
            para_index: para_idx,
            control_index: ctrl_idx,
        });

        // [Task #439] 누적 정책:
        // - Square wrap (어울림): max(pre_height, v_off + table_total)
        //     호스트 텍스트와 표가 같은 y 영역을 공유하므로 더 큰 쪽만 누적.
        // - 그 외 (TopAndBottom 등): pre_height + table_total 합산 (기존 동작).
        if is_wrap_around_table && pre_height > 0.0 {
            let v_off_px = crate::renderer::hwpunit_to_px(vertical_offset as i32, self.dpi);
            let table_bottom = v_off_px + table_total_height;
            st.current_height += pre_height.max(table_bottom);
        } else {
            st.current_height += pre_height + table_total_height;
        }

        // post-table 텍스트
        let is_last_table = !para.controls.iter().skip(ctrl_idx + 1)
            .any(|c| matches!(c, Control::Table(_)));
        let tac_table_count = para.controls.iter()
            .filter(|c| matches!(c, Control::Table(t) if t.attr & 0x01 != 0))
            .count();
        let post_table_start = if table.attr & 0x01 != 0 {
            pre_table_end_line.max(1)
        } else if is_last_table && !is_first_table {
            0
        } else {
            pre_table_end_line
        };
        // 중복 방지: 이전 표가 이미 같은 문단의 pre-text(start_line=0)를 추가했으면 건너뜀
        // (engine.rs:1418-1421 와 동일한 가드 — 다중 TopAndBottom 표 문단에서
        //  같은 line 범위가 두 번 emit되어 본문이 두 번 렌더되는 문제 차단)
        let pre_text_exists = post_table_start == 0 && st.current_items.iter().any(|item| {
            matches!(item, PageItem::PartialParagraph { para_index, start_line, .. }
                if *para_index == para_idx && *start_line == 0)
        });
        let should_add_post_text = is_last_table && tac_table_count <= 1 && !para.text.is_empty() && total_lines > post_table_start && !pre_text_exists;
        if should_add_post_text {
            let post_height: f64 = fmt.line_advances_sum(post_table_start..total_lines);
            st.current_items.push(PageItem::PartialParagraph {
                para_index: para_idx,
                start_line: post_table_start,
                end_line: total_lines,
            });
            st.current_height += post_height;
        }

        // TAC 표: trailing line_spacing 복원 (Paginator place_table_fits:777-783 동일)
        // has_post_text는 tac_table_count와 무관하게 텍스트 줄 존재 여부만 확인
        let is_tac = table.attr & 0x01 != 0;
        let has_post_text = !para.text.is_empty() && total_lines > post_table_start;
        if is_tac && fmt.total_height > fmt.height_for_fit && !has_post_text {
            st.current_height += fmt.total_height - fmt.height_for_fit;
        }
    }

    /// 비-TAC 블록 표의 조판: fits → place / split(Break Token 기반).
    /// 기존 Paginator의 split_table_rows와 동일한 세밀한 분할 로직.
    fn typeset_block_table(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        table: &crate::model::table::Table,
        ft: &FormattedTable,
        fmt: &FormattedParagraph,
        mt: Option<&MeasuredTable>,
    ) {
        // 표 내 각주를 고려한 가용 높이 계산 (Paginator engine.rs:583-586 동일)
        let table_fn_h = ft.table_footnote_height;
        let fn_separator = if table_fn_h > 0.0 && st.is_first_footnote_on_page {
            st.footnote_separator_overhead
        } else {
            0.0
        };
        let total_footnote = st.current_footnote_height + table_fn_h + fn_separator;
        let fn_margin = if total_footnote > 0.0 { st.footnote_safety_margin } else { 0.0 };
        let available = (st.base_available_height() - total_footnote - fn_margin - st.current_zone_y_offset).max(0.0);

        let host_spacing_total = ft.host_spacing.before + ft.host_spacing.after;
        let table_total = ft.effective_height + host_spacing_total;

        // Task #321 v5: Paper-anchored TopAndBottom block 표는 절대 좌표로 그려지므로
        // cur_h advance 에 표 effective_height 를 그대로 더하면 본문 LINE_SEG vpos 와
        // mismatch (= 21_언어 page 1 col 0 의 +76 px drift). 본문 좌표계와 동기화 하기
        // 위해 host paragraph 의 first_vpos 만큼 cur_h 를 미리 jump 하고 표 advance 를
        // 본문 라인 만큼으로 축소.
        use crate::model::shape::{TextWrap, VertRelTo};
        let is_paper_topbottom_block =
            !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Paper);
        if is_paper_topbottom_block && st.current_column == 0 {
            if let Some(first_seg) = para.line_segs.first() {
                let target_y = crate::renderer::hwpunit_to_px(first_seg.vertical_pos as i32, self.dpi);
                // 호스트 본문 lines + 표는 절대 좌표 → cur_h 는 first_vpos + host lines 만 진행.
                let pre_lines_h = fmt.line_advances_sum(0..fmt.line_heights.len());
                if target_y > st.current_height
                    && target_y + pre_lines_h <= available
                {
                    st.current_height = target_y;
                    // table_total = 0: 표 자체는 cur_h advance 에 영향 없음 (Paper-absolute).
                    // 호스트 본문 lines 만 place_table_with_text 가 pre_height 로 추가.
                    self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, 0.0);
                    return;
                }
            }
        }

        // fits: 전체가 현재 페이지에 들어가는가?
        if st.current_height + table_total <= available {
            self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, table_total);
            return;
        }

        // MeasuredTable이 없거나 행이 없으면 강제 배치
        let mt = match mt {
            Some(m) if !m.row_heights.is_empty() => m,
            _ => {
                if !st.current_items.is_empty() {
                    st.advance_column_or_new_page();
                }
                st.current_items.push(PageItem::Table {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                });
                st.current_height += ft.effective_height;
                return;
            }
        };

        let row_count = mt.row_heights.len();
        let cs = mt.cell_spacing;
        let header_row_height = mt.row_heights[0];
        let can_intra_split = !mt.cells.is_empty();
        let base_available = st.base_available_height();
        let table_available = available; // 각주/존 오프셋 차감된 가용 높이

        // 첫 행이 남은 공간보다 크면 다음 페이지로 (인트라-로우 분할 가능성 확인).
        // Task #398: rowspan>1 셀이 행 0의 시작점이면 블록 전체 높이로 판정.
        let remaining_on_page = (table_available - st.current_height).max(0.0);
        let (first_block_start, first_block_end, first_block_h) = if row_count > 0 {
            mt.row_block_for(0)
        } else { (0, 0, 0.0) };
        let first_block_size = first_block_end.saturating_sub(first_block_start);
        let first_block_is_single_row = first_block_size == 1;
        // [Task #474] RowBreak 표는 보호 블록 정책 비적용
        let first_block_protected = !mt.allows_row_break_split()
            && first_block_size >= 2
            && first_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
        // Task #398 v2: 보호 블록(2~3 rows)만 블록 전체 높이로 판정. 큰 rowspan(>3)은 행 단위 분할.
        let split_unit_h = if first_block_protected {
            first_block_h
        } else {
            mt.row_heights.first().copied().unwrap_or(0.0)
        };
        if remaining_on_page < split_unit_h && !st.current_items.is_empty() {
            let first_row_splittable = (first_block_is_single_row || !first_block_protected)
                && can_intra_split
                && mt.is_row_splittable(0);
            let min_content = if first_row_splittable {
                mt.min_first_line_height_for_row(0, 0.0) + mt.max_padding_for_row(0)
            } else {
                f64::MAX
            };
            if !first_row_splittable || remaining_on_page < min_content {
                st.advance_column_or_new_page();
            }
        }

        // 캡션 처리
        let caption_is_top = para.controls.get(ctrl_idx).and_then(|c| {
            if let Control::Table(t) = c {
                t.caption.as_ref().map(|cap|
                    matches!(cap.direction, CaptionDirection::Top))
            } else { None }
        }).unwrap_or(false);

        let host_line_spacing_for_caption = para.line_segs.first()
            .map(|seg| hwpunit_to_px(seg.line_spacing, self.dpi))
            .unwrap_or(0.0);
        let caption_base_overhead = {
            let ch = ft.caption_height;
            if ch > 0.0 {
                let cs_val = para.controls.get(ctrl_idx).and_then(|c| {
                    if let Control::Table(t) = c {
                        t.caption.as_ref().map(|cap| hwpunit_to_px(cap.spacing as i32, self.dpi))
                    } else { None }
                }).unwrap_or(0.0);
                ch + cs_val
            } else {
                0.0
            }
        };
        let caption_overhead = if caption_base_overhead > 0.0 && !caption_is_top {
            caption_base_overhead + host_line_spacing_for_caption
        } else {
            caption_base_overhead
        };

        // 행 단위 + 인트라-로우 분할 루프 (기존 Paginator split_table_rows 동일)
        let mut cursor_row: usize = 0;
        let mut is_continuation = false;
        let mut content_offset: f64 = 0.0;

        while cursor_row < row_count {
            // 이전 분할에서 모든 콘텐츠가 소진된 행은 건너뜀
            if content_offset > 0.0 && can_intra_split
                && mt.remaining_content_for_row(cursor_row, content_offset) <= 0.0
            {
                cursor_row += 1;
                content_offset = 0.0;
                continue;
            }

            let caption_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 && caption_is_top {
                caption_overhead
            } else {
                0.0
            };
            let page_avail = if is_continuation {
                base_available
            } else {
                (table_available - st.current_height - caption_extra).max(0.0)
            };

            let header_overhead = if is_continuation && mt.repeat_header && mt.has_header_cells && row_count > 1 {
                header_row_height + cs
            } else {
                0.0
            };
            let avail_for_rows = (page_avail - header_overhead).max(0.0);

            let effective_first_row_h = if content_offset > 0.0 && can_intra_split {
                mt.effective_row_height(cursor_row, content_offset)
            } else {
                mt.row_heights[cursor_row]
            };

            // 현재 페이지에 들어갈 행 범위 결정 (find_break_row + 인트라-로우)
            let mut end_row = cursor_row;
            let mut split_end_limit: f64 = 0.0;

            {
                const MIN_SPLIT_CONTENT_PX: f64 = 10.0;

                let approx_end_raw = mt.find_break_row(avail_for_rows, cursor_row, effective_first_row_h);
                // Task #398: rowspan 묶음 중간에서 잘리지 않도록 블록 경계로 스냅
                let approx_end = mt.snap_to_block_boundary(approx_end_raw);

                let (cur_b_start, cur_b_end, _) = mt.row_block_for(cursor_row);
                let cur_block_size = cur_b_end.saturating_sub(cur_b_start);
                let cur_block_single = cur_block_size == 1;
                // [Task #474] RowBreak 표는 보호 블록 정책 비적용
                let cur_block_protected = !mt.allows_row_break_split()
                    && cur_block_size >= 2
                    && cur_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
                let cur_can_intra_split = (cur_block_single || !cur_block_protected) && can_intra_split;

                if approx_end <= cursor_row {
                    let r = cursor_row;
                    let splittable = cur_can_intra_split && mt.is_row_splittable(r);
                    if splittable {
                        let padding = mt.max_padding_for_row(r);
                        let avail_content = (avail_for_rows - padding).max(0.0);
                        let total_content = mt.remaining_content_for_row(r, content_offset);
                        let remaining_content = total_content - avail_content;
                        let min_first_line = mt.min_first_line_height_for_row(r, content_offset);
                        if avail_content >= MIN_SPLIT_CONTENT_PX
                            && avail_content >= min_first_line
                            && remaining_content >= MIN_SPLIT_CONTENT_PX
                        {
                            end_row = r + 1;
                            split_end_limit = avail_content;
                        } else {
                            end_row = r + 1;
                        }
                    } else if cur_can_intra_split && effective_first_row_h > avail_for_rows {
                        let padding = mt.max_padding_for_row(r);
                        let avail_content = (avail_for_rows - padding).max(0.0);
                        if avail_content >= MIN_SPLIT_CONTENT_PX {
                            end_row = r + 1;
                            split_end_limit = avail_content;
                        } else {
                            end_row = r + 1;
                        }
                    } else if cur_block_protected {
                        // Task #398: 보호 블록(2~3 rows)이 들어가지 않으면 블록 전체 배치.
                        end_row = cur_b_end;
                    } else {
                        end_row = r + 1;
                    }
                } else if approx_end < row_count {
                    end_row = approx_end;
                    let r = approx_end;
                    let delta = if content_offset > 0.0 && can_intra_split {
                        mt.row_heights[cursor_row] - effective_first_row_h
                    } else {
                        0.0
                    };
                    let range_h = mt.range_height(cursor_row, approx_end) - delta;
                    let remaining_avail = avail_for_rows - range_h;
                    let (next_b_start, next_b_end, _) = mt.row_block_for(r);
                    let next_block_size = next_b_end.saturating_sub(next_b_start);
                    let next_block_single = next_block_size == 1;
                    // [Task #474] RowBreak 표는 보호 블록 정책 비적용
                    let next_block_protected = !mt.allows_row_break_split()
                        && next_block_size >= 2
                        && next_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
                    let next_can_intra_split = (next_block_single || !next_block_protected) && can_intra_split;
                    if next_can_intra_split && mt.is_row_splittable(r) {
                        let row_cs = cs;
                        let padding = mt.max_padding_for_row(r);
                        let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
                        let total_content = mt.remaining_content_for_row(r, 0.0);
                        let remaining_content = total_content - avail_content_for_r;
                        let min_first_line = mt.min_first_line_height_for_row(r, 0.0);
                        // [Task #713] avail_content_for_r 가 한 줄 정도로 너무 작으면 (orphan)
                        // 분할 대신 행 전체를 다음 페이지로 push. 한컴은 페이지 끝의 작은
                        // sliver(예: 17.6 px) 를 두지 않고 행 단위로 이동
                        // (2022 국립국어원 p31 row 8 케이스). 임계값 25 px 는
                        // synam-001 의 정합 분할 (27.3 px) 과 본 결함 (17.6 px) 사이.
                        const MIN_TOP_KEEP_PX: f64 = 25.0;
                        if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
                            && avail_content_for_r >= min_first_line
                            && avail_content_for_r >= MIN_TOP_KEEP_PX
                            && remaining_content >= MIN_SPLIT_CONTENT_PX
                        {
                            end_row = r + 1;
                            split_end_limit = avail_content_for_r;
                        }
                    } else if next_can_intra_split && mt.row_heights[r] > base_available {
                        let row_cs = cs;
                        let padding = mt.max_padding_for_row(r);
                        let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
                        if avail_content_for_r >= MIN_SPLIT_CONTENT_PX {
                            end_row = r + 1;
                            split_end_limit = avail_content_for_r;
                        }
                    }
                } else {
                    end_row = row_count;
                }
            }

            if end_row <= cursor_row {
                end_row = cursor_row + 1;
            }

            // 이 범위의 높이 계산
            let partial_height: f64 = {
                let delta = if content_offset > 0.0 && can_intra_split {
                    mt.row_heights[cursor_row] - effective_first_row_h
                } else {
                    0.0
                };
                if split_end_limit > 0.0 {
                    let complete_range = if end_row > cursor_row + 1 {
                        mt.range_height(cursor_row, end_row - 1) - delta
                    } else {
                        0.0
                    };
                    let split_row = end_row - 1;
                    let split_row_h = split_end_limit + mt.max_padding_for_row(split_row);
                    let split_row_cs = if split_row > cursor_row { cs } else { 0.0 };
                    complete_range + split_row_cs + split_row_h + header_overhead
                } else {
                    mt.range_height(cursor_row, end_row) - delta + header_overhead
                }
            };

            let actual_split_start = content_offset;
            let actual_split_end = split_end_limit;

            // 마지막 파트에 Bottom 캡션 공간 확보
            if end_row >= row_count && split_end_limit == 0.0 && !caption_is_top && caption_overhead > 0.0 {
                let total_with_caption = partial_height + caption_overhead;
                let avail = if is_continuation {
                    (page_avail - header_overhead).max(0.0)
                } else {
                    page_avail
                };
                if total_with_caption > avail {
                    end_row = end_row.saturating_sub(1);
                    if end_row <= cursor_row {
                        end_row = cursor_row + 1;
                    }
                }
            }

            if end_row >= row_count && split_end_limit == 0.0 {
                // 나머지 전부가 현재 페이지에 들어감
                let bottom_caption_extra = if !caption_is_top { caption_overhead } else { 0.0 };
                if cursor_row == 0 && !is_continuation && content_offset == 0.0 {
                    st.current_items.push(PageItem::Table {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                    st.current_height += partial_height + host_spacing_total;
                } else {
                    st.current_items.push(PageItem::PartialTable {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                        start_row: cursor_row,
                        end_row,
                        is_continuation,
                        split_start_content_offset: actual_split_start,
                        split_end_content_limit: 0.0,
                    });
                    // 마지막 fragment: spacing_after만 포함 (Paginator engine.rs:1051 동일)
                    // host_line_spacing과 outer_bottom은 포함하지 않음
                    st.current_height += partial_height + bottom_caption_extra + ft.host_spacing.spacing_after_only;
                }
                break;
            }

            // 중간 fragment 배치
            st.current_items.push(PageItem::PartialTable {
                para_index: para_idx,
                control_index: ctrl_idx,
                start_row: cursor_row,
                end_row,
                is_continuation,
                split_start_content_offset: actual_split_start,
                split_end_content_limit: actual_split_end,
            });
            st.advance_column_or_new_page();

            // 커서 전진
            if split_end_limit > 0.0 {
                let split_row = end_row - 1;
                if split_row == cursor_row {
                    content_offset += split_end_limit;
                } else {
                    content_offset = split_end_limit;
                }
                cursor_row = split_row;
            } else {
                cursor_row = end_row;
                content_offset = 0.0;
            }
            is_continuation = true;
        }
    }

    // ========================================================
    // 다단 문단 처리
    // ========================================================

    /// 다단 레이아웃에서 문단 내 단 경계를 감지한다.
    fn detect_column_breaks_in_paragraph(para: &Paragraph) -> Vec<usize> {
        let mut breaks = vec![0usize];
        if para.line_segs.len() <= 1 {
            return breaks;
        }
        for i in 1..para.line_segs.len() {
            if para.line_segs[i].vertical_pos < para.line_segs[i - 1].vertical_pos {
                breaks.push(i);
            }
        }
        breaks
    }

    /// 다단 문단의 단별 분할
    fn typeset_multicolumn_paragraph(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        fmt: &FormattedParagraph,
        col_breaks: &[usize],
    ) {
        let line_count = fmt.line_heights.len();
        for (bi, &break_start) in col_breaks.iter().enumerate() {
            let break_end = if bi + 1 < col_breaks.len() {
                col_breaks[bi + 1]
            } else {
                line_count
            };

            if break_start >= line_count || break_end > line_count {
                break;
            }

            let part_height = fmt.line_advances_sum(break_start..break_end);

            if break_start == 0 && break_end >= line_count {
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
            } else {
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: break_start,
                    end_line: break_end,
                });
            }
            st.current_height += part_height;

            // 마지막 단이 아니면 다음 단으로 flush
            if bi + 1 < col_breaks.len() {
                st.flush_column();
                if st.current_column + 1 < st.col_count {
                    st.current_column += 1;
                    st.current_height = 0.0;
                }
            }
        }
    }

    // ========================================================
    // 다단 나누기 처리
    // ========================================================

    fn process_multicolumn_break(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        page_def: &PageDef,
    ) {
        st.flush_column();

        let vpos_zone_height = if para_idx > 0 {
            let mut max_vpos_end: i32 = 0;
            for prev_idx in (0..para_idx).rev() {
                if let Some(last_seg) = paragraphs[prev_idx].line_segs.last() {
                    let vpos_end = last_seg.vertical_pos + last_seg.line_height + last_seg.line_spacing;
                    if vpos_end > max_vpos_end {
                        max_vpos_end = vpos_end;
                    }
                    break;
                }
            }
            if max_vpos_end > 0 {
                hwpunit_to_px(max_vpos_end, self.dpi)
            } else {
                st.current_height
            }
        } else {
            st.current_height
        };
        st.current_zone_y_offset += vpos_zone_height;
        st.current_column = 0;
        st.current_height = 0.0;
        st.on_first_multicolumn_page = true;

        for ctrl in &paragraphs[para_idx].controls {
            if let Control::ColumnDef(cd) = ctrl {
                st.col_count = cd.column_count.max(1);
                let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                st.current_zone_layout = Some(new_layout.clone());
                st.layout = new_layout;
                // [Task #702] 새 zone 의 ColumnType 반영. Distribute(배분) 단에서
                // 짧은 컬럼 vpos-reset 검출 임계값 완화용.
                st.current_zone_column_type = cd.column_type;
                break;
            }
        }
    }

    // ========================================================
    // 머리말/꼬리말/쪽 번호 처리
    // ========================================================

    fn collect_header_footer_controls(
        paragraphs: &[Paragraph],
        section_index: usize,
    ) -> (
        Vec<(usize, HeaderFooterRef, bool, HeaderFooterApply)>,
        Option<crate::model::control::PageNumberPos>,
        Vec<(usize, u16)>,
        Vec<(usize, crate::model::control::PageHide)>,
    ) {
        let mut hf_entries = Vec::new();
        let mut page_number_pos = None;
        let mut new_page_numbers = Vec::new();
        let mut page_hides: Vec<(usize, crate::model::control::PageHide)> = Vec::new();

        for (pi, para) in paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Header(h) => {
                        let r = HeaderFooterRef {
                            para_index: pi,
                            control_index: ci,
                            source_section_index: section_index,
                        };
                        hf_entries.push((pi, r, true, h.apply_to));
                    }
                    Control::Footer(f) => {
                        let r = HeaderFooterRef {
                            para_index: pi,
                            control_index: ci,
                            source_section_index: section_index,
                        };
                        hf_entries.push((pi, r, false, f.apply_to));
                    }
                    Control::PageNumberPos(pnp) => {
                        page_number_pos = Some(pnp.clone());
                    }
                    Control::NewNumber(nn) => {
                        if nn.number_type == crate::model::control::AutoNumberType::Page {
                            new_page_numbers.push((pi, nn.number));
                        }
                    }
                    Control::PageHide(ph) => {
                        page_hides.push((pi, ph.clone()));
                    }
                    Control::Table(table) => {
                        Self::collect_pagehide_in_table(table, pi, &mut page_hides);
                    }
                    _ => {}
                }
            }
        }

        (hf_entries, page_number_pos, new_page_numbers, page_hides)
    }

    /// 표 셀 안 paragraph 의 PageHide 를 재귀 수집.
    /// 외부 paragraph index `pi` 를 그대로 사용해 페이지 매핑 정합성 유지.
    fn collect_pagehide_in_table(
        table: &crate::model::table::Table,
        pi: usize,
        page_hides: &mut Vec<(usize, crate::model::control::PageHide)>,
    ) {
        for cell in &table.cells {
            for cp in &cell.paragraphs {
                for ctrl in &cp.controls {
                    match ctrl {
                        Control::PageHide(ph) => {
                            page_hides.push((pi, ph.clone()));
                        }
                        Control::Table(inner) => {
                            Self::collect_pagehide_in_table(inner, pi, page_hides);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// 페이지 번호 + 머리말/꼬리말 최종 할당 (기존 Paginator::finalize_pages와 동일)
    fn finalize_pages(
        pages: &mut [PageContent],
        hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
        page_number_pos: &Option<crate::model::control::PageNumberPos>,
        new_page_numbers: &[(usize, u16)],
        page_hides: &[(usize, crate::model::control::PageHide)],
        _section_index: usize,
    ) {
        // 쪽번호: PageNumberAssigner 가 NewNumber 1회 적용 + 단조 증가를 보장 (Issue #353)
        let mut current_header: Option<HeaderFooterRef> = None;
        let mut current_footer: Option<HeaderFooterRef> = None;
        let mut assigner = crate::renderer::page_number::PageNumberAssigner::new(new_page_numbers, 1);

        for page in pages.iter_mut() {
            let page_num = assigner.assign(page);

            // 이 페이지에 속하는 머리말/꼬리말 갱신
            let page_last_para = page.column_contents.iter()
                .flat_map(|col| col.items.iter())
                .map(|item| match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph { para_index, .. } => *para_index,
                    PageItem::Table { para_index, .. } => *para_index,
                    PageItem::PartialTable { para_index, .. } => *para_index,
                    PageItem::Shape { para_index, .. } => *para_index,
                })
                .max();

            if let Some(last_pi) = page_last_para {
                for (hf_pi, hf_ref, is_header, apply) in hf_entries {
                    if *hf_pi <= last_pi {
                        let applies = match apply {
                            HeaderFooterApply::Both => true,
                            HeaderFooterApply::Even => page_num.is_multiple_of(2),
                            HeaderFooterApply::Odd => page_num % 2 == 1,
                        };
                        if applies {
                            if *is_header {
                                current_header = Some(hf_ref.clone());
                            } else {
                                current_footer = Some(hf_ref.clone());
                            }
                        }
                    }
                }
            }

            page.page_number = page_num;
            page.active_header = current_header.clone();
            page.active_footer = current_footer.clone();
            page.page_number_pos = page_number_pos.clone();

            // PageHide: 해당 문단이 이 페이지에서 **처음** 시작하는 경우만 적용
            // (engine.rs 의 동일 로직과 일치 — 머리말/꼬리말/바탕쪽/페이지번호 감추기)
            for (ph_para, ph) in page_hides {
                let starts = page.column_contents.iter().any(|col| {
                    col.items.iter().any(|item| match item {
                        PageItem::FullParagraph { para_index } => *para_index == *ph_para,
                        PageItem::PartialParagraph { para_index, start_line, .. } =>
                            *para_index == *ph_para && *start_line == 0,
                        PageItem::Table { para_index, .. } => *para_index == *ph_para,
                        PageItem::PartialTable { para_index, .. } => *para_index == *ph_para,
                        PageItem::Shape { para_index, .. } => *para_index == *ph_para,
                    })
                });
                if starts {
                    page.page_hide = Some(ph.clone());
                    break;
                }
            }
        }
    }

    // ========================================================
    // 유틸리티
    // ========================================================

    /// 문단에 블록 표 컨트롤이 있는지 감지
    fn paragraph_has_table(&self, para: &Paragraph) -> bool {
        use crate::renderer::height_measurer::is_tac_table_inline;
        let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
        para.controls.iter().any(|c| {
            matches!(c, Control::Table(t) if t.attr & 0x01 == 0
                || (t.attr & 0x01 != 0 && !is_tac_table_inline(t, seg_width, &para.text, &para.controls)))
        })
    }

    /// 표의 세로 오프셋 추출 (Paginator와 동일).
    ///
    /// `raw_ctrl_data` 의 첫 4바이트는 `attr` 비트 플래그이고 `vertical_offset` 은
    /// 다음 4바이트 (`raw_ctrl_data[4..8]`) 이지만, IR 의 `common.vertical_offset` 가
    /// 파서가 채운 권위 있는 값이므로 이를 직접 사용한다 (#178).
    fn get_table_vertical_offset(table: &crate::model::table::Table) -> u32 {
        table.common.vertical_offset as u32
    }
}

/// Task #321: 단일 문단의 컨트롤에서 body-wide TopAndBottom 표/도형이 차지하는 높이 계산.
///
/// col 1+ advance 시 current_height 시작값으로 사용하여 layout의 `body_wide_reserved`
/// 와 동일한 가용 공간 축소를 적용한다.
///
/// **Paper(용지) 기준 도형 가드 (v3 정밀화 #326)**: vert_rel_to=Paper 인 도형 중
/// 본문 영역과 겹치지 않는(머리말 영역에만 위치하는) 도형만 제외. body 와 겹치는
/// Paper 도형은 col 1 시작에 영향 → reserve 대상으로 포함.
fn compute_body_wide_top_reserve_for_para(
    para: &Paragraph,
    layout: &PageLayoutInfo,
    dpi: f64,
) -> f64 {
    use crate::model::shape::{TextWrap, VertRelTo};
    let body_w = layout.body_area.width;
    let body_h = layout.available_body_height();
    let body_top = layout.body_area.y;
    let mut max_bottom: f64 = 0.0;
    for ctrl in &para.controls {
        let common = match ctrl {
            Control::Shape(s) => s.common(),
            Control::Table(t) if !t.common.treat_as_char => &t.common,
            Control::Picture(p) if !p.common.treat_as_char => &p.common,
            _ => continue,
        };
        if !matches!(common.text_wrap, TextWrap::TopAndBottom) || common.treat_as_char {
            continue;
        }
        let shape_w = crate::renderer::hwpunit_to_px(common.width as i32, dpi);
        if shape_w < body_w * 0.8 {
            continue;
        }
        let shape_h = crate::renderer::hwpunit_to_px(common.height as i32, dpi);
        let raw_v_offset = crate::renderer::hwpunit_to_px(common.vertical_offset as i32, dpi);

        // body-rel 기준 시작/끝 y 계산.
        // - VertRelTo::Paper: vertical_offset 이 용지 상단(= 0) 기준 → body_top 차감.
        //   본문과 전혀 겹치지 않으면(머리말만 점유) 제외.
        //   본문 위쪽으로 일부 빠져나가면(shape_top_abs < body_top) 본문 침범 영역만 reserve.
        // - VertRelTo::Page / Para: vertical_offset 이 본문/단 top 기준 → body-rel 그대로.
        let (body_y, body_bottom) = if matches!(common.vert_rel_to, VertRelTo::Paper) {
            let shape_top_abs = raw_v_offset;
            let shape_bottom_abs = shape_top_abs + shape_h;
            if shape_bottom_abs <= body_top {
                continue;
            }
            ((shape_top_abs - body_top).max(0.0), shape_bottom_abs - body_top)
        } else {
            (raw_v_offset, raw_v_offset + shape_h)
        };

        if body_y > body_h / 3.0 {
            continue;
        }
        let outer_bottom = crate::renderer::hwpunit_to_px(common.margin.bottom as i32, dpi);
        let bottom = body_bottom + outer_bottom;
        if bottom > max_bottom {
            max_bottom = bottom;
        }
    }
    max_bottom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::{Paragraph, LineSeg};
    use crate::model::page::{PageDef, ColumnDef};
    use crate::renderer::composer::ComposedParagraph;
    use crate::renderer::height_measurer::HeightMeasurer;
    use crate::renderer::pagination::Paginator;
    use crate::renderer::style_resolver::ResolvedStyleSet;

    fn a4_page_def() -> PageDef {
        PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            margin_gutter: 0,
            ..Default::default()
        }
    }

    fn make_paragraph_with_height(line_height: i32) -> Paragraph {
        Paragraph {
            line_segs: vec![LineSeg {
                line_height,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// 두 PaginationResult의 페이지 수와 각 페이지의 항목 수가 동일한지 비교
    fn assert_pagination_match(
        old: &PaginationResult,
        new: &PaginationResult,
        label: &str,
    ) {
        assert_eq!(
            old.pages.len(),
            new.pages.len(),
            "{}: 페이지 수 불일치 (old={}, new={})",
            label,
            old.pages.len(),
            new.pages.len(),
        );

        for (pi, (old_page, new_page)) in old.pages.iter().zip(new.pages.iter()).enumerate() {
            assert_eq!(
                old_page.column_contents.len(),
                new_page.column_contents.len(),
                "{}: p{} 단 수 불일치",
                label, pi,
            );

            for (ci, (old_col, new_col)) in old_page.column_contents.iter()
                .zip(new_page.column_contents.iter()).enumerate()
            {
                assert_eq!(
                    old_col.items.len(),
                    new_col.items.len(),
                    "{}: p{} col{} 항목 수 불일치 (old={}, new={})",
                    label, pi, ci,
                    old_col.items.len(),
                    new_col.items.len(),
                );
            }
        }
    }

    #[test]
    fn test_typeset_engine_creation() {
        let engine = TypesetEngine::new(96.0);
        assert_eq!(engine.dpi, 96.0);
    }

    #[test]
    fn test_typeset_empty_paragraphs() {
        let engine = TypesetEngine::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        let result = engine.typeset_section(
            &[], &composed, &styles,
            &a4_page_def(), &ColumnDef::default(), 0, &[], false,
        );

        assert_eq!(result.pages.len(), 1, "빈 문서도 최소 1페이지");
    }

    #[test]
    fn test_typeset_single_paragraph() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let paras = vec![make_paragraph_with_height(400)];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let new_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        assert_pagination_match(&old_result, &new_result, "single_paragraph");
    }

    #[test]
    fn test_typeset_page_overflow() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let paras: Vec<Paragraph> = (0..100)
            .map(|_| make_paragraph_with_height(2000))
            .collect();
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let new_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        assert_pagination_match(&old_result, &new_result, "page_overflow");
    }

    #[test]
    fn test_typeset_line_split() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 여러 줄이 있는 큰 문단 (페이지 경계에서 줄 단위 분할)
        let paras = vec![Paragraph {
            line_segs: (0..50).map(|_| LineSeg {
                line_height: 1800,
                line_spacing: 200,
                ..Default::default()
            }).collect(),
            ..Default::default()
        }];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let new_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        assert_pagination_match(&old_result, &new_result, "line_split");
    }

    #[test]
    fn test_typeset_mixed_paragraphs() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 다양한 높이의 문단 혼합
        let paras: Vec<Paragraph> = vec![
            make_paragraph_with_height(400),
            make_paragraph_with_height(10000),  // 큰 문단
            make_paragraph_with_height(400),
            make_paragraph_with_height(800),
            make_paragraph_with_height(20000),  // 매우 큰 문단
            make_paragraph_with_height(400),
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let new_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        assert_pagination_match(&old_result, &new_result, "mixed_paragraphs");
    }

    #[test]
    fn test_typeset_page_break() {
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();

        // 강제 쪽 나누기가 있는 문단
        let paras = vec![
            make_paragraph_with_height(400),
            {
                let mut p = make_paragraph_with_height(400);
                p.column_type = ColumnBreakType::Page;
                p
            },
            make_paragraph_with_height(400),
        ];
        let composed: Vec<ComposedParagraph> = Vec::new();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();

        let (old_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let new_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        assert_pagination_match(&old_result, &new_result, "page_break");
        assert_eq!(new_result.pages.len(), 2, "쪽 나누기로 2페이지");
    }

    // ========================================================
    // 실제 HWP 파일 비교 테스트
    // ========================================================

    /// 실제 HWP 파일로 기존 Paginator와 TypesetEngine 결과 비교
    fn compare_with_hwp_file(path: &str) {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("skip: {} not found", path);
                return;
            }
        };
        let doc = match crate::document_core::DocumentCore::from_bytes(&data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip: {} parse error: {}", path, e);
                return;
            }
        };

        let engine = TypesetEngine::with_default_dpi();

        for (sec_idx, section) in doc.document.sections.iter().enumerate() {
            let composed = &doc.composed[sec_idx];
            let measured_tables = &doc.measured_tables[sec_idx];
            let column_def = crate::document_core::DocumentCore::find_initial_column_def(
                &section.paragraphs,
            );

            // 구역에 표가 포함되어 있는지 확인
            let has_tables = section.paragraphs.iter().any(|p|
                p.controls.iter().any(|c| matches!(c, Control::Table(_)))
            );

            let new_result = engine.typeset_section(
                &section.paragraphs,
                composed,
                &doc.styles,
                &section.section_def.page_def,
                &column_def,
                sec_idx,
                measured_tables,
                section.section_def.hide_empty_line,
            );

            let old_result = &doc.pagination[sec_idx];
            let label = format!("{} sec{}", path, sec_idx);

            if has_tables {
                // 표가 포함된 구역: Phase 2 전환 전까지 차이 허용 (경고만 출력)
                if old_result.pages.len() != new_result.pages.len() {
                    eprintln!(
                        "WARN {}: 표 포함 구역 페이지 수 차이 (old={}, new={}) — Phase 2에서 해결",
                        label, old_result.pages.len(), new_result.pages.len(),
                    );
                }
            } else {
                // 비-표 구역: 완전 일치 필수
                assert_eq!(
                    old_result.pages.len(),
                    new_result.pages.len(),
                    "{}: 페이지 수 불일치 (old={}, new={})",
                    label, old_result.pages.len(), new_result.pages.len(),
                );

                for (pi, (old_page, new_page)) in old_result.pages.iter()
                    .zip(new_result.pages.iter()).enumerate()
                {
                    assert_eq!(
                        old_page.column_contents.len(),
                        new_page.column_contents.len(),
                        "{}: p{} 단 수 불일치",
                        label, pi,
                    );
                }
            }
        }
    }

    #[test]
    fn test_typeset_vs_paginator_p222() {
        // p222.hwp sec2는 표가 많아 Phase 2 전환 전까지 차이 발생 가능
        // Phase 1에서는 비-표 문단만 검증
        compare_with_hwp_file("samples/p222.hwp");
    }

    #[test]
    fn test_typeset_vs_paginator_hongbo() {
        compare_with_hwp_file("samples/20250130-hongbo.hwp");
    }

    #[test]
    fn test_typeset_vs_paginator_biz_plan() {
        compare_with_hwp_file("samples/biz_plan.hwp");
    }

    /// Issue #703: BehindText/InFrontOfText 표는 본문 흐름에서 제외되어야 한다.
    ///
    /// 글뒤로 (BehindText) / 글앞으로 (InFrontOfText) 표는 시각적으로 본문 텍스트 뒤/앞에
    /// 절대 좌표로 배치되는 데코레이션 (워터마크/배경 등) 이며, 본문 흐름의 vertical advance 에
    /// 영향을 주지 않는다. `pagination/engine.rs:976-981` 와 동일 시멘틱.
    ///
    /// 결함 메커니즘: typeset_block_table → place_table_with_text → `cur_h += table_total_height`
    /// (line 1594) 가 BehindText/InFrontOfText 표에 대해서도 적용되어 본문 흐름 누적이 발생.
    ///
    /// 본 테스트는 BIG BehindText 표 (≈300 mm 높이) 를 1 페이지 본문 안에 넣어두고 후속
    /// paragraph 가 동일 페이지에 들어감을 검증한다. 결함 시 BehindText 표의 거대 height 가
    /// cur_h 에 가산되어 후속 paragraph 가 다음 페이지로 밀림.
    #[test]
    fn test_typeset_703_behind_text_table_no_flow_advance() {
        use crate::model::shape::TextWrap;
        let engine = TypesetEngine::with_default_dpi();
        let paginator = Paginator::with_default_dpi();
        let styles = ResolvedStyleSet::default();
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let composed: Vec<ComposedParagraph> = Vec::new();

        // BehindText 1×1 표: 본문 높이의 약 80% 차지 (60000 HU ≈ 800 px @96dpi).
        // BehindText 는 데코레이션이므로 본문 흐름 누적 0 이어야 정상.
        // 결함 시 cur_h 에 800 px 가산 → 후속 1 단락도 fit 실패 → 페이지 분할.
        let mut table = crate::model::table::Table {
            row_count: 1,
            col_count: 1,
            cells: vec![crate::model::table::Cell {
                col: 0, row: 0, col_span: 1, row_span: 1,
                width: 51974, height: 60000,
                paragraphs: vec![Paragraph::default()],
                ..Default::default()
            }],
            ..Default::default()
        };
        table.common.text_wrap = TextWrap::BehindText;
        table.common.treat_as_char = false;
        table.common.width = 51974;
        table.common.height = 60000; // ≈800 px @96dpi — 본문 80% 점유 (결함 시 가산되는 양)

        let host_para = Paragraph {
            line_segs: vec![LineSeg {
                line_height: 1000, line_spacing: 600,
                ..Default::default()
            }],
            controls: vec![crate::model::control::Control::Table(Box::new(table))],
            ..Default::default()
        };

        // 후속 5 단락 — 본문 정상 흐름이면 호스트(21px) + 5 × 13px = 86 px (1 페이지 여유)
        // 결함 시 호스트(21+800=821px) + 첫 단락(13px) = 834 px 도 fit, 더 추가 시 결국 분할
        // → 단순히 페이지 수 정확히 비교 필요.
        let mut paras = vec![host_para];
        for _ in 0..5 {
            paras.push(make_paragraph_with_height(1000));
        }

        let (paginator_result, measured) = paginator.paginate(
            &paras, &composed, &styles, &page_def, &col_def, 0,
        );
        let typeset_result = engine.typeset_section(
            &paras, &composed, &styles, &page_def, &col_def, 0,
            &measured.tables, false,
        );

        // 검증 1: paginator (engine.rs reference) 는 1 페이지에 모두 배치
        assert_eq!(
            paginator_result.pages.len(), 1,
            "[reference] BehindText 표 + 5 후속 paragraph 는 paginator 에서 1 페이지에 들어가야 함",
        );

        // 검증 2: typeset 결과도 1 페이지 (현재 결함 시 RED — typeset 이 BehindText 표 height 를 누적)
        assert_eq!(
            typeset_result.pages.len(), 1,
            "[BUG #703] typeset 도 1 페이지여야 함. 결함 시 BehindText 표 height ≈800 px 가 \
             cur_h 에 가산되어 후속 paragraph 가 다음 페이지로 밀림 (RED)",
        );
    }
}
