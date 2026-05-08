//! 페이지 분할 (Pagination)
//!
//! IR(Document Model)의 문단 목록을 페이지 단위로 분할한다.
//! 각 페이지에 어떤 문단(또는 문단의 일부)이 배치되는지 결정한다.
//!
//! 2-패스 페이지네이션:
//! 1. HeightMeasurer로 모든 콘텐츠의 실제 렌더링 높이를 측정
//! 2. 측정된 높이를 기반으로 정확한 페이지 분할 수행

use crate::model::control::Control;
use crate::model::header_footer::HeaderFooterApply;
use crate::model::paragraph::{Paragraph, ColumnBreakType};
use crate::model::page::{PageDef, ColumnDef};
use crate::model::shape::CaptionDirection;
use super::composer::ComposedParagraph;
use super::height_measurer::{HeightMeasurer, MeasuredSection};
use super::page_layout::PageLayoutInfo;
use super::style_resolver::ResolvedStyleSet;

/// 페이지 분할 결과: 페이지별 콘텐츠 참조
#[derive(Debug)]
pub struct PaginationResult {
    /// 페이지별 콘텐츠 목록
    pub pages: Vec<PageContent>,
    /// 어울림 배치 표와 나란히 배치되는 빈 리턴 문단 목록 (전체)
    pub wrap_around_paras: Vec<WrapAroundPara>,
    /// 빈 줄 감추기로 높이 0 처리된 문단 인덱스 집합
    pub hidden_empty_paras: std::collections::HashSet<usize>,
}

/// 한 페이지에 배치될 콘텐츠
#[derive(Debug)]
pub struct PageContent {
    /// 페이지 인덱스 (0-based)
    pub page_index: u32,
    /// 실제 쪽 번호 (NewNumber 반영, 1-based)
    pub page_number: u32,
    /// 소속 구역 인덱스
    pub section_index: usize,
    /// 페이지 레이아웃 정보
    pub layout: PageLayoutInfo,
    /// 단별 콘텐츠
    pub column_contents: Vec<ColumnContent>,
    /// 이 페이지에 적용할 머리말 (None이면 머리말 없음)
    pub active_header: Option<HeaderFooterRef>,
    /// 이 페이지에 적용할 꼬리말 (None이면 꼬리말 없음)
    pub active_footer: Option<HeaderFooterRef>,
    /// 쪽 번호 위치 (None이면 쪽 번호 표시 안 함)
    pub page_number_pos: Option<crate::model::control::PageNumberPos>,
    /// 감추기 설정 (None이면 감추기 없음)
    pub page_hide: Option<crate::model::control::PageHide>,
    /// 이 페이지에 배치될 각주 목록
    pub footnotes: Vec<FootnoteRef>,
    /// 이 페이지에 적용할 바탕쪽 (None이면 바탕쪽 없음)
    pub active_master_page: Option<MasterPageRef>,
    /// 확장 바탕쪽 (임의 쪽 등, 기본 바탕쪽에 추가로 적용)
    pub extra_master_pages: Vec<MasterPageRef>,
}

/// 바탕쪽 참조
#[derive(Debug, Clone)]
pub struct MasterPageRef {
    /// 구역 인덱스
    pub section_index: usize,
    /// master_pages 배열 내 인덱스
    pub master_page_index: usize,
}

/// 머리말/꼬리말 참조
#[derive(Debug, Clone)]
pub struct HeaderFooterRef {
    /// Header/Footer 컨트롤이 있는 문단 인덱스
    pub para_index: usize,
    /// 해당 문단 내 컨트롤 인덱스
    pub control_index: usize,
    /// Header/Footer 컨트롤이 속한 구역 인덱스 (구역 간 상속 시 원본 구역 추적용)
    pub source_section_index: usize,
}

/// 각주 출처 (본문 문단 또는 표 셀 내)
#[derive(Debug, Clone)]
pub enum FootnoteSource {
    /// 본문 문단 내 각주
    Body {
        para_index: usize,
        control_index: usize,
    },
    /// 표 셀 내 각주
    TableCell {
        para_index: usize,
        table_control_index: usize,
        cell_index: usize,
        cell_para_index: usize,
        cell_control_index: usize,
    },
    /// 글상자(Shape TextBox) 내 각주
    ShapeTextBox {
        para_index: usize,
        shape_control_index: usize,
        tb_para_index: usize,
        tb_control_index: usize,
    },
}

/// 페이지에 배치되는 각주 참조
#[derive(Debug, Clone)]
pub struct FootnoteRef {
    /// 각주 번호 (1-based)
    pub number: u16,
    /// 출처
    pub source: FootnoteSource,
}

