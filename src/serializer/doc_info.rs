//! DocInfo 스트림 직렬화
//!
//! `parse_doc_info()`의 역방향으로, DocInfo/DocProperties를
//! HWP 레코드 바이너리 스트림으로 변환한다.
//!
//! 직렬화 순서:
//! DOCUMENT_PROPERTIES → ID_MAPPINGS → BIN_DATA → FACE_NAME →
//! BORDER_FILL → CHAR_SHAPE → TAB_DEF → NUMBERING → PARA_SHAPE → STYLE

use super::byte_writer::ByteWriter;
use super::record_writer::write_record;

use crate::model::bin_data::{BinData, BinDataType};
use crate::model::document::{DocInfo, DocProperties};
use crate::model::style::{
    BorderFill, BorderLineType, Bullet, CenterLine, CharShape, FillType, Font, ImageFillMode,
    Numbering, ParaShape, Style, TabDef,
};
use crate::parser::tags;

/// DocInfo + DocProperties를 레코드 바이너리 스트림으로 직렬화
pub fn serialize_doc_info(doc_info: &DocInfo, doc_props: &DocProperties) -> Vec<u8> {
    // 원본 스트림이 있고 변경되지 않았으면 그대로 반환 (완벽한 라운드트립)
    if !doc_info.raw_stream_dirty {
        if let Some(ref raw) = doc_info.raw_stream {
            let mut result = raw.clone();
            // 배포용 문서 해제 시 DISTRIBUTE_DOC_DATA 레코드 제거
            if doc_info.distribute_doc_data_removed {
                surgical_remove_records(&mut result, tags::HWPTAG_DISTRIBUTE_DOC_DATA);
            }
            return result;
        }
    }

    let mut stream = Vec::new();

    // 1. DOCUMENT_PROPERTIES
    stream.extend(write_record(
        tags::HWPTAG_DOCUMENT_PROPERTIES,
        0,
        &serialize_document_properties(doc_props),
    ));

    // 2. ID_MAPPINGS
    stream.extend(write_record(
        tags::HWPTAG_ID_MAPPINGS,
        0,
        &serialize_id_mappings(doc_info),
    ));

    // 3~10: ID_MAPPINGS 하위 레코드 (모두 level 1)
    for bin_data in &doc_info.bin_data_list {
        let data = bin_data
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_bin_data(bin_data));
        stream.extend(write_record(tags::HWPTAG_BIN_DATA, 1, &data));
    }

    for lang_fonts in &doc_info.font_faces {
        for font in lang_fonts {
            let data = font
                .raw_data
                .clone()
                .unwrap_or_else(|| serialize_face_name(font));
            stream.extend(write_record(tags::HWPTAG_FACE_NAME, 1, &data));
        }
    }

    for bf in &doc_info.border_fills {
        let data = bf
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_border_fill(bf));
        stream.extend(write_record(tags::HWPTAG_BORDER_FILL, 1, &data));
    }

    for cs in &doc_info.char_shapes {
        let data = cs
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_char_shape(cs));
        stream.extend(write_record(tags::HWPTAG_CHAR_SHAPE, 1, &data));
    }

    for td in &doc_info.tab_defs {
        let data = td.raw_data.clone().unwrap_or_else(|| serialize_tab_def(td));
        stream.extend(write_record(tags::HWPTAG_TAB_DEF, 1, &data));
    }

    for numbering in &doc_info.numberings {
        let data = numbering
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_numbering(numbering));
        stream.extend(write_record(tags::HWPTAG_NUMBERING, 1, &data));
    }

    for bullet in &doc_info.bullets {
        let data = bullet
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_bullet(bullet));
        stream.extend(write_record(tags::HWPTAG_BULLET, 1, &data));
    }

    for ps in &doc_info.para_shapes {
        let data = ps
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_para_shape(ps));
        stream.extend(write_record(tags::HWPTAG_PARA_SHAPE, 1, &data));
    }

    for style in &doc_info.styles {
        let data = style
            .raw_data
            .clone()
            .unwrap_or_else(|| serialize_style(style));
        stream.extend(write_record(tags::HWPTAG_STYLE, 1, &data));
    }

    // 미지원 레코드 원본 보존
    for record in &doc_info.extra_records {
        stream.extend(write_record(record.tag_id, record.level, &record.data));
    }

    stream
}

// ============================================================
// 개별 레코드 직렬화
// ============================================================

