//! 관계형·고정형: 시장구조조사의 빈 줄 흐름·부록 표 소속·전체 쪽수와 내용 보존.
//! 주석 갱신: 2026-11-22T22:52:44+09:00. 최초 등록: 2026-11-21T23:18:23+09:00, 91f4f7f018350dff964c964e2d4d5b41afb148b7.
//! 2026-11-21 측정: PDFium, Print 96 DPI, 색상 경계 보정 전 반경 2px 실루엣 raw.
//! 94·95·100·247·315쪽 Native/fresh WASM 각각 100.00%; 246쪽 각각 99.97%.
//! 전체 315쪽, Native/fresh WASM 전쪽 최저 각각 98.40%.
//! 근거: mydocs/pr/assets/issues7754_7765/market-source7/README.md.
//! 독립 Print: pdf/issue7755/market-2017-manual-print-2024.pdf; 생산 소스 a21e590a7.

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::{BTreeMap, BTreeSet};

const SAMPLE: &str =
    "samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp";

fn nodes(root: &RenderNode) -> Vec<&RenderNode> {
    fn visit<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            visit(child, out);
        }
    }
    let mut out = Vec::new();
    visit(root, &mut out);
    out
}
fn ink(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn text(node: &RenderNode) -> String {
    nodes(node)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}
fn line(root: &RenderNode, para: usize) -> &RenderNode {
    nodes(root)
        .into_iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(para))
        })
        .expect("원본 문단의 글줄")
}
fn table(root: &RenderNode, para: usize) -> &RenderNode {
    nodes(root)
        .into_iter()
        .find(|n| {
            matches!(&n.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para))
        })
        .expect("원본 문단이 소유한 표")
}
fn source_table(core: &DocumentCore, para: usize) -> &Table {
    core.document().sections[0].paragraphs[para]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("원본 표")
}
fn below(a: &RenderNode, b: &RenderNode) {
    assert!(
        a.bbox.y + a.bbox.height <= b.bbox.y,
        "앞 요소의 점유가 다음 요소와 겹친다: {:?}, {:?}",
        a.bbox,
        b.bbox
    );
}

/// #7147의 실제 원본 표: 개체 높이는 첫 조각이고 행 선언은 전체 표의 높이다.
fn declared_rows_keep_their_size_across_page_fragments(core: &DocumentCore) {
    let source = source_table(core, 934);
    // TABLE.row_sizes는 행별 셀 수다. 실제 높이는 원본 LIST_HEADER의 셀 선언에서 읽는다.
    let declared_rows: Vec<_> = (0..source.row_count)
        .map(|row| {
            source
                .cells
                .iter()
                .filter(|cell| cell.row == row && cell.row_span == 1)
                .map(|cell| cell.height)
                .max()
                .expect("각 행의 독립 높이 선언")
        })
        .collect();
    assert!(
        declared_rows
            .iter()
            .map(|height| u64::from(*height))
            .sum::<u64>()
            > u64::from(source.common.height)
    );
    let mut seen_body_rows = BTreeSet::new();
    for page in [93, 94] {
        let tree = core.build_page_render_tree(page).unwrap();
        let fragment = table(&tree.root, 934);
        for row in [0, 1] {
            let cells: Vec<_> = fragment.children.iter().filter(|node| {
                matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == row && cell.row_span == 1)
            }).collect();
            assert!(!cells.is_empty(), "머리행은 양쪽 조각에 반복된다");
            let declared = f64::from(declared_rows[usize::from(row)]) * 96.0 / 7200.0;
            for cell in cells {
                assert!((cell.bbox.height - declared).abs() <= 96.0 / 7200.0,
                    "p{} r{row}: 실제 머리행 상자는 원본 선언을 지킨다; got={}, declared={declared}", page+1, cell.bbox.height);
            }
        }
        let body_rows: BTreeSet<_> = fragment
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TableCell(cell) if cell.row >= 2 => Some(cell.row),
                _ => None,
            })
            .collect();
        assert!(!body_rows.is_empty(), "양쪽에 실제 본문 행이 있어야 한다");
        for row in body_rows {
            assert!(seen_body_rows.insert(row), "본문 행 중복");
        }
        if page == 94 {
            let following = line(&tree.root, 937);
            assert_eq!(ink(&text(following)), "제2절품목시장집중률추이");
            below(fragment, following);
        }
    }
    assert_eq!(
        seen_body_rows,
        (2..source.row_count).collect(),
        "분할 전후 원본 행의 소속·누락·중복"
    );
}

