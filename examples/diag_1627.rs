//! Issue #1627 진단 — HWPX roundtrip 첫 문단 char_shapes 오프셋 shift 원인 분석.
//! 사용: cargo run --release --example diag_1627 -- <file.hwpx> [para_idx]

use rhwp::model::control::Control;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;
use std::env;
use std::fs;

fn dump_para(tag: &str, doc: &rhwp::model::document::Document, pi: usize) {
    let p = &doc.sections[0].paragraphs[pi];
    let nchars = p.text.chars().count();
    let cs: Vec<(u32, u32)> = p
        .char_shapes
        .iter()
        .map(|c| (c.start_pos, c.char_shape_id))
        .collect();
    let ctrls: Vec<String> = p
        .controls
        .iter()
        .map(|c| {
            let s = format!("{c:?}");
            s.chars().take(22).collect::<String>()
        })
        .collect();
    println!("[{tag}] p[{pi}] text_chars={nchars} controls={ctrls:?}");
    println!("       char_shapes={cs:?}");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let file = &args[0];
    let pi: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let data = fs::read(file).expect("read");

    let doc1 = parse_hwpx(&data).expect("parse1");
    dump_para("parse ", &doc1, pi);

    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    dump_para("reparse", &doc2, pi);

    // 첫 문단 text 의 char 별 코드포인트(앞 40자) — 객체 치환문자 위치 확인
    let p1 = &doc1.sections[0].paragraphs[pi];
    let p2 = &doc2.sections[0].paragraphs[pi];
    let cps1: Vec<u32> = p1.text.chars().take(40).map(|c| c as u32).collect();
    let cps2: Vec<u32> = p2.text.chars().take(40).map(|c| c as u32).collect();
    println!("parse   text cp[..40]={cps1:?}");
    println!("reparse text cp[..40]={cps2:?}");
}
