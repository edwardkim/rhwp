//! Stored left/right fragments are one indivisible physical row during cell splitting.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn document(rows: usize, page_height: u32, paired: bool, synthetic: bool) -> DocumentCore {
    let text: String = (0..rows * 2).map(|i| char::from(b'A' + i as u8)).collect();
    let segs = (0..rows * 2)
        .map(|i| LineSeg {
            text_start: i as u32,
            vertical_pos: (i / 2) as i32 * 1200,
            line_height: 1200,
            text_height: 1200,
            baseline_distance: 1000,
            column_start: if paired && i % 2 == 1 { 9000 } else { 0 },
            segment_width: 8000,
            tag: if synthetic {
                LineSeg::TAG_IMPLEMENTATION_PROPERTY
            } else {
                0
            },
            ..Default::default()
        })
        .collect();
    let para = Paragraph {
        char_count: text.len() as u32,
        char_offsets: (0..=text.len() as u32).collect(),
        text,
        line_segs: segs,
        ..Default::default()
    };
    let cell = Cell {
        row: 1,
        col_span: 1,
        row_span: 1,
        width: 20000,
        height: rows as u32 * 1200,
        paragraphs: vec![para],
        vertical_align: VerticalAlign::Top,
        ..Default::default()
    };
    let mut table = Table {
        row_count: 2,
        col_count: 1,
        cells: vec![
            Cell {
                width: 20000,
                height: 1,
                col_span: 1,
                row_span: 1,
                ..Default::default()
            },
            cell,
        ],
        page_break: TablePageBreak::RowBreak,
        ..Default::default()
    };
    table.common.width = 20000;
    table.common.height = rows as u32 * 1200;
    table.common.treat_as_char = false;
    table.common.flow_with_text = true;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.rebuild_grid();
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 22500,
        height: page_height,
        ..Default::default()
    };
    section.paragraphs.push(Paragraph {
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    });
    let mut doc = Document {
        sections: vec![section],
        ..Default::default()
    };
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    let mut core = DocumentCore::new_empty();
    core.set_document(doc);
    core
}

fn glyph_pages(core: &DocumentCore) -> std::collections::BTreeMap<char, (u32, f64)> {
    fn collect(
        node: &RenderNode,
        page: u32,
        out: &mut std::collections::BTreeMap<char, (u32, f64)>,
    ) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            for c in run.text.chars().filter(char::is_ascii_uppercase) {
                assert!(
                    out.insert(c, (page, node.bbox.y)).is_none(),
                    "duplicate {c}"
                );
            }
        }
        for child in &node.children {
            collect(child, page, out);
        }
    }
    let mut out = std::collections::BTreeMap::new();
    for page in 0..core.page_count() {
        collect(
            &core.build_page_render_tree(page).expect("page").root,
            page,
            &mut out,
        );
    }
    out
}

const KOREAN_SQUARE_PICTURE_SAMPLE: &[u8] = include_bytes!(
    "../../samples/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp"
);
const CHINESE_SQUARE_PICTURE_SAMPLE: &[u8] = include_bytes!(
    "../../samples/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp"
);

fn assert_issue_6712_two_page_oracle(label: &str, bytes: &[u8]) {
    let core =
        DocumentCore::from_bytes(bytes).unwrap_or_else(|error| panic!("{label} 열기: {error:?}"));
    assert_eq!(
        core.page_count(),
        2,
        "{label}: 한컴 기준 PDF는 2쪽이다. 저장된 Square 그림 옆 줄의 그림 높이를 중복 계상하면 3쪽이 된다."
    );
    let _ = core.take_overflow_cell_lines();
    for page in 0..core.page_count() {
        let _ = core.build_page_render_tree(page).expect("render page");
        assert_eq!(
            core.take_overflow_cell_lines(),
            0,
            "{label} page {page}: missing tail lines"
        );
    }
}

fn visible_svg_lines(core: &DocumentCore, page: u32) -> Vec<String> {
    let svg = core.render_page_svg_native(page).expect("SVG");
    let xml = roxmltree::Document::parse(&svg).expect("SVG XML");
    let height: f64 = xml
        .root_element()
        .attribute("height")
        .unwrap()
        .parse()
        .unwrap();
    let mut lines = std::collections::BTreeMap::<i64, String>::new();
    for glyph in xml.descendants().filter(|node| node.has_tag_name("text")) {
        let Some(y) = glyph
            .attribute("y")
            .and_then(|value| value.parse::<f64>().ok())
        else {
            continue;
        };
        let inside_clips = glyph.ancestors().all(|ancestor| {
            let Some(id) = ancestor
                .attribute("clip-path")
                .and_then(|value| value.strip_prefix("url(#"))
                .and_then(|value| value.strip_suffix(')'))
            else {
                return true;
            };
            let clip = xml
                .descendants()
                .find(|node| node.attribute("id") == Some(id))
                .expect("referenced clip");
            let rect = clip
                .children()
                .find(|node| node.has_tag_name("rect"))
                .expect("rect clip");
            let top: f64 = rect.attribute("y").unwrap().parse().unwrap();
            let h: f64 = rect.attribute("height").unwrap().parse().unwrap();
            y >= top && y <= top + h
        });
        if (0.0..=height).contains(&y) && inside_clips {
            let text = glyph
                .descendants()
                .filter_map(|node| node.is_text().then(|| node.text()).flatten())
                .collect::<String>();
            lines
                .entry((y * 10.0).round() as i64)
                .or_default()
                .extend(text.chars().filter(|c| !c.is_whitespace()));
        }
    }
    lines.into_values().collect()
}

