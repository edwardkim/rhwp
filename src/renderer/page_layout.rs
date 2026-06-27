//! 페이지 레이아웃 계산 (PageDef → 렌더링 영역)

use super::{hwpunit_to_px, DEFAULT_DPI};
use crate::model::page::{ColumnDef, PageAreas, PageDef};
use crate::model::Rect;

/// 페이지 레이아웃 정보 (픽셀 단위로 변환된 영역)
#[derive(Debug, Clone)]
pub struct PageLayoutInfo {
    /// 페이지 전체 크기 (px)
    pub page_width: f64,
    pub page_height: f64,
    /// 머리말 영역 (px)
    pub header_area: LayoutRect,
    /// 본문 영역 (px)
    pub body_area: LayoutRect,
    /// 단별 본문 영역 (px)
    pub column_areas: Vec<LayoutRect>,
    /// 각주 영역 (px)
    pub footnote_area: LayoutRect,
    /// 꼬리말 영역 (px)
    pub footer_area: LayoutRect,
    /// DPI
    pub dpi: f64,
    /// 단 구분선 종류 (0=없음, 1=실선, 2=점선, 3=파선...)
    pub separator_type: u8,
    /// 단 구분선 굵기 (border_width 코드)
    pub separator_width: u8,
    /// 단 구분선 색상
    pub separator_color: u32,
    /// 페이지네이션 하단 허용치 (px). body_area 를 변경하지 않고 paginator 에게만 추가 공간 제공.
    pub pagination_tolerance_px: f64,
}

/// 레이아웃 영역 (픽셀 단위)
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl LayoutRect {
    pub fn from_hwpunit_rect(rect: &Rect, dpi: f64) -> Self {
        let scale = dpi / super::HWPUNIT_PER_INCH;
        Self {
            x: rect.left as f64 * scale,
            y: rect.top as f64 * scale,
            width: rect.width() as f64 * scale,
            height: rect.height() as f64 * scale,
        }
    }
}

impl PageLayoutInfo {
    /// PageDef와 ColumnDef로부터 페이지 레이아웃을 계산한다.
    pub fn from_page_def(page_def: &PageDef, column_def: &ColumnDef, dpi: f64) -> Self {
        Self::from_page_def_for_page(page_def, column_def, dpi, 1)
    }

    /// PageDef, ColumnDef, 최종 쪽번호로부터 페이지 레이아웃을 계산한다.
    pub fn from_page_def_for_page(
        page_def: &PageDef,
        column_def: &ColumnDef,
        dpi: f64,
        page_number: u32,
    ) -> Self {
        // landscape=true이면 width/height 교환
        let (width_hwp, height_hwp) = if page_def.landscape {
            (page_def.height, page_def.width)
        } else {
            (page_def.width, page_def.height)
        };
        let page_width = hwpunit_to_px(width_hwp as i32, dpi);
        let page_height = hwpunit_to_px(height_hwp as i32, dpi);

        let areas = PageAreas::from_page_def_for_page(page_def, page_number);

        let header_area = LayoutRect::from_hwpunit_rect(&areas.header_area, dpi);
        let body_area = LayoutRect::from_hwpunit_rect(&areas.body_area, dpi);
        let footer_area = LayoutRect::from_hwpunit_rect(&areas.footer_area, dpi);
        let footnote_area = LayoutRect::from_hwpunit_rect(&areas.footnote_area, dpi);

        // 다단 영역 계산
        let column_areas = calculate_column_areas(&body_area, column_def, dpi);

        let pagination_tolerance_px =
            hwpunit_to_px(page_def.pagination_bottom_tolerance as i32, dpi);

        Self {
            page_width,
            page_height,
            header_area,
            body_area,
            column_areas,
            footnote_area,
            footer_area,
            dpi,
            separator_type: column_def.separator_type,
            separator_width: column_def.separator_width,
            separator_color: column_def.separator_color,
            pagination_tolerance_px,
        }
    }

    /// 기본 DPI(96)로 계산
    pub fn from_page_def_default(page_def: &PageDef, column_def: &ColumnDef) -> Self {
        Self::from_page_def(page_def, column_def, DEFAULT_DPI)
    }

