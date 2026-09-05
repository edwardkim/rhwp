#![cfg(not(target_arch = "wasm32"))]

//! [#5890] raw 캐시 인벤토리 문서와 소스의 일치.
//!
//! `mydocs/eng/tech/raw_cache_inventory.md` 는 #5890 완료 정의의 절반 —
//! "남아 있는 raw 캐시의 목록(필드별 해석 상태)이 문서화된다" — 을 담는다.
//! 문서는 손으로 유지하면 조용히 낡으므로, `src/model/**` 의 `pub raw_*` 필드
//! 전수와 문서 표를 기계로 맞춘다. 필드를 더하거나 지우면 이 시험이 깨져
//! 문서 갱신을 강제한다.

use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `src/model/*.rs` 의 `pub raw_*` 필드를 `이름@파일` 별 **개수**로 모은다.
///
/// 라인 번호는 대조에 쓰지 않는다 — 문서에는 위치를 적어 두되, 주석 한 줄 추가처럼
/// 무관한 편집으로 시험이 깨지면 가드가 소음이 된다. 대신 같은 파일 안 동명 필드가
/// 여럿이므로(`style.rs` 의 `raw_data` 8개) 개수까지 세어 누락·삭제를 잡는다.
fn fields_in_source() -> BTreeMap<String, usize> {
    let dir = repo().join("src/model");
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    let entries = std::fs::read_dir(&dir).expect("src/model 디렉터리");
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let file = path
            .file_name()
            .and_then(|f| f.to_str())
            .expect("파일명")
            .to_string();
        let text = std::fs::read_to_string(&path).expect("모델 파일 읽기");
        for (i, line) in text.lines().enumerate() {
            let t = line.trim_start();
            if !t.starts_with("pub raw_") {
                continue;
            }
            // `pub raw_foo: Type,` 에서 이름만.
            let after_pub = &t["pub ".len()..];
            let name = match after_pub.find(':') {
                Some(p) => after_pub[..p].trim(),
                None => continue,
            };
            if name.is_empty() || name.contains(' ') {
                continue;
            }
            let _ = i;
            *out.entry(format!("{name}@{file}")).or_insert(0) += 1;
        }
    }
    out
}

/// 문서 표의 행에서 같은 형태를 모은다 — `| name | mod.rs:line | …` 의 앞 두 칸.
fn fields_in_doc() -> BTreeMap<String, usize> {
    let path = repo().join("mydocs/eng/tech/raw_cache_inventory.md");
    let text = std::fs::read_to_string(&path).expect("인벤토리 문서");
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(|c| c.trim()).collect();
        if cells.len() < 2 {
            continue;
        }
        let name = cells[0].trim_matches('`');
        let loc = cells[1].trim_matches('`');
        if !name.starts_with("raw_") || !loc.ends_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let file = loc.split(':').next().unwrap_or(loc);
        *out.entry(format!("{name}@{file}")).or_insert(0) += 1;
    }
    out
}

#[test]
fn raw_cache_inventory_matches_model_fields() {
    let src = fields_in_source();
    let doc = fields_in_doc();
    assert!(
        !src.is_empty(),
        "소스에서 pub raw_* 를 하나도 못 찾았다 — 수집기가 깨졌다"
    );

    let missing: Vec<String> = src
        .iter()
        .filter(|(k, n)| doc.get(*k).copied().unwrap_or(0) < **n)
        .map(|(k, n)| {
            format!(
                "{k} (소스 {n}개, 문서 {}개)",
                doc.get(k).copied().unwrap_or(0)
            )
        })
        .collect();
    let stale: Vec<String> = doc
        .iter()
        .filter(|(k, n)| src.get(*k).copied().unwrap_or(0) < **n)
        .map(|(k, n)| {
            format!(
                "{k} (문서 {n}개, 소스 {}개)",
                src.get(k).copied().unwrap_or(0)
            )
        })
        .collect();

    assert!(
        missing.is_empty(),
        "인벤토리 문서에 없는 raw 필드가 있다 — mydocs/eng/tech/raw_cache_inventory.md 의 표에 추가하라: {missing:?}"
    );
    assert!(
        stale.is_empty(),
        "문서에만 있고 소스에 없는 raw 필드가 있다 — 표에서 지우거나 위치를 맞춰라: {stale:?}"
    );
}
