#[test]
fn check_variant_flags() {
    let files = ["samples/k-water-rfp.hwp", "samples/k-water-rfp-2024.hwp"];
    for f in files.iter() {
        let bytes = std::fs::read(f).expect("read");
        let doc = rhwp::parser::parse_hwp(&bytes).expect("parse");
        eprintln!("{}: is_hwp3_variant={}", f, doc.is_hwp3_variant);
    }
}
