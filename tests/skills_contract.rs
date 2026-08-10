//! [#4508] 스킬 표류 가드 — `.claude/skills` 가 참조하는 rhwp 명령의 실재를
//! **테스트 시점의 자기서술**(capabilities --json ∪ --help)로 검증한다.
//!
//! 스킬은 에이전트 유입의 첫 표면이고, 명령이 개명·개편되면 조용히 썩는다 —
//! 죽은 명령을 안내하는 스킬은 유입을 유출로 바꾼다. 실재 명령 집합을 골든
//! 파일로 박제하지 않고 바이너리 자기서술에서 매번 재구성하므로, CLI 가
//! 진화하면 가드의 기준도 함께 진화한다.
//!
//! 검증 깊이: 머리 토큰(`rhwp <토큰>`) + **그룹 하위명령 2단**(`rhwp edit
//! replace-text`). 하위명령의 실명 출처는 `--help` 다 — capabilities 는
//! edit·inspect 를 우산 이름 하나로만 싣는다(실측: 2단 이름 0개).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn rhwp(args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행");
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// 실재 명령 집합 — capabilities(계약) ∪ --help(사람용 목록) ∪ {help}.
///
/// 두 출처를 합치는 이유: capabilities 는 `--json` 계약 명령만 싣고, 도움말은
/// mcp-serve 같은 비봉투 명령까지 싣는다. 스킬은 양쪽을 다 안내할 수 있다.
fn known_commands() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    // `capabilities` 는 단독 호출이 곧 JSON 자기서술이다 (--json 은 --search 전용).
    let caps: serde_json::Value =
        serde_json::from_str(&rhwp(&["capabilities"])).expect("capabilities 봉투");
    for c in caps["commands"].as_array().expect("commands 배열") {
        if let Some(name) = c["name"].as_str() {
            set.insert(name.to_string());
            if let Some(head) = name.split_whitespace().next() {
                set.insert(head.to_string());
            }
        }
    }
    for line in rhwp(&["--help"]).lines() {
        if let Some(rest) = line.strip_prefix("  ") {
            if let Some(tok) = rest.split_whitespace().next() {
                if !tok.is_empty()
                    && tok
                        .chars()
                        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
                {
                    set.insert(tok.to_string());
                }
            }
        }
    }
    set.insert("help".to_string());
    assert!(
        set.len() >= 20,
        "실재 명령 집합이 너무 작다 — 자기서술 파싱 회귀 의심: {set:?}"
    );
    set
}

fn is_token(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && s.chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

/// 그룹 명령의 하위명령 실명 — `--help` 의 "  <머리> <하위> …" 줄에서 수확한다.
///
/// 두 표기 모두 거둔다: 실토큰 줄("edit replace-text …")과 대안 나열
/// 플레이스홀더("batch <export-text|info|…>"). 후자를 안 거두면 batch 가
/// `batch fill` 줄 하나 때문에 {fill}만 가진 그룹으로 등록되어, 정당한
/// `rhwp batch info` 참조가 오탐된다(개발 중 실측).
fn group_subcommands() -> BTreeMap<String, BTreeSet<String>> {
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in rhwp(&["--help"]).lines() {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        let mut it = rest.split_whitespace();
        let (Some(head), Some(sub)) = (it.next(), it.next()) else {
            continue;
        };
        if !is_token(head) {
            continue;
        }
        if is_token(sub) {
            map.entry(head.to_string())
                .or_default()
                .insert(sub.to_string());
        } else if sub.starts_with('<') && sub.contains('|') {
            // `batch <export-text|info|…>` — 대안 나열 플레이스홀더도 하위명령
            // 실명이다. 이것을 안 거두면 batch 가 {fill}만 가진 그룹이 되어
            // 정당한 `rhwp batch info` 참조가 오탐된다(실측).
            for alt in sub.trim_matches(|c| c == '<' || c == '>').split('|') {
                if is_token(alt) {
                    map.entry(head.to_string())
                        .or_default()
                        .insert(alt.to_string());
                }
            }
        }
    }
    assert!(
        map.get("edit").is_some_and(|s| s.len() >= 4),
        "edit 하위명령 수확 회귀 의심: {map:?}"
    );
    map
}

/// (스킬 이름, SKILL.md 본문) 전수.
fn skill_files() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".claude")
        .join("skills");
    let mut out = Vec::new();
    for entry in fs::read_dir(&root).expect(".claude/skills 읽기") {
        let dir = entry.expect("폴더 항목").path();
        if dir.is_dir() {
            let name = dir.file_name().unwrap().to_string_lossy().into_owned();
            let md = dir.join("SKILL.md");
            let body = fs::read_to_string(&md)
                .unwrap_or_else(|e| panic!("{name}/SKILL.md 읽기 실패: {e}"));
            out.push((name, body));
        }
    }
    out.sort();
    assert!(
        out.len() >= 5,
        "스킬이 너무 적다 — 경로 회귀 의심: {}개",
        out.len()
    );
    out
}

