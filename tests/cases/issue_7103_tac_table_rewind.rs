//! [#7103 / #7096] 구조 제어 사이의 TAC 표는 실제 저장 줄의 위치를 사용한다.
//! 한컴 기준: pdf/ari-tutoring-application-2020.pdf (engine 2020).
//! 단순 비겹침뿐 아니라 표 경계와 양수/음수 저장 간격을 독립 검증한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/issue_7103/ari-tutoring-application.hwp";
const PAGE: u32 = 0;
const HOST_PARA: usize = 0;
const TITLE_TABLE_CONTROL: usize = 3;
const CONSENT_TABLE_CONTROL: usize = 5;

#[derive(Clone, Copy, Debug)]
struct TableBox {
    control_index: usize,
    y: f64,
    bottom: f64,
}

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read #7103 fixture"))
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()))
}

fn collect_host_tables(node: &RenderNode, in_column: bool, out: &mut Vec<TableBox>) {
    if in_column {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(HOST_PARA) {
                if let Some(control_index) = table.control_index {
                    if [TITLE_TABLE_CONTROL, CONSENT_TABLE_CONTROL].contains(&control_index) {
                        out.push(TableBox {
                            control_index,
                            y: node.bbox.y,
                            bottom: node.bbox.y + node.bbox.height,
                        });
                    }
                }
            }
            return;
        }
    }

    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        collect_host_tables(child, in_column, out);
    }
}

#[test]
fn consecutive_tac_tables_do_not_rewind_to_the_previous_line_segment() {
    let core = load();
    let page = core
        .build_page_render_tree(PAGE)
        .expect("render #7103 page 1");
    let mut tables = Vec::new();
    collect_host_tables(&page.root, false, &mut tables);
    tables.sort_by(|a, b| a.control_index.cmp(&b.control_index));

    let title = tables
        .iter()
        .find(|table| table.control_index == TITLE_TABLE_CONTROL)
        .unwrap_or_else(|| {
            panic!("제목 TAC 표(ci={TITLE_TABLE_CONTROL})를 찾을 수 없음: {tables:?}")
        });
    let consent = tables
        .iter()
        .find(|table| table.control_index == CONSENT_TABLE_CONTROL)
        .unwrap_or_else(|| {
            panic!("개인정보 TAC 표(ci={CONSENT_TABLE_CONTROL})를 찾을 수 없음: {tables:?}")
        });

    assert!(
        consent.y + 0.5 >= title.bottom,
        "#7103: 둘째 TAC 표는 제목 표 아래에서 시작해야 한다. \
         title={title:?}, consent={consent:?}"
    );
    // 절대 PDF 좌표 대신 원문 표 높이와 서로 다른 저장 줄의 소유를 검증한다.
    // 실제 위치와 모양은 독립 PDF의 Native/fresh WASM 전쪽 비교로 확인한다.
    let para = &core.document().sections[0].paragraphs[HOST_PARA];
    let rhwp::model::control::Control::Table(source_title) = &para.controls[TITLE_TABLE_CONTROL]
    else {
        panic!("원문 제목 표 누락");
    };
    let owner = |control: usize| {
        para.line_segs
            .iter()
            .find(|line| line.text_start == (control * 8) as u32)
            .expect("원문 제어 스트림의 표 소유 줄")
    };
    assert!(
        (title.bottom - title.y - f64::from(source_title.common.height) / 75.0).abs() < 0.5,
        "원문 제목 표 높이 보존: {title:?}"
    );
    let stored_delta =
        owner(CONSENT_TABLE_CONTROL).vertical_pos - owner(TITLE_TABLE_CONTROL).vertical_pos;
    assert!(
        (consent.y - title.y - f64::from(stored_delta) / 75.0).abs() < 0.5,
        "표 사이 저장 줄 간격 보존: title={title:?}, consent={consent:?}"
    );
}

fn all_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        all_nodes(child, out);
    }
}

#[test]
fn hancom_privacy_and_tutor_boundaries_match() {
    let core = load();
    assert_eq!(core.page_count(), 1);
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    all_nodes(&page.root, &mut nodes);
    let privacy = nodes.iter().find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.row_count == 2 && t.col_count == 3)).unwrap();
    let body = nodes.iter().find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(0) && t.control_index == Some(5))).unwrap();
    let tutor = body
        .children
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 1 && c.col == 0))
        .unwrap();
    // 한컴 PDF에서 개인정보 표 다음에 튜터 인적사항 행이 나온다.
    // 절대 픽셀을 고정하지 않고 본문 표 안의 포함·순서·내용을 검증한다.
    for node in [*privacy, tutor] {
        assert!(
            node.bbox.y >= body.bbox.y - 0.5
                && node.bbox.y + node.bbox.height <= body.bbox.y + body.bbox.height + 0.5,
            "본문 표 내부 포함: {:?}, 본문 {:?}",
            node.bbox,
            body.bbox
        );
    }
    assert!(privacy.bbox.y + privacy.bbox.height <= tutor.bbox.y + 0.5);
    let text = |node: &RenderNode| {
        let mut descendants = Vec::new();
        all_nodes(node, &mut descendants);
        descendants
            .into_iter()
            .fold(String::new(), |mut text, node| {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    text.push_str(&run.text);
                }
                text
            })
    };
    assert!(text(privacy).contains("필수 항목"), "개인정보 표 내용 누락");
    assert!(text(tutor).contains("인적사항"), "튜터 행 내용 누락");
}

fn stored_gap_survives(gap: i32) {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let mut doc = rhwp::parser::parse_document(&bytes).unwrap();
    let para = &mut doc.sections[0].paragraphs[0];
    // 원시 24/40은 두 표의 위치이며 header/footer의 원래 순서는 유지한다.
    let delta = para.line_segs[1].line_height + gap;
    para.line_segs[3].vertical_pos = para.line_segs[1].vertical_pos + delta;
    let bytes = rhwp::serializer::serialize_document(&doc).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let page = core.build_page_render_tree(0).unwrap();
    let mut tables = Vec::new();
    collect_host_tables(&page.root, false, &mut tables);
    tables.sort_by_key(|table| table.control_index);
    assert_eq!(tables.len(), 2);
    let actual = tables[1].y - tables[0].y;
    assert!(
        (actual - f64::from(delta) / 75.0).abs() < 0.1,
        "stored gap={gap}: {tables:?}"
    );
}

#[test]
fn positive_saved_gap_is_preserved() {
    stored_gap_survives(600);
}

#[test]
fn negative_saved_gap_is_not_clamped() {
    stored_gap_survives(-100);
}
