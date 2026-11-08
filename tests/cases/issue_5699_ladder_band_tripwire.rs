//! [#5699] 자리차지 표와 뒤 본문의 소속·순서 보존.
//!
//! 영월 원본 HWP3에는 글자 높이만 남은 줄 레코드가 있다. 줄 구성·좌표가
//! 없는 레코드를 저장 사다리로 수용하면 줄바꿈과 표 밴드의 점유가 다르게 계산된다.
//! 독립 한컴 2020 Print 전4쪽과 Native/fresh WASM 최저 99.53097%를 대조했다.
//! 과거 SVG 글자 y>570 검사는 표/문단 소유와 앞뒤 포함 관계로 교체했다.
//! 쪽수와 각 서식의 내용 소속을 함께 검사한다. 원본과 PDF는 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5699/20099369_yeongwol_forms.hwp";

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("영월 원본 읽기")).expect("영월 원본 파싱")
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}

fn text(node: &RenderNode) -> String {
    if !node.visible || node.editor_only {
        return String::new();
    }
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn body_lines<'a>(
    node: &'a RenderNode,
    owner: (usize, usize),
    in_cell: bool,
    found: &mut Vec<&'a RenderNode>,
) {
    if !node.visible || node.editor_only {
        return;
    }
    let in_cell = in_cell || matches!(node.node_type, RenderNodeType::TableCell(_));
    if !in_cell {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)) {
                found.push(node);
            }
        }
    }
    for child in &node.children {
        body_lines(child, owner, in_cell, found);
    }
}

/// 독립 Print의 쪽수와 함께 각 별지 제목의 대응·누락·중복을 보호한다.
#[test]
fn issue_5699_yeongwol_page_count_matches_hangul_oracle() {
    let core = load();
    let titles: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(si, sec)| {
            sec.paragraphs
                .iter()
                .enumerate()
                .filter(|(_, p)| p.text.contains("별지 제"))
                .map(move |(pi, p)| ((si, pi), compact(&p.text)))
        })
        .collect();
    assert_eq!(titles.len(), 4, "원본 별지 4개");
    assert_eq!(core.page_count(), 4, "독립 한컴 Print 전4쪽");
    let trees: Vec<_> = (0..core.page_count())
        .map(|p| core.build_page_render_tree(p).unwrap())
        .collect();
    for (expected_page, (owner, source)) in titles.iter().enumerate() {
        let mut occurrences = Vec::new();
        for (page, tree) in trees.iter().enumerate() {
            let mut lines = Vec::new();
            body_lines(&tree.root, *owner, false, &mut lines);
            let painted = compact(&lines.iter().map(|line| text(line)).collect::<String>());
            if !painted.is_empty() {
                occurrences.push((page, painted));
            }
        }
        assert_eq!(
            occurrences,
            vec![(expected_page, source.clone())],
            "각 별지 제목은 대응 Print 쪽에서 원본 내용을 한 번 소유한다"
        );
    }
}

/// 계약조건 제목은 같은 원본의 선행 표 아래에 있어야 한다.
#[test]
fn issue_5699_yeongwol_body_starts_below_table_band() {
    fn tables<'a>(
        node: &'a RenderNode,
        owner: (usize, usize, usize),
        found: &mut Vec<&'a RenderNode>,
    ) {
        if !node.visible || node.editor_only {
            return;
        }
        if let RenderNodeType::Table(table) = &node.node_type {
            if (table.section_index, table.para_index, table.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2))
                && table.cell_context.is_none()
            {
                found.push(node);
            }
        }
        for child in &node.children {
            tables(child, owner, found);
        }
    }
    let core = load();
    let mut owners = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, paragraph) in section.paragraphs.iter().enumerate() {
            if compact(&paragraph.text) != "임대차계약조건" {
                continue;
            }
            let preceding = section.paragraphs[..pi]
                .iter()
                .enumerate()
                .rev()
                .find_map(|(table_pi, host)| {
                    host.controls
                        .iter()
                        .enumerate()
                        .find(|(_, c)| matches!(c, Control::Table(_)))
                        .map(|(ci, _)| (si, table_pi, ci))
                })
                .expect("계약조건 앞 원본 표");
            owners.push(((si, pi), preceding));
        }
    }
    assert_eq!(owners.len(), 1, "원본 계약조건 제목과 선행 표를 식별한다");
    let (paragraph, table) = owners[0];
    let mut matches = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut lines = Vec::new();
        body_lines(&tree.root, paragraph, false, &mut lines);
        if lines.is_empty() {
            continue;
        }
        let mut frames = Vec::new();
        tables(&tree.root, table, &mut frames);
        assert_eq!(frames.len(), 1, "제목과 선행 표는 같은 쪽을 소유한다");
        assert_eq!(
            compact(&lines.iter().map(|line| text(line)).collect::<String>()),
            "임대차계약조건"
        );
        for line in &lines {
            assert!(
                line.bbox.y >= frames[0].bbox.y + frames[0].bbox.height - 0.5,
                "계약조건 본문이 선행 표를 관통하면 안 된다"
            );
        }
        matches.push(page);
    }
    assert_eq!(matches.len(), 1, "원본 계약조건 제목의 누락·중복 방지");
}

