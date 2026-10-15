//! [#5885] 중첩 표 호스트 문단이 셀 마지막일 때 그 문단 뒤 간격이 유닛 흐름에서
//! 유실돼, 행이 짧은 칸 기준으로 전진하고 바깥 행 구분선이 중첩 표 마지막 행
//! 한가운데를 가로지르던 회귀 가드.
//!
//! 3171199(설계업자 사업수행능력 세부평가기준, 별표 서식) 2쪽 실측 — 수정 전:
//! `⑵재정상태건실도` 행의 셀 두 개는 bottom 870.1, 중첩 표를 품은 칸만 876.4 로
//! 6.3px 더 길었고 다음 행이 870.1 에서 겹쳐 그려졌다. 한글 2022 실측은 행 바닥을
//! 882.4 하나로 닫고 중첩 표 하단(878.7)을 그 안에 둔다. 수정 후 rhwp 는 행 바닥
//! 879.7 로 세 칸을 동일하게 닫고 중첩 표 하단(876.0)이 행 안에 들어온다.
//!
//! 근인: 저장 사다리에서 문단 뒤 간격(9.6px)은 다음 문단 유닛의 corrected line
//! height 가 흡수하는데, 중첩 표 호스트 유닛은 표 행 높이 분해값이라 그 간격이
//! 없고, 호스트가 셀 마지막 문단이면 흡수할 다음 유닛도 없어 유닛 합이 저장
//! 종점보다 짧아진다 (521.7 vs 531.4).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5885/3171199_design_capability_criteria.hwp";

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-8 * a.abs().max(b.abs()).max(1.0)
}

fn outer(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(3)) {
        return Some(node);
    }
    node.children.iter().find_map(outer)
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn nested_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    for child in &node.children {
        if matches!(child.node_type, RenderNodeType::Table(_)) {
            out.push(child);
        }
        nested_tables(child, out);
    }
}

#[test]
fn issue_5885_outer_row_closes_below_nested_table_bottom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&fs::read(path).expect("원본")).expect("열기");
    assert_eq!(core.page_count(), 7, "독립 Print의 전체 쪽 소유");
    let page = core.build_page_render_tree(1).expect("2쪽");
    let frame = outer(&page.root).expect("원본 바깥 표");
    let row_cells: Vec<_> = frame
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 5))
        .collect();
    assert_eq!(row_cells.len(), 3, "원본 재정상태건실도 행의 세 소유자");
    let top = row_cells[0].bbox.y;
    let bottom = top + row_cells[0].bbox.height;
    let mut nested_count = 0;
    for cell in row_cells {
        assert!(near(cell.bbox.y, top), "같은 행의 시작은 공통이다");
        assert!(
            near(cell.bbox.y + cell.bbox.height, bottom),
            "중첩 표 소유자도 같은 행 바닥을 공유해야 한다"
        );
        let mut nested = Vec::new();
        nested_tables(cell, &mut nested);
        nested_count += nested.len();
        for table in nested {
            assert!(
                table.bbox.y >= top && table.bbox.y + table.bbox.height <= bottom,
                "독립 Print처럼 두 중첩 표는 실제 부모 셀 안에 있어야 한다"
            );
        }
    }
    assert_eq!(nested_count, 2, "원본의 구분/점수 중첩 표 두 개를 보존한다");
    let next: Vec<_> = frame
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 6))
        .collect();
    assert!(!next.is_empty(), "원본의 다음 기술개발 행");
    for cell in next {
        assert!(
            near(cell.bbox.y, bottom),
            "다음 행은 부모 행 바닥에서 시작하며 중첩 표를 가로지르지 않는다"
        );
    }
}

