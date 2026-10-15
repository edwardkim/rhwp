//! 가맹사업 원본의 저장 앞 띠·반복 프레임·선택 줄 정렬 관계.
//! 같은 원본 전142쪽 Native/fresh WASM 최저93.16816%·쪽수 일치와 직접 판독 후 등록했다.
//! 증거: mydocs/pr/assets/semanticist21-20261008/franchise-whole-dual-current-proof-v453/README.md.
//! 기대 관계는 독립 한컴 2020 Print와 원본 Center/LineSeg에서 정한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{document_core::DocumentCore, model::control::Control};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::Command,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

const SAMPLE: &str = "samples/issue5966/1130000-202100008_franchise_review_report.hwp";

struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn walk<'a>(node: &'a Value, nodes: &mut Vec<&'a Value>) {
    nodes.push(node);
    for child in node["children"].as_array().into_iter().flatten() {
        walk(child, nodes);
    }
}
fn number(node: &Value, field: &str) -> f64 {
    node["bbox"][field].as_f64().unwrap()
}
fn text(node: &Value) -> String {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| n["text"].as_str())
        .collect::<String>()
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
fn trees() -> &'static Vec<Value> {
    static TREES: OnceLock<Vec<Value>> = OnceLock::new();
    TREES.get_or_init(|| {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = OutputDir(
            std::env::temp_dir().join(format!("rhwp-stored-frame-{}-{nonce}", std::process::id())),
        );
        std::fs::create_dir(&dir.0).unwrap();
        let binary = std::env::var("CARGO_BIN_EXE_rhwp")
            .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
        let output = Command::new(binary)
            .arg("export-render-tree")
            .arg(sample_path())
            .args(["--compat", "2022", "-o"])
            .arg(&dir.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("render_tree_") && name.ends_with(".json"))
            })
            .collect();
        files.sort();
        assert_eq!(files.len(), 142, "독립 Print 전쪽 수와 대응한다");
        let result = files
            .iter()
            .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
            .collect();
        result
    })
}
fn table(page: &Value, pi: usize) -> &Value {
    let mut nodes = Vec::new();
    walk(page, &mut nodes);
    let matches: Vec<_> = nodes
        .into_iter()
        .filter(|n| n["type"] == "Table" && n["pi"] == pi)
        .collect();
    assert_eq!(matches.len(), 1, "소유 문단의 표 조각은 한 번만 출력한다");
    matches[0]
}
fn cell(table: &Value, row: u16, col: u16) -> &Value {
    let matches: Vec<_> = table["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "Cell" && n["row"] == row && n["col"] == col)
        .collect();
    assert_eq!(matches.len(), 1, "행·열 소유 셀은 한 번만 출력한다");
    matches[0]
}
fn check_center(cell: &Value, stored_lead_hu: i32) {
    let lines: Vec<_> = cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "TextLine")
        .collect();
    assert!(!lines.is_empty());
    let top = lines
        .iter()
        .map(|l| number(l, "y"))
        .fold(f64::INFINITY, f64::min);
    let bottom = lines
        .iter()
        .map(|l| number(l, "y") + number(l, "h"))
        .fold(f64::NEG_INFINITY, f64::max);
    let above = top - number(cell, "y");
    let below = number(cell, "y") + number(cell, "h") - bottom;
    assert!(above >= 0.0 && below >= 0.0, "선택 줄은 소유 셀 안에 있다");
    let line_height = number(lines[0], "h");
    let expected = f64::from(stored_lead_hu) / 75.0 / line_height;
    let actual = (above - below) / line_height;
    // CLI bbox의 0.1 단위 반올림 여섯 항이 만드는 최대 오차를 줄 높이로 정규화한다.
    // 절대 위치·영역이나 임시 배치를 기대값으로 고정하지 않는다.
    let rounding_bound = 6.0 * 0.05 / line_height;
    assert!(
        (actual - expected).abs() <= rounding_bound + f64::EPSILON * 128.0,
        "Center 앞·뒤 띠 관계 actual={actual}, source={expected}"
    );
}
fn core() -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(sample_path()).expect("가맹사업 원본")).unwrap()
}
#[test]
fn complete_centered_cells_own_the_first_stored_before_band() {
    let core = core();
    let doc = core.document();
    let Control::Table(source) = &doc.sections[0].paragraphs[557].controls[0] else {
        panic!("원본 판례 표")
    };
    for (page, row) in [(54, 0), (54, 1), (56, 4)] {
        for col in [0, 1] {
            let original = source
                .cells
                .iter()
                .find(|c| c.row == row && c.col == col)
                .unwrap();
            assert_eq!(
                original.vertical_align,
                rhwp::model::table::VerticalAlign::Center
            );
            let first = &original.paragraphs[0].line_segs[0];
            let ps = &doc.doc_info.para_shapes[original.paragraphs[0].para_shape_id as usize];
            assert_eq!(
                first.vertical_pos,
                ps.spacing_before / 2,
                "저장 앞 띠의 독립 입력 계약"
            );
            let actual = cell(table(&trees()[page], 557), row, col);
            let expected = original
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>();
            assert_eq!(
                compact(&text(actual)),
                compact(&expected),
                "완료 셀 내용의 순서·보존"
            );
            check_center(actual, first.vertical_pos);
        }
    }
    // 57쪽의 이전 행 꼬리는 첫 원본 문단을 소유하지 않는다. 앞 띠를 다시 넣지 않는다.
    check_center(cell(table(&trees()[56], 557), 3, 1), 0);
}
#[test]
fn repeated_source_frame_centers_selected_lines_without_losing_the_tail() {
    let core = core();
    let doc = core.document();
    let Control::Table(source) = &doc.sections[0].paragraphs[719].controls[0] else {
        panic!("원본 거래조건 표")
    };
    let first = table(&trees()[77], 719);
    let repeated = table(&trees()[78], 719);
    let bottom = |n: &Value| number(n, "y") + number(n, "h");
    assert!(
        (bottom(first) - bottom(repeated)).abs() <= 4.0 * 0.05,
        "중간부터 시작한 첫 조각과 반복 조각은 같은 저장 프레임 하단을 닫는다"
    );
    for col in [0, 1] {
        let original = source
            .cells
            .iter()
            .find(|c| c.row == 6 && c.col == col)
            .unwrap();
        let current = cell(repeated, 6, col);
        let following = cell(table(&trees()[79], 719), 6, col);
        check_center(current, original.paragraphs[0].line_segs[0].vertical_pos);
        let expected = original
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>();
        assert_eq!(
            compact(&(text(current) + &text(following))),
            compact(&expected),
            "종료 프레임과 다음 조각 사이에 내용 누락·중복·순서 변경이 없다"
        );
        if col == 1 {
            let first_tail = following["children"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["type"] == "TextLine")
                .unwrap();
            assert_eq!(
                first_tail["pi"], 1,
                "다음 조각은 다음 원본 문단에서 이어진다"
            );
        } else {
            assert!(
                text(following).is_empty(),
                "이미 소비한 이름은 되풀이하지 않는다"
            );
        }
    }
}

