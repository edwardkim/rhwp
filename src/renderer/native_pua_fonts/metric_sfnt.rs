//! TrueType 글꼴의 문자 매핑·전진폭에 필요한 표만 세션 메모리에 보관한다.

/// 윤곽을 복사하지 않는 독립 SFNT를 만든다. 원본 프로그램과 파일은 변경하지 않는다.
/// 가변 글꼴은 변형 관련 표의 전체 계약을 보존하도록 원본 경로를 사용한다.
pub(super) fn compact_truetype_metrics(bytes: &[u8], index: u32) -> Option<Vec<u8>> {
    let face = ttf_parser::Face::parse(bytes, index).ok()?;
    if face.is_variable() || face.tables().cff.is_some() || face.tables().cff2.is_some() {
        return None;
    }
    let raw = face.raw_face();
    let wanted = [
        b"OS/2", b"cmap", b"head", b"hhea", b"hmtx", b"maxp", b"name", b"post",
    ];
    let mut tables = Vec::new();
    for record in raw.table_records {
        if wanted
            .iter()
            .any(|name| record.tag == ttf_parser::Tag::from_bytes(name))
        {
            tables.push((record.tag, raw.table(record.tag)?));
        }
    }
    let count = u16::try_from(tables.len()).ok()?;
    if count == 0 {
        return None;
    }
    let max_power = 1u16 << (15 - count.leading_zeros());
    let search_range = max_power.checked_mul(16)?;
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 1, 0, 0]);
    out.extend_from_slice(&count.to_be_bytes());
    out.extend_from_slice(&search_range.to_be_bytes());
    out.extend_from_slice(&(15u16 - max_power.leading_zeros() as u16).to_be_bytes());
    out.extend_from_slice(&(count.checked_mul(16)? - search_range).to_be_bytes());
    out.resize(12usize.checked_add(usize::from(count).checked_mul(16)?)?, 0);
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        let offset = out.len();
        out.extend_from_slice(data);
        while out.len() % 4 != 0 {
            out.push(0);
        }
        let checksum = out[offset..].chunks_exact(4).fold(0u32, |sum, part| {
            sum.wrapping_add(u32::from_be_bytes(part.try_into().unwrap()))
        });
        let at = 12 + i * 16;
        out[at..at + 4].copy_from_slice(&tag.to_bytes());
        out[at + 4..at + 8].copy_from_slice(&checksum.to_be_bytes());
        out[at + 8..at + 12].copy_from_slice(&u32::try_from(offset).ok()?.to_be_bytes());
        out[at + 12..at + 16].copy_from_slice(&u32::try_from(data.len()).ok()?.to_be_bytes());
    }
    ttf_parser::Face::parse(&out, 0).ok()?;
    Some(out)
}
