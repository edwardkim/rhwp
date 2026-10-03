//! HwpSummaryInformation 의 모든 UTF-16LE string 추출 — 한컴 오피스 version 식별 가능 여부 확인

use cfb::CompoundFile;
use std::io::Cursor;

fn extract_summary_strings(path: &str) -> Vec<String> {
    let bytes = std::fs::read(path).unwrap();
    let mut cfb = CompoundFile::open(Cursor::new(&bytes)).unwrap();
    let mut data = Vec::new();
    for name in &[
        "/\u{0005}HwpSummaryInformation",
        "\u{0005}HwpSummaryInformation",
    ] {
        if let Ok(mut stream) = cfb.open_stream(name) {
            use std::io::Read;
            stream.read_to_end(&mut data).ok();
            break;
        }
    }
    // UTF-16LE 4글자 이상 string 추출
    let utf16: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let mut strings = Vec::new();
    let mut cur = String::new();
    for &u in &utf16 {
        if u == 0 {
            if cur.chars().count() >= 4 {
                strings.push(cur.clone());
            }
            cur.clear();
        } else if let Some(c) = char::from_u32(u as u32) {
            if c.is_control() {
                if cur.chars().count() >= 4 {
                    strings.push(cur.clone());
                }
                cur.clear();
            } else {
                cur.push(c);
            }
        }
    }
    if cur.chars().count() >= 4 {
        strings.push(cur);
    }
    strings
}

#[test]
fn extract_sample16_summary_strings() {
    for f in [
        "samples/hwp3-sample16-hwp5.hwp",
        "samples/hwp3-sample16-hwp5-2010.hwp",
        "samples/hwp3-sample16-hwp5-2018.hwp",
        "samples/hwp3-sample16-hwp5-2022.hwp",
        "samples/hwp3-sample16-hwp5-2024.hwp",
    ]
    .iter()
    {
        let strs = extract_summary_strings(f);
        eprintln!("\n=== {} ===", f);
        for s in &strs {
            eprintln!("  '{}'", s);
        }
    }
}