/// 본문에서 `rhwp <토큰> [하위토큰]` 참조를 추출한다.
///
/// 토큰은 소문자 ASCII 로 시작하는 `[a-z0-9-]+` 만 — 한글 조사("rhwp 를"),
/// 플레이스홀더("rhwp <명령>"), 대문자 산문("rhwp CLI")은 참조가 아니다.
/// 하위토큰은 공백 하나 뒤 같은 규칙(단, `-` 시작 플래그 제외)으로만 잡는다.
fn referenced_commands(body: &str) -> Vec<(String, Option<String>)> {
    let pat = "rhwp ";
    let mut refs = Vec::new();
    let mut idx = 0usize;
    while let Some(pos) = body[idx..].find(pat) {
        let start = idx + pos + pat.len();
        let tok: String = body[start..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
            .collect();
        if tok.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            let after = start + tok.len();
            let sub = body[after..].strip_prefix(' ').and_then(|tail| {
                let s: String = tail
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
                    .collect();
                (is_token(&s)).then_some(s)
            });
            refs.push((tok, sub));
        }
        idx = start;
    }
    refs
}

#[test]
fn skills_reference_only_real_commands() {
    let known = known_commands();
    let groups = group_subcommands();
    let mut dead: Vec<String> = Vec::new();
    for (name, body) in skill_files() {
        for (tok, sub) in referenced_commands(&body) {
            if !known.contains(&tok) {
                dead.push(format!("  {name}: `rhwp {tok}`"));
                continue;
            }
            // 2단: 도움말이 하위명령을 선언한 그룹(edit·inspect)에서 하위
            // 토큰이 잡혔다면 그 하위도 실재해야 한다 — `rhwp edit replace`
            // 같은 오기가 여기서 잡힌다.
            if let (Some(subs), Some(sub)) = (groups.get(&tok), sub) {
                if !subs.contains(&sub) {
                    dead.push(format!(
                        "  {name}: `rhwp {tok} {sub}` (실재 하위: {subs:?})"
                    ));
                }
            }
        }
    }
    assert!(
        dead.is_empty(),
        "스킬이 존재하지 않는 명령을 안내한다(표류):\n{}\n실재 명령 집합(자기서술): {:?}",
        dead.join("\n"),
        known
    );
}

#[test]
fn skills_have_valid_frontmatter_and_are_executable() {
    for (name, body) in skill_files() {
        let mut lines = body.lines();
        assert_eq!(
            lines.next(),
            Some("---"),
            "{name}: frontmatter 시작(---) 없음"
        );
        let mut fm_name = None;
        let mut fm_desc = None;
        for line in lines {
            if line == "---" {
                break;
            }
            if let Some(v) = line.strip_prefix("name:") {
                fm_name = Some(v.trim().to_string());
            }
            if let Some(v) = line.strip_prefix("description:") {
                fm_desc = Some(v.trim().to_string());
            }
        }
        assert_eq!(
            fm_name.as_deref(),
            Some(name.as_str()),
            "{name}: frontmatter name 이 폴더명과 다르다"
        );
        let desc_len = fm_desc.unwrap_or_default().chars().count();
        assert!(
            desc_len >= 20,
            "{name}: description 이 너무 짧다({desc_len}자)"
        );
        assert!(
            !referenced_commands(&body).is_empty(),
            "{name}: 실행 가능한 `rhwp <명령>` 참조가 하나도 없다 — 스킬은 안내문이 아니라 실행 규약이다"
        );
    }
}
