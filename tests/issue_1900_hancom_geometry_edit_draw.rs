//! Regression tests for reversible text edits over Hancom line geometry.
//!
//! Edit commands preserve Hancom line records instead of using visible rendering
//! as persistent document state. Rendering must use those records as line-frame
//! geometry: explicit line starts remain stable, while a lone line segment still
//! creates continuation visual lines when the text exceeds that frame.

use rhwp::model::control::Control;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{Caption, CaptionDirection};
use rhwp::model::style::Alignment;
use rhwp::model::table::VerticalAlign;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const TEXT: &str = "가나다라마바사아자차카타파하";
const SECOND_LINE_START: u32 = 7;
const OPEN_SOURCE_CONTEST_SAMPLE: &str = "samples/issue-1900-opensource-developer-contest.hwpx";
const OPEN_SOURCE_CONTEST_PARAGRAPH: &str = "올해로 20주년을 맞은 동 대회는(‘07년 첫 개최) 누적 5,900여 팀(1만 6천여 명)이 참여한 국내 대표 오픈소스 경진대회로, 정부와 오픈소스 기업·전문가가 함께 이끌어 온 민관협력 대회이다.";
const OPEN_SOURCE_CONTEST_FRAGMENTS: [&str; 5] = [
    "올해로 20주년을 맞은",
    "누적 5,900여 팀",
    "국내 대표 오픈소스 경진대회로",
    "정부와 오픈소스 기업·전문가가",
    "민관협력 대회이다",
];

type LineSegSnapshot = Vec<(u32, i32, i32, i32, i32, i32, u32)>;

#[derive(Debug, Clone)]
struct TextLineProbe {
    page: u32,
    text: String,
    bbox: BoundingBox,
    max_run_right: f64,
}

fn utf16_offsets(text: &str) -> Vec<u32> {
    let mut offsets = Vec::with_capacity(text.chars().count());
    let mut pos = 0;
    for ch in text.chars() {
        offsets.push(pos);
        pos += ch.len_utf16() as u32;
    }
    offsets
}

fn real_line_seg(text_start: u32, vertical_pos: i32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height: 1_200,
        text_height: 900,
        baseline_distance: 700,
        line_spacing: 300,
        column_start: 0,
        segment_width: 30_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
    }
}

fn seed_single_segment_geometry(para: &mut Paragraph, text: &str) {
    para.text = text.to_string();
    para.char_offsets = utf16_offsets(text);
    para.char_count = text.chars().count() as u32;
    para.line_segs = vec![real_line_seg(0, 0)];
}

fn seed_hancom_geometry(para: &mut Paragraph, text: &str) {
    para.text = text.to_string();
    para.char_offsets = utf16_offsets(text);
    para.char_count = text.chars().count() as u32;
    para.line_segs = vec![
        real_line_seg(0, 0),
        real_line_seg(SECOND_LINE_START, 12_000),
    ];
}

fn collect_non_empty_text_runs<'a>(node: &'a RenderNode, out: &mut Vec<(String, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push((run.text.clone(), node.bbox.y));
        }
    }
    for child in &node.children {
        collect_non_empty_text_runs(child, out);
    }
}

fn collect_non_empty_text_run_bboxes<'a>(
    node: &'a RenderNode,
    out: &mut Vec<(String, &'a BoundingBox)>,
) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push((run.text.clone(), &node.bbox));
        }
    }
    for child in &node.children {
        collect_non_empty_text_run_bboxes(child, out);
    }
}

fn collect_text_run_bboxes<'a>(node: &'a RenderNode, out: &mut Vec<(String, &'a BoundingBox)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((run.text.clone(), &node.bbox));
    }
    for child in &node.children {
        collect_text_run_bboxes(child, out);
    }
}

