//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-10-31T00:28:31+09:00; 직전 파일 커밋: 2026-10-30T20:08:21+09:00 (`9edfcfb1dfa7b3228397c6cef0760b0ab4a537e7`).
//! 2026-10-28 측정 raw(96dpi, PDFium): issue8168의 multiline above·below 및 page-relative HWP, issue8271의 topandbottom·behindtext HWPX는 각각 1쪽 Native·fresh WASM 100.00000%, 전1쪽 최저 각 100.00000%.
//! issue2007_nested_cell_pagination_42065.hwp: 2026-10-31 측정 3쪽 99.76% / 99.75%; 전 17쪽 최저 94.96% / 96.32%.
//! sungeo.hwp: 21쪽 99.94109% / 99.98302%, 22쪽 99.93511% / 99.97305%; 전94쪽 최저 99.85164% / 99.92219%.
//! hwpspec.hwp: 23쪽 99.99389% / 99.79148%, 54쪽 각 99.99765%; 전178쪽 최저 92.45593% / 92.83271%.
//! 중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp: 1쪽 99.16453% / 99.14176%, 2쪽 93.57998% / 93.34172%; 전2쪽 최저 93.57998% / 93.34172%.
//! issue3307_outline_number.hwpx: 2026-10-31 측정 8쪽 99.99% / 99.94%; 전 9쪽 최저 97.89% / 97.81%.
//! 각 수치 쌍의 순서는 Native / fresh WASM이다. 수정 문서의 근거: mydocs/pr/assets/issue_8371/regression/.
//! 나머지 근거: mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-native-comparison.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/original-floatfix-wasm-comparison.json,
//! mydocs/working/assets/review_integration_20261026/validation/post-rebase-8328/header-evidence-ted3-matrix.json.
//!
//! 저장 프레임·실제 control 줄·표의 물리 상자 소유를 독립 Print와 대조한 회귀.
//!
//! #8133/#8184/#8213/#8206/#8283 통합 보정. 관련 원본의 전쪽 Native/fresh WASM
//! 90% 통과 뒤 추가한다. JSON 골든이나 현재 출력의 절대 좌표를 기대값으로 쓰지 않는다.
//! CLI producer를 바꾸어 동일 검사를 수정 전후 실행할 수 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::table::{Table, VerticalAlign};
use serde_json::Value;

struct ExportDir(PathBuf);

impl ExportDir {
    fn new() -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("output/source-frame-contracts");
        std::fs::create_dir_all(&base).unwrap();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = base.join(format!(
            "{}-{stamp}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for ExportDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(name: &str) -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap(),
    )
    .unwrap()
}

fn page(name: &str, index: u32) -> Value {
    page_for_compat(name, index, "2022")
}

fn page_for_compat(name: &str, index: u32, compat: &str) -> Value {
    let tmp = ExportDir::new();
    let binary =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let output = Command::new(binary)
        .arg("export-render-tree")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .args(["--compat", compat, "-p"])
        .arg(index.to_string())
        .arg("-o")
        .arg(&tmp.0)
        .output()
        .unwrap();
    assert!(output.status.success(), "CLI 실행 실패: {output:?}");
    serde_json::from_slice(
        &std::fs::read(tmp.0.join(format!("render_tree_{:03}.json", index + 1))).unwrap(),
    )
    .unwrap()
}

fn children(node: &Value) -> impl Iterator<Item = &Value> {
    node.get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn text(node: &Value) -> String {
    let mut value = node["text"].as_str().unwrap_or_default().to_owned();
    for child in children(node) {
        value.push_str(&text(child));
    }
    value
}

fn nodes<'a>(root: &'a Value, kind: &str) -> Vec<&'a Value> {
    fn visit<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
        if node["type"] == kind {
            out.push(node);
        }
        for child in children(node) {
            visit(child, kind, out);
        }
    }
    let mut out = Vec::new();
    visit(root, kind, &mut out);
    out
}

