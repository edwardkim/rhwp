//! 관계형: 확인된 HFT 선언과 실제 CFF 공급자가 굵기 전진을 소유하고 가운데 정렬을 보존한다.
//! 주석 갱신: 2026-11-12T01:03:06+09:00. 신규 검사.
//! 측정: 2026-11-12, 96 DPI, PDFium 독립 한컴 2024 Print, raw 2px 실루엣.
//! strawberry-tracking.hwpx 전체 1쪽: Native/fresh WASM 모두 89.84% → 100.00%, 전쪽 최저 100.00%.
//! 12/18/24pt 보통·굵게의 같은 영역을 확대 판독했다. 실제 페이지 px를 고정하지 않는다.
//! 정식 근거: mydocs/pr/assets/issue8957/tracking/ 및 pdf/issue8957/strawberry-tracking-2024.pdf.
//! 공개 검사에서는 원본 프로그램의 family와 실제 굵기 비트를 읽는다.
//! 명시적으로 공급한 원래 family의 확대·가운데 정렬은 통합 검증에서 원본으로 대조했다.

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

fn text_node<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == text) {
        return Some(node);
    }
    node.children.iter().find_map(|node| text_node(node, text))
}

fn render(
    source: &str,
    source_type: u8,
    family: &str,
    bytes: &[u8],
    text: &str,
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
    core.insert_text_native(0, 0, 0, text).unwrap();
    core.set_font_environment(Some(env)).unwrap();
    core.prepare_native_otf_fonts_with(|_| Some((bytes.to_vec(), 0)))
        .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    text_node(&tree.root, text)
        .expect("same paragraph retains the complete text")
        .clone()
}

#[test]
fn fallback_program_does_not_borrow_verified_tracking_in_centered_layout() {
    for (source, family) in [
        ("딸기", "Strawberry"),
        ("Strawberry", "Strawberry"),
        ("신명 중고딕", "Sinmyeong Junggothic"),
    ] {
        for size in [1200, 1800, 2400] {
            let regular = render(source, 2, family, CFF, "가 가", size, false);
            let bold = render(source, 2, family, CFF, "가 가", size, true);
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
            let program = b.style.native_otf_metrics.as_ref().unwrap();
            assert!(program.needs_faux_bold('가'));
            assert!(!program.has_verified_hft_bold_tracking('가'));
            let mut without_verification = b.style.clone();
            without_verification.source_verified_hft_bold_tracking = false;
            assert_eq!(
                EmbeddedTextMeasurer.compute_char_positions(&b.text, &b.style),
                EmbeddedTextMeasurer.compute_char_positions(&b.text, &without_verification),
                "{source} {size}"
            );
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
        ("Unverified declaration", 2, "Strawberry", false),
        ("딸기", 2, "Unverified CFF", false),
        ("딸기", 2, "Strawberry", true),
    ] {
        let (bytes, text) = if real_bold {
            let program =
                include_bytes!("../fixtures/fonts/source-sans-3/SourceSans3-3.052-Bold.otf");
            assert!(ttf_parser::Face::parse(program, 0).unwrap().is_bold());
            (program.as_slice(), "A A")
        } else {
            (CFF, "가 가")
        };
        let regular = render(source, source_type, family, bytes, text, 1800, false);
        let bold = render(source, source_type, family, bytes, text, 1800, true);
        if real_bold {
            let RenderNodeType::TextRun(run) = &bold.node_type else {
                unreachable!()
            };
            let program = run.style.native_otf_metrics.as_ref().unwrap();
            assert!(program.has_glyph('A'));
            assert!(!program.needs_faux_bold('A'));
        }
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
