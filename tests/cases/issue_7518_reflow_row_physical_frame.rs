//! Maintainer correction for #7518: final continuation geometry, not helper output.
//! The original HWP declares row 5 as 22194 HU (295.92px). Its independent
//! Hancom 2020 PDF preserves that height across pages and centers the remaining
//! text/table together. On p2, the first line and nested table start at 83.01
//! and 128.34px: their 45.33px separation includes two 12.8px line slots,
//! the 1203 HU paragraph offset and the 283 HU outer top margin.
//! Synthetic variants below have independently converted Hancom 2020 PDFs in
//! pdf/pr7518_review. Short-paper controls use the valid_orientation inputs;
//! the contradictory original portrait inputs and their PDFs remain preserved.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/issue7500/picture_band_cell_spaces.hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned())
}

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7518-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rendered_pages(input: &Path) -> Vec<Value> {
    let dir = TempDir::new();
    let output = Command::new(rhwp_bin())
        .args(["export-render-tree"])
        .arg(input)
        .arg("-o")
        .arg(&dir.0)
        .output()
        .expect("run actual renderer");
    assert!(output.status.success(), "{output:?}");
    let mut paths: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

fn original_pages() -> Vec<Value> {
    rendered_pages(&Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
}

// Original NO_LS cells have no saved frame to invalidate. Complete-row
// reservation still has to own the measured frame painted around their lines.
#[test]
fn original_no_ls_rows_fit_the_body_and_preserve_the_following_blank_line() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1891/80168_regulatory_analysis.hwpx");
    let source = rhwp::parse_document(&std::fs::read(&input).unwrap()).unwrap();
    let table = source_table(&source.sections[0].paragraphs[1232]);
    assert!(table
        .cells
        .iter()
        .all(|cell| cell.paragraphs.iter().all(|para| {
            para.line_segs.is_empty() || para.line_segs.iter().all(|seg| seg.tag & 0x8000_0000 != 0)
        })));
    let pages = rendered_pages(&input);
    let mut left_cell_text = std::collections::BTreeMap::<u64, String>::new();
    let mut saw_following_line = false;
    for page in &pages {
        let all = nodes(page);
        let Some(column) = all.iter().find(|n| n["type"] == "Column") else {
            continue;
        };
        for owner in all
            .iter()
            .filter(|n| n["type"] == "Table" && n["pi"] == 1232)
        {
            assert!(
                bottom(owner) <= bottom(column) + 0.2,
                "complete NO_LS rows exceed the body: owner {}/{}",
                bottom(owner),
                bottom(column)
            );
            for cell in owner["children"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["type"] == "Cell" && n["col"] == 0)
            {
                left_cell_text
                    .entry(cell["row"].as_u64().unwrap())
                    .or_default()
                    .push_str(
                        &nodes(cell)
                            .into_iter()
                            .filter(|n| n["type"] == "TextRun")
                            .map(|n| n["text"].as_str().unwrap())
                            .collect::<String>(),
                    );
            }
            if let Some(line) = all
                .iter()
                .find(|n| n["type"] == "TextLine" && n["pi"] == 1233)
            {
                // render tree JSON 좌표는 0.1px 로 반올림된다. 표와 뒤 줄이 같은 양만큼
                // 함께 움직여도(#7685: 이어지는 조각 바깥 위 여백 141HU) 두 반올림의 차가
                // 0.1px 바뀔 수 있으므로 기존 0.2px 허용치에 그 해상도를 더한다.
                assert!(
                    coord(line, "y") >= bottom(owner) - 0.3,
                    "following intentional blank line must remain after the final table"
                );
                saw_following_line = true;
            }
        }
    }
    let expected: std::collections::BTreeMap<_, _> = table
        .cells
        .iter()
        .filter(|cell| cell.col == 0)
        .map(|cell| {
            (
                u64::from(cell.row),
                cell.paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
            )
        })
        .collect();
    assert_eq!(
        left_cell_text, expected,
        "every original row label exactly once"
    );
    assert!(saw_following_line, "following blank paragraph is preserved");
}

// The independent Hancom 2024 p22 uses Palatino Linotype for U+00B7:
// the dot closes line two of the original paragraph; line three begins "울".
// The original HFT one-dot leader remains covered separately by issue_5906.
#[test]
fn regulatory_22_legacy_latin_dot_preserves_the_source_word_boundary() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let source = rhwp::parse_document(&std::fs::read(&input).unwrap()).unwrap();
    assert!(source.sections[0].paragraphs[180].text.contains("덮개·울"));
    let page = &rendered_pages(&input)[21];
    let lines: Vec<_> = nodes(page)
        .into_iter()
        .filter(|node| node["type"] == "TextLine" && node["pi"] == 180)
        .map(|line| {
            nodes(line)
                .into_iter()
                .filter(|node| node["type"] == "TextRun")
                .map(|node| node["text"].as_str().unwrap())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines.len(), 3);
    assert!(
        lines[1].ends_with("덮개·"),
        "dot belongs to its source word: {lines:?}"
    );
    assert!(
        lines[2].starts_with("울"),
        "independent PDF line membership: {lines:?}"
    );
    let quoted_lines: Vec<_> = nodes(page)
        .into_iter()
        .filter(|node| node["type"] == "TextLine" && node["pi"] == 181)
        .map(|line| {
            nodes(line)
                .into_iter()
                .filter(|node| node["type"] == "TextRun")
                .map(|node| node["text"].as_str().unwrap())
                .collect::<String>()
        })
        .collect();
    assert_eq!(quoted_lines.len(), 3);
    assert!(
        quoted_lines[0].ends_with("사고"),
        "legacy Latin quotes preserve the PDF word boundary: {quoted_lines:?}"
    );
    assert!(quoted_lines[1].trim_end().ends_with("하다가"));
    assert!(quoted_lines[2].starts_with("회전날"));
}

fn regulatory_page39() -> (rhwp::model::document::Document, Value) {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    (source, rendered_pages(&input)[38].clone())
}

fn source_table(para: &rhwp::model::paragraph::Paragraph) -> &rhwp::model::table::Table {
    match para.controls.as_slice() {
        [rhwp::model::control::Control::Table(table)] => table,
        _ => panic!("one source table"),
    }
}

fn body_node<'a>(page: &'a Value, kind: &str, pi: usize) -> &'a Value {
    nodes(page)
        .into_iter()
        .find(|node| node["type"] == kind && node["pi"] == pi)
        .expect("source paragraph belongs to page 39")
}

