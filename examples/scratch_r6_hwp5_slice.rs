//! scratch: HWP5 구역/문단 창 절단 축소본 생성기 (RHWP_SLICE_SRC/KEEP/OUT/STUB).
//! KEEP="secIdx:paraStart..paraEnd[,secIdx:...]" — 지정 구역만 남기고 창 밖 문단을 버린다.
use std::collections::HashMap;

fn main() {
    let src = std::env::var("RHWP_SLICE_SRC").unwrap();
    let keep_spec = std::env::var("RHWP_SLICE_KEEP").unwrap();
    let out = std::env::var("RHWP_SLICE_OUT").unwrap();
    let mut keep: HashMap<usize, (usize, usize)> = HashMap::new();
    for part in keep_spec.split(',') {
        let (sec, range) = part.split_once(':').unwrap();
        let (a, b) = range.split_once("..").unwrap();
        keep.insert(
            sec.trim().parse().unwrap(),
            (a.trim().parse().unwrap(), b.trim().parse().unwrap()),
        );
    }
    let raw = std::fs::read(&src).unwrap();
    let mut doc = rhwp::parser::parse_document(&raw).unwrap();
    let mut kept_secs = Vec::new();
    for (i, mut sec) in doc.sections.drain(..).enumerate() {
        if let Some(&(a, b)) = keep.get(&i) {
            let n = sec.paragraphs.len();
            sec.paragraphs = sec
                .paragraphs
                .into_iter()
                .skip(a)
                .take(b.saturating_sub(a))
                .collect();
            println!("sec{i}: {n} -> {} paras", sec.paragraphs.len());
            kept_secs.push(sec);
        }
    }
    doc.sections = kept_secs;
    if std::env::var("RHWP_SLICE_STUB").is_ok() {
        const PNG1X1: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        for bd in doc.bin_data_content.iter_mut() {
            bd.data = rhwp::model::bin_data::BinDataBytes::Loaded(PNG1X1.to_vec());
            bd.extension = "png".to_string();
        }
        doc.preview = None;
    }
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(&doc).unwrap();
    std::fs::write(&out, &bytes).unwrap();
    println!("WROTE {out} ({} bytes)", bytes.len());
}
