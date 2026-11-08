use rhwp::model::{control::Control, paragraph::Paragraph};
use std::{env, fs, path::Path};

fn clear_cache(p: &mut Paragraph) {
    p.line_segs.clear();
    p.source_line_seg_vertical_pos = None;
    for c in &mut p.controls {
        if let Control::Table(t) = c {
            for cell in &mut t.cells {
                for p in &mut cell.paragraphs {
                    clear_cache(p);
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err(
            "usage: cell_flow_probe INPUT.hwpx OUTPUT_DIR MODE(whole-reflow|single-child)".into(),
        );
    }
    let mut d = rhwp::parse_document(&fs::read(&args[1])?)?;
    let root = d.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .ok_or("root table required")?;
    if args[3] == "single-child" {
        let cell =
            root.cells
                .iter_mut()
                .find(|c| {
                    c.paragraphs.iter().any(|p| {
                        p.controls.iter()
            .any(|c| matches!(c, Control::Table(t) if t.row_count == 1 && t.col_count == 2))
                    })
                })
                .ok_or("linear child table required")?;
        let host = cell
            .paragraphs
            .iter()
            .find(|p| {
                p.controls
                    .iter()
                    .any(|c| matches!(c, Control::Table(t) if t.row_count == 1 && t.col_count == 2))
            })
            .ok_or("child host required")?
            .clone();
        let mut selected = cell.clone();
        selected.row = 0;
        selected.col = 0;
        selected.row_span = 1;
        selected.col_span = 1;
        selected.height = 20000;
        selected.padding.top = 0;
        selected.paragraphs = vec![host];
        root.cells = vec![selected];
        root.row_count = 1;
        root.col_count = 1;
        root.row_sizes = vec![1];
        root.common.width = root.cells[0].width;
        root.common.height = 20000;
        root.common.horizontal_offset = 10000;
        root.common.vertical_offset = 10000;
        root.rebuild_grid();
    } else if args[3] != "whole-reflow" {
        return Err("unknown probe mode".into());
    }
    if args[3] == "whole-reflow" {
        for p in &mut d.sections[0].paragraphs {
            clear_cache(p);
        }
    }
    let output = Path::new(&args[2]);
    fs::create_dir_all(output)?;
    let bytes = rhwp::serializer::serialize_hwpx(&d)?;
    fs::write(output.join("compiled.hwpx"), &bytes)?;
    fs::write(
        output.join("scope.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input":args[1],"mode":args[3],"diagnosticOnly":true,
            "acceptance":"not a complete-page fixture or original-document acceptance"
        }))?,
    )?;
    println!("diagnostic HWPX written");
    Ok(())
}
