//! 코퍼스 전수: 유일키, 금지 필드, 축소 재계산.

use llm_verifier_repeat_eval::loader::{corpus_dir, load_manifest, load_shards};
use llm_verifier_repeat_eval::reduce::reduce_row_with_intended;
use llm_verifier_repeat_eval::row::blob_has_forbidden_key;
use llm_verifier_repeat_eval::schema::{CLAIM_ID, PROTOCOL_SCHEMA_VERSION};
use std::collections::HashSet;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs()))
}

#[test]
fn manifest_matches_axis() {
    let dir = corpus_dir();
    let man = load_manifest(&dir).expect("manifest");
    assert_eq!(man.schema_version, PROTOCOL_SCHEMA_VERSION);
    assert_eq!(man.claim, CLAIM_ID);
    assert!(man.record_count >= 900, "record_count={}", man.record_count);
    assert!(man.shard_count >= 10, "shard_count={}", man.shard_count);
    assert_eq!(man.uniqueness, "artifactId|k|check");
}

#[test]
fn uniqueness_and_recompute() {
    let dir = corpus_dir();
    let man = load_manifest(&dir).expect("manifest");
    let rows = load_shards(&dir).expect("shards");
    assert_eq!(rows.len() as u64, man.record_count);

    let mut keys = HashSet::new();
    let mut ids = HashSet::new();
    for row in &rows {
        assert!(
            ids.insert(row.record_id.clone()),
            "dup id {}",
            row.record_id
        );
        let key = row.uniqueness().as_string();
        assert_eq!(row.uniqueness_key, key);
        assert!(keys.insert(key.clone()), "dup key {key}");
        assert!(row.k >= 2);
        assert_eq!(row.k as usize, row.trials.len());

        let raw = serde_json::to_value(row).expect("json");
        if let Some(hit) = blob_has_forbidden_key(&raw) {
            panic!("{} has forbidden key {hit}", row.record_id);
        }

        let got = reduce_row_with_intended(row);
        assert_eq!(got.artifact_id, row.artifact.artifact_id);
        assert_eq!(got.k, row.k);
        assert_eq!(got.check, row.check.name);
        assert_eq!(got.votes.counts, row.votes.counts);
        assert_eq!(got.votes.majority, row.votes.majority);
        assert_eq!(got.votes.plurality, row.votes.plurality);
        assert_eq!(got.votes.is_tie, row.votes.is_tie);
        assert!(
            close(got.votes.majority_frac, row.votes.majority_frac),
            "{} frac {} vs {}",
            row.record_id,
            got.votes.majority_frac,
            row.votes.majority_frac
        );
        assert_eq!(got.variance.n, row.variance.n);
        assert_eq!(got.variance.distinct, row.variance.distinct);
        assert!(
            close(got.variance.disagreement, row.variance.disagreement),
            "{} disagreement {} vs {}",
            row.record_id,
            got.variance.disagreement,
            row.variance.disagreement
        );
        match (got.variance.sample_variance, row.variance.sample_variance) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(close(a, b), "{} var {a} vs {b}", row.record_id),
            other => panic!("{} sample_variance {other:?}", row.record_id),
        }
        match (got.variance.mean, row.variance.mean) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(close(a, b), "{} mean {a} vs {b}", row.record_id),
            other => panic!("{} mean {other:?}", row.record_id),
        }
        assert_eq!(got.final_value.reduce, row.final_value.reduce);
        assert_eq!(got.final_value.value, row.final_value.value);
        assert_eq!(got.final_value.tie, row.final_value.tie);
        assert_eq!(got.final_value.pass, row.final_value.pass);
        match (got.final_value.numeric, row.final_value.numeric) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(close(a, b), "{} numeric {a} vs {b}", row.record_id),
            other => panic!("{} numeric {other:?}", row.record_id),
        }
    }
}

#[test]
fn k_ladder_reuses_prefix_seeds() {
    let rows = load_shards(&corpus_dir()).expect("shards");
    let mut by_art_check: std::collections::BTreeMap<(String, String), Vec<_>> =
        std::collections::BTreeMap::new();
    for row in &rows {
        by_art_check
            .entry((row.artifact.artifact_id.clone(), row.check.name.clone()))
            .or_default()
            .push(row);
    }
    let mut ladders = 0u64;
    for group in by_art_check.values() {
        if group.len() < 2 {
            continue;
        }
        ladders += 1;
        let mut sorted = group.clone();
        sorted.sort_by_key(|r| r.k);
        for w in sorted.windows(2) {
            let a = w[0];
            let b = w[1];
            assert!(b.k > a.k);
            for (i, trial) in a.trials.iter().enumerate() {
                assert_eq!(trial.seed, b.trials[i].seed);
                assert_eq!(trial.exit_class, b.trials[i].exit_class);
                assert_eq!(trial.observed, b.trials[i].observed);
            }
        }
    }
    assert!(ladders >= 20, "expected K ladders, got {ladders}");
}

#[test]
fn no_padding_comments() {
    let dir = corpus_dir().join("shards");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("TODO"));
        assert!(!text.contains("pad"));
        assert!(!text.contains("lorem"));
        for line in text.lines() {
            let t = line.trim();
            assert!(!t.starts_with("//"), "{}", path.display());
        }
    }
}