// Independent Hancom p39 preserves the positive paragraph offset, the
// wrapper's 9062 HU minimum and two intentional blank paragraph line slots.
// Derive the relationships from the original source, rather than pinning y.
#[test]
fn regulatory_39_wrapper_offset_and_physical_tail_advance_the_following_flow() {
    let (source, page) = regulatory_page39();
    let paras = &source.sections[0].paragraphs;
    let wrapper = source_table(&paras[370]);
    let nested = source_table(&wrapper.cells[0].paragraphs[0]);
    assert!(paras[370].text.is_empty() && paras[370].line_segs.is_empty());
    assert!(wrapper.common.vertical_offset > 0);
    let heading = &paras[369];
    let style = &source.doc_info.para_shapes[heading.para_shape_id as usize];
    assert_eq!(
        style.line_spacing_type,
        rhwp::model::style::LineSpacingType::Percent
    );
    let em = f64::from(
        source.doc_info.char_shapes[heading.char_shapes[0].char_shape_id as usize].base_size,
    ) / 75.0;
    let padding = wrapper.cells[0].effective_padding(&wrapper.padding);
    let child_top = f64::from(padding.top) / 75.0 + f64::from(nested.outer_margin_top) / 75.0;
    let expected_top = coord(body_node(&page, "TextLine", 369), "y")
        + em * f64::from(style.line_spacing) / 100.0
        + f64::from(style.spacing_after) / 75.0
        + f64::from(wrapper.outer_margin_top) / 75.0
        + f64::from(wrapper.common.vertical_offset) / 75.0
        + child_top;
    let placed = body_node(&page, "Table", 370);
    assert!(
        (coord(placed, "y") - expected_top).abs() < 0.2,
        "the wrapper's offset and top margins must be consumed once"
    );
    let physical_bottom = coord(placed, "y") - child_top
        + f64::from(wrapper.common.height) / 75.0
        + f64::from(wrapper.outer_margin_bottom) / 75.0;
    assert!(
        coord(body_node(&page, "TextLine", 371), "y") + 0.2 >= physical_bottom,
        "the blank paragraph must follow the whole wrapper, including its empty physical tail"
    );
    for (previous, next) in [(371, 372), (372, 373)] {
        let para = &paras[previous];
        assert!(para.text.is_empty() && para.controls.is_empty());
        let style = &source.doc_info.para_shapes[para.para_shape_id as usize];
        assert_eq!(
            style.line_spacing_type,
            rhwp::model::style::LineSpacingType::Percent
        );
        let em = f64::from(
            source.doc_info.char_shapes[para.char_shapes[0].char_shape_id as usize].base_size,
        ) / 75.0;
        let pitch = em * f64::from(style.line_spacing) / 100.0;
        let gap = coord(body_node(&page, "TextLine", next), "y")
            - coord(body_node(&page, "TextLine", previous), "y");
        assert!(
            (gap - pitch).abs() < 0.2,
            "intentional blank line slot {previous}"
        );
    }
}

#[test]
fn regulatory_39_table_follows_its_title_and_blank_paragraph() {
    let (source, page) = regulatory_page39();
    assert!(source.sections[0].paragraphs[373]
        .text
        .contains("이해관계자 의견수렴"));
    let spacer = &source.sections[0].paragraphs[374];
    assert!(spacer.text.is_empty() && spacer.controls.is_empty());
    let title = body_node(&page, "TextLine", 373);
    let blank = body_node(&page, "TextLine", 374);
    let table = body_node(&page, "Table", 375);
    let tail = body_node(&page, "TextLine", 376);
    assert!(bottom(title) <= coord(blank, "y"));
    assert!(
        bottom(blank) <= coord(table, "y"),
        "table overlaps its preceding blank paragraph/title"
    );
    assert!(
        bottom(table) <= coord(tail, "y"),
        "following paragraph overlaps the table"
    );
}

// The original cell declares 510 HU padding on both sides. Hancom's matching
// PDF and editor screenshot show four lines, with the final syllable '견'
// on the fourth line. Equality of height must not trigger padding reduction.
#[test]
fn regulatory_39_exact_fit_opinion_cell_keeps_padding_and_all_four_lines() {
    let (source, page) = regulatory_page39();
    let wrapper = source_table(&source.sections[0].paragraphs[375]);
    let table = source_table(&wrapper.cells[0].paragraphs[0]);
    let source_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == 1 && cell.col == 2)
        .unwrap();
    let padding = source_cell.effective_padding(&table.padding);
    let placed = cell(body_node(&page, "Table", 375), 1, 2);
    let lines: Vec<_> = nodes(placed)
        .into_iter()
        .filter(|node| node["type"] == "TextLine")
        .collect();
    assert_eq!(
        lines.len(),
        4,
        "an exact height fit must retain the four source-width lines"
    );
    for line in &lines {
        assert!(
            (coord(line, "x") - coord(placed, "x") - f64::from(padding.left) / 75.0).abs() < 0.2
        );
        let right = coord(line, "x") + coord(line, "w");
        assert!(
            (coord(placed, "x") + coord(placed, "w") - right - f64::from(padding.right) / 75.0)
                .abs()
                < 0.2
        );
        assert!(
            bottom(line) <= bottom(placed),
            "wrapped content escaped its cell"
        );
    }
    let rendered: String = nodes(placed)
        .into_iter()
        .filter(|node| node["type"] == "TextRun")
        .map(|node| node["text"].as_str().unwrap_or(""))
        .collect();
    let original: String = source_cell
        .paragraphs
        .iter()
        .map(|para| para.text.as_str())
        .collect();
    assert_eq!(
        rendered.split_whitespace().collect::<String>(),
        original.split_whitespace().collect::<String>(),
        "cell content must not be omitted or replayed"
    );
}

