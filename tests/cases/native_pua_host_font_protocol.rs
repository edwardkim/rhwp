//! 실제 글꼴 요청 이름과 명시 exact 소유자의 세션 프로토콜 계약.
//! 실제 페이지 좌표나 한컴 피델리티의 기대값을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

#[test]
fn pua_supply_clear_preserves_explicit_exact_font_owner() {
    let mut core = DocumentCore::new_empty();
    let bytes = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let before: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    let document_before = format!("{:?}", core.document());
    let saved_before = core.export_hwp_native().unwrap();
    let literal_space_program = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    assert!(core
        .register_literal_figure_space_font(
            "RHWP Equation Latin Only",
            false,
            false,
            literal_space_program,
            0,
        )
        .unwrap());
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "리터럴 공백의 세션 프로그램 등록은 문서 IR을 바꾸지 않는다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        saved_before,
        "실제 숫자 glyph를 공급해도 원본 HWP 저장 바이트를 바꾸지 않는다"
    );
    core.clear_native_pua_fonts();
    let after: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        after["status"], "already-registered",
        "PUA만 해제해도 명시 exact 소유자는 동일한 채 유지한다"
    );
    assert_eq!(before["handle"], after["handle"]);
    assert_eq!(before["registry"], after["registry"]);
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "세션 해제는 문서 IR과 원본 raw 상태를 바꾸지 않는다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        saved_before,
        "원 PUA 공급 상태는 저장 HWP 바이트에 반영되지 않는다"
    );
}

#[test]
fn host_font_request_names_are_not_css_quoted() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
        .expect("구역과 첫 문단이 있는 실제 빈 문서");
    core.insert_text_native(0, 0, 0, "\u{f0854}").unwrap();
    let requests = core.native_pua_font_requests();
    assert!(!requests.is_empty(), "원 PUA의 실제 글꼴 요청");
    for request in requests {
        assert_eq!(request.paint_families.first(), Some(&request.family));
        assert!(request
            .paint_families
            .iter()
            .all(|family| !family.starts_with('\'')));
    }
}

#[test]
fn source_pointer_requests_its_program_without_reclassifying_unicode_pointer() {
    for (text, requires_source_program) in [
        ("\u{f02fc}", true),
        ("►", false),
        ("\u{f031c}", true),
        ("■", false),
        ("\u{f03a7}", true),
        ("⊟", false),
        ("\u{f03a8}", true),
        ("⊞", false),
        ("\u{f0848}", true),
        ("━", false),
    ] {
        let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
            .expect("실제 빈 문서");
        core.insert_text_native(0, 0, 0, text).unwrap();
        let before = format!("{:?}", core.document());
        let requests = core.native_pua_font_requests();
        assert_eq!(
            !requests.is_empty(),
            requires_source_program,
            "원 PUA와 일반 Unicode 기호의 공급 요청 계약: {text:?}"
        );
        assert_eq!(
            format!("{:?}", core.document()),
            before,
            "글꼴 공급 요청은 원문과 문서 속성을 변경하지 않는다"
        );
    }
}