fn collect_text_lines(root: &RenderNode, page: u32, out: &mut Vec<TextLineProbe>) {
    if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        let mut runs = Vec::new();
        collect_text_run_bboxes(root, &mut runs);
        let text: String = runs.iter().map(|(text, _)| text.as_str()).collect();
        if !text.trim().is_empty() {
            let max_run_right = runs
                .iter()
                .map(|(_, bbox)| bbox.x + bbox.width)
                .fold(root.bbox.x + root.bbox.width, f64::max);
            out.push(TextLineProbe {
                page,
                text,
                bbox: root.bbox,
                max_run_right,
            });
        }
    }

    for child in &root.children {
        collect_text_lines(child, page, out);
    }
}

fn collect_body_right_edges(root: &RenderNode, page: u32, out: &mut BTreeMap<u32, f64>) {
    if matches!(root.node_type, RenderNodeType::Body { .. }) {
        out.insert(page, root.bbox.x + root.bbox.width);
    }
    for child in &root.children {
        collect_body_right_edges(child, page, out);
    }
}

fn node_text(node: &RenderNode) -> String {
    let mut runs = Vec::new();
    collect_non_empty_text_runs(node, &mut runs);
    runs.into_iter().map(|(text, _)| text).collect()
}

fn normalize_visible_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn load_core(sample: &str) -> DocumentCore {
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&sample_path).unwrap_or_else(|error| panic!("read {sample}: {error}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn render_lines_and_body_rights(core: &DocumentCore) -> (Vec<TextLineProbe>, BTreeMap<u32, f64>) {
    let mut lines = Vec::new();
    let mut body_rights = BTreeMap::new();

    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|error| panic!("render page {page}: {error:?}"));
        collect_text_lines(&tree.root, page, &mut lines);
        collect_body_right_edges(&tree.root, page, &mut body_rights);
    }

    (lines, body_rights)
}

fn lines_matching_fragments(lines: &[TextLineProbe], fragments: &[&str]) -> Vec<TextLineProbe> {
    lines
        .iter()
        .filter(|line| {
            let normalized = normalize_visible_text(&line.text);
            fragments
                .iter()
                .any(|fragment| normalized.contains(fragment))
        })
        .cloned()
        .collect()
}

fn distinct_line_rows(lines: &[TextLineProbe]) -> Vec<(u32, i32)> {
    let mut rows: Vec<(u32, i32)> = lines
        .iter()
        .map(|line| (line.page, (line.bbox.y * 2.0).round() as i32))
        .collect();
    rows.sort();
    rows.dedup();
    rows
}

fn find_table_cell_containing_text<'a>(node: &'a RenderNode, text: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) && node_text(node).contains(text) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table_cell_containing_text(child, text))
}

fn line_seg_snapshot(para: &Paragraph) -> LineSegSnapshot {
    para.line_segs
        .iter()
        .map(|seg| {
            (
                seg.text_start,
                seg.vertical_pos,
                seg.line_height,
                seg.text_height,
                seg.baseline_distance,
                seg.line_spacing,
                seg.tag,
            )
        })
        .collect()
}

fn refresh_from_ir(core: &mut DocumentCore) {
    let document = core.document().clone();
    core.set_document(document);
}

fn parse_usize(json: &str, key: &str) -> usize {
    let value: Value = serde_json::from_str(json).expect("parse command result");
    value[key].as_u64().expect(key) as usize
}

fn para_shape_id_with_alignment(core: &mut DocumentCore, alignment: Alignment) -> u16 {
    let document = core.document_mut();
    if document.doc_info.para_shapes.is_empty() {
        document.doc_info.para_shapes.push(Default::default());
    }
    if let Some((idx, _)) = document
        .doc_info
        .para_shapes
        .iter()
        .enumerate()
        .find(|(_, para_shape)| para_shape.alignment == alignment)
    {
        return idx as u16;
    }

    let mut para_shape = document.doc_info.para_shapes[0].clone();
    para_shape.raw_data = None;
    para_shape.alignment = alignment;
    document.doc_info.para_shapes.push(para_shape);
    (document.doc_info.para_shapes.len() - 1) as u16
}