#[test]
fn a_deferred_picture_band_keeps_the_blank_cell_and_the_next_row_on_its_page() {
    for group in ["valid_orientation", "valid_generated"] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/pr7518_review_page_budget/{group}/nested-split.hwp"
        ));
        let pages = rendered_pages(&input);
        // Independent Hancom 2020 PDF: p2 blank cell rules at 326.97/420.48px;
        // p3 picture-cell rule at 304.63px and all three visible image tops at
        // 60.32px. The following timeline belongs to p3, before the nested table
        // starts on p4. These are visible PDF bounds, not un-clipped image strips.
        let blank = cell(outer(&pages[1]), 4, 1);
        assert!((coord(blank, "y") - 326.97).abs() < 2.0);
        assert!((bottom(blank) - 420.48).abs() < 2.0);
        assert!(!nodes(&pages[1]).iter().any(|n| n["type"] == "Image"));

        let pictures = cell(outer(&pages[2]), 4, 1);
        assert!((bottom(pictures) - 304.63).abs() < 2.0);
        // The independent PDF opens the table's outer frame at the body top,
        // then the cell's own inner margin before the pictures. Compare these
        // separate relationships to source properties: a loose absolute image
        // coordinate check alone cannot detect one missing 141 HU margin.
        let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
        let source_table = source.sections[0].paragraphs[3]
            .controls
            .iter()
            .find_map(|control| match control {
                rhwp::model::control::Control::Table(table) => Some(table),
                _ => None,
            })
            .unwrap();
        let source_cell = source_table
            .cells
            .iter()
            .find(|cell| cell.row == 4 && cell.col == 1)
            .unwrap();
        let column = nodes(&pages[2])
            .into_iter()
            .find(|node| node["type"] == "Column")
            .unwrap();
        let outer_top = f64::from(source_table.outer_margin_top) / 75.0;
        let inner_top = f64::from(source_cell.effective_padding(&source_table.padding).top) / 75.0;
        assert!(
            (coord(pictures, "y") - coord(column, "y") - outer_top).abs() < 0.11,
            "the deferred picture fragment must reopen its source outer top margin"
        );
        let images: Vec<_> = nodes(&pages[2])
            .into_iter()
            .filter(|n| n["type"] == "Image")
            .collect();
        assert_eq!(images.len(), 3);
        for image in &images {
            assert!((coord(image, "y") - 60.32).abs() < 2.0);
            assert!(
                (coord(image, "y") - coord(pictures, "y") - inner_top).abs() < 0.11,
                "pictures must retain the cell's source inner top margin"
            );
            assert!(bottom(image) <= bottom(pictures) + 0.5);
        }
        let next = cell(outer(&pages[2]), 5, 1);
        assert!((coord(next, "y") - bottom(pictures)).abs() < 0.15);
        let timeline_lines: Vec<_> = cell_owned_nodes(next)
            .into_iter()
            .filter(|n| n["type"] == "TextLine" && n["pi"].as_u64().is_some_and(|i| i < 9))
            .collect();
        assert_eq!(timeline_lines.len(), 9);
        assert!(timeline_lines
            .iter()
            .all(|line| bottom(line) <= bottom(next)));
        let continuation = cell(outer(&pages[3]), 5, 1);
        assert!(!cell_owned_nodes(continuation)
            .iter()
            .any(|n| n["type"] == "TextLine" && n["pi"].as_u64().is_some_and(|i| i < 9)));
        assert_eq!(
            pages
                .iter()
                .flat_map(nodes)
                .filter(|n| n["type"] == "Image")
                .count(),
            3
        );
    }
}

#[test]
fn trailing_empty_paragraphs_keep_their_continued_cell_outline() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_generated/nested-split.hwp");
    let pages = rendered_pages(&input);
    assert_eq!(
        pages.len(),
        6,
        "source empty lines must survive the last cut"
    );
    let tail = cell(outer(&pages[5]), 7, 1);
    // The independent PDF's p6 has only the blank cell fragment, from 58.40
    // to 89.84px. Source p12 owns 800+480 HU; reflowed p13 owns its 800 HU em;
    // both padding edges own 141 HU. No visible glyph is required for ownership.
    let source_height = (800.0 + 480.0 + 800.0 + 282.0) / 75.0;
    assert!((coord(tail, "h") - source_height).abs() < 0.2);
    assert!((coord(tail, "h") - (89.84 - 58.40)).abs() < 1.0);
    assert!(!nodes(&pages[5]).iter().any(|n| n["type"] == "Image"
        || n["type"] == "TextRun" && n["text"].as_str().is_some_and(|s| !s.trim().is_empty())));
}