#[test]
fn full_width_source_frame_centers_only_selected_lines() {
    let core = core();
    let doc = core.document();
    let Control::Table(source) = &doc.sections[0].paragraphs[661].controls[0] else {
        panic!("원본 불공정거래행위 표")
    };
    let current = table(&trees()[71], 661);
    let next = table(&trees()[72], 661);
    for col in [0, 1] {
        let original = source
            .cells
            .iter()
            .find(|c| c.row == 2 && c.col == col)
            .unwrap();
        assert_eq!(
            original.vertical_align,
            rhwp::model::table::VerticalAlign::Center
        );
        let actual = cell(current, 2, col);
        let tail = cell(next, 2, col);
        let first = &original.paragraphs[0].line_segs[0];
        check_center(actual, first.vertical_pos);
        let expected = original
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>();
        assert_eq!(
            compact(&(text(actual) + &text(tail))),
            compact(&expected),
            "현재 프레임과 다음 조각의 내용은 원본 순서를 보존하며 누락·중복이 없다"
        );
        if col == 0 {
            let selected = original.paragraphs[..2]
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>();
            assert_eq!(
                compact(&text(actual)),
                compact(&selected),
                "독립 Print의 첫 두 문단만 현재 셀 조각에 속한다"
            );
            let first_tail = tail["children"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["type"] == "TextLine")
                .unwrap();
            assert_eq!(
                first_tail["pi"], 2,
                "다음 조각은 세 번째 원본 문단부터 이어진다"
            );
        } else {
            assert!(
                text(tail).is_empty(),
                "이미 완료한 오른쪽 분류명은 중복 출력하지 않는다"
            );
        }
    }
}

