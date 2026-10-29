//! #7032: 저장된 빈 셀 문단은 LINE_SEG가 없어도 한 줄을 소유한다.
//! HWPX 검증 문서와 메인터너의 한컴 판독을 독립 대조군으로 사용한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, table::Cell};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core(hwpx: bool) -> DocumentCore {
    let file = if hwpx {
        "samples/hwpx/21761835_jeonjik_exemption_table.hwpx"
    } else {
        "samples/task2146/21761835_jeonjik_exemption_table.hwp"
    };
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file))
        .expect("required fixture");
    DocumentCore::from_bytes(&bytes).expect("fixture parse")
}

fn source_cell(core: &DocumentCore) -> &Cell {
    let Control::Table(table) = &core.document().sections[0].paragraphs[4].controls[0] else {
        panic!("fixture table pi=4 ci=0")
    };
    &table.cells[0]
}

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![root];
    for child in &root.children {
        result.extend(nodes(child));
    }
    result
}

fn header(root: &RenderNode) -> &RenderNode {
    let cells: Vec<_> = nodes(root)
        .into_iter()
        .filter(
            |n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == 0),
        )
        .collect();
    assert_eq!(cells.len(), 1, "one repeated title cell per page");
    cells[0]
}

fn paragraph_lines(cell: &RenderNode) -> Vec<&RenderNode> {
    nodes(cell)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .collect()
}

fn assert_near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 0.11, "{actual} != {expected}");
}

