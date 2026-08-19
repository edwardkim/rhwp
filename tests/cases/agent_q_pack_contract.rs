//! rhwp-q-pack 계약. src 에 #[cfg(test)] 없음.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SAMPLE: &str = "samples/form-01.hwp";
const INVENTORY_COMMANDS: &[&str] = &[
    "forms-all",
    "shapes-all",
    "char-overlaps",
    "headers-list",
    "footers-list",
    "footnotes-list",
    "endnotes-list",
    "new-numbers",
    "page-num-ctrls",
    "page-number-pos",
    "column-defs",
    "unknown-ctrls",
    "tables-model",
    "field-ctrls",
    "bookmark-names",
    "treat-as-char",
    "logical-inline",
    "picture-crops",
    "equation-scripts",
    "form-types",
    "hyperlink-hosts",
    "ruby-mains",
    "pagehide-headers",
    "autonumber-nums",
    "index-second-keys",
    "hidden-comment-len",
    "table-rows",
    "table-cells",
    "shape-sizes",
    "header-paras",
    "footer-paras",
    "footnote-paras",
    "endnote-paras",
    "picture-locks",
    "picture-reverse",
    "equation-fonts",
    "form-enabled",
    "field-commands",
    "field-ids",
    "form-sizes",
    "section-defs",
    "caption-tables",
    "ctrl-kinds",
    "page-starts-on",
    "hidden-comment-count",
    "ruby-ratio",
    "char-overlap-len",
    "table-cols",
    "picture-instance",
    "index-first-keys",
];

fn bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp-q-pack")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp-q-pack").to_string())
}

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("rhwp-q-pack")
}

#[test]
fn help_lists_pack_commands() {
    assert_eq!(INVENTORY_COMMANDS.len(), 50);
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    for name in INVENTORY_COMMANDS {
        assert!(text.contains(name), "{name} missing from help:\n{text}");
    }
    assert!(text.contains("volume-probe"));
}

#[test]
fn unknown_command_is_usage() {
    let out = run(&["not-a-command"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn every_inventory_command_emits_a_json_envelope() {
    let p = sample();
    let src = p.to_str().unwrap();
    for command in INVENTORY_COMMANDS {
        let args = [*command, "--json", src];
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "{command}: {:?}", out);
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["tool"], "rhwp-q-pack", "{command}");
        assert_eq!(v["command"], *command, "{command}");
        assert!(v["count"].is_u64(), "{command}");
        let items = v["items"].as_array().expect("items must be an array");

        match *command {
            "caption-tables" => {
                for item in items {
                    assert_eq!(item["hasCaption"], true);
                    assert!(item["captionParagraphs"].is_u64());
                }
            }
            "ctrl-kinds" => {
                assert_eq!(v["count"].as_u64(), Some(items.len() as u64));
                for item in items {
                    assert!(item["kind"].is_string());
                    assert!(item["count"].is_u64());
                }
            }
            "page-starts-on" => {
                for item in items {
                    assert!(matches!(
                        item["pageStartsOn"].as_str(),
                        Some("BOTH" | "EVEN" | "ODD")
                    ));
                }
            }
            _ => {}
        }
    }
}

#[test]
fn every_volume_probe_slot_emits_json() {
    let p = sample();
    let src = p.to_str().unwrap();
    for slot in 0..50_u64 {
        let slot_text = slot.to_string();
        let args = ["volume-probe", "--json", "--slot", &slot_text, src];
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "slot {slot}: {:?}", out);
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["tool"], "rhwp-q-pack");
        assert_eq!(v["command"], "volume-probe");
        assert_eq!(v["slot"].as_u64(), Some(slot));
        assert!(v["acc"].is_number());
    }
}
