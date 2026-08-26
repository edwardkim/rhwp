use llm_verifier_verdict_protocol::{
    classify, corpus_dir, list_shard_paths, load_manifest, load_shard_path, load_shards,
    CommandFamily, ExitClass, UniquenessKey,
};
use std::collections::HashSet;

#[test]
fn manifest_matches_on_disk_shards() {
    let dir = corpus_dir();
    let man = load_manifest(&dir.join("manifest.json")).expect("manifest");
    assert_eq!(man.schema_version, "v-proto.1.0");
    assert!(
        man.record_count >= 2000,
        "record_count={}",
        man.record_count
    );
    let paths = list_shard_paths(&dir).expect("shards");
    assert_eq!(paths.len() as u64, man.shard_count);
}

#[test]
fn sampled_shards_have_unique_keys_and_known_exits() {
    let dir = corpus_dir();
    let paths = list_shard_paths(&dir).expect("shards");
    assert!(!paths.is_empty());
    // 앞·가운데·끝 샤드와 7의 배수 샤드를 표본으로 읽는다.
    let mut sample = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        if i == 0 || i + 1 == paths.len() || i == paths.len() / 2 || i % 7 == 0 {
            sample.push(p.clone());
        }
    }
    let recs = load_shards(&sample).expect("sample shards");
    assert!(recs.len() >= 80, "sampled {}", recs.len());

    let mut keys = HashSet::new();
    let mut exits = HashSet::new();
    let mut commands = HashSet::new();
    for rec in &recs {
        assert!(
            keys.insert(UniquenessKey::from_observation(rec)),
            "{}",
            rec.uniqueness_key()
        );
        exits.insert(rec.exit_class);
        commands.insert(rec.command);
        assert!(ExitClass::ALL.contains(&rec.exit_class));
        assert!(CommandFamily::ALL.contains(&rec.command));
        let decision = classify(rec);
        assert_eq!(decision.exit_class, rec.exit_class);
        assert_eq!(decision.command, rec.command);
    }
    assert!(exits.len() >= 3, "exit classes in sample: {exits:?}");
    assert!(commands.len() >= 5, "commands in sample: {commands:?}");
}

#[test]
fn first_shard_roundtrips_serde() {
    let paths = list_shard_paths(&corpus_dir()).expect("shards");
    let shard = load_shard_path(&paths[0]).expect("first shard");
    let bytes = serde_json::to_vec(&shard).unwrap();
    let again: llm_verifier_verdict_protocol::CorpusShard = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(again.records.len(), shard.records.len());
}
