//! 독립 Print의 표 호스트 본문·후속 표 소유와 완료 1쪽 경계를 보호한다.
//! 전쪽 양쪽 90% 및 1쪽 양쪽 98%·직접 판독 뒤에만 정식 suite에 추가한다.
//! 원본·PDF 해시, 전쪽 TSV와 수정 전후 증거는 task_m100_7642_stage1476.md에 기록한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::HashMap;

fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}

fn tac_after(paragraphs: &[Paragraph], start: usize) -> (usize, usize) {
    (start..paragraphs.len())
        .find_map(|owner| {
            paragraphs[owner]
                .controls
                .iter()
                .enumerate()
                .find_map(|(index, control)| {
                    matches!(control, Control::Table(table) if table.common.treat_as_char)
                        .then_some((owner, index))
                })
        })
        .expect("원문에서 다음 인라인 표 소유 확인")
}

fn tac_before(paragraphs: &[Paragraph], end: usize) -> (usize, usize) {
    (0..end)
        .rev()
        .find_map(|owner| {
            paragraphs[owner]
                .controls
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, control)| {
                    matches!(control, Control::Table(table) if table.common.treat_as_char)
                        .then_some((owner, index))
                })
        })
        .expect("원문에서 직전 인라인 표 소유 확인")
}

fn table_node(node: &RenderNode, owner: (usize, usize)) -> &RenderNode {
    fn find<'a>(node: &'a RenderNode, owner: (usize, usize), output: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::Table(table)
            if (table.para_index, table.control_index) == (Some(owner.0), Some(owner.1)))
        {
            output.push(node);
            return;
        }
        if matches!(
            node.node_type,
            RenderNodeType::TableCell(_) | RenderNodeType::TextBox
        ) {
            return;
        }
        for child in &node.children {
            find(child, owner, output);
        }
    }
    let mut found = Vec::new();
    find(node, owner, &mut found);
    assert_eq!(found.len(), 1, "같은 단에서 표를 한 번만 배치");
    found[0]
}

fn text(node: &RenderNode, output: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        output.push_str(&run.text);
    }
    for child in &node.children {
        text(child, output);
    }
}

fn source_table_text(table: &Table, output: &mut String) {
    for cell in &table.cells {
        for para in &cell.paragraphs {
            output.push_str(&para.text);
            for control in &para.controls {
                if let Control::Table(table) = control {
                    source_table_text(table, output);
                }
            }
        }
    }
}

fn body_rows<'a>(node: &'a RenderNode, owner: (usize, usize), output: &mut Vec<&'a RenderNode>) {
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)))
    {
        output.push(node);
    }
    for child in &node.children {
        body_rows(child, owner, output);
    }
}

fn check_body(
    column: &RenderNode,
    data: &[Paragraph],
    section: usize,
    range: std::ops::Range<usize>,
) -> BoundingBox {
    let mut rows = Vec::new();
    for owner in range {
        let original = &data[owner];
        if clean(&original.text).is_empty() {
            continue;
        }
        let mut owned = Vec::new();
        body_rows(column, (section, owner), &mut owned);
        assert!(!owned.is_empty(), "원문 본문을 왼쪽 단에 보존");
        let mut starts = Vec::new();
        let mut actual = String::new();
        for line in &owned {
            let runs: Vec<_> = line
                .children
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run)
                        if (run.section_index, run.para_index) == (Some(section), Some(owner))
                            && run.char_start.is_some() =>
                    {
                        Some(run)
                    }
                    _ => None,
                })
                .collect();
            assert!(
                runs.iter().all(|run| run.cell_context.is_none()),
                "본문과 셀 소유 분리"
            );
            starts.push(runs.first().expect("가시 본문 런").char_start.unwrap());
            for run in runs {
                actual.push_str(&run.text);
            }
            assert!(
                line.bbox.x >= column.bbox.x
                    && line.bbox.x + line.bbox.width <= column.bbox.x + column.bbox.width,
                "본문 줄의 단 내부 포함"
            );
        }
        // 저장 줄은 컨트롤 슬롯을 포함한 UTF-16 축이고 런은 가시 문자 축이다.
        // 원본 오프셋으로 투영해 두 축을 같은 단위에서 비교한다.
        assert!(
            !original.char_offsets.is_empty(),
            "원문 문자 축의 독립 근거"
        );
        let expected: Vec<_> = (0..original.line_segs.len())
            .map(|i| {
                let raw = original.line_seg_text_start(i);
                original
                    .char_offsets
                    .partition_point(|offset| *offset < raw)
            })
            .collect();
        assert_eq!(starts, expected, "저장 본문 줄의 시작·누락·중복 보존");
        assert_eq!(clean(&actual), clean(&original.text), "본문 원문 내용 보존");
        rows.extend(owned);
    }
    assert!(!rows.is_empty());
    for pair in rows.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "본문 줄과 문단 순서·비겹침"
        );
    }
    let first = rows[0].bbox;
    let last = rows.last().unwrap().bbox;
    BoundingBox {
        x: first.x,
        y: first.y,
        width: column.bbox.width,
        height: last.y + last.height - first.y,
    }
}