fn alignment_cases() -> [(Alignment, &'static str); 6] {
    [
        (Alignment::Left, "left"),
        (Alignment::Right, "right"),
        (Alignment::Center, "center"),
        (Alignment::Justify, "justify"),
        (Alignment::Distribute, "distribute"),
        (Alignment::Split, "split"),
    ]
}

fn vertical_alignment_cases() -> [(VerticalAlign, &'static str); 3] {
    [
        (VerticalAlign::Top, "top"),
        (VerticalAlign::Center, "center"),
        (VerticalAlign::Bottom, "bottom"),
    ]
}

fn assert_tail_edit_round_trip<S, E, D>(
    core: &mut DocumentCore,
    snapshot: S,
    text: D,
    insert_tail: E,
    context: &str,
) where
    S: Fn(&DocumentCore) -> LineSegSnapshot,
    E: FnOnce(&mut DocumentCore, usize),
    D: Fn(&DocumentCore) -> String,
{
    let original_text = text(core);
    let before = snapshot(core);
    let tail = original_text.chars().count();

    insert_tail(core, tail);

    assert_eq!(
        text(core),
        format!("{original_text}123"),
        "{context}: insert did not update text"
    );
    assert_eq!(
        snapshot(core),
        before,
        "{context}: tail insert must not change Hancom line geometry"
    );
}

fn assert_delete_tail_restores<S, D>(
    core: &mut DocumentCore,
    snapshot: S,
    text: D,
    before_text: &str,
    before_geometry: &LineSegSnapshot,
    delete_tail: impl FnOnce(&mut DocumentCore, usize),
    context: &str,
) where
    S: Fn(&DocumentCore) -> LineSegSnapshot,
    D: Fn(&DocumentCore) -> String,
{
    delete_tail(core, before_text.chars().count());

    assert_eq!(
        text(core),
        before_text,
        "{context}: delete did not restore text"
    );
    assert_eq!(
        snapshot(core),
        *before_geometry,
        "{context}: insert/delete cycle did not restore Hancom geometry"
    );
}

fn make_body_core(alignment: Alignment) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let para_shape_id = para_shape_id_with_alignment(&mut core, alignment);
    let para = &mut core.document_mut().sections[0].paragraphs[0];
    para.para_shape_id = para_shape_id;
    seed_hancom_geometry(para, TEXT);
    refresh_from_ir(&mut core);
    core
}

fn body_para(core: &DocumentCore) -> &Paragraph {
    &core.document().sections[0].paragraphs[0]
}

#[test]
fn body_tail_insert_delete_restores_geometry_for_all_alignments() {
    for (alignment, name) in alignment_cases() {
        let mut core = make_body_core(alignment);
        let before_text = body_para(&core).text.clone();
        let before_geometry = line_seg_snapshot(body_para(&core));

        assert_tail_edit_round_trip(
            &mut core,
            |core| line_seg_snapshot(body_para(core)),
            |core| body_para(core).text.clone(),
            |core, tail| {
                core.insert_text_native(0, 0, tail, "123").unwrap();
            },
            &format!("body {name}"),
        );
        assert_delete_tail_restores(
            &mut core,
            |core| line_seg_snapshot(body_para(core)),
            |core| body_para(core).text.clone(),
            &before_text,
            &before_geometry,
            |core, tail| {
                core.delete_text_native(0, 0, tail, 3).unwrap();
            },
            &format!("body {name}"),
        );
    }
}

#[test]
fn body_mid_edit_shifts_line_start_then_delete_restores_geometry() {
    let mut core = make_body_core(Alignment::Left);
    let before_geometry = line_seg_snapshot(body_para(&core));

    core.insert_text_native(0, 0, 3, "123").unwrap();
    let inserted = line_seg_snapshot(body_para(&core));
    assert_eq!(inserted[0], before_geometry[0]);
    assert_eq!(inserted[1].0, SECOND_LINE_START + 3);
    assert_eq!(
        (
            inserted[1].1,
            inserted[1].2,
            inserted[1].3,
            inserted[1].4,
            inserted[1].5,
            inserted[1].6,
        ),
        (
            before_geometry[1].1,
            before_geometry[1].2,
            before_geometry[1].3,
            before_geometry[1].4,
            before_geometry[1].5,
            before_geometry[1].6,
        )
    );

    core.delete_text_native(0, 0, 3, 3).unwrap();
    assert_eq!(body_para(&core).text, TEXT);
    assert_eq!(line_seg_snapshot(body_para(&core)), before_geometry);
}

fn make_table_core(
    alignment: Alignment,
    vertical_align: VerticalAlign,
) -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = core
        .create_table_native(0, 0, 0, 1, 1)
        .expect("create one-cell table");
    let para_idx = parse_usize(&created, "paraIdx");
    let control_idx = parse_usize(&created, "controlIdx");
    let para_shape_id = para_shape_id_with_alignment(&mut core, alignment);

    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Table(table) = &mut para.controls[control_idx] else {
            panic!("created control is not a table");
        };
        let cell = &mut table.cells[0];
        cell.width = 50_000;
        cell.height = 24_000;
        cell.vertical_align = vertical_align;
        if cell.paragraphs.is_empty() {
            cell.paragraphs.push(Paragraph::new_empty());
        }
        cell.paragraphs[0].para_shape_id = para_shape_id;
        seed_hancom_geometry(&mut cell.paragraphs[0], TEXT);
        table.dirty = true;
        table.update_ctrl_dimensions();
    }

    refresh_from_ir(&mut core);
    (core, para_idx, control_idx)
}

