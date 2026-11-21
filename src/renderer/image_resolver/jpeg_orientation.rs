//! 브라우저는 `<img>`·canvas·SVG `<image>`에서 JPEG EXIF 방향(Orientation)을 적용해 그림을
//! 돌려 그린다. 한/글은 그림 바이트를 방향과 상관없이 그대로 그리고, 돌림은 개체 회전으로만 둔다.
//! 브라우저 렌더러에는 방향 값만 1로 바꾼 사본을 넘겨 한/글과 같은 화소 배치로 그리게 한다.

/// 방향이 1(그대로)이 아닌 JPEG면 EXIF 첫 IFD의 방향 값만 1로 바꾼 사본을, 아니면 None을 준다.
pub(crate) fn upright_jpeg(data: &[u8]) -> Option<Vec<u8>> {
    let at = orientation_value_offset(data)?;
    let little = data.get(at.1..at.1 + 2)? == b"II";
    let value = if little {
        u16::from_le_bytes([data[at.0], data[at.0 + 1]])
    } else {
        u16::from_be_bytes([data[at.0], data[at.0 + 1]])
    };
    if value == 1 {
        return None;
    }
    let mut out = data.to_vec();
    let one = if little {
        1u16.to_le_bytes()
    } else {
        1u16.to_be_bytes()
    };
    out[at.0..at.0 + 2].copy_from_slice(&one);
    Some(out)
}

/// (방향 값 자리, TIFF 머리 자리). JPEG가 아니거나 방향 항목이 없으면 None.
fn orientation_value_offset(data: &[u8]) -> Option<(usize, usize)> {
    if !data.starts_with(&[0xFF, 0xD8]) {
        return None;
    }
    let mut at = 2usize;
    while at + 4 <= data.len() && data[at] == 0xFF {
        let marker = data[at + 1];
        let length = u16::from_be_bytes([data[at + 2], data[at + 3]]) as usize;
        // 그림 자료(SOS) 뒤에는 EXIF가 없다.
        if marker == 0xDA || length < 2 {
            return None;
        }
        let end = (at + 2 + length).min(data.len());
        if marker == 0xE1 && data.get(at + 4..at + 10) == Some(b"Exif\0\0") {
            return ifd0_orientation(data, at + 10, end);
        }
        at += 2 + length;
    }
    None
}

fn ifd0_orientation(data: &[u8], tiff: usize, end: usize) -> Option<(usize, usize)> {
    let little = match data.get(tiff..tiff + 2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |at: usize| -> Option<usize> {
        let b = data.get(at..at + 2)?;
        Some(if little {
            u16::from_le_bytes([b[0], b[1]])
        } else {
            u16::from_be_bytes([b[0], b[1]])
        } as usize)
    };
    let u32_at = |at: usize| -> Option<usize> {
        let b: [u8; 4] = data.get(at..at + 4)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        } as usize)
    };
    let ifd = tiff.checked_add(u32_at(tiff + 4)?)?;
    let count = u16_at(ifd)?;
    for i in 0..count {
        let entry = ifd + 2 + i * 12;
        if entry + 12 > end {
            break;
        }
        // 방향은 SHORT(3) 하나이고 값은 항목의 값 칸 앞 2바이트에 있다.
        if u16_at(entry)? == 0x0112 && u16_at(entry + 2)? == 3 {
            return Some((entry + 8, tiff));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::upright_jpeg;

    /// SOI + APP1(Exif, IFD0에 방향 하나) + SOS 머리만 있는 JPEG 조각.
    fn jpeg_with_orientation(value: u16, little: bool) -> Vec<u8> {
        let u16b = |v: u16| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let u32b = |v: u32| {
            if little {
                v.to_le_bytes()
            } else {
                v.to_be_bytes()
            }
        };
        let mut tiff = Vec::new();
        tiff.extend_from_slice(if little { b"II" } else { b"MM" });
        tiff.extend_from_slice(&u16b(42));
        tiff.extend_from_slice(&u32b(8));
        tiff.extend_from_slice(&u16b(1));
        tiff.extend_from_slice(&u16b(0x0112));
        tiff.extend_from_slice(&u16b(3));
        tiff.extend_from_slice(&u32b(1));
        tiff.extend_from_slice(&u16b(value));
        tiff.extend_from_slice(&[0, 0]);
        tiff.extend_from_slice(&u32b(0));
        let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
        out.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
        out.extend_from_slice(b"Exif\0\0");
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x02]);
        out
    }

    #[test]
    fn 방향이_있는_jpeg는_방향_값만_1로_바꾼다() {
        for little in [true, false] {
            let source = jpeg_with_orientation(6, little);
            let upright = upright_jpeg(&source).expect("방향 6은 바꾼다");
            assert_eq!(upright, jpeg_with_orientation(1, little));
            assert_eq!(upright_jpeg(&upright), None);
        }
    }

    #[test]
    fn 방향이_없거나_jpeg가_아니면_그대로_둔다() {
        assert_eq!(upright_jpeg(&jpeg_with_orientation(1, true)), None);
        assert_eq!(upright_jpeg(&[0xFF, 0xD8, 0xFF, 0xDA, 0x00, 0x02]), None);
        assert_eq!(upright_jpeg(b"\x89PNG\r\n\x1a\n"), None);
        // 길이가 잘린 EXIF도 넘어서 읽지 않는다.
        let mut short = jpeg_with_orientation(6, true);
        short.truncate(20);
        assert_eq!(upright_jpeg(&short), None);
    }
}
