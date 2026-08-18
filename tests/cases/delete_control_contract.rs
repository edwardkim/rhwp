//! [#5041] `edit delete-control` 계약.
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
        "rhwp-delctrl-{tag}-{}-{}.hwp",
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

fn add_bookmark(src: &str, name: &str, dest: &Path) {
    let add = Command::new(rhwp_bin())
        .args([
            "edit",
            "add-bookmark",
            src,
            "--name",
            name,
            "--offset",
            "0",
            "-o",
            dest.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(add.status.code(), Some(0), "{:?}", add);
}

fn find_bookmark(listed: &serde_json::Value, name: &str) -> (String, String, String) {
    let bm = listed["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["name"] == name)
        .unwrap_or_else(|| panic!("책갈피 {name}"));
    (
        bm["sec"].as_u64().unwrap().to_string(),
        bm["para"].as_u64().unwrap().to_string(),
        bm["ctrlIdx"].as_u64().unwrap().to_string(),
    )
}

#[test]
fn delete_control_removes_bookmark() {
    let src = sample();
    let inserted = temp("ins");
    add_bookmark(&src, "rhwp-del-ctrl", &inserted);
    let (sec, para, ctrl) = find_bookmark(&bookmarks_json(&inserted), "rhwp-del-ctrl");
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-control",
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], sec.parse::<u64>().unwrap());
    assert_eq!(v["paragraph"], para.parse::<u64>().unwrap());
    assert_eq!(v["ctrl"], ctrl.parse::<u64>().unwrap());
    let after = bookmarks_json(&out);
    assert!(after["bookmarks"]
        .as_array()
        .unwrap()
        .iter()
        .all(|b| b["name"] != "rhwp-del-ctrl"));
    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let inserted = temp("insdry");
    add_bookmark(&src, "rhwp-del-ctrl-dry", &inserted);
    let (sec, para, ctrl) = find_bookmark(&bookmarks_json(&inserted), "rhwp-del-ctrl-dry");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-control",
            inserted.to_str().unwrap(),
            "--section",
            &sec,
            "--para",
            &para,
            "--ctrl",
            &ctrl,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert!(v.get("section").is_some());
    assert!(v.get("paragraph").is_some());
    assert!(v.get("ctrl").is_some());
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-control",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let output = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_delete_control"));
}
