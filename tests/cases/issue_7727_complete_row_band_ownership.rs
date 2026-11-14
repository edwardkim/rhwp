//! 검사 유형: 고정형 — 독립 Print162·166쪽의 닫는 선을 ±2 CSSpx로 보호한다.
//! 검사 유형: 관계형 — 같은 끝 행의 원본 줄·안여백과 실제 높이의 소유를 함께 검사한다.
//! 주석 갱신: 2026-11-14T23:28:54+09:00.
//! CBTA 원본204쪽, 대상162·166쪽은 Native/fresh WASM 각각99.94/99.95%다.
//! 2026-11-14 정본 PNG Print / compat2022 / 96DPI / raw2px, 같은 쪽 review·overlay를 판독했다.
//! 문서 최저가98% 미만이더라도 해당 쪽의 양쪽98% 자격으로 px 회귀를 추가한다.
//! 독립 PDF 선:162쪽724.5706380208334,166쪽726.0093587239584 CSSpx.
//! 근거: pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf와
//! mydocs/pr/assets/issues7727_7732_20261114/complete-row-band/cause.json.
//! 실제 완료한 원본 줄·패딩의 관계도 유지하여 남은 쪽 공간을 끝 행에 붙이는 결함을 검출한다.

#![cfg(feature = "native-skia")]

use rhwp::{model::control::Control, DocumentCore};
use serde_json::Value;
use std::{path::PathBuf, process::Command};

struct OwnedTree(PathBuf);
impl Drop for OwnedTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn check(page: usize, row: u16) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(&source).unwrap()).unwrap();
    let table = core.document().sections[1].paragraphs[5]
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    let cells: Vec<_> = table
        .cells
        .iter()
        .filter(|c| c.row == row && c.row_span == 1)
        .collect();
    assert!(
        cells
            .iter()
            .any(|c| c.paragraphs.iter().any(|p| p.line_segs.len() == 2)),
        "같은 행에서 완료한 두 원본 줄을 비교한다"
    );
    let expected = cells
        .iter()
        .map(|cell| {
            assert_eq!(cell.paragraphs.len(), 1);
            let para = &cell.paragraphs[0];
            assert!(para.controls.is_empty());
            let last = para.line_segs.last().unwrap();
            (f64::from(last.vertical_pos + last.line_height)
                + f64::from(cell.padding.top)
                + f64::from(cell.padding.bottom))
                * 96.0
                / 7200.0
        })
        .fold(0.0_f64, f64::max);
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temporary = OwnedTree(std::env::temp_dir().join(format!(
        "rhwp-7727-closed-row-{}-{suffix}",
        std::process::id()
    )));
    std::fs::create_dir(&temporary.0).unwrap();
    let tree_dir = temporary.0.join("trees");
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-png")
        .arg(source)
        .args([
            "--page",
            &(page - 1).to_string(),
            "--profile",
            "print",
            "--compat",
            "2022",
            "--json",
            "--output",
        ])
        .arg(&temporary.0)
        .arg("--render-tree-output")
        .arg(&tree_dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "Print 트리 산출 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(tree_dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1);
    let tree: Value = serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    fn find(node: &Value, row: u16, out: &mut Vec<(f64, f64)>) {
        if node["type"] == "Cell"
            && node["row"].as_u64() == Some(u64::from(row))
            && node["col"] == 3
        {
            let height = node["bbox"]["h"].as_f64().unwrap();
            out.push((height, node["bbox"]["y"].as_f64().unwrap() + height));
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                find(child, row, out);
            }
        }
    }
    let mut heights = Vec::new();
    find(&tree, row, &mut heights);
    assert_eq!(heights.len(), 1, "끝 행의 누락·중복");
    assert_eq!(
        (heights[0].0 * 10.0).round() as i64,
        (expected * 10.0).round() as i64,
        "{page}쪽 끝 행{row}: 실제{}px, 완료한 원본 줄·안여백{expected}px",
        heights[0].0
    );
    let pdf_close = match page {
        162 => 724.5706380208334,
        166 => 726.0093587239584,
        _ => unreachable!(),
    };
    assert!(
        (heights[0].1 - pdf_close).abs() <= 2.0,
        "{page}쪽 끝 선: 독립 Print{pdf_close}px, 실제{}px",
        heights[0].1
    );
}

#[test]
fn first_closed_continuation_row_keeps_its_content_box() {
    check(162, 188);
}
#[test]
fn later_closed_continuation_row_keeps_its_content_box() {
    check(166, 252);
}
