//! [#3565] `hwp_split_document` 의 쪽 기준이 CLI `extract-pages` 와 같은지 감시한다.
//!
//! rhwp 의 쪽 축은 거의 전부 **0 기준**이다 — `-p`, `export-text` 의 `page`,
//! `search` 의 `matches[].page`. `agent_knowledge_map.md` 가 이를 불변식으로 적어
//! 뒀고 에이전트는 그 규약대로 인자를 만든다. 그런데 `extract-pages` 만 **1 기준**
//! 이다(런타임 오류 문구가 그렇게 말한다).
//!
//! MCP 스키마가 이를 "0 기준" 이라 선언하면 실패가 아니라 **오답**이 나온다.
//! `from: 1` 은 0 기준으로 2쪽을 뜻하는데 CLI 는 1쪽을 자른다 — 한 쪽 밀린 문서가
//! 오류 없이 산출되고, 에이전트는 요청대로 됐다고 믿는다.
//!
//! 이 테스트는 선언된 `minimum` 과 CLI 가 실제로 받아들이는 최소값이 같은지 본다.
//! 어느 한쪽이 바뀌면 여기서 먼저 걸린다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/2010-01-06.hwp";

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn manifest() -> serde_json::Value {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["capabilities", "--mcp"])
        .output()
        .expect("capabilities 실행 실패");
    serde_json::from_slice(&out.stdout).expect("capabilities --mcp JSON")
}

fn split_tool(m: &serde_json::Value) -> serde_json::Value {
    m["tools"]
        .as_array()
        .expect("tools 배열")
        .iter()
        .find(|t| t["name"] == "hwp_split_document")
        .expect("hwp_split_document 선언이 없습니다")
        .clone()
}

#[test]
fn split_document_declares_the_one_based_page_axis() {
    let m = manifest();
    let t = split_tool(&m);
    for key in ["from", "to"] {
        let prop = &t["inputSchema"]["properties"][key];
        assert_eq!(
            prop["minimum"], 1,
            "{key} 의 minimum 이 1 이 아니다 — CLI extract-pages 는 1 기준이라 0 을 받으면 \
             오류다. 0 을 허용한다고 선언하면 에이전트가 첫 쪽을 0 으로 보내 실패한다: {prop}"
        );
        let desc = prop["description"].as_str().unwrap_or("");
        assert!(
            desc.contains("1 기준"),
            "{key} 의 설명이 기준을 말하지 않는다. rhwp 의 다른 page 인자는 0 기준이므로 \
             기준을 적지 않으면 한 쪽 밀린 문서가 조용히 나온다: {desc}"
        );
    }
}

#[test]
fn declared_minimum_is_the_page_the_cli_actually_accepts() {
    let p = sample();
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let m = manifest();
    let t = split_tool(&m);
    let declared_min = t["inputSchema"]["properties"]["from"]["minimum"]
        .as_i64()
        .expect("from.minimum 은 정수");

    let tmp = std::env::temp_dir();
    let run = |from: i64| {
        let out = tmp.join(format!("rhwp_split_base_{from}.hwp"));
        let _ = std::fs::remove_file(&out);
        let r = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .args([
                "extract-pages",
                &p.to_string_lossy(),
                &out.to_string_lossy(),
                "--from",
                &from.to_string(),
                "--to",
                &from.to_string(),
                "--json",
            ])
            .output()
            .expect("extract-pages 실행 실패");
        let _ = std::fs::remove_file(&out);
        r.status.success()
    };

    assert!(
        run(declared_min),
        "선언된 minimum({declared_min})을 CLI 가 거부한다 — 스키마를 따른 에이전트가 \
         첫 호출부터 실패한다"
    );
    assert!(
        !run(declared_min - 1),
        "선언된 minimum({declared_min}) 보다 하나 작은 값도 CLI 가 받아들인다 — 기준이 \
         드리프트했다. 스키마의 minimum 과 설명을 CLI 에 맞춰 갱신하라"
    );
}