#[test]
fn empty_reflow_table_hosts_consume_the_source_margins_once() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    let paras = &source.sections[0].paragraphs;
    let source_table = |pi: usize| {
        assert!(paras[pi].text.is_empty());
        assert!(paras[pi].line_segs.is_empty());
        assert_eq!(paras[pi].controls.len(), 1);
        match &paras[pi].controls[0] {
            rhwp::model::control::Control::Table(table) => table.as_ref(),
            _ => panic!("empty table host"),
        }
    };
    let pages = rendered_pages(&input);
    let all = nodes(&pages[32]);
    let placed = |pi: usize| {
        all.iter()
            .copied()
            .find(|node| node["type"] == "Table" && node["pi"] == pi)
            .expect("table belongs to page 33")
    };
    let preceding = all
        .iter()
        .copied()
        .find(|node| node["type"] == "TextLine" && node["pi"] == 322)
        .expect("preceding subtitle");
    let subtitle = &paras[322];
    let style = &source.doc_info.para_shapes[subtitle.para_shape_id as usize];
    assert_eq!(
        style.line_spacing_type,
        rhwp::model::style::LineSpacingType::Percent
    );
    let em = f64::from(
        source.doc_info.char_shapes[subtitle.char_shapes[0].char_shape_id as usize].base_size,
    ) / 75.0;
    // Independent Hancom p33 opens each table's outer box after the preceding
    // flow, including the subtitle's percentage line slot. Check the ownership
    // relationships from source properties rather than pinning absolute pixels.
    let expected_first = coord(preceding, "y")
        + em * f64::from(style.line_spacing) / 100.0
        + f64::from(source_table(323).outer_margin_top) / 75.0;
    assert!(
        (coord(placed(323), "y") - expected_first).abs() < 0.2,
        "the first empty host discarded its reserved top margin"
    );
    for (previous, next) in [(323, 324), (324, 325)] {
        let gap = coord(placed(next), "y") - bottom(placed(previous));
        let margins = f64::from(source_table(previous).outer_margin_bottom)
            + f64::from(source_table(next).outer_margin_top);
        assert!(
            (gap - margins / 75.0).abs() < 0.2,
            "empty hosts must consume adjacent source margins once: {previous} -> {next}"
        );
    }
    assert!(!all
        .iter()
        .any(|node| node["type"] == "TextLine" && node["pi"] == 325));
}

#[test]
fn continuation_at_a_reflow_paragraph_start_consumes_its_reserved_spacing() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let pages = rendered_pages(&input);
    let find_rationale = |page: &Value| {
        nodes(page)
            .into_iter()
            .find(|node| node["type"] == "Table" && node["pi"] == 325)
            .expect("rationale owner")
            .clone()
    };
    // The original source, independently rendered in the Hancom 2024 PDF,
    // starts child paragraph 9 on p34. It owns spacing_before=1000 HU and
    // starts at line zero. Paragraph spacing uses 1/14400 inch units (6.667px),
    // while the child top padding uses 1/7200 inch units (141 HU = 1.88px).
    // The preceding page starts at child paragraph 0, which owns no before-space.
    for (page_index, para_index, before_hu) in [(32, 0, 0.0), (33, 9, 1000.0)] {
        let owner = find_rationale(&pages[page_index]);
        let child = nested(cell(&owner, 6, 1));
        let line = nodes(child)
            .into_iter()
            .find(|node| node["type"] == "TextLine" && node["pi"] == para_index)
            .expect("first source line of the fragment");
        assert!(
            (coord(line, "y") - coord(child, "y") - 141.0 / 75.0 - before_hu / 150.0).abs() < 0.2,
            "the first source unit reserved spacing which paint discarded: page {}, delta {}",
            page_index + 1,
            coord(line, "y") - coord(child, "y")
        );
    }
}

#[test]
fn a_reflow_cell_start_consumes_the_source_paragraph_before_space() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let pages = rendered_pages(&input);
    let owner = nodes(&pages[33])
        .into_iter()
        .find(|n| n["type"] == "Table" && n["pi"] == 336)
        .unwrap();
    let child = nested(cell(owner, 4, 1));
    let line = nodes(child)
        .into_iter()
        .find(|n| n["type"] == "TextLine")
        .unwrap();
    // The original source declares 2000 HU before-space, independently retained
    // by the fresh Hancom 2024 output. No saved line frame replaces this lead.
    assert!(
        (coord(line, "y") - coord(child, "y") - (141.0 / 75.0 + 2000.0 / 150.0)).abs() < 0.2,
        "first source paragraph before-space was dropped"
    );
    assert!(bottom(line) <= bottom(child));
}

#[test]
fn a_block_table_host_does_not_duplicate_its_line_but_keeps_the_following_empty_paragraph() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let pages = rendered_pages(&input);
    let owner = nodes(&pages[33])
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 336)
        .expect("direct-benefit table");
    let host = cell(owner, 4, 1);
    let child = nested(host);
    // Fresh Hancom 2024 conversion of the exact original source has horizontal
    // rules at 604.7787px and 752.4571px. The source host owns Table + ColumnDef,
    // then a genuinely empty 13pt paragraph. Its column definition adds no line.
    assert!(
        (coord(host, "h") - (752.4571 - 604.7787)).abs() < 1.0,
        "host line or cached object size duplicated the flow: {}",
        coord(host, "h")
    );
    // Preserve the distinct empty line (13pt em) and both outer-cell paddings
    // (223 HU each), independently of the nested table's measured body height.
    assert!(
        (coord(host, "h") - coord(child, "h") - 13.0 * 96.0 / 72.0 - 446.0 / 75.0).abs() < 1.0,
        "following intentional empty paragraph was lost"
    );
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut all = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            all.extend(nodes(child));
        }
    }
    all
}