pub fn serialize_document_properties(props: &DocProperties) -> Vec<u8> {
    // raw_data가 있으면 원본 바이트 사용 (라운드트립 보존)
    if let Some(ref raw) = props.raw_data {
        return raw.clone();
    }
    let mut w = ByteWriter::new();
    w.write_u16(props.section_count).unwrap();
    w.write_u16(props.page_start_num).unwrap();
    w.write_u16(props.footnote_start_num).unwrap();
    w.write_u16(props.endnote_start_num).unwrap();
    w.write_u16(props.picture_start_num).unwrap();
    w.write_u16(props.table_start_num).unwrap();
    w.write_u16(props.equation_start_num).unwrap();
    // 캐럿 위치 정보 (스펙: 전체 26바이트)
    w.write_u32(props.caret_list_id).unwrap();
    w.write_u32(props.caret_para_id).unwrap();
    w.write_u32(props.caret_char_pos).unwrap();
    w.into_bytes()
}

pub fn serialize_id_mappings(doc_info: &DocInfo) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // bin_data_count
    w.write_u32(doc_info.bin_data_list.len() as u32).unwrap();

    // font_counts (7개 언어)
    for lang_idx in 0..7 {
        let count = if lang_idx < doc_info.font_faces.len() {
            doc_info.font_faces[lang_idx].len() as u32
        } else {
            0
        };
        w.write_u32(count).unwrap();
    }

    // border_fill_count
    w.write_u32(doc_info.border_fills.len() as u32).unwrap();
    // char_shape_count
    w.write_u32(doc_info.char_shapes.len() as u32).unwrap();
    // tab_def_count
    w.write_u32(doc_info.tab_defs.len() as u32).unwrap();
    // numbering_count
    w.write_u32(doc_info.numberings.len() as u32).unwrap();
    // bullet_count (파싱된 bullets 배열 크기 우선, 없으면 보존값)
    let bullet_count = if doc_info.bullets.is_empty() {
        doc_info.bullet_count
    } else {
        doc_info.bullets.len() as u32
    };
    w.write_u32(bullet_count).unwrap();
    // para_shape_count
    w.write_u32(doc_info.para_shapes.len() as u32).unwrap();
    // style_count
    w.write_u32(doc_info.styles.len() as u32).unwrap();
    // memo_shape_count (5.0.2.x 이후, 파싱 시 보존된 값 사용)
    w.write_u32(doc_info.memo_shape_count).unwrap();
    // 최근 한컴 생성 HWP는 ID_MAPPINGS 뒤쪽 reserved count 2개까지 포함한
    // 18개 u32 테이블을 쓴다. HWPX에서 DocInfo를 새로 구성할 때도 이
    // 길이를 맞춰 한컴의 DocInfo contract와 동일한 형태로 직렬화한다.
    w.write_u32(0).unwrap();
    w.write_u32(0).unwrap();

    w.into_bytes()
}

pub fn serialize_bin_data(bin_data: &BinData) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u16(bin_data.attr).unwrap();

    match bin_data.data_type {
        BinDataType::Link => {
            if let Some(ref abs_path) = bin_data.abs_path {
                w.write_hwp_string(abs_path).unwrap();
            } else {
                w.write_hwp_string("").unwrap();
            }
            if let Some(ref rel_path) = bin_data.rel_path {
                w.write_hwp_string(rel_path).unwrap();
            } else {
                w.write_hwp_string("").unwrap();
            }
        }
        BinDataType::Embedding | BinDataType::Storage => {
            w.write_u16(bin_data.storage_id).unwrap();
            if let Some(ref ext) = bin_data.extension {
                w.write_hwp_string(ext).unwrap();
            } else {
                w.write_hwp_string("").unwrap();
            }
        }
    }

    w.into_bytes()
}

pub fn serialize_face_name(font: &Font) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // attr 바이트 재구성
    let mut attr = font.alt_type & 0x03;
    if font.alt_name.is_some() {
        attr |= 0x80;
    }
    if font.type_info.is_some() {
        attr |= 0x40;
    }
    if font.default_name.is_some() {
        attr |= 0x20;
    }
    w.write_u8(attr).unwrap();

    w.write_hwp_string(&font.name).unwrap();

    if let Some(ref alt_name) = font.alt_name {
        w.write_u8(font.alt_type & 0x03).unwrap();
        w.write_hwp_string(alt_name).unwrap();
    }
    if let Some(type_info) = font.type_info {
        w.write_bytes(&type_info).unwrap();
    }
    if let Some(ref default_name) = font.default_name {
        w.write_hwp_string(default_name).unwrap();
    }

    w.into_bytes()
}

