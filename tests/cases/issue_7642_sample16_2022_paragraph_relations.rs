//! #8160: 2022 저장본 전 64쪽 Native/fresh 표준 WASM raw 100%, 정수 미대응 0.
//! 고정 화면 위치 대신 원본 문단·표 소유, 꼬리 내용과 포함·겹침을 검사한다.
//! 표 경계 검사는 source5ac의 실제 출력에서 앞 문단과의 겹침을 검출한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwp3-sample16-hwp5-2022.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("독립 Print 대응 원본"))
        .expect("2022 저장본 로드");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 64, "독립 Print 전체 쪽수");
    core
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // 원본 내용 소유는 논리 문자열로 비교한다. PUA의 표시 대체는 Print로 판독한다.
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn body_lines<'a>(node: &'a RenderNode, body: bool, out: &mut Vec<&'a RenderNode>) {
    let body = body || matches!(node.node_type, RenderNodeType::Body { .. });
    if body
        && matches!(
            node.node_type,
            RenderNodeType::Table(_) | RenderNodeType::TableCell(_) | RenderNodeType::TextBox
        )
    {
        return;
    }
    if body && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
    }
    for child in &node.children {
        body_lines(child, body, out);
    }
}

fn owned_lines<'a>(lines: &[&'a RenderNode], owner: (usize, usize)) -> Vec<&'a RenderNode> {
    lines
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(line)
            if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)))
        })
        .collect()
}

#[test]
fn sample16_2022_bcp_keeps_both_authored_rows_and_owned_successor() {
    let core = load();
    let sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .filter_map(move |(paragraph, para)| {
                    normalized(&para.text)
                        .contains("공사정보처리연속성확보를위한비상대응체계(BCP:BusinessContinuityPlanning)수립")
                        .then_some((section, paragraph, para))
                })
        })
        .collect();
    assert_eq!(sources.len(), 1, "BCP 꼬리 문단의 원본 소유");
    let (section, paragraph, source) = sources[0];
    let page = core.build_page_render_tree(2).expect("Print 3쪽 본문");
    let mut lines = Vec::new();
    body_lines(&page.root, false, &mut lines);
    let bcp = owned_lines(&lines, (section, paragraph));
    assert_eq!(
        source
            .line_segs
            .iter()
            .map(|seg| seg.text_start)
            .collect::<Vec<_>>(),
        [0, 64],
        "원본 두 줄 레코드의 문자 경계"
    );
    assert_eq!(bcp.len(), 2, "새 한컴 Print와 같은 원본 두 줄");
    let actual: String = bcp.iter().map(|line| text(line)).collect();
    assert_eq!(
        normalized(&actual),
        normalized(&source.text),
        "원본 내용 순서·누락·중복"
    );
    assert_eq!(
        normalized(&text(bcp[1])),
        "립",
        "Print 두 번째 줄의 끝 글자"
    );
    assert!(normalized(&text(bcp[0])).ends_with('수'));
    // 새 Hancom 2020 Print 3쪽 span origin × 96/72. 논리 글상자 높이와
    // PDF 글자 잉크 높이를 섞지 않고 실제 가시 런의 원점·기준선을 비교한다.
    let print_origins = [(109.919993, 903.199951), (143.359996, 930.879964)];
    let mut baselines = Vec::new();
    for (line, (x, baseline)) in bcp.iter().zip(print_origins) {
        let first = line
            .children
            .iter()
            .find(|node| {
                matches!(&node.node_type,
            RenderNodeType::TextRun(run) if !normalized(run.display_or_text()).is_empty())
            })
            .expect("BCP 두 줄의 첫 가시 런");
        let RenderNodeType::TextRun(run) = &first.node_type else {
            unreachable!()
        };
        let actual_baseline = first.bbox.y + run.baseline;
        assert!((first.bbox.x - x).abs() <= 2.0, "BCP 가로 원점 ±2px");
        assert!((actual_baseline - baseline).abs() <= 2.0, "BCP 기준선 ±2px");
        baselines.push(actual_baseline);
    }
    assert!(
        ((baselines[1] - baselines[0]) - 27.680013).abs() <= 2.0,
        "독립 Print 두 줄의 기준선 간격 ±2px"
    );
    for line in &bcp {
        for node in &line.children {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                assert_eq!(
                    (run.section_index, run.para_index),
                    (Some(section), Some(paragraph))
                );
                assert!(run.cell_context.is_none(), "두 줄 모두 원본 본문 소유");
            }
        }
    }
    assert!(
        bcp[0].bbox.y + bcp[0].bbox.height <= bcp[1].bbox.y,
        "두 원본 줄 비겹침"
    );
    let successor = owned_lines(&lines, (section, paragraph + 1));
    assert!(
        !successor.is_empty(),
        "다음 원본 문단도 같은 Print 쪽에 보존"
    );
    assert!(
        bcp[1].bbox.y + bcp[1].bbox.height <= successor[0].bbox.y,
        "본문 꼬리와 다음 문단이 겹치지 않는다"
    );
    let source_next = &core.document().sections[section].paragraphs[paragraph + 1];
    let actual_next: String = successor.iter().map(|node| text(node)).collect();
    assert_eq!(normalized(&actual_next), normalized(&source_next.text));
}

