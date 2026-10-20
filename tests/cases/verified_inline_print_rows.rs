//! 개체 앞 작성 공백과 명시적 개행을 독립 Print의 소유·내용·중요 경계로 보호한다.
//! 정식 등록은 같은 입력의 전쪽 Native/fresh WASM과 직접 판독이 끝난 뒤에만 한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::{control::Control, paragraph::Paragraph, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn visible_source(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '\n' | '\r' | '\u{fffc}'))
        .collect()
}

fn table_text(table: &Table) -> String {
    table
        .cells
        .iter()
        .flat_map(|cell| &cell.paragraphs)
        .map(|p| p.text.as_str())
        .collect()
}

fn rendered_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        rendered_text(child, out);
    }
}

fn owned<'a>(
    node: &'a RenderNode,
    owner: (usize, usize, usize),
    tables: &mut Vec<&'a RenderNode>,
    rows: &mut Vec<&'a RenderNode>,
) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if (table.section_index, table.para_index, table.control_index)
            == (Some(owner.0), Some(owner.1), Some(owner.2))
        {
            tables.push(node);
        }
        return;
    }
    if matches!(
        node.node_type,
        RenderNodeType::TableCell(_) | RenderNodeType::TextBox
    ) {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(line) if (line.section_index, line.para_index)==(Some(owner.0),Some(owner.1)))
    {
        rows.push(node);
    }
    for child in &node.children {
        owned(child, owner, tables, rows);
    }
}

fn host_rows(rows: &[&RenderNode], source: &Paragraph, owner: (usize, usize), table: &RenderNode) {
    let mut actual = String::new();
    let mut visible_bottom = None::<f64>;
    for row in rows {
        let mut ink = false;
        for child in &row.children {
            if let RenderNodeType::TextRun(run) = &child.node_type {
                if (run.section_index, run.para_index) != (Some(owner.0), Some(owner.1))
                    || run.char_start.is_none()
                {
                    continue;
                }
                assert!(
                    run.cell_context.is_none(),
                    "호스트 본문과 표 셀의 소유 분리"
                );
                let positions = run
                    .layout_positions
                    .as_ref()
                    .expect("공통 계획의 가시 문자 원점");
                assert_eq!(
                    positions.len(),
                    run.text.chars().count() + 1,
                    "개행 메트릭을 가시 문자로 합치지 않는 원점 계약"
                );
                assert!(
                    positions.windows(2).all(|p| p[0] <= p[1]),
                    "문자와 작성 공백의 전진 순서"
                );
                let start = run.char_start.unwrap();
                let expected: String = source
                    .text
                    .chars()
                    .skip(start)
                    .take(run.text.chars().count())
                    .collect();
                assert_eq!(run.text, expected, "원점 배열과 원문 문자 위치의 같은 소유");
                actual.push_str(&run.text);
                ink |= !run.text.trim().is_empty();
            }
        }
        if ink {
            visible_bottom = Some(row.bbox.y + row.bbox.height);
        }
    }
    assert_eq!(
        visible_source(&actual),
        visible_source(&source.text),
        "전쪽 호스트의 원문 문자와 작성 공백 보존"
    );
    assert!(
        rows.windows(2).all(|p| p[0].bbox.y < p[1].bbox.y),
        "물리 줄의 순서·중복 방지"
    );
    assert!(
        rows.len() > source.text.chars().filter(|c| *c == '\n').count(),
        "빈 연속 개행도 실제 줄을 점유"
    );
    if let Some(bottom) = visible_bottom {
        assert!(bottom <= table.bbox.y, "단위 표시와 후속 개체의 비겹침");
    }
}

