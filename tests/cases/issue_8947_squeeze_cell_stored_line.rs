//! 관계형 — [#8947] 한 줄로 입력(lineWrap=SQUEEZE) 칸의 과밀 저장 줄은 다시 나누지 않는다.
//! 주석 갱신: 2026-11-19 02:44 KST (직전 변경 bef571dc9a, 2026-11-11 19:00:10 KST).
//! 원본 재측정(2026-11-19, 같은 PDF·96 DPI): 2쪽 Native 98.21/98.06%, WASM 95.46/97.24%.
//! 편집·재개봉 검사는 원본의 건명 칸을 사용해 셀 내부 포함·글자 순서·문자 경계를 검증한다.
//!
//! 입력: 공개 결재문서 36481164(한컴오피스 2020 저장) 원본. 1쪽 15×30 표의 「건 명」 칸(`name="제목"`,
//! `lineWrap="SQUEEZE"`)에 글 「물품매입요구서(응봉공원 노후시설물 정비사업 관급자재 구매, 금액수정)」가
//! 저장 줄 하나(`horzsize=19808`)로 있다. 글의 자연 폭은 그 줄 폭의 약 1.9배다.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8947/36481164-squeeze-title-cell-2020.pdf`(2쪽) —
//! 한/글은 이 글을 한 줄에 겹쳐 찍는다.
//! - 관계형: 그 칸의 글은 렌더 트리에서 한 줄(TextLine 하나)에 놓인다.
//!
//! 반례: 같은 원본에서 그 칸만 `lineWrap="BREAK"` 로 바꾼 변형. 저장 한 줄이 과밀하므로 여러 줄로
//! 다시 나눈다(#2525 규칙은 SQUEEZE 가 아닌 칸에 그대로 적용된다).
//!
//! 측정(2026-11-11, 측정 코드 #8947 6cb75d5a52 · #8948 3e82a2968a(헤드 02511ea855), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`,
//! raw = `silhouette_raw_match_percent`):
//! - #8947 만(6cb75d5a52 와 같은 트리 a4c120eac1): Native p1 88.05 · p2 15.79, fresh WASM p1 88.00 · p2 16.04 — 2쪽은 #8948 축.
//! - #8947 + #8948(02511ea855): Native 2/2쪽 p1 98.09 · p2 97.47, fresh WASM p1 98.24 · p2 97.82,
//!   전쪽 최저 Native 97.47 · fresh WASM 97.82.
//! - 수정 전(base 28ccebe82a 코드): Native p1 68.92 · p2 15.79, fresh WASM p1 68.40 · p2 16.04.
//! - 증거: `mydocs/pr/assets/issue_8948/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8947/36481164-squeeze-title-cell.hwpx";
const TITLE_CELL_SQUEEZE: &str = r#"<hp:tc name="제목" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="24"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="SQUEEZE""#;
const TITLE_HEAD: &str = "물품매입요구서(응봉공원";
const TITLE_TAIL: &str = "금액수정)";

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// 「건 명」 칸만 `lineWrap="BREAK"` 로 바꾼 HWPX.
fn with_break_title_cell() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(sample_bytes())).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("entry");
        let name = entry.name().expect("name").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/section0.xml" {
            let xml = String::from_utf8(data).expect("utf-8");
            assert_eq!(
                xml.matches(TITLE_CELL_SQUEEZE).count(),
                1,
                "원본 「건 명」 칸"
            );
            data = xml
                .replace(
                    TITLE_CELL_SQUEEZE,
                    &TITLE_CELL_SQUEEZE.replace("SQUEEZE", "BREAK"),
                )
                .into_bytes();
        }
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

/// 1쪽 15×30 본문 표 안의 글줄만 모은다. 맨 위 1×1 제목 표에도 같은 글이 있으므로 뺀다.
fn walk_table_lines<'a>(node: &'a RenderNode, in_table: bool, out: &mut Vec<&'a RenderNode>) {
    let in_table = in_table
        || matches!(&node.node_type, RenderNodeType::Table(table)
            if table.row_count == 15 && table.col_count == 30);
    if in_table && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node);
    }
    for child in &node.children {
        walk_table_lines(child, in_table, out);
    }
}

