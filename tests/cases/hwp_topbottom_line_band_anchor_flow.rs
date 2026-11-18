//! 고정형(독립 Print 좌표·쪽수)·관계형(라벨 소속·순서·전후 간격).
//! 주석 갱신: 2026-11-17 19:34:36 KST. 직전 파일 변경: 2026-11-17T18:30:08+09:00, 73ec4517c42d983d8041c5e1c9a6cdbac1d41eb7.
//! 측정: 2026-11-17, source 2c5e3fe387, canonical 2px raw / 96DPI / compat2024.
//! 원본 1쪽 Native 100.00% / fresh WASM 100.00%, 두 전쪽 최저값도 각각 100.00%.
//! 독립 데스크톱 한컴 2024 Hancom PDF Print: pdf/pr-review-open-prs-20261117/hwp_band-2024.pdf.
//! 근거: mydocs/pr/assets/open_prs_20261117/cases/band/{native,wasm}/candidate/silhouette.tsv 및 review·overlay.
//! 양쪽 review·overlay를 직접 판독한 뒤 고정 좌표를 추가했다. 웹한글기안기 PDF는 판정에 사용하지 않는다.
//! 원본: samples/issue8582/hwp_band.hwpx. HWP5 변환 대조는 같은 속성의 NoLS 라벨 간격을 검사한다.
//! 변환 HWP5의 별도 픽셀 일치율로 원본 HWPX 수치를 재사용하지 않는다.
//! 새 원본 HWPX 검사는 독립 PDF 좌표 ±2px와 라벨 누락·중복·문단 순서를 함께 보호한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

fn hwp5_without_stored_lines(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("재현물");
    let mut doc = rhwp::parser::parse_document(&bytes).expect("HWPX 읽기");
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            para.line_segs.clear();
        }
    }
    rhwp::serializer::serialize_document(&doc).expect("HWP5 저장")
}

fn label_tops(node: &RenderNode, out: &mut Vec<(String, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let text = run.text.trim();
        if text.starts_with("BAND") || text.starts_with("AFTER") {
            out.push((text.to_string(), node.bbox.y));
        }
    }
    for child in &node.children {
        label_tops(child, out);
    }
}

#[test]
fn hwp_target_topbottom_line_keeps_anchor_line_and_pushes_only_overlapping_lines() {
    let bytes = hwp5_without_stored_lines("hwp_band");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 1, "한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tops = Vec::new();
    label_tops(&tree.root, &mut tops);
    let top = |label: &str| {
        tops.iter()
            .find(|(text, _)| text == label)
            .map(|(_, y)| *y)
            .unwrap_or_else(|| panic!("{label} 줄 {tops:?}"))
    };
    let spans: Vec<(i32, f64, f64)> = [(2374, 5789.6727), (600, 5490.0), (0, 5490.0)]
        .into_iter()
        .map(|(offset, expected)| {
            let span = (top(&format!("AFTER{offset}")) - top(&format!("BAND{offset}"))) * HU_PER_PX;
            (offset, span, expected)
        })
        .collect();
    for (offset, span, expected) in &spans {
        assert!(
            (span - expected).abs() <= 60.0,
            "오프셋 {offset}: BAND→AFTER {span:.0} HU, 독립 데스크톱 Print의 라벨 간격 {expected:.2} HU ({spans:?})"
        );
    }
}

/// 독립 데스크톱 Print의 원본 HWPX 좌표를 검사한다. 변환 HWP5만 검사하던 사각지대를 보호한다.
#[test]
fn raw_hwpx_band_labels_keep_independent_print_flow_positions() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue8582/hwp_band.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("HWPX 원본")).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print와 같은 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let mut tops = Vec::new();
    label_tops(&tree.root, &mut tops);
    let expected = [
        ("BAND2374", 115.270825),
        ("AFTER2374", 173.167552),
        ("BAND600", 184.195500),
        ("AFTER600", 239.095502),
        ("BAND0", 250.123450),
        ("AFTER0", 305.023452),
    ];
    assert_eq!(tops.len(), expected.len(), "라벨 누락·중복 없음: {tops:?}");
    let mut previous_y = None;
    for (label, pdf_y_pt) in expected {
        let matches: Vec<_> = tops.iter().filter(|(text, _)| text == label).collect();
        assert_eq!(matches.len(), 1, "{label} 소속은 한 곳: {tops:?}");
        let y = matches[0].1;
        assert!(
            previous_y.is_none_or(|prev| y > prev),
            "문단 순서 보존: {tops:?}"
        );
        previous_y = Some(y);
        let expected_px = pdf_y_pt * 96.0 / 72.0;
        assert!(
            (y - expected_px).abs() <= 2.0,
            "{label}: {y:.4}px, 독립 한컴 Print {expected_px:.4}px (96DPI, ±2px)",
        );
    }
}
