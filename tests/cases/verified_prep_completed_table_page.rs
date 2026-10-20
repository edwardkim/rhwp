//! PrEP 원본의 완료된 70쪽을 표 이어받기·전체 셀·후속 본문으로 보호한다.
//! 전 140쪽 Native/fresh WASM 최저 91.54965/92.43307%, 쪽수 일치.
//! 70쪽은 양쪽 100%이며 독립 Print와 확대 판독을 완료했다.
//! 96dpi 좌상단 좌표와 증적: mydocs/working/task_m100_7642_stage1577.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{control::Control, table::Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

// 공통 판정 시작: 과거 실제 출력과 현재 출력에 같은 함수를 적용한다.
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
        .collect()
}

fn text(node: &Value) -> String {
    let mut result = node["text"].as_str().unwrap_or_default().to_owned();
    for child in node["children"].as_array().into_iter().flatten() {
        result.push_str(&text(child));
    }
    result
}

fn nodes<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        nodes(child, kind, out);
    }
}

fn body_lines<'a>(node: &'a Value, paragraph: u64, out: &mut Vec<&'a Value>) {
    if matches!(node["type"].as_str(), Some("Table" | "Cell")) {
        return;
    }
    if node["type"] == "TextLine" && node["pi"].as_u64() == Some(paragraph) {
        out.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        body_lines(child, paragraph, out);
    }
}

fn number(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("실제 출력 상자")
}

fn bottom(node: &Value) -> f64 {
    number(node, "y") + number(node, "h")
}

fn table<'a>(page: &'a Value, source: &Value) -> &'a Value {
    let mut all = Vec::new();
    nodes(page, "Table", &mut all);
    let found: Vec<_> = all
        .into_iter()
        .filter(|n| {
            n["pi"] == source["pi"]
                && n["ci"] == source["ci"]
                && n["rows"] == source["rows"]
                && n["cols"] == source["cols"]
        })
        .collect();
    assert_eq!(found.len(), 1, "고유 원본 표의 쪽 소유·누락·중복");
    found[0]
}

fn cells(node: &Value) -> Vec<&Value> {
    node["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "Cell")
        .collect()
}

fn address(cell: &Value) -> (u64, u64) {
    (cell["row"].as_u64().unwrap(), cell["col"].as_u64().unwrap())
}

fn check_cells(node: &Value, source: &Value, expected_rows: BTreeSet<u64>) {
    let expected: BTreeMap<_, _> = source["cells"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| expected_rows.contains(&c["row"].as_u64().unwrap()))
        .map(|c| (address(c), c["text"].as_str().unwrap()))
        .collect();
    let rendered = cells(node);
    let rows: BTreeSet<_> = rendered.iter().map(|c| address(c).0).collect();
    assert_eq!(rows, expected_rows, "독립 Print의 실제 행 소유");
    assert_eq!(
        rendered.len(),
        expected.len(),
        "행 안의 빈 셀도 누락하지 않음"
    );
    let mut seen = BTreeSet::new();
    let mut previous_row_bottom = number(node, "y");
    for row in &expected_rows {
        let mut owned: Vec<_> = rendered
            .iter()
            .copied()
            .filter(|c| address(c).0 == *row)
            .collect();
        owned.sort_by_key(|c| address(c).1);
        let top = owned
            .iter()
            .map(|c| number(c, "y"))
            .fold(f64::INFINITY, f64::min);
        assert!(top >= previous_row_bottom - 0.2, "인접 행의 순서·겹침");
        previous_row_bottom = owned.iter().map(|c| bottom(c)).fold(0.0, f64::max);
        for pair in owned.windows(2) {
            assert!(
                number(pair[1], "x") >= number(pair[0], "x") + number(pair[0], "w") - 0.2,
                "원본 열 소유에 따른 좌우 순서·겹침"
            );
        }
    }
    for cell in rendered {
        let owner = address(cell);
        assert!(seen.insert(owner), "중복 셀: {owner:?}");
        assert_eq!(
            compact(&text(cell)),
            *expected.get(&owner).expect("원본에 존재하는 셀"),
            "셀의 전체 내용·순서 보존: {owner:?}"
        );
        // debug JSON은 소수 첫째 자리로 반올림한다. 양 끝의 반올림 오차만 허용한다.
        let roundoff = 0.2;
        assert!(number(cell, "x") + roundoff >= number(node, "x"));
        assert!(number(cell, "y") + roundoff >= number(node, "y"));
        assert!(
            number(cell, "x") + number(cell, "w")
                <= number(node, "x") + number(node, "w") + roundoff
        );
        assert!(bottom(cell) <= bottom(node) + roundoff);
        let mut runs = Vec::new();
        nodes(cell, "TextRun", &mut runs);
        for run in runs.into_iter().filter(|r| !compact(&text(r)).is_empty()) {
            assert!(
                number(run, "x") + roundoff >= number(cell, "x"),
                "셀의 글자 시작"
            );
            assert!(
                number(run, "y") + roundoff >= number(cell, "y"),
                "셀의 글자 위쪽"
            );
            assert!(
                number(run, "x") + number(run, "w")
                    <= number(cell, "x") + number(cell, "w") + roundoff,
                "셀 밖으로 넘지 않는 전체 글자"
            );
            assert!(bottom(run) <= bottom(cell) + roundoff, "셀의 글자 아래쪽");
        }
    }
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {label} ±2 CSS px: {actual} / {expected}"
    );
}