fn border_line_type_to_u8(lt: BorderLineType) -> u8 {
    match lt {
        BorderLineType::None => 0,
        BorderLineType::Solid => 1,
        BorderLineType::Dash => 2,
        BorderLineType::Dot => 3,
        BorderLineType::DashDot => 4,
        BorderLineType::DashDotDot => 5,
        BorderLineType::LongDash => 6,
        BorderLineType::Circle => 7,
        BorderLineType::Double => 8,
        BorderLineType::ThinThickDouble => 9,
        BorderLineType::ThickThinDouble => 10,
        BorderLineType::ThinThickThinTriple => 11,
        BorderLineType::Wave => 12,
        BorderLineType::DoubleWave => 13,
        BorderLineType::Thick3D => 14,
        BorderLineType::Thick3DReverse => 15,
        BorderLineType::Thin3D => 16,
        BorderLineType::Thin3DReverse => 17,
    }
}

fn image_fill_mode_to_u8(mode: ImageFillMode) -> u8 {
    match mode {
        ImageFillMode::TileAll => 0,
        ImageFillMode::TileHorzTop => 1,
        ImageFillMode::TileHorzBottom => 2,
        ImageFillMode::TileVertLeft => 3,
        ImageFillMode::TileVertRight => 4,
        ImageFillMode::Total => 0,
        ImageFillMode::FitToSize => 5,
        ImageFillMode::Center => 6,
        ImageFillMode::CenterTop => 7,
        ImageFillMode::CenterBottom => 8,
        ImageFillMode::LeftCenter => 9,
        ImageFillMode::LeftTop => 10,
        ImageFillMode::LeftBottom => 11,
        ImageFillMode::RightCenter => 12,
        ImageFillMode::RightTop => 13,
        ImageFillMode::RightBottom => 14,
        ImageFillMode::None => 15,
    }
}

pub fn serialize_border_fill(bf: &BorderFill) -> Vec<u8> {
    let mut w = ByteWriter::new();
    let mut attr = bf.attr;
    let center_line = if bf.center_line != CenterLine::None {
        bf.center_line
    } else {
        CenterLine::from_hwp_attr(attr)
    };
    if center_line != CenterLine::None {
        attr &= !((0x07 << 2)
            | (0x07 << 5)
            | (0x03 << 8)
            | (1 << 10)
            | (1 << 11)
            | (1 << 12)
            | (1 << 13));
        attr |= center_line.hwp_binary_attr_bits();
    }
    w.write_u16(attr).unwrap();

    // 4방향 테두리 (인터리브: 종류 + 굵기 + 색상)
    for border in &bf.borders {
        w.write_u8(border_line_type_to_u8(border.line_type))
            .unwrap();
        w.write_u8(border.width).unwrap();
        w.write_color_ref(border.color).unwrap();
    }

    // 대각선
    w.write_u8(bf.diagonal.diagonal_type).unwrap();
    w.write_u8(bf.diagonal.width).unwrap();
    w.write_color_ref(bf.diagonal.color).unwrap();

    // 채우기
    serialize_fill(&mut w, &bf.fill);

    w.into_bytes()
}

fn serialize_fill(w: &mut ByteWriter, fill: &crate::model::style::Fill) {
    let fill_type_val: u32 = match fill.fill_type {
        FillType::None => 0,
        FillType::Solid => 1,
        FillType::Image => 2,
        FillType::Gradient => 4,
    };
    w.write_u32(fill_type_val).unwrap();

    match fill.fill_type {
        FillType::Solid => {
            if let Some(ref solid) = fill.solid {
                w.write_color_ref(solid.background_color).unwrap();
                w.write_color_ref(solid.pattern_color).unwrap();
                w.write_i32(solid.pattern_type).unwrap();
            }
            // 추가 채우기 속성: size(u32) + alpha(u8)
            w.write_u32(1).unwrap();
            w.write_u8(0).unwrap(); // alpha
        }
        FillType::Gradient => {
            if let Some(ref grad) = fill.gradient {
                w.write_u8(grad.gradient_type as u8).unwrap();
                w.write_u32(grad.angle as u32).unwrap();
                w.write_u32(grad.center_x as u32).unwrap();
                w.write_u32(grad.center_y as u32).unwrap();
                w.write_u32(grad.blur as u32).unwrap();
                w.write_u32(grad.colors.len() as u32).unwrap();
                if grad.colors.len() > 2 {
                    for &pos in &grad.positions {
                        w.write_i32(pos).unwrap();
                    }
                }
                for &color in &grad.colors {
                    w.write_color_ref(color).unwrap();
                }
            }
            w.write_u32(1).unwrap();
            w.write_u8(fill.gradient.as_ref().map(|g| g.step_center).unwrap_or(0))
                .unwrap();
            w.write_u8(fill.alpha).unwrap();
        }
        FillType::Image => {
            if let Some(ref img) = fill.image {
                w.write_u8(image_fill_mode_to_u8(img.fill_mode)).unwrap();
                w.write_i8(img.brightness).unwrap();
                w.write_i8(img.contrast).unwrap();
                w.write_u8(img.effect).unwrap();
                w.write_u16(img.bin_data_id).unwrap();
            }
            // 추가 채우기 속성: size(u32)
            w.write_u32(0).unwrap();
        }
        FillType::None => {
            // 추가 채우기 속성: size(u32) = 0
            w.write_u32(0).unwrap();
        }
    }
}

