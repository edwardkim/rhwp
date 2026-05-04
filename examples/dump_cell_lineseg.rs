use std::fs;
use rhwp::model::control::Control;
fn main() {
    let data = fs::read("samples/exam_science.hwp").unwrap();
    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    let p = &doc.sections[0].paragraphs[35];
    if let Some(Control::Table(t)) = p.controls.first() {
        for (i, cell) in t.cells.iter().enumerate().take(8) {
            println!("cell[{}] r={} c={} h={} pad=(l{},t{},r{},b{}) text={:?}",
                i, cell.row, cell.col, cell.height,
                cell.padding.left, cell.padding.top, cell.padding.right, cell.padding.bottom,
                cell.paragraphs[0].text.chars().take(10).collect::<String>());
            for (li, ls) in cell.paragraphs[0].line_segs.iter().enumerate() {
                println!("  ls[{}] ts={} vpos={} lh={} th={} bl={} ls={} cs={} sw={} tag=0x{:08x}",
                    li, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                    ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
            }
            for (ci, cs) in cell.paragraphs[0].char_shapes.iter().enumerate() {
                println!("  cs[{}] start_pos={} char_shape_id={}", ci, cs.start_pos, cs.char_shape_id);
            }
        }
        println!("\n--- char_shapes used (cell ㉠ — cell[1]) ---");
        let cell1 = &t.cells[1];
        let para = &cell1.paragraphs[0];
        for cs in &para.char_shapes {
            let csid = cs.char_shape_id as usize;
            if csid < doc.doc_info.char_shapes.len() {
                let cshape = &doc.doc_info.char_shapes[csid];
                println!("  cs_id={} pos={} font_ids={:?} ratios={:?} char_offsets={:?} relative_sizes={:?} base_size={}",
                    csid, cs.start_pos, cshape.font_ids, cshape.ratios, cshape.char_offsets, cshape.relative_sizes, cshape.base_size);
            }
        }
        println!("\n--- char_shapes used (cell 직선형 — cell[8]) ---");
        let cell8 = &t.cells[8];
        let para = &cell8.paragraphs[0];
        for cs in &para.char_shapes {
            let csid = cs.char_shape_id as usize;
            if csid < doc.doc_info.char_shapes.len() {
                let cshape = &doc.doc_info.char_shapes[csid];
                println!("  cs_id={} pos={} font_ids={:?} ratios={:?} char_offsets={:?} relative_sizes={:?} base_size={}",
                    csid, cs.start_pos, cshape.font_ids, cshape.ratios, cshape.char_offsets, cshape.relative_sizes, cshape.base_size);
            }
        }
    }
}