#[test]
fn prevention_tail_is_visible_once_on_its_original_page() {
    for (bytes, phrases) in [
        (
            KOREAN_SQUARE_PICTURE_SAMPLE,
            [
                "30초이상손씻기",
                "눈,얼굴을손으로만지거나비비지않기",
                "개인물품따로쓰기",
                "사람많은곳에가지않기",
            ],
        ),
        (
            CHINESE_SQUARE_PICTURE_SAMPLE,
            [
                "洗手30秒以上",
                "眼睛和脸部不要用手摸或揉",
                "洗漱用品等个人物品",
                "流行眼病时禁止去游泳池等人多的地方",
            ],
        ),
    ] {
        let core = DocumentCore::from_bytes(bytes).expect("newsletter");
        let first = visible_svg_lines(&core, 0);
        let second = visible_svg_lines(&core, 1);
        for phrase in phrases {
            assert_eq!(
                first.iter().filter(|line| line.contains(phrase)).count(),
                1,
                "missing or duplicate {phrase}: {first:?}"
            );
        }
        // Handwashing advice legitimately repeats on page 2. The eye-disease
        // instruction is the page-specific owner marker, not its shared phrases.
        let owner_marker = phrases[3];
        assert!(
            !second.iter().any(|line| line.contains(owner_marker)),
            "moved to page 2: {owner_marker}"
        );
    }
}

#[test]
fn stored_square_picture_flow_matches_korean_hancom_oracle() {
    assert_issue_6712_two_page_oracle("한국어 가정통신문", KOREAN_SQUARE_PICTURE_SAMPLE);
}

#[test]
fn stored_square_picture_flow_matches_chinese_hancom_oracle() {
    assert_issue_6712_two_page_oracle("중국어 가정통신문", CHINESE_SQUARE_PICTURE_SAMPLE);
}

#[test]
fn whole_table_preserves_both_fragments_on_each_visual_row() {
    let core = document(4, 7200, true, false);
    assert_eq!(core.page_count(), 1, "four 16px rows fit in 96px");
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 8);
    for pair in [b'A', b'C', b'E', b'G'] {
        let left = glyphs[&char::from(pair)];
        let right = glyphs[&char::from(pair + 1)];
        assert_eq!(left.0, right.0);
        assert!((left.1 - right.1).abs() < 0.1);
    }
}

#[test]
fn page_cuts_never_split_the_left_and_right_fragments_of_a_row() {
    let core = document(6, 4200, true, false);
    assert_eq!(core.page_count(), 2, "six 16px rows need two 56px pages");
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 12);
    for pair in [b'A', b'C', b'E', b'G', b'I', b'K'] {
        assert_eq!(
            glyphs[&char::from(pair)].0,
            glyphs[&char::from(pair + 1)].0,
            "pair {}",
            char::from(pair)
        );
        assert!((glyphs[&char::from(pair)].1 - glyphs[&char::from(pair + 1)].1).abs() < 0.1);
    }
}

#[test]
fn row_advance_uses_the_last_fragment_height() {
    let mut core = document(6, 4200, true, false);
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[0].controls[0] else {
        panic!("table");
    };
    for seg in table.cells[1].paragraphs[0].line_segs.iter_mut().step_by(2) {
        seg.line_height = 600;
    }
    core.set_document(doc);
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 12);
    assert_eq!(core.page_count(), 2);
    assert_eq!(glyphs[&'E'].0, 0);
    assert_eq!(glyphs[&'F'].0, 0);
    assert_eq!(glyphs[&'G'].0, 1);
    assert_eq!(glyphs[&'H'].0, 1);
}

#[test]
fn three_fragments_of_a_row_are_also_indivisible() {
    let mut core = document(6, 4200, true, false);
    let mut doc = core.document().clone();
    let Control::Table(table) = &mut doc.sections[0].paragraphs[0].controls[0] else {
        panic!("table");
    };
    for (i, seg) in table.cells[1].paragraphs[0]
        .line_segs
        .iter_mut()
        .enumerate()
    {
        seg.vertical_pos = (i / 3) as i32 * 1200;
        seg.column_start = (i % 3) as i32 * 8000;
        seg.segment_width = 7000;
    }
    core.set_document(doc);
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 12);
    assert_eq!(core.page_count(), 2);
    for first in [b'A', b'D', b'G', b'J'] {
        let page = glyphs[&char::from(first)].0;
        assert_eq!(glyphs[&char::from(first + 1)].0, page);
        assert_eq!(glyphs[&char::from(first + 2)].0, page);
    }
}

#[test]
fn equal_vpos_without_different_columns_is_not_a_wrap_fragment() {
    let core = document(6, 4200, false, false);
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 12);
    assert_ne!(
        glyphs[&'C'].0, glyphs[&'D'].0,
        "same-column rows: {glyphs:?}"
    );
}

#[test]
fn synthetic_segments_are_not_stored_wrap_evidence() {
    let core = document(6, 4200, true, true);
    let glyphs = glyph_pages(&core);
    assert_eq!(glyphs.len(), 12);
    assert_ne!(glyphs[&'C'].0, glyphs[&'D'].0, "synthetic rows: {glyphs:?}");
}
