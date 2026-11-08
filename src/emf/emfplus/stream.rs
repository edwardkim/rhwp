pub(super) struct Record<'a> {
    pub kind: u16,
    pub flags: u16,
    pub data: &'a [u8],
}

/// EMF 레코드 경계 안의 EMF+ 주석만 읽는다. 문자열 검색으로 payload를 찾지 않는다.
pub(super) fn records(bytes: &[u8]) -> Option<Vec<Record<'_>>> {
    if bytes.len() < 88 || u32_at(bytes, 0)? != 1 || bytes.get(40..44)? != b" EMF" {
        return None;
    }
    let mut records = Vec::new();
    let mut offset = 0usize;
    let mut gdi_active = false;
    let mut header = false;
    let mut eof = false;
    while offset.checked_add(8)? <= bytes.len() {
        let kind = u32_at(bytes, offset)?;
        let length = usize::try_from(u32_at(bytes, offset + 4)?).ok()?;
        if length < 8 || length % 4 != 0 {
            return None;
        }
        let end = offset.checked_add(length)?;
        let payload = bytes.get(offset + 8..end)?;
        if kind == 70 && payload.get(4..8) == Some(b"EMF+") {
            let size = usize::try_from(u32_at(payload, 0)?).ok()?;
            let comment = payload.get(4..4usize.checked_add(size)?)?;
            let mut position = 4usize;
            gdi_active = false;
            while position < comment.len() {
                let kind = u16_at(comment, position)?;
                let flags = u16_at(comment, position + 2)?;
                let length = usize::try_from(u32_at(comment, position + 4)?).ok()?;
                let data_length = usize::try_from(u32_at(comment, position + 8)?).ok()?;
                if length < 12 || length % 4 != 0 || data_length > length - 12 {
                    return None;
                }
                let end = position.checked_add(length)?;
                comment.get(position..end)?;
                let data = comment.get(position + 12..position + 12 + data_length)?;
                if records.len() >= 65536 {
                    return None;
                }
                header |= kind == 0x4001;
                eof |= kind == 0x4002;
                gdi_active |= kind == 0x4004;
                records.push(Record { kind, flags, data });
                position = end;
            }
        } else if gdi_active && !matches!(kind, 14 | 28 | 33 | 34 | 36 | 75) {
            // GetDC 아래 실제 GDI 그리기가 있으면 지원 범위 밖이다.
            // EMF+만 표시하여 그 내용을 잃는 대신 호출자의 기존 전체 경로로 돌아간다.
            return None;
        }
        offset = end;
        if kind == 14 {
            break;
        }
    }
    (header && eof).then_some(records)
}

pub(super) fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

pub(super) fn i16_at(bytes: &[u8], offset: usize) -> Option<i16> {
    Some(i16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

pub(super) fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

pub(super) fn f32_at(bytes: &[u8], offset: usize) -> Option<f64> {
    let value = f32::from_bits(u32_at(bytes, offset)?);
    value.is_finite().then_some(f64::from(value))
}

pub(super) fn matrix(bytes: &[u8]) -> Option<[f64; 6]> {
    Some([
        f32_at(bytes, 0)?,
        f32_at(bytes, 4)?,
        f32_at(bytes, 8)?,
        f32_at(bytes, 12)?,
        f32_at(bytes, 16)?,
        f32_at(bytes, 20)?,
    ])
}

pub(super) fn rect(bytes: &[u8], offset: usize) -> Option<String> {
    let (x, y, w, h) = (
        f32_at(bytes, offset)?,
        f32_at(bytes, offset + 4)?,
        f32_at(bytes, offset + 8)?,
        f32_at(bytes, offset + 12)?,
    );
    (w >= 0.0 && h >= 0.0)
        .then(|| format!("<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"/>"))
}
