//! #5846: 혼재 호스트의 내부 행은 쪽별 원본 내용과 표/문단 순서를 보존한다.
//!
//! 독립 기준은 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`이다.
//! 보정235의 Native/fresh 전쪽 66개는 최저 90.30272%, 미달 0이다.
//! 59쪽은 앞 행, 60쪽은 단기연체정보부터 서명까지의 뒤 행을 소유한다.
//! 절대 좌표·픽셀 공차·유닛 번호 대신 소속, 포함 및 표 뒤 문단의 관계를 검사한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::path::Path;

const SAMPLE: &str = "samples/task2097/75544_pii_bunseok.hwpx";
// 좌표 기대값이 아니라 HU 변환의 부동소수 끝자리만 흡수한다.
const FLOAT_EPSILON: f64 = 1e-6;

fn text(node: &RenderNode) -> String {
    if !node.visible {
        return String::new();
    }
    let mut out = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.extend(run.display_or_text().chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

fn owning_cell<'a>(node: &'a RenderNode, marker: &str) -> Option<&'a RenderNode> {
    node.children
        .iter()
        .find_map(|child| owning_cell(child, marker))
        .or_else(|| {
            (matches!(node.node_type, RenderNodeType::TableCell(_)) && text(node).contains(marker))
                .then_some(node)
        })
}

fn line<'a>(node: &'a RenderNode, marker: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && text(node).contains(marker) {
        return Some(node);
    }
    node.children.iter().find_map(|child| line(child, marker))
}

fn table_with<'a>(cell: &'a RenderNode, marker: &str) -> &'a RenderNode {
    cell.children
        .iter()
        .find(|child| {
            matches!(child.node_type, RenderNodeType::Table(_)) && text(child).contains(marker)
        })
        .unwrap_or_else(|| panic!("{marker} 표가 실제 소유 셀에 없음"))
}

#[test]
fn issue_5846_cut_fragment_defers_partial_nested_tail_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(&path).expect("개인정보 원본 읽기"))
        .expect("개인정보 원본 파싱");
    assert_eq!(doc.page_count(), 66, "독립 Print PDF와 전체 쪽수");
    let before = doc.build_page_render_tree(58).expect("59쪽 렌더");
    let after = doc.build_page_render_tree(59).expect("60쪽 렌더");
    assert!(
        text(&before.root).contains("채무불이행정보"),
        "앞 행의 끝 내용은 59쪽 소유"
    );
    for marker in [
        "단기연체정보",
        "금융질서문란정보",
        "※작성요령",
        "○○금융회사대표이사△△△",
    ] {
        assert!(
            !text(&before.root).contains(marker),
            "뒤 행 {marker}가 앞쪽에 중복됨"
        );
        assert_eq!(
            text(&after.root).matches(marker).count(),
            1,
            "뒤 행 {marker}가 60쪽에 한 번 표시되어야 함"
        );
    }
    let cell = owning_cell(&after.root, "금융질서문란정보").expect("뒤 행 소유 셀");
    if let RenderNodeType::TableCell(model) = &cell.node_type {
        assert_eq!(model.row, 1, "원본 둘째 행 소유");
    }
    let first = line(cell, "단기연체정보").expect("첫 항목 줄");
    let following = line(cell, "금융질서문란정보").expect("다음 항목 줄");
    let first_table = table_with(cell, "연체금액");
    let second_table = table_with(cell, "사유");
    assert!(
        first.bbox.y + first.bbox.height <= first_table.bbox.y + FLOAT_EPSILON,
        "첫 항목 뒤에 연체 표가 놓여야 함"
    );
    assert!(
        first_table.bbox.y + first_table.bbox.height <= following.bbox.y + FLOAT_EPSILON,
        "다음 항목이 첫 표 안으로 올라가면 안 됨"
    );
    assert!(
        following.bbox.y + following.bbox.height <= second_table.bbox.y + FLOAT_EPSILON,
        "다음 항목 뒤에 해당 표가 놓여야 함"
    );
    for marker in [
        "단기연체정보",
        "금융질서문란정보",
        "※작성요령",
        "○○금융회사대표이사△△△",
    ] {
        let content = line(cell, marker).unwrap_or_else(|| panic!("{marker} 줄 소실"));
        assert!(
            content.bbox.y >= cell.bbox.y - FLOAT_EPSILON
                && content.bbox.y + content.bbox.height
                    <= cell.bbox.y + cell.bbox.height + FLOAT_EPSILON,
            "{marker} 줄이 원본 소유 셀 안에 있어야 함"
        );
    }
}