pub fn serialize_char_shape(cs: &CharShape) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // font_ids (7 × u16)
    for &id in &cs.font_ids {
        w.write_u16(id).unwrap();
    }
    // ratios (7 × u8)
    for &ratio in &cs.ratios {
        w.write_u8(ratio).unwrap();
    }
    // spacings (7 × i8)
    for &spacing in &cs.spacings {
        w.write_i8(spacing).unwrap();
    }
    // relative_sizes (7 × u8)
    for &size in &cs.relative_sizes {
        w.write_u8(size).unwrap();
    }
    // char_offsets (7 × i8)
    for &offset in &cs.char_offsets {
        w.write_i8(offset).unwrap();
    }
    // base_size
    w.write_i32(cs.base_size).unwrap();
    // attr: 원본 비트를 기반으로, 모델링된 필드 반영
    let mut attr = cs.attr;
    // bit 0: italic
    if cs.italic {
        attr |= 0x01;
    } else {
        attr &= !0x01;
    }
    // bit 1: bold
    if cs.bold {
        attr |= 0x02;
    } else {
        attr &= !0x02;
    }
    // bits 2-3: underline_type (0=none, 1=bottom, 3=top)
    attr &= !0x0C;
    attr |= match cs.underline_type {
        crate::model::style::UnderlineType::Bottom => 1u32 << 2,
        crate::model::style::UnderlineType::Top => 3u32 << 2,
        crate::model::style::UnderlineType::None => 0,
    };
    // bits 8-10: outline_type (hwplib 기준)
    attr &= !(0x07 << 8);
    attr |= (cs.outline_type as u32 & 0x07) << 8;
    // bits 11-12: shadow_type (hwplib 기준)
    attr &= !(0x03 << 11);
    attr |= (cs.shadow_type as u32 & 0x03) << 11;
    // bit 13: emboss
    if cs.emboss {
        attr |= 1u32 << 13;
    } else {
        attr &= !(1u32 << 13);
    }
    // bit 14: engrave
    if cs.engrave {
        attr |= 1u32 << 14;
    } else {
        attr &= !(1u32 << 14);
    }
    // HWP 스펙 표 37: bit 15 = 위첨자(superscript), bit 16 = 아래첨자(subscript)
    if cs.superscript {
        attr |= 1u32 << 15;
    } else {
        attr &= !(1u32 << 15);
    }
    if cs.subscript {
        attr |= 1u32 << 16;
    } else {
        attr &= !(1u32 << 16);
    }
    // bits 4-7: underline_shape (표 27 선 종류)
    attr &= !(0x0F << 4);
    attr |= (cs.underline_shape as u32 & 0x0F) << 4;
    // bits 18-20: strikethrough (≥2 means active)
    if cs.strikethrough {
        if (attr >> 18) & 0x07 < 2 {
            attr = (attr & !(0x07 << 18)) | (2u32 << 18);
        }
    } else {
        attr &= !(0x07 << 18);
    }
    // bits 21-24: emphasis_dot (강조점 종류)
    attr &= !(0x0F << 21);
    attr |= (cs.emphasis_dot as u32 & 0x0F) << 21;
    // bit 25: use font space
    if cs.use_font_space {
        attr |= 1u32 << 25;
    } else {
        attr &= !(1u32 << 25);
    }
    // bits 26-29: strike_shape (취소선 모양, 표 27 선 종류)
    attr &= !(0x0F << 26);
    attr |= (cs.strike_shape as u32 & 0x0F) << 26;
    // bit 30: kerning
    if cs.kerning {
        attr |= 1u32 << 30;
    } else {
        attr &= !(1u32 << 30);
    }
    w.write_u32(attr).unwrap();
    // shadow offsets (i8 × 2)
    w.write_i8(cs.shadow_offset_x).unwrap();
    w.write_i8(cs.shadow_offset_y).unwrap();
    // colors
    w.write_color_ref(cs.text_color).unwrap();
    w.write_color_ref(cs.underline_color).unwrap();
    w.write_color_ref(cs.shade_color).unwrap();
    w.write_color_ref(cs.shadow_color).unwrap();
    // 글자 테두리/배경 ID (5.0.2.1 이상)
    w.write_u16(cs.border_fill_id).unwrap();
    // 취소선 색 (5.0.3.0 이상)
    w.write_color_ref(cs.strike_color).unwrap();

    w.into_bytes()
}

