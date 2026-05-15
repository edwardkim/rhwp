use std::fs;
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

fn main() {
    let path = std::env::args().nth(1).expect("path required");
    let data = fs::read(&path).expect("read");
    let mut cursor = Cursor::new(&data);

    // Skip placeable header if present (22 bytes)
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic).unwrap();
    if magic == [0xD7, 0xCD, 0xC6, 0x9A] {
        cursor.set_position(22);
    } else {
        cursor.set_position(0);
    }

    // Skip standard WMF header (18 bytes)
    cursor.set_position(cursor.position() + 18);

    let mut total = 0u64;
    let mut by_type: std::collections::BTreeMap<u16, u64> = Default::default();

    loop {
        let pos_before = cursor.position();
        let rec_size = match cursor.read_u32::<LittleEndian>() {
            Ok(s) => s,
            Err(_) => break,
        };
        let rec_func = match cursor.read_u16::<LittleEndian>() {
            Ok(f) => f,
            Err(_) => break,
        };
        total += 1;
        *by_type.entry(rec_func).or_insert(0) += 1;
        if rec_func == 0 { break; } // META_EOF

        // rec_size is in WORDs (2 bytes each), minus the 6 bytes of header (size+func)
        let body_size = (rec_size as u64 * 2).saturating_sub(6);
        let new_pos = pos_before + 6 + body_size;
        if new_pos > data.len() as u64 { break; }
        cursor.set_position(new_pos);
    }

    println!("Total records: {}", total);
    println!("Top record types:");
    let mut sorted: Vec<_> = by_type.into_iter().collect();
    sorted.sort_by_key(|&(_, c)| std::cmp::Reverse(c));
    for (func, count) in sorted.iter().take(20) {
        println!("  func={:#06X}  count={}", func, count);
    }
}