/// 한 단(Column)에 배치될 콘텐츠
#[derive(Debug)]
pub struct ColumnContent {
    /// 단 인덱스 (0-based)
    pub column_index: u16,
    /// 배치될 문단 슬라이스 정보
    pub items: Vec<PageItem>,
    /// 이 존의 레이아웃 (None이면 page.layout 사용). 다단 설정 나누기로 같은 페이지 내 단 수 변경 시 사용.
    pub zone_layout: Option<PageLayoutInfo>,
    /// 이 존의 body_area 내 y 시작 오프셋 (px). 이전 존의 높이만큼 아래로 밀림.
    pub zone_y_offset: f64,
    /// 어울림 배치 표와 나란히 배치되는 빈 리턴 문단 인덱스 목록
    /// (표 오른쪽에 문단 부호를 표시하기 위해 사용)
    pub wrap_around_paras: Vec<WrapAroundPara>,
    /// 단을 닫을 시점의 누적 사용 높이 (px). 진단/측정 도구용.
    pub used_height: f64,
    /// [Task #604 R3] anchor 그림/표 옆 wrap text 문단의 wrap context 메타데이터.
    /// typeset.rs 의 wrap_around state machine 매칭 결과 (anchor cs/sw 일치) 를
    /// layout 시점까지 보존. layout 이 본 메타데이터로 wrap zone 판정 + LineSeg cs/sw
    /// 정합 렌더 (PR #589 wrap_precomputed 메커니즘 대체).
    pub wrap_anchors: std::collections::HashMap<usize, WrapAnchorRef>,
}

/// 어울림 배치 표 옆에 배치되는 빈 리턴 문단 정보
#[derive(Debug, Clone)]
pub struct WrapAroundPara {
    /// 어울림 문단의 인덱스
    pub para_index: usize,
    /// 연관된 표의 문단 인덱스
    pub table_para_index: usize,
    /// 텍스트가 있는 문단인지 (false면 빈 리턴)
    pub has_text: bool,
}

/// [Task #604 R3] anchor 그림/표 ↔ wrap text 문단 매칭 메타데이터.
///
/// typeset.rs 의 wrap_around state machine 이 본 문단의 LineSeg cs/sw 가 anchor
/// 의 cs/sw 와 매칭됨을 검출 시 ColumnContent.wrap_anchors 에 등록. layout 단계가
/// 본 메타데이터로 wrap zone 정합 렌더 (LineSeg cs/sw 그대로 사용 — 현 보완6 효과).
#[derive(Debug, Clone)]
pub struct WrapAnchorRef {
    /// anchor 문단 인덱스 (그림/표 보유)
    pub anchor_para_index: usize,
    /// anchor wrap zone column_start (HWPUNIT)
    pub anchor_cs: i32,
    /// anchor wrap zone segment_width (HWPUNIT)
    pub anchor_sw: i32,
    /// [Task #722] anchor image 의 outer margin_right (HWPUNIT).
    /// 한컴 viewer 는 inter-image-text gap 으로 image margin_right 를 추가 적용.
    /// paragraph_layout 의 wrap_anchor 처리에서 cs px 에 +margin_right_px,
    /// sw px 에서 -margin_right_px 보정 (text 시작 위치와 가용 폭 정합).
    pub anchor_image_margin_right: i32,
}

/// 페이지에 배치되는 개별 항목
#[derive(Debug)]
pub enum PageItem {
    /// 문단 전체가 배치됨
    FullParagraph {
        /// 원본 문단 인덱스
        para_index: usize,
    },
    /// 문단 일부가 배치됨 (페이지 넘김)
    PartialParagraph {
        /// 원본 문단 인덱스
        para_index: usize,
        /// 시작 줄 인덱스 (LineSeg 인덱스)
        start_line: usize,
        /// 끝 줄 인덱스 (exclusive)
        end_line: usize,
    },
    /// 표 전체
    Table {
        /// 원본 문단 내 컨트롤 인덱스
        para_index: usize,
        control_index: usize,
    },
    /// 표의 일부 행만 배치 (페이지 분할)
    PartialTable {
        /// 원본 문단 인덱스
        para_index: usize,
        /// 컨트롤 인덱스
        control_index: usize,
        /// 시작 행 (inclusive)
        start_row: usize,
        /// 끝 행 (exclusive)
        end_row: usize,
        /// 연속 페이지 여부 (true면 제목행 반복)
        is_continuation: bool,
        /// 시작행 콘텐츠 시작 오프셋 (px, 패딩 제외). 0.0=처음부터.
        split_start_content_offset: f64,
        /// (end_row-1)행 최대 콘텐츠 높이 제한 (px, 패딩 제외). 0.0=전부.
        split_end_content_limit: f64,
    },
    /// 그리기 개체
    Shape {
        /// 원본 문단 내 컨트롤 인덱스
        para_index: usize,
        control_index: usize,
    },
}