fn table_cell_para(core: &DocumentCore, para_idx: usize, control_idx: usize) -> &Paragraph {
    let para = &core.document().sections[0].paragraphs[para_idx];
    let Control::Table(table) = &para.controls[control_idx] else {
        panic!("control is not a table");
    };
    &table.cells[0].paragraphs[0]
}

#[test]
fn table_cell_tail_insert_delete_restores_geometry_for_alignments_and_vertical_alignments() {
    for (alignment, align_name) in alignment_cases() {
        for (vertical_align, valign_name) in vertical_alignment_cases() {
            let (mut core, para_idx, control_idx) = make_table_core(alignment, vertical_align);
            let before_text = table_cell_para(&core, para_idx, control_idx).text.clone();
            let before_geometry = line_seg_snapshot(table_cell_para(&core, para_idx, control_idx));
            let context = format!("table cell {align_name}/{valign_name}");

            assert_tail_edit_round_trip(
                &mut core,
                |core| line_seg_snapshot(table_cell_para(core, para_idx, control_idx)),
                |core| table_cell_para(core, para_idx, control_idx).text.clone(),
                |core, tail| {
                    core.insert_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, "123")
                        .unwrap();
                },
                &context,
            );
            assert_delete_tail_restores(
                &mut core,
                |core| line_seg_snapshot(table_cell_para(core, para_idx, control_idx)),
                |core| table_cell_para(core, para_idx, control_idx).text.clone(),
                &before_text,
                &before_geometry,
                |core, tail| {
                    core.delete_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, 3)
                        .unwrap();
                },
                &context,
            );
        }
    }
}

