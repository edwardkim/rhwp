//! [#3622] `hwp_split_document` — extract-pages(--json 기보유)의 MCP 노출 + 자기서술
//! 등재. 드리프트 가드가 못 잡던 사각(계약은 완성인데 capabilities·도구 미등재) 봉합.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/hwp3-sample.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-split-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

#[test]
fn split_envelope_and_reread_page_count() {
    let p = sample();
    if !p.exists() {
        eprintln!("표본 없음 — 건너뜀");
        return;
    }
    let out = temp_path("env");
    let args = [
        "extract-pages",
        p.to_str().unwrap(),
        out.to_str().unwrap(),
        "--from",
        "2",
        "--to",
        "4",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(output.status.code(), Some(0), "exit");
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("envelope");
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    let pages_after = v["pagesAfter"].as_u64().expect("pagesAfter");
    assert!(v["pagesBefore"].as_u64().unwrap() > pages_after, "{v}");

    // 재독: 추출본은 원본보다 확실히 작아야 한다. 정확한 쪽수 일치는 계약이 아니다 —
    // extract-pages 는 문단 단위로 지우고 재조판이 흐르므로(#3565 문서화) 봉투의
    // pagesAfter 와 재독 쪽수가 ±α 다를 수 있다(실측: 보고 3 vs 재독 4).
    let info = run(&["info", out.to_str().unwrap(), "--json"]);
    let iv: serde_json::Value = serde_json::from_slice(&info.stdout).expect("info");
    let reread = iv["pageCount"].as_u64().expect("pageCount");
    let before = v["pagesBefore"].as_u64().unwrap();
    assert!(
        reread < before,
        "추출본이 원본보다 작아야 합니다: {reread} vs {before}"
    );
    assert!(
        reread >= pages_after.saturating_sub(1) && reread <= pages_after + 1,
        "재조판 편차는 ±1 이내여야 합니다: 보고 {pages_after} vs 재독 {reread}"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn capabilities_and_mcp_declare_split() {
    // [#3622] 사각 봉합의 본론: json:true 등재 + 도구 대응.
    let caps = run(&["capabilities"]);
    let v: serde_json::Value = serde_json::from_slice(&caps.stdout).expect("caps");
    let entry = v["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "extract-pages")
        .expect("extract-pages 등재");
    assert_eq!(entry["json"], true, "{entry}");

    let mcp = run(&["capabilities", "--mcp"]);
    let m: serde_json::Value = serde_json::from_slice(&mcp.stdout).expect("mcp");
    let tool = m["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "hwp_split_document")
        .expect("hwp_split_document 선언");
    let targs: Vec<&str> = tool["cli"]["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.as_str())
        .collect();
    for ph in ["{path}", "{output}", "{from}", "{to}"] {
        assert!(targs.contains(&ph), "{targs:?}");
    }
    let req: Vec<&str> = tool["inputSchema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r.as_str())
        .collect();
    assert_eq!(req.len(), 4, "required↔자리표시자 1:1: {req:?}");
}
