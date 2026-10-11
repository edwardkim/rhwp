//! #5846: 혼재 호스트의 내부 행은 쪽별 원본 내용과 표/문단 순서를 보존한다.
//!
//! 독립 기준은 `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`이다.
//! 보정235의 Native/fresh 전쪽 66개는 최저 90.30272%, 미달 0이다.
//! 59쪽은 앞 행, 60쪽은 단기연체정보부터 서명까지의 뒤 행을 소유한다.
//! 절대 좌표·픽셀 공차·유닛 번호 대신 소속, 포함 및 표 뒤 문단의 관계를 검사한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use rhwp::DocumentCore;
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

fn owned_table(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(para) && table.control_index == Some(0))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_table(child, para))
}

fn source_cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col)
        })
        .unwrap_or_else(|| panic!("원본 칸 ({row}, {col}) 소실"))
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

#[test]
fn issue_5846_cut_fragment_defers_partial_nested_tail_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("개인정보 원본 읽기");
    let source = DocumentCore::from_bytes(&bytes).expect("독립 원본 속성 읽기");
    let source_paras = &source.document().sections[0].paragraphs;
    let mut doc = HwpDocument::from_bytes(&bytes).expect("개인정보 원본 파싱");
    // 전쪽 시각 수용과 같은 선언 글꼴 환경에서 관계를 검사한다.
    // 미달이 남은 기본 환경의 corpus 수용을 대신하지 않는다.
    let environment = rhwp::renderer::font_environment::FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    ))
    .expect("독립 Print와 대조한 선언 글꼴 환경");
    doc.set_font_environment(Some(environment))
        .expect("측정과 paint에 같은 선언 환경 적용");
    assert_eq!(doc.page_count(), 66, "독립 Print PDF와 전체 쪽수");
    // 독립 Print 2→3쪽 경계: 영향평가 내용은 앞쪽에서 끝나지만 같은
    // 행의 빈 물리 공간은 뒤쪽 상단에 남는다. 비용 행은 그 공간 뒤에서
    // 시작해야 하며, 내용이 끝났다는 이유로 행 상자를 생략하지 않는다.
    let opening = doc.build_page_render_tree(1).expect("2쪽 렌더");
    let closing = doc.build_page_render_tree(2).expect("3쪽 렌더");
    let first_table = owned_table(&opening.root, 13).expect("앞 규제 표 조각");
    let next_table = owned_table(&closing.root, 13).expect("뒤 규제 표 조각");
    for (tree, table) in [(&opening.root, first_table), (&closing.root, next_table)] {
        let area = body(tree).expect("문서 본문");
        assert!(
            table.bbox.y >= area.bbox.y - FLOAT_EPSILON
                && table.bbox.y + table.bbox.height
                    <= area.bbox.y + area.bbox.height + FLOAT_EPSILON,
            "행 조각의 예약과 실제 상자는 본문 세로 영역 안에 있어야 함"
        );
    }
    let cost = source_cell(next_table, 10, 2);
    for col in [2, 6, 10] {
        let first = source_cell(first_table, 9, col);
        let remaining = source_cell(next_table, 9, col);
        assert_eq!(text(first), "해당없음", "영향평가 응답은 앞쪽 소유");
        assert!(text(remaining).is_empty(), "응답을 뒤쪽에 중복하지 않음");
        assert!(
            remaining.bbox.height > 0.0,
            "소비한 내용 뒤의 물리 잔여 보존"
        );
        assert!(
            (remaining.bbox.y - next_table.bbox.y).abs() <= FLOAT_EPSILON
                && (remaining.bbox.y + remaining.bbox.height - cost.bbox.y).abs() <= FLOAT_EPSILON,
            "같은 빈 행의 잔여가 표 위에서 비용 행까지 이어져야 함"
        );
    }
    let cost_marker = "종전과동일한수준으로서";
    assert!(
        !text(first_table).contains(cost_marker),
        "비용 본문을 앞쪽에 소비하지 않음"
    );
    assert_eq!(
        text(cost).matches(cost_marker).count(),
        1,
        "비용 본문은 뒤쪽에 한 번 보존"
    );
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

    // 전66쪽 Native/fresh 수용 후 보강한 경계다. 표 하단은 후속 원본
    // 문단의 원점에서 아래 바깥여백을 제외한 곳이다. 절대 px 위치나
    // 구현 helper의 반환값 대신 원본 속성과 실제 두 상자의 관계를 검사한다.
    let mut frame_failures = Vec::new();
    for (page, host) in [(56, 517), (57, 520)] {
        let Control::Table(original) = &source_paras[host].controls[0] else {
            panic!("원본 종료 프레임의 표 소실");
        };
        let next = &source_paras[host + 1].line_segs[0];
        let original_cell = &original.cells[0];
        assert_eq!(
            i64::from(original_cell.height) - i64::from(original.common.height)
                + i64::from(original.outer_margin_top)
                + i64::from(original.outer_margin_bottom),
            i64::from(next.vertical_pos),
            "독립 원본의 두 표 상자와 후속 원점이 같은 프레임을 닫음"
        );
        let tree = doc.build_page_render_tree(page).expect("종료 프레임 렌더");
        let area = body(&tree.root).expect("원본 쪽 본문");
        let painted = owned_table(&tree.root, host).expect("원본 소유 종료 표");
        let cell = source_cell(painted, 0, 0);
        let successor_origin = hwpunit_to_px(next.vertical_pos, DEFAULT_DPI);
        let closed_origin = cell.bbox.y
            + cell.bbox.height
            + hwpunit_to_px(i32::from(original.outer_margin_bottom), DEFAULT_DPI)
            - area.bbox.y;
        if (closed_origin / successor_origin - 1.0).abs() > FLOAT_EPSILON {
            frame_failures.push(format!(
                "{}쪽: 표 하단과 후속 원점 사이 아래여백을 중복 예약함 (비율 {})",
                page + 1,
                closed_origin / successor_origin
            ));
        }
    }

    // 비점유 ColumnDef(1)이 있어도 첫 빈 저장 줄의 줄 상자는 보존한다.
    // 안내 제목과 중첩 표는 각자 원본의 상대 원점을 한 번만 사용한다.
    let Control::Table(original) = &source_paras[527].controls[0] else {
        panic!("원본 안내서식 소실");
    };
    let original_cell = &original.cells[0];
    let empty = &original_cell.paragraphs[0];
    assert!(empty.text.is_empty());
    assert!(empty.controls.iter().all(|control| {
        matches!(control, Control::ColumnDef(columns) if columns.column_count == 1)
    }));
    let original_title = &original_cell.paragraphs[1];
    let original_child = original_title
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 안내 제목의 중첩 표");
    let outer = owned_table(&before.root, 527).expect("59쪽 안내서식 소유 표");
    let outer_cell = source_cell(outer, 0, 0);
    let title = line(outer_cell, "자동화평가결과안내").expect("원본 안내 제목 줄");
    let empty_line = outer_cell.children.iter().find(|child| {
        matches!(&child.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(0))
    }).expect("원본 첫 빈 줄의 물리 상자");
    let source_lead = original_title.line_segs[0].vertical_pos - empty.line_segs[0].vertical_pos;
    assert_eq!(
        source_lead,
        empty.line_segs[0].line_height + empty.line_segs[0].line_spacing
    );
    if ((title.bbox.y - empty_line.bbox.y) / hwpunit_to_px(source_lead, DEFAULT_DPI) - 1.0).abs()
        > FLOAT_EPSILON
    {
        frame_failures.push("59쪽: 첫 빈 줄과 제목의 원본 줄 상자 연결을 잃음".into());
    }
    let child = table_with(outer_cell, "귀하가");
    let child_origin = source_cell(child, 0, 0).bbox.y;
    let source_offset = hwpunit_to_px(original_child.common.vertical_offset as i32, DEFAULT_DPI);
    assert!(source_offset > 0.0, "독립 원본의 양수 문단 상대 앵커");
    if ((child_origin - title.bbox.y) / source_offset - 1.0).abs() > FLOAT_EPSILON {
        frame_failures.push(format!(
            "59쪽: 제목과 중첩 표가 서로 다른 원점을 소비함 (비율 {})",
            (child_origin - title.bbox.y) / source_offset
        ));
    }

    // 마지막 내부 표의 정확한 원본 컷 뒤에는 이미 소유한 유닛을 다시
    // 예약하지 않는다. 두 쪽의 텍스트가 원문을 정확히 한 번 분할하고,
    // 마지막 빈 host 줄 뒤의 여유가 또 하나의 줄 상자가 되지 않아야 한다.
    let Control::Table(original) = &source_paras[619].controls[0] else {
        panic!("원본 마지막 설명 표 소실");
    };
    let original_cell = original
        .cells
        .iter()
        .find(|cell| cell.row == 4 && cell.col == 1)
        .expect("원본 마지막 설명 칸");
    let original_child = original_cell.paragraphs[0]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Table(table) = control {
                Some(table.as_ref())
            } else {
                None
            }
        })
        .expect("원본 설명의 내부 표");
    let expected_text: String = original_child.cells[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.text.chars())
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let page65 = doc.build_page_render_tree(64).expect("65쪽 내부 표 시작");
    let page66 = doc.build_page_render_tree(65).expect("66쪽 내부 표 종료");
    let opening = source_cell(owned_table(&page65.root, 619).expect("앞 설명 표"), 4, 1);
    let closing = source_cell(owned_table(&page66.root, 619).expect("뒤 설명 표"), 4, 1);
    let first_child = table_with(opening, "인공지능");
    let last_child = table_with(closing, "맞추어");
    assert_eq!(
        text(first_child) + &text(last_child),
        expected_text,
        "정확한 자식 컷이 원본 내용을 누락·중복 없이 두 쪽에 한 번 보존"
    );
    let last_para = original_cell.paragraphs.len() - 1;
    assert!(original_cell.paragraphs[last_para].text.is_empty());
    let tail = closing.children.iter().find(|child| {
        matches!(&child.node_type, RenderNodeType::TextLine(data) if data.para_index == Some(last_para))
    }).expect("내부 표 뒤의 원본 빈 host 줄");
    let remaining_band = closing.bbox.y + closing.bbox.height - (tail.bbox.y + tail.bbox.height);
    if remaining_band < -FLOAT_EPSILON || remaining_band >= tail.bbox.height {
        frame_failures.push(format!(
            "66쪽: 정확한 자식 컷 뒤에 이미 소비한 줄 상자를 다시 예약함 (꼬리/줄 비율 {})",
            remaining_band / tail.bbox.height
        ));
    }
    assert!(
        frame_failures.is_empty(),
        "원본 종료 프레임의 관계 위반: {frame_failures:#?}"
    );
}
