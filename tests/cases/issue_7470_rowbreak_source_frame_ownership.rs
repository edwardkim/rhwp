//! 원본 RowBreak의 물리 조각 소유를 독립 한컴 PDF 18쪽과 대조한다.
//! 정본은 pdf/rowbreak-problem-pages-hwp-2024.pdf이며 Native/fresh WASM을 검증한다.
//! 내용 소유와 원본 패딩·줄 간격을 관계로 검사한다. 양쪽 98% 이상인
//! 13쪽의 제목 기준선은 독립 Print의 ±2 CSSpx로 함께 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    model::{control::Control, table::VerticalAlign},
};
use serde_json::Value;
use std::{path::Path, process::Command, sync::OnceLock};

const SAMPLE: &str = "samples/rowbreak-problem-pages.hwp";

struct TreeDirectory(std::path::PathBuf);

impl Drop for TreeDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let out = TreeDirectory(
            std::env::temp_dir().join(format!("rhwp-rowbreak-tree-{}-{stamp}", std::process::id())),
        );
        std::fs::create_dir(&out.0).expect("temporary render tree directory");
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let result = Command::new(bin)
            .arg("export-render-tree")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
            .args(["--compat", "2024"])
            .arg("-o")
            .arg(&out.0)
            .output()
            .expect("render original sample");
        assert!(
            result.status.success(),
            "render failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(&out.0)
            .unwrap()
            .map(|f| f.unwrap().path())
            .filter(|p| p.extension().is_some_and(|s| s == "json"))
            .collect();
        files.sort();
        files
            .into_iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect()
    })
}

fn walk<'a>(n: &'a Value, cell: Option<&'a Value>, out: &mut Vec<(&'a Value, Option<&'a Value>)>) {
    let cell = if n["type"] == "Cell" { Some(n) } else { cell };
    out.push((n, cell));
    if let Some(children) = n["children"].as_array() {
        for child in children {
            walk(child, cell, out);
        }
    }
}

fn nodes(page: &Value) -> Vec<(&Value, Option<&Value>)> {
    let mut out = Vec::new();
    walk(page, None, &mut out);
    out
}

fn top(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    top(n) + n["bbox"]["h"].as_f64().unwrap()
}
fn line_text(n: &Value) -> String {
    n["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["text"].as_str())
        .collect()
}
fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}
fn occurrences(needle: &str) -> Vec<(usize, &Value, Option<&Value>)> {
    pages()
        .iter()
        .enumerate()
        .flat_map(|(page, p)| {
            nodes(p)
                .into_iter()
                .filter(|(n, _)| n["type"] == "TextLine")
                .filter(|(n, _)| compact(&line_text(n)).contains(needle))
                .map(move |(n, c)| (page, n, c))
        })
        .collect()
}

