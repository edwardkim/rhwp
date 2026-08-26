//! [#4530] 에이전트 대전 커버리지 가드.
//!
//! 대전의 내용 신선도는 `tools/gen_agent_codex.py --check`(멱등 재생성)가
//! 판정하고, 이 테스트는 **커버리지**를 판정한다: 자기서술의 전 명령이
//! 대전 어딘가에 장(`### \`이름\``)을 갖는가. 새 명령을 추가하면 대전을
//! 재생성할 때까지 이 가드가 붉는다 — 교본이 CLI 를 뒤따르게 만드는 장치.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn codex_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs")
        .join("manual")
        .join("agent_codex")
}

fn codex_text() -> String {
    let mut all = String::new();
    for entry in fs::read_dir(codex_dir()).expect("agent_codex 폴더") {
        let path = entry.expect("항목").path();
        if path.extension().is_some_and(|e| e == "md") {
            all.push_str(&fs::read_to_string(&path).expect("장 읽기"));
            all.push('\n');
        }
    }
    all
}

#[test]
fn every_capability_command_has_a_codex_chapter() {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("capabilities")
        .output()
        .expect("rhwp 실행");
    let caps: serde_json::Value = serde_json::from_slice(&out.stdout).expect("capabilities 봉투");
    let text = codex_text();
    let mut missing = Vec::new();
    for c in caps["commands"].as_array().expect("commands") {
        let name = c["name"].as_str().expect("name");
        if !text.contains(&format!("### `{name}`")) {
            missing.push(name.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "대전에 장이 없는 명령 — python tools/gen_agent_codex.py 로 재생성하라: {missing:?}"
    );
}

#[test]
fn codex_has_handwritten_canon_and_generation_markers() {
    let dir = codex_dir();
    for hand in ["README.md", "00_서문.md", "01_판단트리.md"] {
        assert!(dir.join(hand).exists(), "손글 정본 누락: {hand}");
    }
    // 생성 장은 수기 수정 금지 표지를 가져야 한다 — 표지 없는 생성 장이
    // 생기면 유지보수 규약이 무너진다.
    let mut generated = 0;
    for entry in fs::read_dir(&dir).expect("폴더") {
        let path = entry.expect("항목").path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !name.ends_with(".md")
            || ["README.md", "00_서문.md", "01_판단트리.md"].contains(&name.as_str())
        {
            continue;
        }
        let text = fs::read_to_string(&path).expect("장");
        assert!(
            text.contains("generated: tools/gen_agent_codex.py"),
            "{name}: 생성 표지(generated:) 없음"
        );
        generated += 1;
    }
    assert!(
        generated >= 8,
        "생성 장이 너무 적다({generated}) — 생성기 회귀 의심"
    );
}