fn cell_owned_nodes(node: &Value) -> Vec<&Value> {
    if node["type"] == "Table" {
        return Vec::new();
    }
    let mut all = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            all.extend(cell_owned_nodes(child));
        }
    }
    all
}

fn outer(page: &Value) -> &Value {
    nodes(page)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 3)
        .expect("original outer table")
}

fn cell(table: &Value, row: u32, col: u32) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == row && node["col"] == col)
        .expect("outer cell")
}

fn coord(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    coord(node, "y") + coord(node, "h")
}

fn nested(owner: &Value) -> &Value {
    nodes(owner)
        .into_iter()
        .find(|node| node["type"] == "Table")
        .expect("nested table")
}

fn first_line(owner: &Value) -> &Value {
    nodes(owner)
        .into_iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == 8)
        .expect("remaining timeline line")
}

#[test]
fn continuation_preserves_declared_row_minimum_and_following_rows() {
    let pages = original_pages();
    assert_eq!(pages.len(), 2);
    let first = cell(outer(&pages[0]), 5, 1);
    let last = cell(outer(&pages[1]), 5, 1);
    let total = coord(first, "h") + coord(last, "h");
    assert!(
        (total - 22194.0 / 75.0).abs() < 0.15,
        "declared row lost: {total}"
    );
    let following = cell(outer(&pages[1]), 6, 1);
    let final_cell = cell(outer(&pages[1]), 7, 1);
    assert!((coord(following, "y") - bottom(last)).abs() < 0.15);
    assert!((coord(final_cell, "y") - bottom(following)).abs() < 0.15);
    assert!((bottom(outer(&pages[1])) - bottom(final_cell) - 1.0).abs() < 0.2);
}

#[test]
fn remaining_text_and_nested_table_share_the_centered_occupancy() {
    let pages = original_pages();
    let owner = cell(outer(&pages[1]), 5, 1);
    let line = first_line(owner);
    let table = nested(owner);
    // Alignment invariant includes inner padding and both nested outer margins.
    let above = coord(line, "y") - (coord(owner, "y") + 141.0 / 75.0);
    let below = bottom(owner) - 141.0 / 75.0 - (bottom(table) + 283.0 / 75.0);
    assert!(
        (above - below).abs() < 0.2,
        "unequal centered occupancy: {above} / {below}"
    );
    assert!((coord(table, "y") - coord(line, "y") - 45.33).abs() < 0.3);
}

#[test]
fn reflow_nested_table_keeps_its_column_anchor_and_paragraph_offset() {
    let pages = original_pages();
    let owner = cell(outer(&pages[1]), 5, 1);
    let table = nested(owner);
    let expected_x = coord(owner, "x") + (141.0 + 283.0 + 341.0) / 75.0;
    assert!(
        (coord(table, "x") - expected_x).abs() < 0.15,
        "lost LEFT/Column anchor"
    );
    assert!(
        (coord(table, "y") - 128.34).abs() < 2.0,
        "independent Hancom p2 table top"
    );
    assert!(bottom(table) + 283.0 / 75.0 <= bottom(owner) - 141.0 / 75.0);
}

#[test]
fn physical_tail_exceeding_page_budget_finishes_without_replaying_content() {
    use rhwp::model::control::Control;
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut doc = rhwp::parser::parse_hwp(&std::fs::read(input).unwrap()).unwrap();
    let declared_hu = 270000;
    for section in &mut doc.sections {
        section.raw_stream = None;
        section.raw_provenance = None;
        for control in &mut section.paragraphs[3].controls {
            if let Control::Table(table) = control {
                for cell in &mut table.cells {
                    if cell.row == 5 {
                        cell.height = declared_hu;
                    }
                }
            }
        }
    }
    let temp = TempDir::new();
    let input = temp.0.join("declared-tail.hwp");
    std::fs::write(&input, rhwp::serializer::serialize_document(&doc).unwrap()).unwrap();
    let pages = rendered_pages(&input);
    assert!(
        pages.len() >= 4 && pages.len() < 10,
        "finite physical continuation"
    );
    let mut height = 0.0;
    let mut nested_count = 0;
    let mut final_row_count = 0;
    for page in &pages {
        let table = outer(page);
        let column = nodes(page)
            .into_iter()
            .find(|n| n["type"] == "Column")
            .unwrap();
        for node in table["children"].as_array().unwrap() {
            if node["type"] == "Cell" && node["row"] == 5 && node["col"] == 1 {
                height += coord(node, "h");
                assert!(
                    bottom(node) <= bottom(column) + 0.2,
                    "physical tail exceeds budget"
                );
                nested_count += nodes(node).iter().filter(|n| n["type"] == "Table").count();
            }
            if node["type"] == "Cell" && node["row"] == 7 && node["col"] == 1 {
                final_row_count += 1;
            }
        }
    }
    assert!(
        (height - declared_hu as f64 / 75.0).abs() < 0.3,
        "physical minimum {height}"
    );
    assert_eq!(
        nested_count, 1,
        "already consumed nested table must not replay"
    );
    assert_eq!(
        final_row_count, 1,
        "following final row must survive exactly once"
    );
}

