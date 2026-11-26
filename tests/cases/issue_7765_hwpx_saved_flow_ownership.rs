//! 관계형·고정형: 구판 HWPX의 저장 줄·빈 공간·중첩 표 컷과 후속 내용을 보호한다.
//! 주석 갱신: 2026-11-22T02:30:39+09:00; 새 파일이므로 직전 파일 변경 커밋은 없다.
//! 측정: 2026-11-22 KST, Print profile, compat2024, 96DPI, PDFium, 보정 전2px raw.
//! 원본 86712 HWPX 전64쪽: Native/fresh WASM 각각 최저98.69%, 쪽수도 독립 Print와 같다.
//! 11·12·21·25·26·27·28·29·53·64쪽은 양쪽100.00%; 52쪽은 Native99.98/WASM99.99%.
//! 26쪽 근거설명 첫 행의 독립 Print 높이44.91064px를 ±2px로 검사하고 소유·내용 관계도 유지한다.
//! 근거: mydocs/pr/assets/issues7754_7765/source32/README.md와 issue7765-hwpx/{native,wasm}/silhouette.tsv.
//! 실제 선택 프로그램 metadata를 명시적으로 공급한다. 글꼴 파일·시스템 설치·네트워크에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::{control::Control, table::Table},
    renderer::font_environment::FontEnvironment,
    DocumentCore,
};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path, sync::OnceLock};

const SOURCE: &str = "samples/issue1891/86712_regulatory_analysis.hwpx";

fn input() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).unwrap()
}

fn pages() -> &'static Vec<Value> {
    static PAGES: OnceLock<Vec<Value>> = OnceLock::new();
    PAGES.get_or_init(|| {
        let mut core = DocumentCore::from_bytes(&input()).unwrap();
        core.set_hangul2024_compat(true);
        let environment = FontEnvironment::from_json(include_str!(
            "../../mydocs/pr/assets/issues7754_7765/source32/regulatory-hwpx-font-environment.json"
        ))
        .unwrap();
        core.set_font_environment(Some(environment)).unwrap();
        assert_eq!(core.page_count(), 64, "독립 Print와 같은 전체 쪽수");
        (0..core.page_count())
            .map(|page| {
                serde_json::from_str(&core.build_page_render_tree(page).unwrap().root.to_json())
                    .unwrap()
            })
            .collect()
    })
}

fn children(node: &Value) -> impl Iterator<Item = &Value> {
    node["children"].as_array().into_iter().flatten()
}

fn ink(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &Value) -> String {
    let mut result = if node["type"] == "TextRun" {
        ink(node["text"].as_str().unwrap())
    } else {
        String::new()
    };
    for child in children(node) {
        result.push_str(&text(child));
    }
    result
}

fn body_nodes(node: &Value) -> Vec<&Value> {
    if node["type"] == "Column" {
        return children(node).collect();
    }
    children(node).flat_map(body_nodes).collect()
}

fn table(page: &Value, paragraph: usize) -> Option<&Value> {
    body_nodes(page)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == paragraph)
}

fn required_table(page: usize, paragraph: usize) -> &'static Value {
    table(&pages()[page - 1], paragraph).unwrap_or_else(|| panic!("{page}쪽의 원본 표 {paragraph}"))
}

fn line(page: usize, paragraph: usize) -> &'static Value {
    body_nodes(&pages()[page - 1])
        .into_iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == paragraph)
        .unwrap_or_else(|| panic!("{page}쪽의 본문 문단 {paragraph}"))
}

fn y(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}

fn height(node: &Value) -> f64 {
    node["bbox"]["h"].as_f64().unwrap()
}

fn before(first: &Value, next: &Value) {
    assert!(
        y(first) + height(first) <= y(next) + 0.1,
        "원본의 앞뒤 내용은 겹치지 않아야 한다: {:?} / {:?}",
        first["bbox"],
        next["bbox"]
    );
}

// 부모 문단과 자식 표는 별도 흐름 소유자다. 부모 전체 글 뒤에 자식 표를 합치지 않는다.
fn source_paragraphs(table: &Table, path: &str, out: &mut BTreeMap<String, String>) {
    for cell in &table.cells {
        let cell_path = format!("{path}/r{}c{}", cell.row, cell.col);
        for (pi, paragraph) in cell.paragraphs.iter().enumerate() {
            let para_path = format!("{cell_path}/p{pi}");
            assert!(out
                .insert(para_path.clone(), ink(&paragraph.text))
                .is_none());
            for (ci, control) in paragraph.controls.iter().enumerate() {
                if let Control::Table(child) = control {
                    source_paragraphs(child, &format!("{para_path}/c{ci}"), out);
                }
            }
        }
    }
}

fn own_text(node: &Value) -> String {
    if node["type"] == "Table" {
        return String::new();
    }
    let mut result = if node["type"] == "TextRun" {
        ink(node["text"].as_str().unwrap())
    } else {
        String::new()
    };
    for child in children(node) {
        result.push_str(&own_text(child));
    }
    result
}