fn tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node);
    }
    for child in &node.children {
        tables(child, out);
    }
}

fn lines_in<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
    }
    for child in &node.children {
        lines_in(child, out);
    }
}

#[test]
fn sample16_2022_continuation_table_clears_predecessor_and_preserves_owned_rows() {
    let core = load();
    let mut sources = Vec::new();
    for (section, data) in core.document().sections.iter().enumerate() {
        for (paragraph, para) in data.paragraphs.iter().enumerate() {
            for (control, item) in para.controls.iter().enumerate() {
                let Control::Table(table) = item else {
                    continue;
                };
                if table
                    .cells
                    .iter()
                    .flat_map(|cell| &cell.paragraphs)
                    .any(|p| normalized(&p.text).contains("기술제안서제출수량"))
                {
                    sources.push((section, paragraph, control, table));
                }
            }
        }
    }
    assert_eq!(sources.len(), 1, "제출 수량 표의 원본 소유");
    let (section, paragraph, control, source) = sources[0];
    assert!(source.common.treat_as_char, "본문 흐름에 참여하는 원본 표");
    let page = core.build_page_render_tree(6).expect("Print 7쪽 이어받기");
    let mut found = Vec::new();
    tables(&page.root, &mut found);
    let found: Vec<_> = found
        .into_iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
            if (table.section_index, table.para_index, table.control_index)
                == (Some(section), Some(paragraph), Some(control)))
        })
        .collect();
    assert_eq!(found.len(), 1, "표 소유·누락·중복");
    let table = found[0];
    let mut body = Vec::new();
    body_lines(&page.root, false, &mut body);
    let previous = owned_lines(&body, (section, paragraph.checked_sub(1).expect("앞 문단")));
    assert!(!previous.is_empty());
    let previous_bottom = previous
        .iter()
        .map(|line| line.bbox.y + line.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        table.bbox.y >= previous_bottom,
        "이어받은 표가 앞 문단의 실제 점유를 침범한다"
    );
    let mut rows = Vec::new();
    lines_in(table, &mut rows);
    assert_eq!(rows.len(), 3, "독립 Print의 제출 수량·CD·작성 기준 세 줄");
    let source_text: String = source
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|p| p.text.as_str())
        .collect();
    assert_eq!(
        normalized(&text(table)),
        normalized(&source_text),
        "표 내용 순서·보존"
    );
    for pair in rows.windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "표 내부 줄 겹침"
        );
    }
    for line in rows {
        let roundoff = f64::EPSILON * table.bbox.width.max(table.bbox.height).max(1.0) * 128.0;
        assert!(
            line.bbox.x + roundoff >= table.bbox.x
                && line.bbox.y + roundoff >= table.bbox.y
                && line.bbox.x + line.bbox.width <= table.bbox.x + table.bbox.width + roundoff
                && line.bbox.y + line.bbox.height <= table.bbox.y + table.bbox.height + roundoff,
            "원본 표가 내부 줄의 실제 점유를 포함한다"
        );
    }
}
