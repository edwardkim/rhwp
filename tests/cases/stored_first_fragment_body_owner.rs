//! 관계형: 첫 조각과 이어받기의 행 소속·누락·중복을 검사한다. 고정형: 완료된 80·81쪽 표 위끝만 독립 PDF 좌표 ±2px로 검사한다.
//! 주석 갱신: 2026-11-11T18:07:26+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 원본 HPV 코호트 HWP와 독립 한/글 2020 Print(235쪽), PDFium·96 DPI·색상 경계 보정 전 2px raw.
//! R9 제품 499a50db7b: Native/fresh WASM 80쪽 99.90/99.90%, 81쪽 99.51/99.52%. 최소 글꼴 조건도 각각 99.86/99.86%, 99.50/99.51%다.
//! 두 backend의 같은 쪽 review·overlay를 직접 판독했다. PDF 80쪽 표12 윗선 중심 644.25468px, 81쪽 이어받기 윗선 중심 113.15601px.
//! 증적: mydocs/pr/assets/planet6897_remaining39_20261111/maintainer-repair/r9-first-fragment/.
//! 전체 문서의 나머지 미달 또는 최신 전체 회귀 통과를 주장하지 않는다. 설치 원본 글꼴 없이 최소 글꼴 환경에서 실행한다.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

const INPUT: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";

fn binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_rhwp")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_rhwp")))
}

struct TreeOutput(PathBuf);

impl Drop for TreeOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn tree(page: usize) -> Value {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("시각")
        .as_nanos();
    let output = TreeOutput(std::env::temp_dir().join(format!(
        "rhwp-first-fragment-{}-{stamp}-{page}",
        std::process::id()
    )));
    std::fs::create_dir(&output.0).expect("검사 소유 임시 경로");
    let result = Command::new(binary())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["export-render-tree", INPUT, "--compat", "2022", "-p"])
        .arg((page - 1).to_string())
        .arg("-o")
        .arg(&output.0)
        .output()
        .expect("공개 CLI의 실제 paint 트리 생성");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(
        &std::fs::read(output.0.join(format!("render_tree_{page:03}.json"))).expect("지정 쪽 트리"),
    )
    .expect("렌더 트리 JSON")
}

fn host_table(node: &Value) -> Option<&Value> {
    if node["type"] == "Table" {
        // 다른 표의 중첩 칸 문단 번호를 본문의 표 소속으로 세지 않는다.
        return (node["pi"] == 47).then_some(node);
    }
    node.get("children")?
        .as_array()?
        .iter()
        .find_map(host_table)
}

fn rows(table: &Value) -> BTreeSet<u64> {
    table["children"]
        .as_array()
        .expect("표 자식")
        .iter()
        .filter(|node| node["type"] == "Cell")
        .map(|cell| cell["row"].as_u64().expect("원본 행 번호"))
        .collect()
}

#[test]
fn first_fragment_height_does_not_compress_the_whole_body_or_lose_its_tail() {
    let p80 = tree(80);
    let p81 = tree(81);
    let head = host_table(&p80).expect("80쪽 표12 첫 조각");
    let tail = host_table(&p81).expect("81쪽 표12 합계2·총합계 이어받기");
    let first = rows(head);
    let last = rows(tail);
    assert_eq!(first, (0..13).collect(), "첫 조각은 행0~12를 소유한다");
    assert_eq!(
        last,
        (13..15).collect(),
        "합계 두 행은 다음 쪽에서 이어진다"
    );
    assert!(
        first.is_disjoint(&last),
        "종료 행을 양쪽 조각에서 중복하지 않는다"
    );
    assert_eq!(
        first.union(&last).copied().collect::<BTreeSet<_>>(),
        (0..15).collect(),
        "전체 15행을 보존한다"
    );
    for (node, expected) in [(head, 644.25468), (tail, 113.15601)] {
        let y = node["bbox"]["y"].as_f64().expect("실제 paint 표 원점");
        assert!(
            (y - expected).abs() <= 2.0,
            "독립 Print 표 윗선 ±2px: 실제 {y}, 기준 {expected}"
        );
    }
}