pub fn serialize_tab_def(td: &TabDef) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u32(td.attr).unwrap();
    w.write_u32(td.tabs.len() as u32).unwrap();
    for tab in &td.tabs {
        w.write_u32(tab.position).unwrap();
        w.write_u8(tab.tab_type).unwrap();
        w.write_u8(tab.fill_type).unwrap();
        w.write_zeros(2).unwrap(); // 예약
    }
    w.into_bytes()
}

fn serialize_numbering(numbering: &Numbering) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // 수준별(1~7) 문단 머리 정보 + 번호 형식 문자열
    for level in 0..7 {
        let head = &numbering.heads[level];
        w.write_u32(head.attr).unwrap();
        w.write_i16(head.width_adjust).unwrap();
        w.write_i16(head.text_distance).unwrap();
        w.write_u32(head.char_shape_id).unwrap();

        // 번호 형식 문자열
        let fmt_str = &numbering.level_formats[level];
        let utf16: Vec<u16> = fmt_str.encode_utf16().collect();
        w.write_u16(utf16.len() as u16).unwrap();
        for &ch in &utf16 {
            w.write_u16(ch).unwrap();
        }
    }

    // 시작 번호
    w.write_u16(numbering.start_number).unwrap();

    // 수준별 시작 번호 (5.0.2.5 이상)
    for level in 0..7 {
        w.write_u32(numbering.level_start_numbers[level]).unwrap();
    }

    w.into_bytes()
}

/// HWPTAG_BULLET 직렬화 (표 44: 글머리표, 20바이트)
fn serialize_bullet(bullet: &Bullet) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // 문단 머리 정보 (8바이트)
    w.write_u32(bullet.attr).unwrap();
    w.write_i16(bullet.width_adjust).unwrap();
    w.write_i16(bullet.text_distance).unwrap();

    // 글머리표 문자 (WCHAR)
    w.write_u16(bullet.bullet_char as u16).unwrap();

    // 이미지 글머리표 여부 (INT32)
    w.write_i32(bullet.image_bullet).unwrap();

    // 이미지 글머리 데이터 (4바이트)
    for &byte in &bullet.image_data {
        w.write_u8(byte).unwrap();
    }

    // 체크 글머리표 문자 (WCHAR)
    w.write_u16(bullet.check_bullet_char as u16).unwrap();

    w.into_bytes()
}

pub fn serialize_para_shape(ps: &ParaShape) -> Vec<u8> {
    let mut w = ByteWriter::new();
    // attr1: 원본 비트를 기반으로, 모델링된 필드 반영
    let mut attr1 = ps.attr1;
    // bits 0-1: line_spacing_type
    attr1 &= !0x03;
    attr1 |= match ps.line_spacing_type {
        crate::model::style::LineSpacingType::Percent => 0,
        crate::model::style::LineSpacingType::Fixed => 1,
        crate::model::style::LineSpacingType::SpaceOnly => 2,
        crate::model::style::LineSpacingType::Minimum => 3,
    };
    // bits 2-4: alignment
    attr1 &= !(0x07 << 2);
    attr1 |= (match ps.alignment {
        crate::model::style::Alignment::Justify => 0u32,
        crate::model::style::Alignment::Left => 1,
        crate::model::style::Alignment::Right => 2,
        crate::model::style::Alignment::Center => 3,
        crate::model::style::Alignment::Distribute => 4,
        crate::model::style::Alignment::Split => 5,
    }) << 2;
    // bits 23-24: head_type
    attr1 &= !(0x03 << 23);
    attr1 |= (match ps.head_type {
        crate::model::style::HeadType::None => 0u32,
        crate::model::style::HeadType::Outline => 1,
        crate::model::style::HeadType::Number => 2,
        crate::model::style::HeadType::Bullet => 3,
    }) << 23;
    // bits 25-27: para_level
    attr1 &= !(0x07 << 25);
    attr1 |= (ps.para_level as u32 & 0x07) << 25;
    w.write_u32(attr1).unwrap();
    w.write_i32(ps.margin_left).unwrap();
    w.write_i32(ps.margin_right).unwrap();
    w.write_i32(ps.indent).unwrap();
    w.write_i32(ps.spacing_before).unwrap();
    w.write_i32(ps.spacing_after).unwrap();
    w.write_i32(ps.line_spacing).unwrap();
    w.write_u16(ps.tab_def_id).unwrap();
    w.write_u16(ps.numbering_id).unwrap();
    w.write_u16(ps.border_fill_id).unwrap();
    for &spacing in &ps.border_spacing {
        w.write_i16(spacing).unwrap();
    }
    // 속성2 (5.0.1.7 이상)
    w.write_u32(ps.attr2).unwrap();
    // 속성3 - 줄 간격 종류 확장 (5.0.2.5 이상)
    w.write_u32(ps.attr3).unwrap();
    // 줄 간격 (5.0.2.5 이상)
    w.write_u32(ps.line_spacing_v2).unwrap();
    // 한컴 편집기가 현재 HWP5 저장 시 붙이는 PARA_SHAPE 말미 4바이트.
    //
    // 공개 스펙 표 43은 전체 길이를 54바이트로 적지만, 한컴이 HWPX를 HWP로
    // 내보낸 정답지들은 PARA_SHAPE를 58바이트로 저장한다. 이 tail이 없으면
    // 한컴 편집기가 일부 masterpage/header 글상자 내부 줄나눔 폭을 다르게
    // 해석해 페이지 번호가 다음 줄로 밀리는 사례가 있다.
    w.write_u32(0).unwrap();
    w.into_bytes()
}

