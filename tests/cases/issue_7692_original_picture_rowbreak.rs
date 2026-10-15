//! #7692 원본 4쪽의 독립 Hancom Print와 Native/fresh WASM 전쪽 검증 근거.
//! 원본 선언과 마커·문단·셀 소유, 내용 순서·보존, 포함·겹침 관계를 검사한다.
//! 실물 배치를 절대 화면 좌표 또는 SVG hash로 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, style::HeadType, table::Table};
use serde_json::Value;
use std::{collections::BTreeMap, path::PathBuf, process::Command};
const SAMPLE: &str = "samples/issue7182/rowbreak_cell_picture_only_paragraph.hwp";
fn source(c: &DocumentCore) -> (usize, &Table) {
    c.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(ci, control)| match control {
            Control::Table(t) => Some((ci, t.as_ref())),
            _ => None,
        })
        .unwrap()
}
struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn fixture() -> (DocumentCore, Vec<Value>) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&input).unwrap()).unwrap();
    let out = TempDir(std::env::temp_dir().join(format!(
            "rhwp-7692-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    std::fs::create_dir(&out.0).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let run = Command::new(bin)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "-o"])
        .arg(&out.0)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "actual renderer: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let mut paths: Vec<_> = std::fs::read_dir(&out.0)
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    let pages = paths
        .iter()
        .map(|p| serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap())
        .collect();
    (core, pages)
}
fn nodes(n: &Value) -> Vec<&Value> {
    let mut all = vec![n];
    if let Some(children) = n["children"].as_array() {
        for child in children {
            all.extend(nodes(child));
        }
    }
    all
}
fn table(root: &Value, ci: usize) -> Option<&Value> {
    nodes(root)
        .into_iter()
        .find(|n| n["type"] == "Table" && n["pi"] == 0 && n["ci"] == ci)
}
fn cell(t: &Value, row: u16, col: u16) -> Option<&Value> {
    t["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == row && n["col"] == col)
}
fn coord(n: &Value, axis: &str) -> f64 {
    n["bbox"][axis].as_f64().unwrap()
}
fn bottom(n: &Value) -> f64 {
    coord(n, "y") + coord(n, "h")
}
fn compact(t: &str) -> String {
    t.chars().filter(|c| !c.is_whitespace()).collect()
}
#[test]
fn default_bullet_references_keep_markers_and_plain_paragraphs() {
    let (c, pages) = fixture();
    let (ci, t) = source(&c);
    let input = t
        .cells
        .iter()
        .find(|cell| cell.row == 20 && cell.col == 3)
        .unwrap();
    let info = &c.document().doc_info;
    let mut expected = BTreeMap::new();
    let (mut defaults, mut explicit) = (0, 0);
    for (pi, para) in input.paragraphs.iter().enumerate() {
        let style = &info.para_shapes[para.para_shape_id as usize];
        if style.head_type != HeadType::Bullet {
            continue;
        }
        let glyph = if style.numbering_id == 0 {
            defaults += 1;
            "●".to_owned() // 독립 Print가 같은 원본의 기본 참조 0에 그린 기호.
        } else {
            explicit += 1;
            info.bullets[style.numbering_id as usize - 1]
                .bullet_char
                .to_string()
        };
        expected.insert(pi, vec![glyph]);
    }
    assert!(defaults > 0 && explicit > 0);
    let mut actual: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut plain_text: BTreeMap<usize, String> = BTreeMap::new();
    for page in &pages {
        let Some(owner) = table(page, ci).and_then(|t| cell(t, 20, 3)) else {
            continue;
        };
        for n in nodes(owner) {
            if n["type"] != "TextRun" {
                continue;
            }
            let pi = n["pi"].as_u64().unwrap() as usize;
            let text = n["text"].as_str().unwrap();
            if let Some(glyphs) = expected.get(&pi) {
                if glyphs.iter().any(|glyph| glyph == text.trim()) {
                    actual.entry(pi).or_default().push(text.trim().to_owned());
                }
            } else {
                plain_text.entry(pi).or_default().push_str(text);
            }
        }
    }
    assert_eq!(
        actual, expected,
        "원본 마커와 문단 소유를 보존하며 일반 본문 문단에 추가하지 않는다"
    );
    for (pi, para) in input.paragraphs.iter().enumerate() {
        if info.para_shapes[para.para_shape_id as usize].head_type == HeadType::None {
            // 일반 본문의 리터럴 '-'는 마커가 아니다. 본문 보존으로 가짜 마커도 검출한다.
            assert_eq!(
                compact(plain_text.get(&pi).map_or("", String::as_str)),
                compact(&para.text),
                "일반 문단 {pi}의 본문은 가짜 마커 없이 그대로 보존한다"
            );
        }
    }
}
#[test]
fn metadata_host_repeats_source_outer_top_once() {
    let (c, pages) = fixture();
    let host = &c.document().sections[0].paragraphs[0];
    assert!(host.text.is_empty() && host.line_segs.is_empty());
    assert!(host
        .controls
        .iter()
        .any(|c| matches!(c, Control::SectionDef(_))));
    assert!(host
        .controls
        .iter()
        .any(|c| matches!(c, Control::ColumnDef(_))));
    let (ci, input) = source(&c);
    let expected_gap = f64::from(input.outer_margin_top) * 96.0 / 7200.0;
    assert!(expected_gap > 0.0);
    let mut fragments = 0;
    for page in &pages {
        let Some(t) = table(page, ci) else {
            continue;
        };
        let body = page["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["type"] == "Body")
            .unwrap();
        let gap = coord(t, "y") - coord(body, "y");
        // CLI bbox는 소수 첫째 자리로 직렬화한다. 상대 검사로 중복·누락을 구분한다.
        assert!(
            (gap / expected_gap - 1.0).abs() < 0.04,
            "원본 바깥 여백을 조각마다 한 번만 적용한다: {gap}"
        );
        fragments += 1;
    }
    assert!(fragments > 1, "메타데이터 호스트의 이어받기도 실행한다");
}
#[test]
fn partial_plain_row_keeps_source_minimum_and_owned_contents() {
    let (c, pages) = fixture();
    let (ci, source) = source(&c);
    let input = source
        .cells
        .iter()
        .find(|cell| cell.row == 19 && cell.col == 3)
        .unwrap();
    assert!(input.paragraphs.iter().all(|p| p.controls.is_empty()));
    let expected_text = compact(
        &input
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>(),
    );
    let minimum = f64::from(input.height) * 96.0 / 7200.0;
    // 원본 최소 높이에 대한 작은 상대 오차만 bbox 직렬화 반올림으로 허용한다.
    let rounding = minimum * 0.001;
    let mut height = 0.0;
    let mut actual_text = String::new();
    let mut fragments = 0;
    for page in &pages {
        let Some(t) = table(page, ci) else {
            continue;
        };
        let Some(owner) = cell(t, 19, 3) else {
            continue;
        };
        height += coord(owner, "h");
        fragments += 1;
        for n in nodes(owner) {
            if n["type"] != "TextRun" {
                continue;
            }
            let text = n["text"].as_str().unwrap();
            if text == "-" {
                continue;
            } // 원본의 별도 글머리표는 본문 문자에 속하지 않는다.
            actual_text.push_str(&compact(text));
            assert!(coord(n, "y") >= coord(owner, "y") - rounding);
            assert!(
                bottom(n) <= bottom(owner) + rounding,
                "소비한 본문이 셀 조각 밖에 숨지 않는다"
            );
        }
        if let Some(next) = cell(t, 20, 3) {
            assert!(
                bottom(owner) <= coord(next, "y") + rounding,
                "다음 행이 빈 물리 띠와 겹치지 않는다"
            );
        }
    }
    assert!(fragments > 1);
    assert!(
        height + rounding >= minimum,
        "원본 선언의 물리 최소 높이를 지우지 않는다: {height} / {minimum}"
    );
    assert_eq!(
        actual_text, expected_text,
        "분할 전후 본문 순서를 보존하며 누락과 중복을 검출한다"
    );
}