#[test]
fn table_cell_single_segment_wraps_to_hancom_frame() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = core
        .create_table_native(0, 0, 0, 1, 1)
        .expect("create one-cell table");
    let para_idx = parse_usize(&created, "paraIdx");
    let control_idx = parse_usize(&created, "controlIdx");

    let target = "인공지능(AI), 빅데이터, 클라우드, 사물인터넷, 모바일 등 오픈소스 프로젝트";
    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Table(table) = &mut para.controls[control_idx] else {
            panic!("created control is not a table");
        };
        let cell = &mut table.cells[0];
        cell.width = 12_000;
        cell.height = 20_000;
        cell.padding.left = 0;
        cell.padding.right = 0;
        if cell.paragraphs.is_empty() {
            cell.paragraphs.push(Paragraph::new_empty());
        }
        seed_single_segment_geometry(&mut cell.paragraphs[0], target);
        table.dirty = true;
        table.update_ctrl_dimensions();
    }

    refresh_from_ir(&mut core);
    assert_eq!(
        table_cell_para(&core, para_idx, control_idx)
            .line_segs
            .len(),
        1,
        "this regression must exercise one saved Hancom line frame, not source line geometry mutation"
    );

    let tree = core.build_page_render_tree(0).expect("render page");
    let mut text_runs = Vec::new();
    collect_non_empty_text_runs(&tree.root, &mut text_runs);

    let rendered_text: String = text_runs.iter().map(|(text, _)| text.as_str()).collect();
    assert_eq!(
        rendered_text, target,
        "table cell render must preserve the full cell paragraph text"
    );

    let cell = find_table_cell_containing_text(&tree.root, target).expect("rendered text cell");
    let mut cell_text_runs = Vec::new();
    collect_non_empty_text_run_bboxes(cell, &mut cell_text_runs);

    assert!(
        !cell_text_runs.is_empty(),
        "table cell text must render its saved Hancom frame: runs={cell_text_runs:?}"
    );
    let mut ys: Vec<f64> = cell_text_runs.iter().map(|(_, bbox)| bbox.y).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert!(
        ys.len() > 1,
        "single-segment Hancom frame must line-break instead of truncating off the cell: runs={cell_text_runs:?}"
    );
    let cell_text: String = cell_text_runs
        .iter()
        .map(|(text, _)| text.as_str())
        .collect();
    assert_eq!(
        cell_text, target,
        "saved single-line Hancom geometry must not hide or synthesize table-cell text"
    );
}

#[test]
fn opensource_press_paragraph_wraps_before_body_frame_edge() {
    let core = load_core(OPEN_SOURCE_CONTEST_SAMPLE);
    let (lines, body_rights) = render_lines_and_body_rights(&core);
    let target_lines = lines_matching_fragments(&lines, &OPEN_SOURCE_CONTEST_FRAGMENTS);

    let joined = normalize_visible_text(
        &target_lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    for fragment in OPEN_SOURCE_CONTEST_FRAGMENTS {
        assert!(
            joined.contains(fragment),
            "fixture paragraph fragment was not rendered: {fragment:?}; lines={target_lines:#?}"
        );
    }

    let rows = distinct_line_rows(&target_lines);
    assert!(
        rows.len() >= 3,
        "Hancom frame must create multiple visual lines for the open-source press paragraph, not one over-wide row: rows={rows:?} lines={target_lines:#?}"
    );
    assert!(
        target_lines.iter().any(|line| {
            let normalized = normalize_visible_text(&line.text);
            normalized.contains("경진대회로,") && normalized.contains("정부와")
        }),
        "Hancom commits `경진대회로,` and `정부와` to the same source interval; repeated spaces must not become an invented line boundary: lines={target_lines:#?}"
    );

    for line in &target_lines {
        let body_right = body_rights
            .get(&line.page)
            .copied()
            .unwrap_or(f64::INFINITY);
        assert!(
            line.max_run_right <= body_right + 1.0,
            "target paragraph line escapes body frame: page={} body_right={body_right:.2} run_right={:.2} line={:?}",
            line.page,
            line.max_run_right,
            line.text
        );
    }

    assert!(
        joined.contains(OPEN_SOURCE_CONTEST_PARAGRAPH),
        "line breaking must preserve the full logical paragraph text after whitespace normalization: rendered={joined:?}"
    );
}

#[test]
fn table_cell_repeated_spaces_wrap_instead_of_pushing_tail_outside_cell() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = core
        .create_table_native(0, 0, 0, 1, 1)
        .expect("create one-cell table");
    let para_idx = parse_usize(&created, "paraIdx");
    let control_idx = parse_usize(&created, "controlIdx");

    let target = "이 표준하도급계약서에서는 용역                  시행하는 바, 실제 하도급계약을";
    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Table(table) = &mut para.controls[control_idx] else {
            panic!("created control is not a table");
        };
        let cell = &mut table.cells[0];
        cell.width = 38_000;
        cell.height = 24_000;
        cell.padding.left = 0;
        cell.padding.right = 0;
        if cell.paragraphs.is_empty() {
            cell.paragraphs.push(Paragraph::new_empty());
        }
        seed_single_segment_geometry(&mut cell.paragraphs[0], target);
        table.dirty = true;
        table.update_ctrl_dimensions();
    }

    refresh_from_ir(&mut core);
    let tree = core.build_page_render_tree(0).expect("render page");
    let cell = find_table_cell_containing_text(&tree.root, "표준하도급계약서")
        .expect("rendered subcontract text cell");
    let mut cell_text_runs = Vec::new();
    collect_non_empty_text_run_bboxes(cell, &mut cell_text_runs);

    let cell_text = normalize_visible_text(
        &cell_text_runs
            .iter()
            .map(|(text, _)| text.as_str())
            .collect::<Vec<_>>()
            .concat(),
    );
    assert!(
        cell_text.contains("이 표준하도급계약서에서는 용역 시행하는 바, 실제 하도급계약을"),
        "table cell render must preserve the full repeated-space paragraph: text={cell_text:?} runs={cell_text_runs:#?}"
    );

    let rows = distinct_line_rows(
        &cell_text_runs
            .iter()
            .map(|(text, bbox)| TextLineProbe {
                page: 0,
                text: text.clone(),
                bbox: **bbox,
                max_run_right: bbox.x + bbox.width,
            })
            .collect::<Vec<_>>(),
    );
    assert!(
        rows.len() >= 2,
        "repeated-space table paragraph must wrap instead of placing the tail in the same clipped row: rows={rows:?} runs={cell_text_runs:#?}"
    );

    let cell_right = cell.bbox.x + cell.bbox.width;
    for (text, bbox) in &cell_text_runs {
        assert!(
            bbox.x + bbox.width <= cell_right + 1.0,
            "table cell run escapes cell frame: cell_right={cell_right:.2} run_right={:.2} text={text:?}",
            bbox.x + bbox.width
        );
    }
}

