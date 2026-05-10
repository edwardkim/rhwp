//! HWP3 문서 구조체 및 레코드 정의
//! 
//! HWP3 파일 포맷의 다양한 헤더, 문서 정보, 스타일, 개체 레코드의 바이트 수준 구조를 정의한다.
//! 바이너리 스트림에서 직접 구조체로 데이터를 읽어오는 메서드들을 포함한다.

use byteorder::{LittleEndian, ReadBytesExt};
use snafu::ResultExt;
use std::io::{self, Read};
use super::Hwp3Error;

#[derive(Debug, Default)]
pub struct Hwp3DocInfo {
    pub cursor_para: u16,
    pub cursor_pos: u16,
    pub paper_kind: u8,
    pub paper_direction: u8,
    pub paper_length: u16,
    pub paper_width: u16,
    pub top_margin: u16,
    pub bottom_margin: u16,
    pub left_margin: u16,
    pub right_margin: u16,
    pub header_length: u16,
    pub footer_length: u16,
    pub binding_margin: u16,
    pub doc_protected: u32,
    pub reserved1: u16, // 비트 플래그
    pub link_page_number: u8,
    pub link_footnote_number: u8,
    pub link_print_file: String, // 40 바이트 kchar
    pub description: String,     // 24 바이트 kchar
    pub encrypted: u16,
    pub start_page_number: u16,
    pub footnote_start_number: u16,
    pub footnote_reserved: u16,
    pub footnote_line_margin: u16,
    pub footnote_text_margin: u16,
    pub footnote_between_margin: u16,
    pub footnote_bracket: u8,
    pub footnote_line_width: u8,
    pub border_margin_left: u16,
    pub border_margin_right: u16,
    pub border_margin_top: u16,
    pub border_margin_bottom: u16,
    pub border_type: u16,
    pub hide_empty_line: u8,
    pub move_frame: u8,
    pub compressed: u8,
    pub sub_revision: u8,
    pub info_block_length: u16,
}

