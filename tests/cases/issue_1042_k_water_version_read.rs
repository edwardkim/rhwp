#[test]
fn check_remaining_sample_versions() {
    for f in ["samples/k-water-rfp.hwp", "samples/k-water-rfp-2024.hwp"].iter() {
        let bytes = std::fs::read(f).unwrap();
        let doc = rhwp::parser::parse_hwp(&bytes).unwrap();
        let v = &doc.header.version;
        eprintln!(
            "{}: version={}.{}.{}.{} variant={}",
            f, v.major, v.minor, v.build, v.revision, doc.is_hwp3_variant
        );
    }
}
