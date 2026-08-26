//! 코퍼스 전수: 유일 4열, 재계산, STALE_TOOL 합격 금지.

use llm_verifier_tool_version_gate::loader::{corpus_dir, load_manifest, load_shards};
use llm_verifier_tool_version_gate::row::blob_has_forbidden_key;
use llm_verifier_tool_version_gate::schema::{
    CLAIM_ID, KIND, PROTOCOL_SCHEMA_VERSION, TUPLE_FIELDS, UNIQUENESS,
};
use llm_verifier_tool_version_gate::Reason;
use std::collections::HashSet;

const PADDING_MARKERS: &[&str] = &[
    "lorem",
    "ipsum",
    "asdf",
    "qwerty",
    "padding",
    "foo bar",
    "xxx",
    "placeholder text",
];

#[test]
fn manifest_matches_axis_and_size_floor() {
    let man = load_manifest(&corpus_dir()).expect("manifest");
    assert_eq!(man.schema_version, PROTOCOL_SCHEMA_VERSION);
    assert_eq!(man.claim, CLAIM_ID);
    assert_eq!(man.kind, KIND);
    assert_eq!(man.uniqueness, UNIQUENESS);
    assert_eq!(man.tuple_fields, TUPLE_FIELDS);
    assert!(
        man.record_count >= 100_000,
        "record_count {} < 100000",
        man.record_count
    );
    assert!(man.min_line_floor >= 100_000);
    assert!(man.shard_count >= 8);
    assert!(man.accepted_count > 0);
    assert!(man.rejected_count > 0);
    assert!(
        man.stale_tool_count > 10_000,
        "stale_tool_count {}",
        man.stale_tool_count
    );
    assert_eq!(man.accepted_count + man.rejected_count, man.record_count);
}

#[test]
fn every_row_is_distinct_and_matches_gate() {
    let dir = corpus_dir();
    let man = load_manifest(&dir).expect("manifest");
    let rows = load_shards(&dir).expect("shards");
    assert_eq!(rows.len() as u64, man.record_count);
    assert!(rows.len() >= 100_000);

    let mut keys = HashSet::new();
    let mut ids = HashSet::new();
    let mut stale_accepted = 0u64;
    let mut accepted = 0u64;
    let mut stale = 0u64;

    for row in &rows {
        assert!(
            ids.insert(row.record_id.clone()),
            "dup id {}",
            row.record_id
        );
        let key = row.uniqueness().as_string();
        assert_eq!(row.uniqueness_key, key);
        assert!(keys.insert(key.clone()), "dup key {key}");

        let blob = format!(
            "{} {} {}",
            row.attest_version, row.verify_version, row.family
        )
        .to_ascii_lowercase();
        for marker in PADDING_MARKERS {
            assert!(
                !blob.contains(marker),
                "{} looks like padding",
                row.record_id
            );
        }

        let raw = serde_json::to_value(row).expect("json");
        if let Some(hit) = blob_has_forbidden_key(&raw) {
            panic!("{} has forbidden key {hit}", row.record_id);
        }

        let got = row.recompute();
        assert_eq!(got.accepted, row.accepted, "{}", row.record_id);
        assert_eq!(got.reason, row.reason, "{}", row.record_id);
        if row.reason == Reason::StaleTool {
            stale += 1;
            assert_eq!(row.reproduced, Some(true));
            assert!(!row.accepted);
            if row.accepted {
                stale_accepted += 1;
            }
        }
        if row.accepted {
            accepted += 1;
            assert_eq!(row.reason, Reason::FreshReproduced);
        }
    }

    assert_eq!(stale_accepted, 0);
    assert_eq!(stale, man.stale_tool_count);
    assert_eq!(accepted, man.accepted_count);
    assert_eq!(keys.len(), rows.len());
}
