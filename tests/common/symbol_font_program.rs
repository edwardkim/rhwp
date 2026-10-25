//! 저장소의 MIT 합성 font의 cmap만 메모리에서 바꾸는 계약 검사 자료.

pub fn mapped_program(source: &[u8], mappings: &[(char, u16)]) -> Vec<u8> {
    let mut bytes = source.to_vec();
    let table_count = u16::from_be_bytes(source[4..6].try_into().unwrap()) as usize;
    let record = |tag: &[u8; 4]| {
        (0..table_count)
            .map(|i| 12 + i * 16)
            .find(|&offset| &source[offset..offset + 4] == tag)
            .unwrap()
    };
    let cmap_record = record(b"cmap");
    let head_record = record(b"head");
    let head = u32::from_be_bytes(
        source[head_record + 8..head_record + 12]
            .try_into()
            .unwrap(),
    ) as usize;
    let mut cmap = Vec::new();
    for value in [0u16, 1, 3, 10] {
        cmap.extend(value.to_be_bytes());
    }
    cmap.extend(12u32.to_be_bytes());
    cmap.extend(12u16.to_be_bytes());
    cmap.extend(0u16.to_be_bytes());
    let mut mappings = mappings.to_vec();
    mappings.sort_unstable();
    for value in [16 + mappings.len() as u32 * 12, 0, mappings.len() as u32] {
        cmap.extend(value.to_be_bytes());
    }
    for (character, glyph) in mappings {
        for value in [character as u32, character as u32, u32::from(glyph)] {
            cmap.extend(value.to_be_bytes());
        }
    }
    let checksum = |data: &[u8]| {
        data.chunks(4).fold(0u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    let offset = bytes.len().next_multiple_of(4);
    bytes.resize(offset, 0);
    bytes.extend(&cmap);
    bytes[cmap_record + 4..cmap_record + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    bytes[cmap_record + 8..cmap_record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
    bytes[cmap_record + 12..cmap_record + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
    bytes[head + 8..head + 12].fill(0);
    let adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xB1B0AFBA);
    bytes
}