fn completed_row_borders(node: &Value, rows: &[u64], borders: &[f64]) {
    assert_eq!(rows.len() + 1, borders.len());
    near(number(node, "x"), 75.0, "표 왼쪽 괘선");
    near(
        number(node, "x") + number(node, "w"),
        718.65332,
        "표 오른쪽 괘선",
    );
    near(number(node, "y"), borders[0], "표 시작");
    near(bottom(node), *borders.last().unwrap(), "표 종료");
    for (i, row) in rows.iter().enumerate() {
        let owned: Vec<_> = cells(node)
            .into_iter()
            .filter(|c| c["row"].as_u64() == Some(*row))
            .collect();
        assert!(!owned.is_empty(), "독립 Print 행의 셀: {row}");
        for cell in owned {
            near(number(cell, "y"), borders[i], "소유 행 시작");
            near(bottom(cell), borders[i + 1], "소유 행 종료");
        }
    }
}

fn check_completed_page(previous: &Value, page: &Value, source: &Value) {
    let first = table(previous, &source["table27"]);
    let tail = table(page, &source["table27"]);
    let second = table(page, &source["table28"]);
    check_cells(first, &source["table27"], (0..=32).collect());
    check_cells(
        tail,
        &source["table27"],
        [0, 33, 34, 35, 36, 37].into_iter().collect(),
    );
    check_cells(second, &source["table28"], (0..=7).collect());

    let mut caption = Vec::new();
    body_lines(
        previous,
        source["caption27"]["pi"].as_u64().unwrap(),
        &mut caption,
    );
    assert_eq!(caption.len(), 1, "표 27 캡션의 쪽 소유");
    assert_eq!(compact(&text(caption[0])), source["caption27"]["text"]);
    assert!(
        bottom(caption[0]) <= number(first, "y") + 0.2,
        "캡션 뒤에 표 시작"
    );

    let mut last_bottom = bottom(tail);
    for paragraph in source["body"].as_array().unwrap() {
        let mut lines = Vec::new();
        body_lines(page, paragraph["pi"].as_u64().unwrap(), &mut lines);
        assert!(!lines.is_empty(), "표 사이와 뒤의 본문 소유: {paragraph}");
        let actual: String = lines.iter().map(|line| compact(&text(line))).collect();
        assert_eq!(actual, paragraph["text"], "본문의 전체 내용·순서 보존");
        assert!(number(lines[0], "y") >= last_bottom - 0.2, "본문 순서·겹침");
        for pair in lines.windows(2) {
            assert!(
                number(pair[1], "y") >= bottom(pair[0]) - 0.2,
                "문단 내부 줄 순서"
            );
        }
        last_bottom = bottom(lines.last().unwrap());
        let pi = paragraph["pi"].as_u64().unwrap();
        if pi + 1 == source["table28"]["pi"].as_u64().unwrap() {
            assert!(
                last_bottom <= number(second, "y"),
                "표 28 캡션 뒤에 표 시작"
            );
            last_bottom = bottom(second);
        }
    }

    // 실제 독립 PDF 70쪽의 가로 괘선이다. 다른 페이지에는 좌표 핀을 추가하지 않는다.
    completed_row_borders(
        tail,
        &[0, 33, 34, 35, 36, 37],
        &[
            94.297363, 138.888021, 163.501302, 188.274658, 212.886637, 237.5, 262.113363,
        ],
    );
    completed_row_borders(
        second,
        &[0, 1, 2, 3, 4, 5, 6, 7],
        &[
            551.715983, 576.329346, 600.942668, 625.395996, 650.009318, 674.621338, 699.234701,
            723.847982, 748.461344,
        ],
    );
}
// 공통 판정 끝