fn line<'a>(root: &'a Value, needle: &str) -> &'a Value {
    let selected: Vec<_> = nodes(root, "TextLine")
        .into_iter()
        .filter(|node| text(node).contains(needle))
        .collect();
    assert_eq!(selected.len(), 1, "글줄의 유일한 소유: {needle}");
    selected[0]
}

fn bbox(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    bbox(node, "y") + bbox(node, "h")
}
fn px(value: impl Into<f64>) -> f64 {
    value.into() / 75.0
}
fn close(actual: f64, expected: f64, context: &str) {
    // CLI JSON의 소수 한 자리 양자화만 허용한다.
    assert!(
        (actual - expected).abs() <= 0.25,
        "{context}: {actual} vs {expected}"
    );
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch > '\u{1f}')
        .collect()
}

fn textbox_rect(root: &Value) -> &Value {
    let selected: Vec<_> = nodes(root, "Rect")
        .into_iter()
        .filter(|node| text(node).contains("중요"))
        .collect();
    assert_eq!(selected.len(), 1, "한 원본 글상자를 한 번만 그려야 한다");
    selected[0]
}

fn float_shape(core: &DocumentCore) -> &CommonObjAttr {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| {
            if let Control::Shape(shape) = control {
                Some(shape.common())
            } else {
                None
            }
        })
        .unwrap()
}

#[test]
fn multiline_float_keeps_its_control_row_before_exclusion() {
    const ABOVE: &str =
        "samples/issue8168/multiline_tac_picture_para_topandbottom_offset_above.hwp";
    const BELOW: &str =
        "samples/issue8168/multiline_tac_picture_para_topandbottom_offset_below.hwp";
    let above = open(ABOVE);
    let below = open(BELOW);
    assert_eq!(
        above.document().sections[0].paragraphs[0].text,
        below.document().sections[0].paragraphs[0].text
    );
    let (a, b) = (float_shape(&above), float_shape(&below));
    for common in [a, b] {
        assert_eq!(common.vert_rel_to, VertRelTo::Para);
        assert_eq!(common.horz_rel_to, HorzRelTo::Paper);
        assert_eq!(common.horz_align, HorzAlign::Left);
        assert_eq!(common.text_wrap, TextWrap::TopAndBottom);
    }
    let (a_tree, b_tree) = (page(ABOVE, 0), page(BELOW, 0));
    let (a_box, b_box) = (textbox_rect(&a_tree), textbox_rect(&b_tree));
    let (a_third, b_third) = (
        line(&a_tree, "위아래로 배치한"),
        line(&b_tree, "위아래로 배치한"),
    );
    assert!(
        bottom(b_third) < bbox(b_box, "y"),
        "아래 도형은 셋째 글줄을 배제하지 않는다"
    );
    // 두 입력은 같은 원시 control 슬롯을 가진다. 아래 도형의 비배제 글줄이
    // 두 도형 모두의 앵커이며, 배제로 이동한 위 입력의 최종 글줄은 앵커가 아니다.
    for (common, rect) in [(a, a_box), (b, b_box)] {
        close(
            bbox(rect, "y") - bbox(b_third, "y"),
            px(common.vertical_offset) + px(common.margin.top),
            "원본 control 줄 세로 앵커",
        );
        close(
            bbox(rect, "x"),
            px(common.horizontal_offset) + px(common.margin.left),
            "종이 절대 가로 원점과 원본 바깥 여백",
        );
    }
    close(
        bbox(a_third, "y"),
        bottom(a_box) + px(a.margin.bottom),
        "실제 배제 띠 아래 셋째 글줄",
    );
    close(
        bbox(line(&a_tree, "다음 문단"), "y") - bbox(a_third, "y"),
        bbox(line(&b_tree, "다음 문단"), "y") - bbox(b_third, "y"),
        "같은 원본 후속 글줄 전진",
    );
    for tree in [&a_tree, &b_tree] {
        assert_eq!(nodes(tree, "Image").len(), 1, "TAC 그림의 단일 소유");
    }
}

