// stdin 으로 hwp file 받아 첫 EQEDIT raw payload 출력
use std::io::Read;

fn main() {
    let path = std::env::args().nth(1).expect("path");
    let bytes = std::fs::read(&path).unwrap();
    let cursor = std::io::Cursor::new(&bytes);
    let mut comp = cfb::CompoundFile::open(cursor).unwrap();
    let mut fh = comp.open_stream("/FileHeader").unwrap();
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).unwrap();
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let mut sec0 = comp.open_stream("/BodyText/Section0").unwrap();
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).unwrap();
    let data = if is_compressed {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    } else {
        raw
    };

    let mut pos = 0;
    let mut eqedit_count = 0;
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (hdr & 0x3FF) as u16;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        if tag == 88 {
            eqedit_count += 1;
            println!("EQEDIT #{} size={}, payload (hex):", eqedit_count, size);
            for (i, byte) in data[pos..pos + size].iter().enumerate() {
                if i % 16 == 0 {
                    print!("  {:04X}: ", i);
                }
                print!("{:02X} ", byte);
                if i % 16 == 15 {
                    println!();
                }
            }
            if !size.is_multiple_of(16) {
                println!();
            }
            // 첫 2개 EQEDIT만
            if eqedit_count >= 2 {
                break;
            }
        }
        pos += size;
    }
}
