//! EMF 파서 루트 — 스트림 reader + 레코드 디스패처.

pub mod constants;
pub mod objects;
pub mod records;

pub use objects::header::Header;

use super::Error;
use records::Record;

// RecordType 값 상수 — 디스패처 분기용. RecordType enum과 일치.
const RT_HEADER: u32 = 0x00000001;
const RT_EOF: u32 = 0x0000000E;
const RT_SET_WINDOW_EXT_EX: u32 = 0x00000009;
const RT_SET_WINDOW_ORG_EX: u32 = 0x0000000A;
const RT_SET_VIEWPORT_EXT_EX: u32 = 0x0000000B;
const RT_SET_VIEWPORT_ORG_EX: u32 = 0x0000000C;
const RT_SET_MAP_MODE: u32 = 0x00000011;
const RT_SET_BK_MODE: u32 = 0x00000012;
const RT_SET_TEXT_ALIGN: u32 = 0x00000016;
const RT_SET_TEXT_COLOR: u32 = 0x00000018;
const RT_SET_BK_COLOR: u32 = 0x00000019;
const RT_SAVE_DC: u32 = 0x00000021;
const RT_RESTORE_DC: u32 = 0x00000022;
const RT_SET_WORLD_TRANSFORM: u32 = 0x00000023;
const RT_MODIFY_WORLD_TRANSFORM: u32 = 0x00000024;
const RT_SELECT_OBJECT: u32 = 0x00000025;
const RT_CREATE_PEN: u32 = 0x00000026;
const RT_CREATE_BRUSH_INDIRECT: u32 = 0x00000027;
const RT_DELETE_OBJECT: u32 = 0x00000028;
const RT_EXT_CREATE_FONT_INDIRECT_W: u32 = 0x00000052;
// 드로잉 (단계 12)
const RT_MOVE_TO_EX: u32 = 0x0000001B;
const RT_ELLIPSE: u32 = 0x0000002A;
const RT_RECTANGLE: u32 = 0x0000002B;
const RT_ROUND_RECT: u32 = 0x0000002C;
const RT_ARC: u32 = 0x0000002D;
const RT_CHORD: u32 = 0x0000002E;
const RT_PIE: u32 = 0x0000002F;
const RT_LINE_TO: u32 = 0x00000036;
const RT_POLYBEZIER16: u32 = 0x00000055;
const RT_POLYLINE16: u32 = 0x00000056;
const RT_POLYGON16: u32 = 0x00000057;
// 패스 (단계 12)
const RT_BEGIN_PATH: u32 = 0x0000003B;
const RT_END_PATH: u32 = 0x0000003C;
const RT_CLOSE_FIGURE: u32 = 0x0000003D;
const RT_FILL_PATH: u32 = 0x0000003E;
const RT_STROKE_AND_FILL_PATH: u32 = 0x0000003F;
const RT_STROKE_PATH: u32 = 0x00000040;
// 텍스트/비트맵 (단계 13)
const RT_EXT_TEXT_OUT_W: u32 = 0x00000054;
const RT_STRETCH_DI_BITS: u32 = 0x00000051;

/// EMF 바이트를 레코드 시퀀스로 파싱.
pub fn parse(bytes: &[u8]) -> Result<Vec<Record>, Error> {
    let mut cursor = Cursor::new(bytes);
    let mut out = Vec::new();

    // 첫 레코드: EMR_HEADER (필수).
    let first = cursor.peek_record_header()?;
    if first.record_type != RT_HEADER {
        return Err(Error::InvalidFirstRecord {
            got: first.record_type,
        });
    }
    let header_record = records::header::parse(&mut cursor)?;
    out.push(Record::Header(header_record));

    // 나머지 레코드 디스패처.
    while !cursor.is_empty() {
        let rh = cursor.peek_record_header()?;
        let record_start = cursor.position();
        let payload_len = (rh.size as usize)
            .checked_sub(8)
            .ok_or(Error::RecordTooSmall {
                offset: record_start,
                size: rh.size,
            })?;

        // type + size 스킵.
        let _ = cursor.take(8)?;

        // 페이로드 전용 sub-cursor. 레코드 경계를 넘지 않도록 분리.
        let payload = cursor.take(payload_len)?;
        let mut sub = Cursor::new(payload);

        let record = dispatch(rh.record_type, &mut sub, payload_len)?;

        let eof = matches!(record, Record::Eof);
        out.push(record);
        if eof {
            break;
        }
    }

    Ok(out)
}

