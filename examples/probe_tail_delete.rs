use rhwp::document_core::DocumentCore;

fn line_texts(core: &DocumentCore, section: usize, para: usize) -> Vec<(u32, String)> {
    let p = &core.document().sections[section].paragraphs[para];
    let chars: Vec<char> = p.text.chars().collect();
    p.line_segs
        .iter()
        .enumerate()
        .map(|(i, ls)| {
            let start = if i == 0 {
                0
            } else {
                p.char_offsets
                    .iter()
                    .position(|off| *off >= ls.text_start)
                    .unwrap_or(chars.len())
            };
            let end = p
                .line_segs
                .get(i + 1)
                .and_then(|next| {
                    p.char_offsets
                        .iter()
                        .position(|off| *off >= next.text_start)
                })
                .unwrap_or(chars.len());
            (ls.text_start, chars[start..end].iter().collect())
        })
        .collect()
}

fn main() {
    let path = std::env::args().nth(1).expect("path to hwpx/hwp");
    let section: usize = std::env::args()
        .nth(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let para: usize = std::env::args()
        .nth(3)
        .and_then(|v| v.parse().ok())
        .unwrap_or(17);

    let bytes = std::fs::read(&path).expect("read");
    let base = DocumentCore::from_bytes(&bytes).expect("parse");
    let styles = rhwp::renderer::style_resolver::resolve_styles(&base.document().doc_info, 96.0);
    let p = &base.document().sections[section].paragraphs[para];
    let ps = &styles.para_styles[p.para_shape_id as usize];
    println!(
        "para_shape={} indent={:.2}px condense={} eng_break={} kor_break={}",
        p.para_shape_id,
        ps.indent,
        ps.condense_min_space,
        ps.english_break_unit,
        ps.korean_break_unit
    );
    for cs in &p.char_shapes {
        let raw = &base.document().doc_info.char_shapes[cs.char_shape_id as usize];
        let style = &styles.char_styles[cs.char_shape_id as usize];
        println!(
            "cs pos={} id={} family={} size={:.2}px ratio={:.3} spacing={:.3} raw_spacings={:?}",
            cs.start_pos,
            cs.char_shape_id,
            style.font_family,
            style.font_size,
            style.ratio,
            style.letter_spacing,
            raw.spacings
        );
    }
    println!("original:");
    for (ts, text) in line_texts(&base, section, para) {
        println!("  {ts:>3}: {text}");
    }
    let composed = rhwp::renderer::composer::compose_paragraph(p);
    println!(
        "original_line_widths_hwp={:?}",
        composed
            .lines
            .iter()
            .map(
                |line| (rhwp::renderer::composer::estimate_composed_line_width(line, &styles)
                    * 75.0)
                    .round() as i32
            )
            .collect::<Vec<_>>()
    );
    if p.line_segs.len() > 1 {
        let mut extended = p.clone();
        extended.line_segs[1].text_start = 56;
        let extended = rhwp::renderer::composer::compose_paragraph(&extended);
        println!(
            "first_endpoint_56_width_hwp={}",
            (rhwp::renderer::composer::estimate_composed_line_width(&extended.lines[0], &styles)
                * 75.0)
                .round() as i32
        );
    }

    for (numerator, denominator) in [(1i16, 1i16), (7, 8), (3, 4), (1, 2)] {
        let mut candidate = DocumentCore::from_bytes(&bytes).expect("parse candidate");
        for shape in &mut candidate.document_mut().doc_info.char_shapes {
            for spacing in &mut shape.spacings {
                *spacing = (*spacing as i16 * numerator / denominator) as i8;
            }
        }
        candidate.reflow_linesegs_on_demand();
        println!(
            "spacing_factor={numerator}/{denominator} starts={:?} segment_width={:?}",
            candidate.document().sections[section].paragraphs[para]
                .line_segs
                .iter()
                .map(|seg| seg.text_start)
                .collect::<Vec<_>>(),
            candidate.document().sections[section].paragraphs[para]
                .line_segs
                .first()
                .map(|seg| seg.segment_width)
        );
    }

    for delta in 1i16..=4 {
        let mut candidate = DocumentCore::from_bytes(&bytes).expect("parse spacing delta");
        for shape in &mut candidate.document_mut().doc_info.char_shapes {
            for spacing in &mut shape.spacings {
                *spacing = (*spacing as i16 + delta).clamp(-100, 100) as i8;
            }
        }
        candidate.reflow_linesegs_on_demand();
        println!(
            "spacing_delta=+{delta} starts={:?}",
            candidate.document().sections[section].paragraphs[para]
                .line_segs
                .iter()
                .map(|seg| seg.text_start)
                .collect::<Vec<_>>()
        );
    }

    let text = &base.document().sections[section].paragraphs[para].text;
    println!("len={}", text.chars().count());
    for needle in ["관련된", "목표설정과", "볼 수 있다", "운영모형"] {
        if let Some(byte_idx) = text.find(needle) {
            let char_idx = text[..byte_idx].chars().count();
            println!("needle {needle:?} at char {char_idx}");
        }
    }

    for delete_at in [
        0usize,
        39,
        40,
        76,
        77,
        120,
        121,
        160,
        161,
        text.chars().count() - 1,
    ] {
        let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
        let deleted = core.document().sections[section].paragraphs[para]
            .text
            .chars()
            .nth(delete_at)
            .unwrap_or('\0');
        core.delete_text_native(section, para, delete_at, 1)
            .expect("delete");
        let starts: Vec<u32> = core.document().sections[section].paragraphs[para]
            .line_segs
            .iter()
            .map(|ls| ls.text_start)
            .collect();
        println!("\ndelete_at={delete_at} deleted={deleted:?} starts={starts:?}");
        for (ts, text) in line_texts(&core, section, para) {
            println!("  {ts:>3}: {text}");
        }
    }

    for insert_at in [0usize, 39, 40, 77, 121, 161, text.chars().count()] {
        let mut core = DocumentCore::from_bytes(&bytes).expect("parse");
        core.insert_text_native(section, para, insert_at, "1")
            .expect("insert");
        let starts: Vec<u32> = core.document().sections[section].paragraphs[para]
            .line_segs
            .iter()
            .map(|ls| ls.text_start)
            .collect();
        println!("\ninsert_at={insert_at} inserted='1' starts={starts:?}");
        for (ts, text) in line_texts(&core, section, para) {
            println!("  {ts:>3}: {text}");
        }
    }
}