/// [Issue #476] 인라인(treat_as_char) 컨트롤이 라우팅된 페이지/단을 찾는다.
///
/// `pages`: 이미 finalize 된 이전 페이지들의 ColumnContent(items 포함).
/// `current_items`: 현재(마지막) 페이지의 진행 중 항목 목록 (아직 flush 안 된 상태).
///
/// 박스의 char 위치 → line index → 그 line 을 포함하는 PartialParagraph 가 들어있는
/// `(page_idx, column_idx)` 를 반환. 마지막 페이지(현재 처리 중)에 들어있으면 `None` (= 현재).
/// 어디에도 없거나 페이지 분할이 없으면 `None`.
pub fn find_inline_control_target_page(
    pages: &[PageContent],
    current_items: &[PageItem],
    para_idx: usize,
    ctrl_idx: usize,
    para: &Paragraph,
) -> Option<(usize, usize)> {
    let positions = para.control_text_positions();
    let ctrl_text_pos = *positions.get(ctrl_idx)?;
    let target_line = para.line_segs.iter().enumerate()
        .rev()
        .find(|(_, ls)| (ls.text_start as usize) <= ctrl_text_pos)
        .map(|(i, _)| i)
        .unwrap_or(0);

    // 1) 현재(마지막) 페이지의 current_items 검사 — 박스 line 이 여기 있으면 None (= 현재)
    let in_current = current_items.iter().any(|item| match item {
        PageItem::FullParagraph { para_index } if *para_index == para_idx => true,
        PageItem::PartialParagraph { para_index, start_line, end_line }
            if *para_index == para_idx && (*start_line..*end_line).contains(&target_line) => true,
        _ => false,
    });
    if in_current {
        return None;
    }

    // 2) 이전 페이지/단 검색
    for (page_idx, page) in pages.iter().enumerate() {
        for (col_idx, col) in page.column_contents.iter().enumerate() {
            let hit = col.items.iter().any(|item| match item {
                PageItem::FullParagraph { para_index } if *para_index == para_idx => true,
                PageItem::PartialParagraph { para_index, start_line, end_line }
                    if *para_index == para_idx && (*start_line..*end_line).contains(&target_line) => true,
                _ => false,
            });
            if hit {
                return Some((page_idx, col_idx));
            }
        }
    }
    None
}

impl PageItem {
    /// 항목의 para_index를 반환한다.
    pub fn para_index(&self) -> usize {
        match self {
            PageItem::FullParagraph { para_index } => *para_index,
            PageItem::PartialParagraph { para_index, .. } => *para_index,
            PageItem::Table { para_index, .. } => *para_index,
            PageItem::PartialTable { para_index, .. } => *para_index,
            PageItem::Shape { para_index, .. } => *para_index,
        }
    }

    /// para_index를 offset만큼 조정한 새 항목을 반환한다.
    pub fn with_offset(&self, offset: i32) -> Self {
        let adjust = |pi: usize| (pi as i64 + offset as i64).max(0) as usize;
        match self {
            PageItem::FullParagraph { para_index } =>
                PageItem::FullParagraph { para_index: adjust(*para_index) },
            PageItem::PartialParagraph { para_index, start_line, end_line } =>
                PageItem::PartialParagraph { para_index: adjust(*para_index), start_line: *start_line, end_line: *end_line },
            PageItem::Table { para_index, control_index } =>
                PageItem::Table { para_index: adjust(*para_index), control_index: *control_index },
            PageItem::PartialTable { para_index, control_index, start_row, end_row, is_continuation,
                split_start_content_offset, split_end_content_limit } =>
                PageItem::PartialTable { para_index: adjust(*para_index), control_index: *control_index,
                    start_row: *start_row, end_row: *end_row, is_continuation: *is_continuation,
                    split_start_content_offset: *split_start_content_offset, split_end_content_limit: *split_end_content_limit },
            PageItem::Shape { para_index, control_index } =>
                PageItem::Shape { para_index: adjust(*para_index), control_index: *control_index },
        }
    }