#[test]
fn unsplit_reflow_uses_the_same_nested_anchor_and_flow_lead() {
    use rhwp::model::control::Control;
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut doc = rhwp::parser::parse_hwp(&std::fs::read(input).unwrap()).unwrap();
    for section in &mut doc.sections {
        section.raw_stream = None;
        section.raw_provenance = None;
        section.section_def.page_def.height = 225000;
        for para in &mut section.paragraphs {
            for control in &mut para.controls {
                if let Control::SectionDef(def) = control {
                    def.page_def.height = 225000;
                }
            }
        }
    }
    let temp = TempDir::new();
    let input = temp.0.join("unsplit.hwp");
    std::fs::write(&input, rhwp::serializer::serialize_document(&doc).unwrap()).unwrap();
    let pages = rendered_pages(&input);
    assert_eq!(pages.len(), 1);
    let owner = cell(outer(&pages[0]), 5, 1);
    let table = nested(owner);
    let line = nodes(owner)
        .into_iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 0)
        .unwrap();
    assert!((coord(table, "x") - coord(owner, "x") - (141.0 + 283.0 + 341.0) / 75.0).abs() < 0.15);
    // Nine timeline paragraphs plus their following empty paragraph own ten slots.
    let delta = 10.0 * 12.8 + (1203.0 + 283.0) / 75.0;
    assert!(
        (coord(table, "y") - coord(line, "y") - delta).abs() < 0.3,
        "full/cut flow mismatch"
    );
    assert!(bottom(table) + 283.0 / 75.0 <= bottom(owner) - 141.0 / 75.0);
}

#[test]
fn split_nested_rows_reserve_the_lead_once_and_stay_inside_each_page() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_orientation/nested-split.hwp");
    let pages = rendered_pages(&input);
    let mut first = true;
    let mut owned_cells = std::collections::BTreeSet::new();
    let mut fragments = 0;
    for page in &pages {
        let outer = outer(page);
        let column = nodes(page)
            .into_iter()
            .find(|n| n["type"] == "Column")
            .unwrap();
        for owner in outer["children"].as_array().unwrap() {
            if owner["type"] != "Cell" || owner["row"] != 5 || owner["col"] != 1 {
                continue;
            }
            assert!(
                bottom(owner) <= bottom(column) + 0.2,
                "remaining content was counted as a physical tail"
            );
            for table in nodes(owner).into_iter().filter(|n| n["type"] == "Table") {
                fragments += 1;
                let delta = coord(table, "y") - coord(owner, "y") - 141.0 / 75.0;
                let expected = if first { (1203.0 + 283.0) / 75.0 } else { 0.0 };
                assert!(
                    (delta - expected).abs() < 0.2,
                    "lead was lost or repeated: {delta}"
                );
                first = false;
                for child in table["children"].as_array().unwrap() {
                    if child["type"] == "Cell" {
                        let key = (
                            child["row"].as_u64().unwrap(),
                            child["col"].as_u64().unwrap(),
                        );
                        assert!(owned_cells.insert(key), "nested row/cell replayed: {key:?}");
                        assert!((coord(child, "h") - 9000.0 / 75.0).abs() < 0.15);
                    }
                }
            }
        }
    }
    assert!(fragments >= 2, "exercise a cut inside the nested table");
    assert_eq!(
        owned_cells.len(),
        20,
        "all four rows of five cells must survive"
    );
}

#[test]
fn a_complete_content_cut_keeps_center_alignment_before_the_physical_tail() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/terminal-tail.hwp");
    let pages = rendered_pages(&input);
    assert_eq!(pages.len(), 3);
    let owner = cell(outer(&pages[1]), 5, 1);
    let child = nested(owner);
    // The independently converted Hancom 2020 PDF places the child top at
    // 537.17px on p2. Its complete remaining content is centered in this frame;
    // p3 owns physical space only, so an end-cut ledger is not a content cut.
    assert!(
        (coord(child, "y") - 537.17).abs() < 2.0,
        "complete content incorrectly kept the top origin: {}",
        coord(child, "y")
    );
    let line = first_line(owner);
    let above = coord(line, "y") - coord(owner, "y") - 141.0 / 75.0;
    let below = bottom(owner) - 141.0 / 75.0 - bottom(child) - 283.0 / 75.0;
    assert!(
        (above - below).abs() < 0.2,
        "remaining occupancy lost alignment"
    );
    assert!(nodes(cell(outer(&pages[2]), 5, 1))
        .iter()
        .all(|n| n["type"] != "TextRun" && n["type"] != "Table"));
}

#[test]
fn terminal_physical_tail_is_drawn_after_the_last_content_unit() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_generated/terminal-follower.hwp");
    let pages = rendered_pages(&input);
    let mut height = 0.0;
    for page in &pages {
        // An exhausted content cursor must not leave an empty final page.
        let table = outer(page);
        for owner in table["children"].as_array().unwrap() {
            if owner["type"] == "Cell" && owner["row"] == 5 && owner["col"] == 1 {
                height += coord(owner, "h");
            }
        }
    }
    assert_eq!(pages.len(), 3, "two full frames and a finite physical tail");
    assert!(
        (height - 92165.0 / 75.0).abs() < 0.3,
        "terminal minimum was discarded: {height}"
    );
    let last = cell(outer(pages.last().unwrap()), 5, 1);
    assert!(
        coord(last, "h") > 0.5 && coord(last, "h") < 25.0,
        "exercise the old sliver guard"
    );
    // Hancom's PDF splits the nested table across p1/p2. Multiple table
    // fragments are legal; replay means duplicated source cell content.
    assert_terminal_nested_contents(&input, &pages);
    assert!(
        nodes(last)
            .iter()
            .all(|n| n["type"] != "TextRun" && n["type"] != "Table"),
        "the last frame owns space, not content"
    );
    let last_page = pages.last().unwrap();
    let following = nodes(last_page)
        .into_iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == 4)
        .expect("following paragraph on the final frame page");
    assert!(
        coord(following, "y") + 0.15 >= bottom(last),
        "following paragraph overlaps physical tail"
    );
    let follower_count = pages
        .iter()
        .flat_map(nodes)
        .filter(|node| node["type"] == "TextRun" && node["text"] == "PHYSICAL TAIL FOLLOWER")
        .count();
    assert_eq!(
        follower_count, 1,
        "following text must survive exactly once"
    );
}

