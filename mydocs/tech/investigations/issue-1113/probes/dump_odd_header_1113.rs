// Task #1113 — ODD/EVEN 머리말 head subtree record byte 정밀 dump.
//
// CTRL_HEADER head (apply_type=ODD/EVEN) 자식 trees 의 모든 record 를
// 정답지 / 저장본 양쪽에서 완전 byte 추출 + record-by-record 비교.
//
// 사용:
//   dump_odd_header_1113 [oracle.hwp] [target.hwp]
// 기본: samples/exam_social.hwp  vs  output/poc/issue_1113/exam_social-current.hwp

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags::HWPTAG_CTRL_HEADER;

const CTRL_HEAD: u32 = u32::from_le_bytes(*b"daeh"); // "head" reversed (LE u32)

fn load_section0(path: &str) -> Vec<Record> {
    let bytes = std::fs::read(path).expect("read");
    let mut cfb = CfbReader::open(&bytes).expect("cfb open");
    let header = cfb.read_file_header().expect("filehdr");
    let compressed = (header[36] & 0x01) != 0;
    let data = cfb
        .read_body_text_section(0, compressed, false)
        .expect("section0");
    Record::read_all(&data).expect("records")
}

fn ctrl_id_le(payload: &[u8]) -> Option<u32> {
    if payload.len() < 4 {
        None
    } else {
        Some(u32::from_le_bytes([
            payload[0], payload[1], payload[2], payload[3],
        ]))
    }
}

fn header_apply_type(payload: &[u8]) -> Option<u32> {
    if payload.len() < 8 {
        None
    } else {
        Some(u32::from_le_bytes([
            payload[4], payload[5], payload[6], payload[7],
        ]))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn find_head_subtrees(records: &[Record]) -> Vec<(usize, usize, u32)> {
    // (start_idx, end_idx_exclusive, apply_type)
    let mut out = Vec::new();
    for (i, r) in records.iter().enumerate() {
        if r.tag_id != HWPTAG_CTRL_HEADER {
            continue;
        }
        if ctrl_id_le(&r.data) != Some(CTRL_HEAD) {
            continue;
        }
        let apply = header_apply_type(&r.data).unwrap_or(99);
        let head_level = r.level;
        let mut j = i + 1;
        while j < records.len() {
            if records[j].level <= head_level {
                break;
            }
            j += 1;
        }
        out.push((i, j, apply));
    }
    out
}

fn dump_subtree(records: &[Record], start: usize, end: usize, label: &str) {
    println!(
        "############### {} subtree (rec {}..{}) ###############",
        label, start, end
    );
    for (k, r) in records[start..end].iter().enumerate() {
        let indent = "  ".repeat(r.level as usize);
        println!(
            "  [{:>3}] lv={} {:<18} size={:<3}  {}{}",
            start + k,
            r.level,
            r.tag_name(),
            r.size,
            indent,
            hex(&r.data),
        );
    }
}

fn diff_subtrees(
    oracle: &[Record],
    o_range: (usize, usize),
    gen: &[Record],
    g_range: (usize, usize),
    label: &str,
) {
    println!(
        "=============== {} 정답지 vs 저장본 record diff ===============",
        label
    );
    let o = &oracle[o_range.0..o_range.1];
    let g = &gen[g_range.0..g_range.1];
    let n = o.len().max(g.len());
    for i in 0..n {
        let or = o.get(i);
        let gr = g.get(i);
        match (or, gr) {
            (Some(o), Some(g)) => {
                let same = o.tag_id == g.tag_id && o.data == g.data;
                if !same {
                    let mark = if o.tag_id != g.tag_id {
                        "TAG차이"
                    } else {
                        "BYTE차이"
                    };
                    println!(
                        "  [{}] {} {} (o:size={} g:size={})",
                        i,
                        mark,
                        o.tag_name(),
                        o.size,
                        g.size
                    );
                    println!("      oracle: {}", hex(&o.data));
                    println!("      gen   : {}", hex(&g.data));
                }
            }
            (Some(o), None) => println!(
                "  [{}] 저장본 누락: {} size={} {}",
                i,
                o.tag_name(),
                o.size,
                hex(&o.data)
            ),
            (None, Some(g)) => println!(
                "  [{}] 저장본 추가: {} size={} {}",
                i,
                g.tag_name(),
                g.size,
                hex(&g.data)
            ),
            (None, None) => {}
        }
    }
    println!("  (차이 없으면 위에 출력 없음)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let oracle_path: &str = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("samples/exam_social.hwp");
    let gen_path: &str = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("output/poc/issue_1113/exam_social-current.hwp");

    println!("oracle: {}", oracle_path);
    println!("gen   : {}", gen_path);
    println!();

    let oracle = load_section0(oracle_path);
    let gen = load_section0(gen_path);
    println!("oracle Section0 records: {}", oracle.len());
    println!("gen    Section0 records: {}", gen.len());
    println!();

    let oh = find_head_subtrees(&oracle);
    let gh = find_head_subtrees(&gen);

    println!("oracle heads:");
    for (s, e, a) in &oh {
        println!("  apply_type={} range={}..{} ({} records)", a, s, e, e - s);
    }
    println!("gen heads:");
    for (s, e, a) in &gh {
        println!("  apply_type={} range={}..{} ({} records)", a, s, e, e - s);
    }
    println!();

    // apply_type: 1, 2 (어느 것이 ODD/EVEN 인지 dump 로 확인)
    let pick = |heads: &[(usize, usize, u32)], want: u32| -> Option<(usize, usize, u32)> {
        heads.iter().find(|(_, _, a)| *a == want).copied()
    };

    for at in [1u32, 2u32] {
        let (Some(o), Some(g)) = (pick(&oh, at), pick(&gh, at)) else {
            continue;
        };
        let label = format!("apply_type={}", at);
        dump_subtree(&oracle, o.0, o.1, &format!("ORACLE {}", label));
        println!();
        dump_subtree(&gen, g.0, g.1, &format!("GEN {}", label));
        println!();
        diff_subtrees(&oracle, (o.0, o.1), &gen, (g.0, g.1), &label);
        println!();
    }
}
