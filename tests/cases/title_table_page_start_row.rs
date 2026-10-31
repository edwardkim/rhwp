//! 관계형: 제목 표·선행 빈 줄·후속 TAC의 소유, 순서, 내용과 저장 줄 간격.
//! 측정 2026-10-30, 96 DPI, 색상 경계 보정 전 2px silhouette_raw_match_percent.
//! 원본 title-table-then-inline-table.hwpx: 1쪽, Native 94.77782%, fresh WASM 94.57889%.
//! 원본 title-table-after-empty-paragraph.hwp: 1쪽, Native 94.76313%, fresh WASM 94.55817%.
//! 원본 title-table-after-empty-paragraph.hwpx: 1쪽, Native 94.76313%, fresh WASM 94.55817%.
//! 각 문서는 전체 1쪽이므로 대상 쪽 수치와 전쪽 최저값이 같다. 원 PR의 독립 2024 PDF 사용.
//! nine-leading-empty.hwp: 1쪽, Native/최저 98.84715%, fresh WASM/최저 98.80599%.
//! nine-leading-empty-page-break.hwp: 2쪽, p1 양쪽 100%, p2/전쪽 최저 Native 98.86738%, WASM 98.82916%.
//! 실제 편집: 후속 표 문단 pi4 / 선행 빈 문단 pi0는 각 1쪽, Native 98.85%, WASM 98.81%.
//! 같은 host pi0 편집: 1쪽, Native 98.72%, WASM 98.69%. 각각 전쪽 최저값이며 동일 저장본 Print 사용.
//! 원본 same-host: 1쪽, Native 98.84484%, WASM 98.80408%.
//! source 21ba44e3ea; 기준 PDF pdf/title_page_start/{원본,maintainer/}; 전체 TSV·PNG·글꼴·명령:
//! mydocs/pr/assets/pr_8144/README.md.
//!
//! 쪽 위를 쪽·종이 기준 어울림 제목 표가 차지하면 한글은 그 쪽의 첫 줄을 제목 표 바깥
//! 상자 아래에서 시작하고, 저장 첫 줄 vpos 가 그 상자 바닥과 같다. 그 쪽은 저장 위치를
//! 그대로 쓴다: 쪽 첫 문단의 줄은 상자 바닥에, 뒤 문단의 글자처럼 취급 표는 그 저장
//! 위치에 놓인다.
//!
//! 고치기 전에는 쪽 첫 vpos 를 쪽 원점으로 빼서 첫 줄을 본문 위끝에 두었고, 다음 문단의
//! 표가 제목 표를 덮었다(85px). 표본은 `paper-square-table-tac-host.hwpx` 의 글자처럼 취급
//! 표를 다음 문단으로 옮겨 한글 2024 로 다시 저장한 HWPX 다.
//!
//! 제목 표 문단 앞에 빈 문단(구역 정의만 든 문단)이 있어도 같다. 한글은 그 빈 문단에도
//! 줄을 주고(저장 사다리가 그 문단을 지나며 전진한다) 제목 표 문단, 글자처럼 취급 표 문단이
//! 차례로 그 아래에 놓인다. 이 경계는 같은 표본에 빈 문단을 넣고 한글 2024 가 HWPX·HWP 로
//! 각각 다시 저장한 두 표본으로 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/title-table-then-inline-table.hwpx";
const LEADING_EMPTY_HWPX: &str = "samples/title-table-after-empty-paragraph.hwpx";
const LEADING_EMPTY_HWP: &str = "samples/title-table-after-empty-paragraph.hwp";

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 쪽 render tree 의 (종류, 문단, 컨트롤, 위끝, 아래끝). 표 안 글줄은 뺀다.
fn boxes(doc: &HwpDocument, page: u32) -> Vec<(String, u64, Option<u64>, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(String, u64, Option<u64>, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if matches!(kind, "Table" | "TextLine") {
            if let Some(pi) = node["pi"].as_u64() {
                let y = node["bbox"]["y"].as_f64().unwrap();
                let bottom = y + node["bbox"]["h"].as_f64().unwrap();
                out.push((kind.to_string(), pi, node["ci"].as_u64(), y, bottom));
            }
        }
        if kind != "Table" {
            for child in node["children"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
    }
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

#[test]
fn page_opened_below_title_table_keeps_stored_positions() {
    assert_title_page_keeps_stored_positions(SAMPLE);
}

#[test]
fn empty_paragraph_before_title_table_keeps_its_line_in_hwpx() {
    assert_title_page_keeps_stored_positions(LEADING_EMPTY_HWPX);
}

#[test]
fn empty_paragraph_before_title_table_keeps_its_line_in_hwp() {
    assert_title_page_keeps_stored_positions(LEADING_EMPTY_HWP);
}

fn assert_title_page_keeps_stored_positions(sample: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.set_hangul2024_compat(true);
    assert_eq!(doc.page_count(), 1, "{sample}: 독립 한글 출력과 같은 쪽 수");

    assert_table_cells_preserved(&doc);
    let paragraphs = &doc.document().sections[0].paragraphs;
    let (host, square, square_outer_bottom) = paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, para)| {
            para.controls
                .iter()
                .enumerate()
                .find_map(|(index, control)| match control {
                    Control::Table(table) if !table.common.treat_as_char => {
                        Some((pi, index as u64, table.outer_margin_bottom))
                    }
                    _ => None,
                })
        })
        .expect("쪽·종이 기준 어울림 제목 표");
    let (inline_host, inline) = paragraphs
        .iter()
        .enumerate()
        .skip(host + 1)
        .find_map(|(pi, para)| {
            para.controls
                .iter()
                .position(|control| {
                    matches!(control, Control::Table(table) if table.common.treat_as_char)
                })
                .map(|index| (pi, index as u64))
        })
        .expect("제목 표 뒤 문단의 글자처럼 취급 표");
    // 쪽 첫 문단의 저장 줄은 제목 표 바깥 상자 바닥에 있다. 글자처럼 취급 표는 그 줄부터
    // 저장 사다리가 전진한 만큼 아래에 있다.
    let advance =
        paragraphs[inline_host].line_segs[0].vertical_pos - paragraphs[0].line_segs[0].vertical_pos;
    assert!(
        advance > 0,
        "{sample}: 글자처럼 취급 표의 저장 위치가 쪽 첫 줄 아래다"
    );

    let boxes = boxes(&doc, 0);
    let table = |pi: u64, ci: u64| {
        boxes
            .iter()
            .find(|(kind, p, c, _, _)| kind == "Table" && *p == pi && *c == Some(ci))
            .map(|(_, _, _, top, bottom)| (*top, *bottom))
            .unwrap_or_else(|| panic!("{sample}: 문단 {pi} 컨트롤 {ci} 표가 1쪽에 없다"))
    };
    let (_, square_bottom) = table(host as u64, square);
    let (inline_top, inline_bottom) = table(inline_host as u64, inline);
    let floor = square_bottom + hu_to_px(i32::from(square_outer_bottom));
    let expected = floor + hu_to_px(advance);
    assert!(
        (inline_top - expected).abs() <= 0.5,
        "{sample}: 글자처럼 취급 표({inline_top:.1})가 저장 위치({expected:.1}, 제목 표 바깥 상자 {floor:.1} + 저장 전진)에 있지 않다"
    );

    // 그 뒤 문단의 표와 글줄은 모두 그 표 아래에서 시작하고 서로 겹치지 않는다.
    let mut successors: Vec<_> = boxes
        .iter()
        .filter(|(_, pi, _, _, _)| *pi > inline_host as u64)
        .map(|(kind, pi, _, top, bottom)| (kind.as_str(), *pi, *top, *bottom))
        .collect();
    assert!(!successors.is_empty(), "{sample}: 뒤 문단이 1쪽에 있다");
    for (kind, pi, top, _) in &successors {
        assert!(
            *top >= inline_bottom - 0.5,
            "{sample}: 문단 {pi} {kind}({top:.1})가 표 아래끝({inline_bottom:.1}) 위로 올라왔다"
        );
    }
    successors.sort_by(|a, b| a.2.total_cmp(&b.2));
    for pair in successors.windows(2) {
        let (kind_a, pi_a, _, bottom_a) = pair[0];
        let (kind_b, pi_b, top_b, _) = pair[1];
        assert!(
            top_b >= bottom_a - 0.5,
            "{sample}: 문단 {pi_a} {kind_a} 와 문단 {pi_b} {kind_b} 가 겹친다"
        );
    }
}

#[test]
fn all_authored_empty_lines_before_title_survive_beyond_eight_paragraphs() {
    assert_title_page_keeps_stored_positions("tests/fixtures/pr8144/nine-leading-empty.hwp");
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|ch| {
            !ch.is_whitespace() && !ch.is_control() && !matches!(ch, '\u{FFFC}' | '\u{F081C}' | '•')
        })
        .collect()
}

