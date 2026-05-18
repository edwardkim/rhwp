//! 비트맵 레코드 — EMR_STRETCHDIBITS (MS-EMF 2.3.1.7).
//!
//! 레이아웃 (레코드 시작 기준, type+size 8B 포함):
//!   08..24  Bounds: RectL
//!   24..28  xDest: i32
//!   28..32  yDest: i32
//!   32..36  xSrc: i32
//!   36..40  ySrc: i32
//!   40..44  cxSrc: i32
//!   44..48  cySrc: i32
//!   48..52  offBmiSrc: u32
//!   52..56  cbBmiSrc: u32
//!   56..60  offBitsSrc: u32
//!   60..64  cbBitsSrc: u32
//!   64..68  UsageSrc: u32
//!   68..72  BitBltRasterOperation: u32
//!   72..76  cxDest: i32
//!   76..80  cyDest: i32
//!   offBmiSrc..: BITMAPINFO (BMI header + optional palette)
//!   offBitsSrc..: pixel bits
//!
//! 페이로드(= record 시작 + 8)를 받는다. 내부 offset은 `off - 8`로 환산.

use crate::emf::parser::objects::RectL;
use crate::emf::Error;

#[derive(Debug, Clone)]
pub struct StretchDIBits {
    pub bounds: RectL,
    pub x_dest: i32,
    pub y_dest: i32,
    pub cx_dest: i32,
    pub cy_dest: i32,
    /// BITMAPINFO 섹션 바이트 (BITMAPINFOHEADER + 팔레트).
    pub bmi: Vec<u8>,
    /// 픽셀 비트.
    pub bits: Vec<u8>,
}

const HEADER_BYTES: usize = 8;
const FIXED: usize = 72;

pub fn parse(payload: &[u8]) -> Result<StretchDIBits, Error> {
    if payload.len() < FIXED {
        return Err(Error::UnexpectedEof { at: 0, need: FIXED });
    }

    let bounds = RectL {
        left: read_i32(&payload[0..4]),
        top: read_i32(&payload[4..8]),
        right: read_i32(&payload[8..12]),
        bottom: read_i32(&payload[12..16]),
    };
    let x_dest = read_i32(&payload[16..20]);
    let y_dest = read_i32(&payload[20..24]);
    // xSrc, ySrc, cxSrc, cySrc 사용 안 함(단계 13 범위 외).
    let off_bmi = read_u32(&payload[40..44]) as usize;
    let cb_bmi = read_u32(&payload[44..48]) as usize;
    let off_bits = read_u32(&payload[48..52]) as usize;
    let cb_bits = read_u32(&payload[52..56]) as usize;
    let cx_dest = read_i32(&payload[64..68]);
    let cy_dest = read_i32(&payload[68..72]);

    let bmi = slice_at(payload, off_bmi, cb_bmi)?;
    let bits = slice_at(payload, off_bits, cb_bits)?;

    Ok(StretchDIBits {
        bounds,
        x_dest,
        y_dest,
        cx_dest,
        cy_dest,
        bmi: bmi.to_vec(),
        bits: bits.to_vec(),
    })
}

fn slice_at(payload: &[u8], record_off: usize, len: usize) -> Result<&[u8], Error> {
    let start = record_off
        .checked_sub(HEADER_BYTES)
        .ok_or(Error::UnexpectedEof {
            at: 0,
            need: HEADER_BYTES,
        })?;
    if start + len > payload.len() {
        return Err(Error::UnexpectedEof {
            at: start,
            need: len,
        });
    }
    Ok(&payload[start..start + len])
}

fn read_u32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
fn read_i32(b: &[u8]) -> i32 {
    read_u32(b) as i32
}
