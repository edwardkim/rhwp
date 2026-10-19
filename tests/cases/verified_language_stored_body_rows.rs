//! 독립 Print가 보존한 정상 저장 본문의 줄 분할과 앞뒤 문단을 보호한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

struct OwnedRow {
    page: u32,
    column: u16,
    start: usize,
    text: String,
    bounds: BoundingBox,
    column_bounds: BoundingBox,
}

fn collect_rows(
    node: &RenderNode,
    page: u32,
    column: Option<(u16, BoundingBox)>,
    owner: (usize, usize),
    output: &mut Vec<OwnedRow>,
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
        let (index, bounds) = column.expect("본문 줄의 실제 단 소유");
        let first = runs.first().expect("저장 본문 줄의 텍스트 런");
        assert!(
            runs.iter().all(|run| (run.section_index, run.para_index)
                == (Some(owner.0), Some(owner.1))
                && run.cell_context.is_none()),
            "본문과 셀의 소유 분리"
        );
        output.push(OwnedRow {
            page,
            column: index,
            start: first.char_start.expect("원문 문자 시작점"),
            text: runs.iter().map(|run| run.text.as_str()).collect(),
            bounds: node.bbox,
            column_bounds: bounds,
        });
    }
    for child in &node.children {
        collect_rows(child, page, column, owner, output);
    }
}

#[test]
fn verified_language_print_preserves_clean_body_rows_and_neighbor_ownership() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21_언어_기출_편집가능본.hwp");
    let bytes = std::fs::read(input).unwrap();
    let source = rhwp::parser::parse_document(&bytes).expect("배치와 독립적인 저장 원문 파싱");
    let mut owners = Vec::new();
    for (section, data) in source.sections.iter().enumerate() {
        for (paragraph, original) in data.paragraphs.iter().enumerate() {
            if original
                .text
                .starts_with("수피즘은 신과의 영적 합일을 통한 개인적 구원을 추구한다")
            {
                owners.push((section, paragraph));
            }
        }
    }
    assert_eq!(owners.len(), 1, "내용으로 유일하게 식별한 원본 문단");
    let (section, paragraph) = owners[0];
    let before = paragraph.checked_sub(1).unwrap();
    let after = paragraph + 1;
    let data = &source.sections[section].paragraphs;
    assert!(data[before].text.starts_with("18세기 후반 이후"));
    assert!(data[after].text.starts_with("북아프리카의 경우"));
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(
        core.page_count(),
        15,
        "독립 Print 전쪽 수와 내용 보존을 함께 확인"
    );
    let mut fragments = Vec::new();
    for owner in [before, paragraph, after] {
        let original = &data[owner];
        assert!(
            original.text.chars().all(|ch| ch.len_utf16() == 1),
            "원본 문자 축"
        );
        let mut rows = Vec::new();
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            for body in tree
                .root
                .children
                .iter()
                .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
            {
                collect_rows(body, page, None, (section, owner), &mut rows);
            }
        }
        let expected: Vec<_> = (0..original.line_segs.len())
            .map(|i| original.line_seg_text_start(i) as usize)
            .collect();
        assert_eq!(
            rows.iter().map(|r| r.start).collect::<Vec<_>>(),
            expected,
            "독립 Print와 일치하는 저장 줄 분할·원문 시작·누락·중복 보존"
        );
        let content = |text: &str| {
            text.chars()
                .filter(|ch| !ch.is_whitespace() && !ch.is_control())
                .collect::<String>()
        };
        assert_eq!(
            content(&rows.iter().map(|r| r.text.as_str()).collect::<String>()),
            content(&original.text)
        );
        for row in &rows {
            assert_eq!(
                (row.page, row.column),
                (7, 1),
                "독립 Print 8쪽 오른쪽 단 소유"
            );
            assert!(
                row.bounds.x >= row.column_bounds.x
                    && row.bounds.x + row.bounds.width
                        <= row.column_bounds.x + row.column_bounds.width,
                "본문 줄의 단 내부 포함"
            );
        }
        for pair in rows.windows(2) {
            assert!(pair[0].start < pair[1].start, "원문 순서 보존");
            assert!(
                pair[0].bounds.y + pair[0].bounds.height <= pair[1].bounds.y,
                "연속 본문 줄의 비겹침"
            );
        }
        fragments.push(rows);
    }
    assert_eq!(fragments[1].len(), 10, "독립 Print의 해당 원문 열 줄");
    for pair in fragments.windows(2) {
        let previous = pair[0].last().unwrap();
        let following = pair[1].first().unwrap();
        assert!(
            previous.bounds.y + previous.bounds.height <= following.bounds.y,
            "앞뒤 본문 문단의 순서와 비겹침"
        );
    }
}
