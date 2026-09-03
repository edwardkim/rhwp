//! HWPX 라운드트립 — 입력 파일을 파싱 → 재직렬화 → 출력.
//!
//! 사용:
//!   cargo run --example hwpx_roundtrip --release -- <입력.hwpx> <출력.hwpx>
//!
//! 현재 범위 (Stage 2.3): 텍스트·문단·탭·소프트 브레이크만 보존.
//! 표/이미지/스타일은 Stage 2.5+ 이후 지원.

use std::env;
use std::fs;
use std::path::PathBuf;

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::serialize_hwpx;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: hwpx_roundtrip <input.hwpx> <output.hwpx>");
        std::process::exit(2);
    }
    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let bytes = fs::read(&input)?;
    println!("Read {} ({} bytes)", input.display(), bytes.len());

    let doc = parse_hwpx(&bytes)?;
    println!(
        "Parsed: sections={}, paragraphs={}",
        doc.sections.len(),
        doc.sections
            .iter()
            .map(|s| s.paragraphs.len())
            .sum::<usize>()
    );

    let out_bytes = serialize_hwpx(&doc)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, &out_bytes)?;
    println!("Wrote {} ({} bytes)", output.display(), out_bytes.len());
    Ok(())
}