/// 첫 물리 조각은 첫 줄 뒤 간격이 아닌 가시 줄과 안 여백에서 닫힌다.
#[test]
fn issue_5885_first_fragment_closes_at_independent_pdf_border() {
    let bytes =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("원본 첫 조각 입력");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 첫 조각 열기");
    assert_eq!(core.page_count(), 7, "독립 PDF의 전체 쪽 소유");
    let first = core.build_page_render_tree(0).expect("첫 조각 쪽");
    let frame = outer(&first.root).expect("첫 쪽 원본 표");
    let borders = frame
        .children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::Line(line) = &node.node_type {
                if (line.y1 - line.y2).abs() < 0.01
                    && (line.x2 - line.x1).abs() > frame.bbox.width - 1.0
                {
                    return Some((line.y1, line.style.width));
                }
            }
            None
        })
        .collect::<Vec<_>>();
    assert!(borders.len() >= 2, "표의 실제 paint 괘선");
    let top = borders
        .iter()
        .map(|line| line.0)
        .fold(f64::INFINITY, f64::min);
    let bottom = borders
        .iter()
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("마지막 괘선");
    assert!(near(top, frame.bbox.y), "괘선과 실제 조각의 공통 시작");
    let last_cells: Vec<_> = frame
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 4))
        .collect();
    assert_eq!(last_cells.len(), 4, "첫 컷이 실제로 소유한 일반 셀 네 개");
    let rhwp::model::control::Control::Table(source) =
        &core.document().sections[0].paragraphs[3].controls[0]
    else {
        panic!("원본 표")
    };
    let mut visible_end = f64::NEG_INFINITY;
    fn lines(node: &RenderNode, out: &mut Vec<f64>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            out.push(node.bbox.y + node.bbox.height);
        }
        for child in &node.children {
            lines(child, out);
        }
    }
    for cell in last_cells {
        let RenderNodeType::TableCell(owner) = &cell.node_type else {
            unreachable!()
        };
        let original = source
            .cells
            .iter()
            .find(|c| c.row == owner.row && c.col == owner.col)
            .expect("같은 원본 셀의 안 여백");
        // 안 여백 지정 플래그는 셀 고유 여백과 표 기본 여백을 구분한다.
        // 현재 구현의 컷 높이/줄 전진량을 기대값으로 다시 계산하지 않는다.
        let padding = if original.apply_inner_margin {
            original.padding.bottom
        } else {
            source.padding.bottom
        };
        assert!(padding >= 0, "이 실제 원본의 유효 안 여백");
        let mut ends = Vec::new();
        lines(cell, &mut ends);
        assert!(!ends.is_empty(), "빈 문자열도 유효한 글줄 상자를 소유한다");
        visible_end = visible_end
            .max(ends.into_iter().fold(f64::NEG_INFINITY, f64::max) + f64::from(padding) / 75.0);
    }
    assert!(near(bottom.0, visible_end),
        "독립 Print처럼 첫 조각은 가시 글줄과 원본 안 여백에서 닫히고 다음 줄 전진량을 다시 점유하지 않는다");
    assert!(
        frame.bbox.y + frame.bbox.height <= bottom.0 + bottom.1 / 2.0,
        "본문 점유는 실제 외곽선 뒤의 간격을 다시 포함하지 않는다"
    );
    let pages = core.dump_page_items_json(None);
    let reserved_end = pages[0]["bodyArea"]["y"].as_f64().expect("본문 원점")
        + pages[0]["columns"][0]["usedHeight"]
            .as_f64()
            .expect("실제 예약 높이");
    assert!(
        near(reserved_end, bottom.0),
        "예약 끝{reserved_end}과 실제 괘선{}은 같은 물리 컷 높이를 소비해야 함",
        bottom.0
    );
    let items = pages
        .as_array()
        .expect("쪽 목록")
        .iter()
        .flat_map(|page| page["columns"].as_array().expect("단 목록"))
        .flat_map(|col| col["items"].as_array().expect("원본 소유 항목"))
        .filter(|item| item["paraIndex"] == 3 && item["kind"] == "partialTable")
        .collect::<Vec<_>>();
    assert_eq!(items[0]["startRow"], 0);
    assert_eq!(items[0]["endRow"], 5);
    let end_cut: Vec<_> = items[0]["endCut"]
        .as_array()
        .expect("원본 셀 컷")
        .iter()
        .map(|unit| unit.as_u64().expect("소유 유닛 인덱스"))
        .collect();
    assert_eq!(end_cut, [1, 1, 1, 1, 0, 0, 0]);
    assert_eq!(items[1]["startRow"], 4);
    assert_eq!(
        items[1]["startCut"], items[0]["endCut"],
        "소유 유닛 누락/중복 금지"
    );
    let last = core.build_page_render_tree(6).expect("마지막 쪽");
    let last_body = body(&last.root).expect("마지막 쪽 본문");
    let body_bounds = (last_body.bbox.y, last_body.bbox.y + last_body.bbox.height);
    fn final_text(node: &RenderNode, text: &mut String, bounds: (f64, f64)) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.para_index == Some(50) {
                text.push_str(&run.text);
                assert!(
                    node.bbox.y >= bounds.0 && node.bbox.y + node.bbox.height <= bounds.1,
                    "마지막 문단의 실제 글줄이 본문 안에 보존돼야 함"
                );
            }
        }
        for child in &node.children {
            final_text(child, text, bounds);
        }
    }
    let mut text = String::new();
    final_text(&last.root, &mut text, body_bounds);
    let text: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    assert!(
        text.starts_with("7.발주자는당해설계용역의특성에적합하도록"),
        "독립 PDF 마지막 문단 시작의 소유"
    );
    assert!(
        text.ends_with("세부적으로설계용역평가기준을정한후입찰공고시제시하여야한다."),
        "독립 PDF 마지막 문단 끝의 누락 금지"
    );
}