fn verify(input: &str, frames: &[[f64; 4]]) {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input)).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let hosts: Vec<_> = source
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, s)| {
            s.paragraphs
                .iter()
                .enumerate()
                .flat_map(move |(paragraph, p)| {
                    p.controls
                        .iter()
                        .enumerate()
                        .filter_map(move |(control, c)| match c {
                            Control::Table(t) if t.common.treat_as_char => {
                                Some((section, paragraph, control, p, t.as_ref()))
                            }
                            _ => None,
                        })
                })
        })
        .collect();
    assert_eq!(
        hosts.len(),
        frames.len(),
        "원본의 페이지별 단일 개체 호스트"
    );
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(
        core.page_count() as usize,
        frames.len(),
        "독립 Print 전체 쪽수와 원문 소유를 함께 확인"
    );
    for (page, ((section, paragraph, control, original, original_table), expected)) in
        hosts.iter().zip(frames).enumerate()
    {
        let tree = core.build_page_render_tree(page as u32).unwrap();
        let mut tables = Vec::new();
        let mut rows = Vec::new();
        for body in tree
            .root
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        {
            owned(
                body,
                (*section, *paragraph, *control),
                &mut tables,
                &mut rows,
            );
        }
        assert_eq!(
            tables.len(),
            1,
            "원문 개체를 자기 페이지에 한 번만 배치: page={}",
            page + 1
        );
        assert!(!rows.is_empty(), "호스트 줄이 없는 범위의 통과 금지");
        let table = tables[0];
        let mut actual = String::new();
        rendered_text(table, &mut actual);
        assert_eq!(
            visible_source(&actual),
            visible_source(&table_text(original_table)),
            "셀 원문 내용의 누락·중복 방지"
        );
        host_rows(&rows, original, (*section, *paragraph), table);
        // 같은 원본의 독립 PDF 벡터를 96dpi 좌상단 좌표로 환산한 값이다.
        // 완료 페이지의 양쪽 98%·확대 판독 선행 조건과 관계 검사를 함께 유지한다.
        for (actual, expected) in [
            table.bbox.x,
            table.bbox.y,
            table.bbox.width,
            table.bbox.height,
        ]
        .iter()
        .zip(expected)
        {
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 중요 경계 ±2px: page={}, actual={actual}, expected={expected}",
                page + 1
            );
        }
    }
}

#[test]
fn verified_authored_object_separator_groups_preserve_print_boundary_and_space_ownership() {
    verify(
        "tests/fixtures/inline_flow/print-object-space-boundaries.hwpx",
        BOUNDARY_FRAMES,
    );
}
#[test]
fn verified_overflow_spaces_and_styled_break_preserve_print_rows() {
    verify(
        "tests/fixtures/inline_flow/print-object-space-overflow.hwpx",
        OVERFLOW_FRAMES,
    );
}
#[test]
fn verified_same_style_and_consecutive_breaks_preserve_print_rows_and_source_positions() {
    verify(
        "tests/fixtures/inline_flow/print-authored-break-metrics.hwpx",
        BREAK_FRAMES,
    );
}

const BOUNDARY_FRAMES: &[[f64; 4]] = &[
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        141.12532552083334,
        629.2613220214844,
        23.974690755208314,
    ],
    [
        82.99600219726562,
        141.12532552083334,
        629.2613220214844,
        23.974690755208314,
    ],
    [
        82.99600219726562,
        141.12532552083334,
        629.2613220214844,
        23.974690755208314,
    ],
    [
        82.99600219726562,
        141.12532552083334,
        629.2613220214844,
        23.974690755208314,
    ],
    [
        90.5106709798177,
        121.94669596354167,
        629.262685139974,
        23.97330729166667,
    ],
    [
        91.79066975911458,
        121.94669596354167,
        629.4213256835938,
        23.97330729166667,
    ],
    [
        92.27067057291667,
        121.94669596354167,
        629.4213053385416,
        23.97330729166667,
    ],
    [
        92.75067138671875,
        121.94669596354167,
        629.4213663736979,
        23.97330729166667,
    ],
    [
        93.22932942708333,
        121.94669596354167,
        629.4226888020834,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
];

