use std::fs;
use rhwp::document_core::DocumentCore;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 { eprintln!("usage: {} <hwp> <bin_id> <out>", args[0]); std::process::exit(1); }
    let data = fs::read(&args[1]).expect("read");
    let bin_id: u16 = args[2].parse().unwrap();
    let outpath = &args[3];
    let core = DocumentCore::from_bytes(&data).expect("parse");
    let doc = core.document();
    for (i, bd) in doc.bin_data_content.iter().enumerate() {
        eprintln!("  bin[{}]: id={} ext={} len={}", i, bd.id, bd.extension, bd.data.len());
    }
    if let Some(bd) = doc.bin_data_content.iter().find(|b| b.id == bin_id) {
        fs::write(outpath, &bd.data).expect("write");
        println!("Wrote {} bytes to {} (ext={})", bd.data.len(), outpath, bd.extension);
    } else {
        eprintln!("bin_id {} not found", bin_id);
    }
}
