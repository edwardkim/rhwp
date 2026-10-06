//! Stored left/right fragments are one indivisible physical row during cell splitting.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{ShapeObject, TextWrap, VertRelTo};
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
fn empty_last_wrap_fragment_still_advances_the_visible_row() {
    fn tops(node: &RenderNode, out: &mut Vec<f64>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.para_index.is_some_and(|pi| (35..=38).contains(&pi))
                && line.line_index == Some(0)
            {
                out.push(node.bbox.y);
            }
        }
        for child in &node.children {
            tops(child, out);
        }
    }
    let core = DocumentCore::from_bytes(CHINESE_SQUARE_PICTURE_SAMPLE).expect("Chinese newsletter");
    let mut y = Vec::new();
    tops(&core.build_page_render_tree(0).expect("page").root, &mut y);
    assert_eq!(y.len(), 4);
    for pair in y.windows(2) {
        assert!(
            (pair[1] - pair[0] - 1760.0 / 75.0).abs() < 0.1,
            "stored visible rows must advance: {y:?}"
        );
    }
}

#[test]
fn chinese_footer_is_painted_inside_the_second_page_clip() {
    let core = DocumentCore::from_bytes(CHINESE_SQUARE_PICTURE_SAMPLE).expect("Chinese newsletter");
    let lines = visible_svg_lines(&core, 1);
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("儿童之家婴幼儿健康管理访问项目团"))
            .count(),
        1,
        "missing footer: {lines:?}"
    );
}

#[test]
fn terminal_frame_does_not_cross_the_recovered_footer_line() {
    fn outer_table(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(table) if table.cell_context.is_none()) {
            return Some(node);
        }
        node.children.iter().find_map(outer_table)
    }
    let core = DocumentCore::from_bytes(CHINESE_SQUARE_PICTURE_SAMPLE).expect("newsletter");
    let tree = core.build_page_render_tree(1).expect("page");
    let table = outer_table(&tree.root).expect("outer table");
    let cell = table.children.iter().find(|node|
        matches!(&node.node_type, RenderNodeType::TableCell(meta) if meta.model_cell_index == Some(3)))
        .expect("content cell");
    let footer = cell.children.iter().find(|node|
        matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(65)))
        .expect("footer line");
    let frame_bottom = table
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Line(line) if (line.y1 - line.y2).abs() < 0.01 => Some(line.y1),
            _ => None,
        })
        .reduce(f64::max)
        .expect("horizontal frame");
    let footer_bottom = footer.bbox.y + footer.bbox.height;
    assert!(
        frame_bottom >= footer_bottom + 850.0 / 75.0 - 0.1,
        "frame={frame_bottom}, footer={footer_bottom}"
    );
    assert!(frame_bottom <= tree.root.bbox.height, "frame stays on page");
}