fn assert_header_flow(core: &DocumentCore) {
    let cell = source_cell(core);
    let para = &cell.paragraphs[0];
    let shape = &core.document().doc_info.para_shapes[para.para_shape_id as usize];
    assert_eq!(
        shape.line_spacing_type,
        rhwp::model::style::LineSpacingType::Fixed
    );
    // HWP 고정 줄간격은 실제 HWPUNIT 전진값의 두 배로 저장된다.
    let advance = f64::from(shape.line_spacing) / 2.0 * 96.0 / 7200.0;
    // #7445: HWP의 6→7쪽 실패와 첫 여섯 쪽의 머리행 계약을 구분합니다.
    // 정상 HWPX의 6쪽 핀은 해당 검사에서 유지합니다.
    for page in 0..6 {
        let tree = core.build_page_render_tree(page).expect("page render tree");
        let cell = header(&tree.root);
        let lines = paragraph_lines(cell);
        assert_eq!(lines.len(), 2, "p{} empty paragraph must remain", page + 1);
        for (pi, line) in lines.iter().enumerate() {
            let RenderNodeType::TextLine(info) = &line.node_type else {
                unreachable!()
            };
            assert_eq!(info.para_index, Some(pi));
        }
        let empty_runs: Vec<_> = nodes(lines[0])
            .into_iter()
            .filter_map(|n| {
                if let RenderNodeType::TextRun(run) = &n.node_type {
                    Some(run)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(empty_runs.len(), 1);
        assert!(empty_runs[0].text.is_empty());
        assert!(empty_runs[0].is_para_end);
        assert!(empty_runs[0].cell_context.is_some());
        assert!(nodes(lines[1])
            .iter()
            .any(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text == "직렬")));
        assert_near(lines[1].bbox.y - lines[0].bbox.y, advance);
        assert_near(cell.bbox.height, 52.4);
    }
}

#[test]
fn hwp_retains_empty_line_in_all_repeated_headers() {
    assert_header_flow(&core(false));
}

#[test]
fn hwpx_stored_empty_line_remains_unchanged() {
    let core = core(true);
    assert_eq!(core.page_count(), 6);
    assert_header_flow(&core);
}

#[test]
fn both_parsers_preserve_the_same_two_paragraphs() {
    for hwpx in [false, true] {
        let core = core(hwpx);
        let cell = source_cell(&core);
        assert_eq!(cell.paragraphs.len(), 2);
        assert!(cell.paragraphs[0].text.is_empty());
        assert_eq!(cell.paragraphs[1].text.trim(), "직렬");
        for para in &cell.paragraphs {
            assert!(para.controls.is_empty());
            assert!(para.char_count > 0);
            assert_eq!(para.line_segs.len(), usize::from(hwpx));
        }
    }
}

#[test]
fn hwp_header_text_origin_matches_stored_hwpx_control() {
    let hwp = core(false);
    let hwpx = core(true);
    for page in 0..6 {
        let a = hwp.build_page_render_tree(page).unwrap();
        let b = hwpx.build_page_render_tree(page).unwrap();
        let a = paragraph_lines(header(&a.root));
        let b = paragraph_lines(header(&b.root));
        assert_eq!((a.len(), b.len()), (2, 2));
        for (a, b) in a.iter().zip(&b) {
            assert_near(a.bbox.y, b.bbox.y);
            // 고정 줄간격 대체 경로와 저장 LINE_SEG는 줄 상자를 서로 다르게
            // 인코딩할 수 있다. 두 형식의 내부 높이·간격 분리가 아니라 줄의
            // 소유와 다음 원점이 일치해야 한다.
        }
    }
}

// 저장된 한컴 검증 문서나 쪽 나눔 정답지가 아닌 내부 렌더링 계약이다.
// A4를 유지하고 정규 문단 단위의 명시적 절단 범위를 제공한다.
fn fragment(
    cut: Option<(usize, usize)>,
    align: rhwp::model::table::VerticalAlign,
    percent: bool,
    spacing: bool,
) -> rhwp::renderer::render_tree::PageRenderTree {
    use rhwp::model::{
        page::{ColumnDef, PageDef},
        paragraph::Paragraph,
        table::TablePageBreak,
    };
    use rhwp::renderer::{
        composer::compose_paragraph,
        height_measurer::HeightMeasurer,
        layout::LayoutEngine,
        pagination::{PageItem, Paginator},
        style_resolver::resolve_styles,
    };
    let core = core(false);
    let mut doc = core.document().clone();
    let para_shape = source_cell(&core).paragraphs[0].para_shape_id as usize;
    if percent {
        doc.doc_info.para_shapes[para_shape].line_spacing_type =
            rhwp::model::style::LineSpacingType::Percent;
        doc.doc_info.para_shapes[para_shape].line_spacing = 160;
    }
    if spacing {
        doc.doc_info.para_shapes[para_shape].spacing_before = 300;
        doc.doc_info.para_shapes[para_shape].spacing_after = 450;
    }
    let Control::Table(mut table) = doc.sections[0].paragraphs[4].controls[0].clone() else {
        unreachable!()
    };
    table.cells.truncate(2);
    table.row_count = 1;
    table.col_count = 2;
    table.repeat_header = false;
    table.page_break = TablePageBreak::CellBreak;
    table.common.treat_as_char = false;
    table.common.width = table.cells.iter().map(|c| c.width).sum();
    table.common.height = 12000;
    let empty = table.cells[0].paragraphs[0].clone();
    let text = table.cells[0].paragraphs[1].clone();
    for cell in &mut table.cells {
        cell.is_header = false;
        cell.height = 12000;
        cell.vertical_align = align;
        // 빈 문단, 빈 문단, 텍스트, 빈 문단 순서로 문단당 정규 단위 하나를 둔다.
        cell.paragraphs = vec![empty.clone(), empty.clone(), text.clone(), empty.clone()];
    }
    let paragraphs = vec![Paragraph {
        controls: vec![Control::Table(table)],
        ..Default::default()
    }];
    let styles = resolve_styles(&doc.doc_info, 96.0);
    let composed = paragraphs.iter().map(compose_paragraph).collect::<Vec<_>>();
    let measured = HeightMeasurer::new(96.0).measure_section(&paragraphs, &composed, &styles, None);
    let mut pages = Paginator::new(96.0).paginate_with_measured(
        &paragraphs,
        &measured,
        &PageDef::default(),
        &ColumnDef::default(),
        0,
        &styles.para_styles,
    );
    let page = &mut pages.pages[0];
    page.column_contents[0].items = vec![PageItem::PartialTable {
        para_index: 0,
        control_index: 0,
        start_row: 0,
        end_row: 1,
        is_continuation: false,
        start_cut: cut.map(|(s, _)| vec![s; 2]).unwrap_or_default(),
        end_cut: cut.map(|(_, e)| vec![e; 2]).unwrap_or_default(),
        is_block_split: false,
        start_cut_is_block: false,
        row_cursor_is_nested: false,
        // 정렬을 비교할 때 물리 프레임을 고정한다. 그렇지 않으면 자연 분할이
        // 미분할과 달리 소비한 내용에 맞춰 조각 크기를 정한다.
        end_row_height_override: cut.map(|_| measured.tables[0].row_heights[0]),
        start_row_height_override: None,
        straddle_row_relief: Vec::new(),
        straddle_page_bands: Vec::new(),
        fragment_row_heights: None,
        fragment_block_frames: Vec::new(),
        end_row_owns_stored_frame: false,
        first_fragment_origin: None,
    }];
    LayoutEngine::new(96.0).build_render_tree(
        page,
        &paragraphs,
        &[],
        &[],
        &composed,
        &styles,
        &Default::default(),
        &[],
        None,
        &measured.tables,
        None,
        0,
        &[],
    )
}

#[test]
fn full_atom_window_preserves_uncut_alignment_and_spacing() {
    use rhwp::model::table::VerticalAlign::{Bottom, Center, Top};
    for align in [Top, Center, Bottom] {
        for percent in [false, true] {
            for spacing in [false, true] {
                let uncut = fragment(None, align, percent, spacing);
                let cut = fragment(Some((0, 4)), align, percent, spacing);
                assert_near(
                    header(&uncut.root).bbox.height,
                    header(&cut.root).bbox.height,
                );
                let a = paragraph_lines(header(&uncut.root));
                let b = paragraph_lines(header(&cut.root));
                assert_eq!((a.len(), b.len()), (4, 4));
                for (a, b) in a.iter().zip(&b) {
                    assert_near(a.bbox.y, b.bbox.y);
                    assert_near(a.bbox.height, b.bbox.height);
                }
            }
        }
    }
}

#[test]
fn cut_window_emits_only_owned_empty_paragraphs_once() {
    use rhwp::model::table::VerticalAlign::Top;
    let reference = core(false);
    let para_shape = source_cell(&reference).paragraphs[0].para_shape_id as usize;
    let advance =
        f64::from(reference.document().doc_info.para_shapes[para_shape].line_spacing) / 2.0 * 96.0
            / 7200.0;
    let first = fragment(Some((0, 1)), Top, false, false);
    let origin = paragraph_lines(header(&first.root))[0].bbox.y - header(&first.root).bbox.y;
    for (start, end) in [(0, 1), (0, 2), (1, 3), (2, 4), (3, 4), (4, 4)] {
        let tree = fragment(Some((start, end)), Top, false, false);
        assert!(matches!(&header(&tree.root).node_type,
            RenderNodeType::TableCell(c) if c.page_fragment));
        let lines = paragraph_lines(header(&tree.root));
        let owners: Vec<_> = lines
            .iter()
            .map(|n| {
                let RenderNodeType::TextLine(l) = &n.node_type else {
                    unreachable!()
                };
                l.para_index.unwrap()
            })
            .collect();
        assert_eq!(
            owners,
            (start..end).collect::<Vec<_>>(),
            "cut {start}..{end}"
        );
        if let Some(first) = lines.first() {
            // 절단 범위 밖의 문단은 세로 공간을 소비하지 않아야 한다.
            assert_near(first.bbox.y - header(&tree.root).bbox.y, origin);
        }
        for pair in lines.windows(2) {
            // 연속으로 선택된 각 단위는 정확히 한 줄의 전진 공간을 소비한다.
            assert_near(pair[1].bbox.y - pair[0].bbox.y, advance);
        }
        for line in lines {
            assert!(nodes(line).iter().any(
                |n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.cell_context.is_some())
            ));
        }
    }
}

#[test]
fn last_empty_cell_paragraph_uses_em_without_trailing_spacing() {
    use rhwp::model::table::VerticalAlign::Top;
    let reference = core(false);
    let para = &source_cell(&reference).paragraphs[0];
    let fs = reference.document().doc_info.char_shapes[para.char_shapes[0].char_shape_id as usize]
        .base_size;
    let expected = f64::from(fs) * 96.0 / 7200.0;
    for cut in [None, Some((0, 4)), Some((3, 4)), Some((0, 1))] {
        let tree = fragment(cut, Top, false, false);
        let lines = paragraph_lines(header(&tree.root));
        assert_near(lines.last().unwrap().bbox.height, expected);
    }
}
