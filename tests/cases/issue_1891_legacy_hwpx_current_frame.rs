//! 구판 자체 변환 HWPX의 현재 줄 문맥과 물리 조각 소유를 검증한다.
//!
//! 원본: samples/issue1891/80168_regulatory_analysis.hwpx.
//! 독립 기준: pdf/issue1891/80168_regulatory_analysis-hwpx-2020.pdf 전156쪽.
//! 원문 단위·본문/셀 포함 관계와 독립 Print의 줄·쪽 소속을 검사한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const INPUT: &str = "samples/issue1891/80168_regulatory_analysis.hwpx";

fn source() -> rhwp::model::document::Document {
    rhwp::parse_document(
        &std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(INPUT)).unwrap(),
    )
    .unwrap()
}

fn pages() -> &'static [Value] {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let out = std::env::temp_dir().join(format!(
            "rhwp-1891-current-frame-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let bin = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
        let run = Command::new(bin)
            .args(["export-render-tree", "--compat", "2022"])
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(INPUT))
            .arg("-o")
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("render_tree_")
            })
            .collect();
        files.sort();
        let result: Vec<Value> = files
            .iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect();
        std::fs::remove_dir_all(out).unwrap();
        assert_eq!(result.len(), 156, "독립 Print와 전체 쪽수가 같아야 한다");
        result
    })
}