fn text_of(node: &Value) -> String {
    let mut text = if node["type"] == "TextRun" {
        node["text"].as_str().unwrap_or_default().to_owned()
    } else {
        String::new()
    };
    for child in node["children"].as_array().into_iter().flatten() {
        text.push_str(&text_of(child));
    }
    text
}

/// 전쪽의 표 소유자·셀 원문을 대조해 쪽수만으로 누락·중복을 놓치지 않는다.
fn assert_table_cells_preserved(doc: &HwpDocument) {
    fn tables<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
        if node["type"] == "Table" {
            out.push(node);
            return;
        }
        for child in node["children"].as_array().into_iter().flatten() {
            tables(child, out);
        }
    }
    let pages: Vec<Value> = (0..doc.page_count())
        .map(|page| serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap())
        .collect();
    let mut actual = Vec::new();
    for page in &pages {
        tables(page, &mut actual);
    }
    let mut expected = 0;
    for (pi, para) in doc.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, control) in para.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            expected += 1;
            let owners: Vec<_> = actual
                .iter()
                .filter(|n| n["pi"] == pi && n["ci"] == ci)
                .collect();
            assert_eq!(owners.len(), 1, "문단 {pi} 표 {ci}의 누락 또는 중복");
            let cells: Vec<_> = owners[0]["children"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|node| node["type"] == "Cell")
                .collect();
            assert_eq!(
                cells.len(),
                table.cells.len(),
                "문단 {pi} 표 {ci}의 셀 보존"
            );
            for cell in &table.cells {
                let nodes: Vec<_> = cells
                    .iter()
                    .filter(|n| n["row"] == cell.row as u64 && n["col"] == cell.col as u64)
                    .collect();
                assert_eq!(nodes.len(), 1, "셀의 행·열 소유를 보존한다");
                let source: String = cell
                    .paragraphs
                    .iter()
                    .map(|p| normalized(&p.text))
                    .collect();
                assert_eq!(
                    normalized(&text_of(nodes[0])),
                    source,
                    "셀의 글자 순서·누락·중복"
                );
            }
        }
    }
    assert_eq!(
        actual.len(),
        expected,
        "원본의 모든 표가 정확히 한 번 배치된다"
    );
}

