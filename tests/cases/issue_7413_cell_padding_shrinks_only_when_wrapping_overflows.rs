//! [#7413] 칸 안 여백 축소는 **줄바꿈으로 해결되지 않을 때만** 발화한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `shrunk_cell_horizontal_padding` 은 칸 문단의 **자연 폭**(줄바꿈하지 않은 한 줄 폭)이
//! 가용 너비의 1.15배를 넘으면 선언된 칸 안 여백을 양쪽 1.0px 까지 깎았다. 그 조건은
//! "이 문단은 줄바꿈이 필요하다"는 뜻일 뿐이라, **줄바꿈하면 칸에 들어가는 문단까지 전부**
//! 걸렸다.
//!
//! 축소가 실제로 지탱하는 계약은 "칸 안 글자가 칸 밖으로 새지 않는다" 하나다. 축소를
//! 통째로 끄고 `samples/**` 전수를 재면 겹침이 4건 늘고, 그 넷이 전부 **줄바꿈 결과가
//! 행 높이를 넘는** 자리였다.
//!
//! 기존 영월군 검사는 `월별누계`를 한 줄로 요구했다. 원본 HWP3의 해당 줄은
//! 시작·좌표·구분 정보가 모두 비어 있는 글자 높이 레코드라 저장 줄 구성의 근거가 아니다.
//! 독립 한컴 2020 Print와 같은 원본의 한컴 HWPX 변환본은 `월별` / `누계` 두 줄이다.
//! 마지막 줄 뒤 간격을 제외한 두 줄은 원본 셀 높이에 들어간다.
//!
//! 그래서 묻는 것을 바꿨다 — **줄바꿈한 결과가 칸 높이를 넘는가.** 넘지 않으면 줄바꿈이
//! 답이고 선언 여백을 그대로 둔다.
//!
//! # 현재 검사 범위
//!
//! 이 파일은 `table-text.hwpx`의 선언 여백 검사만 유지한다.
//! 영월 서식의 고정 위치·한 줄 기대값은 독립 한컴 2020 Print와 상충한다.
//! Native/fresh WASM 전 4쪽 최저 99.29% 확인 후 새 파일
//! `issue_7642_yeongwol_monthly_cell_relations.rs`에서 원본 소유·월별/누계 두 줄·
//! 순서·비겹침·셀 내부 포함을 검사한다. 다른 입력의 검사는 일괄 제거하지 않는다.
//!
//! `table-text.hwpx`의 기존 선언 여백 검사는 별도의 Print 출처 확인 대상이다.
//! 영월 검증 결과를 이 입력의 시각 근거로 사용하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 줄바꿈이 칸 높이에 들어가는 칸 — 선언 여백이 남아야 한다.
const FITS: &str = "samples/hwpx/table-text.hwpx";

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// `(칸 bbox, 글자 왼끝, 글자 오른끝)` 를 칸 단위로 모은다.
fn cell_text_extents(
    node: &RenderNode,
    cell: Option<&RenderNode>,
    out: &mut Vec<(f64, f64, f64, f64)>,
) {
    let next_cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (RenderNodeType::TextRun(_), Some(c)) = (&node.node_type, next_cell) {
        out.push((
            c.bbox.x,
            c.bbox.width,
            node.bbox.x,
            node.bbox.x + node.bbox.width,
        ));
    }
    for child in &node.children {
        cell_text_extents(child, next_cell, out);
    }
}

