//! #7642 저장된 문단 가운데 정렬점과 수식·문자의 서로 다른 기준선.
//! 주석 갱신: 2026-11-07 13:54 KST.
//! 고정형: centered_text_baselines_match_independent_print는 독립 Print 4쪽의
//! 다섯 문자 기준선을 96 DPI 쪽 좌표계에서 현행 지침의 ±2px로 보호한다.
//! 관계형: inline_equations_keep_source_centers_and_content는 원본 줄의 중심,
//! 수식 소유·순서·누락·중복과 수식만 있는 선택지의 기준선 정렬을 보호한다.
//! Native/fresh WASM 전 4쪽 최저 99.19496%/99.22469%, 대상 4쪽
//! 99.40548%/99.42983%에서 review·overlay를 직접 판독했다(2022 호환).
//! HWP SHA-256: 22d29786a80d68a9b2ad9294c2dab4915e0eced941e790e37390b14312b8b6a8.
//! PDF: pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf,
//! SHA-256: 1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14.
//! 정식 증적: mydocs/pr/assets/issue7642/stored-inline-center-20261107/.
//! 검사는 좌표를 생산하는 공통 render tree를 쓰며 사설 글꼴 설치에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_science.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 과학 문서"))
        .expect("원본 HWP 읽기");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 4, "독립 Print 전체 쪽수");
    core
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if node.visible && !node.editor_only {
        out.push(node);
        for child in &node.children {
            walk(child, out);
        }
    }
}

// 화면상의 사각 영역으로 대상을 고르지 않고 원본 셀 경로로 줄을 식별한다.
fn condition_paragraph(node: &RenderNode) -> Option<usize> {
    let RenderNodeType::TextLine(_) = &node.node_type else {
        return None;
    };
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes.into_iter().find_map(|child| {
        let RenderNodeType::TextRun(run) = &child.node_type else {
            return None;
        };
        let context = run.cell_context.as_ref()?;
        let [entry] = context.path.as_slice() else {
            return None;
        };
        (run.section_index == Some(0)
            && context.parent_para_index == 127
            && !context.in_textbox
            && entry.control_index == 0
            && entry.cell_index == 0
            && (1..=3).contains(&entry.cell_para_index))
        .then_some(entry.cell_para_index)
    })
}

#[test]
fn centered_text_baselines_match_independent_print() {
    let core = load();
    let tree = core.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let mut found = Vec::new();
    // 독립 PDF 문자 원점(pt)을 96/72로 환산했다. 후보 출력에서 복사하지 않는다.
    let expected = [
        (Some(1), 0, 910.5599772135416),
        (Some(2), 0, 941.6000162760416),
        (Some(2), 1, 972.6399739583334),
        (Some(3), 0, 1013.1199544270834),
        (None, 0, 1071.6800130208333),
    ];
    for node in nodes {
        let RenderNodeType::TextLine(line) = &node.node_type else {
            continue;
        };
        let owner = condition_paragraph(node);
        if owner.is_none() && !(line.section_index == Some(0) && line.para_index == Some(128)) {
            continue;
        }
        let index = line.line_index.expect("저장 줄 인덱스");
        let &(_, _, baseline) = expected
            .iter()
            .find(|&&(paragraph, at, _)| paragraph == owner && at == index)
            .expect("독립 Print에 대응하는 줄");
        let actual = node.bbox.y + line.baseline;
        assert!(
            (actual - baseline).abs() <= 2.0,
            "독립 Print의 문자 기준선: 소유={owner:?}, 줄={index}, 실제={actual}, 기준={baseline}"
        );
        found.push((owner, index));
    }
    let expected_owners: Vec<_> = expected.iter().map(|&(p, i, _)| (p, i)).collect();
    assert_eq!(found, expected_owners, "다섯 줄의 순서·누락·중복 보존");
}

#[test]
fn inline_equations_keep_source_centers_and_content() {
    let core = load();
    let Control::Table(table) = &core.document().sections[0].paragraphs[127].controls[0] else {
        panic!("원본 조건 표");
    };
    let cell = &table.cells[0];
    let tree = core.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let mut scripts = Vec::new();
    let mut equation_only = 0;
    for node in nodes {
        let RenderNodeType::TextLine(line) = &node.node_type else {
            continue;
        };
        let paragraph = condition_paragraph(node);
        let mut children = Vec::new();
        walk(node, &mut children);
        for child in children {
            let RenderNodeType::Equation(equation) = &child.node_type else {
                continue;
            };
            if let Some(paragraph) = paragraph {
                assert_eq!(equation.section_index, Some(0));
                assert_eq!(equation.para_index, Some(127));
                assert_eq!(equation.cell_index, Some(0));
                assert_eq!(equation.cell_para_index, Some(paragraph));
                let source = &cell.paragraphs[paragraph];
                let shape = &core.document().doc_info.para_shapes[source.para_shape_id as usize];
                assert_eq!((shape.attr1 >> 20) & 3, 2, "원본 문단의 가운데 정렬");
                let segment = &source.line_segs[line.line_index.unwrap() as usize];
                assert_eq!(segment.baseline_distance, segment.text_height / 2);
                let stored_center = node.bbox.y + f64::from(segment.baseline_distance) / 75.0;
                let equation_center = child.bbox.y + child.bbox.height / 2.0;
                assert!(
                    (equation_center - stored_center).abs() < 1e-7,
                    "수식 논리 상자는 문자의 ascent 대신 원본 줄 정렬점을 소비한다"
                );
                assert!(node.bbox.y + line.baseline > stored_center);
                scripts.push((paragraph, equation.script.clone()));
            } else if equation.para_index == Some(129) {
                // 같은 쪽의 수식만 있는 선택지: 문자 가운데 정렬 보정의 비적용 대조군.
                let actual = child.bbox.y + equation.layout_box.baseline;
                let baseline = node.bbox.y + line.baseline;
                // 논리 상자의 소수 기준선을 저장 HWPUNIT로 바꿀 때의 1단위 정밀도다.
                assert!((actual - baseline).abs() <= 1.0 / 75.0);
                equation_only += 1;
            }
        }
    }
    let source_scripts: Vec<_> = cell
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(index, _)| (1..=3).contains(index))
        .flat_map(|(index, paragraph)| {
            paragraph.controls.iter().filter_map(move |control| {
                if let Control::Equation(equation) = control {
                    Some((index, equation.script.clone()))
                } else {
                    None
                }
            })
        })
        .collect();
    assert_eq!(scripts, source_scripts, "원본 수식의 소유·순서·내용 보존");
    assert_eq!(scripts.len(), 5, "조건 안의 다섯 수식");
    assert_eq!(equation_only, 5, "별도 선택지 다섯 개도 그대로 방출한다");
}