const OVERFLOW_FRAMES: &[[f64; 4]] = &[
    [
        82.99600219726562,
        135.85196940104166,
        629.2613220214844,
        23.973388671875,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        121.94669596354167,
        629.2613220214844,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        146.239990234375,
        629.2613220214844,
        23.973307291666657,
    ],
    [
        82.99600219726562,
        146.239990234375,
        629.2613220214844,
        23.973307291666657,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        91.3106689453125,
        121.94669596354167,
        629.4213460286459,
        23.97330729166667,
    ],
    [
        82.99600219726562,
        146.239990234375,
        629.2613220214844,
        23.973307291666657,
    ],
];

const BREAK_FRAMES: &[[f64; 4]] = &[
    [
        82.99600219726562,
        132.33463541666666,
        629.2613220214844,
        23.974690755208343,
    ],
    [
        82.99600219726562,
        149.75602213541666,
        629.2613220214844,
        23.973307291666686,
    ],
    [
        82.99600219726562,
        146.239990234375,
        629.2613220214844,
        23.973307291666657,
    ],
    [
        82.99600219726562,
        139.36800130208334,
        629.2613220214844,
        23.973307291666657,
    ],
    [
        82.99600219726562,
        170.53336588541666,
        629.2613220214844,
        23.973307291666686,
    ],
    [
        82.99600219726562,
        132.33463541666666,
        629.2613220214844,
        23.974690755208343,
    ],
];

