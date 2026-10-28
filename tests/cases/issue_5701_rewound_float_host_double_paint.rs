//! #5701: 자리차지 표 위 선행 글줄과 다음 쪽 이어받기의 실제 소속을 보존한다.
//!
//! 이전 IR 슬라이스는 원문 쪽 좌표를 남긴 수동 생성본이었다. 독립 한컴2020
//! 출력은 3쪽이고, 후속 문단의 첫 줄은 1쪽 표 위에, 나머지는 2쪽 표 아래에 있다.
//! 기존 2쪽 핀과 후속 문단 전체가 표 아래라는 전제는 이 정본과 달랐다.
//! 한컴2020에서 같은 슬라이스를 정상 재저장한 기존 픽스처로 검사한다.
//! 이전 IR 원본과 두 입력의 정확한 PDF는 검토 자산에 보존한다.
//! Native/fresh WASM 전3쪽 90% 이상 근거는 slice5701_prefix_validation.json을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::HeadType;
use serde_json::Value;

const SAMPLE: &str = "samples/issue5701/1270000-202200012_slice_p76_rewound_host.hwp";

#[test]
fn issue_5701_float_preserves_host_and_split_follower_ownership() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        3,
        "정확한 입력의 독립 한컴 PDF는 3쪽이다"
    );
    let mut pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
        let json: Value = serde_json::from_str(&tree.root.to_json()).expect("렌더 트리 JSON");
        let mut facts = PageFacts::default();
        collect(&json, &mut facts);
        pages.push(facts);
    }
    assert_eq!(
        pages[0].lines_for(7).len(),
        4,
        "호스트 네 줄은 1쪽에서 한 번만 그린다"
    );
    assert!(pages[1..].iter().all(|page| page.lines_for(7).is_empty()));
    assert_eq!(
        pages[0].lines_for(8).len(),
        1,
        "후속 첫 줄은 1쪽 표 위 공간을 소유한다"
    );
    assert_eq!(
        pages[1].lines_for(8).len(),
        4,
        "2쪽은 이미 소비한 첫 줄 다음부터 이어받는다"
    );
    assert!(pages[2].lines_for(8).is_empty());
    assert_eq!(pages[0].tables.len(), 1);
    assert_eq!(pages[1].tables.len(), 1);
    assert!(pages[2].tables.is_empty());
    let first = &pages[0].tables[0];
    let last = &pages[1].tables[0];
    assert_eq!(first.cells.len(), 14 * 4, "정본의 첫 조각은 14행이다");
    assert_eq!(
        last.cells.len(),
        2 * 4,
        "정본의 이어받기는 마지막 두 행이다"
    );
    let follower_first = pages[0].lines_for(8)[0];
    assert!(pages[0].lines_for(7).last().unwrap().bottom <= follower_first.top);
    assert!(
        follower_first.bottom <= first.top,
        "선행 후속 줄은 표 상자에 겹치지 않는다"
    );
    assert!(
        last.bottom <= pages[1].lines_for(8)[0].top,
        "표 아래 여백을 닫은 뒤 본문을 잇는다"
    );
    assert_eq!(pages[1].lines_for(18).len(), 1);
    assert_eq!(pages[2].lines_for(18).len(), 1);
    for page in &pages {
        assert!(page
            .lines
            .iter()
            .all(|line| line.bottom <= page.body_bottom));
    }
    let source = &core.document().sections[0].paragraphs;
    for pi in [7, 8, 18] {
        let painted: String = pages
            .iter()
            .flat_map(|page| page.lines_for(pi))
            .map(|line| line.text.as_str())
            .collect();
        // 자동 글머리표는 문단 본문에 없으므로 원본 스타일의 문자도 함께 보존한다.
        let shape = &core.document().doc_info.para_shapes[source[pi].para_shape_id as usize];
        let mut expected = source[pi].text.clone();
        if shape.head_type == HeadType::Bullet {
            let bullet = &core.document().doc_info.bullets[(shape.numbering_id - 1) as usize];
            expected.insert(0, bullet.bullet_char);
        }
        assert_eq!(
            visible(&painted),
            visible(&expected),
            "문단 {pi}: 누락·중복 없이 원문 순서를 보존한다"
        );
    }
    let Control::Table(table) = &source[7].controls[0] else {
        panic!("호스트의 표 컨트롤")
    };
    let expected: Vec<String> = table
        .cells
        .iter()
        .map(|cell| {
            visible(
                &cell
                    .paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
            )
        })
        .collect();
    let painted: Vec<String> = pages
        .iter()
        .flat_map(|page| &page.tables)
        .flat_map(|table| table.cells.iter().map(|text| visible(text)))
        .collect();
    assert_eq!(painted, expected, "표 모든 칸을 원문 순서로 한 번씩 그린다");
}

#[derive(Default)]
struct PageFacts {
    body_bottom: f64,
    lines: Vec<Line>,
    tables: Vec<Table>,
}
impl PageFacts {
    fn lines_for(&self, pi: usize) -> Vec<&Line> {
        self.lines.iter().filter(|line| line.pi == pi).collect()
    }
}
struct Line {
    pi: usize,
    top: f64,
    bottom: f64,
    text: String,
}
struct Table {
    top: f64,
    bottom: f64,
    cells: Vec<String>,
}

fn visible(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}
fn text(node: &Value) -> String {
    if node["type"] == "TextRun" {
        return node["text"].as_str().unwrap_or("").to_owned();
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(text)
        .collect()
}
fn collect(node: &Value, page: &mut PageFacts) {
    let top = node["bbox"]["y"].as_f64().unwrap_or(0.0);
    let bottom = top + node["bbox"]["h"].as_f64().unwrap_or(0.0);
    match node["type"].as_str() {
        Some("Body") => page.body_bottom = bottom,
        Some("Table") => {
            let cells = node["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|child| child["type"] == "Cell")
                .map(text)
                .collect();
            page.tables.push(Table { top, bottom, cells });
            return;
        }
        Some("TextLine") => {
            if let Some(pi) = node["pi"].as_u64() {
                page.lines.push(Line {
                    pi: pi as usize,
                    top,
                    bottom,
                    text: text(node),
                });
            }
            return;
        }
        _ => {}
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, page);
    }
}
