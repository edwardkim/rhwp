// [Task #927] reflow_linesegs_on_demand 회귀 검증.
//
// 빈 lineseg / 비표준 lineseg 가 reflow 된 후 paginator 의 vpos_h 기반
// current_height 조정이 정상 동작하는지 확인. 회귀 시 페이지 수 폭증.
//
// 사용:
//   cargo run --release --example diag_927_reflow -- [path/to/file.hwp]

use rhwp::document_core::DocumentCore;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("samples/hwp3-sample16-hwp5.hwp"));
    let bytes = std::fs::read(&path).expect("read file");
    let mut core = DocumentCore::from_bytes(&bytes).expect("parse");

    let pages_before = core.page_count();
    let reflowed = core.reflow_linesegs_on_demand();
    let pages_after = core.page_count();

    println!("파일: {:?}", path);
    println!("페이지 (reflow 전): {}", pages_before);
    println!("reflow 된 문단 수: {}", reflowed);
    println!("페이지 (reflow 후): {}", pages_after);
    println!(
        "차이: {:+} 페이지",
        pages_after as i64 - pages_before as i64
    );
}
