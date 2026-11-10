//! 관계형: 생성 번호의 문자군·원본 언어별 크기·장평을 실제 참조 노드에서 검사한다.
//! 주석 갱신: 2026-11-10T10:07:17+09:00.
//! report-628 95쪽 최소 글꼴 Native97.64481% / fresh WASM97.64831%.
//! 원본 글꼴 대조99.99111%/99.99110%와 혼용하지 않는다. 절대 px를 고정하지 않는다.
//! 독립 Print 참조56)의 내장 Gulim 세 글리프는 실제 설치 Gulim의 윤곽·전진과 같다.
//! source char shape75는 한글 휴먼명조·영문 HCI Poppy를 구별한다.
//! 설치 글꼴을 요구하지 않으며 source 슬롯·비율과 번호 소유의 관계만 검사한다.

use rhwp::{
    model::control::Control,
    renderer::{
        render_tree::{FootnoteMarkerNode, RenderNode, RenderNodeType},
        style_resolver::resolve_styles,
    },
    DocumentCore,
};
fn core() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("mydocs/pr/assets/planet6897_20261106/pr8642/report-628-original.hwp"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn marker(core: &DocumentCore) -> (FootnoteMarkerNode, f64) {
    fn find(node: &RenderNode, found: &mut Vec<(FootnoteMarkerNode, f64)>) {
        if let RenderNodeType::FootnoteMarker(m) = &node.node_type {
            if (m.section_index, m.para_index, m.control_index) == (3, 472, 0) {
                found.push((m.clone(), node.bbox.width));
            }
        }
        for child in &node.children {
            find(child, found);
        }
    }
    let tree = core.build_page_render_tree(94).unwrap();
    let mut found = vec![];
    find(&tree.root, &mut found);
    assert_eq!(found.len(), 1, "원본 참조의 누락·중복·쪽 소속");
    found.pop().unwrap()
}
#[test]
fn generated_decimal_reference_uses_its_latin_source_font_slot() {
    let core = core();
    let styles = resolve_styles(&core.document().doc_info, 96.0);
    let (m, _) = marker(&core);
    assert_eq!(m.text, "56)");
    assert_ne!(
        styles.char_styles[75].font_family_for_lang(0),
        styles.char_styles[75].font_family_for_lang(1)
    );
    assert_eq!(
        m.font_family,
        styles.char_styles[75].font_family_for_lang(1),
        "생성 숫자를 앞 한글 run의 face로 그리지 않는다"
    );
}
#[test]
fn generated_reference_consumes_its_language_relative_size_and_ratio() {
    let mut widths = vec![];
    for ratio in [80, 120] {
        let mut core = core();
        let mut doc = core.document().clone();
        let shape = &mut doc.doc_info.char_shapes[75];
        shape.relative_sizes[1] = 110;
        shape.ratios[1] = ratio;
        let source_base = f64::from(shape.base_size) / 75.0;
        core.set_document(doc);
        let (m, width) = marker(&core);
        assert!(
            (m.base_font_size / source_base - 1.1).abs() < 0.000001,
            "원본 영문 슬롯의 상대 크기를 보존한다"
        );
        widths.push(width);
    }
    assert!(
        widths[0] < widths[1],
        "같은 번호의 원본 영문 장평을 공통 폭에 반영한다: {widths:?}"
    );
}
#[test]
fn user_hangul_reference_keeps_its_own_script_instead_of_forcing_latin() {
    let mut core = core();
    let mut doc = core.document().clone();
    let Control::Footnote(note) = &mut doc.sections[3].paragraphs[472].controls[0] else {
        panic!("원본 각주")
    };
    note.number = 1;
    let auto = note.paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|c| match c {
            Control::AutoNumber(a) => Some(a),
            _ => None,
        })
        .expect("원본 번호 서식");
    auto.format = 0x81;
    auto.user_symbol = '가';
    auto.prefix_char = '\0';
    auto.suffix_char = ')';
    core.set_document(doc);
    let styles = resolve_styles(&core.document().doc_info, 96.0);
    let (m, _) = marker(&core);
    assert_eq!(m.text, "가)");
    assert_eq!(
        m.font_family,
        styles.char_styles[75].font_family_for_lang(0),
        "사용자 지정 한글 번호는 한글 슬롯을 유지한다"
    );
}