#[test]
fn page_relative_float_preserves_source_margins_and_single_ownership() {
    for name in [
        "samples/issue8271/tac_picture_topandbottom_textbox.hwpx",
        "samples/issue8271/tac_picture_behindtext_textbox.hwpx",
        "samples/issue8168/tac_picture_with_page_relative_topandbottom_textbox.hwp",
    ] {
        let core = open(name);
        let common = float_shape(&core);
        assert_eq!(common.horz_rel_to, HorzRelTo::Paper);
        let tree = page(name, 0);
        let rect = textbox_rect(&tree);
        let reference_top = match common.vert_rel_to {
            VertRelTo::Paper => 0.0,
            VertRelTo::Page => {
                let columns = nodes(&tree, "Column");
                assert_eq!(columns.len(), 1, "원본 단일 본문 영역");
                bbox(columns[0], "y")
            }
            other => panic!("종이/본문 기준 대조군이어야 한다: {other:?}"),
        };
        close(
            bbox(rect, "x"),
            px(common.horizontal_offset) + px(common.margin.left),
            name,
        );
        close(
            bbox(rect, "y"),
            reference_top + px(common.vertical_offset) + px(common.margin.top),
            name,
        );
        assert_eq!(nodes(&tree, "Image").len(), 1);
    }
}

fn source_tables<'a>(paragraphs: &'a [Paragraph], out: &mut Vec<(&'a Paragraph, &'a Table)>) {
    for para in paragraphs {
        for control in &para.controls {
            if let Control::Table(table) = control {
                out.push((para, table));
                for cell in &table.cells {
                    source_tables(&cell.paragraphs, out);
                }
            }
        }
    }
}

#[test]
fn whole_tac_line_keeps_declared_cell_frame_and_all_source_text() {
    const INPUT: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";
    let core = open(INPUT);
    let mut tables = Vec::new();
    for section in &core.document().sections {
        source_tables(&section.paragraphs, &mut tables);
    }
    let (host, source) = tables
        .into_iter()
        .find(|(_, table)| {
            table
                .cells
                .first()
                .and_then(|cell| cell.paragraphs.first())
                .is_some_and(|para| {
                    para.text
                        .trim_start_matches(|ch: char| !ch.is_alphanumeric())
                        .starts_with("국세징수법")
                })
        })
        .expect("국세징수법을 소유한 원본 내부 표");
    let cell = &source.cells[0];
    assert!(source.common.treat_as_char);
    assert_eq!(cell.vertical_align, VerticalAlign::Center);
    assert_eq!(cell.height, source.common.height);
    assert_eq!(host.line_segs.len(), 1);
    assert_eq!(
        i64::from(host.line_segs[0].line_height),
        i64::from(source.common.height)
            + i64::from(source.common.margin.top)
            + i64::from(source.common.margin.bottom)
    );
    let tree = page(INPUT, 10);
    let painted = nodes(&tree, "Table")
        .into_iter()
        .find(|node| {
            text(node)
                .trim_start_matches(|ch: char| !ch.is_alphanumeric())
                .starts_with("국세징수법")
        })
        .unwrap();
    close(
        bbox(painted, "h"),
        px(cell.height),
        "전체 TAC 줄이 소유한 선언 최소 높이",
    );
    let source_text: String = cell
        .paragraphs
        .iter()
        .map(|para| para.text.as_str())
        .collect();
    assert_eq!(
        compact(&text(painted)),
        compact(&source_text),
        "원본 칸 내용의 순서·완전성"
    );
    for line in nodes(painted, "TextLine") {
        assert!(
            bbox(line, "y") >= bbox(painted, "y") && bottom(line) <= bottom(painted),
            "원본 글줄이 실제 물리 칸 안에 포함되어야 한다"
        );
    }
}

