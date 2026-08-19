//! 고유 축: 버전 불일치 + reproduced:true → accepted:false.

use llm_verifier_tool_version_gate::{gate, Reason};

#[test]
fn reproduced_true_does_not_override_stale_binary() {
    let d = gate("0.8.3", "0.8.4", Some(true));
    assert_eq!(d.reproduced, Some(true));
    assert!(!d.accepted);
    assert_eq!(d.reason, Reason::StaleTool);
}

#[test]
fn same_version_rerun_is_not_this_axis() {
    let same = gate("0.8.4", "0.8.4", Some(true));
    assert!(same.accepted);
    assert_eq!(same.reason, Reason::FreshReproduced);
    let drifted = gate("0.8.4", "0.8.5", Some(true));
    assert!(!drifted.accepted);
    assert_eq!(drifted.reason, Reason::StaleTool);
}

#[test]
fn identity_is_exact_string_after_trim() {
    assert!(gate("0.8.4 ", "0.8.4", Some(true)).accepted);
    assert!(gate("0.8.4\n", "0.8.4", Some(true)).accepted);
    let build = gate("0.8.4+1", "0.8.4+2", Some(true));
    assert_eq!(build.reason, Reason::StaleTool);
    assert!(!build.accepted);
}
