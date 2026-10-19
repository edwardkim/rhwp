//! 여백으로 나뉜 일반 행은 내용 컷뿐 아니라 원본 셀의 물리 공간도 보존한다.
//!
//! 같은 입력의 독립 Print PDF와 전3쪽 Native/fresh WASM 선행 검증:
//! task_m100_7445_rebase_repair_stage8.md C437의 ordinary-partial-row-frame 증적.
//! 이어받기3쪽은 양쪽 raw100%, 앞쪽2쪽은 각각98.54204/98.55721%다.
//! 좌표는 독립 PDF 벡터에서 얻은96dpi CSS px이며 ±2px와 내용 관계를 함께 검사한다.

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn open(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(path).expect("독립 PDF 대응 원본"))
        .expect("원본 문서 열기")
}

fn text(node: &RenderNode) -> String {
    let mut value = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.clone(),
        _ => String::new(),
    };
    for child in &node.children {
        value.push_str(&text(child));
    }
    value
}

fn row_cells<'a>(node: &'a RenderNode, row: u16, result: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(29) && table.row_count == 8 && table.col_count == 2 {
            for child in &node.children {
                if matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == row) {
                    result.push(child);
                }
            }
            return;
        }
    }
    for child in &node.children {
        row_cells(child, row, result);
    }
}

struct Fragment {
    page: u32,
    y: f64,
    height: f64,
    label: String,
    body: String,
    next_row_y: Option<f64>,
}

fn fragments(core: &DocumentCore) -> Vec<Fragment> {
    let mut result = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 쪽 배치");
        let mut cells = Vec::new();
        row_cells(&tree.root, 4, &mut cells);
        if cells.is_empty() {
            continue;
        }
        assert_eq!(cells.len(), 2, "원본 두 열의 단일 소유");
        let cell_at = |column| {
            *cells
                .iter()
                .find(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.col == column))
                .expect("원본 열")
        };
        let left = cell_at(0);
        let right = cell_at(1);
        assert!((left.bbox.y - right.bbox.y).abs() < 1e-6, "같은 행의 시작");
        assert!(
            (left.bbox.height - right.bbox.height).abs() < 1e-6,
            "내용이 끝난 왼쪽 칸도 오른쪽 칸과 물리 상자를 공유"
        );
        let mut next_cells = Vec::new();
        row_cells(&tree.root, 5, &mut next_cells);
        result.push(Fragment {
            page,
            y: left.bbox.y,
            height: left.bbox.height,
            label: text(left),
            body: text(right),
            next_row_y: next_cells.first().map(|node| node.bbox.y),
        });
    }
    result
}

fn split_contract(sample: &str) {
    let core = open(sample);
    assert_eq!(core.page_count(), 3, "독립 Print의 전3쪽 소유");
    let Control::Table(table) = &core.document().sections[0].paragraphs[29].controls[0] else {
        panic!("원본 표");
    };
    let source_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == 4 && cell.col == 0)
        .expect("원본의 독립 최소 높이 소유 칸");
    let source_minimum = f64::from(source_cell.height) * 96.0 / 7200.0;
    let parts = fragments(&core);
    assert_eq!(
        parts.iter().map(|part| part.page).collect::<Vec<_>>(),
        [1, 2],
        "같은 행은 독립 Print의 앞뒤 두 쪽에만 속한다"
    );
    assert!(parts[0].label.contains("2차 면접") && parts[1].label.is_empty());
    for (part, expected, absent) in [
        (&parts[0], "임원면접", "일정 및 세부사항"),
        (&parts[1], "일정 및 세부사항", "임원면접"),
    ] {
        assert_eq!(part.body.matches(expected).count(), 1, "문단 누락·중복");
        assert!(!part.body.contains(absent), "앞뒤 문단의 쪽 소유");
    }
    let accepted = parts.iter().map(|part| part.height).sum::<f64>();
    assert!(
        accepted + 1e-6 >= source_minimum,
        "앞뒤 수용 상자가 원본 최소를 보존해야 함: {accepted} < {source_minimum}"
    );
    // 같은 동작의 완료 페이지 중 양쪽 raw100%인3쪽을 우선 보호한다.
    // Print의 위 괘선과 다음 행 시작: 77.356 / 107.5626667 CSS px.
    for (actual, expected) in [
        (parts[1].y, 77.356),
        (parts[1].y + parts[1].height, 107.562_666_666_666_67),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "{sample} 3쪽 이어받기 경계: actual={actual}, independent={expected}"
        );
    }
    let next_y = parts[1].next_row_y.expect("이어받기 뒤의 최종합격 행");
    assert!(
        (next_y - parts[1].y - parts[1].height).abs() < 1e-6,
        "다음 행은 수용한 물리 상자의 끝에서 이어져야 한다"
    );
}

#[test]
fn footer_margin_partial_row_preserves_source_space() {
    split_contract("samples/issue1133/page-number-footer15.hwpx");
}

#[test]
fn bottom_margin_partial_row_preserves_source_space() {
    split_contract("samples/issue1133/page-number-bottom15.hwpx");
}

#[test]
fn completed_original_frame_does_not_create_an_empty_tail() {
    for sample in ["samples/issue_1133.hwp", "samples/hwpx/issue_1133.hwpx"] {
        let core = open(sample);
        assert_eq!(core.page_count(), 3, "완료한 원본 행의 쪽 소유");
        let parts = fragments(&core);
        assert_eq!(
            parts.len(),
            1,
            "완료한 원본 행에 새 빈 꼬리를 만들지 않는다"
        );
        assert_eq!(parts[0].page, 1);
        assert_eq!(parts[0].body.matches("임원면접").count(), 1);
        assert_eq!(parts[0].body.matches("일정 및 세부사항").count(), 1);
    }
}
