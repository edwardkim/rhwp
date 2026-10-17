//! Sampling declared by a bounded PDF importer in a PNG resource.
//! `rhWP` is a private ancillary, unsafe-to-copy chunk immediately after IHDR.
//! Raw BinData preservation keeps this contract through HWP/HWPX save/reopen;
//! ordinary images without the declaration retain the renderer's profile policy.
pub(crate) fn interpolation(data: &[u8]) -> Option<bool> {
    if data.len() < 47
        || &data[..8] != b"\x89PNG\r\n\x1a\n"
        || data[8..16] != [0, 0, 0, 13, b'I', b'H', b'D', b'R']
        || data[33..41] != [0, 0, 0, 2, b'r', b'h', b'W', b'P']
        || data[41] != 1
        || data[42] > 1
    {
        return None;
    }
    let expected = u32::from_be_bytes(data[43..47].try_into().ok()?);
    (crc32fast::hash(&data[37..43]) == expected).then_some(data[42] != 0)
}

pub(crate) fn preserve(source: &[u8], output: &mut Vec<u8>) {
    if interpolation(source).is_some()
        && interpolation(output).is_none()
        && output.len() >= 33
        && output[..16] == source[..16]
    {
        output.splice(33..33, source[33..47].iter().copied());
    }
}

/// PDF 32000-1 8.9.5.3 defines interpolation for source samples enlarged onto
/// device pixels. Minification still needs the backend's coverage/prefilter;
/// forcing point sampling there discards source pixels (and thin image lines).
pub(crate) fn magnification(
    declared: Option<bool>,
    source: (f64, f64),
    destination: (f64, f64),
) -> Option<bool> {
    if [source.0, source.1, destination.0, destination.1]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0.)
        || destination.0 < source.0
        || destination.1 < source.1
    {
        return None;
    }
    declared
}

pub(crate) fn dimensions(data: &[u8]) -> Option<(f64, f64)> {
    interpolation(data)?;
    Some((
        u32::from_be_bytes(data[16..20].try_into().ok()?) as f64,
        u32::from_be_bytes(data[20..24].try_into().ok()?) as f64,
    ))
}