    /// 두 항목이 구조적으로 동일한지 비교 (para_index offset 적용).
    fn matches_with_offset(&self, other: &PageItem, offset: i32) -> bool {
        let adj = |pi: usize| (pi as i64 + offset as i64) as usize;
        match (self, other) {
            (PageItem::FullParagraph { para_index: a }, PageItem::FullParagraph { para_index: b }) =>
                *a == adj(*b),
            (PageItem::PartialParagraph { para_index: a, start_line: s1, end_line: e1 },
             PageItem::PartialParagraph { para_index: b, start_line: s2, end_line: e2 }) =>
                *a == adj(*b) && s1 == s2 && e1 == e2,
            (PageItem::Table { para_index: a, control_index: c1 },
             PageItem::Table { para_index: b, control_index: c2 }) =>
                *a == adj(*b) && c1 == c2,
            (PageItem::PartialTable { para_index: a, control_index: c1, start_row: sr1, end_row: er1, .. },
             PageItem::PartialTable { para_index: b, control_index: c2, start_row: sr2, end_row: er2, .. }) =>
                *a == adj(*b) && c1 == c2 && sr1 == sr2 && er1 == er2,
            (PageItem::Shape { para_index: a, control_index: c1 },
             PageItem::Shape { para_index: b, control_index: c2 }) =>
                *a == adj(*b) && c1 == c2,
            _ => false,
        }
    }
}

impl PaginationResult {
    /// 이전 결과와 비교하여 수렴 페이지를 찾는다.
    /// offset: 문단 인덱스 변화량 (삽입=+1, 삭제=-1)
    /// 반환: 수렴 시작 페이지 인덱스 (None이면 수렴 없음)
    pub fn find_convergence(&self, old: &PaginationResult, offset: i32) -> Option<usize> {
        if offset == 0 { return Some(0); }
        for page_idx in 0..self.pages.len().min(old.pages.len()) {
            let new_page = &self.pages[page_idx];
            let old_page = &old.pages[page_idx];
            if new_page.column_contents.len() != old_page.column_contents.len() { continue; }
            let matched = new_page.column_contents.iter()
                .zip(old_page.column_contents.iter())
                .all(|(nc, oc)| {
                    nc.items.len() == oc.items.len()
                    && nc.items.iter().zip(oc.items.iter())
                        .all(|(ni, oi)| ni.matches_with_offset(oi, offset))
                });
            if matched {
                return Some(page_idx);
            }
        }
        None
    }

    /// 수렴 이후 페이지를 이전 결과에서 복사한다 (para_index offset 적용).
    pub fn copy_converged_pages(&mut self, old: &PaginationResult, converge_page: usize, offset: i32) {
        // 수렴 페이지 이후를 이전 결과에서 복사
        self.pages.truncate(converge_page);
        for old_page in &old.pages[converge_page..] {
            let mut new_page = PageContent {
                page_index: old_page.page_index,
                page_number: old_page.page_number,
                section_index: old_page.section_index,
                layout: old_page.layout.clone(),
                column_contents: old_page.column_contents.iter().map(|cc| {
                    ColumnContent {
                        column_index: cc.column_index,
                        items: cc.items.iter().map(|it| it.with_offset(offset)).collect(),
                        zone_layout: cc.zone_layout.clone(),
                        zone_y_offset: cc.zone_y_offset,
                        wrap_around_paras: cc.wrap_around_paras.iter().map(|w| WrapAroundPara {
                            para_index: (w.para_index as i64 + offset as i64).max(0) as usize,
                            table_para_index: (w.table_para_index as i64 + offset as i64).max(0) as usize,
                            has_text: w.has_text,
                        }).collect(),
                        used_height: cc.used_height,
                        wrap_anchors: cc.wrap_anchors.iter().map(|(k, v)| {
                            ((*k as i64 + offset as i64).max(0) as usize, WrapAnchorRef {
                                anchor_para_index: (v.anchor_para_index as i64 + offset as i64).max(0) as usize,
                                anchor_cs: v.anchor_cs,
                                anchor_sw: v.anchor_sw,
                                anchor_image_margin_right: v.anchor_image_margin_right,
                            })
                        }).collect(),
                    }
                }).collect(),
                active_header: old_page.active_header.clone(),
                active_footer: old_page.active_footer.clone(),
                page_number_pos: old_page.page_number_pos.clone(),
                page_hide: old_page.page_hide.clone(),
                footnotes: old_page.footnotes.iter().map(|f| {
                    let source = match &f.source {
                        FootnoteSource::Body { para_index, control_index } =>
                            FootnoteSource::Body { para_index: (*para_index as i64 + offset as i64).max(0) as usize, control_index: *control_index },
                        FootnoteSource::TableCell { para_index, table_control_index, cell_index, cell_para_index, cell_control_index } =>
                            FootnoteSource::TableCell { para_index: (*para_index as i64 + offset as i64).max(0) as usize,
                                table_control_index: *table_control_index, cell_index: *cell_index,
                                cell_para_index: *cell_para_index, cell_control_index: *cell_control_index },
                        FootnoteSource::ShapeTextBox { para_index, shape_control_index, tb_para_index, tb_control_index } =>
                            FootnoteSource::ShapeTextBox { para_index: (*para_index as i64 + offset as i64).max(0) as usize,
                                shape_control_index: *shape_control_index, tb_para_index: *tb_para_index, tb_control_index: *tb_control_index },
                    };
                    FootnoteRef { number: f.number, source }
                }).collect(),
                active_master_page: old_page.active_master_page.clone(),
                extra_master_pages: old_page.extra_master_pages.clone(),
            };
            // hidden_empty_paras는 별도 처리
            self.pages.push(new_page);
        }
        // wrap_around_paras도 복사
        for w in &old.wrap_around_paras {
            let shifted_pi = (w.para_index as i64 + offset as i64).max(0) as usize;
            let shifted_tpi = (w.table_para_index as i64 + offset as i64).max(0) as usize;
            if !self.wrap_around_paras.iter().any(|e| e.para_index == shifted_pi) {
                self.wrap_around_paras.push(WrapAroundPara {
                    para_index: shifted_pi,
                    table_para_index: shifted_tpi,
                    has_text: w.has_text,
                });
            }
        }
        // hidden_empty_paras offset
        let mut new_hidden = std::collections::HashSet::new();
        for &pi in &old.hidden_empty_paras {
            new_hidden.insert((pi as i64 + offset as i64).max(0) as usize);
        }
        self.hidden_empty_paras = new_hidden;
    }
}

