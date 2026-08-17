//! `edit delete-text-in-footnote` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/footnote-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-fndeltxt-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 지정한 각주/미주 문단 본문. 비어 있어도 주소를 유지한다.
fn note_text_at(path: &Path, si: usize, pi: usize, ci: usize, fi: usize) -> Option<String> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let ctrl = doc
        .document()
        .sections
        .get(si)?
        .paragraphs
        .get(pi)?
        .controls
        .get(ci)?;
    let paras = match ctrl {
        Control::Footnote(f) => &f.paragraphs,
        Control::Endnote(e) => &e.paragraphs,
        _ => return None,
    };
    paras.get(fi).map(|fp| fp.text.clone())
}

/// 첫 각주/미주 중 비어 있지 않은 문단. (0,0,0) 을 가정하지 않는다.
fn first_note_text(path: &Path) -> Option<(usize, usize, usize, usize, usize, String)> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                let paras = match ctrl {
                    Control::Footnote(f) => &f.paragraphs,
                    Control::Endnote(e) => &e.paragraphs,
                    _ => continue,
                };
                for (fi, fp) in paras.iter().enumerate() {
                    let n = fp.text.chars().count();
                    // 공백만 있는 기본 각주는 저장 왕복에서 다시 채워질 수 있다.
                    if n > 0 && fp.text.chars().any(|c| !c.is_whitespace()) {
                        return Some((si, pi, ci, fi, n, fp.text.clone()));
                    }
                }
            }
        }
    }
    None
}

fn letters(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn delete_text_in_footnote_shortens() {
    let src = sample();
    let (si, pi, ci, fi, before, text) =
        first_note_text(Path::new(&src)).expect("샘플 각주에 본문 글자가 있어야 한다");
    assert!(before >= 1, "삭제할 글자가 있어야 한다: {text:?}");
    // HWP5 저장이 각주 앞뒤 공백을 다시 맞추므로, 공백이 아닌 글자를 지우고
    // 공백을 뺀 본문만 대조한다.
    let offset = text
        .chars()
        .position(|c| !c.is_whitespace())
        .expect("본문 글자");
    let expect = {
        let s = letters(&text);
        s.chars().skip(1).collect::<String>()
    };
    assert!(
        !expect.is_empty(),
        "지운 뒤에도 본문이 남아야 한다: {text:?}"
    );
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text-in-footnote",
            src.as_str(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            &fi.to_string(),
            "--offset",
            &offset.to_string(),
            "--count",
            "1",
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["fnPara"], fi);
    assert_eq!(v["count"], 1);
    let after = note_text_at(&out, si, pi, ci, fi).expect("각주가 남아 있어야 한다");
    assert_eq!(
        letters(&after),
        expect,
        "addr=({si},{pi},{ci},{fi}) offset={offset} text={text:?} after={after:?}"
    );
    HwpDocument::from_bytes(&std::fs::read(&out).unwrap()).expect("산출물 재파싱");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (si, pi, ci, fi, _, _) = first_note_text(Path::new(&src)).expect("샘플 각주");
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text-in-footnote",
            src.as_str(),
            "--section",
            &si.to_string(),
            "--para",
            &pi.to_string(),
            "--ctrl",
            &ci.to_string(),
            "--fn-para",
            &fi.to_string(),
            "--count",
            "1",
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
    assert_eq!(v["count"], 1);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "delete-text-in-footnote",
            src.as_str(),
            "--count",
            "1",
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
        .any(|t| t["name"] == "hwp_delete_text_in_footnote"));
}
