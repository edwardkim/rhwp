//! #7699: 원본 저장 줄의 다단 전폭 Square 표·옆 차선·빈 줄을 최종 출력에서 보호한다.
//! 독립 한컴 Print 전쪽 Native/fresh WASM90% 및 고정 첫 쪽98% 이상과 확대 판독 뒤 추가했다.
//! 표 아래 본문·다음 단·쪽간 내용 보존과 원래 겹쳐 놓은 안내 글상자를 함께 검사한다.
//! px 기준선은 PDF 문자 origin의72pt→96dpi 환산값이며 ±2px다.
//! 입력/PDF SHA·양쪽 TSV·페이지 선정·before FAIL/after PASS는
//! mydocs/pr/assets/pr7699/README.md와 mydocs/pr/archives/pr_7699_review_impl.md 참조.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
};
const INPUTS: &str = "tests/fixtures/issue_7699/";
#[derive(Debug)]
struct Line {
    para: usize,
    rect: BoundingBox,
    text: String,
    baseline: f64,
}
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}
fn load(name: &str) -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(INPUTS)
                .join(name),
        )
        .expect("전쪽 검증한 원본 fixture"),
    )
    .unwrap()
}
fn lines(node: &RenderNode, out: &mut Vec<Line>) {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        for n in &node.children {
            if let RenderNodeType::TextLine(line) = &n.node_type {
                let Some(para) = line.para_index else {
                    continue;
                };
                let mut text = String::new();
                let mut baseline = None;
                for c in &n.children {
                    if let RenderNodeType::TextRun(run) = &c.node_type {
                        text.push_str(&run.text);
                        if !run.text.trim().is_empty() {
                            baseline.get_or_insert(c.bbox.y + run.baseline);
                        }
                    }
                }
                out.push(Line {
                    para,
                    rect: n.bbox,
                    text,
                    baseline: baseline.unwrap_or(n.bbox.y),
                });
            }
        }
        return;
    }
    if matches!(
        node.node_type,
        RenderNodeType::Page(_) | RenderNodeType::Body { .. }
    ) {
        for c in &node.children {
            lines(c, out);
        }
    }
}
fn title_table(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(&node.node_type,RenderNodeType::Table(t) if t.para_index==Some(0)&&t.control_index==Some(5))
    {
        return Some(node.bbox);
    }
    node.children.iter().find_map(title_table)
}
fn all_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        all_text(child, text);
    }
}

