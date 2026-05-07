// Task #672 진단 도구: 모든 fixture 의 TAC 표 비례 축소 발동 영역 sweep
use std::fs::File;
use std::io::Read;

use rhwp::model::control::Control;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::style_resolver::resolve_styles;

fn main() {
    let mut paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        for entry in std::fs::read_dir("samples").unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension() {
                    if ext == "hwp" || ext == "hwpx" {
                        paths.push(p.to_string_lossy().into_owned());
                    }
                }
            }
        }
        if let Ok(rd) = std::fs::read_dir("samples/hwpx") {
            for entry in rd {
                let entry = entry.unwrap();
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension() {
                        if ext == "hwpx" {
                            paths.push(p.to_string_lossy().into_owned());
                        }
                    }
                }
            }
        }
    }
    paths.sort();

    let dpi: f64 = 96.0;
    println!("# Sweep target: {} fixtures", paths.len());
    let measurer = HeightMeasurer::new(dpi);

    let mut total_tac = 0;
    let mut total_shrink = 0;
    let mut diffs: Vec<(String, f64)> = Vec::new();

    for path in &paths {
        let mut buf = Vec::new();
        if File::open(path).and_then(|mut f| f.read_to_end(&mut buf)).is_err() {
            continue;
        }
        let doc = match rhwp::parser::parse_document(&buf) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let styles = resolve_styles(&doc.doc_info, dpi);

        for (s_idx, sec) in doc.sections.iter().enumerate() {
            // composed paragraphs
            let composed: Vec<_> = sec.paragraphs.iter()
                .map(compose_paragraph)
                .collect();
            let measured = measurer.measure_section(&sec.paragraphs, &composed, &styles);

            for mt in &measured.tables {
                let pi = mt.para_index;
                let ci = mt.control_index as usize;
                let sec_local = doc.sections.get(s_idx);
                let para = sec_local.and_then(|s| s.paragraphs.get(pi));
                let ctrl = para.and_then(|p| p.controls.get(ci));
                if let Some(Control::Table(tbl)) = ctrl {
                    if !tbl.common.treat_as_char { continue; }
                    total_tac += 1;
                    let common_h_px = tbl.common.height as f64 * dpi / 7200.0;
                    if common_h_px <= 0.0 { continue; }
                    let cell_spacing = tbl.cell_spacing as f64 * dpi / 7200.0;
                    let row_count = mt.row_heights.len();
                    let raw_h: f64 = mt.row_heights.iter().sum::<f64>()
                        + cell_spacing * (row_count.saturating_sub(1) as f64);
                    let diff = raw_h - common_h_px;
                    let ratio = diff / common_h_px * 100.0;
                    let scaled = raw_h > common_h_px + 1.0;
                    if scaled {
                        total_shrink += 1;
                        diffs.push((path.clone(), ratio));
                        println!("{} sec={} pi={} ci={} | rows={}x{} | common={:.2} raw={:.2} diff={:+.2} ({:+.2}%)",
                            path, s_idx, pi, ci,
                            tbl.row_count, tbl.col_count,
                            common_h_px, raw_h, diff, ratio);
                    }
                }
            }
        }
    }
    println!("\n# Total TAC tables: {}, Shrink發動: {}", total_tac, total_shrink);
    if !diffs.is_empty() {
        diffs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        println!("# Min diff%: {:+.4}", diffs.first().unwrap().1);
        println!("# Max diff%: {:+.4}", diffs.last().unwrap().1);
        println!("# Distribution buckets:");
        let buckets = [0.0, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, f64::INFINITY];
        for w in buckets.windows(2) {
            let count = diffs.iter().filter(|(_, r)| *r > w[0] && *r <= w[1]).count();
            println!("  {:>6.2}% ~ {:>6.2}%: {} tables", w[0], w[1], count);
        }
    }
}
