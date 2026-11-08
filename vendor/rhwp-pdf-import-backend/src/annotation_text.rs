//! PDF annotation strings, preserving the shared PDFDocEncoding and BOM rules.
use crate::{error::Result, import_storage::ImportStorage};
fn utf8_lossy(bytes: &[u8], allocation: ImportStorage<'_>) -> Result<String> {
    let count = bytes.utf8_chunks().try_fold(0usize, |n, c| {
        n.checked_add(c.valid().len())
            .and_then(|n| n.checked_add(if c.invalid().is_empty() { 0 } else { 3 }))
            .ok_or_else(|| allocation.reject("annotation UTF-8 size"))
    })?;
    let mut output = allocation.string_capacity(count)?;
    for chunk in bytes.utf8_chunks() {
        output.push_str(chunk.valid());
        if !chunk.invalid().is_empty() {
            output.push('\u{fffd}');
        }
    }
    Ok(output)
}
pub(crate) fn decode(bytes: &[u8], allocation: ImportStorage<'_>) -> Result<String> {
    allocation.bytes(0)?;
    let endian = if bytes.starts_with(&[0xfe, 0xff]) {
        Some(true)
    } else if bytes.starts_with(&[0xff, 0xfe]) {
        Some(false)
    } else {
        None
    };
    if let Some(big) = endian {
        let count = (bytes.len() - 2) / 2;
        let maximum = count
            .checked_mul(3)
            .ok_or_else(|| allocation.reject("annotation UTF-16 size"))?;
        let mut output = allocation.string_capacity(maximum)?;
        let units = bytes[2..].chunks_exact(2).map(|pair| {
            if big {
                u16::from_be_bytes([pair[0], pair[1]])
            } else {
                u16::from_le_bytes([pair[0], pair[1]])
            }
        });
        for scalar in char::decode_utf16(units) {
            match scalar {
                Ok(value) => output.push(value),
                Err(_) => return utf8_lossy(bytes, allocation),
            }
        }
        return Ok(output);
    }
    let count = bytes
        .iter()
        .filter_map(|&b| crate::fonts::font_dict::pdfdoc_encoding_lookup(b))
        .try_fold(0usize, |n, c| {
            n.checked_add(c.len_utf8())
                .ok_or_else(|| allocation.reject("annotation PDFDocEncoding size"))
        })?;
    let mut output = allocation.string_capacity(count)?;
    for scalar in bytes
        .iter()
        .filter_map(|&b| crate::fonts::font_dict::pdfdoc_encoding_lookup(b))
    {
        output.push(scalar);
    }
    Ok(output)
}
pub(crate) fn trim(text: &str, allocation: ImportStorage<'_>) -> Result<String> {
    let text = text.trim();
    let mut output = allocation.string_capacity(text.len())?;
    output.push_str(text);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import_limits::{ImportBudget, PdfImportLimits};
    #[test]
    fn both_utf16_endians_roundtrip_every_unicode_scalar() {
        for n in 0..=0x10ffff {
            let Some(scalar) = char::from_u32(n) else {
                continue;
            };
            let mut units = [0u16; 2];
            let units = scalar.encode_utf16(&mut units);
            let mut utf8 = [0u8; 4];
            let expected: &str = scalar.encode_utf8(&mut utf8);
            for big in [true, false] {
                let mut bytes = [0u8; 6];
                bytes[..2].copy_from_slice(if big { &[0xfe, 0xff] } else { &[0xff, 0xfe] });
                for (i, &unit) in units.iter().enumerate() {
                    let encoded = if big {
                        unit.to_be_bytes()
                    } else {
                        unit.to_le_bytes()
                    };
                    bytes[2 + i * 2..4 + i * 2].copy_from_slice(&encoded);
                }
                assert_eq!(
                    decode(&bytes[..2 + units.len() * 2], ImportStorage(None)).unwrap(),
                    expected
                );
            }
        }
    }
    #[test]
    fn malformed_utf16_keeps_the_original_lossy_fallback_and_odd_byte_contract() {
        for bytes in [
            &[0xfe, 0xff, 0xd8, 0x00][..],
            &[0xff, 0xfe, 0x00, 0xd8][..],
            &[0xfe, 0xff, 0x00, 0x41, 0xff][..],
            &[0xff, 0xfe, 0x41, 0x00, 0xff][..],
        ] {
            let big = bytes[0] == 0xfe;
            let units: Vec<_> = bytes[2..]
                .chunks_exact(2)
                .map(|p| {
                    if big {
                        u16::from_be_bytes([p[0], p[1]])
                    } else {
                        u16::from_le_bytes([p[0], p[1]])
                    }
                })
                .collect();
            let expected = String::from_utf16(&units)
                .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string());
            assert_eq!(decode(bytes, ImportStorage(None)).unwrap(), expected);
        }
    }
    #[test]
    fn string_payloads_reject_before_allocation_and_fresh_requests_recover() {
        for bytes in [
            &b"annotation text"[..],
            &[0xfe, 0xff, 0xac, 0x00][..],
            &[0xff, 0xfe, 0x00, 0xac][..],
            &[0xfe, 0xff, 0xd8, 0x00][..],
        ] {
            let tiny = ImportBudget::new(PdfImportLimits {
                decoded_bytes: 1,
                ..Default::default()
            })
            .unwrap();
            assert!(decode(bytes, ImportStorage(Some(&tiny))).is_err());
            assert_eq!(tiny.snapshot().decoded_bytes, 0);
            let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
            assert_eq!(
                decode(bytes, ImportStorage(Some(&fresh))).unwrap(),
                decode(bytes, ImportStorage(None)).unwrap()
            );
        }
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(trim("  \u{a0}Hello\t", ImportStorage(Some(&fresh))).unwrap(), "Hello");
    }
}
