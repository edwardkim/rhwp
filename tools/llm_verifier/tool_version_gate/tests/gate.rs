//! 닫힌 판정: 낡은 도구의 reproduced:true 는 합격이 아니다.

use llm_verifier_tool_version_gate::loader::parse_tsv_line;
use llm_verifier_tool_version_gate::{
    accept_reproduced, gate, gate_against_this_binary, gate_reproduced_true, Reason,
    VERIFIER_BINARY_VERSION,
};
use std::path::PathBuf;

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/golden_decision_table.tsv")
}

#[test]
fn stale_tool_never_accepts_reproduced_true() {
    let cases = [
        ("0.8.3", "0.8.4"),
        ("0.7.15", "0.8.4"),
        ("0.8.4", "1.0.0"),
        ("0.8.4", "0.8.4+git.deadbeef"),
        ("v0.8.4", "0.8.4"),
        ("rhwp 0.8.4", "0.8.4"),
        ("1.0.0-rc.1", "1.0.0"),
        ("devel-20260818", "0.8.4"),
    ];
    for (attest, verify) in cases {
        let d = gate_reproduced_true(attest, verify);
        assert_eq!(d.reason, Reason::StaleTool, "{attest} vs {verify}");
        assert!(!d.accepted, "{attest} vs {verify} must not be accepted");
        assert!(!accept_reproduced(attest, verify, true));
    }
}

#[test]
fn matching_versions_accept_only_reproduced_true() {
    assert!(accept_reproduced("0.8.4", "0.8.4", true));
    assert!(!accept_reproduced("0.8.4", "0.8.4", false));
    assert!(!gate("0.8.4", "0.8.4", None).accepted);
}

#[test]
fn this_binary_is_crate_version_not_rhwp_cli() {
    assert_eq!(VERIFIER_BINARY_VERSION, "0.1.0");
    let stale = gate_against_this_binary("0.8.4", Some(true));
    assert_eq!(stale.reason, Reason::StaleTool);
    assert!(!stale.accepted);
    let fresh = gate_against_this_binary("0.1.0", Some(true));
    assert!(fresh.accepted);
}

#[test]
fn golden_table_matches_gate() {
    let text = std::fs::read_to_string(golden_path()).expect("golden");
    let mut n = 0u32;
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.is_empty() {
            continue;
        }
        let row = parse_tsv_line(line, i + 1).expect(line);
        row.validate_shape().expect(line);
        n += 1;
    }
    assert!(n >= 16, "golden rows {n}");
}
