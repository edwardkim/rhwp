// Dump equation IR + raw CFB records for two HWP files and compare contracts.
// Task #1061 oracle 방법론 — 정답지 vs 저장본 record-level diff.
use rhwp::model::control::Control;
use rhwp::parser::parse_document;

fn dump_equation_ir(prefix: &str, eq: &rhwp::model::control::Equation) {
    println!("{}Equation:", prefix);
    println!("{}  script={:?}", prefix, eq.script);
    println!("{}  font_size={}", prefix, eq.font_size);
    println!("{}  color=0x{:08X}", prefix, eq.color);
    println!("{}  baseline={}", prefix, eq.baseline);
    println!("{}  font_name={:?}", prefix, eq.font_name);
    println!("{}  version_info={:?}", prefix, eq.version_info);
    println!(
        "{}  raw_ctrl_data.len()={} (first 32: {:02X?})",
        prefix,
        eq.raw_ctrl_data.len(),
        &eq.raw_ctrl_data[..eq.raw_ctrl_data.len().min(32)],
    );
    println!(
        "{}  common: treat_as_char={} text_wrap={:?} attr=0x{:08X} instance_id=0x{:08X}",
        prefix, eq.common.treat_as_char, eq.common.text_wrap, eq.common.attr, eq.common.instance_id,
    );
    println!(
        "{}  common.size: w={} h={} (z_order={})",
        prefix, eq.common.width, eq.common.height, eq.common.z_order
    );
    println!(
        "{}  common.pos: vert_rel={:?} vert_align={:?} horz_rel={:?} horz_align={:?}",
        prefix,
        eq.common.vert_rel_to,
        eq.common.vert_align,
        eq.common.horz_rel_to,
        eq.common.horz_align,
    );
    println!(
        "{}  common.margin: l={} t={} r={} b={} (raw_extra.len={})",
        prefix,
        eq.common.margin.left,
        eq.common.margin.top,
        eq.common.margin.right,
        eq.common.margin.bottom,
        eq.common.raw_extra.len(),
    );
}

fn dump_para_context(prefix: &str, para: &rhwp::model::paragraph::Paragraph) {
    println!(
        "{}para: text={:?} char_count={} char_count_msb={} controls={} control_mask=0x{:X}",
        prefix,
        para.text,
        para.char_count,
        para.char_count_msb,
        para.controls.len(),
        para.control_mask,
    );
    println!("{}  char_offsets={:?}", prefix, para.char_offsets);
    println!(
        "{}  para_shape_id={} style_id={}",
        prefix, para.para_shape_id, para.style_id
    );
}

fn walk(label: &str, paragraphs: &[rhwp::model::paragraph::Paragraph], section_idx: usize) {
    let mut eq_idx = 0usize;
    for (pi, para) in paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if let Control::Equation(eq) = ctrl {
                println!(
                    "\n=== [{}] section {} pi {} ci {} — Equation #{} ===",
                    label, section_idx, pi, ci, eq_idx
                );
                dump_para_context("  ", para);
                dump_equation_ir("    ", eq);
                eq_idx += 1;
            }
        }
    }
}

/// CFB stream BodyText/Section0 에서 'eqed' control + EQEDIT record 추출.
fn dump_raw_records(label: &str, hwp_bytes: &[u8]) {
    use std::io::Read;
    let cursor = std::io::Cursor::new(hwp_bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb open");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FileHeader");
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read Section0");
    let data = if is_compressed {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("inflate");
        decompressed
    } else {
        raw
    };

    println!(
        "\n=== [{}] raw Section0 records (total {} bytes) ===",
        label,
        data.len()
    );
    let mut pos = 0;
    let mut eq_count = 0;
    let mut eqedit_count = 0;
    let mut ctrl_header_count = 0;
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (hdr & 0x3FF) as u16;
        let level = ((hdr >> 10) & 0x3FF) as u16;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        let rec_pos = pos;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        // HWPTAG_CTRL_HEADER = 71, HWPTAG_EQEDIT = HWPTAG_BEGIN (0x010=16) + 72 = 88
        if tag == 71 {
            ctrl_header_count += 1;
            // ctrl_id = first 4 byte UINT32 LE; ASCII 표시
            if size >= 4 {
                let cid =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                let bytes = cid.to_le_bytes();
                let ascii: String = bytes.iter().map(|&b| b as char).collect();
                if ascii == "eqed" {
                    eq_count += 1;
                    println!(
                        "  @ {} CTRL_HEADER level={} size={} ctrl_id=\"{}\" \
                         payload[0..32]={:02X?}",
                        rec_pos,
                        level,
                        size,
                        ascii,
                        &data[pos..pos + size.min(32)],
                    );
                }
            }
        }
        // HWPTAG_EQEDIT = HWPTAG_BEGIN (0x010=16) + 72 = 88
        if tag == 88 {
            eqedit_count += 1;
            println!(
                "  @ {} EQEDIT (tag=88) level={} size={} payload[0..48]={:02X?}",
                rec_pos,
                level,
                size,
                &data[pos..pos + size.min(48)],
            );
        }
        pos += size;
    }
    println!(
        "  → eqed CTRL_HEADER count: {}, EQEDIT record count: {}, total CTRL_HEADER: {}",
        eq_count, eqedit_count, ctrl_header_count
    );
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("Usage: dump_equation_records <hwp1> [hwp2 ...]");
        std::process::exit(2);
    }
    for path in &paths {
        let bytes = std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("read {}: {}", path, e);
            std::process::exit(1);
        });
        let doc = parse_document(&bytes).unwrap_or_else(|e| {
            eprintln!("parse {}: {:?}", path, e);
            std::process::exit(1);
        });
        for (si, section) in doc.sections.iter().enumerate() {
            walk(path, &section.paragraphs, si);
        }
        dump_raw_records(path, &bytes);
    }
}
