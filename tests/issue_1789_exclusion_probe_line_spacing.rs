//! #1789: 표 위 글줄은 표 위에 남고, 표 뒤 빈 줄도 공간을 점유한다.
//!
//! 독립 한컴2020 PDF는 2쪽이다. 위원구성은 첫쪽 표 위에 있고 회의내용과
//! 다섯 항목은 표 뒤에 있다. 행정사항·첨부 목록·결재 표는 둘째쪽에 있다.
//! 이전 절대 좌표 검사는 불필요한 3쪽과 표 뒤 빈 줄 소실을 발견하지 못했다.
//! 전2쪽 Native/fresh WASM 근거는 probe1789_origin_validation.json을 따른다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::path::Path;

const SAMPLE: &str = "samples/task1789/exclusion_probe_line_spacing.hwpx";

#[test]
fn issue_1789_line_above_para_float_table_stays_at_saved_vpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        2,
        "정본의 회의계획과 행정사항은 각각 한 쪽이다"
    );
    let mut pages = Vec::new();
    let mut tables = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
        let json: Value = serde_json::from_str(&tree.root.to_json()).expect("렌더 트리 JSON");
        let mut lines = Vec::new();
        let mut boxes = Vec::new();
        collect(&json, &mut lines, &mut boxes);
        pages.push(lines);
        tables.push(boxes);
    }
    let first = &pages[0];
    let line = |pi| {
        first
            .iter()
            .find(|(owner, _, _, _)| *owner == pi)
            .unwrap_or_else(|| panic!("첫쪽 본문 문단 {pi} 누락"))
    };
    let table = tables[0]
        .iter()
        .find(|(pi, _, _)| *pi == 5)
        .expect("첫쪽 위원 표");
    assert!(line(8).2 <= table.1, "위원구성 글줄은 표 위 공간에 남는다");
    assert!(table.2 <= line(9).1, "빈 줄의 상자는 표 뒤에 있어야 한다");
    assert!(
        line(9).2 <= line(10).1,
        "회의내용 앞 빈 줄의 높이를 소비한다"
    );
    for pi in 10..=15 {
        assert_eq!(
            first.iter().filter(|(owner, _, _, _)| *owner == pi).count(),
            1
        );
        assert!(
            !pages[1].iter().any(|(owner, _, _, _)| *owner == pi),
            "회의 항목은 둘째쪽으로 밀리지 않는다"
        );
        if pi > 10 {
            assert!(
                line(pi - 1).2 <= line(pi).1,
                "회의 항목 순서와 비겹침을 보존한다"
            );
        }
    }
    for pi in 17..=27 {
        assert!(!first.iter().any(|(owner, _, _, _)| *owner == pi));
        assert_eq!(
            pages[1]
                .iter()
                .filter(|(owner, _, _, _)| *owner == pi)
                .count(),
            1,
            "행정사항과 첨부는 둘째쪽에서 한 번 그린다"
        );
    }
    assert!(
        tables[1].iter().any(|(pi, _, _)| *pi == 28),
        "결재 표는 둘째쪽에 남는다"
    );
    for (pi, paragraph) in core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .take(28)
    {
        let painted: String = pages
            .iter()
            .flatten()
            .filter(|(owner, _, _, _)| *owner == pi)
            .map(|(_, _, _, text)| text.as_str())
            .collect();
        let visible = |text: &str| {
            text.chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
        };
        assert_eq!(
            visible(&painted),
            visible(&paragraph.text),
            "문단 {pi} 본문 누락·중복 없이 원문을 보존한다"
        );
    }
}

fn collect(
    node: &Value,
    lines: &mut Vec<(usize, f64, f64, String)>,
    tables: &mut Vec<(usize, f64, f64)>,
) {
    let kind = node["type"].as_str().unwrap_or("");
    if matches!(kind, "Table" | "Rect" | "TextBox" | "Header" | "Footer") {
        if kind == "Table" {
            if let (Some(pi), Some(y), Some(h)) = (
                node["pi"].as_u64(),
                node["bbox"]["y"].as_f64(),
                node["bbox"]["h"].as_f64(),
            ) {
                tables.push((pi as usize, y, y + h));
            }
        }
        return;
    }
    if kind == "TextLine" {
        if let (Some(pi), Some(y), Some(h)) = (
            node["pi"].as_u64(),
            node["bbox"]["y"].as_f64(),
            node["bbox"]["h"].as_f64(),
        ) {
            let text = node["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|child| child["text"].as_str())
                .collect();
            lines.push((pi as usize, y, y + h, text));
        }
        return;
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, lines, tables);
    }
}
