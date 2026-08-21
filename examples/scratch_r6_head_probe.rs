//! scratch: HWP5 스트림 N 에서 head/foot 컨트롤 서브트리 덤프 (RHWP_HP_SRC/SEC).
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;

fn main() {
    let src = std::env::var("RHWP_HP_SRC").unwrap();
    let sec: usize = std::env::var("RHWP_HP_SEC").unwrap().parse().unwrap();
    let data = std::fs::read(&src).unwrap();
    let mut cfb = CfbReader::open(&data).unwrap();
    let header = cfb.read_file_header().unwrap();
    let fh = rhwp::parser::header::parse_file_header(&header).unwrap();
    let section = cfb
        .read_body_text_section(sec as u32, fh.flags.compressed, false)
        .unwrap();
    let records = Record::read_all(&section).unwrap();
    println!("stream {sec}: {} records", records.len());
    let mut in_sub = None::<u16>;
    for (i, rec) in records.iter().enumerate() {
        let is_hf_ctrl = rec.tag_id == 71
            && rec.data.len() >= 4
            && (rec.data[..4] == [0x64, 0x61, 0x65, 0x68] || rec.data[..4] == [0x74, 0x6f, 0x6f, 0x66]);
        if is_hf_ctrl {
            in_sub = Some(rec.level);
        } else if let Some(base) = in_sub {
            if rec.level <= base && rec.tag_id != 71 {
                in_sub = None;
            }
        }
        if is_hf_ctrl || in_sub.is_some() {
            let hex: String = rec
                .data
                .iter()
                .take(48)
                .map(|b| format!("{b:02x} "))
                .collect();
            println!("[{i:4}] tag={} lv={} sz={} | {}", rec.tag_id, rec.level, rec.data.len(), hex);
            if rec.tag_id == 67 {
                let txt: String = rec
                    .data
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .map(|u| {
                        if (32..0xd800).contains(&u) {
                            char::from_u32(u as u32).unwrap_or('?')
                        } else {
                            '·'
                        }
                    })
                    .collect();
                println!("        TEXT: {txt}");
            }
        }
    }
}
