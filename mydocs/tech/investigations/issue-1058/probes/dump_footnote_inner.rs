// Dump footnote inner_para for two HWP files and compare contracts.
use rhwp::model::control::Control;
use rhwp::parser::parse_document;

fn dump_para_contract(prefix: &str, para: &rhwp::model::paragraph::Paragraph) {
    println!(
        "{}text={:?} char_count={} char_count_msb={} text_chars={} controls={} control_mask=0x{:X}",
        prefix,
        para.text,
        para.char_count,
        para.char_count_msb,
        para.text.chars().count(),
        para.controls.len(),
        para.control_mask,
    );
    println!("{}  char_offsets={:?}", prefix, para.char_offsets);
    println!(
        "{}  para_shape_id={} style_id={}",
        prefix, para.para_shape_id, para.style_id
    );
    for (i, cs) in para.char_shapes.iter().enumerate() {
        println!(
            "{}  char_shapes[{}] start_pos={} char_shape_id={}",
            prefix, i, cs.start_pos, cs.char_shape_id
        );
    }
    for (i, ctrl) in para.controls.iter().enumerate() {
        let label = match ctrl {
            Control::AutoNumber(an) => format!(
                "AutoNumber type={:?} number={} suffix={:?}",
                an.number_type, an.number, an.suffix_char
            ),
            other => format!("{:?}", std::mem::discriminant(other)),
        };
        println!("{}  controls[{}] = {}", prefix, i, label);
    }
    for (i, ls) in para.line_segs.iter().enumerate() {
        println!(
            "{}  line_segs[{}] ts={} lh={} sw={} tag=0x{:X}",
            prefix, i, ls.text_start, ls.line_height, ls.segment_width, ls.tag
        );
    }
}

fn walk_paragraphs(
    label: &str,
    paragraphs: &[rhwp::model::paragraph::Paragraph],
    section_idx: usize,
) {
    let mut footnote_idx = 0usize;
    for (pi, para) in paragraphs.iter().enumerate() {
        for ctrl in &para.controls {
            if let Control::Footnote(fn_) = ctrl {
                println!(
                    "\n=== [{}] section {} pi {} — Footnote #{} (paragraphs={}) ===",
                    label,
                    section_idx,
                    pi,
                    footnote_idx,
                    fn_.paragraphs.len()
                );
                for (fpi, inner) in fn_.paragraphs.iter().enumerate() {
                    println!("  -- inner_para[{}] --", fpi);
                    dump_para_contract("    ", inner);
                }
                footnote_idx += 1;
            }
        }
    }
}

fn main() {
    let paths = std::env::args().skip(1).collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("Usage: dump_footnote_inner <hwp1> [hwp2 ...]");
        std::process::exit(2);
    }
    for path in &paths {
        let data = std::fs::read(path).unwrap_or_else(|e| {
            eprintln!("read {} failed: {}", path, e);
            std::process::exit(1);
        });
        let doc = parse_document(&data).unwrap_or_else(|e| {
            eprintln!("parse {} failed: {:?}", path, e);
            std::process::exit(1);
        });
        for (si, section) in doc.sections.iter().enumerate() {
            walk_paragraphs(path, &section.paragraphs, si);
        }
    }
}