#[test]
fn original_page_count_and_cut_labels_keep_their_owners() {
    assert_eq!(pages().len(), 18, "same original PDF has 18 pages");
    for (label, owner) in [
        ("4.2공통표준도메인관리항목", 10),
        ("4.3공통표준도메인정의기준", 11),
    ] {
        let found = occurrences(label);
        assert_eq!(
            found.len(),
            1,
            "{label} must occur once, without losing or replaying a unit"
        );
        assert_eq!(found[0].0, owner, "{label} must keep its PDF page owner");
    }

    let title = "<공공데이터제공·관리실무매뉴얼,3.공공데이터구축단계품질관리(P.58~60)발췌>";
    let found = occurrences(title);
    assert_eq!(found.len(), 1, "첫 제목은 누락·재생 없이 한 번 표시한다");
    let (page, line, owner) = found[0];
    assert_eq!(page, 12, "독립 PDF에서 첫 제목은 13쪽 조각이 소유한다");
    let owner = owner.expect("첫 제목에는 물리 셀 소유자가 있다");

    let mut source = DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    source.set_hangul2024_compat(true);
    let document = source.document();
    let mut sources = Vec::new();
    for section in &document.sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                for cell in &table.cells {
                    if cell
                        .paragraphs
                        .first()
                        .is_some_and(|p| compact(&p.text).contains(title))
                    {
                        sources.push((table, cell));
                    }
                }
            }
        }
    }
    assert_eq!(sources.len(), 1, "원본 첫 제목의 셀 소유를 식별한다");
    let (table, cell) = sources[0];
    assert_eq!(cell.vertical_align, VerticalAlign::Center);
    let first = &cell.paragraphs[0];
    let stored_band_hu = f64::from(first.line_segs[0].vertical_pos);
    let before_hu =
        f64::from(document.doc_info.para_shapes[first.para_shape_id as usize].spacing_before) / 2.0;
    assert!(stored_band_hu > 0.0 && before_hu > 0.0);
    let padding = cell.effective_padding(&table.padding);
    let actual_band_hu = (top(line) - top(owner)) * 75.0 - f64::from(padding.top);
    // 저장 앞 띠와 문단 앞 간격은 각각 보존한다. 원본 Center의 남은
    // 물리 공간까지 0으로 고정하지 않으며, 실제 소유 내용의 양쪽 여유를 대조한다.
    // 독립 Print의 첫 제목 기준선과 현재 출력 대조는 Stage70 증적을 따른다.
    let serialization_quantum_px = 0.1;
    let source_leading_hu = stored_band_hu + before_hu;
    assert!(
        actual_band_hu + serialization_quantum_px * 75.0 >= source_leading_hu,
        "첫 저장 띠 또는 문단 앞 간격을 잃었습니다: 실제 {actual_band_hu}HU, 원본 {source_leading_hu}HU"
    );
    let last_content_bottom = nodes(owner)
        .into_iter()
        .filter(|(node, _)| node["type"] == "TextLine")
        .map(|(line, _)| bottom(line))
        .reduce(f64::max)
        .expect("첫 조각이 실제로 소유한 마지막 줄 상자");
    let leading_space = top(line) - top(owner) - f64::from(padding.top) / 75.0 - before_hu / 75.0;
    let trailing_space = bottom(owner) - f64::from(padding.bottom) / 75.0 - last_content_bottom;
    assert!(
        (leading_space - trailing_space).abs() <= serialization_quantum_px,
        "원본 Center의 첫 조각 여유가 갈라졌습니다: 앞 {leading_space}, 뒤 {trailing_space}"
    );

    // 전 18쪽 양쪽 90% 이상·13쪽 양쪽 98% 이상과 확대 판독을 확인했다.
    // 독립 Print의 제목 기준선 354.720001pt를 96 DPI CSSpx로 환산한다.
    // 글리프 잉크 외곽을 줄 기준선으로 대용하지 않는다.
    fn rendered_text(node: &rhwp::renderer::render_tree::RenderNode) -> String {
        use rhwp::renderer::render_tree::RenderNodeType;
        let mut result = match &node.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
            _ => String::new(),
        };
        for child in &node.children {
            result.push_str(&rendered_text(child));
        }
        result
    }
    fn first_baseline(node: &rhwp::renderer::render_tree::RenderNode) -> Option<f64> {
        use rhwp::renderer::render_tree::RenderNodeType;
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.display_or_text().trim().is_empty() {
                return Some(node.bbox.y + run.baseline);
            }
        }
        node.children.iter().find_map(first_baseline)
    }
    fn title_baseline(node: &rhwp::renderer::render_tree::RenderNode, title: &str) -> Option<f64> {
        use rhwp::renderer::render_tree::RenderNodeType;
        if matches!(&node.node_type, RenderNodeType::TextLine(_))
            && compact(&rendered_text(node)) == title
        {
            return first_baseline(node);
        }
        node.children
            .iter()
            .find_map(|child| title_baseline(child, title))
    }
    let tree = source
        .build_page_render_tree(12)
        .expect("완료된 독립 PDF 13쪽");
    let baseline = title_baseline(&tree.root, title).expect("같은 소유 제목의 실제 기준선");
    let printed_baseline = 354.720001 * 96.0 / 72.0;
    assert!(
        (baseline - printed_baseline).abs() <= 2.0,
        "완료된 13쪽 제목 기준선: 실제 {baseline}, 독립 Print {printed_baseline}"
    );
}

#[test]
fn nested_terminal_line_is_owned_once_inside_the_continuation_cell() {
    let found = occurrences("하여정보를제공하거나정보의제공을매개하는자를말한다.");
    assert_eq!(
        found.len(),
        1,
        "last nested unit must neither disappear nor repeat"
    );
    let (page, line, cell) = found[0];
    assert_eq!(page, 7, "nested terminal sentence belongs to PDF page 8");
    let cell = cell.expect("nested line has a physical cell owner");
    assert!(
        top(line) >= top(cell) - 0.05 && bottom(line) <= bottom(cell) + 0.05,
        "terminal line escapes its source fragment: line {:?}, cell {:?}",
        line["bbox"],
        cell["bbox"]
    );
}

