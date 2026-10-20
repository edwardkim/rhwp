//! 2023년 9월 HWP/HWPX 각 문서의 완료 쪽에서 미주·후속 본문을 보호한다.
//! 전 20쪽 양쪽 최저 96% 이상, 14쪽 양쪽 최솟값은 각각 98.95307%/98.99232%.
//! 원본별 Print 좌표·SHA·전쪽 TSV·직접 확대 판독: task_m100_7642_stage1351.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::footnote::Endnote;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

type Located<'a> = (&'a RenderNode, u16, BoundingBox);

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn lines<'a>(node: &'a RenderNode, column: Option<(u16, BoundingBox)>, out: &mut Vec<Located<'a>>) {
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    let column = match node.node_type {
        RenderNodeType::Column(index) => Some((index, node.bbox)),
        _ => column,
    };
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        if let Some((index, frame)) = column {
            out.push((node, index, frame));
        }
    }
    for child in &node.children {
        lines(child, column, out);
    }
}

fn source_note(core: &DocumentCore, number: u16) -> ((usize, usize, usize), &Endnote) {
    let mut found = Vec::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, source) in data.paragraphs.iter().enumerate() {
            for (control, item) in source.controls.iter().enumerate() {
                if let Control::Endnote(note) = item {
                    if note.number == number {
                        found.push(((section, paragraph, control), note.as_ref()));
                    }
                }
            }
        }
    }
    assert_eq!(found.len(), 1, "번호로 식별한 단일 원본 미주: {number}");
    found[0]
}

fn assert_equation_sources(
    node: &RenderNode,
    owner: (usize, usize, usize),
    note: &Endnote,
    first_equation_paragraph: usize,
    first_equations: &mut Vec<String>,
) -> usize {
    let mut count = 0;
    if let RenderNodeType::Equation(eq) = &node.node_type {
        if let Some(reference) = &eq.note_ref {
            if (
                reference.section_index,
                reference.para_index,
                reference.control_index,
            ) == owner
            {
                assert_eq!(reference.kind, "endnote");
                let source = &note.paragraphs[reference.note_para_index];
                assert!(
                    source.controls.iter().any(|item| {
                        matches!(item, Control::Equation(saved) if saved.script == eq.script)
                    }),
                    "출력 수식은 해당 원본 미주의 같은 문단에 소속한다"
                );
                count += 1;
                if reference.note_para_index == first_equation_paragraph {
                    first_equations.push(eq.script.clone());
                }
            }
        }
    }
    count
        + node
            .children
            .iter()
            .map(|child| {
                assert_equation_sources(
                    child,
                    owner,
                    note,
                    first_equation_paragraph,
                    first_equations,
                )
            })
            .sum::<usize>()
}

fn near_print(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print ±2 CSS px: actual={actual}, expected={expected}"
    );
}

