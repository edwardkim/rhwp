//! [#5027] `edit delete-bookmark` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-delbm-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn bookmarks_json(path: &Path) -> serde_json::Value {
    let out = Command::new(rhwp_bin())
        .args(["bookmarks", path.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{:?}", out);
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn delete_bookmark_removes_name() {
    let src = sample();
    let inserted = temp("ins");
    let add = Command::new(rhwp_bin())
        .args([
            "edit",
            "add-bookmark",
            src.as_str(),
            "--name",
            "rhwp-del-bm",
            "--offset",
            "0",
            "-o",
            inserted.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(add.status.code(), Some(0), "{:?}", add);
    let listed = bookmarks_json(&inserted);
    let bm = listed["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["name"] == "rhwp-del-bm")
        .expect("추가한 책갈피");
    let sec = bm["sec"].as_u64().unwrap().to_string();
    let para = bm["para"].as_u64().unwrap().to_string();
    let ctrl = bm["ctrlIdx"].as_u64().unwrap().to_string();
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-bookmark",
            inserted.to_str().unwrap(),
            "--section",
            &sec,
            "--para",
            &para,
            "--ctrl",
            &ctrl,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let after = bookmarks_json(&out);
    assert!(after["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|b| b["name"] != "rhwp-del-bm"));
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let inserted = temp("insdry");
    let add = Command::new(rhwp_bin())
        .args([
            "edit",
            "add-bookmark",
            src.as_str(),
            "--name",
            "rhwp-del-bm-dry",
            "-o",
            inserted.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(add.status.code(), Some(0), "{:?}", add);
    let listed = bookmarks_json(&inserted);
    let bm = listed["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["name"] == "rhwp-del-bm-dry")
        .expect("추가한 책갈피");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-bookmark",
            inserted.to_str().unwrap(),
            "--section",
            &bm["sec"].as_u64().unwrap().to_string(),
            "--para",
            &bm["para"].as_u64().unwrap().to_string(),
            "--ctrl",
            &bm["ctrlIdx"].as_u64().unwrap().to_string(),
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let _ = std::fs::remove_file(&inserted);
}