#[test]
fn explicit_page_break_keeps_blank_prefix_and_square_successor_on_separate_pages() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr8144/nine-leading-empty-page-break.hwp");
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.set_hangul2024_compat(true);
    assert_eq!(doc.page_count(), 2, "독립 Print의 빈 첫 쪽과 본문 둘째 쪽");
    assert_table_cells_preserved(&doc);
    let paras = &doc.document().sections[0].paragraphs;
    let host = paras
        .iter()
        .position(|p| p.column_type == rhwp::model::paragraph::ColumnBreakType::Page)
        .expect("원본의 명시적 쪽 나눔");
    let first = boxes(&doc, 0);
    assert!(first
        .iter()
        .all(|(kind, pi, _, _, _)| kind != "Table" && (*pi as usize) < host));
    for pi in 0..host {
        assert!(
            first
                .iter()
                .any(|(kind, owner, _, _, _)| kind == "TextLine" && *owner == pi as u64),
            "빈 문단 {pi}도 첫 쪽에서 실제 줄을 소유한다"
        );
    }
    let second = boxes(&doc, 1);
    let square = second
        .iter()
        .find(|(kind, pi, _, _, _)| kind == "Table" && *pi == host as u64)
        .unwrap();
    let inline = second
        .iter()
        .find(|(kind, pi, _, _, _)| kind == "Table" && *pi == (host + 1) as u64)
        .unwrap();
    let margin = paras[host]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) if !t.common.treat_as_char => Some(t.outer_margin_bottom),
            _ => None,
        })
        .unwrap();
    assert!(
        inline.3 >= square.4 + hu_to_px(i32::from(margin)) - 0.5,
        "옆 차선의 앵커 줄 뒤 TAC는 제목 표 바깥 아래에 놓인다"
    );
    for (_, pi, _, top, _) in second
        .iter()
        .filter(|(_, pi, _, _, _)| *pi > (host + 1) as u64)
    {
        assert!(*top >= inline.4 - 0.5, "뒤 문단 {pi}가 TAC 아래에 이어진다");
    }
}

fn assert_edited_title_relationships(doc: &HwpDocument) {
    assert_eq!(doc.page_count(), 1, "편집 저장본의 독립 Print는 1쪽이다");
    assert_table_cells_preserved(doc);
    let rendered = boxes(doc, 0);
    let mut square = None;
    let mut inline = None;
    for (pi, para) in doc.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, control) in para.controls.iter().enumerate() {
            if let Control::Table(table) = control {
                let node = rendered
                    .iter()
                    .find(|(kind, p, c, _, _)| {
                        kind == "Table" && *p == pi as u64 && *c == Some(ci as u64)
                    })
                    .unwrap();
                if !table.common.treat_as_char {
                    square = Some(node.4 + hu_to_px(i32::from(table.outer_margin_bottom)));
                } else if inline.is_none() {
                    inline = Some((pi, node.3, node.4));
                }
            }
        }
    }
    let (host, top, bottom) = inline.unwrap();
    assert!(
        top >= square.unwrap() - 0.5,
        "편집 뒤 TAC가 제목 표 바깥 상자를 침범하지 않는다"
    );
    for (_, pi, _, top, _) in rendered.iter().filter(|(_, pi, _, _, _)| *pi > host as u64) {
        assert!(
            *top >= bottom - 0.5,
            "편집 뒤 문단 {pi}가 TAC 아래에 이어진다"
        );
    }
}

