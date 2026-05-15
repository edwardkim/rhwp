/// [Task #902 v2 Stage 16] RasterPlayer 시각 검증 도구.
///
/// Usage:
///   cargo run --release --example wmf_raster_test -- <input.wmf> <out.png> [width] [height]

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <input.wmf> <out.png> [width] [height]", args[0]);
        std::process::exit(1);
    }
    let input = &args[1];
    let output = &args[2];
    let width: f32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(800.0);
    let height: f32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(600.0);

    let wmf_data = fs::read(input).expect("read input wmf");
    eprintln!("Input WMF: {} bytes", wmf_data.len());

    let png = rhwp::renderer::svg::rasterize_wmf_direct_pub(&wmf_data, width, height)
        .expect("RasterPlayer 렌더링 실패");

    fs::write(output, &png).expect("write output png");
    eprintln!("Output PNG: {} bytes → {}", png.len(), output);
}