/// 줄바꿈으로 해결되는 칸은 선언 안 여백을 지킨다.
#[test]
fn a_cell_that_wraps_within_its_height_keeps_its_declared_padding() {
    use rhwp::model::control::Control;
    use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

    fn text(node: &RenderNode) -> String {
        if !node.visible || node.editor_only {
            return String::new();
        }
        let mut out = match &node.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
            _ => String::new(),
        };
        for child in &node.children {
            out.push_str(&text(child));
        }
        out
    }
    fn find_table(node: &RenderNode, owner: (usize, usize, usize)) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Table(meta)
            if (meta.section_index, meta.para_index, meta.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2)) && meta.cell_context.is_none())
        {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_table(child, owner))
    }
    fn cell_nodes<'a>(node: &'a RenderNode, index: usize, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::TableCell(meta)
            if meta.model_cell_index == Some(index as u32))
        {
            out.push(node);
            return;
        }
        for child in &node.children {
            cell_nodes(child, index, out);
        }
    }
    fn text_extents(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if !node.visible || node.editor_only {
            return;
        }
        if matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.display_or_text().trim().is_empty())
        {
            out.push((node.bbox.x, node.bbox.x + node.bbox.width));
        }
        for child in &node.children {
            text_extents(child, out);
        }
    }

    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FITS)).unwrap();
    let source = rhwp::parse_document(&bytes).expect("원본 표 속성");
    let mut owners = Vec::new();
    for (si, section) in source.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                if let Control::Table(table) = control {
                    if let Some(amount) = table.cells.iter().find(|cell| {
                        cell.paragraphs
                            .iter()
                            .map(|p| p.text.as_str())
                            .collect::<String>()
                            == "65,063,026,600"
                    }) {
                        owners.push(((si, pi, ci), table, amount.col));
                    }
                }
            }
        }
    }
    assert_eq!(owners.len(), 1, "독립 Print의 첫 금액 열 소유가 유일하다");
    let (owner, table, column) = owners[0];
    let targets: Vec<_> = table
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.col == column && cell.col_span == 1)
        .collect();
    assert_eq!(
        targets
            .iter()
            .map(|(_, cell)| cell
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>())
            .collect::<Vec<_>>(),
        ["2023년", "65,063,026,600"],
        "연도와 금액의 원본 순서"
    );
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        1,
        "독립 Print와 같은 한 쪽에 표가 완결된다"
    );
    let page = core.build_page_render_tree(0).expect("원문 표의 소속 쪽");
    let painted_table = find_table(&page.root, owner).expect("실제 paint의 원본 표 소유");
    for (index, original) in targets {
        let mut cells = Vec::new();
        cell_nodes(painted_table, index, &mut cells);
        assert_eq!(cells.len(), 1, "대상 셀의 누락·중복이 없다");
        let cell = cells[0];
        let expected: String = original
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect();
        assert_eq!(text(cell), expected, "셀 원문의 내용·순서를 보존한다");
        // 독립 한컴 Print의 clip/괘선 사각형(pt)을 96DPI CSS px로 변환했다.
        // 해당 1쪽의 Native/fresh WASM 전쪽 98% 이상·확대 판독을 확인한 px 계약이다.
        // 위치로 대상을 선택하지 않으며 위의 원문·셀 소속·순서 검사를 함께 유지한다.
        let expected_y = if expected == "2023년" {
            165.4186666666666
        } else {
            201.85866666666666
        };
        for (actual, independent) in [
            (cell.bbox.x, 115.29733333333333),
            (cell.bbox.y, expected_y),
            (cell.bbox.width, 76.28),
            (cell.bbox.height, 36.44),
        ] {
            assert!(
                (actual - independent).abs() <= 2.0,
                "독립 Print 셀 상자의 ±2 CSSpx 계약: 실제={actual}, 독립={independent}"
            );
        }
        let padding = original.effective_padding(&table.padding);
        let left = cell.bbox.x + hwpunit_to_px(i32::from(padding.left), DEFAULT_DPI);
        let right =
            cell.bbox.x + cell.bbox.width - hwpunit_to_px(i32::from(padding.right), DEFAULT_DPI);
        let mut extents = Vec::new();
        text_extents(cell, &mut extents);
        assert!(!extents.is_empty(), "셀 안에 실제 글자가 있어야 한다");
        let source_unit = hwpunit_to_px(1, DEFAULT_DPI);
        for (start, end) in extents {
            assert!(start + source_unit >= left && end <= right + source_unit,
                "줄이 셀에 들어가면 선언 안 여백을 줄이지 않는다: 글자={start}..{end}, 원본 내부={left}..{right}");
        }
    }
}
