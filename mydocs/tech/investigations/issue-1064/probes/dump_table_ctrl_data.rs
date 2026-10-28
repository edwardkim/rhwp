// Task #1064 — 표 control 의 CTRL_DATA payload 정답지 vs 저장본 비교.
// HWPTAG_CTRL_DATA = 87 (HWPTAG_BEGIN + 71). 표 control 직후 level=2 자식 record.
// ParameterSet 구조 (hwplib ForParameterSet.java 정합):
//   - outer.id (UINT2)
//   - outer.count (SINT2)
//   - skip 2 byte
//   - outer items: [id (UINT2), type (UINT2), value (가변)]
//
// ParameterType:
//   0=NULL, 1=String, 2=Integer1, 3=Integer2, 4=Integer4, 5=Integer,
//   6=UnsignedInteger1, 7=UnsignedInteger2, 8=UnsignedInteger4, 9=UnsignedInteger,
//   0x8000=ParameterSet, 0x8001=Array, 0x8002=BINDataID
use std::io::Read;

fn read_section0(path: &str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read");
    let cursor = std::io::Cursor::new(bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FH");
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);
    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read Section0");
    if is_compressed {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out).expect("inflate");
        out
    } else {
        raw
    }
}

fn parse_parameter_set(data: &[u8], indent: &str) -> usize {
    if data.len() < 8 {
        println!("{}<too short>", indent);
        return data.len();
    }
    let outer_id = u16::from_le_bytes([data[0], data[1]]);
    let outer_count = i16::from_le_bytes([data[2], data[3]]);
    // skip 2
    println!(
        "{}ParameterSet id=0x{:04X} count={}",
        indent, outer_id, outer_count
    );
    let mut pos = 6;
    for i in 0..outer_count as usize {
        if pos + 4 > data.len() {
            println!("{}  [item {}] <too short>", indent, i);
            break;
        }
        let item_id = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let item_type = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
        let type_name = match item_type {
            0 => "NULL",
            1 => "String",
            2 => "I1",
            3 => "I2",
            4 => "I4",
            5 => "I",
            6 => "UI1",
            7 => "UI2",
            8 => "UI4",
            9 => "UI",
            0x8000 => "ParameterSet",
            0x8001 => "Array",
            0x8002 => "BINDataID",
            _ => "Unknown",
        };
        pos += 4;
        // value
        match item_type {
            4 => {
                if pos + 4 <= data.len() {
                    let val = i32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]);
                    println!(
                        "{}  [{}] id=0x{:04X} type={} value={} (0x{:08X})",
                        indent, i, item_id, type_name, val, val as u32
                    );
                    pos += 4;
                }
            }
            0x8000 => {
                println!(
                    "{}  [{}] id=0x{:04X} type={} (recurse):",
                    indent, i, item_id, type_name
                );
                let consumed = parse_parameter_set(&data[pos..], &format!("{}    ", indent));
                pos += consumed;
            }
            _ => {
                println!(
                    "{}  [{}] id=0x{:04X} type={} (skip — unhandled type)",
                    indent, i, item_id, type_name
                );
                break;
            }
        }
    }
    pos
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("Usage: dump_table_ctrl_data <hwp1> [hwp2 ...]");
        std::process::exit(2);
    }
    for path in &paths {
        let data = read_section0(path);
        println!("\n=== [{}] section0 ({} bytes) ===", path, data.len());

        let mut pos = 0;
        let mut rec_idx = 0;
        let mut last_ctrl_id: Option<String> = None;
        let mut ctrl_data_idx = 0;
        while pos + 4 <= data.len() {
            let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            let tag = (hdr & 0x3FF) as u16;
            let level = ((hdr >> 10) & 0x3FF) as u16;
            let mut size = ((hdr >> 20) & 0xFFF) as usize;
            pos += 4;
            if size == 0xFFF && pos + 4 <= data.len() {
                size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
                pos += 4;
            }
            if tag == 71 && size >= 4 {
                // CTRL_HEADER
                let ascii: String = data[pos..pos + 4].iter().map(|&b| b as char).collect();
                last_ctrl_id = Some(ascii.clone());
            }
            if tag == 87 {
                // CTRL_DATA
                ctrl_data_idx += 1;
                println!(
                    "\nCTRL_DATA #{} rec_idx={} level={} size={} (prev_ctrl_id={:?})",
                    ctrl_data_idx, rec_idx, level, size, last_ctrl_id
                );
                let payload = &data[pos..pos + size];
                // hex
                for i in (0..payload.len()).step_by(16) {
                    let hex = (0..16)
                        .filter_map(|j| payload.get(i + j))
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("  {:04X}: {}", i, hex);
                }
                println!("  ── parsed ──");
                parse_parameter_set(payload, "  ");
            }
            pos += size;
            rec_idx += 1;
        }
        println!(
            "\n→ total CTRL_DATA records: {}, last_ctrl_id processed: {:?}",
            ctrl_data_idx, last_ctrl_id
        );
    }
}
