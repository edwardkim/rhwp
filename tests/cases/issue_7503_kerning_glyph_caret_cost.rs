//! [#7503] 등록 글꼴로 커닝한 글자를 커닝 전 폭으로, 커닝한 자리에 그린다.
//!
//! 글꼴은 저장소의 합성 글꼴이다(`pos A V -80`, `pos T o -40`, 1000 em, advance 600).
//! 26pt(34.67px)에서 `A`의 advance는 24.47px이고 `AV` 커닝은 -2.77px이다.
//! 같은 문서를 글꼴 등록 없이 그린 결과를 커닝 전 기준으로 쓴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const KERNING_FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const TEXT: &str = "AVTo";

/// 글자모양을 모두 커닝·26pt로 바꾼 새 문서.
fn kerning_blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("새 문서 템플릿");
    let mut document = core.document().clone();
    for char_shape in &mut document.doc_info.char_shapes {
        char_shape.raw_data = None;
        char_shape.kerning = true;
        char_shape.base_size = 2_600;
    }
    core.set_document(document);
    core
}

/// 모든 글자모양의 영문 slot(언어 1)에 글꼴을 등록한다.
fn register_all(core: &mut DocumentCore, font: &[u8]) {
    let count = core.document().doc_info.char_shapes.len() as u32;
    for char_shape_id in 0..count {
        core.register_exact_font_source_native(char_shape_id, 1, font, 0)
            .expect("글꼴 등록");
    }
}

/// 본문 문단 0, 표 셀(문단 1의 표), 머리말, 각주에 `AVTo`를 넣는다.
fn fixture(register: bool) -> (DocumentCore, usize) {
    let mut core = kerning_blank();
    core.insert_text_native(0, 0, 0, TEXT).expect("본문 입력");
    core.split_paragraph_native(0, 0, TEXT.len(), None)
        .expect("문단 나누기");
    core.create_table_native(0, 1, 0, 1, 1).expect("1×1 표");
    let table_ctrl = core.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .expect("표 컨트롤");
    core.insert_text_in_cell_native(0, 1, table_ctrl, 0, 0, 0, TEXT)
        .expect("셀 입력");
    core.create_header_footer_native(0, true, 0)
        .expect("머리말");
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, TEXT)
        .expect("머리말 입력");
    core.insert_footnote_native(0, 0, TEXT.len()).expect("각주");
    let footnote_ctrl = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Footnote(_)))
        .expect("각주 컨트롤");
    core.insert_text_in_footnote_native(0, 0, footnote_ctrl, 0, 0, TEXT)
        .expect("각주 입력");
    if register {
        register_all(&mut core, KERNING_FONT);
    }
    (core, table_ctrl)
}

#[derive(Debug, Clone, PartialEq)]
struct SvgGlyph {
    ch: char,
    x: f64,
    y: f64,
    text_length: Option<f64>,
}

fn svg_attr(tag: &str, name: &str) -> Option<f64> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let end = start + tag[start..].find('"')?;
    tag[start..end].parse().ok()
}

/// SVG의 한 글자 `<text>`를 기준선별로 모아 `AVTo` 줄만 돌려준다.
fn svg_lines(svg: &str) -> Vec<Vec<SvgGlyph>> {
    let mut glyphs = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let tag = format!(" {}", &chunk[..tag_end]);
        let Some(content_end) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        let content = &chunk[tag_end + 1..tag_end + 1 + content_end];
        let mut chars = content.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            continue;
        };
        if !TEXT.contains(ch) {
            continue;
        }
        let (Some(x), Some(y)) = (svg_attr(&tag, "x"), svg_attr(&tag, "y")) else {
            continue;
        };
        glyphs.push(SvgGlyph {
            ch,
            x,
            y,
            text_length: svg_attr(&tag, "textLength"),
        });
    }
    let mut lines: Vec<Vec<SvgGlyph>> = Vec::new();
    for glyph in glyphs {
        match lines
            .iter_mut()
            .find(|line| (line[0].y - glyph.y).abs() < 1e-6)
        {
            Some(line) => line.push(glyph),
            None => lines.push(vec![glyph]),
        }
    }
    for line in &mut lines {
        line.sort_by(|a, b| a.x.total_cmp(&b.x));
    }
    lines.retain(|line| line.iter().map(|glyph| glyph.ch).eq(TEXT.chars()));
    lines.sort_by(|a, b| a[0].y.total_cmp(&b[0].y));
    lines
}

#[test]
fn issue_7503_kerned_glyphs_keep_their_natural_width() {
    let (plain, _) = fixture(false);
    let (kerned, _) = fixture(true);
    let plain_lines = svg_lines(&plain.render_page_svg_native(0).expect("커닝 전 SVG"));
    let kerned_lines = svg_lines(&kerned.render_page_svg_native(0).expect("커닝 SVG"));
    assert_eq!(plain_lines.len(), 4, "본문·셀·머리말·각주: {plain_lines:?}");
    assert_eq!(kerned_lines.len(), plain_lines.len());

    let mut kerned_line_count = 0;
    for (plain_line, kerned_line) in plain_lines.iter().zip(&kerned_lines) {
        // 커닝은 자리만 옮긴다. 글리프 폭은 커닝 전 advance 그대로다.
        for (plain_glyph, kerned_glyph) in plain_line.iter().zip(kerned_line) {
            assert!(plain_glyph.text_length.is_some());
            assert_eq!(
                kerned_glyph.text_length, plain_glyph.text_length,
                "'{}'가 커닝만큼 눌렸다: {kerned_line:?}",
                kerned_glyph.ch
            );
        }
        if kerned_line[1].x < plain_line[1].x - 1.0 {
            kerned_line_count += 1;
            // `V`는 `A` 뒤 커닝한 자리, `o`는 `T` 뒤 커닝한 자리에 놓인다.
            assert!(kerned_line[3].x - kerned_line[2].x < plain_line[3].x - plain_line[2].x - 0.5);
        }
    }
    assert!(
        kerned_line_count >= 3,
        "본문·셀·머리말은 커닝해야 한다: {kerned_lines:?}"
    );
}
