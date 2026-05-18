//! EMR_HEADER 구조체 — MS-EMF 2.3.4.2.
//!
//! 최소 고정부 88바이트 + 선택 확장(1: +12B, 2: +8B). `Size` 필드 값에 따라
//! 확장을 읽는다. Description/PixelFormat/OpenGL/Micrometers는 구조만 보존하고
//! rhwp 렌더에서 직접 사용하지 않는다.

use super::rectl::{RectL, SizeL};
use crate::emf::parser::Cursor;
use crate::emf::Error;

/// " EMF" 시그니처(offset 40).
pub const SIGNATURE: u32 = 0x464D4520;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub bounds: RectL,  // 논리 좌표 bbox
    pub frame: RectL,   // 0.01mm bbox
    pub signature: u32, // " EMF"
    pub version: u32,
    pub bytes: u32, // 파일 전체 크기
    pub records: u32,
    pub handles: u16,
    pub reserved: u16,
    pub n_description: u32,
    pub off_description: u32,
    pub n_pal_entries: u32,
    pub device: SizeL, // 참조 장치 픽셀
    pub millimeters: SizeL,

    // 확장 1 (선택)
    pub ext1: Option<HeaderExt1>,
    // 확장 2 (선택)
    pub ext2: Option<HeaderExt2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderExt1 {
    pub cb_pixel_format: u32,
    pub off_pixel_format: u32,
    pub b_open_gl: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderExt2 {
    pub micrometers_x: u32,
    pub micrometers_y: u32,
}

impl Header {
    /// EMR_HEADER를 파싱한다. `cursor`는 type+size를 포함한 레코드 선두를 가리킨다.
    ///
    /// 끝에서 레코드 크기만큼 커서를 전진시킨다(남은 바이트는 Description 등으로 스킵).
    pub fn read(cursor: &mut Cursor<'_>) -> Result<Self, Error> {
        let record_start = cursor.position();

        let record_type = cursor.u32()?;
        debug_assert_eq!(record_type, 1);
        let size = cursor.u32()?;

        // 고정 88바이트 중 type+size 제외 80바이트 영역에서 순차 읽기.
        let bounds = RectL::read(cursor)?;
        let frame = RectL::read(cursor)?;
        let signature = cursor.u32()?;
        if signature != SIGNATURE {
            return Err(Error::InvalidSignature { got: signature });
        }
        let version = cursor.u32()?;
        let bytes = cursor.u32()?;
        let records = cursor.u32()?;
        let handles = cursor.u16()?;
        let reserved = cursor.u16()?;
        let n_description = cursor.u32()?;
        let off_description = cursor.u32()?;
        let n_pal_entries = cursor.u32()?;
        let device = SizeL::read(cursor)?;
        let millimeters = SizeL::read(cursor)?;

        let mut ext1 = None;
        let mut ext2 = None;

        // 확장 1: Size >= 100 (88 + 12).
        if size >= 100 {
            ext1 = Some(HeaderExt1 {
                cb_pixel_format: cursor.u32()?,
                off_pixel_format: cursor.u32()?,
                b_open_gl: cursor.u32()?,
            });
        }
        // 확장 2: Size >= 108 (100 + 8).
        if size >= 108 {
            ext2 = Some(HeaderExt2 {
                micrometers_x: cursor.u32()?,
                micrometers_y: cursor.u32()?,
            });
        }

        // 남은 바이트(Description 등)는 스킵하여 레코드 끝으로 이동.
        let consumed = cursor.position() - record_start;
        let remaining = (size as usize).saturating_sub(consumed);
        if remaining > 0 {
            let _ = cursor.take(remaining)?;
        }

        Ok(Self {
            bounds,
            frame,
            signature,
            version,
            bytes,
            records,
            handles,
            reserved,
            n_description,
            off_description,
            n_pal_entries,
            device,
            millimeters,
            ext1,
            ext2,
        })
    }
}
