//! [R36] capabilities `recordFields` 합집합 ⊆ 지식지도 §2-2 전수 사전 — 드리프트 래칫.
//!
//! 지식지도의 완결 기준은 "매니페스트가 내는 필드 = 가이드가 설명하는 필드"인데,
//! 이를 대조하는 가드가 없어 새 명령이 필드를 들고 와도 사전이 조용히 뒤처졌다
//! (이 가드 착지 시점에 13개 필드가 이미 새고 있었다). 방향은 **부분집합**이다 —
//! 사전은 세션 도구(`docId`)와 실측 전용 필드(`assertions`·`preview`,
//! recordFields 밖임이 본문에 명시됨)를 의도적으로 더 실으므로 등호가 아니라
//! "표면이 선언하는 필드는 사전에 반드시 등재된다"를 요구한다.
//!
//! 파싱 계약 — 문서 형식이 바뀌면 이 규칙을 같이 고친다:
//! - 대상 절: `### 2-2.` 헤딩부터 다음 `### ` 헤딩 직전까지.
//! - 필드 행: `| ` + 백틱으로 시작하는 행의 **첫 칸**에서 백틱 토큰 전부 —
//!   `` `a` / `b` `` 병기 행은 두 필드로 센다.
//! - 헤딩의 "N개 필드"는 유니크 토큰 수와 일치해야 한다(자기일관).
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const MAP: &str = "mydocs/manual/agent_knowledge_map.md";

fn dictionary_section() -> String {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(MAP))
        .expect("지식지도를 읽을 수 없다");
    let start = text.find("### 2-2.").expect("§2-2 헤딩이 없다");
    let rest = &text[start..];
    let end = rest[8..]
        .find("\n### ")
        .map(|i| i + 8)
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

/// 표 행 첫 칸의 백틱 토큰들. `| `a` / `b` | ...` → ["a", "b"].
fn dictionary_fields(section: &str) -> Vec<String> {
    let mut fields = Vec::new();
    for line in section.lines() {
        if !line.starts_with("| `") {
            continue;
        }
        let first_cell = line[2..].split(" | ").next().unwrap_or("");
        let mut in_tick = false;
        let mut token = String::new();
        for ch in first_cell.chars() {
            match (ch, in_tick) {
                ('`', false) => in_tick = true,
                ('`', true) => {
                    fields.push(std::mem::take(&mut token));
                    in_tick = false;
                }
                (_, true) => token.push(ch),
                _ => {}
            }
        }
    }
    fields
}

fn record_fields_union() -> BTreeSet<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("capabilities")
        .output()
        .expect("rhwp capabilities 실행 실패");
    let cap: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("capabilities 봉투가 JSON 이 아니다");
    let mut union = BTreeSet::new();
    for c in cap["commands"].as_array().expect("commands") {
        if c["json"] != serde_json::Value::Bool(true) {
            continue;
        }
        for f in c["recordFields"].as_array().into_iter().flatten() {
            if let Some(f) = f.as_str() {
                union.insert(f.to_string());
            }
        }
    }
    union
}

#[test]
fn every_declared_record_field_is_in_the_dictionary() {
    let section = dictionary_section();
    let dict: BTreeSet<String> = dictionary_fields(&section).into_iter().collect();
    let missing: Vec<String> = record_fields_union()
        .into_iter()
        .filter(|f| !dict.contains(f))
        .collect();
    assert!(
        missing.is_empty(),
        "capabilities 가 선언하는데 §2-2 사전에 없는 필드 {}개: {}\n\
         새 명령이 필드를 들고 왔다 — 지식지도 §2-2 해당 소절에 행을 추가하고 \
         헤딩의 필드 수를 갱신하세요.",
        missing.len(),
        missing.join(", "),
    );
}

#[test]
fn dictionary_heading_count_matches_rows() {
    let section = dictionary_section();
    let unique: BTreeSet<String> = dictionary_fields(&section).into_iter().collect();
    let heading: usize = section
        .lines()
        .next()
        .and_then(|h| {
            h.split('—')
                .nth(1)?
                .trim()
                .strip_suffix("개 필드")?
                .trim()
                .parse()
                .ok()
        })
        .expect("§2-2 헤딩에서 'N개 필드' 를 읽지 못했다");
    assert_eq!(
        heading,
        unique.len(),
        "§2-2 헤딩은 {}개라는데 표의 유니크 필드는 {}개다 — 행을 넣고 빼면 헤딩도 같이 고친다",
        heading,
        unique.len(),
    );
}