fn make_table_caption_core() -> (DocumentCore, usize, usize) {
    let (mut core, para_idx, control_idx) = make_table_core(Alignment::Left, VerticalAlign::Top);

    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Table(table) = &mut para.controls[control_idx] else {
            panic!("control is not a table");
        };
        table.caption = Some(Caption {
            direction: CaptionDirection::Bottom,
            max_width: 50_000,
            paragraphs: vec![Paragraph::new_empty()],
            ..Default::default()
        });
        let caption = table.caption.as_mut().expect("table caption exists");
        seed_hancom_geometry(&mut caption.paragraphs[0], TEXT);
        table.dirty = true;
    }

    refresh_from_ir(&mut core);
    (core, para_idx, control_idx)
}

fn table_caption_para(core: &DocumentCore, para_idx: usize, control_idx: usize) -> &Paragraph {
    let para = &core.document().sections[0].paragraphs[para_idx];
    let Control::Table(table) = &para.controls[control_idx] else {
        panic!("control is not a table");
    };
    &table.caption.as_ref().expect("table caption").paragraphs[0]
}

#[test]
fn table_caption_tail_insert_delete_restores_geometry() {
    let (mut core, para_idx, control_idx) = make_table_caption_core();
    let before_text = table_caption_para(&core, para_idx, control_idx)
        .text
        .clone();
    let before_geometry = line_seg_snapshot(table_caption_para(&core, para_idx, control_idx));

    assert_tail_edit_round_trip(
        &mut core,
        |core| line_seg_snapshot(table_caption_para(core, para_idx, control_idx)),
        |core| table_caption_para(core, para_idx, control_idx).text.clone(),
        |core, tail| {
            core.insert_text_in_cell_native(0, para_idx, control_idx, 65534, 0, tail, "123")
                .unwrap();
        },
        "table caption",
    );
    assert_delete_tail_restores(
        &mut core,
        |core| line_seg_snapshot(table_caption_para(core, para_idx, control_idx)),
        |core| table_caption_para(core, para_idx, control_idx).text.clone(),
        &before_text,
        &before_geometry,
        |core, tail| {
            core.delete_text_in_cell_native(0, para_idx, control_idx, 65534, 0, tail, 3)
                .unwrap();
        },
        "table caption",
    );
}