fn assert_terminal_nested_contents(input: &Path, pages: &[Value]) {
    use std::collections::BTreeMap;
    let source = rhwp::parser::parse_hwp(&std::fs::read(input).unwrap()).unwrap();
    let table = source_table(&source.sections[0].paragraphs[3]);
    let owner = table
        .cells
        .iter()
        .find(|cell| cell.row == 5 && cell.col == 1)
        .expect("source owner cell");
    let child = owner
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .find_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("source nested table");
    let normalize = |text: String| {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let expected: BTreeMap<_, _> = child
        .cells
        .iter()
        .map(|cell| {
            let text = cell
                .paragraphs
                .iter()
                .map(|para| para.text.as_str())
                .collect();
            ((u64::from(cell.row), u64::from(cell.col)), normalize(text))
        })
        .collect();
    let mut actual = BTreeMap::<(u64, u64), String>::new();
    for page in pages {
        let owner = cell(outer(page), 5, 1);
        for table in nodes(owner).into_iter().filter(|n| n["type"] == "Table") {
            for cell in table["children"].as_array().unwrap() {
                if cell["type"] != "Cell" {
                    continue;
                }
                let id = (cell["row"].as_u64().unwrap(), cell["col"].as_u64().unwrap());
                let text = actual.entry(id).or_default();
                for run in nodes(cell).into_iter().filter(|n| n["type"] == "TextRun") {
                    text.push_str(run["text"].as_str().unwrap());
                }
            }
        }
    }
    let actual: BTreeMap<_, _> = actual
        .into_iter()
        .map(|(id, text)| (id, normalize(text)))
        .collect();
    assert_eq!(
        actual, expected,
        "nested cell ownership/content must survive exactly once"
    );
}

#[test]
fn terminal_cell_ownership_rejects_replayed_or_missing_content() {
    fn first_child_table(node: &mut Value) -> Option<&mut Value> {
        if node["type"] == "Table" && node["pi"] == 10 {
            return Some(node);
        }
        node["children"]
            .as_array_mut()?
            .iter_mut()
            .find_map(first_child_table)
    }
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_generated/terminal-follower.hwp");
    let pages = rendered_pages(&input);
    assert_terminal_nested_contents(&input, &pages);
    for replay in [false, true] {
        let mut changed = pages.clone();
        let children = first_child_table(&mut changed[0]).expect("first nested fragment")
            ["children"]
            .as_array_mut()
            .unwrap();
        let first_cell = children
            .iter()
            .position(|n| {
                n["type"] == "Cell"
                    && nodes(n).iter().any(|run| {
                        run["type"] == "TextRun" && !run["text"].as_str().unwrap().trim().is_empty()
                    })
            })
            .expect("a source cell with nonempty content");
        if replay {
            children.push(children[first_cell].clone());
        } else {
            children.remove(first_cell);
        }
        assert!(
            std::panic::catch_unwind(|| assert_terminal_nested_contents(&input, &changed)).is_err(),
            "source content check must reject replay={replay}"
        );
    }
}

#[test]
fn independently_regenerated_reflow_context_preserves_units_and_following_rows() {
    for (name, expected_pages, child_cells) in [
        ("nested-split", 6, 20),
        ("nested-auto-row", 8, 2),
        ("nested-mixed-cell", 8, 1),
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/pr7518_review_page_budget/valid_generated/{name}.hwp"
        ));
        let pages = rendered_pages(&input);
        // Independent Hancom 2020 conversions of the exact regenerated inputs
        // produce 6/8/8 pages. The reflow target's recursive source lines were
        // cleared; the saved surrounding frame comes from Hancom, not a manual ladder.
        assert_eq!(pages.len(), expected_pages, "{name}: page budget");
        let all: Vec<_> = pages.iter().flat_map(nodes).collect();
        let mut identities = std::collections::BTreeSet::new();
        let mut child_count = 0;
        for page in &pages {
            let column = nodes(page)
                .into_iter()
                .find(|n| n["type"] == "Column")
                .unwrap();
            for owner in outer(page)["children"].as_array().unwrap() {
                if owner["type"] == "Cell" && owner["row"] == 5 && owner["col"] == 1 {
                    assert!(
                        bottom(owner) <= bottom(column) + 0.2,
                        "{name}: physical frame overflow"
                    );
                    for child in nodes(owner)
                        .into_iter()
                        .filter(|n| n["type"] == "Cell" && *n != owner)
                    {
                        identities.insert((
                            child["row"].as_u64().unwrap(),
                            child["col"].as_u64().unwrap(),
                        ));
                        child_count += 1;
                    }
                }
            }
        }
        assert_eq!(identities.len(), child_cells, "{name}: child ownership");
        if name == "nested-split" {
            assert_eq!(child_count, 20, "no child replay");
        } else {
            assert_recursive_child_page_owners(&pages);
            for i in 0..65 {
                let expected = format!("UNIT {i:03}");
                assert_eq!(
                    all.iter()
                        .filter(|n| n["type"] == "TextRun" && n["text"] == expected)
                        .count(),
                    1,
                    "{name}: {expected}"
                );
            }
        }
        let last = pages.last().unwrap();
        let tail = cell(outer(last), 7, 1);
        assert!(
            coord(tail, "h") > 0.5,
            "{name}: following final row discarded"
        );
        assert!(
            bottom(tail)
                <= bottom(
                    nodes(last)
                        .into_iter()
                        .find(|n| n["type"] == "Column")
                        .unwrap()
                ) + 0.2
        );
    }
}

