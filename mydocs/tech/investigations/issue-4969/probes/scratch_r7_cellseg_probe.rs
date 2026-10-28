//! scratch: 셀 문단 lineseg 원값 확인 (RHWP_CS_SRC/PAT).
use rhwp::model::control::Control;

fn walk(paras: &[rhwp::model::paragraph::Paragraph], pat: &str, depth: usize, found: &mut usize) {
    for p in paras {
        if p.text.contains(pat) && *found < 3 {
            *found += 1;
            println!(
                "depth={depth} text={:?} segs={:?}",
                p.text.chars().take(12).collect::<String>(),
                p.line_segs
                    .iter()
                    .map(|s| (
                        s.line_height,
                        s.text_height,
                        s.baseline_distance,
                        s.line_spacing
                    ))
                    .collect::<Vec<_>>()
            );
        }
        for c in &p.controls {
            if let Control::Table(t) = c {
                for cell in &t.cells {
                    walk(&cell.paragraphs, pat, depth + 1, found);
                }
            }
        }
    }
}

fn main() {
    let src = std::env::var("RHWP_CS_SRC").unwrap();
    let pat = std::env::var("RHWP_CS_PAT").unwrap();
    let data = std::fs::read(&src).unwrap();
    let doc = rhwp::parser::parse_document(&data).unwrap();
    let mut found = 0usize;
    for sec in &doc.sections {
        walk(&sec.paragraphs, &pat, 0, &mut found);
    }
}
