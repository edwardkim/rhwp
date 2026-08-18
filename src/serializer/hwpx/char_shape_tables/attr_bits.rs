//! HWP5 CHAR_SHAPE `attr` 비트 배치. 직렬화가 필드에서 비트를 다시 조립할 때 쓴다.

/// italic
pub const ATTR_BIT_ITALIC: u32 = 0;
/// bold
pub const ATTR_BIT_BOLD: u32 = 1;
/// underline type, width 2
pub const ATTR_BIT_UNDERLINE_TYPE: u32 = 2;
pub const ATTR_WIDTH_UNDERLINE_TYPE: u32 = 2;
/// underline shape, width 4
pub const ATTR_BIT_UNDERLINE_SHAPE: u32 = 4;
pub const ATTR_WIDTH_UNDERLINE_SHAPE: u32 = 4;
/// outline type, width 3
pub const ATTR_BIT_OUTLINE: u32 = 8;
pub const ATTR_WIDTH_OUTLINE: u32 = 3;
/// shadow type, width 2
pub const ATTR_BIT_SHADOW: u32 = 11;
pub const ATTR_WIDTH_SHADOW: u32 = 2;
/// emboss
pub const ATTR_BIT_EMBOSS: u32 = 13;
/// engrave
pub const ATTR_BIT_ENGRAVE: u32 = 14;
/// superscript
pub const ATTR_BIT_SUPERSCRIPT: u32 = 15;
/// subscript
pub const ATTR_BIT_SUBSCRIPT: u32 = 16;
/// strike style, width 3 (한컴 placeholder 와 혼재)
pub const ATTR_BIT_STRIKE_STYLE: u32 = 18;
pub const ATTR_WIDTH_STRIKE_STYLE: u32 = 3;
/// emphasis dot, width 4
pub const ATTR_BIT_EMPHASIS: u32 = 21;
pub const ATTR_WIDTH_EMPHASIS: u32 = 4;
/// use font space
pub const ATTR_BIT_USE_FONT_SPACE: u32 = 25;
/// strike shape, width 4
pub const ATTR_BIT_STRIKE_SHAPE: u32 = 26;
pub const ATTR_WIDTH_STRIKE_SHAPE: u32 = 4;
/// kerning
pub const ATTR_BIT_KERNING: u32 = 30;

pub fn attr_flag(attr: u32, bit: u32) -> bool {
    (attr >> bit) & 1 == 1
}

pub fn attr_field(attr: u32, offset: u32, width: u32) -> u32 {
    (attr >> offset) & ((1 << width) - 1)
}

pub fn set_attr_flag(attr: &mut u32, bit: u32, enabled: bool) {
    if enabled {
        *attr |= 1 << bit;
    } else {
        *attr &= !(1 << bit);
    }
}

pub fn set_attr_field(attr: &mut u32, offset: u32, width: u32, value: u32) {
    let mask = ((1 << width) - 1) << offset;
    *attr = (*attr & !mask) | ((value << offset) & mask);
}
