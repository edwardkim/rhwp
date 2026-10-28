//! [Issue #5184] HWP3 빈 셀 문단의 저장 `vertsize=1000` 은 HWPX 왕복에서
//! 표 높이로 바뀌면 안 된다.
//!
//! `samples/hwp3-empty-cell.hwp` 를 HWPX 로 저장해 다시 열면
//! `paragraph[5].linesegs[0].vertsize` 가 1000→23476,
//! `paragraph[7]` 이 1000→29096 이 됐다. TAC host LINE_SEG 확대는
//! 기본 lh=100 합성 seg 에만 적용한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::path::Path;

const SAMPLE: &str = "samples/hwp3-empty-cell.hwp";

#[test]
fn issue_5184_hwpx_roundtrip_keeps_hwp3_empty_cell_vertsize() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let orig = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let hwpx = orig.export_hwpx_native().expect("export hwpx");
    let back = DocumentCore::from_bytes(&hwpx).expect("reparse hwpx");

    let orig_sec = &orig.document().sections[0];
    let back_sec = &back.document().sections[0];
    for pi in [5usize, 7] {
        let expected = orig_sec.paragraphs[pi].line_segs[0].line_height;
        let actual = back_sec.paragraphs[pi].line_segs[0].line_height;
        assert_eq!(
            actual, expected,
            "paragraph[{pi}] linesegs[0].vertsize HWPX 왕복이 표 높이로 바뀌면 안 된다"
        );
    }
}
