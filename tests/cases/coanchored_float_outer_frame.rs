//! 같은 빈 호스트의 형제 표는 원문 순서와 바깥 여백 상자를 보존한다.
//! 독립 Print와 Native/fresh WASM 전쪽100%, 중요2쪽100% 증거는 제출 asset에 보존한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const HWP: &[u8] = include_bytes!("../../samples/issue3587/c-form-labnote-001-stage11-filled.hwp");
const HWPX: &[u8] =
    include_bytes!("../../samples/issue3587/c-form-labnote-001-stage11-filled.hwpx");
const ZERO: &[u8] =
    include_bytes!("../../mydocs/pr/assets/coanchored_float_outer_frame_20261020/outer-zero.hwpx");
const TOP: &[u8] = include_bytes!(
    "../../mydocs/pr/assets/coanchored_float_outer_frame_20261020/outer-top-only.hwpx"
);
const BOTTOM: &[u8] = include_bytes!(
    "../../mydocs/pr/assets/coanchored_float_outer_frame_20261020/outer-bottom-only.hwpx"
);

fn normalize(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text(child, out);
    }
}
fn tables<'a>(node: &'a RenderNode, owner: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.section_index == Some(0) && table.para_index == Some(owner)
            && table.cell_context.is_none())
    {
        out.push(node);
    }
    for child in &node.children {
        tables(child, owner, out);
    }
}
fn check(bytes: &[u8], coordinates: bool) {
    let core = DocumentCore::from_bytes(bytes).unwrap();
    assert_eq!(core.page_count(), 2, "독립 Print 전쪽 수와 소유 검사");
    let owners = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, para)| {
            para.controls
                .iter()
                .filter(|c| matches!(c, Control::Table(_)))
                .count()
                == 3
        })
        .collect::<Vec<_>>();
    assert_eq!(owners.len(), 1, "원본에서 독립적으로 확인한 세 표의 호스트");
    let (owner, para) = owners[0];
    let source = para
        .controls
        .iter()
        .enumerate()
        .filter_map(|(ci, control)| {
            let Control::Table(table) = control else {
                return None;
            };
            assert!(!table.common.treat_as_char);
            Some((ci, table))
        })
        .collect::<Vec<_>>();
    let mut placed = Vec::<(u32, usize, BoundingBox, String)>::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        tables(&tree.root, owner, &mut nodes);
        for node in nodes {
            let RenderNodeType::Table(table) = &node.node_type else {
                unreachable!()
            };
            let ci = table.control_index.expect("실제 원문 컨트롤 소유");
            let original = source.iter().find(|(index, _)| *index == ci).unwrap().1;
            assert_eq!(table.row_count, original.row_count, "행 누락·불필요한 분할");
            assert_eq!(table.col_count, original.col_count);
            let mut rendered = String::new();
            text(node, &mut rendered);
            let mut cells = original.cells.iter().collect::<Vec<_>>();
            cells.sort_by_key(|cell| (cell.row, cell.col));
            let expected = cells
                .iter()
                .flat_map(|cell| &cell.paragraphs)
                .map(|p| normalize(&p.text))
                .collect::<String>();
            assert_eq!(normalize(&rendered), expected, "셀 내용의 순서·누락·중복");
            placed.push((page, ci, node.bbox, normalize(&rendered)));
        }
    }
    assert_eq!(placed.len(), source.len(), "표 누락·중복·불필요한 이어받기");
    placed.sort_by_key(|record| record.1);
    for (index, (page, ci, _, _)) in placed.iter().enumerate() {
        assert_eq!(*page, 1, "세 표는 독립 Print의2쪽에 함께 소속된다");
        assert_eq!(*ci, source[index].0, "원문 컨트롤 순서");
    }
    for index in 1..placed.len() {
        let previous = placed[index - 1].2;
        let next = placed[index].2;
        let gap = next.y - previous.y - previous.height;
        let original_hu = i32::from(source[index - 1].1.outer_margin_bottom)
            + i32::from(source[index].1.outer_margin_top);
        let expected_gap = rhwp::renderer::hwpunit_to_px(original_hu, 96.0);
        assert!(
            (gap - expected_gap).abs() < 1e-7,
            "앞 표 아래·뒤 표 위 여백을 한 번씩 점유한다: ci={}, gap={gap}, source={original_hu}HU",
            placed[index].1
        );
        // HU 환산의 부동소수점 합산 오차만 허용한다.
        assert!(gap >= -1e-7, "형제 표 겹침");
    }
    if coordinates {
        // 2쪽은 양쪽 raw100%와 직접 판독을 마친 페이지다. 96DPI 종이 원점,
        // 독립 PDF vector 외곽선(72dpi×96/72)을 ±2px로 보호한다.
        for (actual, expected_y) in placed.iter().zip([41.5546875, 136.01066081, 994.11197917]) {
            assert!(
                (actual.2.x - 41.57733409).abs() <= 2.0,
                "독립 Print의 왼쪽 외곽"
            );
            assert!(
                (actual.2.y - expected_y).abs() <= 2.0,
                "독립 Print의 위쪽 외곽"
            );
        }
        let last = placed.last().unwrap().2;
        assert!(
            (last.y + last.height - 1065.87337240).abs() <= 2.0,
            "서명 표 끝과 후속 공간"
        );
    }
}
#[test]
fn native_hwp_sibling_tables_preserve_source_order_content_and_outer_gaps() {
    check(HWP, false);
}
#[test]
fn hwp_origin_hwpx_sibling_tables_preserve_source_order_content_and_outer_gaps() {
    check(HWPX, false);
}
#[test]
fn native_hwp_completed_second_page_keeps_independent_print_boundary_coordinates() {
    check(HWP, true);
}
#[test]
fn hwp_origin_hwpx_completed_second_page_keeps_independent_print_boundary_coordinates() {
    check(HWPX, true);
}
#[test]
fn zero_outer_margins_do_not_invent_a_gap_between_sibling_tables() {
    check(ZERO, false);
}
#[test]
fn source_outer_top_alone_is_consumed_once_between_sibling_tables() {
    check(TOP, false);
}
#[test]
fn source_outer_bottom_alone_is_consumed_once_between_sibling_tables() {
    check(BOTTOM, false);
}
