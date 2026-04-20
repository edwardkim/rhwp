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
use crate::model::page::{PageDef, ColumnDef};
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
}

impl TypesetState {
    fn new(
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
        if self.current_items.is_empty() {
            return;
        }
        let col_content = ColumnContent {
            column_index: self.current_column,
            items: std::mem::take(&mut self.current_items),
            zone_layout: self.current_zone_layout.clone(),
            zone_y_offset: self.current_zone_y_offset,
            wrap_around_paras: Vec::new(),
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
            wrap_around_paras: Vec::new(),
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
            self.current_height = 0.0;
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
    ) -> PaginationResult {
        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        let col_count = column_def.column_count.max(1);
        let footnote_separator_overhead = hwpunit_to_px(400, self.dpi);
        let footnote_safety_margin = hwpunit_to_px(3000, self.dpi);

        let mut st = TypesetState::new(
            layout, col_count, section_index,
            footnote_separator_overhead, footnote_safety_margin,
        );

        // 머리말/꼬리말/쪽 번호/새 번호 컨트롤 수집
        let (hf_entries, page_number_pos, new_page_numbers) =
            Self::collect_header_footer_controls(paragraphs, section_index);

        for (para_idx, para) in paragraphs.iter().enumerate() {
            // 표 컨트롤 감지
            let has_table = self.paragraph_has_table(para);

            // 다단 나누기
            if para.column_type == ColumnBreakType::MultiColumn {
                self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
            }

            // 단 나누기
            if para.column_type == ColumnBreakType::Column && !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }

            // 쪽 나누기
            let force_page_break = para.column_type == ColumnBreakType::Page
                || para.column_type == ColumnBreakType::Section;
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let para_style_break = para_style.map(|s| s.page_break_before).unwrap_or(false);

            if (force_page_break || para_style_break) && !st.current_items.is_empty() {
                st.force_new_page();
            }

            st.ensure_page();

            if !has_table {
                // --- 핵심: format → fits → place/split ---
                let formatted = self.format_paragraph(para, composed.get(para_idx), styles);
                self.typeset_paragraph(&mut st, para_idx, para, &formatted);
            } else {
                // 표 문단: Phase 2에서 전환 예정. 현재는 기존 방식 호환용 stub.
                self.typeset_table_paragraph(
                    &mut st, para_idx, para, composed.get(para_idx),
                    styles, measured_tables, page_def,
                );
            }

            // 인라인 컨트롤 처리: 도형/그림/수식/각주 (Paginator engine.rs:509-525 동일)
            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                        if !has_table {
                            st.current_items.push(PageItem::Shape {
                                para_index: para_idx,
                                control_index: ctrl_idx,
                            });
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
            &new_page_numbers, section_index,
        );

        PaginationResult { pages: st.pages, wrap_around_paras: Vec::new(), hidden_empty_paras: std::collections::HashSet::new() }
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
    ) {
        let available = st.available_height();

        // 다단 레이아웃에서 문단 내 단 경계 감지
        let col_breaks = if st.col_count > 1 && st.current_column == 0 && st.on_first_multicolumn_page {
            Self::detect_column_breaks_in_paragraph(para)
        } else {
            vec![0]
        };

        if col_breaks.len() > 1 {
            self.typeset_multicolumn_paragraph(st, para_idx, para, fmt, &col_breaks);
            return;
        }

        // fits: 문단 전체가 현재 공간에 들어가는가?
        if st.current_height + fmt.height_for_fit <= available {
            // place: 전체 배치
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            st.current_height += fmt.total_height;
            return;
        }

        // split: 줄 단위 분할
        let line_count = fmt.line_heights.len();
        if line_count == 0 {
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            st.current_height += fmt.total_height;
            return;
        }

        let base_available = st.base_available_height();

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
            let avail_for_lines = (page_avail - sp_b).max(0.0);

            // 현재 페이지에 들어갈 줄 범위 결정
            let mut cumulative = 0.0;
            let mut end_line = cursor_line;
            for li in cursor_line..line_count {
                let content_h = fmt.line_heights[li];
                if cumulative + content_h > avail_for_lines && li > cursor_line {
                    break;
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
                Control::Picture(_) | Control::Equation(_) => {
                    st.current_items.push(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
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

        // pre-table 텍스트 (첫 번째 표에서만)
        let is_first_table = !para.controls.iter().take(ctrl_idx)
            .any(|c| matches!(c, Control::Table(_)));
        if pre_table_end_line > 0 && is_first_table {
            let pre_height: f64 = fmt.line_advances_sum(0..pre_table_end_line);
            st.current_items.push(PageItem::PartialParagraph {
                para_index: para_idx,
                start_line: 0,
                end_line: pre_table_end_line,
            });
            st.current_height += pre_height;
        }

        // 표 배치
        st.current_items.push(PageItem::Table {
            para_index: para_idx,
            control_index: ctrl_idx,
        });
        st.current_height += table_total_height;

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
        let should_add_post_text = is_last_table && tac_table_count <= 1 && !para.text.is_empty() && total_lines > post_table_start;
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

        // 첫 행이 남은 공간보다 크면 다음 페이지로 (인트라-로우 분할 가능성 확인)
        let remaining_on_page = (table_available - st.current_height).max(0.0);
        let first_row_h = mt.row_heights[0];
        if remaining_on_page < first_row_h && !st.current_items.is_empty() {
            let first_row_splittable = can_intra_split && mt.is_row_splittable(0);
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

                let approx_end = mt.find_break_row(avail_for_rows, cursor_row, effective_first_row_h);

                if approx_end <= cursor_row {
                    let r = cursor_row;
                    let splittable = can_intra_split && mt.is_row_splittable(r);
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
                    } else if can_intra_split && effective_first_row_h > avail_for_rows {
                        let padding = mt.max_padding_for_row(r);
                        let avail_content = (avail_for_rows - padding).max(0.0);
                        if avail_content >= MIN_SPLIT_CONTENT_PX {
                            end_row = r + 1;
                            split_end_limit = avail_content;
                        } else {
                            end_row = r + 1;
                        }
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
                    if can_intra_split && mt.is_row_splittable(r) {
                        let row_cs = cs;
                        let padding = mt.max_padding_for_row(r);
                        let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
                        let total_content = mt.remaining_content_for_row(r, 0.0);
                        let remaining_content = total_content - avail_content_for_r;
                        let min_first_line = mt.min_first_line_height_for_row(r, 0.0);
                        if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
                            && avail_content_for_r >= min_first_line
                            && remaining_content >= MIN_SPLIT_CONTENT_PX
                        {
                            end_row = r + 1;
                            split_end_limit = avail_content_for_r;
                        }
                    } else if can_intra_split && mt.row_heights[r] > base_available {
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
    ) {
        let mut hf_entries = Vec::new();
        let mut page_number_pos = None;
        let mut new_page_numbers = Vec::new();

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
                    _ => {}
                }
            }
        }

        (hf_entries, page_number_pos, new_page_numbers)
    }

    /// 페이지 번호 + 머리말/꼬리말 최종 할당 (기존 Paginator::finalize_pages와 동일)
    fn finalize_pages(
        pages: &mut [PageContent],
        hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
        page_number_pos: &Option<crate::model::control::PageNumberPos>,
        new_page_numbers: &[(usize, u16)],
        _section_index: usize,
    ) {
        // 기존 Paginator::finalize_pages 로직을 그대로 재사용
        // (별도 함수로 추출하여 공유하는 것이 이상적이나, Phase 1에서는 복제)

        let mut current_header: Option<HeaderFooterRef> = None;
        let mut current_footer: Option<HeaderFooterRef> = None;
        let mut page_num: u32 = 1;

        for page in pages.iter_mut() {
            // 새 번호 지정 확인
            let first_para = page.column_contents.first()
                .and_then(|col| col.items.first())
                .map(|item| match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph { para_index, .. } => *para_index,
                    PageItem::Table { para_index, .. } => *para_index,
                    PageItem::PartialTable { para_index, .. } => *para_index,
                    PageItem::Shape { para_index, .. } => *para_index,
                });

            if let Some(fp) = first_para {
                for &(nn_pi, nn_num) in new_page_numbers {
                    if nn_pi <= fp {
                        page_num = nn_num as u32;
                    }
                }
            }

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

            page_num += 1;
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
            &a4_page_def(), &ColumnDef::default(), 0, &[],
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
            &measured.tables,
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
            &measured.tables,
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
            &measured.tables,
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
            &measured.tables,
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
            &measured.tables,
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
}
