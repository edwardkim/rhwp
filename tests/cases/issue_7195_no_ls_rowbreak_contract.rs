//! 합성 IR 계약: 다른 문단의 저장 LS가 NO_LS 표 host의 통째 이월 근거가 될 수 없다.
//! 한컴 출력 fixture가 아니다. 4개의 독립 행과 본문 예산으로 분할/fit을 검증한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn text(s: &str) -> Paragraph {
    let mut p = Paragraph::default();
    p.insert_text_at(0, s);
    p
}

fn fixture(page_height: u32, sibling_ls: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    let mut doc =
        rhwp::parser::parse_document(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    // 빈 한컴 문서의 스타일만 재사용한 합성 문서다. 원본 스트림 봉인을 위조하지 않는다.
    doc.sections[0].raw_stream = None;
    doc.sections[0].raw_provenance = None;
    let page = &mut doc.sections[0].section_def.page_def;
    page.height = page_height;
    page.margin_top = 0;
    page.margin_bottom = 0;
    page.margin_header = 0;
    page.margin_footer = 0;
    // 문단 간격의 저장 단위(1/14400 inch)로 7500은50px다.
    // 표 선언192px는 작은 쪽240px에도 단독 fit하지만 prefix 뒤에는 fit하지 않는다.
    let mut before_style = doc.doc_info.para_shapes[0].clone();
    before_style.spacing_after = 7500;
    let style_id = doc.doc_info.para_shapes.len() as u16;
    doc.doc_info.para_shapes.push(before_style);
    let mut before = text("before");
    before.para_shape_id = style_id;
    let mut sibling = text("sibling");
    if sibling_ls {
        sibling.line_segs = vec![LineSeg {
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            segment_width: 10000,
            ..Default::default()
        }];
    }
    let mut table = Table {
        row_count: 4,
        col_count: 1,
        page_break: TablePageBreak::RowBreak,
        ..Default::default()
    };
    table.common.width = 10000;
    table.common.height = 14400;
    table.common.text_wrap = TextWrap::TopAndBottom;
    table.common.vert_rel_to = VertRelTo::Para;
    table.common.horz_rel_to = HorzRelTo::Para;
    for row in 0..4 {
        table.cells.push(Cell {
            row,
            col: 0,
            row_span: 1,
            col_span: 1,
            width: 10000,
            height: 3600,
            paragraphs: (0..3)
                .map(|line| text(&format!("row{row}line{line}")))
                .collect(),
            ..Default::default()
        });
    }
    table.rebuild_grid();
    let host = Paragraph {
        controls: vec![Control::Table(Box::new(table))],
        char_count: 9,
        control_mask: 1 << 11,
        has_para_text: true,
        ..Default::default()
    };
    assert!(host.line_segs.is_empty());
    doc.sections[0].paragraphs = vec![sibling, before, host, text("after")];
    core.set_document(doc);
    core
}

fn body(n: &RenderNode) -> Option<&RenderNode> {
    if matches!(n.node_type, RenderNodeType::Body { .. }) {
        return Some(n);
    }
    n.children.iter().find_map(body)
}
fn content(n: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &n.node_type {
        return run.text.clone();
    }
    n.children.iter().map(content).collect()
}
fn tables<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(2)) {
        out.push(n);
        return;
    }
    for c in &n.children {
        tables(c, out);
    }
}

#[test]
fn issue_7195_no_ls_rowbreak_uses_remaining_space_not_sibling_evidence() {
    for sibling_ls in [false, true] {
        for height in [18000, 36000] {
            let core = fixture(height, sibling_ls);
            assert!(
                !core.document().layout_profile().session_edited(),
                "통째 이월의 편집 우회로가 아닌 합성 NO_LS 경로"
            );
            assert_eq!(
                core.document().sections[0].paragraphs[0]
                    .line_segs
                    .is_empty(),
                !sibling_ls
            );
            assert!(core.document().sections[0].paragraphs[2]
                .line_segs
                .is_empty());
            let mut fragments = Vec::new();
            let mut whole_text = String::new();
            for page in 0..core.page_count() {
                let tree = core.build_page_render_tree(page).unwrap();
                whole_text.push_str(&content(&tree.root));
                let bounds = body(&tree.root).unwrap().bbox;
                let mut found = Vec::new();
                tables(&tree.root, &mut found);
                for table in found {
                    assert!(table.bbox.y >= bounds.y - 0.5);
                    assert!(table.bbox.y + table.bbox.height <= bounds.y + bounds.height + 0.5);
                    fragments.push((page, content(table)));
                }
            }
            assert_eq!(
                fragments.first().map(|f| f.0),
                Some(0),
                "head rows must use available space: {fragments:?}"
            );
            assert_eq!(
                fragments.iter().map(|f| f.1.as_str()).collect::<String>(),
                (0..4)
                    .flat_map(|row| (0..3).map(move |line| format!("row{row}line{line}")))
                    .collect::<String>()
            );
            assert_eq!(whole_text.matches("before").count(), 1);
            assert_eq!(whole_text.matches("after").count(), 1);
            assert!(whole_text.find("before").unwrap() < whole_text.find("row0line0").unwrap());
            assert!(whole_text.find("row3line2").unwrap() < whole_text.find("after").unwrap());
            if height == 18000 {
                assert!(fragments.len() > 1, "whole table cannot fit after prefix");
            } else {
                assert_eq!(fragments.len(), 1, "fitting table must not split");
            }
        }
    }
}