fn second_page_wrap_starts(core: &DocumentCore) -> [f64; 3] {
    fn collect(node: &RenderNode, out: &mut [Vec<f64>; 3]) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if let Some(pi @ 41..=43) = line.para_index {
                if line.line_index == Some(0) {
                    out[pi - 41].push(node.bbox.x);
                }
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut starts: [Vec<f64>; 3] = Default::default();
    collect(
        &core.build_page_render_tree(1).expect("page 2").root,
        &mut starts,
    );
    starts.map(|values| {
        assert_eq!(values.len(), 1, "one source line: {values:?}");
        values[0]
    })
}

#[test]
fn following_paragraph_wraps_beside_a_picture_with_its_own_title() {
    let core = DocumentCore::from_bytes(KOREAN_SQUARE_PICTURE_SAMPLE).expect("newsletter");
    let [title, following, below] = second_page_wrap_starts(&core);
    assert!(
        (following - title).abs() < 0.1,
        "title={title}, following={following}"
    );
    assert!(
        (following - below - 5297.0 / 75.0).abs() < 0.1,
        "following={following}, below={below}"
    );
}

#[test]
fn following_paragraph_requires_real_adjacent_stored_segments() {
    for synthetic in [false, true] {
        let mut core = DocumentCore::from_bytes(KOREAN_SQUARE_PICTURE_SAMPLE).expect("newsletter");
        let mut doc = core.document().clone();
        let Control::Table(table) = &mut doc.sections[0].paragraphs[0].controls[2] else {
            panic!("outer table");
        };
        for seg in &mut table.cells[3].paragraphs[42].line_segs {
            if synthetic {
                seg.tag |= LineSeg::TAG_IMPLEMENTATION_PROPERTY;
            } else {
                seg.column_start = 0;
                seg.segment_width = 46208;
            }
        }
        core.set_document(doc);
        let [title, following, below] = second_page_wrap_starts(&core);
        assert!(
            (following - below).abs() < 0.1,
            "synthetic={synthetic}: {title}/{following}/{below}"
        );
    }
}

#[test]
fn footer_overlay_group_images_are_owned_once_by_the_last_page() {
    fn source_images(shape: &ShapeObject, ids: &mut Vec<u16>) {
        match shape {
            ShapeObject::Picture(picture) => ids.push(picture.image_attr.bin_data_id),
            ShapeObject::Group(group) => {
                for child in &group.children {
                    source_images(child, ids);
                }
            }
            _ => {}
        }
    }
    fn painted_images(node: &RenderNode, ids: &mut Vec<u16>) {
        if !node.visible {
            return;
        }
        if let RenderNodeType::Image(image) = &node.node_type {
            ids.push(image.bin_data_id);
        }
        for child in &node.children {
            painted_images(child, ids);
        }
    }
    let core = DocumentCore::from_bytes(KOREAN_SQUARE_PICTURE_SAMPLE).expect("newsletter");
    let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
        panic!("table");
    };
    let mut expected = Vec::new();
    for control in &table.cells[3].paragraphs[67].controls {
        if let Control::Shape(shape) = control {
            assert_eq!(shape.common().text_wrap, TextWrap::InFrontOfText);
            source_images(shape, &mut expected);
        }
    }
    assert!(!expected.is_empty(), "footer group images in source");
    assert_eq!(core.page_count(), 2);
    for page in 0..core.page_count() {
        let mut painted = Vec::new();
        painted_images(
            &core.build_page_render_tree(page).expect("page").root,
            &mut painted,
        );
        for id in &expected {
            assert_eq!(
                painted.iter().filter(|actual| *actual == id).count(),
                usize::from(page == 1),
                "page {page}: bin {id}, actual={painted:?}"
            );
        }
    }
}

#[test]
fn empty_picture_anchor_preserves_its_distinct_stored_row_advance() {
    fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.para_index == Some(para) && line.line_index == Some(0) {
                return Some(node.bbox.y);
            }
        }
        node.children.iter().find_map(|child| line_top(child, para))
    }
    for (bytes, anchor, page) in [
        (KOREAN_SQUARE_PICTURE_SAMPLE, 8, 0),
        (KOREAN_SQUARE_PICTURE_SAMPLE, 34, 0),
        (CHINESE_SQUARE_PICTURE_SAMPLE, 40, 1),
    ] {
        let core = DocumentCore::from_bytes(bytes).expect("newsletter");
        let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
            panic!("table");
        };
        let para = &table.cells[3].paragraphs[anchor];
        let next = &table.cells[3].paragraphs[anchor + 1];
        assert!(para.text.trim().is_empty());
        let styles = &core.document().doc_info.para_shapes;
        let step = para.line_segs[0].line_height
            + para.line_segs[0].line_spacing
            + (styles[para.para_shape_id as usize].spacing_after
                + styles[next.para_shape_id as usize].spacing_before)
                / 2;
        assert_eq!(
            next.line_segs[0].vertical_pos - para.line_segs[0].vertical_pos,
            step
        );
        let tree = core.build_page_render_tree(page).expect("page");
        let actual = line_top(&tree.root, anchor + 1).expect("next line")
            - line_top(&tree.root, anchor).expect("anchor line");
        assert!(
            (actual - f64::from(step) / 75.0).abs() < 0.1,
            "anchor={anchor}: actual={actual}, stored={step} HU"
        );
    }
}