/// 페이지 분할 옵션
#[derive(Debug, Clone, Default)]
pub struct PaginationOpts {
    /// 빈 줄 숨김 (SectionDef.hide_empty_line)
    pub hide_empty_line: bool,
    /// LINE_SEG vpos-reset (vertical_pos==0, line>0) 위치를 강제 단/페이지 경계로 처리
    pub respect_vpos_reset: bool,
}

/// 페이지 분할 엔진
pub struct Paginator {
    /// DPI
    dpi: f64,
}

impl Paginator {
    pub fn new(dpi: f64) -> Self {
        Self { dpi }
    }

    /// 기본 DPI(96)로 생성
    pub fn with_default_dpi() -> Self {
        Self::new(super::DEFAULT_DPI)
    }

    /// 문단 내 단 경계를 감지한다.
    /// HWP에서 같은 너비 다단 레이아웃의 문단은 한 문단이 여러 단에 걸칠 수 있다.
    /// LineSeg의 vertical_pos가 급격히 감소(이전 줄의 vpos보다 작아짐)하면 단이 변경된 것.
    /// 반환: 각 단의 시작 줄 인덱스 목록 (첫 번째는 항상 0)
    fn detect_column_breaks_in_paragraph(para: &Paragraph) -> Vec<usize> {
        let mut breaks = vec![0usize];
        if para.line_segs.len() <= 1 {
            return breaks;
        }
        for i in 1..para.line_segs.len() {
            let prev_vpos = para.line_segs[i - 1].vertical_pos;
            let curr_vpos = para.line_segs[i].vertical_pos;
            // vpos가 이전보다 작아지면 단 경계
            if curr_vpos < prev_vpos {
                breaks.push(i);
            }
        }
        breaks
    }

    /// 구역의 문단 목록을 페이지로 분할한다.
    ///
    /// 2-패스 페이지네이션:
    /// 1. HeightMeasurer로 모든 콘텐츠의 실제 렌더링 높이를 사전 측정
    /// 2. 측정된 높이를 기반으로 정확한 페이지 분할 수행
    ///
    /// - 본문 영역 높이를 초과하면 새 페이지 시작
    /// - ColumnBreakType::Page이면 강제 페이지 넘김
    pub fn paginate(
        &self,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
    ) -> (PaginationResult, MeasuredSection) {
        // === 1-패스: 높이 사전 측정 ===
        let measurer = HeightMeasurer::new(self.dpi);
        let measured = measurer.measure_section(paragraphs, composed, styles);

        // === 2-패스: 측정된 높이로 페이지 분할 ===
        let result = self.paginate_with_measured(paragraphs, &measured, page_def, column_def, section_index, &styles.para_styles);
        (result, measured)
    }
}

mod engine;
mod state;

#[cfg(test)]
mod tests;