fn check_document(input: &str, compatibility_2024: bool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("개별 공식 원본")).unwrap();
    core.set_hangul2024_compat(compatibility_2024);
    assert_eq!(core.page_count(), 20, "해당 원본의 독립 Print 전체 쪽수");
    // PDF 텍스트 원점을 96/72로 환산한다. y는 MediaBox 위쪽에서 측정한 기준선이다.
    let targets = [(20, 0, 34.08, 415.20), (22, 1, 402.56, 239.04)];
    let mut occurrences = vec![Vec::new(); targets.len()];
    for page_index in 0..core.page_count() {
        let page = core
            .build_page_render_tree(page_index)
            .expect("실제 쪽 출력");
        let mut found = Vec::new();
        lines(&page.root, None, &mut found);
        for (index, &(number, column, print_x, print_y)) in targets.iter().enumerate() {
            let prefix = format!("문{number}）");
            let headings: Vec<_> = found
                .iter()
                .filter(|(line, _, _)| text(line).starts_with(&prefix))
                .collect();
            for &&(heading, actual_column, frame) in &headings {
                occurrences[index].push(page_index);
                assert_eq!(page_index, 13, "미주 시작의 Print 쪽 소속");
                assert_eq!(actual_column, column, "좌·우 단 소속");
                let RenderNodeType::TextLine(head) = &heading.node_type else {
                    unreachable!()
                };
                let paragraph = head.para_index.expect("배치된 미주의 문단 소유");
                let (source_owner, note) = source_note(&core, number);
                let label = normalized(&text(heading));
                assert_eq!(
                    label,
                    format!("{prefix}{}", normalized(&note.paragraphs[0].text)),
                    "원본 미주 제목의 내용 순서·누락·중복"
                );
                let runs: Vec<_> = heading
                    .children
                    .iter()
                    .filter_map(|node| match &node.node_type {
                        RenderNodeType::TextRun(run) => Some((node, run)),
                        _ => None,
                    })
                    .collect();
                assert!(!runs.is_empty());
                near_print(runs[0].0.bbox.x, print_x);
                let epsilon = f64::EPSILON * frame.width.max(1.0) * 128.0;
                let mut previous_end = None;
                for (node, run) in runs {
                    assert_eq!(run.para_index, Some(paragraph));
                    assert!(run.cell_context.is_none());
                    near_print(node.bbox.y + run.baseline, print_y);
                    assert!(
                        node.bbox.x + epsilon >= frame.x
                            && node.bbox.x + node.bbox.width <= frame.x + frame.width + epsilon,
                        "미주 제목의 단 내부 포함"
                    );
                    if let Some(end) = previous_end {
                        assert!(node.bbox.x + epsilon >= end, "제목 원문 런 비겹침");
                    }
                    previous_end = Some(node.bbox.x + node.bbox.width);
                }
                let following: Vec<_> = found
                    .iter()
                    .filter(|(line, index, _)| {
                        *index == column
                            && matches!(&line.node_type, RenderNodeType::TextLine(data)
                                if data.para_index == Some(paragraph + 1))
                    })
                    .collect();
                assert!(
                    !following.is_empty(),
                    "미주 첫 본문도 같은 쪽·단에 보존한다"
                );
                assert!(
                    heading.bbox.y + heading.bbox.height <= following[0].0.bbox.y,
                    "미주 제목 뒤에 첫 본문을 겹침 없이 배치한다"
                );
                let following_text: String =
                    following.iter().map(|(node, _, _)| text(node)).collect();
                assert_eq!(
                    normalized(&following_text),
                    normalized(&note.paragraphs[1].text),
                    "첫 본문의 원문 순서·누락·중복"
                );
                let first_equation_paragraph = note
                    .paragraphs
                    .iter()
                    .position(|source| {
                        source
                            .controls
                            .iter()
                            .any(|item| matches!(item, Control::Equation(_)))
                    })
                    .expect("독립 Print에서 확인한 미주의 첫 수식 문단");
                let expected_equations: Vec<_> = note.paragraphs[first_equation_paragraph]
                    .controls
                    .iter()
                    .filter_map(|item| match item {
                        Control::Equation(eq) => Some(eq.script.clone()),
                        _ => None,
                    })
                    .collect();
                let mut actual_equations = Vec::new();
                assert!(
                    assert_equation_sources(
                        &page.root,
                        source_owner,
                        note,
                        first_equation_paragraph,
                        &mut actual_equations,
                    ) > 0,
                    "제목 위치뿐 아니라 같은 원본 미주의 실제 수식을 검증한다"
                );
                assert_eq!(
                    actual_equations, expected_equations,
                    "첫 수식 문단의 소유·순서·누락·중복"
                );
            }
        }
    }
    for (target, pages) in targets.iter().zip(occurrences) {
        assert_eq!(
            pages,
            [13],
            "문{}는 Print 14쪽에 한 번만 표시한다",
            target.0
        );
    }
}

#[test]
fn completed_september_2023_hwp_note_starts_keep_print_coordinates_and_owned_equations() {
    check_document("samples/3-09월_교육_통합_2023.hwp", false);
}

#[test]
fn completed_september_2023_hwpx_note_starts_keep_print_coordinates_and_owned_equations() {
    check_document("samples/3-09월_교육_통합_2023.hwpx", true);
}