fn assert_edit_reopen_undo(sample: &str, paragraph: usize) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.set_hangul2024_compat(true);
    let before = doc.get_page_render_tree(0).unwrap();
    let snapshot = doc.save_snapshot_native();
    doc.insert_text_native(0, paragraph, 0, "검증 ").unwrap();
    assert!(doc.document().sections[0].paragraphs[paragraph]
        .text
        .starts_with("검증 "));
    assert_edited_title_relationships(&doc);
    let saved = doc.export_hwpx_native().unwrap();
    let mut reopened = HwpDocument::from_bytes(&saved).unwrap();
    reopened.set_hangul2024_compat(true);
    assert_eq!(
        reopened.document().sections[0].paragraphs[paragraph].text,
        doc.document().sections[0].paragraphs[paragraph].text
    );
    assert_edited_title_relationships(&reopened);
    doc.restore_snapshot_native(snapshot).unwrap();
    assert_eq!(
        doc.get_page_render_tree(0).unwrap(),
        before,
        "되돌리기는 원래 소유·내용·배치를 복원한다"
    );
}

#[test]
fn editing_successor_table_host_preserves_title_reopen_and_undo() {
    assert_edit_reopen_undo(LEADING_EMPTY_HWPX, 4);
}

#[test]
fn editing_leading_blank_preserves_title_reopen_and_undo() {
    assert_edit_reopen_undo(LEADING_EMPTY_HWPX, 0);
}

#[test]
fn editing_same_square_and_inline_host_preserves_reopen_and_undo() {
    assert_edit_reopen_undo("samples/paper-square-table-tac-host.hwpx", 0);
}

#[test]
fn no_color_note_separator_roundtrips_without_losing_reserved_note_space() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/paper-square-table-tac-host.hwpx");
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    doc.set_hangul2024_compat(true);
    assert_eq!(
        doc.document().sections[0]
            .section_def
            .footnote_shape
            .separator_color,
        rhwp::model::color::NONE
    );
    let saved = doc.export_hwpx_native().unwrap();
    let mut reopened = HwpDocument::from_bytes(&saved).unwrap();
    reopened.set_hangul2024_compat(true);
    assert_eq!(
        reopened.document().sections[0]
            .section_def
            .footnote_shape
            .separator_color,
        rhwp::model::color::NONE
    );
    fn note(node: &Value) -> Option<&Value> {
        if node["type"] == "FootnoteArea" {
            return Some(node);
        }
        node["children"].as_array()?.iter().find_map(note)
    }
    let original: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let original_note = note(&original).unwrap();
    let children = original_note["children"].as_array().unwrap();
    assert!(
        children.iter().all(|n| n["type"] != "Line"),
        "명시적 색 없음에는 구분선 잉크가 없다"
    );
    assert!(
        !text_of(original_note).is_empty(),
        "각주 내용은 숨기지 않는다"
    );
    let roundtrip: Value =
        serde_json::from_str(&reopened.get_page_render_tree(0).unwrap()).unwrap();
    assert_eq!(
        note(&roundtrip).unwrap(),
        original_note,
        "저장 후에도 각주 내용과 예약 영역을 보존한다"
    );

    // 원본의 색만 검정으로 바꾼 계약 대조군: 색은 잉크만 바꾸며 여백·문단 위치를 바꾸지 않는다.
    let mut black = doc.document().clone();
    black.sections[0].section_def.footnote_shape.separator_color = 0;
    doc.set_document(black);
    let black_tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let black_note = note(&black_tree).unwrap();
    assert_eq!(black_note["bbox"], original_note["bbox"]);
    let black_children = black_note["children"].as_array().unwrap();
    assert_eq!(
        black_children
            .iter()
            .filter(|n| n["type"] == "Line")
            .count(),
        1
    );
    assert_eq!(
        black_children
            .iter()
            .filter(|n| n["type"] != "Line")
            .collect::<Vec<_>>(),
        children.iter().collect::<Vec<_>>()
    );
}
