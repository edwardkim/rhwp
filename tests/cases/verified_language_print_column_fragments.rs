//! 원본 언어 문서의 두 단 이어받기와 후속 본문을 보호한다.
//! 전 15쪽 Native/fresh WASM 최저 94.96332% / 94.93966%.
//! 완료 10쪽 99.46707% / 99.50862%, 독립 Print 직접 확대 판독 완료.
//! 입력·Print SHA, 96dpi 좌표, 전쪽 TSV와 PNG: task_m100_7642_stage1383.md.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

type Row = (u32, u16, usize, String, BoundingBox, BoundingBox, f64);

fn collect(
    node: &RenderNode,
    page: u32,
    column: Option<(u16, BoundingBox)>,
    owner: (usize, usize),
    rows: &mut Vec<Row>,
) {
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
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)))
    {
        let runs: Vec<_> = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some(run),
                _ => None,
            })
            .collect();
        let (index, bounds) = column.expect("원문 줄의 단 소유");
        let first = runs.first().expect("가시 내용의 첫 런");
        assert!(runs.iter().all(|run| (run.section_index, run.para_index)
            == (Some(owner.0), Some(owner.1))
            && run.cell_context.is_none()));
        rows.push((
            page,
            index,
            first.char_start.unwrap(),
            runs.iter().map(|run| run.text.as_str()).collect(),
            node.bbox,
            bounds,
            first.baseline
                + node
                    .children
                    .iter()
                    .find(|child| matches!(&child.node_type, RenderNodeType::TextRun(_)))
                    .unwrap()
                    .bbox
                    .y,
        ));
    }
    for child in &node.children {
        collect(child, page, column, owner, rows);
    }
}

#[test]
fn verified_language_print_keeps_source_column_fragments_and_following_paragraph() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/21_언어_기출_편집가능본.hwp"),
    )
    .unwrap();
    let source = rhwp::parser::parse_document(&bytes).expect("배치와 별도로 파싱한 원본");
    let mut owners = Vec::new();
    for (section, data) in source.sections.iter().enumerate() {
        for (paragraph, original) in data.paragraphs.iter().enumerate() {
            if original
                .text
                .contains("이이는 현상 세계의 모든 존재는 리와 기가")
            {
                owners.push((section, paragraph));
            }
        }
    }
    assert_eq!(owners.len(), 1, "내용으로 식별한 원본 문단");
    let owner = owners[0];
    let original = &source.sections[owner.0].paragraphs[owner.1];
    assert!(
        original.text.chars().all(|ch| ch.len_utf16() == 1),
        "이 원본 범위의 문자 축"
    );
    let expected: Vec<_> = (0..original.line_segs.len())
        .map(|i| original.line_seg_text_start(i) as usize)
        .collect();
    let mut columns = Vec::new();
    let mut column = 0;
    for (i, row) in original.line_segs.iter().enumerate() {
        if i > 0
            && (row.vertical_pos < original.line_segs[i - 1].vertical_pos
                || row.is_first_line_of_column())
        {
            column += 1;
        }
        columns.push(column);
    }
    assert_eq!(column, 1, "독립 Print 10쪽의 좌·우 두 단");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 15, "독립 Print 전쪽 수");
    let mut actual = Vec::new();
    let mut following = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let bodies: Vec<_> = tree
            .root
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
            .collect();
        assert_eq!(bodies.len(), 1);
        collect(bodies[0], page, None, owner, &mut actual);
        collect(
            bodies[0],
            page,
            None,
            (owner.0, owner.1 + 1),
            &mut following,
        );
    }
    assert_eq!(
        actual.iter().map(|row| row.2).collect::<Vec<_>>(),
        expected,
        "원본 저장 줄의 시작·누락·중복 보존"
    );
    let clean = |text: &str| {
        text.chars()
            .filter(|c| !c.is_whitespace() && !c.is_control())
            .collect::<String>()
    };
    assert_eq!(
        clean(&actual.iter().map(|row| row.3.as_str()).collect::<String>()),
        clean(&original.text)
    );
    for (row, column) in actual.iter().zip(columns) {
        assert_eq!((row.0, row.1), (9, column), "원본 쪽·단 소속");
        assert!(
            row.4.x >= row.5.x && row.4.x + row.4.width <= row.5.x + row.5.width,
            "본문 줄의 단 내부 포함"
        );
    }
    let last = actual.last().unwrap();
    assert!(!following.is_empty());
    assert_eq!((following[0].0, following[0].1), (last.0, last.1));
    assert!(
        last.4.y + last.4.height <= following[0].4.y,
        "이어받은 마지막 줄 뒤에 후속 문단 배치"
    );
    assert_eq!(
        clean(
            &following
                .iter()
                .map(|row| row.3.as_str())
                .collect::<String>()
        ),
        clean(&source.sections[owner.0].paragraphs[owner.1 + 1].text)
    );
    // 구현 계산값이 아니라 독립 Print의 문자 원점을 72pt → 96dpi로 바꾼 값이다.
    // 대상은 원본 내용·줄 소속으로 고르며 px 영역으로 찾지 않는다.
    let near_print = |actual: f64, expected: f64| {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print ±2 CSS px: actual={actual}, expected={expected}"
        );
    };
    near_print(actual[0].4.x, 141.76);
    near_print(actual[0].6, 1076.96);
    let resumed = actual.iter().find(|row| row.1 == 1).unwrap();
    near_print(resumed.4.x, 591.52);
    near_print(resumed.6, 220.80);
    near_print(last.4.x, 591.52);
    near_print(last.6, 244.96);
    near_print(following[0].4.x, 604.64);
    near_print(following[0].6, 272.80);
}
