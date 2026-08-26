//! HWP5 CHAR_SHAPE / PARA_CHAR_SHAPE 바이너리 배치.

/// CHAR_SHAPE 최소 페이로드 (shadow offsets + 4색 + border + strike).
pub const CHAR_SHAPE_MIN_BYTES: usize = 74;
/// font_ids 시작.
pub const OFF_FONT_IDS: usize = 0;
/// ratios 시작.
pub const OFF_RATIOS: usize = 14;
/// spacings 시작.
pub const OFF_SPACINGS: usize = 21;
/// relative_sizes 시작.
pub const OFF_RELATIVE_SIZES: usize = 28;
/// char_offsets 시작.
pub const OFF_CHAR_OFFSETS: usize = 35;
/// base_size i32.
pub const OFF_BASE_SIZE: usize = 42;
/// attr u32.
pub const OFF_ATTR: usize = 46;
/// shadow_offset_x.
pub const OFF_SHADOW_X: usize = 50;
/// shadow_offset_y.
pub const OFF_SHADOW_Y: usize = 51;
/// text_color.
pub const OFF_TEXT_COLOR: usize = 52;
/// underline_color.
pub const OFF_UNDERLINE_COLOR: usize = 56;
/// shade_color.
pub const OFF_SHADE_COLOR: usize = 60;
/// shadow_color.
pub const OFF_SHADOW_COLOR: usize = 64;
/// border_fill_id u16.
pub const OFF_BORDER_FILL_ID: usize = 68;
/// strike_color.
pub const OFF_STRIKE_COLOR: usize = 70;

/// PARA_CHAR_SHAPE entry 한 칸.
pub const PARA_CHAR_SHAPE_ENTRY_BYTES: usize = 8;

/// 기준 크기 HWPUNIT. 100 = 1pt.
pub const BASE_SIZE_UNITS_PER_PT: i32 = 100;

/// #3500 샘플의 본문 글자 크기(10pt).
pub const ISSUE_3500_BODY_BASE_SIZE: i32 = 1000;
/// #3500 샘플에 같이 들어 있는 9pt 슬롯.
pub const ISSUE_3500_NINE_PT_BASE_SIZE: i32 = 900;

/// 음영 없음 sentinel (`model::color::NONE` 과 동일).
pub const SHADE_NONE: u32 = 0xFFFF_FFFF;
