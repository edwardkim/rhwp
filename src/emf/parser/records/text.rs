//! 텍스트 레코드 — EMR_EXTTEXTOUTW (MS-EMF 2.3.5.7).
//!
//! 레이아웃(레코드 시작 기준, type+size 8B 포함 offset):
//!   08..24  Bounds: RectL
//!   24..28  iGraphicsMode: u32
//!   28..32  exScale: f32
//!   32..36  eyScale: f32
//!   36..76  EmrText (고정부 40B):
//!     36..44  Reference: PointL
//!     44..48  Chars: u32 (nChars)
//!     48..52  offString: u32 (record 시작 기준)
//!     52..56  Options: u32
//!     56..72  Rectangle: RectL
//!     72..76  offDx: u32
//!   offString.. : UTF-16LE OutputString[nChars]
//!
//! 본 모듈은 페이로드(= record 시작 + 8)를 받는다. 레코드 내 offset은 `off - 8`로 환산.

use crate::emf::parser::objects::{PointL, RectL};
use crate::emf::Error;

#[derive(Debug, Clone)]
pub struct ExtTextOut {
    pub bounds: RectL,
    pub graphics_mode: u32,
    pub ex_scale: f32,
    pub ey_scale: f32,
    pub reference: PointL,
    pub options: u32,
    pub rectangle: RectL,
    pub text: String, // UTF-16 → UTF-8
}

const HEADER_BYTES: usize = 8;

pub fn parse(payload: &[u8]) -> Result<ExtTextOut, Error> {
    if payload.len() < 68 {
        return Err(Error::UnexpectedEof { at: 0, need: 68 });
    }

    // payload 시작 = record offset 8. 각 필드를 payload offset 기준으로 읽음.
    let bounds = read_rectl(&payload[0..16])?;
    let graphics_mode = read_u32(&payload[16..20]);
    let ex_scale = f32::from_le_bytes(payload[20..24].try_into().unwrap());
    let ey_scale = f32::from_le_bytes(payload[24..28].try_into().unwrap());

    // EmrText (payload offset 28..68)
    let reference = PointL {
        x: read_i32(&payload[28..32]),
        y: read_i32(&payload[32..36]),
    };
    let n_chars = read_u32(&payload[36..40]) as usize;
    let off_string = read_u32(&payload[40..44]) as usize;
    let options = read_u32(&payload[44..48]);
    let rectangle = read_rectl(&payload[48..64])?;
    let _off_dx = read_u32(&payload[64..68]);

    // OutputString: offset은 record 기준이므로 payload 기준 off_string - 8.
    let text = if n_chars == 0 {
        String::new()
    } else {
        let start = off_string
            .checked_sub(HEADER_BYTES)
            .ok_or(Error::UnexpectedEof {
                at: 0,
                need: HEADER_BYTES,
            })?;
        let byte_len = n_chars
            .checked_mul(2)
            .ok_or(Error::UnexpectedEof { at: start, need: 0 })?;
        if start + byte_len > payload.len() {
            return Err(Error::UnexpectedEof {
                at: start,
                need: byte_len,
            });
        }
        let slice = &payload[start..start + byte_len];
        let mut utf16 = Vec::with_capacity(n_chars);
        for i in 0..n_chars {
            utf16.push(u16::from_le_bytes([slice[i * 2], slice[i * 2 + 1]]));
        }
        String::from_utf16_lossy(&utf16)
    };

    Ok(ExtTextOut {
        bounds,
        graphics_mode,
        ex_scale,
        ey_scale,
        reference,
        options,
        rectangle,
        text,
    })
}

fn read_u32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
fn read_i32(b: &[u8]) -> i32 {
    read_u32(b) as i32
}
fn read_rectl(b: &[u8]) -> Result<RectL, Error> {
    Ok(RectL {
        left: read_i32(&b[0..4]),
        top: read_i32(&b[4..8]),
        right: read_i32(&b[8..12]),
        bottom: read_i32(&b[12..16]),
    })
}