pub fn serialize_style(style: &Style) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_hwp_string(&style.local_name).unwrap();
    w.write_hwp_string(&style.english_name).unwrap();
    w.write_u8(style.style_type).unwrap();
    w.write_u8(style.next_style_id).unwrap();
    // [Task #1058 후속] HWP5 spec 표 47 정합 — lang_id (INT16) 추가.
    // 누락 시 ps_id/cs_id 가 2 byte 앞당겨져 한컴이 잘못된 ParaShape 적용 →
    // 신규 각주 추가 시 본문 paragraph 의 ParaShape (60.0 pt 여백 + 160% 줄간격) 부여.
    let lang_id = if style.lang_id == 0 {
        1042
    } else {
        style.lang_id
    };
    w.write_i16(lang_id).unwrap();
    w.write_u16(style.para_shape_id).unwrap();
    w.write_u16(style.char_shape_id).unwrap();
    // [Task #1058 후속] 한컴 정답지 STYLE record 마지막 2 byte zero — 스펙 미문서화 영역.
    // footnote-01.hwp 의 모든 STYLE record 가 끝에 0x0000 (UINT16) 보유.
    // 누락 시 record size mismatch + DocInfo record 순서 shift.
    w.write_u16(0).unwrap();
    w.into_bytes()
}

// ============================================================
// Surgical Insert/Remove: raw_stream 원본 보존 + 새 레코드 삽입/제거
// ============================================================

/// raw_stream 내 레코드 위치 정보
struct RecordPos {
    tag_id: u16,
    #[allow(dead_code)]
    level: u16,
    data_size: u32,
    /// 레코드 헤더 시작 오프셋
    header_offset: usize,
    /// 레코드 데이터 시작 오프셋
    data_offset: usize,
    /// 레코드 총 바이트 수 (헤더 + [확장크기] + 데이터)
    total_bytes: usize,
}

/// raw_stream을 스캔하여 모든 레코드 위치를 반환
fn scan_records(stream: &[u8]) -> Vec<RecordPos> {
    let mut positions = Vec::new();
    let mut offset = 0;

    while offset + 4 <= stream.len() {
        let header = u32::from_le_bytes([
            stream[offset],
            stream[offset + 1],
            stream[offset + 2],
            stream[offset + 3],
        ]);
        let tag_id = (header & 0x3FF) as u16;
        let level = ((header >> 10) & 0x3FF) as u16;
        let mut size = (header >> 20) as u32;

        let header_bytes;
        let data_offset;
        if size == 0xFFF {
            if offset + 8 > stream.len() {
                break;
            }
            size = u32::from_le_bytes([
                stream[offset + 4],
                stream[offset + 5],
                stream[offset + 6],
                stream[offset + 7],
            ]);
            header_bytes = 8;
            data_offset = offset + 8;
        } else {
            header_bytes = 4;
            data_offset = offset + 4;
        }

        if data_offset + size as usize > stream.len() {
            break;
        }

        positions.push(RecordPos {
            tag_id,
            level,
            data_size: size,
            header_offset: offset,
            data_offset,
            total_bytes: header_bytes + size as usize,
        });

        offset += header_bytes + size as usize;
    }

    positions
}