impl Hwp3DocInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let cursor_para = reader.read_u16::<LittleEndian>()?;
        let cursor_pos = reader.read_u16::<LittleEndian>()?;
        let paper_kind = reader.read_u8()?;
        let paper_direction = reader.read_u8()?;
        let paper_length = reader.read_u16::<LittleEndian>()?;
        let paper_width = reader.read_u16::<LittleEndian>()?;
        let top_margin = reader.read_u16::<LittleEndian>()?;
        let bottom_margin = reader.read_u16::<LittleEndian>()?;
        let left_margin = reader.read_u16::<LittleEndian>()?;
        let right_margin = reader.read_u16::<LittleEndian>()?;
        let header_length = reader.read_u16::<LittleEndian>()?;
        let footer_length = reader.read_u16::<LittleEndian>()?;
        let binding_margin = reader.read_u16::<LittleEndian>()?;
        let doc_protected = reader.read_u32::<LittleEndian>()?;
        let reserved1 = reader.read_u16::<LittleEndian>()?;
        let link_page_number = reader.read_u8()?;
        let link_footnote_number = reader.read_u8()?;

        let mut link_print_file_buf = [0u8; 40];
        reader.read_exact(&mut link_print_file_buf)?;
        let link_print_file = crate::parser::hwp3::encoding::decode_hwp3_string(&link_print_file_buf);

        let mut description_buf = [0u8; 24];
        reader.read_exact(&mut description_buf)?;
        let description = crate::parser::hwp3::encoding::decode_hwp3_string(&description_buf);

        let encrypted = reader.read_u16::<LittleEndian>()?;
        let start_page_number = reader.read_u16::<LittleEndian>()?;
        let footnote_start_number = reader.read_u16::<LittleEndian>()?;
        let footnote_reserved = reader.read_u16::<LittleEndian>()?;
        let footnote_line_margin = reader.read_u16::<LittleEndian>()?;
        let footnote_text_margin = reader.read_u16::<LittleEndian>()?;
        let footnote_between_margin = reader.read_u16::<LittleEndian>()?;
        let footnote_bracket = reader.read_u8()?;
        let footnote_line_width = reader.read_u8()?;
        
        let border_margin_left = reader.read_u16::<LittleEndian>()?;
        let border_margin_right = reader.read_u16::<LittleEndian>()?;
        let border_margin_top = reader.read_u16::<LittleEndian>()?;
        let border_margin_bottom = reader.read_u16::<LittleEndian>()?;
        let border_type = reader.read_u16::<LittleEndian>()?;
        
        let hide_empty_line = reader.read_u8()?;
        let move_frame = reader.read_u8()?;
        let compressed = reader.read_u8()?;
        let sub_revision = reader.read_u8()?;
        let info_block_length = reader.read_u16::<LittleEndian>()?;

        Ok(Hwp3DocInfo {
            cursor_para,
            cursor_pos,
            paper_kind,
            paper_direction,
            paper_length,
            paper_width,
            top_margin,
            bottom_margin,
            left_margin,
            right_margin,
            header_length,
            footer_length,
            binding_margin,
            doc_protected,
            reserved1,
            link_page_number,
            link_footnote_number,
            link_print_file,
            description,
            encrypted,
            start_page_number,
            footnote_start_number,
            footnote_reserved,
            footnote_line_margin,
            footnote_text_margin,
            footnote_between_margin,
            footnote_bracket,
            footnote_line_width,
            border_margin_left,
            border_margin_right,
            border_margin_top,
            border_margin_bottom,
            border_type,
            hide_empty_line,
            move_frame,
            compressed,
            sub_revision,
            info_block_length,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DocSummary {
    pub title: String,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub keywords: [String; 2],
    pub etc: [String; 3],
}

impl Hwp3DocSummary {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let read_hchar_string = |reader: &mut R| -> Result<String, io::Error> {
            let mut buf = [0u8; 112]; // 56 hchar (각 2바이트)
            reader.read_exact(&mut buf)?;
            
            // hchar는 EUC-KR 이나 조합형, 그리고 HWP 내부 코드 체계를 가짐.
            // 일단 바이너리 원본을 저장하고 나중에 문자열로 변환하는 것이 좋지만,
            // Hwp3 파서의 초기 버전에서는 간단하게 읽어들입니다.
            Ok(crate::parser::hwp3::encoding::decode_hwp3_string(&buf))
        };

        let title = read_hchar_string(&mut reader)?;
        let subject = read_hchar_string(&mut reader)?;
        let author = read_hchar_string(&mut reader)?;
        let date = read_hchar_string(&mut reader)?;
        
        let kw1 = read_hchar_string(&mut reader)?;
        let kw2 = read_hchar_string(&mut reader)?;
        
        let etc1 = read_hchar_string(&mut reader)?;
        let etc2 = read_hchar_string(&mut reader)?;
        let etc3 = read_hchar_string(&mut reader)?;

        Ok(Hwp3DocSummary {
            title,
            subject,
            author,
            date,
            keywords: [kw1, kw2],
            etc: [etc1, etc2, etc3],
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3CharShape {
    pub size: u16,
    pub font_indices: [u8; 7],
    pub ratios: [u8; 7],
    pub spacings: [i8; 7],
    pub shade_color: u8,
    pub text_color: u8,
    pub shade_ratio: u8,
    pub attr: u8,
    pub reserved: [u8; 4],
}

impl Hwp3CharShape {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let size = reader.read_u16::<LittleEndian>()?;
        let mut font_indices = [0u8; 7];
        reader.read_exact(&mut font_indices)?;
        let mut ratios = [0u8; 7];
        reader.read_exact(&mut ratios)?;
        
        let mut spacings_u8 = [0u8; 7];
        reader.read_exact(&mut spacings_u8)?;
        let spacings = spacings_u8.map(|x| x as i8);
        
        let shade_color = reader.read_u8()?;
        let text_color = reader.read_u8()?;
        let shade_ratio = reader.read_u8()?;
        let attr = reader.read_u8()?;
        let mut reserved = [0u8; 4];
        reader.read_exact(&mut reserved)?;

        Ok(Hwp3CharShape {
            size, font_indices, ratios, spacings,
            shade_color, text_color, shade_ratio, attr, reserved,
        })
    }

    pub fn is_italic(&self) -> bool { (self.attr & 0x01) != 0 }
    pub fn is_bold(&self) -> bool { (self.attr & 0x02) != 0 }
    pub fn is_underline(&self) -> bool { (self.attr & 0x04) != 0 }
    pub fn is_outline(&self) -> bool { (self.attr & 0x08) != 0 }
    pub fn is_shadow(&self) -> bool { (self.attr & 0x10) != 0 }
    pub fn is_superscript(&self) -> bool { (self.attr & 0x20) != 0 }
    pub fn is_subscript(&self) -> bool { (self.attr & 0x40) != 0 }
    pub fn is_font_blank(&self) -> bool { (self.attr & 0x80) != 0 }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Hwp3TabDef {
    pub position: u16,
    pub tab_type: u8,
    pub leader: u8,
}

impl Hwp3TabDef {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        // [Task #741 Stage 6] HWP3 tab struct 실제 byte 순서: tab_type(u8) → leader(u8) → position(u16 LE).
        // 기본 탭 default 패턴 검증: bytes [0, 0, 0xE8, 0x03] → position=1000 hunit (slot 0),
        // [0, 0, 0xD0, 0x07] → 2000 hunit (slot 1) 등 1000 hunit 간격 default tab.
        let tab_type = reader.read_u8()?;
        let leader = reader.read_u8()?;
        let position = reader.read_u16::<LittleEndian>()?;
        Ok(Hwp3TabDef { position, tab_type, leader })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Hwp3ColumnDef {
    pub count: u8,
    pub divider: u8,
    pub gap: u16,
    pub reserved: [u8; 4],
}

impl Hwp3ColumnDef {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let count = reader.read_u8()?;
        let divider = reader.read_u8()?;
        let gap = reader.read_u16::<LittleEndian>()?;
        let mut reserved = [0u8; 4];
        reader.read_exact(&mut reserved)?;
        Ok(Hwp3ColumnDef { count, divider, gap, reserved })
    }
}

#[derive(Debug)]
pub struct Hwp3ParaShape {
    pub left_margin: u16,
    pub right_margin: u16,
    pub indent: i16,
    pub line_spacing: u16,
    pub margin_bottom: u16,
    pub word_spacing: u8,
    pub align: u8,
    pub tabs: [Hwp3TabDef; 40],
    pub column_def: Hwp3ColumnDef,
    pub shade_ratio: u8,
    pub border: u8,
    pub border_connection: u8,
    pub margin_top: u16,
    pub reserved: [u8; 2],
}

impl Default for Hwp3ParaShape {
    fn default() -> Self {
        Hwp3ParaShape {
            left_margin: 0,
            right_margin: 0,
            indent: 0,
            line_spacing: 0,
            margin_bottom: 0,
            word_spacing: 0,
            align: 0,
            tabs: [Hwp3TabDef::default(); 40],
            column_def: Hwp3ColumnDef::default(),
            shade_ratio: 0,
            border: 0,
            border_connection: 0,
            margin_top: 0,
            reserved: [0; 2],
        }
    }
}

impl Hwp3ParaShape {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let left_margin = reader.read_u16::<LittleEndian>()?;
        let right_margin = reader.read_u16::<LittleEndian>()?;
        let indent = reader.read_i16::<LittleEndian>()?;
        let line_spacing = reader.read_u16::<LittleEndian>()?;
        let margin_bottom = reader.read_u16::<LittleEndian>()?;
        let word_spacing = reader.read_u8()?;
        let align = reader.read_u8()?;
        let mut tabs = [Hwp3TabDef::default(); 40];
        for i in 0..40 {
            tabs[i] = Hwp3TabDef::read(&mut reader)?;
        }
        let column_def = Hwp3ColumnDef::read(&mut reader)?;
        let shade_ratio = reader.read_u8()?;
        let border = reader.read_u8()?;
        let border_connection = reader.read_u8()?;
        let margin_top = reader.read_u16::<LittleEndian>()?;
        let mut reserved = [0u8; 2];
        reader.read_exact(&mut reserved)?;

        Ok(Hwp3ParaShape {
            left_margin, right_margin, indent, line_spacing, margin_bottom,
            word_spacing, align, tabs, column_def, shade_ratio, border,
            border_connection, margin_top, reserved,
        })
    }

    pub fn alignment(&self) -> u8 {
        self.align
    }

    pub fn has_border(&self) -> bool {
        self.border == 1
    }

    pub fn border_connection(&self) -> bool {
        self.border_connection == 1
    }
}

#[derive(Debug, Default)]
pub struct Hwp3Style {
    pub name: String,
    pub char_shape: Hwp3CharShape,
    pub para_shape: Hwp3ParaShape,
}

impl Hwp3Style {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let mut name_buf = [0u8; 20];
        reader.read_exact(&mut name_buf)?;
        let name = crate::parser::hwp3::encoding::decode_hwp3_string(&name_buf);

        let char_shape = Hwp3CharShape::read(&mut reader)?;
        let para_shape = Hwp3ParaShape::read(&mut reader)?;

        Ok(Hwp3Style {
            name,
            char_shape,
            para_shape,
        })
    }
}

#[derive(Debug)]
pub struct Hwp3InfoBlock {
    pub id: u16,
    pub length: u16,
    pub data: Vec<u8>,
}

impl Hwp3InfoBlock {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let id = reader.read_u16::<LittleEndian>()?;
        let length = reader.read_u16::<LittleEndian>()?;
        let mut data = vec![0u8; length as usize];
        reader.read_exact(&mut data)?;
        Ok(Hwp3InfoBlock { id, length, data })
    }
}

#[derive(Debug)]
pub struct Hwp3AdditionalInfoBlock {
    pub id: u32,
    pub length: u32,
    pub data: Vec<u8>,
}

impl Hwp3AdditionalInfoBlock {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let id = reader.read_u32::<LittleEndian>()?;
        if id == 0 {
            // 끝을 의미
            return Ok(Hwp3AdditionalInfoBlock { id, length: 0, data: Vec::new() });
        }
        let length = reader.read_u32::<LittleEndian>()?;
        let mut data = vec![0u8; length as usize];
        reader.read_exact(&mut data)?;
        Ok(Hwp3AdditionalInfoBlock { id, length, data })
    }
}
