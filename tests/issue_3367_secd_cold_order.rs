//! [Issue #3367/#4433] export-hwpx 가 구역 시작 문단의 secd/cold 컨트롤 순서를
//! 뒤집던 계약을 고정한다.
//!
//! HWP5 원본(field-01)은 전 구역이 `[cold, secd]` 순서인데, HWPX writer 의
//! 템플릿 고정 순서(secPr → ctrl/colPr)로 내보내면 재파싱 IR 이 `[secd, cold]`
//! 로 뒤집혀 ir-diff(--verify)가 컨트롤 type 차이 6건을 냈다. OWPML 스키마는
//! run 자식 순서를 규정하지 않고(choice), 한컴 원산 실물도 두 순서를 모두
//! 쓰므로(20 vs 315건 실측) **문서 순서가 보존 대상**이다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parse_document;

const SAMPLE: &str = "samples/field-01.hwp";

fn read_sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// 구역 첫 문단에서 (ColumnDef 위치, SectionDef 위치)를 얻는다.
fn secd_cold_positions(para: &rhwp::model::paragraph::Paragraph) -> (Option<usize>, Option<usize>) {
    let cold = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::ColumnDef(_)));
    let secd = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::SectionDef(_)));
    (cold, secd)
}

#[test]
fn issue3367_export_hwpx_preserves_cold_before_secd_order() {
    let bytes = read_sample();
    let original = parse_document(&bytes).expect("parse");

    // 샘플 전제: 원본은 cold 가 secd 앞이다 (#3367 실측 — 3개 구역 전부).
    let mut checked = 0usize;
    for sec in &original.sections {
        let (cold, secd) = secd_cold_positions(&sec.paragraphs[0]);
        if let (Some(c), Some(s)) = (cold, secd) {
            assert!(c < s, "샘플 전제 위반: 원본이 cold→secd 순서가 아니다");
            checked += 1;
        }
    }
    assert!(
        checked >= 2,
        "샘플 전제 위반: cold+secd 구역이 2개 이상이어야 한다"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export-hwpx");
    let roundtripped = parse_document(&hwpx).expect("reparse hwpx");

    assert_eq!(roundtripped.sections.len(), original.sections.len());
    for (si, (a, b)) in original
        .sections
        .iter()
        .zip(roundtripped.sections.iter())
        .enumerate()
    {
        let (a_cold, a_secd) = secd_cold_positions(&a.paragraphs[0]);
        let (b_cold, b_secd) = secd_cold_positions(&b.paragraphs[0]);
        if let (Some(ac), Some(asd)) = (a_cold, a_secd) {
            let (bc, bsd) = (b_cold.expect("cold 보존"), b_secd.expect("secd 보존"));
            assert_eq!(
                (ac < asd),
                (bc < bsd),
                "구역 {si}: secd/cold 상대 순서가 왕복에서 뒤집혔다 (#3367) — \
                 원본 cold@{ac}/secd@{asd}, 왕복 cold@{bc}/secd@{bsd}"
            );
        }
    }
}
