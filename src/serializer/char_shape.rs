//! HWP5 CHAR_SHAPE 레코드 직렬화.
//!
//! 원본 HWP 레코드는 `raw_data` 우선 경로로 보존된다. 이 모듈은 raw payload가 없는
//! HWPX/IR 유래 `CharShape`를 HWP5 bit field로만 재인코딩한다.

use super::byte_writer::ByteWriter;
use crate::model::style::{CharShape, UnderlineType};

pub fn serialize_char_shape(cs: &CharShape) -> Vec<u8> {
    let mut w = ByteWriter::new();
    write_char_shape(cs, &mut w).expect("Vec binary writer");
    w.into_bytes()
}

pub(crate) fn write_char_shape<W: std::io::Write>(
    cs: &CharShape,
    w: &mut ByteWriter<W>,
) -> std::io::Result<()> {
    for &id in &cs.font_ids {
        w.write_u16(id)?;
    }
    for &ratio in &cs.ratios {
        w.write_u8(ratio)?;
    }
    for &spacing in &cs.spacings {
        w.write_i8(spacing)?;
    }
    for &size in &cs.relative_sizes {
        w.write_u8(size)?;
    }
    for &offset in &cs.char_offsets {
        w.write_i8(offset)?;
    }
    w.write_i32(cs.base_size)?;

    let mut attr = cs.attr;
    set_flag(&mut attr, 0, cs.italic);
    set_flag(&mut attr, 1, cs.bold);

    write_field(
        &mut attr,
        2,
        2,
        match cs.underline_type {
            UnderlineType::Bottom => 1,
            UnderlineType::Top => 3,
            UnderlineType::None => 0,
        },
    );
    write_field(&mut attr, 4, 4, cs.underline_shape as u32);
    write_field(&mut attr, 8, 3, cs.outline_type as u32);
    write_field(&mut attr, 11, 2, cs.shadow_type as u32);
    set_flag(&mut attr, 13, cs.emboss);
    set_flag(&mut attr, 14, cs.engrave);
    set_flag(&mut attr, 15, cs.superscript);
    set_flag(&mut attr, 16, cs.subscript);

    if cs.strikethrough {
        let style = ((attr >> 18) & 0x07).max(2);
        write_field(&mut attr, 18, 3, style);
    } else {
        write_field(&mut attr, 18, 3, 0);
    }
    write_field(&mut attr, 26, 4, cs.strike_shape as u32);
    write_field(&mut attr, 21, 4, cs.emphasis_dot as u32);
    set_flag(&mut attr, 25, cs.use_font_space);
    set_flag(&mut attr, 30, cs.kerning);
    w.write_u32(attr)?;

    w.write_i8(cs.shadow_offset_x)?;
    w.write_i8(cs.shadow_offset_y)?;
    w.write_color_ref(cs.text_color)?;
    w.write_color_ref(cs.underline_color)?;
    w.write_color_ref(cs.shade_color)?;
    w.write_color_ref(cs.shadow_color)?;
    w.write_u16(cs.border_fill_id)?;
    w.write_color_ref(cs.strike_color)?;

    Ok(())
}

fn set_flag(attr: &mut u32, bit: u32, enabled: bool) {
    if enabled {
        *attr |= 1 << bit;
    } else {
        *attr &= !(1 << bit);
    }
}

fn write_field(attr: &mut u32, offset: u32, width: u32, value: u32) {
    let mask = ((1 << width) - 1) << offset;
    *attr = (*attr & !mask) | ((value << offset) & mask);
}