fn source_table(core: &DocumentCore, rows: u16, cols: u16) -> Value {
    let mut found = Vec::new();
    for (section, s) in core.document().sections.iter().enumerate() {
        for (paragraph, p) in s.paragraphs.iter().enumerate() {
            for (control, c) in p.controls.iter().enumerate() {
                let Control::Table(t) = c else { continue };
                if (t.row_count, t.col_count) != (rows, cols) {
                    continue;
                }
                let value = table_spec(t, section, paragraph, control);
                let all = value["cells"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c["text"].as_str().unwrap())
                    .collect::<String>();
                if all.contains("10,452") && all.contains("105,534") {
                    found.push(value);
                }
            }
        }
    }
    assert_eq!(found.len(), 1, "내용과 원본 구조로 고유 표 식별");
    found.remove(0)
}

fn table_spec(t: &Table, section: usize, paragraph: usize, control: usize) -> Value {
    let cells: Vec<_> = t
        .cells
        .iter()
        .map(|c| {
            assert_eq!(c.row_span, 1, "독립 Print에서 확인한 한 행 단위 셀");
            let text: String = c.paragraphs.iter().map(|p| compact(&p.text)).collect();
            json!({"row":c.row,"col":c.col,"text":text})
        })
        .collect();
    json!({"section":section,"pi":paragraph,"ci":control,"rows":t.row_count,"cols":t.col_count,"cells":cells})
}

fn check_table_sections(node: &RenderNode, source: &Value) {
    if let RenderNodeType::Table(t) = &node.node_type {
        for name in ["table27", "table28"] {
            let owner = &source[name];
            if t.para_index == owner["pi"].as_u64().map(|n| n as usize)
                && t.control_index == owner["ci"].as_u64().map(|n| n as usize)
                && t.row_count as u64 == owner["rows"].as_u64().unwrap()
                && t.col_count as u64 == owner["cols"].as_u64().unwrap()
            {
                assert_eq!(
                    t.section_index,
                    owner["section"].as_u64().map(|n| n as usize)
                );
                assert!(t.cell_context.is_none(), "본문 표의 원본 소유");
            }
        }
    }
    for child in &node.children {
        check_table_sections(child, source);
    }
}

#[test]
fn prep_completed_page_preserves_continuation_cells_and_print_boundaries() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 140, "독립 원본 Print 전체 쪽수");
    let first = source_table(&core, 38, 4);
    let second = source_table(&core, 8, 6);
    assert_eq!(first["section"], second["section"]);
    let section = first["section"].as_u64().unwrap() as usize;
    let first_pi = first["pi"].as_u64().unwrap() as usize;
    let second_pi = second["pi"].as_u64().unwrap() as usize;
    let ps = &core.document().sections[section].paragraphs;
    let body: Vec<_> = (first_pi + 1..=second_pi + 5)
        .filter_map(|pi| {
            if pi == second_pi || compact(&ps[pi].text).is_empty() {
                return None;
            }
            Some(json!({"pi":pi,"text":compact(&ps[pi].text)}))
        })
        .collect();
    assert_eq!(body.len(), 10, "두 표 사이와 뒤의 본문 문단");
    let spec = json!({"table27":first,"table28":second,"body":body,
        "caption27":{"pi":first_pi-1,"text":compact(&ps[first_pi-1].text)}});
    let previous = core.build_page_render_tree(68).unwrap();
    let page = core.build_page_render_tree(69).unwrap();
    check_table_sections(&previous.root, &spec);
    check_table_sections(&page.root, &spec);
    let previous: Value = serde_json::from_str(&previous.root.to_json()).unwrap();
    let page: Value = serde_json::from_str(&page.root.to_json()).unwrap();
    check_completed_page(&previous, &page, &spec);
}
