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

/// (테이블 깊이, row, y, bottom) 목록 수집 — 깊이 1 = 바깥 표 셀, 2 = 중첩 표 셀.
fn collect_cells(node: &RenderNode, table_depth: usize, acc: &mut Vec<(usize, u16, f64, f64)>) {
    let next_depth = if matches!(node.node_type, RenderNodeType::Table(_)) {
        table_depth + 1
    } else {
        table_depth
    };
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        acc.push((
            table_depth,
            tc.row,
            node.bbox.y,
            node.bbox.y + node.bbox.height,
        ));
    }
    for child in &node.children {
        collect_cells(child, next_depth, acc);
    }
}

#[test]
fn issue_5885_outer_row_closes_below_nested_table_bottom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5885 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5885 fixture");

    assert_eq!(core.page_count(), 7, "3171199 은 7쪽 문서다 (한글과 동일)");

    let page = core.build_page_render_tree(1).expect("render p2");
    let mut cells = Vec::new();
    collect_cells(&page.root, 0, &mut cells);

    // 바깥 표 `⑵재정상태건실도`는 원본의 5번 행이다.
    // 과거 출력 y344.6으로 소유를 찾으면 바깥 위여백 복원 뒤 해당 행을 놓친다.
    // 독립 한컴2020 PDF2쪽의 괘선 시작은347.94px이다.
    let row_cells: Vec<_> = cells
        .iter()
        .filter(|(d, row, _, _)| *d == 1 && *row == 5)
        .collect();
    assert!(
        row_cells.len() >= 3,
        "p2 재정상태건실도 행 셀 3개를 찾아야 함; got {}",
        row_cells.len()
    );
    for (_, _, top, _) in &row_cells {
        assert!(
            (*top - 347.94).abs() < 0.6,
            "재정상태건실도 행의 실제 원점은 독립 PDF347.94px: {top}"
        );
    }
    let bottoms: Vec<f64> = row_cells.iter().map(|(_, _, _, b)| *b).collect();
    let min_b = bottoms.iter().cloned().fold(f64::MAX, f64::min);
    let max_b = bottoms.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        max_b - min_b <= 0.5,
        "같은 행의 셀 바닥은 하나로 닫혀야 함 (수정 전 870.1 vs 876.4); got {bottoms:?}"
    );

    // 그 행 안의 중첩 표(`구 분`/`점 수`) 마지막 행 하단은 바깥 행 바닥 안에 있어야 한다.
    let nested_bottom = cells
        .iter()
        .filter(|(d, _, y, _)| *d == 2 && *y > 800.0 && *y < 900.0)
        .map(|(_, _, _, b)| *b)
        .fold(f64::MIN, f64::max);
    assert!(
        nested_bottom > 800.0,
        "p2 중첩 표 셀을 찾아야 함; got {nested_bottom}"
    );
    assert!(
        nested_bottom <= min_b + 0.5,
        "중첩 표 하단({nested_bottom:.1})은 바깥 행 바닥({min_b:.1}) 안에 있어야 함 — \
         수정 전엔 행 구분선이 중첩 표 마지막 행을 가로질렀다"
    );

    // 다음 행(`라. 기술개발 및`)은 행 바닥에서 시작한다 — 겹침 없음.
    let next_row_top = cells
        .iter()
        .filter(|(d, _, y, _)| *d == 1 && *y > min_b - 1.0 && *y < min_b + 5.0)
        .map(|(_, _, y, _)| *y)
        .fold(f64::MAX, f64::min);
    assert!(
        (next_row_top - max_b).abs() <= 0.5,
        "다음 행 시작({next_row_top:.1})은 이전 행 바닥({max_b:.1})과 일치해야 함"
    );
}

/// 첫 물리 조각은 첫 줄 뒤 간격이 아닌 가시 줄과 안 여백에서 닫힌다.
#[test]
fn issue_5885_first_fragment_closes_at_independent_pdf_border() {
    let bytes =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("원본 첫 조각 입력");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 첫 조각 열기");
    assert_eq!(core.page_count(), 7, "독립 PDF의 전체 쪽 소유");
    fn outer(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(3)) {
            return Some(node);
        }
        node.children.iter().find_map(outer)
    }
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
    assert!(
        (top - 189.233_333).abs() < 0.6,
        "독립 PDF 첫 표 상단: {top}"
    );
    // PDF 괘선 중심과 glyph/선 두께의 잔여 오차를 분리해 측정한다.
    assert!(
        (bottom.0 - 1_002.902_667).abs() < 1.3,
        "독립 PDF 첫 조각 하단1002.902667px: {}",
        bottom.0
    );
    assert!(
        frame.bbox.y + frame.bbox.height <= bottom.0 + bottom.1 / 2.0 + 0.05,
        "본문 점유는 실제 표 외곽선 뒤의 간격을 다시 포함하면 안 됨"
    );
    let pages = core.dump_page_items_json(None);
    let reserved_end = pages[0]["bodyArea"]["y"].as_f64().expect("본문 원점")
        + pages[0]["columns"][0]["usedHeight"]
            .as_f64()
            .expect("실제 예약 높이");
    assert!(
        (reserved_end - bottom.0).abs() < 0.05,
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
    assert_eq!(items[0]["endCut"], serde_json::json!([1, 1, 1, 1, 0, 0, 0]));
    assert_eq!(items[1]["startRow"], 4);
    assert_eq!(
        items[1]["startCut"], items[0]["endCut"],
        "소유 유닛 누락/중복 금지"
    );
    let last = core.build_page_render_tree(6).expect("마지막 쪽");
    fn final_text(node: &RenderNode, text: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.para_index == Some(50) {
                text.push_str(&run.text);
                assert!(
                    node.bbox.y >= 94.0 && node.bbox.y + node.bbox.height < 1_009.2,
                    "마지막 문단의 실제 글줄이 본문 안에 보존돼야 함"
                );
            }
        }
        for child in &node.children {
            final_text(child, text);
        }
    }
    let mut text = String::new();
    final_text(&last.root, &mut text);
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