fn rendered_paragraphs(table: &Value, path: &str, out: &mut BTreeMap<String, String>) {
    fn cell_flow(node: &Value, path: &str, out: &mut BTreeMap<String, String>) {
        if node["type"] == "Table" {
            let nested = format!(
                "{path}/p{}/c{}",
                node["pi"].as_u64().unwrap(),
                node["ci"].as_u64().unwrap()
            );
            rendered_paragraphs(node, &nested, out);
            return;
        }
        if node["type"] == "TextLine" {
            let pi = node["pi"].as_u64().expect("원본 셀 문단의 소유 번호");
            out.entry(format!("{path}/p{pi}"))
                .or_default()
                .push_str(&own_text(node));
            // TAC 자식 표도 자신의 셀·문단 경로로 검사한다.
            fn nested_tables(node: &Value, path: &str, out: &mut BTreeMap<String, String>) {
                for child in children(node) {
                    if child["type"] == "Table" {
                        cell_flow(child, path, out);
                    } else {
                        nested_tables(child, path, out);
                    }
                }
            }
            nested_tables(node, path, out);
            return;
        }
        for child in children(node) {
            cell_flow(child, path, out);
        }
    }
    for cell in children(table).filter(|node| node["type"] == "Cell") {
        let cell_path = format!(
            "{path}/r{}c{}",
            cell["row"].as_u64().unwrap(),
            cell["col"].as_u64().unwrap()
        );
        for child in children(cell) {
            cell_flow(child, &cell_path, out);
        }
    }
}

#[test]
fn saved_empty_prefix_and_tail_keep_their_real_page_owners() {
    let before_cut = required_table(11, 43);
    let after_cut = required_table(12, 43);
    assert!(!text(before_cut).is_empty() && !text(after_cut).is_empty());
    let heading = line(12, 45);
    let following = required_table(12, 47);
    before(after_cut, heading);
    before(heading, following);
    assert!(text(heading).contains("규제목표"));
    assert!(
        table(&pages()[12], 47).is_none(),
        "다음 표를 불필요하게 13쪽으로 밀지 않는다"
    );

    let completed = required_table(25, 161);
    let phrase = "각주민대표단별로약1년간운영하는것으로가정";
    assert_eq!(text(completed).matches(phrase).count(), 1);
    assert!(
        table(&pages()[25], 161).is_none(),
        "전체 표와 실제 빈 prefix는 앞쪽에서 소비했다"
    );
    before(required_table(26, 171), required_table(26, 172));

    before(required_table(21, 101), line(21, 102));
    assert_eq!(text(line(21, 102)), "3");
    before(required_table(28, 172), required_table(28, 182));
    before(required_table(53, 378), line(53, 380));
    before(line(53, 380), required_table(53, 382));
    assert!(text(required_table(64, 493)).contains("실질적으로발생하는추가비용은없다고판단됨"));
}

#[test]
fn first_nested_cut_reserves_blank_minimum_without_consuming_the_next_line() {
    let first = required_table(26, 172);
    let continued = required_table(27, 172);
    let row = children(first)
        .find(|node| node["type"] == "Cell" && node["row"] == 27 && node["col"] == 1)
        .expect("첫 조각의 근거설명 셀");
    // 독립 Print p26의 괘선 996.35067..1041.26131 CSSpx에서 읽은 높이.
    // 두 경로 raw100.00%와 직접 확대 판독 후 ±2px로 고정한다.
    let reference_height = 1041.26131 - 996.35067;
    assert!(
        (height(row) - reference_height).abs() <= 2.0,
        "첫 자식 컷은 글자 외에 선언 최소의 빈 물리 띠도 수용한다: {}",
        height(row)
    );
    let beneficiary = "(편익수혜자)주민대표단을구성하는정비사업토지등소유자";
    let kind = "(편익의종류)정비사업기간단축에따른사업비이자비용감소분";
    assert_eq!(text(first).matches(beneficiary).count(), 1);
    assert!(!text(continued).contains(beneficiary));
    assert!(!text(first).contains(kind));
    assert_eq!(text(continued).matches(kind).count(), 1);
    assert!(text(required_table(28, 172)).contains("2035년입주분"));
    assert!(text(required_table(29, 182)).contains("주택가격안정화가능"));
}

#[test]
fn every_cell_keeps_its_text_once_across_all_real_fragments() {
    let source = rhwp::parser::parse_document(&input()).unwrap();
    for paragraph in [43, 47, 101, 161, 172, 182, 378, 382, 493] {
        let original = source.sections[0].paragraphs[paragraph]
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Table(table) => Some(table.as_ref()),
                _ => None,
            })
            .expect("원본 표");
        let mut expected = BTreeMap::new();
        let mut rendered = BTreeMap::new();
        source_paragraphs(original, "root", &mut expected);
        for page in pages() {
            if let Some(fragment) = table(page, paragraph) {
                rendered_paragraphs(fragment, "root", &mut rendered);
            }
        }
        for (path, actual) in &rendered {
            assert!(
                expected.contains_key(path),
                "원본에 없는 소유 경로: 표 {paragraph}, {path}, {actual}"
            );
        }
        for (path, expected_text) in &expected {
            let actual = rendered.get(path).map(String::as_str).unwrap_or("");
            assert_eq!(
                actual, expected_text,
                "표 {paragraph}의 {path}: 원문 문단은 모든 조각에서 순서대로 한 번만 있어야 한다"
            );
        }
    }
}
