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
// 코멘트 — EMF+ 이중 스트림 판별용 (단계 밖, #5637)
const RT_COMMENT: u32 = 0x00000046;
// MS-EMF 정의 레코드 타입 최댓값 (EMR_CREATECOLORSPACEW = 0x7A).
const RT_MAX: u32 = 0x0000007A;
// [#5637] 재동기 시도 상한 — 적대적 입력에서의 반복 폭주 방지.
const MAX_RESYNCS: usize = 8;

/// EMF 바이트를 레코드 시퀀스로 파싱.
///
/// [#5637] EMF+ 이중 스트림(GDI+ 레코드를 `EMR_COMMENT`에 내장) 중 일부 생산자는
/// 코멘트 레코드의 Size 필드가 실데이터보다 작게 적혀 있어, 코멘트 뒤에서 레코드
/// 프레이밍이 임의 바이트 위에 얹힌다. 그런 스트림도 뒤쪽에는 온전한 GDI 폴백
/// 레코드(EMR_STRETCHDIBITS 등)가 이어지므로, **EMF+ 코멘트를 본 스트림에 한해**
/// 구조 파단 지점부터 다음 그럴듯한 레코드 연쇄를 훑어 재동기한다.
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

    let mut emfplus_comment_seen = false;
    let mut resyncs = 0usize;

    // 나머지 레코드 디스패처.
    while !cursor.is_empty() {
        let record_start = cursor.position();
        let step = parse_one_record(&mut cursor, &mut emfplus_comment_seen);
        match step {
            Ok(record) => {
                let eof = matches!(record, Record::Eof);
                out.push(record);
                if eof {
                    break;
                }
            }
            Err(err) => {
                // EMF+ 이중 스트림이면 재동기 시도.
                if emfplus_comment_seen && resyncs < MAX_RESYNCS {
                    if let Some(next) = find_resync(bytes, record_start + 4) {
                        resyncs += 1;
                        cursor = Cursor::new(bytes);
                        let _ = cursor.take(next)?;
                        continue;
                    }
                }
                // 재동기가 불가능해도 그릴 내용이 있는 EMF+ 프리픽스는 살린다 —
                // 손상 지점 전까지의 온전한 레코드 렌더가 빈 placeholder 보다 낫다.
                // EMF+ 시그니처 없는 일반 EMF는 종전처럼 형식 오류를 반환한다.
                if emfplus_comment_seen && out.iter().any(is_paintable) {
                    break;
                }
                return Err(err);
            }
        }
    }

    Ok(out)
}

/// 손상 스트림 프리픽스 구제 판단용 — Player 가 실제로 그리는 레코드인지.
fn is_paintable(record: &Record) -> bool {
    matches!(
        record,
        Record::StretchDIBits(_)
            | Record::ExtTextOutW(_)
            | Record::Rectangle(_)
            | Record::Ellipse(_)
            | Record::RoundRect { .. }
            | Record::Arc { .. }
            | Record::Chord { .. }
            | Record::Pie { .. }
            | Record::LineTo(_)
            | Record::Polyline16 { .. }
            | Record::Polygon16 { .. }
            | Record::PolyBezier16 { .. }
            | Record::FillPath(_)
            | Record::StrokePath(_)
            | Record::StrokeAndFillPath(_)
    )
}

/// 커서 위치에서 레코드 1개를 읽어 디스패치한다. EMF+ 시그니처 코멘트를 만나면
/// `emfplus_comment_seen`을 세운다.
fn parse_one_record(
    cursor: &mut Cursor<'_>,
    emfplus_comment_seen: &mut bool,
) -> Result<Record, Error> {
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
    if rh.record_type == RT_COMMENT && payload.len() >= 8 && &payload[4..8] == b"EMF+" {
        *emfplus_comment_seen = true;
    }
    let mut sub = Cursor::new(payload);

    dispatch(rh.record_type, &mut sub, payload_len)
}

/// [#5637] `from`부터 4바이트 스텝으로 "그럴듯한 레코드 연쇄"의 시작을 찾는다.
///
/// 후보 조건: 알려진 타입 범위(1..=0x7A) + Size ≥ 8·4의 배수·버퍼 안. 임의 바이트
/// 위양성을 줄이기 위해 후보 뒤로도 같은 조건의 레코드가 이어지는지(또는 EOF/버퍼
/// 끝에 닿는지) 2단계 연쇄를 확인한다.
fn find_resync(bytes: &[u8], from: usize) -> Option<usize> {
    let mut p = from.checked_add(3)? & !3usize;
    while p + 8 <= bytes.len() {
        if let Some(h) = plausible_header_at(bytes, p) {
            // 큰 비트맵 레코드는 뒤가 다시 파단이어도 내부 필드 자기-일관성으로
            // 단독 수용한다 — 이중 스트림 파일은 폴백 비트맵 뒤가 또 깨진 경우가 있다.
            if chain_is_plausible(bytes, p, &h, 2) || stretch_dibits_self_consistent(bytes, p, &h) {
                return Some(p);
            }
        }
        p += 4;
    }
    None
}

/// EMR_STRETCHDIBITS 후보의 내부 오프셋/크기 필드가 레코드 안에서 맞물리는지 검사.
/// 42KB급 레코드에서 이 조합이 임의 바이트로 성립할 확률은 무시할 수 있다.
fn stretch_dibits_self_consistent(bytes: &[u8], pos: usize, header: &RecordHeader) -> bool {
    if header.record_type != RT_STRETCH_DI_BITS {
        return false;
    }
    let size = header.size as usize;
    // 고정부(레코드 시작 기준 80바이트) + BITMAPINFOHEADER 최소 40바이트
    if size < 80 + 40 {
        return false;
    }
    let f = |off: usize| -> usize {
        let b = &bytes[pos + off..pos + off + 4];
        u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
    };
    let (off_bmi, cb_bmi, off_bits, cb_bits) = (f(48), f(52), f(56), f(60));
    off_bmi >= 80
        && cb_bmi >= 40
        && cb_bits > 0
        && off_bmi.saturating_add(cb_bmi) <= size
        && off_bits.saturating_add(cb_bits) <= size
        // BITMAPINFOHEADER.biSize == 40
        && f(off_bmi) == 40
}

/// `pos`의 8바이트가 그럴듯한 레코드 헤더인지 검사.
fn plausible_header_at(bytes: &[u8], pos: usize) -> Option<RecordHeader> {
    let b = bytes.get(pos..pos + 8)?;
    let record_type = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    let size = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
    if !(1..=RT_MAX).contains(&record_type) {
        return None;
    }
    if size < 8 || size % 4 != 0 {
        return None;
    }
    if pos + size as usize > bytes.len() {
        return None;
    }
    Some(RecordHeader { record_type, size })
}

/// 후보 레코드 뒤로 `depth`개의 레코드가 이어서 그럴듯한지 확인한다.
/// EOF 레코드나 버퍼 끝 도달은 유효한 종결로 본다.
fn chain_is_plausible(bytes: &[u8], pos: usize, header: &RecordHeader, depth: usize) -> bool {
    if header.record_type == RT_EOF || depth == 0 {
        return true;
    }
    let next = pos + header.size as usize;
    if next == bytes.len() {
        return true;
    }
    match plausible_header_at(bytes, next) {
        Some(h) => chain_is_plausible(bytes, next, &h, depth - 1),
        None => false,
    }
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