fn make_textbox_core() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = core
        .create_shape_control_native(
            0,
            0,
            0,
            50_000,
            24_000,
            0,
            0,
            true,
            "TopAndBottom",
            "textbox",
            false,
            false,
            &[],
        )
        .expect("create textbox");
    let para_idx = parse_usize(&created, "paraIdx");
    let control_idx = parse_usize(&created, "controlIdx");

    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Shape(shape) = &mut para.controls[control_idx] else {
            panic!("created control is not a shape");
        };
        let text_box = shape
            .drawing_mut()
            .and_then(|drawing| drawing.text_box.as_mut())
            .expect("created shape has text box");
        text_box.margin_left = 0;
        text_box.margin_right = 0;
        text_box.vertical_align = VerticalAlign::Top;
        if text_box.paragraphs.is_empty() {
            text_box.paragraphs.push(Paragraph::new_empty());
        }
        seed_hancom_geometry(&mut text_box.paragraphs[0], TEXT);
    }

    refresh_from_ir(&mut core);
    (core, para_idx, control_idx)
}

fn textbox_para(core: &DocumentCore, para_idx: usize, control_idx: usize) -> &Paragraph {
    let para = &core.document().sections[0].paragraphs[para_idx];
    let Control::Shape(shape) = &para.controls[control_idx] else {
        panic!("control is not a shape");
    };
    let text_box = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
        .expect("shape has text box");
    &text_box.paragraphs[0]
}

#[test]
fn textbox_tail_insert_delete_restores_geometry() {
    let (mut core, para_idx, control_idx) = make_textbox_core();
    let before_text = textbox_para(&core, para_idx, control_idx).text.clone();
    let before_geometry = line_seg_snapshot(textbox_para(&core, para_idx, control_idx));

    assert_tail_edit_round_trip(
        &mut core,
        |core| line_seg_snapshot(textbox_para(core, para_idx, control_idx)),
        |core| textbox_para(core, para_idx, control_idx).text.clone(),
        |core, tail| {
            core.insert_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, "123")
                .unwrap();
        },
        "textbox",
    );
    assert_delete_tail_restores(
        &mut core,
        |core| line_seg_snapshot(textbox_para(core, para_idx, control_idx)),
        |core| textbox_para(core, para_idx, control_idx).text.clone(),
        &before_text,
        &before_geometry,
        |core, tail| {
            core.delete_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, 3)
                .unwrap();
        },
        "textbox",
    );
}

fn minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

fn make_picture_caption_core() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = core
        .insert_picture_native(
            0,
            0,
            0,
            &[],
            &minimal_png(),
            20_000,
            10_000,
            1,
            1,
            "png",
            "captioned picture",
            None,
            None,
        )
        .expect("insert picture");
    let para_idx = parse_usize(&created, "paraIdx");
    let control_idx = parse_usize(&created, "controlIdx");

    {
        let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
        let Control::Picture(picture) = &mut para.controls[control_idx] else {
            panic!("created control is not a picture");
        };
        let mut caption = Caption {
            direction: CaptionDirection::Bottom,
            max_width: 50_000,
            width: 50_000,
            paragraphs: vec![Paragraph::new_empty()],
            ..Default::default()
        };
        seed_hancom_geometry(&mut caption.paragraphs[0], TEXT);
        picture.caption = Some(caption);
    }

    refresh_from_ir(&mut core);
    (core, para_idx, control_idx)
}

fn picture_caption_para(core: &DocumentCore, para_idx: usize, control_idx: usize) -> &Paragraph {
    let para = &core.document().sections[0].paragraphs[para_idx];
    let Control::Picture(picture) = &para.controls[control_idx] else {
        panic!("control is not a picture");
    };
    &picture
        .caption
        .as_ref()
        .expect("picture caption")
        .paragraphs[0]
}