fn assert_recursive_reflow_child(input: &str, expected_final_child_rows: usize) {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_orientation")
        .join(input);
    let pages = rendered_pages(&input);
    assert_recursive_child_page_owners(&pages);
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    let source_outer = source_table(&source.sections[0].paragraphs[3]);
    let source_owner = source_outer
        .cells
        .iter()
        .find(|cell| cell.row == 5 && cell.col == 1)
        .unwrap();
    let source_child = source_table(source_owner.paragraphs.last().unwrap());
    let normalize = |text: &str| text.split_whitespace().collect::<String>();
    let expected_following: std::collections::BTreeMap<_, _> = source_outer
        .cells
        .iter()
        .filter(|cell| cell.row == 7)
        .map(|cell| {
            (
                u64::from(cell.col),
                normalize(
                    &cell
                        .paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>(),
                ),
            )
        })
        .collect();
    let mut first = true;
    let mut fragments = 0;
    let mut source_units = std::collections::BTreeSet::new();
    let mut final_child_rows = 0;
    let mut following_content = std::collections::BTreeMap::<u64, String>::new();
    for page in &pages {
        let all = nodes(page);
        let column = all.iter().find(|n| n["type"] == "Column").unwrap();
        for owner in outer(page)["children"].as_array().unwrap() {
            if owner["type"] != "Cell" {
                continue;
            }
            if owner["row"] == 7 {
                let text: String = cell_owned_nodes(owner)
                    .into_iter()
                    .filter(|node| node["type"] == "TextRun")
                    .filter_map(|node| node["text"].as_str())
                    .collect();
                following_content
                    .entry(owner["col"].as_u64().unwrap())
                    .or_default()
                    .push_str(&normalize(&text));
            }
            if owner["row"] != 5 || owner["col"] != 1 {
                continue;
            }
            assert!(
                bottom(owner) <= bottom(column) + 0.2,
                "paint expanded the accepted row budget"
            );
            for child in nodes(owner).into_iter().filter(|n| n["type"] == "Table") {
                fragments += 1;
                // The PDF repeats the declared outer top at each recursive
                // fragment. Only the paragraph offset belongs to its first.
                let lead = (f64::from(source_child.outer_margin_top)
                    + if first {
                        f64::from(source_child.common.vertical_offset)
                    } else {
                        0.0
                    })
                    / 75.0;
                assert!(
                    (coord(child, "y") - coord(owner, "y") - 141.0 / 75.0 - lead).abs() < 0.2,
                    "recursive paint added the paragraph offset twice"
                );
                assert!(
                    (coord(child, "x") - coord(owner, "x") - (141.0 + 283.0 + 341.0) / 75.0).abs()
                        < 0.2
                );
                // Padding is measured from the child cell frame. Table's paint
                // bbox includes half of the border stroke beyond that frame.
                let child_frame_bottom = child["children"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|node| node["type"] == "Cell")
                    .map(bottom)
                    .reduce(f64::max)
                    .expect("recursive child frame");
                assert!(child_frame_bottom <= bottom(owner) - 141.0 / 75.0 + 0.2);
                assert!(
                    bottom(child) <= bottom(owner) + 0.2,
                    "child stroke leaves its owner"
                );
                first = false;
                for node in nodes(child) {
                    if node["type"] != "TextRun" {
                        continue;
                    }
                    let Some(text) = node["text"].as_str() else {
                        continue;
                    };
                    if text.starts_with("UNIT ") {
                        assert!(source_units.insert(text.to_owned()), "source unit replayed");
                    }
                    if text == "FINAL CHILD ROW" {
                        final_child_rows += 1;
                    }
                }
            }
        }
    }
    assert!(fragments >= 2, "exercise a cut inside the child row");
    let expected: std::collections::BTreeSet<_> = (0..65).map(|i| format!("UNIT {i:03}")).collect();
    assert_eq!(
        source_units, expected,
        "every child paragraph must survive once"
    );
    assert_eq!(final_child_rows, expected_final_child_rows);
    assert_eq!(
        following_content, expected_following,
        "following row content must survive exactly once across its physical fragments"
    );
}

fn assert_recursive_child_page_owners(pages: &[Value]) {
    // Exact-input Hancom 2020 PDFs in valid_orientation and valid_generated:
    // p4 owns 000..017, p5 018..036, p6 037..055, p7 056..064.
    // p8 contains the following outer row, not another child unit/blank page.
    assert_eq!(pages.len(), 8);
    for (index, page) in pages.iter().enumerate() {
        let actual: Vec<_> = nodes(page)
            .into_iter()
            .filter(|node| node["type"] == "TextRun")
            .filter_map(|node| node["text"].as_str())
            .filter(|text| text.starts_with("UNIT "))
            .map(str::to_owned)
            .collect();
        let range = match index {
            3 => 0..18,
            4 => 18..37,
            5 => 37..56,
            6 => 56..65,
            _ => 0..0,
        };
        let expected: Vec<_> = range.map(|unit| format!("UNIT {unit:03}")).collect();
        assert_eq!(
            actual,
            expected,
            "Hancom child owners on page {}",
            index + 1
        );
    }
    let last = pages.last().unwrap();
    assert!(cell_owned_nodes(cell(outer(last), 7, 1))
        .into_iter()
        .any(|node| node["type"] == "TextRun"
            && node["text"]
                .as_str()
                .is_some_and(|text| !text.trim().is_empty())));
}

#[test]
fn auto_height_nested_row_shares_its_reserved_origin_and_child_cut() {
    assert_recursive_reflow_child("nested-auto-row.hwp", 1);
}

#[test]
fn one_cell_nested_fragments_reuse_the_child_unit_ledger() {
    assert_recursive_reflow_child("nested-mixed-cell.hwp", 0);
}
