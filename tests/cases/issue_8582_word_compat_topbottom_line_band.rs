//! #8582: 호환 대상이 MS 워드인 문서에서 저장 줄 없는 빈 문단에 붙은 문단 기준 자리차지 직선은
//! 그 띠에 걸리는 줄(앵커 문단의 줄 포함)을 띠 아래로 민다.
//!
//! 재현물 `samples/issue8582/{hwp,word}_band.hwpx` (`samples/test-image.hwpx` 조합): `BAND<o>` 글 문단,
//! 자리차지 직선(높이 120 HU, 문단 기준 세로 오프셋 o = 2374·600·0 HU)만 가진 빈 문단, 빈 문단 3개,
//! `AFTER<o>` 글 문단. 실문서(HWP5, 저장 줄 없음)와 같은 경로를 지나도록 시험에서 저장 줄을 지우고
//! HWP5 로 다시 저장해 잰다.
//!
//! 기준: 데스크톱 한컴 2024 인쇄 PDF(Hancom PDF 드라이버)의 글자 기준선(같은 HWPX 판. 레이아웃
//! 호환 플래그가 켜진 HWP5 는 HWPX 와 같은 줄 높이로 인쇄된다).
//! 줄 피치 1860 HU 에서 `BAND` → `AFTER` 거리(HU): o=2374 9936, o=600 10020, o=0 9420.
//! 밀림 없이 다섯 줄(앵커 줄 포함)이면 9300 HU 이므로 더해진 양은 각각 636·720·120 HU 다:
//! - o=2374: 앵커 줄(0~1860) 아래 첫 빈 줄(1860~3720)이 띠(2374~2494)에 걸려 띠 아래로 간다.
//! - o=600·0: 앵커 줄 자신이 띠에 걸려 띠 아래(720·120)에서 시작한다.
//!
//! 한/글 문서(`hwp_band`)는 앵커 줄 상자 안의 띠를 줄이 흡수하는 다른 규칙이며 이 시험의 범위 밖이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    parser::tags,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

/// 호환 대상이 MS 워드인 실문서 HWP5 의 레이아웃 호환 플래그(사내 302건 중 279건의 값).
const WORD_LAYOUT_FLAGS: [u8; 20] = [
    0xe3, 0x01, 0, 0, 0xff, 0xf8, 0xcf, 0, 0x4f, 0, 0, 0, 0xec, 0x01, 0, 0, 0, 0, 0, 0,
];

fn hwp5_without_stored_lines(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("#8582 재현물");
    let mut doc = rhwp::parser::parse_document(&bytes).expect("HWPX 읽기");
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            para.line_segs.clear();
        }
    }
    // 실문서 HWP5 처럼 호환 대상(MS 워드)과 레이아웃 호환 플래그를 제자리에 둔다(플래그가 0 이면
    // 데스크톱 한컴도 워드 줄 높이를 쓰지 않는다).
    for record in &mut doc.doc_info.extra_records {
        if record.tag_id == tags::HWPTAG_COMPATIBLE_DOCUMENT {
            record.data = 2u32.to_le_bytes().to_vec();
        }
        if record.tag_id == tags::HWPTAG_LAYOUT_COMPATIBILITY {
            record.data = WORD_LAYOUT_FLAGS.to_vec();
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
fn word_target_topbottom_line_pushes_overlapping_lines_below_band() {
    let bytes = hwp5_without_stored_lines("word_band");
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
    let spans: Vec<(i32, f64, f64)> = [(2374, 9936.0), (600, 10020.0), (0, 9420.0)]
        .into_iter()
        .map(|(offset, expected)| {
            let span = (top(&format!("AFTER{offset}")) - top(&format!("BAND{offset}"))) * HU_PER_PX;
            (offset, span, expected)
        })
        .collect();
    for (offset, span, expected) in &spans {
        assert!(
            (span - expected).abs() <= 60.0,
            "오프셋 {offset}: BAND→AFTER {span:.0} HU, 데스크톱 한컴 {expected:.0} HU ({spans:?})"
        );
    }
}
