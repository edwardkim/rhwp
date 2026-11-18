//! 관계형·고정형: 저장 줄 없는 HWP5 글자처럼 취급 표의 칸 글줄은 칸 안에 들고 표는 겹치지 않는다(관계형).
//! 머리 칸 「의무설치기한」은 접힌 줄이 순서대로 모두 남는다(관계형). 전체 쪽수 1쪽은 독립 한컴
//! Print PDF 두 본(2022·2020) 모두의 값으로 고정한다(고정형).
//! 주석 갱신: 2026-11-06T03:01:24+09:00 (새 파일).
//! 측정: 2026-11-06, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 기준
//! `pdf/issue8639/sign_board_register_card-print-2022.pdf`(Hwp 2022 12.0.0.4547 Print, 1쪽),
//! 글꼴 Haansoft Batang(`HBATANG.TTF`, PDF 내장 글꼴과 같음).
//! p1 Native 100.00% / fresh WASM 99.11% (수정 전 base 36.92% / 38.35%). 전체 쪽수 rhwp 1 = 한컴 1,
//! 전쪽 최저 Native 100.00% · WASM 99.11%. 증거 `mydocs/pr/assets/issue_8639/`.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

/// 성북구 보조금지원 표지판 관리대장·관리카드 서식(HWP5, 셀 문단에 저장 LineSeg 없음).
const SOURCE: &str = "samples/issue8639/sign_board_register_card.hwp";
/// 독립 한컴 2022 Print(`pdf/issue8639/sign_board_register_card-print-2022.pdf`)의 전체 쪽수.
const PRINT_PAGES: usize = 1;
/// 칸 경계와 글줄 상자 비교의 반올림 여유.
const EPS: f64 = 0.5;

struct Outputs(PathBuf);
impl Drop for Outputs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn render_tree_pages() -> Vec<Value> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let out =
        Outputs(std::env::temp_dir().join(format!("rhwp-8639-{}-{stamp}", std::process::id())));
    fs::create_dir(&out.0).unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE);
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(&source)
        .arg("-o")
        .arg(&out.0)
        .output()
        .expect("render tree 출력");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut files: Vec<_> = fs::read_dir(&out.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut result = vec![node];
    for child in node["children"].as_array().into_iter().flatten() {
        result.extend(nodes(child));
    }
    result
}

fn bbox(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}

fn bottom(node: &Value) -> f64 {
    bbox(node, "y") + bbox(node, "h")
}

fn text_of(node: &Value) -> String {
    nodes(node)
        .iter()
        .filter_map(|n| n["text"].as_str())
        .flat_map(str::chars)
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn hwp5_no_lineseg_tac_table_rows_hold_their_wrapped_cell_lines() {
    let source = fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SOURCE)).unwrap();
    let document = rhwp::parser::parse_document(&source).unwrap();
    // 전제: 원본이 저장 줄 없는 셀의 TAC 표다(수정 대상 경로의 입력 조건).
    let mut tac_tables = 0;
    for paragraph in &document.sections[0].paragraphs {
        for control in &paragraph.controls {
            if let rhwp::model::control::Control::Table(table) = control {
                assert!(table.common.treat_as_char, "원본 표는 글자처럼 취급");
                assert!(
                    table
                        .cells
                        .iter()
                        .flat_map(|cell| &cell.paragraphs)
                        .all(|p| p.line_segs.is_empty()),
                    "원본 셀 문단에 저장 줄이 없다"
                );
                tac_tables += 1;
            }
        }
    }
    assert_eq!(tac_tables, 3, "원본의 TAC 표 3개");

    let pages = render_tree_pages();
    assert_eq!(pages.len(), PRINT_PAGES, "독립 Print 의 전체 쪽수");
    let all = nodes(&pages[0]);
    let tables: Vec<&Value> = all
        .iter()
        .copied()
        .filter(|n| n["type"] == "Table")
        .collect();
    assert_eq!(tables.len(), 3, "원본 표 3개가 모두 그려진다");

    // 표는 원본 순서대로 놓이고 서로 겹치지 않는다.
    for pair in tables.windows(2) {
        assert!(
            bottom(pair[0]) <= bbox(pair[1], "y") + EPS,
            "앞 표 아래({:.1})가 뒤 표 위({:.1})를 넘지 않는다",
            bottom(pair[0]),
            bbox(pair[1], "y")
        );
    }

    // 모든 칸의 글줄은 그 칸 안에 든다(선언 높이로 눌러 줄이 칸 밖으로 잘리지 않는다).
    for table in &tables {
        for cell in nodes(table).into_iter().filter(|n| n["type"] == "Cell") {
            for line in nodes(cell).into_iter().filter(|n| n["type"] == "TextLine") {
                assert!(
                    bottom(line) <= bottom(cell) + EPS && bbox(line, "y") >= bbox(cell, "y") - EPS,
                    "행 {} 열 {} 글줄 {:?} [{:.1}, {:.1}] 이 칸 [{:.1}, {:.1}] 밖",
                    cell["row"],
                    cell["col"],
                    text_of(line),
                    bbox(line, "y"),
                    bottom(line),
                    bbox(cell, "y"),
                    bottom(cell)
                );
            }
        }
    }

    // 첫 표 머리 행 「의무설치기한」은 칸 폭에서 접힌 여러 줄이 순서대로 모두 남는다.
    // 몇 줄로 접히는지는 글꼴에 달렸다(서울체 원본 2020 Print 2줄, Haansoft Batang 대체 2022
    // Print 3줄)이므로 줄 수는 고정하지 않는다.
    let header = nodes(tables[0])
        .into_iter()
        .find(|n| n["type"] == "Cell" && n["row"] == 0 && text_of(n) == "의무설치기한")
        .expect("머리 행 의무설치기한 칸");
    let lines: Vec<String> = nodes(header)
        .into_iter()
        .filter(|n| n["type"] == "TextLine")
        .map(text_of)
        .collect();
    assert!(lines.len() >= 2, "머리 칸이 접히지 않았다: {lines:?}");
    assert_eq!(lines.concat(), "의무설치기한", "줄 순서와 보존: {lines:?}");
}
