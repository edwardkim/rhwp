//! HWP3 파일 파서 메인 모듈
//! 
//! HWP3(.hwp) 문서 포맷을 읽고 파싱하여 애플리케이션의 공통 문서 모델로 변환한다.
//! 문서 정보, 요약, 문단, 스타일 등을 종합적으로 처리하는 진입점 역할을 한다.
use crate::model::document::Document;
use snafu::Snafu;
use std::io::{self, Cursor, Read};

pub mod records;
pub mod paragraph;
pub mod johab_map;
pub mod johab;
pub mod special_char;
pub mod encoding;
pub mod drawing;
pub mod ole;
use records::{Hwp3DocInfo, Hwp3DocSummary};
use paragraph::{Hwp3LineInfo, Hwp3ParaInfo};
use special_char::Hwp3SpecialChar;

#[derive(Debug, Snafu)]
pub enum Hwp3Error {
    #[snafu(display("파일 크기가 너무 작습니다."))]
    FileTooSmall,
    #[snafu(display("지원하지 않는 HWP 3.0 기능입니다: {}", feature))]
    UnsupportedFeature { feature: String },
    #[snafu(display("잘못된 파일 시그니처입니다."))]
    InvalidSignature,
    #[snafu(display("입출력 오류가 발생했습니다: {}", source))]
    IoError { source: io::Error },
    #[snafu(display("파싱 오류가 발생했습니다: {}", message))]
    ParseError { message: String },
    #[snafu(display("특수 문자 파싱 오류가 발생했습니다: {:?}", source))]
    SpecialCharError { source: special_char::Hwp3SpecialCharError },
}

impl From<io::Error> for Hwp3Error {
    fn from(error: io::Error) -> Self {
        Hwp3Error::IoError { source: error }
    }
}

impl From<special_char::Hwp3SpecialCharError> for Hwp3Error {
    fn from(error: special_char::Hwp3SpecialCharError) -> Self {
        Hwp3Error::SpecialCharError { source: error }
    }
}

/// HWP3 개체의 CommonObjAttr 필드들에서 HWP5 attr 비트필드를 계산한다.
/// serialize_common_obj_attr이 common.attr 값을 직접 기록하므로,
/// 필드를 설정한 뒤 반드시 이 함수로 attr을 갱신해야 저장→재열기 후 속성이 유지된다.
fn build_common_obj_attr(common: &crate::model::shape::CommonObjAttr) -> u32 {
    use crate::model::shape::{VertRelTo, HorzRelTo, VertAlign, HorzAlign, TextWrap};
    let mut attr: u32 = 0;
    if common.treat_as_char { attr |= 0x01; }
    attr |= (match common.vert_rel_to {
        VertRelTo::Paper  => 0u32,
        VertRelTo::Page   => 1,
        VertRelTo::Para   => 2,
    }) << 3;
    attr |= (match common.vert_align {
        VertAlign::Top     => 0u32,
        VertAlign::Center  => 1,
        VertAlign::Bottom  => 2,
        VertAlign::Inside  => 3,
        VertAlign::Outside => 4,
    }) << 5;
    attr |= (match common.horz_rel_to {
        HorzRelTo::Paper  => 0u32,
        HorzRelTo::Page   => 1,
        HorzRelTo::Column => 2,
        HorzRelTo::Para   => 3,
    }) << 8;
    attr |= (match common.horz_align {
        HorzAlign::Left    => 0u32,
        HorzAlign::Center  => 1,
        HorzAlign::Right   => 2,
        HorzAlign::Inside  => 3,
        HorzAlign::Outside => 4,
    }) << 10;
    attr |= (match common.text_wrap {
        TextWrap::Square        => 0u32,
        TextWrap::TopAndBottom  => 1,
        TextWrap::BehindText    => 2,
        TextWrap::InFrontOfText => 3,
        _                       => 0,
    }) << 21;
    attr
}

fn build_raw_ctrl_data(common: &crate::model::shape::CommonObjAttr) -> Vec<u8> {
    let mut data = Vec::with_capacity(42);
    data.extend_from_slice(&common.attr.to_le_bytes());
    data.extend_from_slice(&common.vertical_offset.to_le_bytes());
    data.extend_from_slice(&common.horizontal_offset.to_le_bytes());
    data.extend_from_slice(&common.width.to_le_bytes());
    data.extend_from_slice(&common.height.to_le_bytes());
    data.extend_from_slice(&common.z_order.to_le_bytes());
    data.extend_from_slice(&common.margin.left.to_le_bytes());
    data.extend_from_slice(&common.margin.right.to_le_bytes());
    data.extend_from_slice(&common.margin.top.to_le_bytes());
    data.extend_from_slice(&common.margin.bottom.to_le_bytes());
    data.extend_from_slice(&common.instance_id.to_le_bytes());
    data.extend_from_slice(&common.prevent_page_break.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes()); // empty description
    data
}

pub(crate) fn convert_char_shape(hwp3_cs: &crate::parser::hwp3::records::Hwp3CharShape) -> crate::model::style::CharShape {
    let mut cs = crate::model::style::CharShape::default();
    // HWP 3.0에서 크기는 pt당 25 단위로 주어집니다. 내부 모델의 base_size는 HWPUNIT(pt당 100 단위)입니다.
    // 따라서 size * 4를 하면 올바른 base_size를 얻을 수 있습니다.

    cs.base_size = (hwp3_cs.size as i32) * 4;
    cs.font_ids = [
        hwp3_cs.font_indices[0] as u16,
        hwp3_cs.font_indices[1] as u16,
        hwp3_cs.font_indices[2] as u16,
        hwp3_cs.font_indices[3] as u16,
        hwp3_cs.font_indices[4] as u16,
        hwp3_cs.font_indices[5] as u16,
        hwp3_cs.font_indices[6] as u16,
    ];
    cs.ratios = hwp3_cs.ratios;
    cs.spacings = hwp3_cs.spacings;
    cs.attr = hwp3_cs.attr as u32;
    cs.italic = hwp3_cs.is_italic();
    cs.bold = hwp3_cs.is_bold();
    cs.underline_type = if hwp3_cs.is_underline() { crate::model::style::UnderlineType::Bottom } else { crate::model::style::UnderlineType::None };
    cs.outline_type = if hwp3_cs.is_outline() { 1 } else { 0 };
    cs.shadow_type = if hwp3_cs.is_shadow() { 1 } else { 0 };
    cs
}

pub(crate) fn convert_para_shape(hwp3_ps: &crate::parser::hwp3::records::Hwp3ParaShape) -> crate::model::style::ParaShape {
    let mut ps = crate::model::style::ParaShape::default();
    // HWP 3.0에서 여백과 들여쓰기는 hunit(1/1800 인치) 또는 shunit 단위로 제공됩니다.
    // 내부 모델은 HWPUNIT(1/7200 인치)을 사용합니다.
    // 따라서 4를 곱합니다.
    ps.margin_left = (hwp3_ps.left_margin as i32) * 4;
    ps.margin_right = (hwp3_ps.right_margin as i32) * 4;
    ps.indent = (hwp3_ps.indent as i32) * 4;
    
    // 줄 간격: MSB가 1이면 hunit 단위의 절대 간격을 의미하고, 그 외에는 퍼센트를 의미합니다.
    if (hwp3_ps.line_spacing & 0x8000) != 0 {
        ps.line_spacing_type = crate::model::style::LineSpacingType::Fixed;
        ps.line_spacing = ((hwp3_ps.line_spacing & 0x7FFF) as i32) * 4;
    } else {
        ps.line_spacing_type = crate::model::style::LineSpacingType::Percent;
        ps.line_spacing = hwp3_ps.line_spacing as i32;
    }
    
    ps.spacing_after = (hwp3_ps.margin_bottom as i32) * 4;
    ps.spacing_before = (hwp3_ps.margin_top as i32) * 4;
    ps.alignment = match hwp3_ps.align {
        0 => crate::model::style::Alignment::Justify,
        1 => crate::model::style::Alignment::Left,
        2 => crate::model::style::Alignment::Right,
        3 => crate::model::style::Alignment::Center,
        4 => crate::model::style::Alignment::Distribute,
        5 => crate::model::style::Alignment::Split,
        _ => crate::model::style::Alignment::Justify,
    };

    ps
}