fn line_text(line: &RenderNode) -> String {
    line.children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

/// 「건 명」 칸 글이 놓인 줄 수와, 머리·꼬리 글자가 같은 줄에 있는지.
fn title_lines(bytes: &[u8], label: &str) -> (usize, bool) {
    let core = DocumentCore::from_bytes(bytes).unwrap_or_else(|e| panic!("open {label}: {e:?}"));
    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut nodes = Vec::new();
    walk_table_lines(&page.root, false, &mut nodes);
    let lines: Vec<String> = nodes.iter().map(|node| line_text(node)).collect();
    let head = lines
        .iter()
        .position(|text| text.contains(TITLE_HEAD))
        .unwrap_or_else(|| panic!("{label}: 「건 명」 칸 글 머리"));
    let tail = lines
        .iter()
        .position(|text| text.contains(TITLE_TAIL))
        .unwrap_or_else(|| panic!("{label}: 「건 명」 칸 글 꼬리"));
    assert!(head <= tail, "{label}: 글 머리가 꼬리보다 앞 줄이어야 한다");
    (tail - head + 1, head == tail)
}

#[test]
fn squeeze_title_cell_keeps_its_single_stored_line() {
    let (lines, same) = title_lines(&sample_bytes(), "원본");
    assert!(
        same && lines == 1,
        "한 줄로 입력 칸의 글이 {lines}줄로 나뉘었다 — 한/글처럼 저장 줄 하나에 놓여야 한다"
    );
}

#[test]
fn break_title_cell_rewraps_the_overfull_stored_line() {
    let (lines, _) = title_lines(&with_break_title_cell(), "BREAK 변형");
    assert!(
        lines >= 2,
        "자동 줄바꿈 칸에서 과밀 저장 줄({lines}줄)은 다시 나눠야 한다(#2525)"
    );
}

// 원본 저장 줄을 편집하면 새 텍스트의 모든 문자 경계가 한 줄 안에 있어야 한다.
// 탭을 포함하면 저장 시 줄 캐시도 생략되므로 재개봉 경로까지 검사한다.
#[test]
fn edited_title_fits_one_line_with_distinct_character_boundaries_after_save() {
    use serde_json::Value;

    fn check(core: &DocumentCore, expected: &str) {
        let layout: Value =
            serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
        let cells: Value = serde_json::from_str(
            &core
                .get_table_cell_bboxes_at_page_native(0, 2, 1, 0)
                .unwrap(),
        )
        .unwrap();
        let cell = cells
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["cellIdx"] == 1)
            .unwrap();
        let left = cell["x"].as_f64().unwrap();
        let right = left + cell["w"].as_f64().unwrap();
        let runs: Vec<_> = layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["parentParaIdx"] == 2 && r["controlIdx"] == 1 && r["cellIdx"] == 1)
            .collect();
        let text: String = runs.iter().map(|r| r["text"].as_str().unwrap()).collect();
        assert_eq!(text, expected);
        for run in &runs {
            assert_eq!(run["y"], runs[0]["y"]);
            let x = run["x"].as_f64().unwrap();
            assert!(x >= left && x + run["w"].as_f64().unwrap() <= right);
            assert!(run["charX"]
                .as_array()
                .unwrap()
                .windows(2)
                .all(|pair| pair[0].as_f64().unwrap() < pair[1].as_f64().unwrap()));
        }
    }

    for (separator, enable_after_edit) in [(" ", false), ("\t", false), ("\t", true)] {
        let prefix = " ".repeat(20) + &format!("iiilll{separator}Long cell contents ").repeat(7);
        let mut core = DocumentCore::from_bytes(&sample_bytes()).unwrap();
        let rhwp::model::control::Control::Table(table) =
            &core.document().sections[0].paragraphs[2].controls[1]
        else {
            panic!("건 명 표")
        };
        let expected = prefix.clone() + &table.cells[1].paragraphs[0].text;
        if enable_after_edit {
            core.set_cell_properties_native(0, 2, 1, 1, r#"{"lineWrap":0}"#)
                .unwrap();
        }
        core.insert_text_in_cell_native(0, 2, 1, 1, 0, 0, &prefix)
            .unwrap();
        if enable_after_edit {
            core.set_cell_properties_native(0, 2, 1, 1, r#"{"lineWrap":1}"#)
                .unwrap();
        }
        check(&core, &expected);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            check(&DocumentCore::from_bytes(&bytes).unwrap(), &expected);
        }
    }
}

#[test]
fn cell_line_wrap_partial_update_preserves_neighbors_and_rejects_invalid_values() {
    let mut core = DocumentCore::from_bytes(&sample_bytes()).unwrap();
    let props = |core: &DocumentCore, cell| -> serde_json::Value {
        serde_json::from_str(
            &core
                .get_cell_properties_by_cell_path_native(0, 2, &[(1, 0, 0)], cell)
                .unwrap(),
        )
        .unwrap()
    };
    let neighbor = props(&core, 0);
    core.set_cell_properties_native(0, 2, 1, 1, r#"{"lineWrap":2}"#)
        .unwrap();
    core.set_cell_properties_native(0, 2, 1, 1, r#"{"isHeader":true}"#)
        .unwrap();
    assert_eq!(props(&core, 1)["lineWrap"], 2);
    for value in ["-1", "3", "257", "1.5", "null", r#""1""#] {
        let update = format!(r#"{{"isHeader":false,"lineWrap":{value}}}"#);
        assert!(core
            .set_cell_properties_native(0, 2, 1, 1, &update)
            .is_err());
        assert_eq!(props(&core, 1)["isHeader"], true);
        assert_eq!(props(&core, 1)["lineWrap"], 2);
    }
    assert_eq!(props(&core, 0), neighbor);
}