    /// 기존 레이아웃을 최종 쪽번호 기준 좌우 여백으로 이동한다.
    ///
    /// ColumnDef가 다른 zone layout일 수 있으므로 단 너비/간격은 보존하고, page body의
    /// 기준 x 이동량만 각 영역에 적용한다.
    pub fn apply_page_number_margins(&mut self, page_def: &PageDef, page_number: u32) {
        let target_areas = PageAreas::from_page_def_for_page(page_def, page_number);
        let target_header = LayoutRect::from_hwpunit_rect(&target_areas.header_area, self.dpi);
        let target_body = LayoutRect::from_hwpunit_rect(&target_areas.body_area, self.dpi);
        let target_footer = LayoutRect::from_hwpunit_rect(&target_areas.footer_area, self.dpi);

        let delta_x = target_body.x - self.body_area.x;
        if delta_x.abs() < f64::EPSILON
            && (target_body.width - self.body_area.width).abs() < f64::EPSILON
        {
            return;
        }

        self.header_area.x = target_header.x;
        self.header_area.width = target_header.width;
        self.body_area.x = target_body.x;
        self.body_area.width = target_body.width;
        self.footer_area.x = target_footer.x;
        self.footer_area.width = target_footer.width;

        if self.footnote_area.width > 0.0 || self.footnote_area.height > 0.0 {
            self.footnote_area.x += delta_x;
            self.footnote_area.width = target_body.width;
        }

        for column_area in &mut self.column_areas {
            column_area.x += delta_x;
        }
    }

    /// 본문 영역의 사용 가능한 높이 (각주 영역 제외 + 페이지네이션 허용치 포함)
    pub fn available_body_height(&self) -> f64 {
        self.body_area.height - self.footnote_area.height + self.pagination_tolerance_px
    }

    /// 단 너비 (HWPUNIT) — vpos 보정에서 segment_width 비교에 사용
    pub fn column_width_hu(&self) -> i32 {
        self.column_areas
            .first()
            .map(|a| super::px_to_hwpunit(a.width, self.dpi))
            .unwrap_or(super::px_to_hwpunit(self.body_area.width, self.dpi))
    }

    /// 각주 영역을 동적으로 계산하여 레이아웃을 갱신한다.
    ///
    /// 각주 높이만큼 본문 영역 하단에 각주 영역을 설정한다.
    pub fn update_footnote_area(&mut self, footnote_height: f64) {
        if footnote_height <= 0.0 {
            return;
        }
        let h = footnote_height.min(self.body_area.height);
        self.footnote_area = LayoutRect {
            x: self.body_area.x,
            y: self.body_area.y + self.body_area.height - h,
            width: self.body_area.width,
            height: h,
        };
    }
}

