//! [Task #902 v2 Stage 12] WMF raster Player 의 device context state.
//!
//! 알고리즘 출처: LibreOffice emfio (MPL 2.0)
//! mtftools.cxx 의 MtfTools 클래스 의 state members 를 Rust 로 포팅.

use crate::wmf::parser::*;
use std::collections::HashMap;

/// MM_TEXT 가 default — 1 logical unit = 1 device pixel.
#[derive(Clone, Debug)]
pub struct RasterState {
    /// Window origin (logical 좌표계 기준점)
    pub window_origin: (i16, i16),
    /// Window extent (logical 좌표계 크기)
    pub window_ext: (i16, i16),
    /// Viewport origin (device 좌표계 기준점)
    pub viewport_origin: (i16, i16),
    /// Viewport extent (device 좌표계 크기)
    pub viewport_ext: (i16, i16),
    /// SetViewportExt 명시 호출 여부
    pub viewport_ext_set: bool,
    pub map_mode: MapMode,

    /// 현재 drawing position (LO maActPos)
    pub current_position: (i16, i16),

    /// 텍스트 정렬 모드 (TA_LEFT/CENTER/RIGHT + TA_TOP/BASELINE/BOTTOM + TA_UPDATECP)
    pub text_align: u16,
    /// Background mode (TRANSPARENT or OPAQUE)
    pub bk_mode: MixMode,
    pub bk_color: ColorRef,
    pub text_color: ColorRef,
    /// Polygon fill rule (ALTERNATE or WINDING)
    pub poly_fill_mode: PolyFillMode,
    /// ROP2 binary raster operation
    pub rop2: Option<BinaryRasterOperation>,

    /// Object table (graphics objects pen, brush, font, region, palette).
    /// Index → Object kind.
    pub object_table: HashMap<u16, RasterObject>,
    /// Currently selected pen object index.
    pub selected_pen: Option<u16>,
    /// Currently selected brush object index.
    pub selected_brush: Option<u16>,
    /// Currently selected font object index.
    pub selected_font: Option<u16>,
    /// [Stage 21] 현재 clip rectangle (device pixel 좌표). None 시 clip 없음 (canvas 전체).
    pub clip_rect: Option<(i32, i32, i32, i32)>,
}

impl Default for RasterState {
    fn default() -> Self {
        Self {
            window_origin: (0, 0),
            window_ext: (1024, 1024),
            viewport_origin: (0, 0),
            viewport_ext: (1, 1),
            viewport_ext_set: false,
            map_mode: MapMode::MM_TEXT,
            current_position: (0, 0),
            text_align: 0,
            bk_mode: MixMode::TRANSPARENT,
            bk_color: ColorRef::white(),
            text_color: ColorRef::black(),
            poly_fill_mode: PolyFillMode::ALTERNATE,
            rop2: None,
            object_table: HashMap::new(),
            selected_pen: None,
            selected_brush: None,
            selected_font: None,
            clip_rect: None,
        }
    }
}

impl RasterState {
    /// Logical 좌표 → device 좌표 변환.
    /// MM_ANISOTROPIC: device = (logical - WindowOrg) × (ViewportExt / WindowExt) + ViewportOrg
    pub fn logical_to_device(&self, x: i16, y: i16) -> (f32, f32) {
        let dx_logical = f32::from(x - self.window_origin.0);
        let dy_logical = f32::from(y - self.window_origin.1);

        let (sx, sy) = if self.viewport_ext_set {
            (
                f32::from(self.viewport_ext.0)
                    / f32::from(self.window_ext.0.max(1)),
                f32::from(self.viewport_ext.1)
                    / f32::from(self.window_ext.1.max(1)),
            )
        } else {
            (1.0, 1.0)
        };

        (
            dx_logical * sx + f32::from(self.viewport_origin.0),
            dy_logical * sy + f32::from(self.viewport_origin.1),
        )
    }
}

/// WMF graphics object kinds — pen / brush / font / region / palette.
#[derive(Clone, Debug)]
pub enum RasterObject {
    Pen(PenInfo),
    Brush(BrushInfo),
    Font(FontInfo),
    Region,    // 향후 구현
    Palette,   // 향후 구현
}

#[derive(Clone, Debug)]
pub struct PenInfo {
    pub color: ColorRef,
    pub width: i32,
    pub style: u16,
    pub is_null: bool,
}

impl Default for PenInfo {
    fn default() -> Self {
        Self { color: ColorRef::black(), width: 1, style: 0, is_null: false }
    }
}

#[derive(Clone, Debug)]
pub struct BrushInfo {
    pub color: ColorRef,
    pub style: u16,
    pub is_null: bool,
}

impl Default for BrushInfo {
    fn default() -> Self {
        Self { color: ColorRef::white(), style: 0, is_null: false }
    }
}

#[derive(Clone, Debug)]
pub struct FontInfo {
    pub height: i16,
    pub width: i16,
    pub weight: i16,
    pub italic: bool,
    pub underline: bool,
    pub strike_out: bool,
    pub charset: CharacterSet,
    pub facename: String,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            height: 12,
            width: 0,
            weight: 400,
            italic: false,
            underline: false,
            strike_out: false,
            charset: CharacterSet::DEFAULT_CHARSET,
            facename: String::new(),
        }
    }
}
