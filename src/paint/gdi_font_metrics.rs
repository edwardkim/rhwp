//! GDI의 수평 TrueType 문자 셀에 쓰이는 장치 픽셀 단위 상하단.

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn signed_at(bytes: &[u8], offset: usize) -> Option<i16> {
    u16_at(bytes, offset).map(|value| value as i16)
}

/// OpenType VDMX의 첫 대응 aspect group만 수용한다. 없는 크기를 보간하지 않는다.
fn vdmx_extents(bytes: &[u8], ppem: u16) -> Option<(i32, i32)> {
    if u16_at(bytes, 0)? > 1 {
        return None;
    }
    let ratios = usize::from(u16_at(bytes, 4)?);
    let offsets = 6usize.checked_add(ratios.checked_mul(4)?)?;
    bytes.get(..offsets.checked_add(ratios.checked_mul(2)?)?)?;
    for index in 0..ratios {
        let ratio = 6 + index * 4;
        let x = bytes[ratio + 1];
        let low = bytes[ratio + 2];
        let high = bytes[ratio + 3];
        if !((x == 0 && low == 0 && high == 0) || (low <= x && x <= high)) {
            continue;
        }
        let group = usize::from(u16_at(bytes, offsets + index * 2)?);
        let count = usize::from(u16_at(bytes, group)?);
        for entry in 0..count {
            let at = group.checked_add(4)?.checked_add(entry.checked_mul(6)?)?;
            let size = u16_at(bytes, at)?;
            if size == ppem {
                let top = i32::from(signed_at(bytes, at + 2)?);
                let bottom = i32::from(signed_at(bytes, at + 4)?);
                return (top > bottom).then_some((top, bottom));
            }
            if size > ppem {
                return None;
            }
        }
        return None;
    }
    None
}

pub(super) fn cell_extents(face: &ttf_parser::Face<'_>, ppem: u16) -> Option<(i32, i32)> {
    // 이번 계약은 수평 TrueType의 실제 Print 대조다. CFF·세로 쓰기로 확대하지 않는다.
    face.raw_face()
        .table(ttf_parser::Tag::from_bytes(b"glyf"))?;
    face.raw_face()
        .table(ttf_parser::Tag::from_bytes(b"VDMX"))
        .and_then(|bytes| vdmx_extents(bytes, ppem))
        .or_else(|| {
            let os2 = face
                .raw_face()
                .table(ttf_parser::Tag::from_bytes(b"OS/2"))?;
            let ascent = u16_at(os2, 74)?;
            let descent = u16_at(os2, 76)?;
            let units = f64::from(face.units_per_em());
            (ascent != 0 || descent != 0).then(|| {
                (
                    (f64::from(ascent) * f64::from(ppem) / units).round() as i32,
                    -(f64::from(descent) * f64::from(ppem) / units).round() as i32,
                )
            })
        })
}