#[test]
fn issue7755_blank_flow_and_appendix_rows_keep_their_source_ownership() {
    let bytes =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(
        core.page_count(),
        315,
        "독립 Print의 전체 쪽수와 핵심 내용 소속을 함께 검사한다"
    );

    declared_rows_keep_their_size_across_page_fragments(&core);

    let p100 = core.build_page_render_tree(99).unwrap();
    let blank = line(&p100.root, 974);
    let heading = line(&p100.root, 975);
    let body = line(&p100.root, 976);
    assert!(text(blank).trim().is_empty() && blank.bbox.width > 0.0 && blank.bbox.height > 0.0);
    assert_eq!(ink(&text(heading)), "제3절소결");
    assert!(ink(&text(body)).starts_with("본장에서는제Ⅴ장에서산출한산업집중도"));
    below(blank, heading);
    below(heading, body);

    let p246 = core.build_page_render_tree(245).unwrap();
    let prior = table(&p246.root, 1298);
    let prior_source = source_table(&core, 1298);
    let last_prior_row = prior_source.row_count - 1;
    assert!(prior
        .children
        .iter()
        .any(|n| matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == last_prior_row)));
    let mut previous_end = None;
    for pi in 1299..=1315 {
        assert!(core.document().sections[0].paragraphs[pi]
            .text
            .trim()
            .is_empty());
        let blank = line(&p246.root, pi);
        assert!(text(blank).trim().is_empty() && blank.bbox.width > 0.0 && blank.bbox.height > 0.0);
        if let Some(end) = previous_end {
            assert!(end <= blank.bbox.y);
        }
        previous_end = Some(blank.bbox.y + blank.bbox.height);
    }
    for pi in [1316, 1317] {
        let suppressed = line(&p246.root, pi);
        assert!(text(suppressed).trim().is_empty());
        assert_eq!(
            suppressed.bbox.width, 0.0,
            "쓸 수 있는 본문 폭이 없는 저장 빈 줄은 추가 흐름을 점유하지 않는다"
        );
    }
    assert!(!nodes(&p246.root)
        .iter()
        .any(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(1320))));

    let p247 = core.build_page_render_tree(246).unwrap();
    let heading = line(&p247.root, 1318);
    let unit = line(&p247.root, 1319);
    let first = table(&p247.root, 1320);
    assert!(ink(&text(heading)).contains("2017년도광업및제조업품목집중도"));
    assert!(ink(&text(unit)).contains("단위:개,십억원,%"));
    below(heading, unit);
    below(unit, first);

    let source = source_table(&core, 1320);
    let mut actual = BTreeMap::new();
    let mut row_pages = BTreeMap::new();
    for page in 246..core.page_count() {
        let tree = core.build_page_render_tree(page as u32).unwrap();
        let fragment = table(&tree.root, 1320);
        let mut page_rows = BTreeSet::new();
        for node in &fragment.children {
            let RenderNodeType::TableCell(cell) = &node.node_type else {
                continue;
            };
            page_rows.insert(cell.row);
            for n in nodes(node) {
                if matches!(n.node_type, RenderNodeType::TextLine(..)) && !text(n).trim().is_empty()
                {
                    assert!(
                        n.bbox.y >= node.bbox.y
                            && n.bbox.y + n.bbox.height <= node.bbox.y + node.bbox.height + 1.0,
                        "p{} r{}c{}: 글줄이 셀의 세로 영역을 벗어났다",
                        page + 1,
                        cell.row,
                        cell.col
                    );
                }
            }
            if cell.row == 0 {
                continue;
            }
            assert!(
                actual
                    .insert((cell.row, cell.col), ink(&text(node)))
                    .is_none(),
                "같은 원본 셀을 중복 배치했다"
            );
        }
        assert!(page_rows.contains(&0), "반복 머리행 보존");
        for row in page_rows.into_iter().filter(|r| *r > 0) {
            assert!(
                row_pages.insert(row, page).is_none(),
                "원본 행이 두 쪽에 중복됐다"
            );
        }
    }
    assert_eq!(
        row_pages.keys().copied().collect::<Vec<_>>(),
        (1..source.row_count).collect::<Vec<_>>(),
        "본문 행 누락·중복 없이 원본 순서 유지"
    );
    assert_eq!(
        row_pages[&1], 246,
        "부록 첫 본문 행은 제목 뒤 첫 표 조각에 있다"
    );
    assert_eq!(
        row_pages[&(source.row_count - 1)],
        core.page_count() - 1,
        "마지막 원본 행과 실제 마지막 쪽이 일치한다"
    );
    let mut expected = BTreeMap::new();
    for cell in &source.cells {
        if cell.row == 0 {
            continue;
        }
        expected.insert(
            (cell.row, cell.col),
            ink(&cell
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>()),
        );
    }
    assert_eq!(
        actual.len(),
        expected.len(),
        "반복 머리행을 제외한 원본 셀의 수"
    );
    for (address, content) in expected {
        assert_eq!(
            actual.get(&address),
            Some(&content),
            "셀 {address:?}의 원본 내용 보존"
        );
    }
    let pages: Vec<_> = row_pages.values().copied().collect();
    assert!(
        pages.windows(2).all(|pair| pair[0] <= pair[1]),
        "원본 행의 문서 순서 보존"
    );
}
