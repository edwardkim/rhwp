//! Issue #4138: 저장된 분할 셀의 텍스트 프레임과 개체 앵커를 구분한다.
//!
//! 과거 native 분할 저장본 d9bc와 그 입력의 독립 한컴2024 Print를 사용한다.
//! 새 생성본으로 바꾸지 않는다. 검사 범위 25/60/89/156쪽은 Native/fresh WASM
//! 모두90% 이상이며 stage5 C43의 전쪽 TSV·review에 연결한다. 이97%대 쪽은
//! 절대 px 좌표를 고정하지 않고 줄 소속·원본 보존·개체 포함 관계를 검증한다.
use rhwp::model::{control::Control, document::Document, table::Cell};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn input() -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/issue4138/tab-period-split.hwp"),
    )
    .expect("독립 Print와 대응하는 과거 분할 저장본")
}
fn body(document: &Document) -> &Cell {
    let Control::Table(table) = &document.sections[0].paragraphs[0].controls[2] else {
        panic!("원문 표");
    };
    table
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap()
}
fn outer_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(0) && table.control_index == Some(2)
            && table.cell_context.is_none())
    {
        return Some(node);
    }
    node.children.iter().find_map(outer_table)
}
fn source_cell(table: &RenderNode) -> &RenderNode {
    table
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TableCell(cell)
            if cell.row == 2 && cell.col == 0)
        })
        .unwrap()
}
fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.display_or_text().to_owned();
    }
    node.children.iter().map(text).collect()
}
fn normalized(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn invalid_text_frame_is_repaired_without_replacing_a_valid_stored_partition() {
    let bytes = input();
    let source = rhwp::parser::parse_hwp(&bytes).unwrap();
    let mut document = DocumentCore::from_bytes(&bytes).unwrap();
    document.set_hangul2024_compat(true);
    // 같은 원점·폭의89쪽 저장 컷은 유효하다. 원래 줄을 폭 추정으로 다시 합치지 않는다.
    let stored_keys = |cell: &Cell| {
        cell.paragraphs[1189]
            .line_segs
            .iter()
            .map(|line| (line.text_start, line.column_start, line.segment_width))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        stored_keys(body(document.document())),
        stored_keys(body(&source))
    );
    for (page, owner, independent_line) in [
        (59, 749, "는 최소 1개의 대체장치로 설치"),
        (88, 1189, "되는 구역은 난연수지 3회 도포 또"),
    ] {
        let tree = document.build_page_render_tree(page).unwrap();
        let cell = source_cell(outer_table(&tree.root).unwrap());
        let lines = cell
            .children
            .iter()
            .filter(|line| {
                matches!(&line.node_type, RenderNodeType::TextLine(data)
                if data.para_index == Some(owner))
            })
            .collect::<Vec<_>>();
        assert!(
            lines
                .iter()
                .any(|line| normalized(&text(line)) == normalized(independent_line)),
            "독립 Print의 문단{owner} 줄 소속"
        );
        let actual: String = lines.iter().map(|line| text(line)).collect();
        assert_eq!(
            normalized(&actual),
            normalized(&body(&source).paragraphs[owner].text),
            "같은 쪽의 원문 문단이 누락·중복 없이 한 번 표시됨"
        );
        for pair in lines.windows(2) {
            assert!(
                pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
                "동일 문단의 연속 줄 상자 비겹침"
            );
        }
        assert!(
            lines.iter().all(|line| line.bbox.y >= cell.bbox.y
                && line.bbox.y + line.bbox.height <= cell.bbox.y + cell.bbox.height),
            "본문 줄은 소유 셀 조각 안에 있음"
        );
    }
}

#[test]
fn table_prefix_keeps_its_saved_anchor_and_all_nested_rows() {
    let bytes = input();
    let source = rhwp::parser::parse_hwp(&bytes).unwrap();
    let mut document = DocumentCore::from_bytes(&bytes).unwrap();
    document.set_hangul2024_compat(true);
    for (page, owner) in [(24, 322), (155, 2001)] {
        let original = &body(&source).paragraphs[owner];
        assert!(!original.text.is_empty() && original.text.chars().all(char::is_whitespace));
        let Control::Table(nested) = &original.controls[0] else {
            panic!("원문 중첩 표")
        };
        assert!(nested.common.treat_as_char);
        let actual = &body(document.document()).paragraphs[owner];
        // 누적 vpos는 앞 문단의 복구를 반영한다. 나머지 저장 앵커/줄 상자는 그 개체의
        // 계약이며 텍스트 폭 재조판의 산출물로 대체하지 않는다.
        let anchor = |para: &rhwp::model::paragraph::Paragraph| {
            para.line_segs
                .iter()
                .map(|line| {
                    (
                        line.text_start,
                        line.column_start,
                        line.segment_width,
                        line.line_height,
                        line.text_height,
                        line.baseline_distance,
                        line.line_spacing,
                        line.tag,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            anchor(actual),
            anchor(original),
            "공백·TAC 표의 저장 앵커 보존"
        );
        assert_eq!(actual.text, original.text, "표 앞 원문 공백 보존");
        assert_eq!(
            actual.control_text_positions(),
            original.control_text_positions()
        );
        let Control::Table(after) = &actual.controls[0] else {
            panic!("보존된 중첩 표")
        };
        assert_eq!(after.row_count, nested.row_count);
        assert_eq!(after.col_count, nested.col_count);
        let cell_texts = |table: &rhwp::model::table::Table| {
            table
                .cells
                .iter()
                .map(|cell| {
                    cell.paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            cell_texts(after),
            cell_texts(nested),
            "모든 원문 행과 셀 내용 보존"
        );
        let tree = document.build_page_render_tree(page).unwrap();
        let outer = outer_table(&tree.root).unwrap();
        fn nested_table(node: &RenderNode, owner: usize) -> Option<&RenderNode> {
            if matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(owner) && table.control_index == Some(0))
            {
                return Some(node);
            }
            node.children
                .iter()
                .find_map(|child| nested_table(child, owner))
        }
        let placed = nested_table(outer, owner).expect("독립 Print와 같은 쪽의 중첩 표");
        let RenderNodeType::Table(table) = &placed.node_type else {
            unreachable!()
        };
        assert_eq!(table.row_count, nested.row_count);
        assert_eq!(table.col_count, nested.col_count);
        assert!(
            placed.bbox.x >= outer.bbox.x
                && placed.bbox.x + placed.bbox.width <= outer.bbox.x + outer.bbox.width,
            "중첩 표가 원래 바깥 표의 가로 점유 안에 있음"
        );
    }
}