#[test]
fn restarted_heading_and_its_body_keep_their_source_page() {
    const INPUT: &str = "samples/basic/sungeo.hwp";
    for (index, heading, body) in [
        (20, "노류장화", "길 가의 버들과"),
        (21, "농조연운", "새장 속에 있는 새는"),
    ] {
        let tree = page(INPUT, index);
        let title = line(&tree, heading);
        let paragraph = line(&tree, body);
        assert!(
            bottom(title) < bbox(paragraph, "y"),
            "원본 표제와 본문은 같은 쪽에서 순서 유지"
        );
        let first = nodes(&tree, "TextLine").into_iter().next().unwrap();
        assert_eq!(
            first["pi"], title["pi"],
            "앞 쪽의 본문을 새 쪽의 prefix로 재생하지 않는다"
        );
    }
}

#[test]
fn coanchored_host_keeps_both_tables_on_the_same_source_page() {
    let tree = page("samples/hwpspec.hwp", 22);
    let tables = nodes(&tree, "Table");
    let history = tables
        .iter()
        .find(|node| text(node).contains("DocHistory"))
        .unwrap();
    let item = tables
        .iter()
        .find(|node| text(node).contains("히스토리 아이템"))
        .unwrap();
    assert!(
        bottom(history) < bbox(item, "y"),
        "공동 호스트의 첫 표와 다음 표가 같은 쪽에서 순서 유지"
    );
    assert!(bbox(history, "y") >= bbox(&tree, "y") && bottom(item) <= bottom(&tree));
}

#[test]
fn table_split_policy_does_not_duplicate_the_heading_gap() {
    let tree = page("samples/hwpspec.hwp", 53);
    let a_heading = line(&tree, "4.2.9.6.");
    let b_heading = line(&tree, "4.2.10.");
    let tables = nodes(&tree, "Table");
    let a = tables
        .iter()
        .find(|node| text(node).contains("개체 공통 속성"))
        .unwrap();
    let b = tables
        .iter()
        .find(|node| text(node).contains("MAKE_4CHID"))
        .unwrap();
    close(
        bbox(a, "y") - bbox(a_heading, "y"),
        bbox(b, "y") - bbox(b_heading, "y"),
        "독립 Print에서 동일한 표제·표 앞 간격은 분할 정책과 무관하게 한 번 소비한다",
    );
}

#[test]
fn horizontal_fragments_share_one_centered_source_frame() {
    const NAME: &str =
        "samples/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp";
    const BODY: &str = "在炎热的夏季";
    let core = open(NAME);
    let outer = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .unwrap();
    let source = outer
        .cells
        .iter()
        .find(|cell| cell.paragraphs.iter().any(|para| para.text.contains(BODY)))
        .unwrap();
    assert_eq!(source.vertical_align, VerticalAlign::Center);
    let reset = source
        .paragraphs
        .windows(2)
        .position(|pair| {
            pair[1].line_segs.first().unwrap().vertical_pos == 0
                && pair[0].line_segs.last().unwrap().vertical_pos > 0
        })
        .expect("독립 Print의 다음 쪽으로 이어지는 저장 프레임")
        + 1;
    let last = &source.paragraphs[reset - 1].line_segs;
    assert!(last.windows(2).any(|pair| {
        pair[0].vertical_pos == pair[1].vertical_pos && pair[0].column_start != pair[1].column_start
    }));
    // 마지막 빈 좌우 조각도 한 줄 상자를 소유한다. 원본의 줄 끝을
    // 쓰며 조각 수만큼 더한 구현 높이나 현재 출력 좌표를 기대값으로 쓰지 않는다.
    let closing = last.last().unwrap();
    let source_height = px(closing.vertical_pos + closing.line_height);
    let first_body = source
        .paragraphs
        .iter()
        .find(|para| para.text.contains(BODY))
        .unwrap();
    let padding = source.effective_padding(&outer.padding);
    let tree = page_for_compat(NAME, 0, "2024");
    let cells: Vec<_> = nodes(&tree, "Cell")
        .into_iter()
        .filter(|cell| text(cell).contains(BODY))
        .collect();
    assert_eq!(cells.len(), 1, "본문은 한 셀 조각이 소유한다");
    let cell = cells[0];
    let origin = bbox(line(cell, BODY), "y") - px(first_body.line_segs[0].vertical_pos);
    let top_gap = origin - bbox(cell, "y") - px(padding.top);
    let bottom_gap = bottom(cell) - px(padding.bottom) - origin - source_height;
    assert!(
        top_gap >= -0.25 && bottom_gap >= -0.25,
        "원본 프레임의 셀 내부 포함"
    );
    close(
        top_gap,
        bottom_gap,
        "선택한 원본 프레임의 위·아래 여유는 같다",
    );
}