#[test]
fn nested_source_frame_keeps_the_opening_paragraph_centered_and_the_tail_owned() {
    let core = core();
    let doc = core.document();
    let Control::Table(outer) = &doc.sections[1].paragraphs[461].controls[0] else {
        panic!("원본 부당한 거래상대방 강제 조항의 바깥 표")
    };
    let owner = outer
        .cells
        .iter()
        .find(|c| c.row == 4 && c.col == 0)
        .unwrap();
    let Control::Table(nested) = &owner.paragraphs[32].controls[0] else {
        panic!("해당 조항을 소유한 중첩 표")
    };
    let original = &nested.cells[0];
    assert_eq!(
        original.vertical_align,
        rhwp::model::table::VerticalAlign::Center
    );
    let first = &original.paragraphs[0];
    let shape = &doc.doc_info.para_shapes[first.para_shape_id as usize];
    assert_eq!(shape.spacing_before, shape.spacing_after);
    let lead = first.line_segs[0].vertical_pos;
    assert!(lead > 0);
    assert_eq!(lead, shape.spacing_before / 2);
    // 독립 Print 118쪽은 첫 문단만 셀 중앙에, 119쪽은 다음 세 문단에 속한다.
    // 저장 앞 띠를 넘는 상하 여유의 편차는 가운데 정렬이 아니다.
    // 절대 좌표나 현재 구현에서 계산한 높이를 기대값으로 쓰지 않는다.
    let opening = cell(table(&trees()[117], 32), 0, 0);
    let following = cell(table(&trees()[118], 32), 0, 0);
    assert_eq!(compact(&text(opening)), compact(&first.text));
    let tail = original.paragraphs[1..]
        .iter()
        .map(|p| p.text.as_str())
        .collect::<String>();
    assert_eq!(compact(&text(following)), compact(&tail));
    let lines: Vec<_> = opening["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "TextLine")
        .collect();
    assert_eq!(lines.len(), first.line_segs.len());
    assert!(lines.iter().all(|n| n["pi"] == 0));
    let top = number(lines[0], "y");
    let bottom = lines
        .iter()
        .map(|n| number(n, "y") + number(n, "h"))
        .fold(f64::NEG_INFINITY, f64::max);
    let above = top - number(opening, "y");
    let below = number(opening, "y") + number(opening, "h") - bottom;
    let line_height = number(lines[0], "h");
    let imbalance = (above - below) / line_height;
    let source_band = f64::from(lead) / 75.0 / line_height;
    let rounding_bound = 6.0 * 0.05 / line_height;
    assert!(above >= 0.0 && below >= 0.0, "글줄은 소유 셀 안에 있다");
    assert!(
        imbalance >= -rounding_bound && imbalance <= source_band + rounding_bound,
        "가운데 정렬의 여유 편차가 원본 앞 띠 범위를 벗어났다: {imbalance} / {source_band}"
    );
    let first_tail = following["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "TextLine")
        .unwrap();
    assert_eq!(
        first_tail["pi"], 1,
        "이어받기는 원본 다음 문단부터 시작한다"
    );
}
