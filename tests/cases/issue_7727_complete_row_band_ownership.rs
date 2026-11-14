//! 검사 유형: 관계형 — 완료한 끝 행은 실제 원본 줄·안여백의 상자를 유지한다.
//! 주석 갱신: 2026-11-14T22:35:49+09:00.
//! CBTA 전204쪽은 Native 최저97.00%(96쪽)로 아직 문서 전체98% 자격 전이다.
//! 대상162·166쪽은 Native99.94/99.95%이며 fresh WASM도 각각99.94/99.95%다.
//! 독립 한컴2020 Print의 닫는 선은 각각724.57064/726.00936 CSSpx다.
//! 그 절대 좌표를 고정하지 않고, 같은 줄을 온전히 받은 마지막 행의 높이가
//! 원본 마지막 줄 끝+상하 안여백과 같다는 소유 관계를 검사한다.
//! 조각 전체의 남은 공간을 그 행에 강제로 넣은 이전 코드는 이 관계를 위반한다.
//! compact render tree의 소수 첫째 자리 직렬화 격자만 비교하며 원본 글꼴 설치에 의존하지 않는다.

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
    fn find(node: &Value, row: u16, out: &mut Vec<f64>) {
        if node["type"] == "Cell"
            && node["row"].as_u64() == Some(u64::from(row))
            && node["col"] == 3
        {
            out.push(node["bbox"]["h"].as_f64().unwrap());
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
        (heights[0] * 10.0).round() as i64,
        (expected * 10.0).round() as i64,
        "{page}쪽 끝 행{row}: 실제{}px, 완료한 원본 줄·안여백{expected}px",
        heights[0]
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