/// 태그 ID → ID_MAPPINGS 내 필드 오프셋 (바이트)
fn tag_to_id_mappings_offset(tag_id: u16) -> Option<usize> {
    match tag_id {
        tags::HWPTAG_BIN_DATA => Some(0),
        // FACE_NAME은 언어별로 다름 (4~28) → 별도 처리 필요
        tags::HWPTAG_BORDER_FILL => Some(32),
        tags::HWPTAG_CHAR_SHAPE => Some(36),
        tags::HWPTAG_TAB_DEF => Some(40),
        tags::HWPTAG_NUMBERING => Some(44),
        // BULLET => Some(48),
        tags::HWPTAG_PARA_SHAPE => Some(52),
        tags::HWPTAG_STYLE => Some(56),
        _ => None,
    }
}

/// DocInfo raw_stream에 새 레코드를 삽입하고 ID_MAPPINGS 카운트를 갱신한다.
///
/// 원본 스트림의 기존 레코드는 바이트 단위로 완벽히 보존된다.
/// 새 레코드는 동일 tag_id의 마지막 레코드 뒤에 삽입된다.
pub fn surgical_insert_record(
    raw_stream: &mut Vec<u8>,
    tag_id: u16,
    level: u16,
    data: &[u8],
) -> Result<(), String> {
    use super::record_writer::write_record;

    let positions = scan_records(raw_stream);

    // 삽입 위치: 동일 tag_id의 마지막 레코드 뒤
    let insert_offset = if let Some(last) = positions.iter().rev().find(|r| r.tag_id == tag_id) {
        last.header_offset + last.total_bytes
    } else {
        // 동일 tag_id 레코드가 없으면 tag_id 순서에 맞는 위치에 삽입
        if let Some(next) = positions.iter().find(|r| r.tag_id > tag_id) {
            next.header_offset
        } else {
            raw_stream.len()
        }
    };

    // ID_MAPPINGS 위치를 삽입 전에 저장
    let id_mappings_info = positions
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS)
        .map(|r| (r.data_offset, r.data_size));

    // 새 레코드 바이트 생성 및 삽입
    let new_record = write_record(tag_id, level, data);
    let new_len = new_record.len();
    raw_stream.splice(insert_offset..insert_offset, new_record.into_iter());

    // ID_MAPPINGS 카운트 갱신
    if let Some((mut data_off, data_size)) = id_mappings_info {
        if insert_offset <= data_off {
            data_off += new_len;
        }
        if let Some(field_off) = tag_to_id_mappings_offset(tag_id) {
            let abs = data_off + field_off;
            if abs + 4 <= raw_stream.len() && field_off + 4 <= data_size as usize {
                let cur = u32::from_le_bytes([
                    raw_stream[abs],
                    raw_stream[abs + 1],
                    raw_stream[abs + 2],
                    raw_stream[abs + 3],
                ]);
                raw_stream[abs..abs + 4].copy_from_slice(&(cur + 1).to_le_bytes());
            }
        }
    }

    Ok(())
}

