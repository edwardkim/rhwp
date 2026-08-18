use llm_verifier_process_steps::{
    corpus_dir, list_shard_paths, load_manifest, load_shard_path, load_shards, score_step,
    scored_reward, CheckKind, ExitClass, StepKind, UniquenessKey, SCHEMA_VERSION,
};
use std::collections::HashSet;

#[test]
fn manifest_matches_on_disk_shards() {
    let dir = corpus_dir();
    let man = load_manifest(&dir.join("manifest.json")).expect("manifest");
    assert_eq!(man.schema_version, SCHEMA_VERSION);
    assert!(
        man.record_count >= 2000,
        "record_count={}",
        man.record_count
    );
    let paths = list_shard_paths(&dir).expect("shards");
    assert_eq!(paths.len() as u64, man.shard_count);
}

#[test]
fn sampled_shards_are_unique_and_rewards_recompute() {
    let dir = corpus_dir();
    let paths = list_shard_paths(&dir).expect("shards");
    assert!(!paths.is_empty());
    let mut sample = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        if i == 0 || i + 1 == paths.len() || i == paths.len() / 2 || i % 5 == 0 {
            sample.push(p.clone());
        }
    }
    let recs = load_shards(&sample).expect("sample shards");
    assert!(recs.len() >= 80, "sampled {}", recs.len());

    let mut keys = HashSet::new();
    let mut kinds = HashSet::new();
    let mut checks_seen = HashSet::new();
    let mut pass_n = 0u64;
    let mut fail_n = 0u64;
    for rec in &recs {
        assert!(
            keys.insert(UniquenessKey::from_step(rec)),
            "{}",
            rec.uniqueness_key()
        );
        kinds.insert(rec.step_kind);
        assert!(StepKind::ALL.contains(&rec.step_kind));
        assert!(ExitClass::ALL.contains(&rec.edit_exit_class));
        assert_eq!(rec.checks.len(), 4, "{}", rec.record_id);
        for c in &rec.checks {
            checks_seen.insert(c.check);
            assert!(CheckKind::ALL.contains(&c.check));
        }
        assert!(
            scored_reward(&rec.checks, &rec.process_reward),
            "reward mismatch {}",
            rec.record_id
        );
        let computed = score_step(&rec.checks);
        assert_eq!(computed.pass, rec.process_reward.pass);
        if rec.process_reward.pass {
            pass_n += 1;
        } else {
            fail_n += 1;
        }
    }
    assert_eq!(checks_seen.len(), 4, "missing check kinds: {checks_seen:?}");
    assert!(kinds.len() >= 8, "step kinds in sample: {kinds:?}");
    assert!(pass_n >= 1 && fail_n >= 1, "pass={pass_n} fail={fail_n}");
}

#[test]
fn first_shard_roundtrips_serde() {
    let paths = list_shard_paths(&corpus_dir()).expect("shards");
    let shard = load_shard_path(&paths[0]).expect("first shard");
    let bytes = serde_json::to_vec(&shard).unwrap();
    let again: llm_verifier_process_steps::CorpusShard = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(again.records.len(), shard.records.len());
}

#[test]
fn no_best_of_n_fields_in_sample() {
    let paths = list_shard_paths(&corpus_dir()).expect("shards");
    let raw = std::fs::read_to_string(&paths[0]).unwrap();
    assert!(!raw.contains("bestOfN"));
    assert!(!raw.contains("\"rank\""));
    assert!(!raw.contains("candidateRank"));
}