fn nodes(root: &Value) -> Vec<&Value> {
    let mut out = vec![root];
    if let Some(children) = root["children"].as_array() {
        for child in children {
            out.extend(nodes(child));
        }
    }
    out
}
fn table(page: usize, pi: u64) -> &'static Value {
    nodes(&pages()[page - 1])
        .into_iter()
        .find(|n| n["type"] == "Table" && n["pi"] == pi)
        .expect("원본 표 소유자")
}
fn cell(table: &Value, row: u64, col: u64) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == row && n["col"] == col)
        .unwrap()
}
fn lines(owner: &Value, pi: u64) -> Vec<&Value> {
    owner["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "TextLine" && n["pi"] == pi)
        .collect()
}
fn text(node: &Value) -> String {
    nodes(node)
        .into_iter()
        .filter(|n| n["type"] == "TextRun")
        .map(|n| n["text"].as_str().unwrap())
        .collect()
}
fn normalized(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn y(n: &Value) -> f64 {
    n["bbox"]["y"].as_f64().unwrap()
}
fn h(n: &Value) -> f64 {
    n["bbox"]["h"].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    y(n) + h(n)
}
fn source_table(doc: &rhwp::model::document::Document, pi: usize) -> &rhwp::model::table::Table {
    doc.sections[0].paragraphs[pi]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .unwrap()
}

#[test]
fn previous_margin_cache_does_not_move_the_last_sentence_to_another_page() {
    let doc = source();
    let t = source_table(&doc, 15);
    let original = t.cells.iter().find(|c| c.row == 1 && c.col == 1).unwrap();
    assert_eq!(
        original.paragraphs[0].line_segs.len(),
        13,
        "원본 캐시는 보존한다"
    );
    let current = cell(table(4, 15), 1, 1);
    assert_eq!(
        lines(current, 0).len(),
        12,
        "독립 Print의 현재 문단 줄 구성"
    );
    assert_eq!(
        text(lines(current, 5).last().unwrap()),
        "군수등이 정하여 고시한다."
    );
    assert_eq!(
        normalized(&text(current)),
        normalized(
            &original
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>()
        ),
        "원문을 한 번씩 보존한다"
    );
}

#[test]
fn legacy_latin_without_a_true_type_substitute_keeps_its_source_dot_boundary() {
    let p = &pages()[29];
    let rows: Vec<_> = nodes(p)
        .into_iter()
        .filter(|n| n["type"] == "TextLine" && n["pi"] == 242)
        .collect();
    assert_eq!(rows.len(), 9, "독립 Print의 문단 줄 수");
    assert!(text(rows[2]).contains("시장·군수"));
    assert_eq!(text(rows[8]), "됨");
    assert_eq!(
        normalized(&rows.iter().map(|n| text(n)).collect::<String>()),
        normalized(&source().sections[0].paragraphs[242].text)
    );
}

#[test]
fn a_visible_tail_line_cannot_start_outside_its_body_page() {
    let mut emitted = String::new();
    for page in pages() {
        let all = nodes(page);
        let column = all.iter().find(|n| n["type"] == "Column").unwrap();
        for line in all
            .iter()
            .filter(|n| n["type"] == "TextLine" && n["pi"] == 896)
        {
            assert!(
                y(line) < bottom(column),
                "보이는 줄의 시작은 자기 본문 안이다"
            );
            emitted.push_str(&text(line));
        }
    }
    assert!(nodes(&pages()[101]).iter().any(|n| n["type"] == "TextLine"
        && n["pi"] == 896
        && text(n).contains("용할 필요가 있음")));
    assert_eq!(
        normalized(&emitted),
        normalized(&source().sections[0].paragraphs[896].text)
    );
}

#[test]
fn a_blank_cell_line_stays_with_the_closing_frame_and_the_next_title_restarts() {
    let mut owners = Vec::new();
    for (p, page) in pages().iter().enumerate() {
        for t in nodes(page)
            .into_iter()
            .filter(|n| n["type"] == "Table" && n["pi"] == 936)
        {
            let c = cell(t, 1, 1);
            for line in lines(c, 14) {
                owners.push(p + 1);
                assert!(text(line).is_empty());
            }
        }
    }
    assert_eq!(
        owners,
        [107],
        "한컴 재저장 문단14의 vpos65520은 앞 쪽 소유다"
    );
    let c = cell(table(108, 936), 1, 1);
    let first = c["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine")
        .unwrap();
    assert_eq!(first["pi"], 15, "다음 제목이 이어받기 첫 글줄이다");
}

#[test]
fn adjustable_single_row_uses_current_lines_and_source_inner_padding() {
    let doc = source();
    let t = source_table(&doc, 481);
    let src = &t.cells[0];
    let pad = src.effective_padding(&t.padding);
    assert_eq!(
        src.height,
        (pad.top + pad.bottom) as u32,
        "선언 최소는 안여백뿐이다"
    );
    let owner = cell(table(56, 481), 0, 0);
    let content: Vec<_> = owner["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "TextLine")
        .collect();
    assert!((y(content[0]) - y(owner) - f64::from(pad.top) / 75.0).abs() < 0.2);
    assert!(
        (bottom(owner) - bottom(content.last().unwrap()) - f64::from(pad.bottom) / 75.0).abs()
            < 0.2
    );
    assert!(
        h(owner) < f64::from(t.common.height) / 75.0,
        "옛 개체 높이를 현재 행의 추가 최소로 쓰지 않는다"
    );
}

#[test]
fn nested_current_height_and_following_empty_line_share_the_parent_frame() {
    let doc = source();
    let t = source_table(&doc, 911);
    let src = t.cells.iter().find(|c| c.row == 4 && c.col == 1).unwrap();
    let pad = src.effective_padding(&t.padding);
    let owner = cell(table(104, 911), 4, 1);
    let children = owner["children"].as_array().unwrap();
    let child = children.iter().find(|n| n["type"] == "Table").unwrap();
    let trailing = children
        .iter()
        .find(|n| n["type"] == "TextLine" && n["pi"] == 1)
        .unwrap();
    assert!((y(child) - y(owner) - f64::from(pad.top) / 75.0).abs() < 0.2);
    assert!(
        (bottom(child) - y(trailing)).abs() < 0.2,
        "후속 빈 줄은 자식 표 끝에서 전진한다"
    );
    assert!(
        (bottom(owner) - bottom(trailing) - f64::from(pad.bottom) / 75.0).abs() < 0.2,
        "부모가 옛 자식 높이를 다시 예약하지 않는다"
    );
}

#[test]
fn the_last_small_row_stays_before_the_nested_explanation_continuation() {
    let first = table(137, 1232);
    assert_eq!(normalized(&text(cell(first, 3, 0))), "일시적/반복적");
    let next = table(138, 1232);
    assert!(next["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "Cell")
        .all(|n| n["row"] == 4));
    assert_eq!(text(cell(next, 4, 0)), "근거설명");
}

/// 독립 한컴 Print의 벡터 테두리 중심선을 96 DPI CSS px로 환산한다.
/// 입력 SHA: 1e00b54e1139515258260a657217e0b3baa0b1b652a2cd28dd4b9bc48aa262fe.
/// PDF SHA: 88996e41ed3e68b15a3cd0eb4eda98845d34225c5dcd2ca547c80ab6e1242744.
/// 전156쪽 양쪽 raw90% 이상, 이 완료 쪽들은 양쪽98% 이상 및 직접 review/overlay 확인.
/// 기존 문단 소유·원문 보존·다음 쪽 이어받기 관계 검사는 함께 유지한다.
#[test]
fn completed_table_edges_follow_independent_print_within_two_css_pixels() {
    // p104 두 상자는 양쪽100%, p56은100/99.99745%, p101은99.99195/99.98924%.
    // 각 상자는 그 내용의 소유 문단으로 선택한다. 좌표로 개체를 고르지 않는다.
    let expected = [
        (
            104,
            911,
            "매도청구소송",
            [77.2387, 240.2174, 712.2573, 496.896],
        ),
        (104, 923, "재산권", [77.2387, 577.448, 712.2573, 834.128]),
        (56, 481, "제21조", [96.108, 392.8507, 731.2867, 623.6373]),
        (101, 890, "추진일정", [79.1573, 534.136, 714.176, 628.912]),
    ];
    for (page, paragraph, owner_text, edges) in expected {
        let owner = table(page, paragraph);
        assert!(
            normalized(&text(owner)).contains(owner_text),
            "p{page} 원본 문단{paragraph}의 내용 소유"
        );
        let bbox = &owner["bbox"];
        let left = bbox["x"].as_f64().unwrap();
        let top = bbox["y"].as_f64().unwrap();
        let actual = [
            left,
            top,
            left + bbox["w"].as_f64().unwrap(),
            top + bbox["h"].as_f64().unwrap(),
        ];
        for (edge, (observed, expected)) in actual.into_iter().zip(edges).enumerate() {
            assert!(
                (observed - expected).abs() <= 2.0,
                "p{page} 문단{paragraph} 테두리{edge}: 독립 Print {expected}px, 실제 {observed}px"
            );
        }
    }
}