#[test]
fn repeated_hyphen_runs_remain_visible_and_owned_by_cells() {
    // 두 원본의 전쪽 Native/fresh WASM과 한컴 Print를 대조했다. 해당 쪽은
    // Native가 98% 미만이므로 좌표를 고정하지 않고 원문·소유·표시 관계를 검사한다.
    for (input, page) in [
        ("samples/task3307/issue3307_outline_number.hwpx", 7),
        (
            "samples/basic/issue2007_nested_cell_pagination_42065.hwp",
            2,
        ),
    ] {
        let tree = page_for_compat(input, page, "2022");
        let cells = nodes(&tree, "Cell");
        let runs: Vec<_> = nodes(&tree, "TextRun")
            .into_iter()
            .filter(|run| text(run).contains("---"))
            .collect();
        assert!(!runs.is_empty(), "연속 하이픈 원문 보존: {input}");
        for run in &runs {
            let center_x = bbox(run, "x") + bbox(run, "w") / 2.0;
            let center_y = bbox(run, "y") + bbox(run, "h") / 2.0;
            assert!(bbox(run, "w") > 0.0 && bbox(run, "h") > 0.0);
            assert!(
                cells.iter().any(|cell| {
                    center_x >= bbox(cell, "x")
                        && center_x <= bbox(cell, "x") + bbox(cell, "w")
                        && center_y >= bbox(cell, "y")
                        && center_y <= bottom(cell)
                }),
                "하이픈 글줄은 원본 표 셀이 소유한다: {input}"
            );
        }
        let source_dashes: usize = runs.iter().map(|run| text(run).matches('-').count()).sum();
        let tmp = ExportDir::new();
        let binary = std::env::var_os("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
        let output = Command::new(binary)
            .arg("export-svg")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(input))
            .args(["--profile", "print", "--compat", "2022", "-p"])
            .arg(page.to_string())
            .arg("-o")
            .arg(&tmp.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "SVG 출력 실패: {output:?}");
        let svg_path = std::fs::read_dir(&tmp.0)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|extension| extension == "svg"))
            .expect("선택한 인쇄 쪽 SVG");
        let svg = std::fs::read_to_string(svg_path).unwrap();
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let dashes: Vec<_> = doc
            .descendants()
            .filter(|node| {
                node.is_element() && node.tag_name().name() == "text" && node.text() == Some("-")
            })
            .collect();
        assert!(
            dashes.len() >= source_dashes,
            "원문의 하이픈을 누락하지 않는다: {input}"
        );
        for dash in dashes {
            if dash.attribute("fill-opacity") != Some("0") {
                continue;
            }
            let x = dash
                .attribute("x")
                .and_then(|value| value.parse::<f64>().ok());
            // prev_siblings는 현재 노드부터 시작하므로 실제 이전 형제부터 찾는다.
            let replacement = dash.prev_siblings().skip(1).find(|node| node.is_element());
            assert!(
                replacement.is_some_and(|line| {
                    let start = line
                        .attribute("x1")
                        .and_then(|value| value.parse::<f64>().ok());
                    let end = line
                        .attribute("x2")
                        .and_then(|value| value.parse::<f64>().ok());
                    line.tag_name().name() == "line"
                        && start == x
                        && matches!((start, end), (Some(a), Some(b)) if b > a)
                        && line.attribute("y1") == line.attribute("y2")
                }),
                "숨긴 하이픈은 같은 원점의 인쇄 선분이 대체한다: {input}"
            );
        }
    }
}
