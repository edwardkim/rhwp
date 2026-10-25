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

#[allow(dead_code)]
pub fn with_bitmap_strikes(source: &[u8], ink: &[bool]) -> Vec<u8> {
    // EBDT format 1 / EBLC index format 1: 12×12, gid 1, 1-bit.
    // strike가 여러 개일 때 중간 크기의 정상 잉크도 반드시 보존해야 한다.
    let mut ebdt = 0x00020000u32.to_be_bytes().to_vec();
    let mut eblc = 0x00020000u32.to_be_bytes().to_vec();
    eblc.extend((ink.len() as u32).to_be_bytes());
    let array_start = 8 + ink.len() * 48;
    let mut arrays = Vec::new();
    for (index, has_ink) in ink.iter().enumerate() {
        let mut record = vec![0; 48];
        record[..4].copy_from_slice(&((array_start + arrays.len()) as u32).to_be_bytes());
        record[4..8].copy_from_slice(&24u32.to_be_bytes());
        record[8..12].copy_from_slice(&1u32.to_be_bytes());
        record[40..42].copy_from_slice(&1u16.to_be_bytes());
        record[42..44].copy_from_slice(&1u16.to_be_bytes());
        record[44..48].copy_from_slice(&[12 + index as u8 * 2, 12 + index as u8 * 2, 1, 1]);
        eblc.extend(record);
        for value in [1u16, 1] {
            arrays.extend(value.to_be_bytes());
        }
        arrays.extend(8u32.to_be_bytes());
        for value in [1u16, 1] {
            arrays.extend(value.to_be_bytes());
        }
        arrays.extend((ebdt.len() as u32).to_be_bytes());
        arrays.extend(0u32.to_be_bytes());
        arrays.extend(29u32.to_be_bytes());
        ebdt.extend([12, 12, 0, 10, 12]);
        let mut pixels = [0; 24];
        if *has_ink {
            pixels[0] = 0x80;
        }
        ebdt.extend(pixels);
    }
    eblc.extend(arrays);
    let count = u16::from_be_bytes(source[4..6].try_into().unwrap()) as usize;
    let mut tables = std::collections::BTreeMap::new();
    for i in 0..count {
        let record = &source[12 + i * 16..12 + (i + 1) * 16];
        let tag: [u8; 4] = record[..4].try_into().unwrap();
        let start = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
        tables.insert(tag, source[start..start + len].to_vec());
    }
    tables.insert(*b"EBLC", eblc);
    tables.insert(*b"EBDT", ebdt);
    tables.get_mut(b"head").unwrap()[8..12].fill(0);
    let count = tables.len();
    let power = 1 << count.ilog2();
    let mut out = source[..4].to_vec();
    for value in [
        count,
        power * 16,
        count.ilog2() as usize,
        count * 16 - power * 16,
    ] {
        out.extend((value as u16).to_be_bytes());
    }
    out.resize(12 + count * 16, 0);
    let checksum = |data: &[u8]| {
        data.chunks(4).fold(0u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    let mut head = 0;
    for (i, (tag, data)) in tables.into_iter().enumerate() {
        out.resize(out.len().next_multiple_of(4), 0);
        let offset = out.len();
        if &tag == b"head" {
            head = offset;
        }
        let record = 12 + i * 16;
        out[record..record + 4].copy_from_slice(&tag);
        out[record + 4..record + 8].copy_from_slice(&checksum(&data).to_be_bytes());
        out[record + 8..record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
        out[record + 12..record + 16].copy_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend(data);
    }
    out.resize(out.len().next_multiple_of(4), 0);
    let adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum(&out));
    out[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&out), 0xB1B0AFBA);
    out
}