#[test]
fn verified_distribution_budget_table_preserves_unit_label_and_following_caption() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task1768/distribution_doc.hwpx");
    let bytes = std::fs::read(input).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let mut hosts = Vec::new();
    for (section, s) in source.sections.iter().enumerate() {
        for (paragraph, p) in s.paragraphs.iter().enumerate() {
            if p.text.trim() == "(단위 : 천원)" {
                for (control, c) in p.controls.iter().enumerate() {
                    if let Control::Table(t) = c {
                        if t.common.treat_as_char {
                            hosts.push((section, paragraph, control, p, t.as_ref()));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        hosts.len(),
        1,
        "원문 단위 표시와 인라인 예산 표의 유일한 소유"
    );
    let (section, paragraph, control, original, original_table) = hosts[0];
    let caption = source.sections[section]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(i, p)| *i > paragraph && p.text.contains("연간 예산액"))
        .expect("원문의 표 뒤 주석");
    let title = source.sections[section]
        .paragraphs
        .iter()
        .enumerate()
        .rev()
        .find(|(i, p)| *i < paragraph && p.text.contains("집행예산"))
        .expect("원문의 표 앞 제목");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 3, "독립 Print 전체 쪽수");
    let trees: Vec<_> = (0..core.page_count())
        .map(|p| core.build_page_render_tree(p).unwrap())
        .collect();
    let mut placements = Vec::new();
    for (page, tree) in trees.iter().enumerate() {
        let mut tables = Vec::new();
        let mut rows = Vec::new();
        for body in tree
            .root
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        {
            owned(body, (section, paragraph, control), &mut tables, &mut rows);
        }
        if !tables.is_empty() {
            assert_eq!(tables.len(), 1);
            placements.push((page, tables[0], rows));
        } else {
            assert!(
                rows.is_empty(),
                "단위 표시와 예산 표를 다른 쪽에 분리하지 않음"
            );
        }
    }
    assert_eq!(placements.len(), 1, "예산 표의 전쪽 누락·중복 방지");
    let (page, table, rows) = &placements[0];
    assert_eq!(*page, 2, "독립 Print의 내용 대응 페이지");
    host_rows(rows, original, (section, paragraph), table);
    let mut actual = String::new();
    rendered_text(table, &mut actual);
    assert_eq!(
        visible_source(&actual),
        visible_source(&table_text(original_table)),
        "예산 셀 전체 원문 보존"
    );
    let body = trees[*page]
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let mut ignored = Vec::new();
    let mut title_rows = Vec::new();
    let mut caption_rows = Vec::new();
    owned(
        body,
        (section, title.0, usize::MAX),
        &mut ignored,
        &mut title_rows,
    );
    owned(
        body,
        (section, caption.0, usize::MAX),
        &mut ignored,
        &mut caption_rows,
    );
    assert!(
        !title_rows.is_empty() && !caption_rows.is_empty(),
        "표 앞 제목과 뒤 주석의 쪽 소유"
    );
    assert!(
        title_rows.last().unwrap().bbox.y + title_rows.last().unwrap().bbox.height
            <= rows[0].bbox.y,
        "제목 뒤 단위 표시의 순서"
    );
    assert!(
        table.bbox.y + table.bbox.height <= caption_rows[0].bbox.y,
        "예산 표 뒤 주석의 순서·비겹침"
    );
    let mut actual = String::new();
    for row in caption_rows {
        rendered_text(row, &mut actual);
    }
    assert_eq!(
        visible_source(&actual),
        visible_source(&caption.1.text),
        "후속 주석의 원문 보존"
    );
    // 독립 Print 3쪽 외곽 벡터의 96dpi 좌상단 좌표다. 양쪽 완료 98% 조건으로만 고정한다.
    let expected = [
        90.83066813151042,
        542.4466552734375,
        630.3813273111979,
        244.53202311197913,
    ];
    for (actual, expected) in [
        table.bbox.x,
        table.bbox.y,
        table.bbox.width,
        table.bbox.height,
    ]
    .iter()
    .zip(expected)
    {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print 예산 표 중요 경계 ±2px: actual={actual}, expected={expected}"
        );
    }
}

#[test]
fn verified_distribution_whitespace_rows_belong_to_the_budget_table() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task1768/distribution_doc.hwpx");
    let bytes = std::fs::read(path).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let mut hosts = Vec::new();
    for (section, s) in source.sections.iter().enumerate() {
        for (paragraph, p) in s.paragraphs.iter().enumerate() {
            if p.text.trim() == "(단위 : 천원)" {
                for (control, c) in p.controls.iter().enumerate() {
                    if matches!(c, Control::Table(t) if t.common.treat_as_char) {
                        hosts.push((section, paragraph, control, p));
                    }
                }
            }
        }
    }
    assert_eq!(hosts.len(), 1, "원문의 단위 표시와 단일 예산 표");
    let (section, paragraph, control, original) = hosts[0];
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let page = core.build_page_render_tree(2).unwrap();
    let mut tables = Vec::new();
    let mut rows = Vec::new();
    owned(
        &page.root,
        (section, paragraph, control),
        &mut tables,
        &mut rows,
    );
    assert_eq!(tables.len(), 1, "독립 Print 3쪽에 한 번만 있는 예산 표");
    let table = tables[0];
    host_rows(&rows, original, (section, paragraph), table);
    let mut labels = Vec::new();
    let mut object_rows = Vec::new();
    for row in rows {
        let mut text = String::new();
        rendered_text(row, &mut text);
        if text.trim().is_empty() {
            assert!(
                row.bbox.y <= table.bbox.y
                    && table.bbox.y + table.bbox.height <= row.bbox.y + row.bbox.height,
                "가시 글자 없는 줄은 원문 개체를 포함해야 하며 별도 빈 줄을 만들지 않는다"
            );
            object_rows.push(row);
        } else {
            assert_eq!(text.trim(), "(단위 : 천원)", "단위 표시의 내용 보존");
            labels.push(row);
        }
    }
    assert_eq!(labels.len(), 1, "단위 표시의 누락·중복 방지");
    assert_eq!(object_rows.len(), 1, "다음 줄을 점유하는 원문 표의 소유");
    assert!(
        labels[0].bbox.y + labels[0].bbox.height <= object_rows[0].bbox.y,
        "단위 표시 뒤 표의 줄 순서·비겹침"
    );
}
