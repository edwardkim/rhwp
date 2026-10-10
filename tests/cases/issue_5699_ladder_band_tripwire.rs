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
