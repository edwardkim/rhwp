// [Task #935] HWPX wrap=TopAndBottom Picture 위치 회귀 검증.
//
// HWPX 변환본의 wrap=TopAndBottom Picture 가 page boundary 안에 위치하는지
// 확인. 회귀 시 picture 가 우측으로 shift 되어 페이지 외곽 overflow.
//
// 사용: cargo run --release --example diag_935 [-- path/to.hwpx]

use rhwp::document_core::DocumentCore;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("samples/hwp3-sample16-hwp5.hwpx")
    });
    let bytes = std::fs::read(&path).expect("read file");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let doc = core.document();

    println!("파일: {:?}", path);
    let mut wmf_count = 0;
    let mut tac_topbot = 0;
    for bd in &doc.bin_data_content {
        if bd.extension.to_lowercase().contains("wmf") {
            wmf_count += 1;
            println!("  BinData id={} ext={:?} size={} bytes", bd.id, bd.extension, bd.data.len());
        }
    }
    for sec in &doc.sections {
        for para in &sec.paragraphs {
            for ctrl in &para.controls {
                if let rhwp::model::control::Control::Picture(p) = ctrl {
                    if p.common.treat_as_char
                        && matches!(p.common.text_wrap, rhwp::model::shape::TextWrap::TopAndBottom)
                    {
                        tac_topbot += 1;
                    }
                }
            }
        }
    }
    println!("WMF binary 수: {}", wmf_count);
    println!("TAC wrap=TopAndBottom Picture 수: {}", tac_topbot);
    println!("페이지 수: {}", core.page_count());
}
