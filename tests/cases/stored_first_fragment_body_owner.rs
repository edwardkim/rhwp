//! 관계형: 첫 조각과 이어받기의 행 소속·누락·중복을 검사한다. 고정형: 완료된 80·81쪽 표 위끝만 독립 PDF 좌표 ±2px로 검사한다.
//! 주석 갱신: 2026-11-11T18:52:50+09:00 (Asia/Seoul). R10 종료 본체 관계 검사를 추가했다.
//! 원본 HPV 코호트 HWP와 독립 한/글 2020 Print(235쪽), PDFium·96 DPI·색상 경계 보정 전 2px raw.
//! R9 제품 499a50db7b: Native/fresh WASM 80쪽 99.90/99.90%, 81쪽 99.51/99.52%. 최소 글꼴 조건도 각각 99.86/99.86%, 99.50/99.51%다.
//! 두 backend의 같은 쪽 review·overlay를 직접 판독했다. PDF 80쪽 표12 윗선 중심 644.25468px, 81쪽 이어받기 윗선 중심 113.15601px.
//! 증적: mydocs/pr/assets/planet6897_remaining39_20261111/maintainer-repair/r9-first-fragment/.
//! 추가 관계형 83쪽은 R10 Native/fresh WASM 92.23484/92.25131%. 표의 실제 마지막 행과 다음 빈 문단의 경계만 비교하며 절대 px 좌표를 고정하지 않는다.
//! 83쪽 증적: mydocs/pr/assets/planet6897_remaining39_20261111/maintainer-repair/r10-closed-body/.
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

fn table_for_para(node: &Value, para: u64) -> Option<&Value> {
    if node["type"] == "Table" {
        // 다른 표의 중첩 칸 문단 번호를 본문의 표 소속으로 세지 않는다.
        return (node["pi"] == para).then_some(node);
    }
    node.get("children")?
        .as_array()?
        .iter()
        .find_map(|child| table_for_para(child, para))
}

fn host_table(node: &Value) -> Option<&Value> {
    table_for_para(node, 47)
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

fn body_line(node: &Value, para: u64) -> Option<&Value> {
    if node["type"] == "Table" {
        return None;
    }
    if node["type"] == "TextLine" && node["pi"] == para {
        return Some(node);
    }
    node.get("children")?
        .as_array()?
        .iter()
        .find_map(|child| body_line(child, para))
}

#[test]
fn completed_row_body_does_not_reappear_as_a_gap_before_the_successor() {
    let p83 = tree(83);
    let table = table_for_para(&p83, 65).expect("표15 마지막 조각");
    let last_row = table["rows"].as_u64().expect("원본 행 수") - 1;
    let closing: Vec<_> = table["children"]
        .as_array()
        .expect("표 자식")
        .iter()
        .filter(|cell| cell["type"] == "Cell" && cell["row"] == last_row)
        .collect();
    assert!(!closing.is_empty(), "원본 마지막 합계 행을 보존한다");
    let last_bottom = closing
        .iter()
        .map(|cell| {
            cell["bbox"]["y"].as_f64().expect("마지막 행 위끝")
                + cell["bbox"]["h"].as_f64().expect("마지막 행 실제 높이")
        })
        .fold(f64::NEG_INFINITY, f64::max);
    let successor = body_line(&p83, 66).expect("표 뒤 원본 빈 문단");
    let next_top = successor["bbox"]["y"].as_f64().expect("다음 줄 위끝");
    // CLI 트리의 소수 첫째 자리 직렬화 오차만 정수 격자로 비교한다.
    // 절대 좌표나 문서별 간격을 기대값으로 넣지 않는다.
    assert_eq!(next_top.round(), last_bottom.round(),
        "같은 닫힌 행 본체를 후속 빈 띠로 다시 예약하지 않는다: 행 끝 {last_bottom}, 다음 줄 {next_top}");
    let body = body_line(&p83, 67).expect("이어지는 원본 본문");
    let empty_bottom = next_top + successor["bbox"]["h"].as_f64().expect("빈 줄 점유 높이");
    assert!(
        body["bbox"]["y"].as_f64().expect("본문 위끝") >= empty_bottom,
        "빈 문단의 실제 줄 상자를 보존하고 다음 본문과 겹치지 않는다"
    );
}
