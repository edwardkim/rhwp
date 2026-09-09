//! scratch: 문단 컨트롤의 wrap/TAC/offset 요약 (RHWP_CP_SRC/SEC/PARA).
use rhwp::model::control::Control;

fn main() {
    let src = std::env::var("RHWP_CP_SRC").unwrap();
    let sec: usize = std::env::var("RHWP_CP_SEC").unwrap().parse().unwrap();
    let pi: usize = std::env::var("RHWP_CP_PARA").unwrap().parse().unwrap();
    let data = std::fs::read(&src).unwrap();
    let doc = rhwp::parser::parse_document(&data).unwrap();
    let para = &doc.sections[sec].paragraphs[pi];
    for (ci, c) in para.controls.iter().enumerate() {
        if let Control::Table(t) = c {
            println!(
                "[{ci}] table {}x{} tac={} wrap={:?} vert_rel={:?} v_off={} h_off={} h={}",
                t.row_count,
                t.col_count,
                t.common.treat_as_char,
                t.common.text_wrap,
                t.common.vert_rel_to,
                t.common.vertical_offset as i32,
                t.common.horizontal_offset as i32,
                t.common.height,
            );
        }
    }
    for (li, ls) in para.line_segs.iter().enumerate() {
        println!("ls[{li}] vpos={} lh={}", ls.vertical_pos, ls.line_height);
    }
}