pub(crate) fn parse_paragraph_list(
    body_cursor: &mut Cursor<&[u8]>,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    pic_name_to_id: &mut std::collections::HashMap<String, u16>,
    body_left_hu: i32,
    column_width_hu: i32,
) -> Result<Vec<crate::model::paragraph::Paragraph>, Hwp3Error> {
    use crate::model::paragraph::{Paragraph, LineSeg, CharShapeRef};
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Read;
    
    let mut paragraphs = Vec::new();
    let mut current_para_shape_id = 0u16;
    let mut prev_para_had_flags_break: bool = false;
    let mut prev_last_pgy: u16 = 0;
    // Square wrap 그림 어울림 구역: (column_start, segment_width, pgy_start, pgy_end)
    // 떠다니는 Square wrap 그림 문단을 만나면 갱신, pgy가 pgy_end를 넘으면 초기화.
    let mut active_wrap_zone: Option<(i32, i32, u16, u16)> = None;
    // [Task #604 Stage A+D] Square wrap 그림 영역 끝 vpos (HU, section 누적 절대값).
    // wrap zone 안 LineSeg 의 vpos 가 본 값을 넘으면 cs/sw=0/full 전환.
    // 0 = wrap zone 비활성. 새 그림 만나면 (anchor 시작 vpos + total_height) 로 갱신.
    let mut wrap_zone_end_vpos: i32 = 0;
    // [Task #604 Stage D-2] active wrap zone cs/sw — 후속 paragraph 가 wrap zone 안일 때
    // 본 cs/sw 로 cs/sw=0 LineSeg 을 정합 채움 (HWP3 의 pgy-based 검출 실패 보완).
    let mut active_wrap_cs_sw: Option<(i32, i32)> = None;
    // section 단위 누적 vpos (HWP5 IR 표준: page 상단 기준 절대값).
    // 새 paragraph 시작 시 이 값을 첫 LineSeg vpos 로 사용.
    let mut acc_section_vpos: i32 = 0;
    // body_left/right_hu 는 column_width_hu 로 사용
    let _section_acc_marker = 0;

    loop {
        let para_start_pos = body_cursor.position();
        let para_info = Hwp3ParaInfo::read(&mut *body_cursor)?;
        if para_info.char_count == 0 {
            break; // 빈 문단, 리스트 끝
        }

        if para_info.follow_prev_para_shape == 0 {
            if let Some(ref hwp3_ps) = para_info.para_shape {
                let mut ps = convert_para_shape(hwp3_ps);
                if hwp3_ps.shade_ratio > 0 {
                    let ratio = hwp3_ps.shade_ratio.min(100) as u32;
                    let gray = (255 * (100 - ratio) / 100) as u8;
                    let color = u32::from_le_bytes([gray, gray, gray, 0]);
                    let mut bf = crate::model::style::BorderFill::default();
                    bf.fill.fill_type = crate::model::style::FillType::Solid;
                    bf.fill.solid = Some(crate::model::style::SolidFill {
                        background_color: color,
                        pattern_color: 0,
                        pattern_type: 0,
                    });
                    doc_border_fills.push(bf);
                    ps.border_fill_id = doc_border_fills.len() as u16; // 1-based (렌더러 규칙)
                }
                doc_para_shapes.push(ps);
                current_para_shape_id = (doc_para_shapes.len() - 1) as u16;
            }
        }
        let para_shape_id = current_para_shape_id;
        
        doc_char_shapes.push(convert_char_shape(&para_info.rep_char_shape));
        let rep_char_shape_id = (doc_char_shapes.len() - 1) as u16;

        let mut line_infos = Vec::with_capacity(para_info.line_count as usize);
        for _ in 0..para_info.line_count {
            line_infos.push(Hwp3LineInfo::read(&mut *body_cursor)?);
        }

        let mut hwp3_inline_shapes = Vec::new();
        if para_info.include_char_shape != 0 {
            for i in 0..para_info.char_count {
                let flag = body_cursor.read_u8().map_err(|e| Hwp3Error::IoError { source: e })?;
                if flag != 1 {
                    use crate::parser::hwp3::records::Hwp3CharShape;
                    let shape = Hwp3CharShape::read(&mut *body_cursor)?;
                    doc_char_shapes.push(convert_char_shape(&shape));
                    let shape_id = (doc_char_shapes.len() - 1) as u16;
                    hwp3_inline_shapes.push((i as usize, shape_id));
                }
            }
        }

        let mut controls = Vec::new();

        let mut ctrl_data_records = Vec::new();
        let mut text_string = String::new();
        let mut char_offsets = Vec::with_capacity(para_info.char_count as usize);
        let mut hwp3_char_to_utf16_pos = vec![0; para_info.char_count as usize];
        let mut utf16_len = 0;

        let mut i = 0;
        while i < para_info.char_count as usize {
            if i < hwp3_char_to_utf16_pos.len() {
                hwp3_char_to_utf16_pos[i] = utf16_len;
            }
            let ch_pos = body_cursor.position();
            let ch = body_cursor.read_u16::<LittleEndian>().map_err(|e| Hwp3Error::IoError { source: e })?;

            i += 1;

            if ch > 0 && ch <= 31 && ch != 13 {
                match ch {
                    30 | 31 => {
                        let mut buf = [0u8; 2];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        if i < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i] = utf16_len; }
                        i += 1;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push(if ch == 30 { '\u{00A0}' } else { ' ' });
                    }
                    24 | 25 => {
                        let mut buf = [0u8; 4];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..2usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 2;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('-'); 
                    }
                    9 => {
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\t');
                    }
                    18..=21 => {
                        let mut buf = [0u8; 6];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..3usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 3;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        // AutoNumber(ch=18)은 HWP5 패턴("  ")과 일치하도록 공백으로 저장
                        if ch == 18 {
                            text_string.push(' ');
                        } else {
                            text_string.push('\u{FFFC}');
                        }

                        let ctrl = match ch {
                            18 => {
                                let mut auto_num = crate::model::control::AutoNumber::default();
                                let n_type = (&buf[0..2]).read_u16::<LittleEndian>().unwrap_or(0);
                                auto_num.number_type = match n_type {
                                    1 => crate::model::control::AutoNumberType::Footnote,
                                    2 => crate::model::control::AutoNumberType::Endnote,
                                    3 => crate::model::control::AutoNumberType::Picture,
                                    4 => crate::model::control::AutoNumberType::Table,
                                    5 => crate::model::control::AutoNumberType::Equation,
                                    _ => crate::model::control::AutoNumberType::Page,
                                };
                                auto_num.number = (&buf[2..4]).read_u16::<LittleEndian>().unwrap_or(0);
                                crate::model::control::Control::AutoNumber(auto_num)
                            },
                            19 => {
                                let mut new_num = crate::model::control::NewNumber::default();
                                let n_type = (&buf[0..2]).read_u16::<LittleEndian>().unwrap_or(0);
                                new_num.number_type = match n_type {
                                    1 => crate::model::control::AutoNumberType::Footnote,
                                    2 => crate::model::control::AutoNumberType::Endnote,
                                    3 => crate::model::control::AutoNumberType::Picture,
                                    4 => crate::model::control::AutoNumberType::Table,
                                    5 => crate::model::control::AutoNumberType::Equation,
                                    _ => crate::model::control::AutoNumberType::Page,
                                };
                                new_num.number = (&buf[2..4]).read_u16::<LittleEndian>().unwrap_or(0);
                                crate::model::control::Control::NewNumber(new_num)
                            },
                            20 => {
                                let mut pos = crate::model::control::PageNumberPos::default();
                                pos.position = (&buf[0..2]).read_u16::<LittleEndian>().unwrap_or(0) as u8;
                                let format_code = (&buf[2..4]).read_u16::<LittleEndian>().unwrap_or(0) as u8;
                                match format_code {
                                    0 => pos.format = 0, // 숫자
                                    1 => pos.format = 2, // 대문자 로마자
                                    2 => pos.format = 3, // 소문자 로마자
                                    3 => { pos.format = 0; pos.dash_char = '-'; },
                                    4 => { pos.format = 2; pos.dash_char = '-'; },
                                    5 => { pos.format = 3; pos.dash_char = '-'; },
                                    _ => pos.format = 0,
                                }
                                crate::model::control::Control::PageNumberPos(pos)
                            },
                            21 => {
                                let kind = (&buf[0..2]).read_u16::<LittleEndian>().unwrap_or(0);
                                if kind == 1 {
                                    let mut hide = crate::model::control::PageHide::default();
                                    let flags = (&buf[2..4]).read_u16::<LittleEndian>().unwrap_or(0);
                                    hide.hide_header = (flags & 1) != 0;
                                    hide.hide_footer = (flags & 2) != 0;
                                    hide.hide_page_num = (flags & 4) != 0;
                                    hide.hide_border = (flags & 8) != 0;
                                    crate::model::control::Control::PageHide(hide)
                                } else {
                                    crate::model::control::Control::Unknown(crate::model::control::UnknownControl { ctrl_id: ch as u32 })
                                }
                            },
                            _ => crate::model::control::Control::Unknown(crate::model::control::UnknownControl { ctrl_id: ch as u32 }),
                        };
                        controls.push(ctrl);
                        ctrl_data_records.push(None);
                    }
                    7 | 8 => {
                        let mut buf = [0u8; 6];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..3usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 3;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\u{FFFC}');
                        controls.push(crate::model::control::Control::Unknown(crate::model::control::UnknownControl { ctrl_id: ch as u32 }));
                        ctrl_data_records.push(None);
                    }
                    23 => {
                        let mut buf = [0u8; 8];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..4usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 4;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\u{FFFC}');
                        let mut overlap = crate::model::control::CharOverlap::default();
                        // buf[0..2] 또는 buf[2..8]은 문자와 테두리 종류를 포함할 수 있습니다.
                        // 가능한 부분을 매핑하지만, 테스트 없이 정확한 오프셋을 찾기는 까다로우므로
                        // 구조체는 유지하되 완벽하게 채우지 않을 수도 있습니다.
                        controls.push(crate::model::control::Control::CharOverlap(overlap));
                        ctrl_data_records.push(None);
                    }
                    22 => {
                        let mut buf = [0u8; 22];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..11usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 11;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\u{FFFC}');
                        let name_buf = &buf[2..22];
                        let name = crate::parser::hwp3::encoding::decode_hwp3_string(name_buf).trim_end_matches('\0').to_string();
                        let mut field = crate::model::control::Field::default();
                        field.field_type = crate::model::control::FieldType::MailMerge;
                        field.command = name;
                        controls.push(crate::model::control::Control::Field(field));
                        ctrl_data_records.push(None);
                    }
                    26 => {
                        let mut buf = [0u8; 244];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..122usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 122;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\u{FFFC}');
                        
                        let kw1_bytes = &buf[0..120];
                        let kw2_bytes = &buf[120..240];
                        
                        let mut field = crate::model::control::Field::default();
                        field.field_type = crate::model::control::FieldType::Unknown;
                        field.command = format!("IndexMark:{}:{}",
                            crate::parser::hwp3::encoding::decode_hwp3_string(kw1_bytes).trim_end_matches('\0'),
                            crate::parser::hwp3::encoding::decode_hwp3_string(kw2_bytes).trim_end_matches('\0')
                        );
                        
                        controls.push(crate::model::control::Control::Field(field));
                        ctrl_data_records.push(None);
                    }
                    28 => {
                        let mut buf = [0u8; 62];
                        if let Err(_) = body_cursor.read_exact(&mut buf) { break; }
                        for k in 0..31usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 31;
                        char_offsets.push(utf16_len);
                        utf16_len += 1;
                        text_string.push('\u{FFFC}');
                        
                        let kind = (&buf[0..2]).read_u16::<LittleEndian>().unwrap_or(0);
                        let shape = buf[2];
                        let level = buf[3];
                        
                        let mut field = crate::model::control::Field::default();
                        field.field_type = crate::model::control::FieldType::Unknown;
                        field.command = format!("Outline:kind={}:shape={}:level={}", kind, shape, level);
                        
                        controls.push(crate::model::control::Control::Field(field));
                        ctrl_data_records.push(None);
                    }
                    _ => {
                        let header_val1 = match body_cursor.read_u32::<LittleEndian>() {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        let _ch2 = match body_cursor.read_u16::<LittleEndian>() {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        for k in 0..3usize { if i + k < hwp3_char_to_utf16_pos.len() { hwp3_char_to_utf16_pos[i + k] = utf16_len; } }
                        i += 3; // 8바이트 헤더는 char_count에서 4개의 hchar를 차지합니다 (여기서 1개 읽고 3개 건너뜀)
                        
                        let mut nested_paragraphs = Vec::new();
                        let mut parsed_table = None;
                        let mut parsed_equation = None;
                        let mut parsed_picture = None;
                        let mut parsed_line = None;
                        let mut parsed_drawing_object: Option<crate::model::shape::ShapeObject> = None;
                        let mut parsed_obj_type = 0;
                        let mut parsed_is_hypertext = false;
                        
                        let mut info_buf = Vec::new();

                        if ch == 10 { // 표 / 글상자 / 수식 / 버튼
                            info_buf.resize(84, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            let obj_type = if info_buf.len() >= 80 { (&info_buf[78..80]).read_u16::<LittleEndian>().unwrap_or(0) } else { 0 };
                            let other_options = if info_buf.len() >= 16 { (&info_buf[14..16]).read_u16::<LittleEndian>().unwrap_or(0) } else { 0 };
                            parsed_obj_type = obj_type;
                            parsed_is_hypertext = (other_options & 0x10) != 0;
                            let cell_count = if info_buf.len() >= 82 { (&info_buf[80..82]).read_u16::<LittleEndian>().unwrap_or(1) } else { 1 };
                            
                            // 이들은 모두 같은 구조를 가집니다: 84바이트 정보 -> 각 셀당 27바이트 -> 셀당 문단 리스트 -> 캡션 문단.
                            let mut table = crate::model::table::Table::default();
                                
                                table.outer_margin_left = (&info_buf[18..20]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.outer_margin_right = (&info_buf[20..22]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.outer_margin_top = (&info_buf[22..24]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.outer_margin_bottom = (&info_buf[24..26]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.common.margin.left = table.outer_margin_left;
                                table.common.margin.right = table.outer_margin_right;
                                table.common.margin.top = table.outer_margin_top;
                                table.common.margin.bottom = table.outer_margin_bottom;

                                table.padding.left = (&info_buf[26..28]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.padding.right = (&info_buf[28..30]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.padding.top = (&info_buf[30..32]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                                table.padding.bottom = (&info_buf[32..34]).read_i16::<LittleEndian>().unwrap_or(0) * 4;

                                table.common.width = ((&info_buf[42..44]).read_u16::<LittleEndian>().unwrap_or(0) as u32) * 4;
                                table.common.height = ((&info_buf[44..46]).read_u16::<LittleEndian>().unwrap_or(0) as u32) * 4;
                                
                                let ref_pos = info_buf[8];
                                table.common.treat_as_char = ref_pos == 0;
                                match ref_pos {
                                    1 => {
                                        table.common.horz_rel_to = crate::model::shape::HorzRelTo::Para;
                                        table.common.vert_rel_to = crate::model::shape::VertRelTo::Para;
                                    },
                                    2 => {
                                        table.common.horz_rel_to = crate::model::shape::HorzRelTo::Page;
                                        table.common.vert_rel_to = crate::model::shape::VertRelTo::Page;
                                    },
                                    3 => {
                                        table.common.horz_rel_to = crate::model::shape::HorzRelTo::Paper;
                                        table.common.vert_rel_to = crate::model::shape::VertRelTo::Paper;
                                    },
                                    _ => {}
                                }

                                // 그림 피함(offset 9): 0=자리차지(TopAndBottom), 1=투명, 2=어울림
                                let text_wrap = info_buf[9];
                                // table.common.treat_as_char remains ref_pos == 0
                                table.common.text_wrap = match text_wrap {
                                    0 => crate::model::shape::TextWrap::TopAndBottom, // 자리차지
                                    1 => crate::model::shape::TextWrap::BehindText, // 투명 (글자 뒤)
                                    2 => crate::model::shape::TextWrap::Square, // 어울림
                                    _ => crate::model::shape::TextWrap::Square,
                                };

                                let horz_align = (&info_buf[10..12]).read_i16::<LittleEndian>().unwrap_or(0);
                                if horz_align == -1 {
                                    table.common.horz_align = crate::model::shape::HorzAlign::Left;
                                } else if horz_align == -2 {
                                    table.common.horz_align = crate::model::shape::HorzAlign::Right;
                                } else if horz_align == -3 {
                                    table.common.horz_align = crate::model::shape::HorzAlign::Center;
                                } else {
                                    table.common.horz_align = crate::model::shape::HorzAlign::Left;
                                    table.common.horizontal_offset = (horz_align as i32 * 4) as u32;
                                }

                                let vert_align = (&info_buf[12..14]).read_i16::<LittleEndian>().unwrap_or(0);
                                if vert_align == -1 {
                                    table.common.vert_align = crate::model::shape::VertAlign::Top;
                                } else if vert_align == -2 {
                                    table.common.vert_align = crate::model::shape::VertAlign::Bottom;
                                } else if vert_align == -3 {
                                    table.common.vert_align = crate::model::shape::VertAlign::Center;
                                } else {
                                    table.common.vert_align = crate::model::shape::VertAlign::Top;
                                    table.common.vertical_offset = (vert_align as i32 * 4) as u32;
                                }
                                table.common.attr = build_common_obj_attr(&table.common);
                                // typeset.rs는 table.attr(=common.attr)로 is_tac/text_wrap을 판정한다.
                                // HWP5 파서도 table.attr = table.common.attr 로 동기화하므로 동일하게 설정한다.
                                table.attr = table.common.attr;
                                // HWP5 저장 시 serialize_table이 raw_ctrl_data를 그대로 기록한다.
                                // 미리 채워두면 serializer/hwpx_to_hwp 수정 없이 attr가 올바르게 저장된다.
                                table.raw_ctrl_data = build_raw_ctrl_data(&table.common);

                                let cell_padding_left = (&info_buf[34..36]).read_i16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                                let cell_padding_right = (&info_buf[36..38]).read_i16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                                let cell_padding_top = (&info_buf[38..40]).read_i16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                                let cell_padding_bottom = (&info_buf[40..42]).read_i16::<LittleEndian>().unwrap_or(0) as u32 * 4;

                                table.padding.left = cell_padding_left as i16;
                                table.padding.right = cell_padding_right as i16;
                                table.padding.top = cell_padding_top as i16;
                                table.padding.bottom = cell_padding_bottom as i16;

                                let caption_width = (&info_buf[46..48]).read_u16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                                let caption_pos = (&info_buf[70..72]).read_u16::<LittleEndian>().unwrap_or(0);

                                let mut cells = Vec::new();
                                let mut cell_buf = vec![0u8; 27 * (cell_count as usize)];
                                if let Err(_) = body_cursor.read_exact(&mut cell_buf) { break; }
                                
                                let mut xs_raw = Vec::new();
                                let mut ys_raw = Vec::new();
                                
                                for i in 0..cell_count as usize {
                                    let offset = i * 27;
                                    let cell_info = &cell_buf[offset..offset+27];
                                    let x = (&cell_info[4..6]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let y = (&cell_info[6..8]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let w = (&cell_info[8..10]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let h = (&cell_info[10..12]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    xs_raw.push(x);
                                    xs_raw.push(x + w);
                                    ys_raw.push(y);
                                    ys_raw.push(y + h);
                                }
                                
                                xs_raw.sort_unstable();
                                ys_raw.sort_unstable();
                                
                                let mut xs = Vec::new();
                                for &x in &xs_raw {
                                    if let Some(&last) = xs.last() {
                                        if i32::abs(x - last) < 40 {
                                            continue;
                                        }
                                    }
                                    xs.push(x);
                                }
                                
                                let mut ys = Vec::new();
                                for &y in &ys_raw {
                                    if let Some(&last) = ys.last() {
                                        if i32::abs(y - last) < 40 {
                                            continue;
                                        }
                                    }
                                    ys.push(y);
                                }
                                
                                table.col_count = if xs.len() > 1 { (xs.len() - 1) as u16 } else { 1 };
                                table.row_count = if ys.len() > 1 { (ys.len() - 1) as u16 } else { 1 };
                                
                                for i in 0..cell_count as usize {
                                    let offset = i * 27;
                                    let cell_info = &cell_buf[offset..offset+27];
                                    
                                    let mut cell = crate::model::table::Cell::default();
                                    
                                    let x = (&cell_info[4..6]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let y = (&cell_info[6..8]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let w = (&cell_info[8..10]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    let h = (&cell_info[10..12]).read_u16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                                    
                                    let c1 = xs.iter().position(|&val| (val - x).abs() < 40).unwrap_or(cell_info[1] as usize);
                                    let c2 = xs.iter().position(|&val| (val - (x + w)).abs() < 40).unwrap_or(c1 + 1);
                                    let r1 = ys.iter().position(|&val| (val - y).abs() < 40).unwrap_or(cell_info[0] as usize);
                                    let r2 = ys.iter().position(|&val| (val - (y + h)).abs() < 40).unwrap_or(r1 + 1);
                                    
                                    cell.row = r1 as u16;
                                    cell.col = c1 as u16;
                                    cell.col_span = (c2.saturating_sub(c1)).max(1) as u16;
                                    cell.row_span = (r2.saturating_sub(r1)).max(1) as u16;
                                    
                                    cell.width = w as u32;
                                    cell.height = h as u32;
                                    
                                    cell.padding.left = cell_padding_left as i16;
                                    cell.padding.right = cell_padding_right as i16;
                                    cell.padding.top = cell_padding_top as i16;
                                    cell.padding.bottom = cell_padding_bottom as i16;
                                    
                                    let v_align = cell_info[19];
                                    cell.vertical_align = match v_align {
                                        1 => crate::model::table::VerticalAlign::Center,
                                        2 => crate::model::table::VerticalAlign::Bottom,
                                        _ => crate::model::table::VerticalAlign::Top,
                                    };

                                    let mut border_fill = crate::model::style::BorderFill::default();
                                    
                                    let mut hwp3_line_to_border = |line_val: u8| -> crate::model::style::BorderLine {
                                        use crate::model::style::BorderLineType;
                                        // HWP3 선 종류: 0=투명, 1=실선, 2=굵은 실선, 3=점선, 4=2중 실선
                                        let (line_type, width) = match line_val {
                                            1 => (BorderLineType::Solid, 0), // 0.1mm
                                            2 => (BorderLineType::Solid, 6), // 0.4mm (굵은 실선)
                                            3 => (BorderLineType::Dot, 0),   // 0.1mm
                                            4 => (BorderLineType::Double, 6),// 0.4mm (이중선 두께 확보)
                                            _ => (BorderLineType::None, 0),
                                        };
                                        crate::model::style::BorderLine {
                                            line_type,
                                            width,
                                            color: 0,
                                        }
                                    };
                                    
                                    border_fill.borders[0] = hwp3_line_to_border(cell_info[20]); // 왼쪽
                                    border_fill.borders[1] = hwp3_line_to_border(cell_info[21]); // 오른쪽
                                    border_fill.borders[2] = hwp3_line_to_border(cell_info[22]); // 위쪽
                                    border_fill.borders[3] = hwp3_line_to_border(cell_info[23]); // 아래쪽
                                    
                                    let shade = cell_info[24];
                                    if shade > 0 && shade <= 100 {
                                        let mut fill = crate::model::style::Fill::default();
                                        fill.fill_type = crate::model::style::FillType::Solid;
                                        let c = 255 - (shade as u32 * 255 / 100) as u8;
                                        let color = u32::from_le_bytes([c, c, c, 0]);
                                        fill.solid = Some(crate::model::style::SolidFill {
                                            background_color: color,
                                            pattern_color: 0,
                                            pattern_type: 0,
                                        });
                                        border_fill.fill = fill;
                                    }

                                    let diag = cell_info[25] & 0x03;
                                    if diag != 0 {
                                        border_fill.diagonal.diagonal_type = 1; // 실선 (BorderLineType::Solid = 1)
                                        border_fill.diagonal.width = 0; // 0.1mm thickness
                                        match diag {
                                            1 => { // 역슬래시 \
                                                border_fill.attr |= 0b010 << 5;
                                            },
                                            2 => { // 슬래시 /
                                                border_fill.attr |= 0b010 << 2;
                                            },
                                            3 => { // 교차 X
                                                border_fill.attr |= (0b010 << 2) | (0b010 << 5);
                                            },
                                            _ => {}
                                        }
                                    }

                                    doc_border_fills.push(border_fill);
                                    cell.border_fill_id = doc_border_fills.len() as u16; // 1-based (렌더러 규칙)

                                    // 중복된 스팬 계산 제거됨
                                    
                                    let nested = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                                    cell.paragraphs = nested;
                                    cells.push(cell);
                                }
                                table.cells = cells;
                                table.rebuild_grid();
                                table.row_sizes = (0..table.row_count).map(|r| table.cells.iter().filter(|c| c.row == r).count() as i16).collect();
                                let caption_paras = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                                let caption_direction = match caption_pos {
                                    0 => crate::model::shape::CaptionDirection::Bottom,
                                    1 => crate::model::shape::CaptionDirection::Top,
                                    2 => crate::model::shape::CaptionDirection::Left,
                                    3 => crate::model::shape::CaptionDirection::Right,
                                    _ => crate::model::shape::CaptionDirection::Bottom,
                                };
                                table.caption = Some(crate::model::shape::Caption {
                                    direction: caption_direction,
                                    width: caption_width as _,
                                    paragraphs: caption_paras,
                                    ..Default::default()
                                });
                                
                                if obj_type == 2 {
                                    let mut eq = crate::model::control::Equation::default();
                                    eq.baseline = (&info_buf[76..78]).read_i16::<LittleEndian>().unwrap_or(0);
                                    if let Some(cell) = table.cells.first() {
                                        let mut script_text = String::new();
                                        for para in &cell.paragraphs {
                                            script_text.push_str(&para.text);
                                            script_text.push('\n');
                                        }
                                        eq.script = script_text.trim().to_string();
                                    }
                                    parsed_equation = Some(eq);
                                } else {
                                    parsed_table = Some(table);
                                }
                        } else if ch == 11 { // 그림
                            info_buf.resize(348, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            
                            let mut pic = crate::model::image::Picture::default();
                            pic.common.width = ((&info_buf[42..44]).read_u16::<LittleEndian>().unwrap_or(0) as u32) * 4;
                            pic.common.height = ((&info_buf[44..46]).read_u16::<LittleEndian>().unwrap_or(0) as u32) * 4;
                            
                            pic.shape_attr.original_width = pic.common.width;
                            pic.shape_attr.original_height = pic.common.height;
                            pic.shape_attr.current_width = pic.common.width;
                            pic.shape_attr.current_height = pic.common.height;
                            pic.shape_attr.render_sx = 1.0;
                            pic.shape_attr.render_sy = 1.0;
                            
                            let ref_pos = info_buf[8];
                            pic.common.treat_as_char = ref_pos == 0;
                            match ref_pos {
                                1 => {
                                    pic.common.horz_rel_to = crate::model::shape::HorzRelTo::Para;
                                    pic.common.vert_rel_to = crate::model::shape::VertRelTo::Para;
                                },
                                2 => {
                                    pic.common.horz_rel_to = crate::model::shape::HorzRelTo::Page;
                                    pic.common.vert_rel_to = crate::model::shape::VertRelTo::Page;
                                },
                                3 => {
                                    pic.common.horz_rel_to = crate::model::shape::HorzRelTo::Paper;
                                    pic.common.vert_rel_to = crate::model::shape::VertRelTo::Paper;
                                },
                                _ => {}
                            }

                            // 그림 피함(offset 9): 0=자리차지(TopAndBottom), 1=투명(InFrontOfText), 2=어울림(Square)
                            let text_wrap = info_buf[9];
                            pic.common.text_wrap = match text_wrap {
                                0 => crate::model::shape::TextWrap::TopAndBottom, // 자리차지
                                1 => crate::model::shape::TextWrap::InFrontOfText, // 투명 (글자 앞)
                                2 => crate::model::shape::TextWrap::Square, // 어울림
                                _ => crate::model::shape::TextWrap::Square,
                            };
                            
                            pic.common.margin.left = (&info_buf[18..20]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.common.margin.right = (&info_buf[20..22]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.common.margin.top = (&info_buf[22..24]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.common.margin.bottom = (&info_buf[24..26]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            
                            pic.padding.left = (&info_buf[26..28]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.padding.right = (&info_buf[28..30]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.padding.top = (&info_buf[30..32]).read_i16::<LittleEndian>().unwrap_or(0) * 4;
                            pic.padding.bottom = (&info_buf[32..34]).read_i16::<LittleEndian>().unwrap_or(0) * 4;

                            let horz_align = (&info_buf[10..12]).read_i16::<LittleEndian>().unwrap_or(0);
                            if horz_align == -1 {
                                pic.common.horz_align = crate::model::shape::HorzAlign::Left;
                            } else if horz_align == -2 {
                                pic.common.horz_align = crate::model::shape::HorzAlign::Right;
                            } else if horz_align == -3 {
                                pic.common.horz_align = crate::model::shape::HorzAlign::Center;
                            } else {
                                pic.common.horz_align = crate::model::shape::HorzAlign::Left;
                                pic.common.horizontal_offset = (horz_align as i32 * 4) as u32;
                            }

                            let vert_align = (&info_buf[12..14]).read_i16::<LittleEndian>().unwrap_or(0);
                            if vert_align == -1 {
                                pic.common.vert_align = crate::model::shape::VertAlign::Top;
                            } else if vert_align == -2 {
                                pic.common.vert_align = crate::model::shape::VertAlign::Bottom;
                            } else if vert_align == -3 {
                                pic.common.vert_align = crate::model::shape::VertAlign::Center;
                            } else {
                                pic.common.vert_align = crate::model::shape::VertAlign::Top;
                                pic.common.vertical_offset = (vert_align as i32 * 4) as u32;
                            }
                            pic.common.attr = build_common_obj_attr(&pic.common);

                            let n_ext_from_buf = (&info_buf[0..4]).read_u32::<LittleEndian>().unwrap_or(0);
                            let n_ext = n_ext_from_buf;

                            let mut ext_buf = vec![0u8; n_ext as usize];
                            if let Err(_) = body_cursor.read_exact(&mut ext_buf) { break; }
                            
                            let pic_type = info_buf[74];
                            if pic_type == 0 || pic_type == 1 || pic_type == 2 {
                                let pic_name_buf = &info_buf[83..83+256];
                                let mut pic_name = crate::parser::hwp3::encoding::decode_hwp3_string(pic_name_buf);
                                pic_name = pic_name.trim_end_matches('\0').to_string();
                                
                                let _block_num = (&info_buf[62..64]).read_u16::<LittleEndian>().unwrap_or(0);
                                let _pic_info_size = (&info_buf[58..62]).read_u32::<LittleEndian>().unwrap_or(0);
                                
                                if !pic_name.is_empty() {
                                    let next_id = (pic_name_to_id.len() + 1) as u16;
                                    let id = *pic_name_to_id.entry(pic_name).or_insert(next_id);
                                    pic.image_attr.bin_data_id = id;
                                }
                            } else if pic_type == 3 {
                                let mut ext_cursor = std::io::Cursor::new(ext_buf.as_slice());
                                match crate::parser::hwp3::drawing::parse_drawing_object_tree(
                                    &mut ext_cursor,
                                    doc_char_shapes,
                                    doc_para_shapes,
                                    doc_border_fills,
                                    pic_name_to_id,
                                ) {
                                    Ok(drawing_obj) => {
                                        parsed_drawing_object = Some(drawing_obj);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse drawing object tree: {:?}", e);
                                    }
                                }
                            }
                            
                            let caption_pos = (&info_buf[70..72]).read_u16::<LittleEndian>().unwrap_or(0);
                            let caption_width = (&info_buf[46..48]).read_u16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                            let caption_paras = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                            let caption_direction = match caption_pos {
                                0 => crate::model::shape::CaptionDirection::Bottom,
                                1 => crate::model::shape::CaptionDirection::Top,
                                2 => crate::model::shape::CaptionDirection::Left,
                                3 => crate::model::shape::CaptionDirection::Right,
                                _ => crate::model::shape::CaptionDirection::Bottom,
                            };
                            
                            let caption = crate::model::shape::Caption {
                                direction: caption_direction,
                                width: caption_width as _,
                                paragraphs: caption_paras,
                                ..Default::default()
                            };

                            if pic_type == 0 || pic_type == 1 || pic_type == 2 {
                                pic.caption = Some(caption);
                                parsed_picture = Some(pic);
                            } else if pic_type == 3 {
                                // For drawing objects, we might attach the caption if the root is a known shape
                                if let Some(mut drawing_obj) = parsed_drawing_object.take() {
                                    match &mut drawing_obj {
                                        crate::model::shape::ShapeObject::Group(g) => {
                                            g.caption = Some(caption);
                                            pic.common.width = g.common.width;
                                            pic.common.height = g.common.height;
                                            g.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Line(l) => {
                                            l.drawing.caption = Some(caption);
                                            pic.common.width = l.common.width;
                                            pic.common.height = l.common.height;
                                            l.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Rectangle(r) => {
                                            r.drawing.caption = Some(caption);
                                            pic.common.width = r.common.width;
                                            pic.common.height = r.common.height;
                                            r.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Ellipse(e) => {
                                            e.drawing.caption = Some(caption);
                                            pic.common.width = e.common.width;
                                            pic.common.height = e.common.height;
                                            e.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Arc(a) => {
                                            a.drawing.caption = Some(caption);
                                            pic.common.width = a.common.width;
                                            pic.common.height = a.common.height;
                                            a.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Polygon(p) => {
                                            p.drawing.caption = Some(caption);
                                            pic.common.width = p.common.width;
                                            pic.common.height = p.common.height;
                                            p.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Curve(c) => {
                                            c.drawing.caption = Some(caption);
                                            pic.common.width = c.common.width;
                                            pic.common.height = c.common.height;
                                            c.common = pic.common.clone();
                                        },
                                        crate::model::shape::ShapeObject::Picture(p) => {
                                            p.caption = Some(caption);
                                            pic.common.width = p.common.width;
                                            pic.common.height = p.common.height;
                                            p.common = pic.common.clone();
                                        },
                                        _ => {}
                                    }
                                    parsed_drawing_object = Some(drawing_obj);
                                }
                            }
                        } else if ch == 14 { // 선
                            info_buf.resize(84, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            
                            let mut line = crate::model::shape::LineShape::default();
                            let base_pos = info_buf.get(8).copied().unwrap_or(0);
                            line.common.horz_rel_to = match base_pos {
                                1 => crate::model::shape::HorzRelTo::Para,
                                2 => crate::model::shape::HorzRelTo::Page,
                                3 => crate::model::shape::HorzRelTo::Paper,
                                _ => crate::model::shape::HorzRelTo::Para, // 0 is Text (treat_as_char)
                            };
                            line.common.vert_rel_to = match base_pos {
                                1 => crate::model::shape::VertRelTo::Para,
                                2 => crate::model::shape::VertRelTo::Page,
                                3 => crate::model::shape::VertRelTo::Paper,
                                _ => crate::model::shape::VertRelTo::Para, // 0 is Text
                            };
                            line.common.treat_as_char = base_pos == 0;
                            
                            line.common.horizontal_offset = ((&info_buf[10..12]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4) as u32;
                            line.common.vertical_offset = ((&info_buf[12..14]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4) as u32;
                            
                            line.common.width = (&info_buf[42..44]).read_u16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                            line.common.height = (&info_buf[44..46]).read_u16::<LittleEndian>().unwrap_or(0) as u32 * 4;
                            
                            line.start.x = (&info_buf[70..72]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                            line.start.y = (&info_buf[72..74]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                            line.end.x = (&info_buf[74..76]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                            line.end.y = (&info_buf[76..78]).read_i16::<LittleEndian>().unwrap_or(0) as i32 * 4;
                            
                            let thickness = (&info_buf[78..80]).read_u16::<LittleEndian>().unwrap_or(0);
                            let shade = (&info_buf[80..82]).read_u16::<LittleEndian>().unwrap_or(0);
                            let color = (&info_buf[82..84]).read_u16::<LittleEndian>().unwrap_or(0);
                            
                            line.drawing.border_line.width = thickness as i32 * 4;
                            line.drawing.border_line.color = color as u32;
                            
                            if shade > 0 && shade <= 100 {
                                let mut fill = crate::model::style::Fill::default();
                                fill.fill_type = crate::model::style::FillType::Solid;
                                let c = 255 - (shade as u32 * 255 / 100) as u8;
                                let fill_color = u32::from_le_bytes([c, c, c, 0]);
                                fill.solid = Some(crate::model::style::SolidFill {
                                    background_color: fill_color,
                                    pattern_color: 0,
                                    pattern_type: 0,
                                });
                                line.drawing.fill = fill;
                            }
                            
                            parsed_line = Some(line);
                        } else if ch == 15 { // 숨은 설명
                            info_buf.resize(8, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            nested_paragraphs = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                        } else if ch == 16 { // 머리말/꼬리말
                            info_buf.resize(10, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            nested_paragraphs = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                        } else if ch == 17 { // 각주/미주
                            info_buf.resize(14, 0);
                            if let Err(_) = body_cursor.read_exact(&mut info_buf) { break; }
                            nested_paragraphs = parse_paragraph_list(body_cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, pic_name_to_id, body_left_hu, column_width_hu)?;
                        } else if ch == 29 { // 상호 참조
                            if header_val1 < 1000000 {
                                info_buf.resize(header_val1 as usize, 0);
                                let _ = body_cursor.read_exact(&mut info_buf);
                            }
                        } else {
                            // 알 수 없음 (코드 0-4, 12, 27 등 예약 문자)
                            // 8바이트 헤더(ch+field+ch2)만 소비. header_val1은 길이 필드가 아님.
                            // ch=3 실증: hex dump에서 ch2=0x2E('.')로 스펙의 반복코드와 불일치.
                            // 헤더 직후가 정상 단락 내용이므로 추가 skip 없음.
                        }

                        // ch=15(숨은설명), ch=16(머리말/꼬리말), ch=17(각주/미주)는
                        // HWP5 모델에서 인라인 앵커가 없는 비인라인 컨트롤이다.
                        // \u{FFFC}를 text_string에 넣으면 폰트 미지원 글리프("?")로 렌더링되므로 생략.
                        let is_non_inline_ctrl = ch == 15 || ch == 16 || ch == 17;
                        if !is_non_inline_ctrl {
                            char_offsets.push(utf16_len);
                            utf16_len += 1;
                            text_string.push('\u{FFFC}');
                        }

                        if ch == 10 {
                            if parsed_is_hypertext {
                                let mut text = String::new();
                                if let Some(table) = &parsed_table {
                                    if let Some(cell) = table.cells.first() {
                                        for para in &cell.paragraphs {
                                            text.push_str(&para.text);
                                            text.push('\n');
                                        }
                                    }
                                }
                                controls.push(crate::model::control::Control::Hyperlink(crate::model::control::Hyperlink {
                                    url: String::new(), // TODO: TagID 3에서 추출
                                    text: text.trim().to_string(),
                                }));
                            } else if let Some(eq) = parsed_equation {
                                controls.push(crate::model::control::Control::Equation(Box::new(eq)));
                            } else if parsed_obj_type == 1 {
                                let mut rect = crate::model::shape::RectangleShape::default();
                                if let Some(table) = parsed_table {
                                    rect.common = table.common.clone();
                                    let mut tb = crate::model::shape::TextBox::default();
                                    if let Some(cell) = table.cells.first() {
                                        tb.paragraphs = cell.paragraphs.clone();
                                        tb.margin_left = cell.padding.left as _;
                                        tb.margin_right = cell.padding.right as _;
                                        tb.margin_top = cell.padding.top as _;
                                        tb.margin_bottom = cell.padding.bottom as _;
                                        tb.vertical_align = cell.vertical_align;
                                        
                                        if let Some(bf) = doc_border_fills.get(cell.border_fill_id.saturating_sub(1) as usize) {
                                            rect.drawing.border_line = crate::model::style::ShapeBorderLine {
                                                width: bf.borders[0].width as i32,
                                                color: bf.borders[0].color,
                                                ..Default::default()
                                            };
                                            rect.drawing.fill = bf.fill.clone();
                                        }
                                    }
                                    rect.drawing.text_box = Some(tb);
                                    rect.drawing.caption = table.caption.clone();
                                }
                                controls.push(crate::model::control::Control::Shape(Box::new(crate::model::shape::ShapeObject::Rectangle(rect))));
                            } else if parsed_obj_type == 3 {
                                let mut form = crate::model::control::FormObject::default();
                                form.form_type = crate::model::control::FormType::PushButton;
                                form.enabled = true;
                                if let Some(table) = parsed_table {
                                    form.width = table.common.width;
                                    form.height = table.common.height;
                                    if let Some(cell) = table.cells.first() {
                                        let mut text = String::new();
                                        for para in &cell.paragraphs {
                                            text.push_str(&para.text);
                                            text.push('\n');
                                        }
                                        form.caption = text.trim().to_string();
                                        form.name = form.caption.clone();
                                        if let Some(bf) = doc_border_fills.get(cell.border_fill_id.saturating_sub(1) as usize) {
                                            if let Some(ref solid) = bf.fill.solid {
                                                form.back_color = solid.background_color;
                                            }
                                        }
                                    }
                                }
                                controls.push(crate::model::control::Control::Form(Box::new(form)));
                            } else if let Some(table) = parsed_table {
                                controls.push(crate::model::control::Control::Table(Box::new(table)));
                            } else {
                                controls.push(crate::model::control::Control::Unknown(crate::model::control::UnknownControl::default()));
                            }
                        } else if ch == 11 {
                            if let Some(drawing) = parsed_drawing_object {
                                controls.push(crate::model::control::Control::Shape(Box::new(drawing)));
                            } else if let Some(pic) = parsed_picture {
                                controls.push(crate::model::control::Control::Picture(Box::new(pic)));
                            } else {
                                controls.push(crate::model::control::Control::Unknown(crate::model::control::UnknownControl::default()));
                            }
                        } else if ch == 14 {
                            if let Some(line) = parsed_line {
                                controls.push(crate::model::control::Control::Shape(Box::new(crate::model::shape::ShapeObject::Line(line))));
                            } else {
                                controls.push(crate::model::control::Control::Unknown(crate::model::control::UnknownControl::default()));
                            }
                        } else if ch == 16 {
                            let apply_to = match info_buf.get(9).copied().unwrap_or(0) {
                                1 => crate::model::header_footer::HeaderFooterApply::Even,
                                2 => crate::model::header_footer::HeaderFooterApply::Odd,
                                _ => crate::model::header_footer::HeaderFooterApply::Both,
                            };
                            let is_footer = info_buf.get(8).copied().unwrap_or(0) == 1;

                            if is_footer {
                                let mut footer = crate::model::header_footer::Footer::default();
                                footer.paragraphs = nested_paragraphs;
                                footer.apply_to = apply_to;
                                footer.raw_ctrl_extra = info_buf.clone();
                                controls.push(crate::model::control::Control::Footer(Box::new(footer)));
                            } else {
                                let mut header = crate::model::header_footer::Header::default();
                                header.paragraphs = nested_paragraphs;
                                header.apply_to = apply_to;
                                header.raw_ctrl_extra = info_buf.clone();
                                controls.push(crate::model::control::Control::Header(Box::new(header)));
                            }
                        } else if ch == 17 {
                            let is_endnote = (&info_buf[10..12]).read_u16::<LittleEndian>().unwrap_or(0) == 1;
                            
                            if is_endnote {
                                let mut endnote = crate::model::footnote::Endnote::default();
                                endnote.paragraphs = nested_paragraphs;
                                controls.push(crate::model::control::Control::Endnote(Box::new(endnote)));
                            } else {
                                let mut footnote = crate::model::footnote::Footnote::default();
                                footnote.paragraphs = nested_paragraphs;
                                controls.push(crate::model::control::Control::Footnote(Box::new(footnote)));
                            }
                        } else if ch == 29 {
                            let mut field = crate::model::control::Field::default();
                            field.field_type = crate::model::control::FieldType::CrossRef;
                            
                            let kind = info_buf.first().copied().unwrap_or(0);
                            let target_name_bytes = if info_buf.len() >= 38 { &info_buf[1..38] } else { &[] };
                            let target_name = crate::parser::hwp3::encoding::decode_hwp3_string(target_name_bytes)
                                .trim_end_matches('\0')
                                .to_string();
                            
                            let ref_type = if info_buf.len() >= 40 { (&info_buf[38..40]).read_u16::<LittleEndian>().unwrap_or(0) } else { 0 };
                            let n = if info_buf.len() >= 42 { (&info_buf[40..42]).read_u16::<LittleEndian>().unwrap_or(0) } else { 0 };
                            
                            let ref_content_bytes = if info_buf.len() >= 46 + (n as usize) {
                                &info_buf[46..46 + (n as usize)]
                            } else if info_buf.len() > 46 {
                                &info_buf[46..]
                            } else {
                                &[]
                            };
                            let ref_content = crate::parser::hwp3::encoding::decode_hwp3_string(ref_content_bytes)
                                .trim_end_matches('\0')
                                .to_string();
                            
                            // 명령어 문자열로 결합하거나 대상 이름을 사용
                            if kind == 0 {
                                field.command = format!("Target:{}", target_name);
                            } else {
                                field.command = format!("Ref:{},Target:{},Content:{}", ref_type, target_name, ref_content);
                            }
                            field.properties = ref_type as u32;
                            field.extra_properties = kind;
                            
                            controls.push(crate::model::control::Control::Field(field));
                        } else {
                            controls.push(crate::model::control::Control::Unknown(crate::model::control::UnknownControl { ctrl_id: ch as u32 }));
                        }
                        ctrl_data_records.push(None);
                    }
                }
            } else if ch != 0 && ch != 13 {
                let s = crate::parser::hwp3::johab::decode_johab(ch);
                // ch 0x0080..0x7FFF 범위: decode_johab가 매핑 못 하면 '?'를 반환한다.
                // ASCII '?'(=0x003F)와 달리, 이 범위의 미지원 코드는 한글/한자/필드
                // 코드일 가능성이 높으므로 '?' 그대로 출력하지 않고 건너뛴다.
                if s == '?' && ch >= 0x0080 {
                    continue;
                }
                char_offsets.push(utf16_len);
                utf16_len += s.len_utf16() as u32;
                text_string.push(s);
            }
        }

        let mut para = Paragraph::default();
        para.char_count = utf16_len;
        para.para_shape_id = para_shape_id;
        para.char_offsets = char_offsets;
        para.text = text_string;
        para.controls = controls;
        para.ctrl_data_records = ctrl_data_records;
        para.has_para_text = !para.text.is_empty() || !para.controls.is_empty();

        let mut char_shapes = Vec::new();
        char_shapes.push(CharShapeRef {
            start_pos: 0,
            char_shape_id: rep_char_shape_id as u32,
        });

        for (idx, shape_id) in hwp3_inline_shapes {
            if idx < hwp3_char_to_utf16_pos.len() {
                let utf16_pos = hwp3_char_to_utf16_pos[idx];
                char_shapes.push(CharShapeRef {
                    start_pos: utf16_pos,
                    char_shape_id: shape_id as u32,
                });
            }
        }
        
        para.char_shapes = char_shapes;

        let mut base_size = 1000;
        let mut line_spacing_ratio = 160;
        let mut fixed_line_spacing = None;
        
        if let Some(char_shape) = doc_char_shapes.get(rep_char_shape_id as usize) {
            base_size = char_shape.base_size;
        }
        if let Some(para_shape) = doc_para_shapes.get(para_shape_id as usize) {
            if para_shape.line_spacing_type == crate::model::style::LineSpacingType::Percent {
                line_spacing_ratio = para_shape.line_spacing as i32;
            } else {
                fixed_line_spacing = Some(para_shape.line_spacing);
            }
        }
        
        let fallback_text_height = base_size as i32;
        // [Task #604 Stage D-2] HWP5 IR 정합: percent 줄간격도 lh=th, ls=th*(ratio-100)/100
        // 분리 인코딩. 시각 줄 높이 (item h) 는 lh 값 → HWP5 변환본과 동등 (lh=900/ls=540
        // 가 lh=1440/ls=0 보다 60% 작은 시각 높이 → 페이지 회귀 해소).
        let (mut fallback_line_height, fallback_line_spacing) = if let Some(fixed) = fixed_line_spacing {
            // fixed: lh=fixed, ls=fixed-th (추가 간격)
            (fixed, fixed - fallback_text_height)
        } else {
            // percent: lh=th, ls=th*(ratio-100)/100
            (fallback_text_height, fallback_text_height * (line_spacing_ratio - 100) / 100)
        };
        fallback_line_height = fallback_line_height.max(100); // 0 방지
        let fallback_baseline_distance = (fallback_text_height as f32 * 0.85) as i32;

        // Square wrap 그림 어울림 구역 계산 (per-line, pgy 기반)
        // controls가 완성된 이후, line_segs 생성 전에 수행한다.
        let first_pgy_here = line_infos.first().map(|l| l.pgy).unwrap_or(0);
        let last_pgy_here = line_infos.last().map(|l| l.pgy).unwrap_or(first_pgy_here);

        // 이 문단에 Square wrap 그림이 있으면 구역 좌표(pgy_start, pgy_end) 계산.
        // horizontal_offset은 용지(paper) 기준 절대 좌표(HU).
        // column-relative로 변환하여 그림이 왼쪽이면 텍스트가 오른쪽에, 오른쪽이면 왼쪽에 흐르게 함.
        let pic_wrap_zone: Option<(i32, i32, u16, u16)> = para.controls.iter().find_map(|c| {
            if let crate::model::control::Control::Picture(pic) = c {
                if !pic.common.treat_as_char
                    && matches!(pic.common.text_wrap, crate::model::shape::TextWrap::Square)
                    && pic.common.horizontal_offset > 0
                {
                    use crate::model::shape::HorzRelTo;
                    let h_off = pic.common.horizontal_offset as i32;
                    let pic_w = pic.common.width as i32;

                    // 용지 기준 오프셋을 컬럼 기준으로 변환
                    let pic_left_col = match pic.common.horz_rel_to {
                        HorzRelTo::Paper => h_off - body_left_hu,
                        _ => h_off, // Para/Page: 이미 컬럼 기준으로 간주
                    };
                    let pic_right_col = pic_left_col + pic_w;

                    // 그림이 컬럼 영역을 완전히 벗어나면 무시
                    if pic_right_col <= 0 || pic_left_col >= column_width_hu {
                        return None;
                    }

                    // 그림 위치에 따라 텍스트 흐름 방향 결정
                    let (cs, sw) = if pic_left_col < column_width_hu / 2 {
                        // 왼쪽 배치: 텍스트가 오른쪽으로 흐름
                        let cs = pic_right_col.max(0);
                        let sw = (column_width_hu - cs).max(0);
                        (cs, sw)
                    } else {
                        // 오른쪽 배치: 텍스트가 왼쪽으로 흐름
                        let sw = pic_left_col.min(column_width_hu).max(0);
                        (0i32, sw)
                    };

                    if sw <= 0 { return None; }

                    let v_off_hunit = (pic.common.vertical_offset / 4) as u16;
                    let h_hunit = (pic.common.height / 4) as u16;
                    // Para-relative: v_off는 문단 기준 상대 좌표 → first_pgy_here에 더함
                    // Paper/Page-relative: v_off는 용지 기준 절대 좌표 → pgy와 직접 비교
                    let pgy_start = match pic.common.vert_rel_to {
                        crate::model::shape::VertRelTo::Para => first_pgy_here.saturating_add(v_off_hunit),
                        _ => v_off_hunit,
                    };
                    let pgy_end = pgy_start.saturating_add(h_hunit);
                    Some((cs, sw, pgy_start, pgy_end))
                } else {
                    None
                }
            } else {
                None
            }
        });

        // 페이지 경계 여부 (pgy 감소 = 새 페이지)
        // [Task #604 Stage D-2] 명시적 페이지 break (이전 para flags&0x02) 도 포함.
        // first_pgy_here=0 케이스 (새 페이지 시작 정확히 pgy=0) 도 정합 검출.
        let is_page_break = prev_para_had_flags_break
            || (prev_last_pgy > 0 && first_pgy_here < prev_last_pgy);

        // 현재 문단에 적용할 어울림 구역:
        // 자신이 그림 호스트면 pic_wrap_zone, 아니면 이전 문단에서 이어진 active_wrap_zone.
        let current_zone: Option<(i32, i32, u16, u16)> =
            pic_wrap_zone.or(if is_page_break { None } else { active_wrap_zone });

        // active_wrap_zone 갱신
        if let Some(new_zone) = pic_wrap_zone {
            active_wrap_zone = Some(new_zone);
        } else if let Some((_, _, _, pgy_end)) = active_wrap_zone {
            if is_page_break || last_pgy_here >= pgy_end {
                active_wrap_zone = None;
            }
        }

        let mut line_segs = Vec::with_capacity(line_infos.len().max(1));
        if line_infos.is_empty() {
            // line_infos 없음: first_pgy_here로 구역 판정
            let cs_sw = current_zone.and_then(|(cs, sw, pgy_start, pgy_end)| {
                if first_pgy_here >= pgy_start && first_pgy_here < pgy_end {
                    Some((cs, sw))
                } else {
                    None
                }
            });
            line_segs.push(LineSeg {
                text_start: 0,
                line_height: fallback_line_height,
                text_height: fallback_text_height,
                baseline_distance: fallback_baseline_distance,
                line_spacing: fallback_line_spacing,
                column_start: cs_sw.map(|(cs, _)| cs).unwrap_or(0),
                segment_width: cs_sw.map(|(_, sw)| sw).unwrap_or(0),
                tag: 0x00060000,
                ..Default::default()
            });
        } else {
            for linfo in &line_infos {
                let char_idx = linfo.start_pos as usize;
                let text_start = if char_idx < hwp3_char_to_utf16_pos.len() {
                    hwp3_char_to_utf16_pos[char_idx]
                } else {
                    utf16_len
                };

                let mut th = (linfo.line_height as i32) * 4;

                let mut lh;
                let mut bl;
                let mut ls;

                if th == 0 {
                    lh = fallback_line_height;
                    th = fallback_text_height;
                    bl = fallback_baseline_distance;
                    ls = fallback_line_spacing;
                } else {
                    // [Task #604 Stage D-2] HWP5 IR 정합: lh=th, ls 분리 인코딩
                    bl = (th as f32 * 0.85) as i32;
                    if let Some(fixed) = fixed_line_spacing {
                        lh = fixed;
                        ls = fixed - th;
                    } else {
                        lh = th;
                        ls = th * (line_spacing_ratio - 100) / 100;
                    }
                }

                // [Task #604 Stage D-2] HWP3 break_flag 의 페이지/단 경계 hint 는 IR
                // tag 에 누설하지 않음. HWP5 IR 정합: tag bit 0/1 은 paragraph/column 의
                // "first line of" semantic 만 표현. HWP3 의 break_flag 는 stale layout
                // hint (원래 HWP3 가 본 줄에서 페이지/단 break 했음) → 본 환경 typeset
                // 의 자체 pagination 과 충돌 → 본 hint 누설 시 강제 페이지 break 발생.
                // Stage A+D vpos 누적 정합화로 자연스러운 pagination 정합.
                let tag = 0x00060000u32;

                // 이 줄의 pgy로 어울림 구역 판정 (per-line)
                //
                // 앵커 문단(pic_wrap_zone.is_some()): 자신이 그림 호스트이므로 pgy 무관하게 적용.
                //
                // [Task #604 Stage 3] 후속 문단: pgy_end 만 검사 (pgy_start 가드 제거).
                // 본 정정 이전: `pgy >= pgy_start && pgy < pgy_end` 양방향 검사. 그러나
                // wrap text 문단의 첫 줄 pgy 가 anchor 의 pgy_start 미만 인 경우 발생
                // (예: hwp3-sample5.hwp pi=75 첫 3 줄). 결과 cs/sw=0 → 그림 좌측 (x=56.7)
                // 에 텍스트 그려짐 → 그림과 겹침 (Issue #604).
                //
                // 본질: 후속 wrap text 문단은 anchor 그림 우측에 정합 배치되어야 하며,
                // pgy_start 미만의 줄도 wrap zone 의 일부. pgy_end 만 가드해 그림 아래로
                // 흘러간 줄 (cs=0 인 정상 줄) 만 wrap zone 외 판정.
                let line_cs_sw = current_zone.and_then(|(cs, sw, _pgy_start, pgy_end)| {
                    if pic_wrap_zone.is_some() || linfo.pgy < pgy_end {
                        Some((cs, sw))
                    } else {
                        None
                    }
                });

                // [Task #604 Stage A+D] HWP3 본질 유지: lh / ls 그대로 (Stage 5 B-2 revert).
                // HWP5 v2024 변환본 분석 결과 lh+ls 누적값이 본 환경 HWP3 의 lh 와 동등
                // (HWP5: lh=900+ls=540=1440 / HWP3: lh=1440+ls=0=1440). vpos 누적 정합화는
                // paragraphs.push 후 후처리에서 처리.
                line_segs.push(LineSeg {
                    text_start,
                    vertical_pos: 0,
                    line_height: lh,
                    text_height: th,
                    baseline_distance: bl,
                    line_spacing: ls,
                    column_start: line_cs_sw.map(|(cs, _)| cs).unwrap_or(0),
                    segment_width: line_cs_sw.map(|(_, sw)| sw).unwrap_or(0),
                    tag,
                });
            }
        }
        let char_count = para.text.chars().count();
        // line_infos가 있으면 한글97 저장 레이아웃을 신뢰하여 reflow 생략.
        // line_infos가 없을 때만 폴백으로 글자 수 기반 reflow를 수행한다.
        if line_infos.is_empty() && line_segs.len() == 1 && !para.text.contains('\n') && char_count > 40 {
            let base_seg = line_segs.remove(0);
            let mut reflowed_segs = Vec::new();
            let mut last_break_utf16 = 0;
            let mut current_utf16 = 0;
            
            let chunk_max = 38;
            let mut current_chunk_len = 0;
            let mut last_space_idx = None;
            let mut last_space_utf16 = None;
            
            for (i, ch) in para.text.chars().enumerate() {
                if ch == ' ' {
                    last_space_idx = Some(i);
                    last_space_utf16 = Some(current_utf16);
                }
                
                current_utf16 += ch.len_utf16() as u32;
                current_chunk_len += 1;
                
                if current_chunk_len > chunk_max {
                    let (break_idx, break_utf16) = if let Some(sp_idx) = last_space_idx {
                        (sp_idx + 1, last_space_utf16.unwrap() + 1)
                    } else {
                        (i, current_utf16 - ch.len_utf16() as u32)
                    };
                    
                    let mut seg = base_seg.clone();
                    seg.text_start = last_break_utf16;
                    reflowed_segs.push(seg);
                    
                    last_break_utf16 = break_utf16;
                    current_chunk_len = (i + 1).saturating_sub(break_idx);
                    last_space_idx = None;
                    last_space_utf16 = None;
                }
            }
            
            if last_break_utf16 < current_utf16 || reflowed_segs.is_empty() {
                let mut seg = base_seg.clone();
                seg.text_start = last_break_utf16;
                reflowed_segs.push(seg);
            }
            
            para.line_segs = reflowed_segs;
        } else {
            para.line_segs = line_segs;
        }

        // TAC 표 문단: 줄간격 배율 미적용 — lh=th (표 높이 그대로, line spacing은 내용 텍스트에만 적용)
        {
            let has_tac_table = para.controls.iter().any(|c| {
                if let crate::model::control::Control::Table(t) = c {
                    t.common.treat_as_char
                } else {
                    false
                }
            });
            if has_tac_table {
                for seg in para.line_segs.iter_mut() {
                    seg.line_height = seg.text_height;
                    seg.line_spacing = 0;
                }
            }
        }

        // HWP3 후처리: tac=false(부동) + 자리차지(TopAndBottom) 그림의
        // caption.width=0 보정 (layout_body_picture 캡션 렌더링에 그림 너비 사용).
        // paginator는 Control::Picture 처리 시 pic_h를 current_height에 추가하므로
        // line_height 보정은 이중 계산을 유발한다 — caption.width만 보정한다.
        for ctrl in para.controls.iter_mut() {
            if let crate::model::control::Control::Picture(pic) = ctrl {
                if !pic.common.treat_as_char
                    && pic.common.text_wrap == crate::model::shape::TextWrap::TopAndBottom
                {
                    if let Some(ref mut caption) = pic.caption {
                        if caption.width == 0 {
                            caption.width = pic.common.width;
                        }
                    }
                }
            }
        }

        // Fix 1: HWP3 그림 자리차지 LINE_SEG 제거
        // HWP3은 비-TAC TopAndBottom 그림 높이를 LINE_SEG(th=0, lh≈그림높이)로 인코딩한다.
        // HWP5/HWPX에는 이 패턴이 없고, 그림 높이는 typeset.rs pushdown_h로만 반영된다.
        // HWP3에서 이 자리차지 LINE_SEG를 유지하면 높이가 이중 계산되므로 제거한다.
        {
            let non_tac_pic_heights: Vec<i32> = para.controls.iter()
                .filter_map(|c| {
                    if let crate::model::control::Control::Picture(pic) = c {
                        if !pic.common.treat_as_char
                            && matches!(pic.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                        {
                            return Some(pic.common.height as i32);
                        }
                    }
                    None
                })
                .collect();
            if !non_tac_pic_heights.is_empty() {
                para.line_segs.retain(|seg| {
                    !(seg.text_height == 0
                        && non_tac_pic_heights.iter().any(|&h| (seg.line_height as i32 - h).abs() < 1000))
                });
            }
        }

        // pgy 추적 (wrap zone 의 is_page_break 판정 용 — 본 영역에서는 column_type
        // 갱신 안 함).
        //
        // [Task #604 Stage D-2] pgy 기반 자연 페이지 wrap 은 column_type=Page 로
        // 인코딩하지 않음. 본 환경 typeset.rs 가 item 높이 기준 자체 pagination →
        // 자연 wrap 은 typeset 책임. column_type=Page 는 명시적 [쪽나누기] (flags&0x02)
        // 만 설정 → vpos reset 도 본 영역만 발생.
        let last_pgy = line_infos.last().map(|l| l.pgy).unwrap_or(0);
        if last_pgy > 0 {
            prev_last_pgy = last_pgy;
        }

        // para_info.flags bit 1 = 명시적 페이지나눔: 이전 문단에 이 플래그가 있으면
        // 현재 문단이 새 페이지에서 시작한다.
        if prev_para_had_flags_break {
            para.column_type = crate::model::paragraph::ColumnBreakType::Page;
        }
        prev_para_had_flags_break = para_info.flags & 0x02 != 0;

        // [Task #604 Stage D-2] HWP3 line_info.break_flag 의 페이지 경계 신호를
        // column_type=Page 로 변환. 본 신호는 HWP3 가 자연 wrap 한 페이지 시작.
        // HWP5 v2024 변환본의 vpos=0 (페이지 상단 시작) 인코딩 영역과 정합.
        // 0x8000 = 신호 마커, 0x0001 = 페이지 경계.
        if let Some(first_line) = line_infos.first() {
            if first_line.break_flag & 0x8001 == 0x8001 {
                para.column_type = crate::model::paragraph::ColumnBreakType::Page;
            }
        }

        // [Task #604 Stage A+D] HWP5 IR 표준 정합화: paragraph 간 vpos 연결 + 그림
        // 영역 끝 시 cs/sw=0/full 전환 + paragraph 내 vpos 누적.
        //
        // 본질 (Stage A 진단):
        // - HWP5 v2024 변환본 분석 결과 LineSeg.vpos 는 section 단위 누적 절대값
        // - paragraph 내 wrap zone 안 줄 (cs>0) → 그림 영역 끝 시 cs=0/sw=full 전환
        //   (예: pi=75 ls[18] cs=37164 → ls[19] cs=0 at vpos=28800)
        // - paragraph 간 vpos 연결: next.vpos = prev.last_vpos + lh + ls
        //
        // 본 정정으로 본 환경 rhwp 의 typeset/layout vpos 기반 로직 (Task #321/332/412
        // 등) 이 HWP3 파서 출력에 정합 동작 → 시각 결함 자연스럽게 정정.
        {
            // 페이지 break 시 vpos reset (anchor 검출 전 reset 필수 — Stage A+D 정정)
            if matches!(para.column_type, crate::model::paragraph::ColumnBreakType::Page) {
                acc_section_vpos = 0;
                wrap_zone_end_vpos = 0;
            }

            // paragraph 시작 시 그림 anchor 검출 → wrap_zone_end_vpos + active_wrap_cs_sw 갱신
            // (Control::Picture / Control::Shape 안의 ShapeObject::Picture 모두 검사)
            #[derive(Default)]
            struct AnchorInfo { total_h: i32, cs: i32, sw: i32, paper_top: bool }
            let pic_anchor: Option<AnchorInfo> = para.controls.iter().find_map(|c| {
                let pic_common = match c {
                    crate::model::control::Control::Picture(pic) => Some(&pic.common),
                    crate::model::control::Control::Shape(s) => {
                        if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                            Some(&pic.common)
                        } else { None }
                    }
                    _ => None,
                };
                if let Some(cm) = pic_common {
                    if !cm.treat_as_char
                        && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)
                        && cm.horizontal_offset > 0
                    {
                        use crate::model::shape::HorzRelTo;
                        let h_off = cm.horizontal_offset as i32;
                        let pic_w = cm.width as i32;
                        let pic_left_col = match cm.horz_rel_to {
                            HorzRelTo::Paper => h_off - body_left_hu,
                            _ => h_off,
                        };
                        let pic_right_col = pic_left_col + pic_w;
                        if pic_right_col <= 0 || pic_left_col >= column_width_hu {
                            return None;
                        }
                        let (cs, sw) = if pic_left_col < column_width_hu / 2 {
                            let cs = pic_right_col.max(0);
                            let sw = (column_width_hu - cs).max(0);
                            (cs, sw)
                        } else {
                            let sw = pic_left_col.min(column_width_hu).max(0);
                            (0i32, sw)
                        };
                        if sw <= 0 { return None; }
                        let total_h = cm.height as i32
                            + cm.margin.top as i32
                            + cm.margin.bottom as i32;
                        // paper-relative 이고 페이지 상단 근처 (offset ≈ body top)
                        // 인 anchor 만 페이지 break 정합 reset 대상.
                        use crate::model::shape::VertRelTo;
                        let paper_top = matches!(cm.vert_rel_to, VertRelTo::Paper)
                            && (cm.vertical_offset as i32) <= body_left_hu.saturating_add(2400);
                        return Some(AnchorInfo { total_h, cs, sw, paper_top });
                    }
                }
                None
            });
            if let Some(anc) = pic_anchor {
                if anc.paper_top {
                    // [Task #604 Stage D-2] paper-top anchor — acc_vpos reset (HWP5 정합).
                    // HWP5 변환본의 paper-relative anchor (pi=74) 는 vpos=0 인코딩 →
                    // typeset Task #321 vpos-reset guard 가 자연스러운 페이지 break 트리거
                    // → 그림 + wrap text 같은 페이지 정합.
                    acc_section_vpos = 0;
                }
                // wrap zone 영역 끝 = anchor 시작 vpos + 그림 total height
                wrap_zone_end_vpos = acc_section_vpos.saturating_add(anc.total_h);
                active_wrap_cs_sw = Some((anc.cs, anc.sw));
            }

            // LineSeg vpos 누적 + wrap zone cs/sw 정합 인코딩 + 끝 시 전환
            // [Task #604 Stage D-2] paragraph 내 line wrap 시 vpos reset 정합:
            // line_infos[i].pgy < line_infos[i-1].pgy → 본 line 이 새 페이지 시작 (HWP3
            // 가 한글97 layout 시점에 본 line 부터 새 페이지 인식). HWP5 v2024 변환본의
            // paragraph 내 ls[i].vpos=0 영역 정합 (typeset Task #321 vpos-reset guard
            // 영역 trigger 정합).
            for (i, seg) in para.line_segs.iter_mut().enumerate() {
                if i > 0 && i < line_infos.len()
                    && line_infos[i].pgy < line_infos[i-1].pgy
                {
                    // 새 페이지 시작 — vpos reset
                    acc_section_vpos = 0;
                    wrap_zone_end_vpos = 0;
                }
                seg.vertical_pos = acc_section_vpos;

                if wrap_zone_end_vpos > 0 && acc_section_vpos < wrap_zone_end_vpos {
                    // wrap zone 영역 안 — cs/sw 정합 인코딩 (HWP3 pgy-based 누락 보완)
                    if seg.column_start == 0 && seg.segment_width == 0 {
                        if let Some((cs, sw)) = active_wrap_cs_sw {
                            seg.column_start = cs;
                            seg.segment_width = sw;
                        }
                    }
                } else if wrap_zone_end_vpos > 0 && acc_section_vpos >= wrap_zone_end_vpos {
                    // wrap zone 영역 끝 — cs/sw=0/full 전환
                    if seg.column_start > 0 {
                        seg.column_start = 0;
                        seg.segment_width = 0;
                    }
                }

                // 다음 줄 vpos 누적
                acc_section_vpos = acc_section_vpos
                    .saturating_add(seg.line_height)
                    .saturating_add(seg.line_spacing);
            }
        }

        paragraphs.push(para);
    }

    // [Task #604 Stage 2b] wrap_precomputed 후처리 제거 — IR 부채 청산.
    // 본 후처리는 PR #589 보완6/8 에서 도입된 HWP3 휴리스틱 (vertical_pos==0
    // 패턴 검출) 을 IR 에 누설했던 메커니즘. typeset.rs 의 wrap_around state machine
    // 매칭 + ColumnContent.wrap_anchors 메타데이터 채널로 정합 대체됨.
    // (anchor 종류 (Picture vs Table) 기반 분기 → typeset.rs:495~)

    Ok(paragraphs)
}

/// HWP 3.0 포맷 바이너리를 파싱하여 내부 Document 모델로 변환한다.
pub fn parse_hwp3(data: &[u8]) -> Result<Document, Hwp3Error> {
    if data.len() < 30 {
        return Err(Hwp3Error::FileTooSmall);
    }

    if &data[0..23] != b"HWP Document File V3.00" {
        return Err(Hwp3Error::InvalidSignature);
    }

    // 기본 Document 껍데기를 생성한다.
    let mut doc = Document::default();
    // version.major=3: assign_auto_numbers()가 HWP3 문단 카운팅 방식을 사용하도록 표시.
    // 직렬화(serialize_file_header)는 raw_data가 Some이면 개별 필드 대신 raw_data를 사용.
    // → raw_data에 HWP5 헤더를 설정하면 저장 시 올바른 HWP5 CFB 파일이 생성된다.
    doc.header.version.major = 3;
    {
        use crate::parser::header::{FILE_HEADER_SIZE, HWP_SIGNATURE};
        let mut hwp5_hdr = vec![0u8; FILE_HEADER_SIZE];
        hwp5_hdr[..HWP_SIGNATURE.len()].copy_from_slice(HWP_SIGNATURE);
        // 버전 5.0.3.0 (major=5, minor=0, build=3, revision=0) — HWP5 일반 호환 버전
        hwp5_hdr[35] = 5; // major
        hwp5_hdr[34] = 0; // minor
        hwp5_hdr[33] = 3; // build
        hwp5_hdr[32] = 0; // revision
        // flags = 0: 비압축, 비암호, 비배포
        doc.header.raw_data = Some(hwp5_hdr);
    }

    let mut cursor = Cursor::new(&data[30..]); // 파일 인식 정보(30 바이트) 건너뜀

    // 1. 문서 정보 파싱 (128 바이트)
    let doc_info = Hwp3DocInfo::read(&mut cursor)?;
    
    // 2. 문서 요약 파싱 (1008 바이트)
    let doc_summary = Hwp3DocSummary::read(&mut cursor)?;

    // 3. 정보 블록 파싱 (`doc_info.info_block_length` 만큼)
    let mut info_blocks = Vec::new();
    let current_pos = cursor.position();
    let info_block_end = current_pos + doc_info.info_block_length as u64;
    while cursor.position() < info_block_end {
        use crate::parser::hwp3::records::Hwp3InfoBlock;
        if let Ok(block) = Hwp3InfoBlock::read(&mut cursor) {
            info_blocks.push(block);
        } else {
            break;
        }
    }
    cursor.set_position(info_block_end);

    // 4. 본문 텍스트 압축 해제 (`doc_info.compressed` 확인 후 `flate2` 사용)
    let remaining_data = &data[(30 + current_pos as usize + doc_info.info_block_length as usize)..];
    
    let mut decompressed_data = Vec::new();
    let body_data = if doc_info.compressed != 0 {
        use flate2::read::DeflateDecoder;
        let mut decoder = DeflateDecoder::new(remaining_data);
        decoder.read_to_end(&mut decompressed_data).map_err(|e| Hwp3Error::IoError { source: e })?;
        &decompressed_data[..]
    } else {
        remaining_data
    };

    let mut body_cursor = Cursor::new(body_data);

    // 5. 글꼴 이름 파싱 (7가지 언어별 반복)
    let mut font_faces = Vec::new();
    for _lang_idx in 0..7u8 {
        use byteorder::{LittleEndian, ReadBytesExt};
        let nfonts = body_cursor.read_u16::<LittleEndian>().map_err(|e| Hwp3Error::IoError { source: e })?;
        let mut face_list = Vec::new();
        for _ in 0..nfonts {
            let mut font_name_buf = [0u8; 40];
            body_cursor.read_exact(&mut font_name_buf).map_err(|e| Hwp3Error::IoError { source: e })?;
            let font_name = crate::parser::hwp3::encoding::decode_hwp3_string(&font_name_buf);
            use crate::model::style::Font;
            let mut font = Font::default();
            font.name = font_name;
            face_list.push(font);
        }
        font_faces.push(face_list);
    }
    doc.doc_info.font_faces = font_faces;

    let mut doc_char_shapes = Vec::new();
    let mut doc_para_shapes = Vec::new();
    let mut doc_styles = Vec::new();
    let mut doc_border_fills = Vec::new();

    doc_char_shapes.push(crate::model::style::CharShape::default());
    doc_para_shapes.push(crate::model::style::ParaShape::default());
    doc_border_fills.push(crate::model::style::BorderFill::default()); // 인덱스 0은 기본 빈값

    // 6. 스타일 파싱
    use byteorder::{LittleEndian, ReadBytesExt};
    let nstyles = body_cursor.read_u16::<LittleEndian>().map_err(|e| Hwp3Error::IoError { source: e })?;
    for _ in 0..nstyles {
        use crate::parser::hwp3::records::Hwp3Style;
        let style = Hwp3Style::read(&mut body_cursor)?;
        
        doc_char_shapes.push(convert_char_shape(&style.char_shape));
        let c_id = (doc_char_shapes.len() - 1) as u16;
        
        doc_para_shapes.push(convert_para_shape(&style.para_shape));
        let p_id = (doc_para_shapes.len() - 1) as u16;

        use crate::model::style::Style;
        let mut modern_style = Style::default();
        modern_style.local_name = style.name.clone();
        modern_style.english_name = style.name;
        modern_style.char_shape_id = c_id;
        modern_style.para_shape_id = p_id;
        doc_styles.push(modern_style);
    }

    let mut pic_name_to_id = std::collections::HashMap::new();

    // 7. 문단 리스트 파싱 및 Document Model(IR)로 매핑 변환
    // Square wrap 어울림 계산을 위해 페이지 레이아웃 정보 전달 (단위: HWPUNIT)
    let body_left_hu = doc_info.left_margin as i32 * 4;
    let body_right_hu = doc_info.right_margin as i32 * 4;
    let paper_width_hu = doc_info.paper_width as i32 * 4;
    let column_width_hu = (paper_width_hu - body_left_hu - body_right_hu).max(1);
    let mut paragraphs = parse_paragraph_list(&mut body_cursor, &mut doc_char_shapes, &mut doc_para_shapes, &mut doc_border_fills, &mut pic_name_to_id, body_left_hu, column_width_hu)?;


    // 추가 정보 블록 읽기 (압축 해제된 스트림의 끝 부분)
    let mut additional_info_blocks = Vec::new();
    let body_end = body_data.len() as u64;
    while body_cursor.position() < body_end {
        use crate::parser::hwp3::records::Hwp3AdditionalInfoBlock;
        if let Ok(block) = Hwp3AdditionalInfoBlock::read(&mut body_cursor) {
            if block.id == 0 && block.length == 0 {
                break;
            }
            additional_info_blocks.push(block);
        } else {
            break;
        }
    }

    let mut doc_bin_data_list = Vec::new();
    let mut temp_bin_data_content = Vec::new();
    let mut processed_ids = std::collections::HashSet::new();
    let mut hyperlink_urls: Vec<String> = Vec::new();

    for block in additional_info_blocks {
        if block.id == 1 { // 포함된 이미지
            if block.data.len() >= 24 {
                let name_buf = &block.data[0..16];
                let mut name = crate::parser::hwp3::encoding::decode_hwp3_string(name_buf);
                name = name.trim_end_matches('\0').to_string();

                let id = if let Some(&id) = pic_name_to_id.get(&name) {
                    id
                } else {
                    let next_id = (pic_name_to_id.len() + 1) as u16;
                    pic_name_to_id.insert(name.clone(), next_id);
                    next_id
                };

                let img_data = block.data[32..].to_vec();

                let ext = if img_data.starts_with(b"\xFF\xD8\xFF") {
                    "jpg"
                } else if img_data.starts_with(b"\x89PNG\r\n\x1a\n") {
                    "png"
                } else if img_data.starts_with(b"GIF87a") || img_data.starts_with(b"GIF89a") {
                    "gif"
                } else if img_data.starts_with(b"BM") {
                    "bmp"
                } else {
                    "bin"
                }.to_string();

                let content = crate::model::bin_data::BinDataContent {
                    id,
                    extension: ext.clone(),
                    data: img_data,
                };
                let bin_data = crate::model::bin_data::BinData {
                    storage_id: id,
                    extension: Some(ext),
                    data_type: crate::model::bin_data::BinDataType::Embedding,
                    compression: crate::model::bin_data::BinDataCompression::Default,
                    attr: 1, // type=Embedding(bits 0-3=1), compression=Default(bits 4-5=0)
                    ..Default::default()
                };
                temp_bin_data_content.push(content);
                doc_bin_data_list.push(bin_data);
                processed_ids.insert(id);
            }
        } else if block.id == 3 {
            // 추가정보블록 #1 TagID 3 = 하이퍼텍스트(HyperLink) 정보
            // 구조 (스펙 §8.3): 각 항목 617바이트, n개 연속
            //   data[  0..256]: 건너뛸 파일 이름(URL) — kchar[256], null 종료
            //   data[256..288]: 건너뛸 책갈피 — hchar[16]
            //   data[288..613]: 매크로 (도스용) — byte[325]
            //   data[613]     : 종류 (0,1=한글 2=HTML/ETC)
            //   data[614..617]: 예약
            const ENTRY_SIZE: usize = 617;
            let n = block.data.len() / ENTRY_SIZE;
            for i in 0..n {
                let offset = i * ENTRY_SIZE;
                if offset + 256 <= block.data.len() {
                    let url = crate::parser::hwp3::encoding::decode_hwp3_string(
                        &block.data[offset..offset + 256]
                    );
                    hyperlink_urls.push(url);
                }
            }
        }
    }

    // 하이퍼링크 URL을 본문 단락의 Control::Hyperlink에 등장 순서대로 적용
    if !hyperlink_urls.is_empty() {
        let mut url_idx = 0;
        for para in &mut paragraphs {
            for ctrl in &mut para.controls {
                if let crate::model::control::Control::Hyperlink(hl) = ctrl {
                    if url_idx < hyperlink_urls.len() {
                        hl.url = hyperlink_urls[url_idx].clone();
                        url_idx += 1;
                    }
                }
            }
        }
    }

    let max_id = pic_name_to_id.values().max().copied().unwrap_or(0);
    let mut doc_bin_data_content: Vec<crate::model::bin_data::BinDataContent> = (0..max_id)
        .map(|_| crate::model::bin_data::BinDataContent {
            id: 0,
            extension: String::new(),
            data: Vec::new(),
        })
        .collect();

    for content in temp_bin_data_content {
        let id = content.id;
        if id > 0 && id <= max_id {
            doc_bin_data_content[(id - 1) as usize] = content;
        }
    }

    for (name, id) in pic_name_to_id.iter() {
        if !processed_ids.contains(id) {
            let ext = name.rsplit('.').next().unwrap_or("bin").to_string();
            let bin_data = crate::model::bin_data::BinData {
                storage_id: *id,
                extension: Some(ext),
                data_type: crate::model::bin_data::BinDataType::Link,
                abs_path: Some(name.clone()),
                rel_path: Some(name.clone()),
                compression: crate::model::bin_data::BinDataCompression::Default,
                ..Default::default()
            };
            doc_bin_data_list.push(bin_data);
        }
    }

    use crate::model::document::{Section, SectionDef};
    use crate::model::page::PageDef;

    let mut section_def = SectionDef::default();
    section_def.page_def = PageDef {
        width: (doc_info.paper_width as u32) * 4,
        height: (doc_info.paper_length as u32) * 4,
        margin_left: (doc_info.left_margin as u32) * 4,
        margin_right: (doc_info.right_margin as u32) * 4,
        margin_top: (doc_info.top_margin as u32) * 4,
        // HWP3 last-line tolerance: 한글97은 마지막 줄이 본문 영역을 약간 넘어도 해당 페이지에 배치한다.
        // 1600 HWPUNIT(= 한 빈 줄 높이)만큼 하단 여백을 줄여 이 동작을 근사한다.
        margin_bottom: ((doc_info.bottom_margin as u32) * 4).saturating_sub(1600),
        margin_header: (doc_info.header_length as u32) * 4,
        margin_footer: (doc_info.footer_length as u32) * 4,
        margin_gutter: (doc_info.binding_margin as u32) * 4,
        landscape: doc_info.paper_direction != 0,
        ..Default::default()
    };

    let section = Section {
        section_def,
        paragraphs,
        raw_stream: None,
    };
    doc.sections.push(section);

    doc.doc_info.char_shapes = doc_char_shapes;
    doc.doc_info.para_shapes = doc_para_shapes;
    doc.doc_info.styles = doc_styles;
    doc.doc_info.border_fills = doc_border_fills;
    doc.doc_info.bin_data_list = doc_bin_data_list;
    doc.bin_data_content = doc_bin_data_content;

    crate::parser::assign_auto_numbers(&mut doc);
    fixup_hwp3_picture_numbers(&mut doc);

    Ok(doc)
}

fn fixup_hwp3_picture_numbers(doc: &mut crate::model::document::Document) {
    let start = doc.doc_properties.picture_start_num.saturating_sub(1);
    let mut pic_counter: u16 = start;
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            assign_pic_numbers_in_controls(&mut para.controls, &mut pic_counter);
        }
    }
}

fn assign_pic_numbers_in_controls(
    controls: &mut [crate::model::control::Control],
    pic_counter: &mut u16,
) {
    use crate::model::control::{Control, AutoNumberType};
    for ctrl in controls.iter_mut() {
        match ctrl {
            Control::Picture(pic) => {
                *pic_counter += 1;
                let num = *pic_counter;
                if let Some(ref mut caption) = pic.caption {
                    for para in &mut caption.paragraphs {
                        for cap_ctrl in &mut para.controls {
                            if let Control::AutoNumber(an) = cap_ctrl {
                                if an.number_type == AutoNumberType::Picture {
                                    an.assigned_number = num;
                                }
                            }
                        }
                    }
                }
            }
            Control::Table(table) => {
                for cell in &mut table.cells {
                    for para in &mut cell.paragraphs {
                        assign_pic_numbers_in_controls(&mut para.controls, pic_counter);
                    }
                }
                if let Some(ref mut caption) = table.caption {
                    for para in &mut caption.paragraphs {
                        assign_pic_numbers_in_controls(&mut para.controls, pic_counter);
                    }
                }
            }
            Control::Header(h) => {
                for para in &mut h.paragraphs {
                    assign_pic_numbers_in_controls(&mut para.controls, pic_counter);
                }
            }
            Control::Footer(f) => {
                for para in &mut f.paragraphs {
                    assign_pic_numbers_in_controls(&mut para.controls, pic_counter);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_parse_sample_dump() {
        let mut data = Vec::new();
        let mut f = File::open("samples/hwp3-sample.hwp").unwrap();
        f.read_to_end(&mut data).unwrap();

        let _doc = match parse_hwp3(&data) {
            Ok(doc) => doc,
            Err(e) => {
                println!("Parse error: {:?}", e);
                panic!("Parse failed");
            }
        };
    }

    #[test]
    fn test_hwp3_save_as_hwp5_roundtrip() {
        // HWP3 파일 → DocumentCore → HWP5 직렬화 → 재로드 라운드트립 검증.
        // 검증 항목:
        //   1. 저장된 파일이 HWP5 CFB 포맷 (올바른 시그니처)
        //   2. 재로드 시 오류 없이 성공 (PAGE_DEF 등 필수 레코드 보존)
        //   3. 재로드 후 페이지 수 > 0 (내용이 있음)
        // 주의: HWP3 vpos 기반 레이아웃 → HWP5 리플로우는 페이지 수가 달라질 수 있으므로
        //       페이지 수 일치를 요구하지 않는다.
        use crate::document_core::DocumentCore;
        use crate::parser::{detect_format, FileFormat};
        use std::fs::File;
        use std::io::Read;

        let mut data = Vec::new();
        let mut f = match File::open("samples/hwp3-sample.hwp") {
            Ok(f) => f,
            Err(_) => return, // CI 환경 등 샘플 없으면 스킵
        };
        f.read_to_end(&mut data).unwrap();

        let mut core = DocumentCore::from_bytes(&data).expect("HWP3 load failed");

        let hwp5_bytes = core.export_hwp_with_adapter().expect("HWP5 export failed");

        // 저장된 파일이 HWP5 CFB 포맷인지 확인 (version=5 + CFB 시그니처)
        assert_eq!(detect_format(&hwp5_bytes), FileFormat::Hwp, "saved file must be HWP5 CFB");

        // 재로드 성공 + 내용 있음
        let reloaded = DocumentCore::from_bytes(&hwp5_bytes).expect("HWP5 reload failed");
        assert!(reloaded.page_count() > 0, "reloaded document must have pages");

        // BinData 보존 확인: 저장된 HWP5에 BIN*.* 스트림이 존재하는지 확인
        // serialize_bin_data의 attr=0 버그가 있으면 BIN*.* 스트림이 누락되어 이미지가 사라진다.
        {
            use crate::parser::cfb_reader::CfbReader;
            let cfb = CfbReader::open(&hwp5_bytes).expect("CFB open failed");
            let bin_streams: Vec<_> = cfb.list_streams()
                .into_iter()
                .filter(|n| n.contains("BIN"))
                .collect();
            assert!(!bin_streams.is_empty(),
                "saved HWP5 must have BinData/BIN* streams, got none (images lost)");
        }
    }
}
