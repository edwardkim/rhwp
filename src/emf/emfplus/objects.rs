use std::fmt::Write;

use super::player::{render, Budget};
use super::stream::{f32_at, rect, u32_at};

pub(super) enum Object {
    Path(String),
    Pen(String),
    Region(Option<String>),
    Image(String),
    ImageAttributes,
}

pub(super) fn parse(kind: u16, data: &[u8], depth: usize, budget: &mut Budget) -> Option<Object> {
    match kind {
        3 => path(data, budget).map(Object::Path),
        2 => pen(data).map(Object::Pen),
        4 if u32_at(data, 4)? == 0 => match u32_at(data, 8)? {
            0x10000000 => rect(data, 12).map(|r| Object::Region(Some(r))),
            0x10000003 => Some(Object::Region(None)),
            _ => None,
        },
        5 if u32_at(data, 4)? == 2 && matches!(u32_at(data, 8)?, 4 | 5) => {
            let size = usize::try_from(u32_at(data, 12)?).ok()?;
            let bytes = data.get(16..16usize.checked_add(size)?)?;
            render(bytes, metafile_source_bounds(bytes)?, depth + 1, budget).map(Object::Image)
        }
        5 if u32_at(data, 4)? == 1 => super::bitmap::compressed(data, budget).map(Object::Image),
        // 색상 효과 없는 TileFlipXY만 수용한다. 다른 이미지 효과를 무시하지 않는다.
        8 if data.len() == 24
            && u32_at(data, 8)? == 3
            && u32_at(data, 12)? == 0
            && u32_at(data, 16)? == 0 =>
        {
            Some(Object::ImageAttributes)
        }
        _ => None,
    }
}

/// SrcUnitPixel의 메타파일 경계는 잉크 Bounds가 아니라 물리 Frame이다.
/// Frame과 기록 장치의 크기로 환산하며, inclusive 경계의 한 소스 화소를 포함한다.
/// 실제 Windows GDI+ GetBounds 관측과 원문 SrcRect가 같은 원점·크기를 증언한다.
fn metafile_source_bounds(bytes: &[u8]) -> Option<(f64, f64, f64, f64)> {
    let signed = |offset: usize| -> Option<i32> {
        Some(i32::from_le_bytes(
            bytes.get(offset..offset + 4)?.try_into().ok()?,
        ))
    };
    let (left, top, right, bottom) = (signed(24)?, signed(28)?, signed(32)?, signed(36)?);
    let (device_w, device_h, mm_w, mm_h) = (signed(72)?, signed(76)?, signed(80)?, signed(84)?);
    if device_w <= 0 || device_h <= 0 || mm_w <= 0 || mm_h <= 0 {
        return None;
    }
    let sx = f64::from(device_w) / (100.0 * f64::from(mm_w));
    let sy = f64::from(device_h) / (100.0 * f64::from(mm_h));
    let width = f64::from(right.checked_sub(left)?) * sx + 1.0;
    let height = f64::from(bottom.checked_sub(top)?) * sy + 1.0;
    (right > left && bottom > top).then_some((
        f64::from(left) * sx,
        f64::from(top) * sy,
        width,
        height,
    ))
}

fn path(data: &[u8], budget: &mut Budget) -> Option<String> {
    let count = usize::try_from(u32_at(data, 4)?).ok()?;
    let flags = u32_at(data, 8)?;
    // 상대 좌표·RLE·정수 좌표는 아직 지원하지 않는다. 예약 비트로 경로를 추측하지 않는다.
    if flags & (0x0800 | 0x1000 | 0x4000) != 0 || count > budget.points {
        return None;
    }
    budget.points -= count;
    let types_start = 12usize.checked_add(count.checked_mul(8)?)?;
    let types = data.get(types_start..types_start.checked_add(count)?)?;
    let point = |i: usize| Some((f32_at(data, 12 + i * 8)?, f32_at(data, 16 + i * 8)?));
    let mut result = String::new();
    let mut index = 0;
    while index < count {
        let (x, y) = point(index)?;
        match types[index] & 7 {
            0 => write!(result, " M{x} {y}").ok()?,
            1 => write!(result, " L{x} {y}").ok()?,
            3 if index + 2 < count && types[index + 1] & 7 == 3 && types[index + 2] & 7 == 3 => {
                let (cx, cy) = point(index + 1)?;
                let (ex, ey) = point(index + 2)?;
                write!(result, " C{x} {y} {cx} {cy} {ex} {ey}").ok()?;
                index += 2;
            }
            _ => return None,
        }
        if types[index] & 0x80 != 0 {
            result.push_str(" Z");
        }
        index += 1;
    }
    Some(result)
}

fn pen(data: &[u8]) -> Option<String> {
    if u32_at(data, 4)? != 0 || u32_at(data, 12)? != 0 {
        return None;
    }
    let flags = u32_at(data, 8)?;
    // 변환·복합 선·사용자 cap은 단순한 선으로 근사하지 않는다.
    if flags & !(2 | 4 | 8 | 16 | 32 | 64 | 128 | 256 | 512) != 0 {
        return None;
    }
    let width = f32_at(data, 16)?;
    if width <= 0.0 {
        return None;
    }
    let mut offset = 20;
    let mut caps = [0u32; 7];
    for (index, bit) in [2, 4, 8, 16, 32, 64, 128].into_iter().enumerate() {
        if flags & bit != 0 {
            caps[index] = u32_at(data, offset)?;
            offset += 4;
        }
    }
    if caps[0] != caps[1] || caps[0] > 2 || caps[2] > 2 || caps[4] != 0 || caps[5] > 2 {
        return None;
    }
    let mut dash = String::new();
    if flags & 256 != 0 {
        let count = usize::try_from(u32_at(data, offset)?).ok()?;
        offset += 4;
        if count > data.len().saturating_sub(offset) / 4 {
            return None;
        }
        for _ in 0..count {
            let value = f32_at(data, offset)?;
            if value <= 0.0 {
                return None;
            }
            write!(dash, "{} ", value * width).ok()?;
            offset += 4;
        }
    }
    if flags & 512 != 0 {
        if u32_at(data, offset)? != 0 {
            return None;
        }
        offset += 4;
    }
    if u32_at(data, offset + 4)? != 0 {
        return None;
    }
    let color = u32_at(data, offset + 8)?;
    let opacity = f64::from(color >> 24) / 255.0;
    let linecap = ["butt", "square", "round"][caps[0] as usize];
    let linejoin = ["miter", "bevel", "round"][caps[2] as usize];
    let mut result = format!("fill=\"none\" stroke=\"#{:06x}\" stroke-opacity=\"{opacity}\" stroke-width=\"{width}\" stroke-linecap=\"{linecap}\" stroke-linejoin=\"{linejoin}\"", color & 0xffffff);
    if !dash.is_empty() {
        write!(result, " stroke-dasharray=\"{dash}\"").ok()?;
        let offset = f32::from_bits(caps[6]);
        if !offset.is_finite() {
            return None;
        }
        write!(
            result,
            " stroke-dashoffset=\"{}\"",
            f64::from(offset) * width
        )
        .ok()?;
    }
    if flags & 16 != 0 {
        let miter = f32::from_bits(caps[3]);
        if !miter.is_finite() || miter < 1.0 {
            return None;
        }
        write!(result, " stroke-miterlimit=\"{miter}\"").ok()?;
    }
    Some(result)
}