fn inspect(name: &str) -> (DocumentCore, Vec<Vec<Line>>, BoundingBox) {
    let core = load(name);
    assert_eq!(core.page_count(), 3, "독립 Print 전체 쪽수");
    let mut pages = Vec::new();
    let mut table = None;
    for p in 0..core.page_count() {
        let tree = core.build_page_render_tree(p).unwrap();
        let mut out = Vec::new();
        lines(&tree.root, &mut out);
        if p == 0 {
            table = title_table(&tree.root);
            let mut text = String::new();
            all_text(&tree.root, &mut text);
            let text = compact(&text);
            for instruction in ["논문양은2페이지제출권장", "최대5페이지까지가능"]
            {
                assert_eq!(
                    text.matches(instruction).count(),
                    1,
                    "독립 Print에도 겹쳐 놓인 안내 글상자는 숨기거나 중복하지 않는다"
                );
            }
        }
        pages.push(out);
    }
    let table = table.expect("첫 쪽 제목 표");
    for pi in [0, 2] {
        let actual: String = pages
            .iter()
            .flatten()
            .filter(|l| l.para == pi)
            .map(|l| l.text.as_str())
            .collect();
        assert_eq!(
            compact(&actual),
            compact(&core.document().sections[0].paragraphs[pi].text),
            "pi={pi}: 단과 쪽을 가로지르는 내용 순서·누락·중복"
        );
    }
    (core, pages, table)
}
fn check_wide(name: &str) {
    let (core, pages, table) = inspect(name);
    let main = &pages[0];
    let source = &core.document().sections[0].paragraphs[0];
    let rhwp::model::control::Control::Table(t) = &source.controls[5] else {
        panic!("제목 표")
    };
    let end = table.y + table.height + f64::from(t.outer_margin_bottom) / 75.0;
    for line in main.iter().filter(|l| [0, 1, 2].contains(&l.para)) {
        assert!(
            line.rect.y + 0.5 >= end,
            "표·바깥 여백 아래의 본문: {line:?}, end={end}"
        );
    }
    let heading = main.iter().find(|l| l.para == 0).unwrap();
    let intro = main.iter().find(|l| l.para == 2).unwrap();
    assert!(
        heading.rect.y + heading.rect.height < intro.rect.y,
        "제목 뒤 빈 줄과 본문 순서"
    );
    assert!(
        main.iter()
            .any(|l| l.para == 2 && l.rect.x > table.x + table.width / 2.0),
        "둘째 단 본문도 보존"
    );
    // 한컴 Print PDF의 72pt 원점을 96dpi로 환산한 독립 기준선.
    assert!(
        (intro.baseline - 733.877197265625).abs() <= 2.0,
        "첫 본문 Print 기준선: {}",
        intro.baseline
    );
    assert!(
        (heading.baseline - 692.9195963541666).abs() <= 2.0,
        "제목 Print 기준선"
    );
    let second = main
        .iter()
        .find(|l| l.para == 2 && l.rect.x > table.x + table.width / 2.0)
        .unwrap();
    assert!(
        (second.baseline - 691.7996419270834).abs() <= 2.0,
        "둘째 단의 Print 시작 기준선"
    );
}
#[test]
fn original_hwp_body_clears_top_table_in_both_columns() {
    check_wide("kiit-standard-original.hwp");
}
#[test]
fn hancom_hwpx_body_clears_top_table_in_both_columns() {
    check_wide("kiit-standard-hancom.hwpx");
}
#[test]
fn allow_overlap_does_not_cancel_text_exclusion() {
    check_wide("kiit-standard-allow-overlap.hwpx");
}
#[test]
fn narrow_square_keeps_side_lanes_and_empty_line_flow() {
    let (_, pages, table) = inspect("kiit-narrow85-hancom.hwp");
    let main = &pages[0];
    let first = main.iter().find(|l| l.para == 2).unwrap();
    assert!(
        first.rect.y < table.y + table.height,
        "본문이 표 옆에서 시작해야 한다"
    );
    assert!(
        (first.baseline - 179.34954833984375).abs() <= 2.0,
        "좁은 표 첫 본문의 독립 Print 기준선"
    );
    for l in main
        .iter()
        .filter(|l| l.para == 2 && l.rect.y < table.y + table.height)
    {
        assert!(
            l.rect.x + l.rect.width <= table.x + 0.5 || l.rect.x >= table.x + table.width - 0.5,
            "표 옆 차선과 표 내부를 구별: {l:?}, {table:?}"
        );
    }
}

#[test]
fn empty_source_line_between_square_host_and_body_owns_flow() {
    let (_, pages, _) = inspect("kiit-narrow85-hancom.hwp");
    let main = &pages[0];
    let blanks: Vec<_> = main.iter().filter(|l| l.para == 1).collect();
    assert_eq!(blanks.len(), 1, "저장된 빈 줄의 소유·누락·중복");
    let blank = blanks[0];
    let heading = main.iter().rev().find(|l| l.para == 0).unwrap();
    let first = main.iter().find(|l| l.para == 2).unwrap();
    assert!(
        blank.rect.y >= heading.rect.y + heading.rect.height,
        "제목 뒤 빈 줄"
    );
    assert!(
        first.rect.y >= blank.rect.y + blank.rect.height,
        "빈 줄 뒤 본문"
    );
}
