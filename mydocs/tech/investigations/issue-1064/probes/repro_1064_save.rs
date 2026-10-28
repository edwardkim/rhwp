// Task #1064 — HWPX 표 CTRL_DATA 합성 reproduce.
//
// 1. Load samples/hwpx/el-school-001.hwpx
// 2. Export HWP via adapter
// 3. Section0 record level CTRL_DATA 개수 + payload 확인
fn main() {
    let src = "samples/hwpx/el-school-001.hwpx";
    let dst = "output/poc/issue_1064/repro_stage2.hwp";

    let bytes = std::fs::read(src).expect("read source");
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpx");

    std::fs::create_dir_all("output/poc/issue_1064").ok();
    let out_bytes = doc.export_hwp_with_adapter().expect("export");
    std::fs::write(dst, &out_bytes).expect("write output");
    println!("saved: {} ({} bytes)", dst, out_bytes.len());

    // section0 tag 87 count check
    use std::io::Read;
    let cursor = std::io::Cursor::new(&out_bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FH");
    let is_c = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);
    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read");
    let data = if is_c {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out).expect("inflate");
        out
    } else {
        raw
    };

    let mut pos = 0;
    let mut ctrl_data_count = 0;
    let mut ctrl_data_sizes = Vec::new();
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (hdr & 0x3FF) as u16;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        if tag == 87 {
            ctrl_data_count += 1;
            ctrl_data_sizes.push(size);
        }
        pos += size;
    }
    println!(
        "CTRL_DATA records: {} (sizes: {:?})",
        ctrl_data_count, ctrl_data_sizes
    );
    println!("정답지 기대: 2 records, size=104");
}
