//! HWP3 문단(Paragraph) 구조 및 파싱
//! 
//! HWP3 문서의 문단 정보를 담고 있는 데이터 구조체(`Hwp3ParaInfo`, `Hwp3LineInfo`)를 정의한다.
//! 문단의 글자 수, 라인 수, 스타일 상속 관계 등을 파싱하여 제공한다.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read};
use super::records::{Hwp3CharShape, Hwp3ParaShape};

#[derive(Debug, Default)]
pub struct Hwp3LineInfo {
    pub start_pos: u16,
    pub space_correction: i16,
    pub line_height: u16,
    pub pgy: u16,   // 한글97이 계산한 줄 Y 좌표 (1/1800인치 단위). pgy 감소 시 새 페이지.
    pub sx: u16,
    pub psx: u16,
    pub break_flag: u16,
}

impl Hwp3LineInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let start_pos = reader.read_u16::<LittleEndian>()?;
        let space_correction = reader.read_i16::<LittleEndian>()?;
        let line_height = reader.read_u16::<LittleEndian>()?;
        let pgy = reader.read_u16::<LittleEndian>()?;
        let sx = reader.read_u16::<LittleEndian>()?;
        let psx = reader.read_u16::<LittleEndian>()?;
        let break_flag = reader.read_u16::<LittleEndian>()?;

        Ok(Hwp3LineInfo {
            start_pos,
            space_correction,
            line_height,
            pgy,
            sx,
            psx,
            break_flag,
        })
    }
}

#[derive(Debug)]
pub struct Hwp3ParaInfo {
    pub follow_prev_para_shape: u8,
    pub char_count: u16,
    pub line_count: u16,
    pub include_char_shape: u8,
    pub flags: u8,
    pub special_char_flags: u32,
    pub style_index: u8,
    pub rep_char_shape: Hwp3CharShape,
    pub para_shape: Option<Hwp3ParaShape>,
}

impl Hwp3ParaInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let follow_prev_para_shape = reader.read_u8()?;
        let char_count = reader.read_u16::<LittleEndian>()?;
        if char_count == 0 {
            // 빈 문단 (리스트의 끝) - 43바이트를 모두 읽어야 함
            // 이미 3바이트(follow_prev, char_count)를 읽었으므로 40바이트 남음
            let mut buf = [0u8; 40];
            reader.read_exact(&mut buf)?;
            return Ok(Hwp3ParaInfo {
                follow_prev_para_shape,
                char_count: 0,
                line_count: 0,
                include_char_shape: 0,
                flags: 0,
                special_char_flags: 0,
                style_index: 0,
                rep_char_shape: Hwp3CharShape::default(),
                para_shape: None,
            });
        }
        let line_count = reader.read_u16::<LittleEndian>()?;
        let include_char_shape = reader.read_u8()?;
        let flags = reader.read_u8()?;
        let special_char_flags = reader.read_u32::<LittleEndian>()?;
        let style_index = reader.read_u8()?;
        
        let rep_char_shape = Hwp3CharShape::read(&mut reader)?;
        
        let para_shape = if follow_prev_para_shape == 0 {
            Some(Hwp3ParaShape::read(&mut reader)?)
        } else {
            None
        };

        Ok(Hwp3ParaInfo {
            follow_prev_para_shape,
            char_count,
            line_count,
            include_char_shape,
            flags,
            special_char_flags,
            style_index,
            rep_char_shape,
            para_shape,
        })
    }
}
