//! HWP 문서의 중간 표현(IR) 데이터 모델
//!
//! HWP 파일에서 파싱된 데이터를 렌더링 백엔드에 독립적인
//! 구조체로 표현한다. 모든 크기 단위는 HWPUNIT(1/7200인치)을 사용한다.

pub mod bin_data;
pub mod control;
pub mod document;
pub mod event;
pub mod footnote;
pub mod header_footer;
pub mod image;
pub mod page;
pub mod paragraph;
pub mod path;
pub mod shape;
pub mod style;
pub mod table;

/// HWP 내부 단위 (1/7200 인치, 부호 없음)
pub type HwpUnit = u32;

/// HWP 내부 단위 (1/7200 인치, 부호 있음)
pub type SHwpUnit = i32;

/// HWP 16비트 내부 단위 (부호 있음)
pub type HwpUnit16 = i16;

/// RGB 색상 값 (0x00BBGGRR 형식)
pub type ColorRef = u32;

/// 2차원 좌표
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// 직사각형 영역
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// 4방향 여백
#[derive(Debug, Clone, Copy, Default)]
pub struct Padding {
    pub left: HwpUnit16,
    pub right: HwpUnit16,
    pub top: HwpUnit16,
    pub bottom: HwpUnit16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_dimensions() {
        let r = Rect {
            left: 100,
            top: 200,
            right: 500,
            bottom: 700,
        };
        assert_eq!(r.width(), 400);
        assert_eq!(r.height(), 500);
    }

    #[test]
    fn test_colorref_format() {
        // 빨간색: R=0xFF, G=0x00, B=0x00 → 0x000000FF
        let red: ColorRef = 0x000000FF;
        assert_eq!(red & 0xFF, 0xFF); // R
        assert_eq!((red >> 8) & 0xFF, 0x00); // G
        assert_eq!((red >> 16) & 0xFF, 0x00); // B
    }
}
