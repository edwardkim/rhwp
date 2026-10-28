//! 미주 줄·컨트롤 진단 출력.

use crate::renderer::typeset::{
    composed_line_char_end, debug_brief_line_text, hwpunit_to_px, tac_control_indices_for_line,
    ComposedParagraph, Control, FormattedParagraph, Paragraph,
};

pub(in crate::renderer::typeset) fn debug_endnote_control_kind(ctrl: &Control) -> &'static str {
    match ctrl {
        Control::Equation(_) => "eq",
        Control::Picture(pic) if pic.common.treat_as_char => "pic:tac",
        Control::Picture(_) => "pic",
        Control::Shape(shape) if shape.common().treat_as_char => "shape:tac",
        Control::Shape(_) => "shape",
        Control::Table(table) if table.common.treat_as_char => "table:tac",
        Control::Table(_) => "table",
        _ => "ctrl",
    }
}

pub(in crate::renderer::typeset) fn debug_endnote_control_height_hu(ctrl: &Control) -> Option<i32> {
    match ctrl {
        Control::Equation(eq) => Some(eq.common.height as i32),
        Control::Picture(pic) => Some(pic.common.height as i32),
        Control::Shape(shape) => Some(shape.common().height as i32),
        Control::Table(table) => Some(table.common.height as i32),
        _ => None,
    }
}

pub(in crate::renderer::typeset) fn debug_print_endnote_line_segments(
    note_number: u16,
    ep_idx: usize,
    para: &Paragraph,
    comp: &ComposedParagraph,
    fmt: &FormattedParagraph,
    dpi: f64,
    endnote_start: i32,
) {
    use std::fmt::Write as _;

    let control_positions = para.control_text_positions();
    let para_text = debug_brief_line_text(&para.text, 120);
    eprintln!(
        "ENDNOTE_LINE note={} ep={} para_chars={} line_segs={} comp_lines={} fmt_lines={} start={} text=\"{}\"",
        note_number,
        ep_idx,
        para.char_count,
        para.line_segs.len(),
        comp.lines.len(),
        fmt.line_heights.len(),
        endnote_start,
        para_text
    );

    for line_idx in 0..fmt
        .line_heights
        .len()
        .max(comp.lines.len())
        .max(para.line_segs.len())
    {
        let seg = para.line_segs.get(line_idx);
        let comp_line = comp.lines.get(line_idx);
        let (comp_start, comp_end, runs_empty, run_text) = if let Some(line) = comp_line {
            let text = line
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>();
            (
                Some(line.char_start),
                Some(composed_line_char_end(comp, line_idx)),
                line.runs.is_empty(),
                debug_brief_line_text(&text, 80),
            )
        } else {
            (None, None, false, String::new())
        };

        let mut tac_desc = String::new();
        let tac_indices = if line_idx < comp.lines.len() {
            tac_control_indices_for_line(para, comp, line_idx)
        } else {
            Vec::new()
        };
        for ci in tac_indices {
            if !tac_desc.is_empty() {
                tac_desc.push(',');
            }
            if let Some(ctrl) = para.controls.get(ci) {
                let pos = control_positions.get(ci).copied();
                let height = debug_endnote_control_height_hu(ctrl)
                    .map(|h| h.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let _ = write!(
                    tac_desc,
                    "{}@{:?}:{}h{}",
                    ci,
                    pos,
                    debug_endnote_control_kind(ctrl),
                    height
                );
            } else {
                let _ = write!(tac_desc, "{}@?:missing", ci);
            }
        }

        let fmt_lh = fmt.line_heights.get(line_idx).copied();
        let fmt_ls = fmt.line_spacings.get(line_idx).copied();
        let fmt_adv = fmt_lh.zip(fmt_ls).map(|(h, s)| h + s);
        eprintln!(
            "ENDNOTE_LINE note={} ep={} line={} seg_ts={:?} seg_char={:?} seg_vpos={:?} seg_abs={:?} seg_lh={:?} seg_th={:?} seg_ls={:?} fmt_lh={:?} fmt_ls={:?} fmt_adv={:?} comp={:?}..{:?} runs_empty={} tac=[{}] text=\"{}\"",
            note_number,
            ep_idx,
            line_idx,
            seg.map(|s| s.text_start),
            seg.map(|s| para.utf16_pos_to_char_idx(s.text_start)),
            seg.map(|s| s.vertical_pos),
            seg.map(|s| s.vertical_pos + endnote_start),
            seg.map(|s| hwpunit_to_px(s.line_height, dpi)),
            seg.map(|s| hwpunit_to_px(s.text_height, dpi)),
            seg.map(|s| hwpunit_to_px(s.line_spacing, dpi)),
            fmt_lh,
            fmt_ls,
            fmt_adv,
            comp_start,
            comp_end,
            runs_empty,
            tac_desc,
            run_text
        );
    }
}