#[test]
fn picture_caption_tail_insert_delete_restores_geometry() {
    let (mut core, para_idx, control_idx) = make_picture_caption_core();
    let before_text = picture_caption_para(&core, para_idx, control_idx)
        .text
        .clone();
    let before_geometry = line_seg_snapshot(picture_caption_para(&core, para_idx, control_idx));

    assert_tail_edit_round_trip(
        &mut core,
        |core| line_seg_snapshot(picture_caption_para(core, para_idx, control_idx)),
        |core| {
            picture_caption_para(core, para_idx, control_idx)
                .text
                .clone()
        },
        |core, tail| {
            core.insert_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, "123")
                .unwrap();
        },
        "picture caption",
    );
    assert_delete_tail_restores(
        &mut core,
        |core| line_seg_snapshot(picture_caption_para(core, para_idx, control_idx)),
        |core| {
            picture_caption_para(core, para_idx, control_idx)
                .text
                .clone()
        },
        &before_text,
        &before_geometry,
        |core, tail| {
            core.delete_text_in_cell_native(0, para_idx, control_idx, 0, 0, tail, 3)
                .unwrap();
        },
        "picture caption",
    );
}

#[derive(Debug, Clone, Copy)]
enum NoteKind {
    Footnote,
    Endnote,
}

fn make_note_core(kind: NoteKind, alignment: Alignment) -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    let created = match kind {
        NoteKind::Footnote => core
            .insert_footnote_native(0, 0, 0)
            .expect("insert footnote"),
        NoteKind::Endnote => core.insert_endnote_native(0, 0, 0).expect("insert endnote"),
    };
    let control_idx = parse_usize(&created, "controlIdx");
    let para_shape_id = para_shape_id_with_alignment(&mut core, alignment);

    {
        let para = &mut core.document_mut().sections[0].paragraphs[0];
        match &mut para.controls[control_idx] {
            Control::Footnote(note) => {
                note.paragraphs[0].para_shape_id = para_shape_id;
                seed_hancom_geometry(&mut note.paragraphs[0], TEXT);
            }
            Control::Endnote(note) => {
                note.paragraphs[0].para_shape_id = para_shape_id;
                seed_hancom_geometry(&mut note.paragraphs[0], TEXT);
            }
            _ => panic!("created control is not a note"),
        }
    }

    refresh_from_ir(&mut core);
    (core, control_idx)
}

fn note_para(core: &DocumentCore, control_idx: usize) -> &Paragraph {
    let para = &core.document().sections[0].paragraphs[0];
    match &para.controls[control_idx] {
        Control::Footnote(note) => &note.paragraphs[0],
        Control::Endnote(note) => &note.paragraphs[0],
        _ => panic!("control is not a note"),
    }
}

#[test]
fn footnote_and_endnote_tail_insert_delete_restore_geometry_for_alignments() {
    for kind in [NoteKind::Footnote, NoteKind::Endnote] {
        for (alignment, name) in alignment_cases() {
            let (mut core, control_idx) = make_note_core(kind, alignment);
            let before_text = note_para(&core, control_idx).text.clone();
            let before_geometry = line_seg_snapshot(note_para(&core, control_idx));
            let context = format!("{kind:?} {name}");

            assert_tail_edit_round_trip(
                &mut core,
                |core| line_seg_snapshot(note_para(core, control_idx)),
                |core| note_para(core, control_idx).text.clone(),
                |core, tail| {
                    core.insert_text_in_footnote_native(0, 0, control_idx, 0, tail, "123")
                        .unwrap();
                },
                &context,
            );
            assert_delete_tail_restores(
                &mut core,
                |core| line_seg_snapshot(note_para(core, control_idx)),
                |core| note_para(core, control_idx).text.clone(),
                &before_text,
                &before_geometry,
                |core, tail| {
                    core.delete_text_in_footnote_native(0, 0, control_idx, 0, tail, 3)
                        .unwrap();
                },
                &context,
            );
        }
    }
}
