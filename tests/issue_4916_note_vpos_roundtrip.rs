//! [Issue #4916/#4660/#3531 계열] HWP5→HWPX 왕복에서 각주·미주 subList lineseg
//! 의 저장 vpos=0 이 재파싱 보정(task 1692 `normalize_hwpx_note_line_vpos`)으로
//! 합성값으로 바뀌어 `--verify` 가 vertpos 차이를 내던 계약을 고정한다.
//!
//! 수정: rhwp 자기 산출 HWPX(HWP5-origin 마커, #1770)는 그 보정을 건너뛴다 —
//! HWP5 원본 저장값의 왕복 보존이 마커 계약이다. 실물 한컴 HWPX 의 보정은
//! 종전 유지(tests/issue_1692.rs 가 그 축을 잡는다).

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::parse_document;

const SAMPLE: &str = "samples/3-09월_교육_통합_2022.hwp";

fn note_vpos_signature(paragraphs: &[Paragraph], out: &mut Vec<Vec<i32>>) {
    for p in paragraphs {
        for c in &p.controls {
            match c {
                Control::Footnote(n) => {
                    for np in &n.paragraphs {
                        out.push(np.line_segs.iter().map(|s| s.vertical_pos).collect());
                    }
                    note_vpos_signature(&n.paragraphs, out);
                }
                Control::Endnote(n) => {
                    for np in &n.paragraphs {
                        out.push(np.line_segs.iter().map(|s| s.vertical_pos).collect());
                    }
                    note_vpos_signature(&n.paragraphs, out);
                }
                Control::Table(t) => {
                    for cell in &t.cells {
                        note_vpos_signature(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn issue4916_note_sublist_vpos_survives_hwpx_roundtrip() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let original = parse_document(&bytes).expect("parse hwp");

    let mut orig_sig = Vec::new();
    for sec in &original.sections {
        note_vpos_signature(&sec.paragraphs, &mut orig_sig);
    }
    assert!(
        orig_sig.iter().any(|v| v.len() > 1 && v[1..].contains(&0)),
        "샘플 전제: 후속 줄 vpos=0 인 노트 subList lineseg 가 있어야 한다"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export hwpx");
    let roundtripped = parse_document(&hwpx).expect("reparse hwpx");

    let mut rt_sig = Vec::new();
    for sec in &roundtripped.sections {
        note_vpos_signature(&sec.paragraphs, &mut rt_sig);
    }
    assert_eq!(
        orig_sig, rt_sig,
        "각주·미주 subList lineseg vpos 가 왕복에서 그대로여야 한다 (#4916 계열)"
    );
}
