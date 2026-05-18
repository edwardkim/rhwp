//! LogFontW — MS-EMF 2.2.13 (92바이트).

use crate::emf::parser::Cursor;
use crate::emf::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFontW {
    pub height: i32,
    pub width: i32,
    pub escapement: i32, // 0.1도 단위, 회전
    pub orientation: i32,
    pub weight: i32, // 400=normal, 700=bold
    pub italic: u8,
    pub underline: u8,
    pub strike_out: u8,
    pub char_set: u8,
    pub out_precision: u8,
    pub clip_precision: u8,
    pub quality: u8,
    pub pitch_and_family: u8,
    pub face_name: String, // UTF-16 → UTF-8 (null-terminated, 최대 32자)
}

impl LogFontW {
    /// 92바이트 고정부 파싱. FaceName은 `[u16; 32]` → null 이전까지 UTF-16 디코딩.
    pub fn read(c: &mut Cursor<'_>) -> Result<Self, Error> {
        let height = c.i32()?;
        let width = c.i32()?;
        let escapement = c.i32()?;
        let orientation = c.i32()?;
        let weight = c.i32()?;
        let b = c.take(8)?;
        let (italic, underline, strike_out, char_set) = (b[0], b[1], b[2], b[3]);
        let (out_precision, clip_precision, quality, pitch_and_family) = (b[4], b[5], b[6], b[7]);

        // FaceName[32]: 64바이트 UTF-16LE, null-terminated.
        let face_bytes = c.take(64)?;
        let mut utf16: Vec<u16> = Vec::with_capacity(32);
        for i in 0..32 {
            let w = u16::from_le_bytes([face_bytes[i * 2], face_bytes[i * 2 + 1]]);
            if w == 0 {
                break;
            }
            utf16.push(w);
        }
        let face_name = String::from_utf16_lossy(&utf16);

        Ok(Self {
            height,
            width,
            escapement,
            orientation,
            weight,
            italic,
            underline,
            strike_out,
            char_set,
            out_precision,
            clip_precision,
            quality,
            pitch_and_family,
            face_name,
        })
    }
}