#[test]
fn stored_square_picture_flow_matches_korean_hancom_oracle() {
    assert_issue_6712_two_page_oracle("한국어 가정통신문", KOREAN_SQUARE_PICTURE_SAMPLE);
}

#[test]
fn text_after_a_nested_square_table_keeps_the_stored_successor_slot() {
    fn first_line(node: &RenderNode, para: usize) -> Option<f64> {
        if matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(para) && line.line_index == Some(0))
        {
            return Some(node.bbox.y);
        }
        node.children
            .iter()
            .find_map(|child| first_line(child, para))
    }
    for bytes in [KOREAN_SQUARE_PICTURE_SAMPLE, CHINESE_SQUARE_PICTURE_SAMPLE] {
        let core = DocumentCore::from_bytes(bytes).expect("newsletter");
        let Control::Table(table) = &core.document().sections[0].paragraphs[0].controls[2] else {
            panic!("outer table");
        };
        let cell = &table.cells[3];
        let host = 17;
        let next = (host + 1..cell.paragraphs.len())
            .find(|&idx| !cell.paragraphs[idx].text.trim().is_empty())
            .expect("text after empty side band");
        let expected = f64::from(
            cell.paragraphs[next].line_segs[0].vertical_pos
                - cell.paragraphs[host].line_segs[0].vertical_pos,
        ) / 75.0;
        let tree = core.build_page_render_tree(0).expect("page");
        let actual = first_line(&tree.root, next).unwrap() - first_line(&tree.root, host).unwrap();
        assert!(
            (actual - expected).abs() < 0.1,
            "actual={actual}, expected={expected}, next={next}"
        );
    }
}

#[test]
fn stored_square_picture_flow_matches_chinese_hancom_oracle() {
    assert_issue_6712_two_page_oracle("중국어 가정통신문", CHINESE_SQUARE_PICTURE_SAMPLE);
}

#[test]
fn nested_square_tables_use_the_host_origin_and_stored_offset() {
    fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
        if matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(para) && line.line_index == Some(0))
        {
            return Some(node.bbox.y);
        }
        node.children.iter().find_map(|child| line_top(child, para))
    }
    fn table_top(node: &RenderNode, para: usize) -> Option<f64> {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(para) && table.cell_context.is_some())
        {
            return Some(node.bbox.y);
        }
        node.children
            .iter()
            .find_map(|child| table_top(child, para))
    }
    for (bytes, second_host) in [
        (KOREAN_SQUARE_PICTURE_SAMPLE, 61),
        (CHINESE_SQUARE_PICTURE_SAMPLE, 60),
    ] {
        let core = DocumentCore::from_bytes(bytes).expect("newsletter");
        let Control::Table(outer) = &core.document().sections[0].paragraphs[0].controls[2] else {
            panic!("outer table");
        };
        for (page, host) in [(0, 17), (1, second_host)] {
            let Control::Table(nested) = &outer.cells[3].paragraphs[host].controls[0] else {
                panic!("nested table");
            };
            let expected = f64::from(nested.common.vertical_offset as i32) / 75.0
                + f64::from(nested.outer_margin_top) / 75.0;
            let tree = core.build_page_render_tree(page).expect("page");
            let actual = table_top(&tree.root, host).expect("nested paint")
                - line_top(&tree.root, host).expect("host line");
            assert!(
                (actual - expected).abs() < 0.1,
                "page={page}, host={host}: actual={actual}, expected={expected}"
            );
        }
    }
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