/// 다단 영역 계산
fn calculate_column_areas(
    body_area: &LayoutRect,
    column_def: &ColumnDef,
    dpi: f64,
) -> Vec<LayoutRect> {
    let col_count = column_def.column_count.max(1) as usize;
    if col_count <= 1 {
        return vec![*body_area];
    }

    // same_width=false이고 개별 너비가 있으면 사용
    if !column_def.same_width && column_def.widths.len() >= col_count {
        let mut areas = Vec::with_capacity(col_count);
        let mut x = body_area.x;

        if column_def.proportional_widths {
            // HWP 5.0 바이너리: widths/gaps는 비례값 (합계=32768)
            // body_area.width에 대한 비례로 변환
            let total: f64 = column_def
                .widths
                .iter()
                .chain(column_def.gaps.iter())
                .map(|&v| (v as u16) as f64)
                .sum();
            let scale = if total > 0.0 {
                body_area.width / total
            } else {
                1.0
            };

            for i in 0..col_count {
                let w = (column_def.widths[i] as u16) as f64 * scale;
                let gap = if i < column_def.gaps.len() {
                    (column_def.gaps[i] as u16) as f64 * scale
                } else {
                    0.0
                };
                areas.push(LayoutRect {
                    x,
                    y: body_area.y,
                    width: w,
                    height: body_area.height,
                });
                x += w + gap;
            }
        } else {
            // HWPX 등: 절대 HWPUNIT 값
            for i in 0..col_count {
                let w = hwpunit_to_px(column_def.widths[i] as i32, dpi);
                let gap = if i < column_def.gaps.len() {
                    hwpunit_to_px(column_def.gaps[i] as i32, dpi)
                } else {
                    0.0
                };
                areas.push(LayoutRect {
                    x,
                    y: body_area.y,
                    width: w,
                    height: body_area.height,
                });
                x += w + gap;
            }
        }
        return areas;
    }

    // same_width=true: 균등 분할
    let spacing = hwpunit_to_px(column_def.spacing as i32, dpi);
    let total_spacing = spacing * (col_count - 1) as f64;
    let col_width = (body_area.width - total_spacing) / col_count as f64;

    (0..col_count)
        .map(|i| LayoutRect {
            x: body_area.x + (col_width + spacing) * i as f64,
            y: body_area.y,
            width: col_width,
            height: body_area.height,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::page::{BindingMethod, ColumnDef, PageDef};

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

    #[test]
    fn test_single_column_layout() {
        let page_def = a4_page_def();
        let col_def = ColumnDef {
            column_count: 1,
            ..Default::default()
        };
        let layout = PageLayoutInfo::from_page_def_default(&page_def, &col_def);

        assert!((layout.page_width - 793.7).abs() < 1.0);
        assert!((layout.page_height - 1122.5).abs() < 1.0);
        assert_eq!(layout.column_areas.len(), 1);
        assert!(layout.body_area.width > 0.0);
        assert!(layout.body_area.height > 0.0);
    }

    #[test]
    fn test_two_column_layout() {
        let page_def = a4_page_def();
        let col_def = ColumnDef {
            column_count: 2,
            spacing: 567, // ~2mm
            ..Default::default()
        };
        let layout = PageLayoutInfo::from_page_def_default(&page_def, &col_def);

        assert_eq!(layout.column_areas.len(), 2);
        let col1 = &layout.column_areas[0];
        let col2 = &layout.column_areas[1];
        // 두 단의 너비는 같아야 함
        assert!((col1.width - col2.width).abs() < 0.01);
        // 두 번째 단은 첫 번째 단 오른쪽에 위치
        assert!(col2.x > col1.x + col1.width);
    }

    #[test]
    fn test_available_body_height() {
        let page_def = a4_page_def();
        let col_def = ColumnDef::default();
        let layout = PageLayoutInfo::from_page_def_default(&page_def, &col_def);

        assert!(layout.available_body_height() > 0.0);
    }

    #[test]
    fn page_layout_duplex_sided_even_page_swaps_body_and_columns() {
        let page_def = PageDef {
            width: 1000,
            height: 1400,
            margin_left: 100,
            margin_right: 200,
            margin_gutter: 30,
            margin_top: 10,
            margin_header: 20,
            margin_bottom: 40,
            margin_footer: 50,
            binding: BindingMethod::DuplexSided,
            ..Default::default()
        };
        let col_def = ColumnDef {
            column_count: 2,
            spacing: 60,
            ..Default::default()
        };

        let odd = PageLayoutInfo::from_page_def_for_page(&page_def, &col_def, DEFAULT_DPI, 1);
        let even = PageLayoutInfo::from_page_def_for_page(&page_def, &col_def, DEFAULT_DPI, 2);

        assert!(even.body_area.x > odd.body_area.x);
        assert!((even.body_area.width - odd.body_area.width).abs() < 0.01);
        assert_eq!(odd.column_areas.len(), 2);
        assert_eq!(even.column_areas.len(), 2);
        assert!((even.column_areas[0].x - even.body_area.x).abs() < 0.01);
        assert!((even.column_areas[0].width - odd.column_areas[0].width).abs() < 0.01);
        assert!(
            (even.column_areas[1].x
                - even.column_areas[0].x
                - (odd.column_areas[1].x - odd.column_areas[0].x))
                .abs()
                < 0.01
        );
    }

    #[test]
    fn page_layout_from_page_def_matches_page_one_layout() {
        let page_def = PageDef {
            binding: BindingMethod::DuplexSided,
            ..a4_page_def()
        };
        let col_def = ColumnDef::default();

        let default_layout = PageLayoutInfo::from_page_def(&page_def, &col_def, DEFAULT_DPI);
        let page_one_layout =
            PageLayoutInfo::from_page_def_for_page(&page_def, &col_def, DEFAULT_DPI, 1);

        assert!((default_layout.body_area.x - page_one_layout.body_area.x).abs() < 0.01);
        assert!((default_layout.body_area.width - page_one_layout.body_area.width).abs() < 0.01);
    }

    #[test]
    fn apply_page_number_margins_moves_existing_zone_layout_without_rebuilding_columns() {
        let page_def = PageDef {
            width: 1000,
            height: 1400,
            margin_left: 100,
            margin_right: 200,
            margin_gutter: 30,
            binding: BindingMethod::DuplexSided,
            ..Default::default()
        };
        let col_def = ColumnDef {
            column_count: 2,
            same_width: false,
            widths: vec![200, 300],
            gaps: vec![40],
            ..Default::default()
        };

        let mut layout =
            PageLayoutInfo::from_page_def_for_page(&page_def, &col_def, DEFAULT_DPI, 1);
        let original_col_delta = layout.column_areas[1].x - layout.column_areas[0].x;

        layout.apply_page_number_margins(&page_def, 2);

        let expected = PageLayoutInfo::from_page_def_for_page(&page_def, &col_def, DEFAULT_DPI, 2);
        assert!((layout.body_area.x - expected.body_area.x).abs() < 0.01);
        assert!((layout.body_area.width - expected.body_area.width).abs() < 0.01);
        assert!((layout.column_areas[0].x - expected.body_area.x).abs() < 0.01);
        assert!(
            (layout.column_areas[1].x - layout.column_areas[0].x - original_col_delta).abs() < 0.01
        );
    }
}
