use super::error::HmlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HmlEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
}

pub struct DecodedHml {
    pub text: String,
    pub encoding: HmlEncoding,
}

pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<DecodedHml, HmlError> {
    if bytes.len() > max_bytes {
        return Err(HmlError::LimitExceeded(format!(
            "XML size {} > {max_bytes} bytes",
            bytes.len()
        )));
    }
    decode_complete(bytes)
}

pub(crate) fn decode_prefix(bytes: &[u8], max_bytes: usize) -> Option<String> {
    let prefix = &bytes[..bytes.len().min(max_bytes)];
    match bom_encoding(prefix) {
        Some((HmlEncoding::Utf8, offset)) => decode_utf8_prefix(&prefix[offset..]),
        Some((encoding, offset)) => decode_units_prefix(&prefix[offset..], encoding),
        None => decode_utf8_prefix(prefix),
    }
}

fn decode_complete(bytes: &[u8]) -> Result<DecodedHml, HmlError> {
    match bom_encoding(bytes) {
        Some((HmlEncoding::Utf8, offset)) => Ok(DecodedHml {
            text: std::str::from_utf8(&bytes[offset..])
                .map_err(|_| HmlError::UnsupportedEncoding)?
                .to_owned(),
            encoding: HmlEncoding::Utf8,
        }),
        Some((encoding, offset)) => Ok(DecodedHml {
            text: decode_units_complete(&bytes[offset..], encoding)?,
            encoding,
        }),
        None => Ok(DecodedHml {
            text: std::str::from_utf8(bytes)
                .map_err(|_| HmlError::UnsupportedEncoding)?
                .to_owned(),
            encoding: HmlEncoding::Utf8,
        }),
    }
}

fn bom_encoding(bytes: &[u8]) -> Option<(HmlEncoding, usize)> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        Some((HmlEncoding::Utf8, 3))
    } else if bytes.starts_with(&[0xff, 0xfe]) {
        Some((HmlEncoding::Utf16Le, 2))
    } else if bytes.starts_with(&[0xfe, 0xff]) {
        Some((HmlEncoding::Utf16Be, 2))
    } else {
        None
    }
}

fn decode_units_complete(bytes: &[u8], encoding: HmlEncoding) -> Result<String, HmlError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(HmlError::UnsupportedEncoding);
    }
    let units = utf16_units(bytes, encoding);
    String::from_utf16(&units).map_err(|_| HmlError::UnsupportedEncoding)
}

fn decode_units_prefix(bytes: &[u8], encoding: HmlEncoding) -> Option<String> {
    let even_len = bytes.len() - (bytes.len() % 2);
    let units = utf16_units(&bytes[..even_len], encoding);
    String::from_utf16(&units).ok()
}

fn utf16_units(bytes: &[u8], encoding: HmlEncoding) -> Vec<u16> {
    bytes
        .chunks_exact(2)
        .map(|pair| match encoding {
            HmlEncoding::Utf16Le => u16::from_le_bytes([pair[0], pair[1]]),
            HmlEncoding::Utf16Be => u16::from_be_bytes([pair[0], pair[1]]),
            HmlEncoding::Utf8 => unreachable!("UTF-8 has no u16 units"),
        })
        .collect()
}

fn decode_utf8_prefix(bytes: &[u8]) -> Option<String> {
    match std::str::from_utf8(bytes) {
        Ok(text) => Some(text.to_owned()),
        Err(error) if error.error_len().is_none() => {
            std::str::from_utf8(&bytes[..error.valid_up_to()])
                .ok()
                .map(str::to_owned)
        }
        Err(_) => None,
    }
}
