//! #8160: ubuntu-ted의 sample16 검사 제거에서 누락된 저장 도형 호스트 회귀 복원.
//! 원본: 3859d1948444deadc6a5bc6e5343911f870f97f1, 제거: 1ef34f94452493db15a8e33e641f97b81ed74916.
//! 저장된 도형 호스트는 실제 배치 프레임과 뒤 표의 쪽 소속을 보존한다.
//! Native/fresh WASM 전쪽·8쪽100%와 독립 Print 좌표는
//! mydocs/pr/assets/stored_shape_host_frame_20261020/README.md에 보존한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HWP: &[u8] = include_bytes!("../../samples/hwp3-sample16-hwp5.hwp");
const HWPX: &[u8] = include_bytes!("../../samples/hwp3-sample16-hwp5.hwpx");
const HEADING: &str = "제안서 작성목차 및 관련서식";

fn load(bytes: &[u8]) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(bytes).expect("전쪽 검증한 원본");
    core.set_hangul2024_compat(true);
    core
}

fn indices(core: &DocumentCore) -> (usize, usize) {
    let paragraphs = &core.document().sections[0].paragraphs;
    let heading = paragraphs
        .iter()
        .position(|p| p.text.contains(HEADING))
        .unwrap();
    let table = paragraphs
        .iter()
        .enumerate()
        .skip(heading + 1)
        .find(|(_, p)| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .map(|(i, _)| i)
        .unwrap();
    (heading, table)
}

fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}

fn is_heading(node: &RenderNode, owner: usize) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.section_index == Some(0) && run.para_index == Some(owner)
            && run.cell_context.is_none() && run.display_or_text().contains(HEADING))
}
fn is_table(node: &RenderNode, owner: usize) -> bool {
    matches!(&node.node_type, RenderNodeType::Table(table)
        if table.section_index == Some(0) && table.para_index == Some(owner)
            && table.cell_context.is_none())
}
fn baseline(node: &RenderNode) -> f64 {
    let RenderNodeType::TextRun(run) = &node.node_type else {
        panic!("글자 기준선")
    };
    node.bbox.y + run.baseline
}
fn normalize(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn assert_source_order(bytes: &[u8]) {
    let core = load(bytes);
    // 독립 Print의 전쪽 수와 아래 내용 소속·순서·완전성을 함께 보호한다.
    assert_eq!(core.page_count(), 64, "원본 Print와 다른 쪽수");
    let (heading_owner, table_owner) = indices(&core);
    let source = &core.document().sections[0].paragraphs[heading_owner];
    let shape = source
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Shape(shape) => Some(shape),
            _ => None,
        })
        .unwrap();
    // 큰 내부 좌표와 작은 실제 프레임이 공존하는 원본을 유지한다.
    assert!(shape.shape_attr().current_height > shape.common().height);
    assert_eq!(
        source.line_segs[0].line_height as u32,
        shape.common().height
    );
    let source_table = core.document().sections[0].paragraphs[table_owner]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    let labels: Vec<_> = source_table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|p| normalize(&p.text))
        .filter(|s| !s.is_empty())
        .collect();
    assert!(labels.len() > 1);
    let (mut headings, mut tables) = (Vec::new(), Vec::new());
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut all = Vec::new();
        nodes(&tree.root, &mut all);
        for node in all {
            if is_heading(node, heading_owner) {
                headings.push((page, baseline(node)));
            }
            if is_table(node, table_owner) {
                let mut children = Vec::new();
                nodes(node, &mut children);
                let rendered: String = children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                        _ => None,
                    })
                    .collect();
                tables.push((page, node.bbox, normalize(&rendered)));
            }
        }
    }
    assert_eq!(headings.len(), 1, "제목 누락·중복");
    assert_eq!(tables.len(), 1, "뒤 표 누락·중복·불필요한 분할");
    assert_eq!(headings[0].0, tables[0].0, "제목과 뒤 표의 쪽 소속");
    assert!(headings[0].1 < tables[0].1.y, "표는 제목 기준선 뒤에 온다");
    let expected = labels.concat();
    let mut after = 0;
    for label in labels {
        let at = tables[0].2[after..]
            .find(&label)
            .unwrap_or_else(|| panic!("표 내용 누락 또는 순서 변경: {label}"));
        after += at + label.len();
    }
    assert_eq!(tables[0].2, expected, "표 전체 내용의 추가·누락·중복");
}

fn assert_print_coordinates(bytes: &[u8]) {
    let core = load(bytes);
    let (heading_owner, table_owner) = indices(&core);
    let tree = core.build_page_render_tree(7).unwrap();
    let mut all = Vec::new();
    nodes(&tree.root, &mut all);
    let heading: Vec<_> = all
        .iter()
        .filter(|node| is_heading(node, heading_owner))
        .collect();
    assert_eq!(heading.len(), 1);
    // 96DPI의 Print 원시 텍스트 원점 265.67999267578125pt × 96/72.
    // 이 페이지는 Native/fresh WASM 모두 raw100%이고 직접 확대 판독했다.
    assert!(
        (baseline(heading[0]) - 354.239990234375).abs() <= 2.0,
        "Print 제목 기준선"
    );
    let table: Vec<_> = all
        .iter()
        .filter(|node| is_table(node, table_owner))
        .collect();
    assert_eq!(table.len(), 1);
    let bbox = table[0].bbox;
    // 독립 PDF의 상단274.5pt, 하단745.9439697265625pt를 같은 좌표계로 환산했다.
    assert!((bbox.y - 366.0).abs() <= 2.0, "Print 표 시작");
    assert!(
        (bbox.y + bbox.height - 994.5919596354166).abs() <= 2.0,
        "Print 표 종료"
    );
    assert!(
        baseline(heading[0]) < bbox.y,
        "좌표 핀과 내용 순서 관계를 함께 유지"
    );
}

#[test]
fn issue_8160_hwp_stored_shape_keeps_following_table_on_its_heading_page() {
    assert_source_order(HWP);
}
#[test]
fn issue_8160_hwpx_stored_shape_keeps_following_table_on_its_heading_page() {
    assert_source_order(HWPX);
}
#[test]
fn issue_8160_hwp_completed_page_protects_print_heading_and_table_boundaries() {
    assert_print_coordinates(HWP);
}
#[test]
fn issue_8160_hwpx_completed_page_protects_print_heading_and_table_boundaries() {
    assert_print_coordinates(HWPX);
}
