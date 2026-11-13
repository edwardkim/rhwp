//! 관계형: 확인된 HFT 선언과 실제 CFF 공급자가 굵기 전진을 소유하고 가운데 정렬을 보존한다.
//! 주석 갱신: 2026-11-12T01:03:06+09:00. 신규 검사.
//! 측정: 2026-11-12, 96 DPI, PDFium 독립 한컴 2024 Print, raw 2px 실루엣.
//! strawberry-tracking.hwpx 전체 1쪽: Native/fresh WASM 모두 89.84% → 100.00%, 전쪽 최저 100.00%.
//! 12/18/24pt 보통·굵게의 같은 영역을 확대 판독했다. 실제 페이지 px를 고정하지 않는다.
//! 정식 근거: mydocs/pr/assets/issue8957/tracking/ 및 pdf/issue8957/strawberry-tracking-2024.pdf.
//! 실행 글꼴은 MIT 합성 RHWPHftOutline.otf의 이름/스타일만 메모리에서 바꾼다.
//! 이 합성 계약은 제공 OTF의 윤곽 일치나 원본 표지 전쪽 통과를 대신하지 않는다.

use rhwp::{
    model::{provenance::SourceFormat, style::Alignment},
    renderer::{
        font_environment::FontEnvironment,
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
use sha2::{Digest, Sha256};

const CFF: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");

fn program(family: &str, real_bold: bool) -> Vec<u8> {
    let utf16 = |s: &str| {
        s.encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<_>>()
    };
    let from = utf16("RHWP HFT Outline Fixture");
    let to = utf16(&format!("{family:width$}", width = from.len() / 2));
    assert_eq!(from.len(), to.len());
    let mut bytes = CFF.to_vec();
    let mut replaced = 0;
    for offset in 0..=bytes.len() - from.len() {
        if bytes[offset..offset + from.len()] == from {
            bytes[offset..offset + to.len()].copy_from_slice(&to);
            replaced += 1;
        }
    }
    assert!(replaced > 0);
    if real_bold {
        let count = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
        let offset = (0..count)
            .find_map(|i| {
                let p = 12 + i * 16;
                (&bytes[p..p + 4] == b"OS/2")
                    .then(|| u32::from_be_bytes(bytes[p + 8..p + 12].try_into().unwrap()) as usize)
            })
            .unwrap();
        // OS/2 fsSelection: BOLD=5, REGULAR=6. The fixture owns no external outline data.
        let selection = u16::from_be_bytes(bytes[offset + 62..offset + 64].try_into().unwrap());
        bytes[offset + 62..offset + 64].copy_from_slice(&((selection | 32) & !64).to_be_bytes());
        assert!(ttf_parser::Face::parse(&bytes, 0).unwrap().is_bold());
    }
    bytes
}

fn text_node(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "가 가") {
        return Some(node);
    }
    node.children.iter().find_map(text_node)
}

fn render(
    source: &str,
    source_type: u8,
    family: &str,
    bytes: &[u8],
    size: i32,
    bold: bool,
) -> RenderNode {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    doc.provenance.format = SourceFormat::Hwpx;
    for face in doc.doc_info.font_faces.iter_mut().flatten() {
        face.name = source.into();
        face.alt_type = source_type;
        face.native_program = None;
    }
    for shape in &mut doc.doc_info.char_shapes {
        shape.font_ids = [0; 7];
        shape.bold = bold;
        shape.base_size = size;
    }
    for shape in &mut doc.doc_info.para_shapes {
        shape.alignment = Alignment::Center;
    }
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.update(0u32.to_be_bytes());
    let id: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let json = format!(
        r#"[{{"query":"{source}","family":"{family}","programId":"{id}","trueType":false,"openTypeCff":true}}]"#
    );
    let env = FontEnvironment::from_available_programs(&doc.doc_info, &json)
        .unwrap()
        .unwrap();
    core.set_document(doc);
    core.insert_text_native(0, 0, 0, "가 가").unwrap();
    core.set_font_environment(Some(env)).unwrap();
    core.prepare_native_otf_fonts_with(|_| Some((bytes.to_vec(), 0)))
        .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    text_node(&tree.root)
        .expect("same paragraph retains the complete text")
        .clone()
}

#[test]
fn verified_hft_tracking_reaches_centered_placement_without_widening_spaces() {
    for (source, family) in [
        ("딸기", "Strawberry"),
        ("Strawberry", "Strawberry"),
        ("신명 중고딕", "Sinmyeong Junggothic"),
    ] {
        let bytes = program(family, false);
        for size in [1200, 1800, 2400] {
            let regular = render(source, 2, family, &bytes, size, false);
            let bold = render(source, 2, family, &bytes, size, true);
            assert!(bold.bbox.width > regular.bbox.width, "{source} {size}");
            assert!(
                bold.bbox.x < regular.bbox.x,
                "centered run expands to the left"
            );
            assert!(
                (bold.bbox.x + bold.bbox.width / 2.0 - regular.bbox.x - regular.bbox.width / 2.0)
                    .abs()
                    < 1e-7
            );
            assert_eq!(bold.bbox.y, regular.bbox.y);
            let (RenderNodeType::TextRun(b), RenderNodeType::TextRun(r)) =
                (&bold.node_type, &regular.node_type)
            else {
                unreachable!()
            };
            assert_eq!(b.para_index, r.para_index);
            assert_eq!(b.text, r.text);
            assert!(b.style.source_verified_hft_bold_tracking);
            let positions = EmbeddedTextMeasurer.compute_char_positions(&b.text, &b.style);
            assert!(positions.windows(2).all(|p| p[1] > p[0]));
            assert!((positions.last().unwrap() - positions[0] - bold.bbox.width).abs() < 1e-7);
            assert_eq!(
                EmbeddedTextMeasurer.compute_char_positions(" ", &b.style),
                EmbeddedTextMeasurer.compute_char_positions(" ", &r.style)
            );
        }
    }
}

#[test]
fn unverified_source_program_and_real_bold_keep_their_own_advances() {
    for (source, source_type, family, real_bold) in [
        ("딸기", 1, "Strawberry", false),
        ("Unverified HFT", 2, "Strawberry", false),
        ("딸기", 2, "Unverified CFF", false),
        ("딸기", 2, "Strawberry", true),
    ] {
        let bytes = program(family, real_bold);
        let regular = render(source, source_type, family, &bytes, 1800, false);
        let bold = render(source, source_type, family, &bytes, 1800, true);
        assert_eq!(
            (
                regular.bbox.x,
                regular.bbox.y,
                regular.bbox.width,
                regular.bbox.height
            ),
            (bold.bbox.x, bold.bbox.y, bold.bbox.width, bold.bbox.height),
            "{source} type={source_type} program={family} real_bold={real_bold}"
        );
    }
}