/// 저장소가 직접 만든 MIT 합성 윤곽의 cmap만 메모리에서 바꾼다.
/// 외부 글꼴·새 배포 파일 없이 원 glyph가 실제로 있는 공급 프로그램을 만든다.
fn synthetic_symbol_program(symbol: char) -> Vec<u8> {
    let source = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let face = ttf_parser::Face::parse(source, 0).unwrap();
    let glyph = face.glyph_index('A').unwrap();
    assert_eq!(face.units_per_em(), 1000);
    assert_eq!(face.glyph_hor_advance(glyph), Some(900));
    let mut bytes = source.to_vec();
    let table_count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let record = |tag: &[u8; 4]| {
        (0..table_count)
            .map(|index| 12 + index * 16)
            .find(|&offset| &source[offset..offset + 4] == tag)
            .unwrap()
    };
    let cmap_record = record(b"cmap");
    let head_record = record(b"head");
    let head_offset = u32::from_be_bytes(
        source[head_record + 8..head_record + 12]
            .try_into()
            .unwrap(),
    ) as usize;
    let mut cmap = Vec::new();
    for value in [0u16, 1, 3, 10] {
        cmap.extend(value.to_be_bytes());
    }
    cmap.extend(12u32.to_be_bytes());
    cmap.extend(12u16.to_be_bytes());
    cmap.extend(0u16.to_be_bytes());
    for value in [
        28u32,
        0,
        1,
        symbol as u32,
        symbol as u32,
        u32::from(glyph.0),
    ] {
        cmap.extend(value.to_be_bytes());
    }
    let checksum = |data: &[u8]| {
        data.chunks(4).fold(0u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    let offset = bytes.len().next_multiple_of(4);
    bytes.resize(offset, 0);
    bytes.extend(&cmap);
    bytes[cmap_record + 4..cmap_record + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    bytes[cmap_record + 8..cmap_record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
    bytes[cmap_record + 12..cmap_record + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
    bytes[head_offset + 8..head_offset + 12].fill(0);
    let adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum(&bytes));
    bytes[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xB1B0AFBA);
    let mapped = ttf_parser::Face::parse(&bytes, 0).unwrap();
    assert_eq!(mapped.glyph_index(symbol), Some(glyph));
    assert!(mapped.glyph_index('►').is_none());
    bytes
}

fn assert_prepared_source_symbol(symbol: char, fallback: &str) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn displayed_pointer(node: &RenderNode, symbol: char) -> Option<&str> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(symbol) {
                return Some(run.display_or_text());
            }
        }
        node.children
            .iter()
            .find_map(|child| displayed_pointer(child, symbol))
    }

    fn source_advance(node: &RenderNode, symbol: char) -> Option<(f64, f64)> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(symbol) {
                assert_eq!(run.style.ratio, 1.0);
                assert_eq!(run.style.letter_spacing, 0.0);
                assert!(run.style.font_size > 0.0);
                assert!(!run.style.superscript && !run.style.subscript);
                return Some((node.bbox.width, run.style.font_size));
            }
        }
        node.children
            .iter()
            .find_map(|child| source_advance(child, symbol))
    }

    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    core.insert_text_native(0, 0, 0, &symbol.to_string())
        .unwrap();
    let mut document = core.document().clone();
    for language in &mut document.doc_info.font_faces {
        for font in language {
            font.name = "RHWP Host Fixture".into();
        }
    }
    core.set_document(document);
    let original = format!("{:?}", core.document());
    let before = core.build_page_render_tree(0).unwrap();
    assert_eq!(displayed_pointer(&before.root, symbol), Some(fallback));
    let program = synthetic_symbol_program(symbol);
    assert!(core
        .prepare_native_pua_fonts_with(|request| {
            assert!(request.family.starts_with("RHWP Host Fixture"));
            Some((program.clone(), 0))
        })
        .unwrap());
    let prepared = core.build_page_render_tree(0).unwrap();
    assert_eq!(
        displayed_pointer(&prepared.root, symbol),
        Some(symbol.to_string().as_str()),
        "실제 원 glyph를 선택한 뒤 이전 Unicode 표시를 남기지 않는다"
    );
    let (advance, size) = source_advance(&prepared.root, symbol).unwrap();
    assert!(
        (advance - size * 0.9).abs() <= 4.0 / 75.0,
        "합성 원본 SFNT의 독립 hmtx 900/1000 전진을 배치도 소비한다"
    );
    assert_eq!(format!("{:?}", core.document()), original);
    core.clear_native_pua_fonts();
    let cleared = core.build_page_render_tree(0).unwrap();
    assert_eq!(displayed_pointer(&cleared.root, symbol), Some(fallback));
    assert_eq!(format!("{:?}", core.document()), original);
}

#[test]
fn prepared_source_glyph_replaces_an_earlier_unicode_display_and_clear_restores_it() {
    assert_prepared_source_symbol('\u{f02fc}', "►");
}

#[test]
fn source_square_uses_supplied_hmtx_and_clear_restores_unicode_fallback() {
    assert_prepared_source_symbol('\u{f031c}', "■");
}

#[test]
fn additional_source_box_minus_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f03a7}', "⊟");
}

#[test]
fn additional_source_box_plus_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f03a8}', "⊞");
}

#[test]
fn additional_source_dash_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f0848}', "━");
}