fn collect_plain_body_text(node: &RenderNode, output: &mut HashMap<(usize, usize), String>) {
    if matches!(
        node.node_type,
        RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let (Some(section), Some(owner), Some(_)) =
            (run.section_index, run.para_index, run.char_start)
        {
            if run.cell_context.is_none() {
                output
                    .entry((section, owner))
                    .or_default()
                    .push_str(&run.text);
            }
        }
    }
    for child in &node.children {
        collect_plain_body_text(child, output);
    }
}

fn verify(input: &str, pages: u32, later_pdf_y: f64) {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input)).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let mut matches = Vec::new();
    for (section, data) in source.sections.iter().enumerate() {
        for (owner, paragraph) in data.paragraphs.iter().enumerate() {
            if paragraph
                .text
                .starts_with("다음 글에서 사회·문화 현상을 바라보는 필자의 관점")
            {
                matches.push((section, owner));
            }
        }
    }
    assert_eq!(matches.len(), 1, "내용으로 찾은 원본 둘째 질문");
    let (section, question2) = matches[0];
    let data = &source.sections[section].paragraphs;
    let question3 = data
        .iter()
        .position(|p| p.text.starts_with("A, B의 일반적인 특징에"))
        .expect("원본 셋째 질문");
    let first_table = tac_after(data, 0);
    let second_before = tac_before(data, question2);
    let second_after = tac_after(data, question2);
    let third_before = tac_before(data, question3);
    let third_after = tac_after(data, question3);
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), pages, "독립 Print 전체 쪽수");
    let trees: Vec<_> = (0..core.page_count())
        .map(|page| core.build_page_render_tree(page).unwrap())
        .collect();
    let mut all_body_text = HashMap::new();
    for tree in &trees {
        for body in tree
            .root
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        {
            collect_plain_body_text(body, &mut all_body_text);
        }
    }
    let mut checked = 0;
    for (section, data) in source.sections.iter().enumerate() {
        for (owner, original) in data.paragraphs.iter().enumerate() {
            if original.controls.is_empty() && !clean(&original.text).is_empty() {
                assert_eq!(clean(all_body_text.get(&(section, owner)).expect("전쪽 본문 소유")), clean(&original.text), "쪽수와 함께 전쪽 일반 본문의 내용·누락·중복을 보호: section={section}, paragraph={owner}");
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "빈 범위 통과 금지");
    let tree = &trees[0];
    let body = tree
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .expect("첫 쪽 본문");
    let column = body
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Column(0)))
        .expect("첫 쪽 왼쪽 단");
    let first = table_node(column, first_table);
    let later = table_node(column, third_after);
    for (owner, node) in [(first_table, first), (third_after, later)] {
        let Control::Table(original) = &data[owner.0].controls[owner.1] else {
            panic!("원문 표");
        };
        let mut expected = String::new();
        source_table_text(original, &mut expected);
        let mut actual = String::new();
        text(node, &mut actual);
        assert_eq!(
            clean(&actual),
            clean(&expected),
            "표 원문과 중첩 셀의 내용 보존"
        );
    }
    let first_body = check_body(column, data, section, 0..first_table.0);
    assert!(
        first_body.y + first_body.height <= first.bbox.y,
        "호스트 본문 뒤의 첫 표"
    );
    let second_body = check_body(column, data, section, question2..second_after.0);
    assert!(
        table_node(column, second_before).bbox.y + table_node(column, second_before).bbox.height
            <= second_body.y,
        "앞 표 뒤에 둘째 질문 배치"
    );
    assert!(
        second_body.y + second_body.height <= table_node(column, second_after).bbox.y,
        "둘째 질문과 후속 표의 비겹침"
    );
    let third_body = check_body(column, data, section, question3..third_after.0);
    assert!(
        table_node(column, third_before).bbox.y + table_node(column, third_before).bbox.height
            <= third_body.y,
        "앞 표 뒤에 셋째 질문 배치"
    );
    assert!(
        third_body.y + third_body.height <= later.bbox.y,
        "셋째 질문·이어지는 문단과 뒤 표의 비겹침"
    );
    // 독립 Print의 96dpi 좌상단 좌표다. 원본·분할본 PDF를 각각 측정했다.
    for (node, x, y, width) in [
        (
            first,
            85.59999593098958,
            324.5559895833333,
            412.48000081380206,
        ),
        (later, 85.59999593098958, later_pdf_y, 408.63999430338544),
    ] {
        for (actual, expected) in [(node.bbox.x, x), (node.bbox.y, y), (node.bbox.width, width)] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "완료 페이지의 독립 Print 중요 경계 ±2px: actual={actual}, expected={expected}"
            );
        }
    }
}

#[test]
fn verified_social_hwp_preserves_print_host_frames_and_boundaries() {
    verify("samples/exam_social.hwp", 4, 1083.6639811197917);
}
#[test]
fn verified_social_hwpx_preserves_print_host_frames_and_boundaries() {
    verify("samples/hwpx/exam_social.hwpx", 4, 1083.6639811197917);
}
#[test]
fn verified_social_split_hwp_preserves_print_host_frames_and_boundaries() {
    verify("samples/exam_social-p1.hwp", 1, 1080.9466145833333);
}
#[test]
fn verified_social_split_hwpx_preserves_print_host_frames_and_boundaries() {
    verify("samples/hwpx/exam_social-p1.hwpx", 1, 1080.9466145833333);
}