/// 전4쪽이 양쪽90% 이상인 문서의 p4는 Native/WASM 모두100%다.
/// 같은 표 밴드의 시작·종료를 독립 Print 괘선으로 보호한다. p3 월별누계
/// 영역은 WASM97.72386%이므로 이 검사를 그 영역의 px 증거로 사용하지 않는다.
#[test]
fn issue_5699_final_ledger_preserves_its_owned_table_frame() {
    fn owned_tables<'a>(
        node: &'a RenderNode,
        owner: (usize, usize, usize),
        out: &mut Vec<&'a RenderNode>,
    ) {
        if !node.visible || node.editor_only {
            return;
        }
        if let RenderNodeType::Table(table) = &node.node_type {
            if (table.section_index, table.para_index, table.control_index)
                == (Some(owner.0), Some(owner.1), Some(owner.2))
                && table.cell_context.is_none()
            {
                out.push(node);
            }
        }
        for child in &node.children {
            owned_tables(child, owner, out);
        }
    }
    fn cells<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if !node.visible || node.editor_only {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            out.push(node);
        }
        for child in &node.children {
            cells(child, out);
        }
    }
    let core = load();
    let mut sources = Vec::new();
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            if compact(&para.text) != "임대료수납대장" {
                continue;
            }
            let (table_pi, ci, table) = section
                .paragraphs
                .iter()
                .enumerate()
                .skip(pi + 1)
                .find_map(|(table_pi, host)| {
                    host.controls.iter().enumerate().find_map(|(ci, c)| {
                        if let Control::Table(table) = c {
                            Some((table_pi, ci, table))
                        } else {
                            None
                        }
                    })
                })
                .expect("수납대장 제목 다음 원본 표");
            sources.push(((si, pi), (si, table_pi, ci), table));
        }
    }
    assert_eq!(
        sources.len(),
        1,
        "원본 제목으로 수납대장을 유일하게 식별한다"
    );
    let (title_owner, table_owner, source) = sources[0];
    assert_eq!(core.page_count(), 4, "독립 Print 쪽수와 서식 소속");
    let mut occurrences = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut tables = Vec::new();
        owned_tables(&tree.root, table_owner, &mut tables);
        if tables.is_empty() {
            continue;
        }
        occurrences += 1;
        assert_eq!(page, 3, "수납대장은 Print p4에만 있다");
        assert_eq!(tables.len(), 1, "표 조각 중복 없음");
        let table = tables[0];
        let mut titles = Vec::new();
        body_lines(&tree.root, title_owner, false, &mut titles);
        assert_eq!(
            compact(&titles.iter().map(|n| text(n)).collect::<String>()),
            "임대료수납대장"
        );
        assert!(!titles.is_empty());
        assert!(
            titles
                .iter()
                .all(|n| n.bbox.y + n.bbox.height <= table.bbox.y),
            "제목 다음 표의 순서·비겹침"
        );
        let mut painted_cells = Vec::new();
        cells(table, &mut painted_cells);
        assert_eq!(
            painted_cells.len(),
            source.cells.len(),
            "빈 입력 행도 누락·중복 없이 보존"
        );
        for (index, original) in source.cells.iter().enumerate() {
            let matches: Vec<_> = painted_cells
                .iter()
                .filter(|n| {
                    matches!(&n.node_type,
                RenderNodeType::TableCell(c) if c.model_cell_index == Some(index as u32))
                })
                .collect();
            assert_eq!(matches.len(), 1, "원본 셀 소유가 유일하다: {index}");
            assert_eq!(
                compact(&text(matches[0])),
                compact(
                    &original
                        .paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>()
                )
            );
        }
        // 독립 Print p4의 외곽 괘선 중심(pt)을 96DPI CSS px로 변환.
        // PDF: x58.049..530.835, y688.886..183.759, page height841pt.
        // 전쪽 TSV·확대 판독: regression-policy-20261015/yeongwol-final-ledger.
        for (actual, expected) in [
            (table.bbox.x, 58.049 * 4.0 / 3.0),
            (table.bbox.y, (841.0 - 688.886) * 4.0 / 3.0),
            (table.bbox.width, (530.835 - 58.049) * 4.0 / 3.0),
            (table.bbox.height, (688.886 - 183.759) * 4.0 / 3.0),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 표 시작·종료 ±2 CSSpx: 실제={actual}, 기준={expected}"
            );
        }
    }
    assert_eq!(occurrences, 1, "마지막 대장 표의 누락·중복 방지");
}
