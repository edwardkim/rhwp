//! 페이지 레이아웃 (PageDef, Margin, PageBorderFill, Column)

use super::*;

/// 용지 설정 (HWPTAG_PAGE_DEF)
#[derive(Debug, Clone, Default)]
pub struct PageDef {
    /// 용지 가로 크기
    pub width: HwpUnit,
    /// 용지 세로 크기
    pub height: HwpUnit,
    /// 왼쪽 여백
    pub margin_left: HwpUnit,
    /// 오른쪽 여백
    pub margin_right: HwpUnit,
    /// 위 여백
    pub margin_top: HwpUnit,
    /// 아래 여백
    pub margin_bottom: HwpUnit,
    /// 머리말 여백
    pub margin_header: HwpUnit,
    /// 꼬리말 여백
    pub margin_footer: HwpUnit,
    /// 제본 여백
    pub margin_gutter: HwpUnit,
    /// 페이지네이션 하단 허용치 (HWPUNIT). margin_bottom 을 변조하지 않고
    /// paginator 에게만 추가 공간을 허용할 때 사용. 기본 0.
    pub pagination_bottom_tolerance: HwpUnit,
    /// 속성 비트 플래그
    pub attr: u32,
    /// 용지 방향 (0: 좁게/세로, 1: 넓게/가로)
    pub landscape: bool,
    /// 제책 방법
    pub binding: BindingMethod,
}

/// 제책 방법
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BindingMethod {
    #[default]
    /// 한쪽 편집
    SingleSided,
    /// 맞쪽 편집
    DuplexSided,
    /// 위로 넘기기
    TopFlip,
}

/// 쪽 테두리/배경 (HWPTAG_PAGE_BORDER_FILL)
#[derive(Debug, Clone, Default)]
pub struct PageBorderFill {
    /// 속성 비트 플래그
    pub attr: u32,
    /// 왼쪽 간격
    pub spacing_left: HwpUnit16,
    /// 오른쪽 간격
    pub spacing_right: HwpUnit16,
    /// 위쪽 간격
    pub spacing_top: HwpUnit16,
    /// 아래쪽 간격
    pub spacing_bottom: HwpUnit16,
    /// 테두리/배경 ID 참조
    pub border_fill_id: u16,
}

/// 단 정의 ('cold' 컨트롤)
#[derive(Debug, Clone, Default)]
pub struct ColumnDef {
    /// 단 종류
    pub column_type: ColumnType,
    /// 단 수
    pub column_count: u16,
    /// 단 방향
    pub direction: ColumnDirection,
    /// 단 너비 동일하게
    pub same_width: bool,
    /// 단 간격
    pub spacing: HwpUnit16,
    /// 단별 너비 목록 (same_width가 false일 때)
    /// HWP 5.0 바이너리: 비례값 (합계=32768), HWPX: 절대 HWPUNIT
    pub widths: Vec<HwpUnit16>,
    /// 단별 간격 목록 (same_width가 false일 때, 각 단 뒤의 간격)
    pub gaps: Vec<HwpUnit16>,
    /// widths/gaps가 비례값(true)인지 절대 HWPUNIT(false)인지
    pub proportional_widths: bool,
    /// 구분선 종류
    pub separator_type: u8,
    /// 구분선 굵기
    pub separator_width: u8,
    /// 구분선 색상
    pub separator_color: ColorRef,
    /// 원본 attr u16 전체 (라운드트립 보존용, 0이면 재구성)
    pub raw_attr: u16,
}

/// 단 종류
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ColumnType {
    #[default]
    Normal,
    /// 배분 (단 너비를 균등 배분)
    Distribute,
    /// 평행 (왼쪽부터 순서대로)
    Parallel,
}

/// 단 방향
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ColumnDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

/// 페이지 렌더링에 필요한 계산된 영역 정보
#[derive(Debug, Clone, Default)]
pub struct PageAreas {
    /// 머리말 영역
    pub header_area: Rect,
    /// 본문 영역
    pub body_area: Rect,
    /// 단별 본문 영역
    pub column_areas: Vec<Rect>,
    /// 각주 영역
    pub footnote_area: Rect,
    /// 꼬리말 영역
    pub footer_area: Rect,
}

impl PageAreas {
    /// PageDef로부터 페이지 영역을 계산한다.
    ///
    /// HWP의 여백 구조 (한컴 도움말 기준):
    /// - margin_header: 용지 상단에서 머리말 시작까지 거리
    /// - margin_top: 머리말 영역의 높이
    /// - 본문 시작 = margin_header + margin_top
    /// - margin_bottom: 꼬리말 영역의 높이
    /// - margin_footer: 용지 하단에서 꼬리말 끝까지 거리
    /// - 본문 끝 = height - margin_footer - margin_bottom
    ///
    /// landscape=true이면 width와 height를 교환하여 가로 방향으로 렌더링
    pub fn from_page_def(page_def: &PageDef) -> Self {
        // landscape=true면 width/height 교환
        let (page_width, page_height) = if page_def.landscape {
            (page_def.height, page_def.width)
        } else {
            (page_def.width, page_def.height)
        };

        let content_left = page_def.margin_left + page_def.margin_gutter;
        let content_right = page_width - page_def.margin_right;
        // HWP 본문 시작 = margin_header + margin_top (한컴 도움말 기준)
        let content_top = page_def.margin_header + page_def.margin_top;
        // HWP 본문 끝 = height - margin_footer - margin_bottom
        let content_bottom = page_height - page_def.margin_footer - page_def.margin_bottom;

        let header_area = Rect {
            left: content_left as i32,
            top: page_def.margin_top as i32,
            right: content_right as i32,
            bottom: content_top as i32,
        };

        let body_area = Rect {
            left: content_left as i32,
            top: content_top as i32,
            right: content_right as i32,
            bottom: content_bottom as i32,
        };

        let footer_area = Rect {
            left: content_left as i32,
            top: content_bottom as i32,
            right: content_right as i32,
            bottom: (page_height - page_def.margin_footer) as i32,
        };

        PageAreas {
            header_area,
            body_area,
            column_areas: vec![body_area],
            footnote_area: Rect::default(),
            footer_area,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_def_a4() {
        // A4 기본 설정 (210mm x 297mm)
        // 1mm = 283.46 HWPUNIT (7200/25.4)
        let page = PageDef {
            width: 59528,      // ~210mm
            height: 84188,     // ~297mm
            margin_left: 8504, // ~30mm
            margin_right: 8504,
            margin_top: 5669,    // ~20mm
            margin_bottom: 4252, // ~15mm
            margin_header: 4252,
            margin_footer: 4252,
            margin_gutter: 0,
            ..Default::default()
        };
        assert!(page.width > 0);
        assert!(page.height > page.width); // 세로 방향
    }

    #[test]
    fn test_page_areas_calculation() {
        let page_def = PageDef {
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
        };
        let areas = PageAreas::from_page_def(&page_def);
        assert!(areas.body_area.width() > 0);
        assert!(areas.body_area.height() > 0);
        assert!(areas.header_area.height() >= 0);
    }

    #[test]
    fn test_column_def_default() {
        let col = ColumnDef::default();
        assert_eq!(col.column_count, 0);
        assert!(!col.same_width);
    }
}