fn dispatch(record_type: u32, c: &mut Cursor<'_>, payload_len: usize) -> Result<Record, Error> {
    use records::{bitmap, drawing, object, path, state, text};

    let rec = match record_type {
        RT_EOF => Record::Eof,

        // 객체
        RT_CREATE_PEN => {
            let (handle, pen) = object::parse_create_pen(c)?;
            Record::CreatePen { handle, pen }
        }
        RT_CREATE_BRUSH_INDIRECT => {
            let (handle, brush) = object::parse_create_brush_indirect(c)?;
            Record::CreateBrushIndirect { handle, brush }
        }
        RT_EXT_CREATE_FONT_INDIRECT_W => {
            let (handle, font) = object::parse_ext_create_font_indirect_w(c, payload_len)?;
            Record::ExtCreateFontIndirectW { handle, font }
        }
        RT_SELECT_OBJECT => Record::SelectObject {
            handle: object::parse_select_object(c)?,
        },
        RT_DELETE_OBJECT => Record::DeleteObject {
            handle: object::parse_delete_object(c)?,
        },

        // 상태 — DC 스택
        RT_SAVE_DC => Record::SaveDC,
        RT_RESTORE_DC => Record::RestoreDC {
            relative: state::parse_restore_dc(c)?,
        },
        RT_SET_WORLD_TRANSFORM => Record::SetWorldTransform(state::parse_set_world_transform(c)?),
        RT_MODIFY_WORLD_TRANSFORM => {
            let (xform, mode) = state::parse_modify_world_transform(c)?;
            Record::ModifyWorldTransform { xform, mode }
        }

        // 좌표계
        RT_SET_MAP_MODE => Record::SetMapMode(state::parse_u32_single(c)?),
        RT_SET_WINDOW_EXT_EX => Record::SetWindowExtEx(state::parse_set_window_ext_ex(c)?),
        RT_SET_WINDOW_ORG_EX => Record::SetWindowOrgEx(state::parse_set_window_org_ex(c)?),
        RT_SET_VIEWPORT_EXT_EX => Record::SetViewportExtEx(state::parse_set_viewport_ext_ex(c)?),
        RT_SET_VIEWPORT_ORG_EX => Record::SetViewportOrgEx(state::parse_set_viewport_org_ex(c)?),

        // 색상/모드
        RT_SET_BK_MODE => Record::SetBkMode(state::parse_u32_single(c)?),
        RT_SET_TEXT_ALIGN => Record::SetTextAlign(state::parse_u32_single(c)?),
        RT_SET_TEXT_COLOR => Record::SetTextColor(state::parse_u32_single(c)?),
        RT_SET_BK_COLOR => Record::SetBkColor(state::parse_u32_single(c)?),

        // 드로잉
        RT_MOVE_TO_EX => Record::MoveToEx(drawing::parse_point(c)?),
        RT_LINE_TO => Record::LineTo(drawing::parse_point(c)?),
        RT_RECTANGLE => Record::Rectangle(drawing::parse_rect(c)?),
        RT_ELLIPSE => Record::Ellipse(drawing::parse_rect(c)?),
        RT_ROUND_RECT => {
            let (rect, corner_w, corner_h) = drawing::parse_round_rect(c)?;
            Record::RoundRect {
                rect,
                corner_w,
                corner_h,
            }
        }
        RT_ARC => {
            let (r, s, e) = drawing::parse_arc_like(c)?;
            Record::Arc {
                rect: r,
                start: s,
                end: e,
            }
        }
        RT_CHORD => {
            let (r, s, e) = drawing::parse_arc_like(c)?;
            Record::Chord {
                rect: r,
                start: s,
                end: e,
            }
        }
        RT_PIE => {
            let (r, s, e) = drawing::parse_arc_like(c)?;
            Record::Pie {
                rect: r,
                start: s,
                end: e,
            }
        }
        RT_POLYLINE16 => {
            let (bounds, points) = drawing::parse_points16(c)?;
            Record::Polyline16 { bounds, points }
        }
        RT_POLYGON16 => {
            let (bounds, points) = drawing::parse_points16(c)?;
            Record::Polygon16 { bounds, points }
        }
        RT_POLYBEZIER16 => {
            let (bounds, points) = drawing::parse_points16(c)?;
            Record::PolyBezier16 { bounds, points }
        }

        // 패스
        RT_BEGIN_PATH => Record::BeginPath,
        RT_END_PATH => Record::EndPath,
        RT_CLOSE_FIGURE => Record::CloseFigure,
        RT_FILL_PATH => Record::FillPath(path::parse_path_bounds(c)?),
        RT_STROKE_PATH => Record::StrokePath(path::parse_path_bounds(c)?),
        RT_STROKE_AND_FILL_PATH => Record::StrokeAndFillPath(path::parse_path_bounds(c)?),

        // 텍스트
        RT_EXT_TEXT_OUT_W => {
            let payload = c.full_buf();
            Record::ExtTextOutW(text::parse(payload)?)
        }

        // 비트맵
        RT_STRETCH_DI_BITS => {
            let payload = c.full_buf();
            Record::StretchDIBits(bitmap::parse(payload)?)
        }

        _ => Record::Unknown {
            record_type,
            payload: c.take(payload_len)?.to_vec(),
        },
    };
    Ok(rec)
}

/// 레코드 공통 헤더(8바이트).
#[derive(Debug, Clone, Copy)]
pub struct RecordHeader {
    pub record_type: u32,
    pub size: u32,
}

/// 리틀엔디언 스트림 리더. EMF 전역에서 재사용.
pub struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    #[inline]
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }
    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pos >= self.buf.len()
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.remaining() < n {
            return Err(Error::UnexpectedEof {
                at: self.pos,
                need: n,
            });
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn peek(&self, n: usize) -> Result<&'a [u8], Error> {
        if self.remaining() < n {
            return Err(Error::UnexpectedEof {
                at: self.pos,
                need: n,
            });
        }
        Ok(&self.buf[self.pos..self.pos + n])
    }

    pub fn u32(&mut self) -> Result<u32, Error> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn i32(&mut self) -> Result<i32, Error> {
        Ok(self.u32()? as i32)
    }
    pub fn u16(&mut self) -> Result<u16, Error> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    /// 커서의 원본 버퍼(페이로드 전체).
    #[must_use]
    pub fn full_buf(&self) -> &'a [u8] {
        self.buf
    }

    pub fn peek_record_header(&self) -> Result<RecordHeader, Error> {
        let b = self.peek(8)?;
        let record_type = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        let size = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
        if size < 8 {
            return Err(Error::RecordTooSmall {
                offset: self.pos,
                size,
            });
        }
        if size % 4 != 0 {
            return Err(Error::MisalignedRecord {
                offset: self.pos,
                size,
            });
        }
        Ok(RecordHeader { record_type, size })
    }
}
