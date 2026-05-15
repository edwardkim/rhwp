/// [Task #902 v2 Stage 24] WMF → SVG 직접 변환 도구 (raster 우회).
/// WASM 환경의 SVG 임베드 출력 검증용.
///
/// Usage: cargo run --release --example wmf_to_svg -- <input.wmf> <out.svg>

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <input.wmf> <out.svg>", args[0]);
        std::process::exit(1);
    }
    let input = &args[1];
    let output = &args[2];

    let wmf_data = fs::read(input).expect("read input wmf");
    eprintln!("Input WMF: {} bytes", wmf_data.len());

    use rhwp::wmf::converter::{SVGPlayer, WMFConverter};
    let player = SVGPlayer::new();
    let converter = WMFConverter::new(wmf_data.as_slice(), player);
    let svg_bytes = converter.run().expect("WMF → SVG 변환 실패");

    fs::write(output, &svg_bytes).expect("write output svg");
    eprintln!("Output SVG: {} bytes → {}", svg_bytes.len(), output);
}
