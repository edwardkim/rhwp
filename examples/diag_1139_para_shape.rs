use std::env;
use std::fs;

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::parse_document;
use rhwp::renderer::style_resolver::resolve_styles_with_variant;

fn collect_endnote_paragraphs<'a>(paras: &'a [Paragraph], out: &mut Vec<&'a Paragraph>) {
    for para in paras {
        for ctrl in &para.controls {
            match ctrl {
                Control::Endnote(en) => out.extend(en.paragraphs.iter()),
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_endnote_paragraphs(&cell.paragraphs, out);
                    }
                }
                Control::Shape(shape) => {
                    if let Some(text_box) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                        collect_endnote_paragraphs(&text_box.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "samples/3-09월_교육_통합_2022.hwp".to_string());
    let bytes = fs::read(&path).expect("read hwp");
    let doc = parse_document(&bytes).expect("parse document");
    let styles = resolve_styles_with_variant(&doc.doc_info, 96.0, doc.is_hwp3_variant);

    let mut endnote_paras = Vec::new();
    for section in &doc.sections {
        collect_endnote_paragraphs(&section.paragraphs, &mut endnote_paras);
    }

    println!(
        "is_hwp3_variant={} endnote_paragraphs={} para_shapes={}",
        doc.is_hwp3_variant,
        endnote_paras.len(),
        doc.doc_info.para_shapes.len()
    );

    for (idx, para) in endnote_paras.iter().enumerate() {
        let text = para.text.replace('\r', "\\r").replace('\n', "\\n");
        let interesting = text.contains("A와 C")
            || text.contains("A와")
            || text.contains("경우의 수는 2")
            || text.contains("즉,")
            || text.contains("(ⅲ)")
            || text.contains("이웃하는 경우");
        if !interesting {
            continue;
        }

        let ps_id = para.para_shape_id as usize;
        let ps = doc.doc_info.para_shapes.get(ps_id);
        let rs = styles.para_styles.get(ps_id);
        println!("\n# endnote_para={idx} ps_id={ps_id}");
        println!("text={text}");
        if let Some(ps) = ps {
            println!(
                "raw attr1=0x{:08x} align={:?} line_type={:?} line={} ml={} mr={} indent={} before={} after={}",
                ps.attr1,
                ps.alignment,
                ps.line_spacing_type,
                ps.line_spacing,
                ps.margin_left,
                ps.margin_right,
                ps.indent,
                ps.spacing_before,
                ps.spacing_after
            );
        }
        if let Some(rs) = rs {
            println!(
                "resolved align={:?} line_type={:?} line={} ml_px={:.3} mr_px={:.3} indent_px={:.3} before_px={:.3} after_px={:.3}",
                rs.alignment,
                rs.line_spacing_type,
                rs.line_spacing,
                rs.margin_left,
                rs.margin_right,
                rs.indent,
                rs.spacing_before,
                rs.spacing_after
            );
        }
        for (li, seg) in para.line_segs.iter().enumerate() {
            let line_end = para
                .line_segs
                .get(li + 1)
                .map(|next| next.text_start)
                .unwrap_or(para.char_count);
            let line_len = line_end.saturating_sub(seg.text_start);
            println!(
                "  line[{li}] vpos={} height={} spacing={} text_h={} base={} col_start={} width={} start={} len={}",
                seg.vertical_pos,
                seg.line_height,
                seg.line_spacing,
                seg.text_height,
                seg.baseline_distance,
                seg.column_start,
                seg.segment_width,
                seg.text_start,
                line_len
            );
        }
    }
}