#[test]
fn visible_lines_stay_inside_both_sides_of_the_source_cut() {
    for page in [6, 7, 10, 11] {
        for (line, cell) in nodes(&pages()[page]) {
            if line["type"] != "TextLine" || compact(&line_text(line)).is_empty() {
                continue;
            }
            let Some(cell) = cell else {
                continue;
            };
            assert!(
                top(line) >= top(cell) - 0.05 && bottom(line) <= bottom(cell) + 0.05,
                "page {}, text {:?}: line {:?} escapes cell {:?}",
                page + 1,
                line_text(line),
                line["bbox"],
                cell["bbox"]
            );
        }
    }
}

#[test]
fn first_fragment_keeps_its_blank_band_below_the_last_owned_line() {
    let source = DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    let Control::Table(table) = &source.document().sections[1].paragraphs[5].controls[0] else {
        panic!("source table");
    };
    let original = table
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 2)
        .unwrap();
    let padding = original.effective_padding(&table.padding);
    let frame = nodes(&pages()[10])
        .into_iter()
        .find(|(n, _)| n["type"] == "Table" && n["pi"] == 5 && n["ci"] == 0)
        .expect("original first physical table frame")
        .0;
    // 원본이 선언한 첫 물리 조각은 독립 PDF의 테두리와 대응한다.
    // 내용 컷으로 그 조각의 빈 띠를 지우면 안 된다.
    assert!(
        bottom(frame) - top(frame) + 0.1 >= f64::from(table.common.height) / 75.0,
        "first frame lost source-owned physical space: {:?}",
        frame["bbox"]
    );
    let found = occurrences("4.2공통표준도메인관리항목");
    assert_eq!(found.len(), 1);
    let (_, line, owner) = found[0];
    let cell = owner.unwrap();
    assert!(bottom(line) + f64::from(padding.bottom) / 75.0 <= bottom(cell) + 0.05,
        "physical first frame must retain source bottom padding and blank band after 4.2; line {:?}, cell {:?}", line["bbox"], cell["bbox"]);
    let next = occurrences("4.3공통표준도메인정의기준");
    assert_eq!(next.len(), 1);
    assert_eq!(next[0].0, 11, "reserving blank space cannot consume 4.3");
}

#[test]
fn visible_successor_uses_the_original_line_origin_without_readding_lead() {
    let source = DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap(),
    )
    .unwrap();
    let original = &source.document().sections[0].paragraphs[12];
    let body = nodes(&pages()[4])
        .into_iter()
        .find(|(n, _)| n["type"] == "Body")
        .unwrap()
        .0;
    let line = nodes(&pages()[4])
        .into_iter()
        .find(|(n, c)| c.is_none() && n["type"] == "TextLine" && n["pi"] == 12)
        .unwrap()
        .0;
    let expected = f64::from(original.line_segs[0].vertical_pos) / 75.0;
    // 저장 줄 원점과 정규화한 PDF 기준선이 대응한다. 글리프 잉크 외곽은 비교하지 않는다.
    assert!((top(line) - top(body) - expected).abs() <= 0.1,
        "visible successor shares the stored table-end boundary; line {:?}, body {:?}, saved={expected}", line["bbox"], body["bbox"]);
}

#[test]
fn coanchored_tac_does_not_skip_or_overlap_its_float_sibling() {
    let all = nodes(&pages()[15]);
    let tables: Vec<_> = all
        .iter()
        .filter(|(n, _)| n["type"] == "Table" && n["pi"] == 28)
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(
        tables.len(),
        2,
        "TAC commitment must leave the float sibling to the normal control loop"
    );
    let tac = tables
        .iter()
        .find(|n| n["ci"] == 1 && n["rows"] == 3)
        .unwrap();
    let float = tables
        .iter()
        .find(|n| n["ci"] == 0 && n["rows"] == 1)
        .unwrap();
    assert!(
        bottom(tac) <= top(float),
        "source TAC must precede the separate float frame: {:?}, {:?}",
        tac["bbox"],
        float["bbox"]
    );
}