/// 동일한 FACE_NAME 레코드를 7개 언어 카테고리 각각의 끝에 삽입한다.
///
/// FACE_NAME 레코드는 언어별로 연속 배치된다:
///   [lang0_font0, lang0_font1, ..., lang1_font0, lang1_font1, ..., lang6_fontN]
/// 각 언어 섹션의 끝에 한 레코드씩 삽입하고 ID_MAPPINGS 카운트를 갱신한다.
pub fn surgical_insert_font_all_langs(raw_stream: &mut Vec<u8>, data: &[u8]) -> Result<(), String> {
    use super::record_writer::write_record;

    let positions = scan_records(raw_stream);

    // ID_MAPPINGS에서 언어별 카운트 읽기
    let id_mappings = positions
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS)
        .ok_or_else(|| "ID_MAPPINGS not found".to_string())?;
    let idm_data_off = id_mappings.data_offset;
    let idm_data_size = id_mappings.data_size as usize;

    let mut lang_counts = [0u32; 7];
    for lang in 0..7 {
        let off = idm_data_off + 4 + lang * 4;
        if off + 4 <= raw_stream.len() && 4 + lang * 4 + 4 <= idm_data_size {
            lang_counts[lang] = u32::from_le_bytes([
                raw_stream[off],
                raw_stream[off + 1],
                raw_stream[off + 2],
                raw_stream[off + 3],
            ]);
        }
    }

    // FACE_NAME 레코드 목록
    let face_recs: Vec<&RecordPos> = positions
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_FACE_NAME)
        .collect();

    // 각 언어 섹션의 끝 오프셋 계산 (뒤에서부터 삽입하기 위해)
    let mut insert_points = Vec::new();
    let mut fn_idx: usize = 0;
    for lang in 0..7 {
        fn_idx += lang_counts[lang] as usize;
        let end_offset = if fn_idx > 0 && fn_idx <= face_recs.len() {
            let rec = face_recs[fn_idx - 1];
            rec.header_offset + rec.total_bytes
        } else if !face_recs.is_empty() {
            let last = face_recs.last().unwrap();
            last.header_offset + last.total_bytes
        } else {
            // FACE_NAME이 없으면 BIN_DATA 뒤 또는 ID_MAPPINGS 뒤
            positions
                .iter()
                .rev()
                .find(|r| r.tag_id == tags::HWPTAG_BIN_DATA)
                .or_else(|| {
                    positions
                        .iter()
                        .find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS)
                })
                .map(|r| r.header_offset + r.total_bytes)
                .unwrap_or(raw_stream.len())
        };
        insert_points.push(end_offset);
    }

    // 뒤에서부터 삽입 (앞쪽 오프셋이 변하지 않도록)
    let new_record = write_record(tags::HWPTAG_FACE_NAME, 1, data);
    for &point in insert_points.iter().rev() {
        raw_stream.splice(point..point, new_record.iter().cloned());
    }

    // ID_MAPPINGS 재스캔 후 7개 언어 카운트 각각 +1
    let positions = scan_records(raw_stream);
    if let Some(idm) = positions
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_ID_MAPPINGS)
    {
        for lang in 0..7usize {
            let field_off = 4 + lang * 4;
            let abs = idm.data_offset + field_off;
            if abs + 4 <= raw_stream.len() && field_off + 4 <= idm.data_size as usize {
                let cur = u32::from_le_bytes([
                    raw_stream[abs],
                    raw_stream[abs + 1],
                    raw_stream[abs + 2],
                    raw_stream[abs + 3],
                ]);
                raw_stream[abs..abs + 4].copy_from_slice(&(cur + 1).to_le_bytes());
            }
        }
    }

    Ok(())
}

/// DocInfo raw_stream에서 특정 tag_id의 모든 레코드를 제거한다.
///
/// convert_to_editable()에서 DISTRIBUTE_DOC_DATA 제거 시 사용.
pub fn surgical_remove_records(raw_stream: &mut Vec<u8>, tag_id: u16) -> usize {
    let positions = scan_records(raw_stream);
    let mut removed = 0;

    // 뒤에서부터 제거 (앞쪽 오프셋이 변하지 않도록)
    for pos in positions.iter().rev().filter(|r| r.tag_id == tag_id) {
        let start = pos.header_offset;
        let end = start + pos.total_bytes;
        raw_stream.drain(start..end);
        removed += 1;
    }

    removed
}

/// DocInfo raw_stream 내 DOCUMENT_PROPERTIES 레코드의 캐럿 위치만 갱신한다.
///
/// raw_stream 전체를 재직렬화하지 않고, 캐럿 위치 3필드(12바이트)만 in-place 수정.
/// DocProperties 레코드 구조:
///   offset 0-1:  section_count (u16)
///   offset 2-13: page/footnote/endnote/picture/table/equation_start_num (u16 × 6)
///   offset 14-17: caret_list_id (u32)
///   offset 18-21: caret_para_id (u32)
///   offset 22-25: caret_char_pos (u32)
pub fn surgical_update_caret(
    raw_stream: &mut Vec<u8>,
    caret_list_id: u32,
    caret_para_id: u32,
    caret_char_pos: u32,
) -> Result<(), String> {
    let positions = scan_records(raw_stream);

    let doc_props_pos = positions
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_DOCUMENT_PROPERTIES)
        .ok_or_else(|| "DOCUMENT_PROPERTIES 레코드를 찾을 수 없음".to_string())?;

    let data_off = doc_props_pos.data_offset;
    if doc_props_pos.data_size < 26 {
        return Err(format!(
            "DOCUMENT_PROPERTIES 데이터 크기 부족: {} < 26",
            doc_props_pos.data_size
        ));
    }

    // 캐럿 위치 필드 업데이트 (offset 14-25)
    raw_stream[data_off + 14..data_off + 18].copy_from_slice(&caret_list_id.to_le_bytes());
    raw_stream[data_off + 18..data_off + 22].copy_from_slice(&caret_para_id.to_le_bytes());
    raw_stream[data_off + 22..data_off + 26].copy_from_slice(&caret_char_pos.to_le_bytes());

    Ok(())
}

#[cfg(test)]
mod tests;
