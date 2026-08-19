//! LLM-as-verifier axis: toolVersion gate (V-fresh).
//!
//! 영수증 `toolVersion` 이 검증기 바이너리 버전과 다르면
//! `reproduced:true` 를 합격으로 받지 않는다 (낡은 도구).
//! 새 rhwp CLI 를 만들지 않는다.
//! V-replay(같은 버전 재실행) 를 구현하지 않는다.

pub mod gate;
pub mod loader;
pub mod reason;
pub mod row;
pub mod schema;
pub mod version;

pub use gate::{
    accept_reproduced, gate, gate_against_this_binary, gate_reproduced_true, parse_reproduced,
    reproduced_token, Decision,
};
pub use loader::{
    corpus_dir, load_manifest, load_shard_path, load_shards, parse_tsv_line, CorpusManifest,
    LoadError, ShardMeta, TSV_COLUMNS,
};
pub use reason::Reason;
pub use row::{blob_has_forbidden_key, GateRow, UniquenessKey};
pub use schema::{
    CLAIM_ID, DECISION_SCHEMA, FORBIDDEN_KEYS, GATE_ROW_SCHEMA, KIND, PROTOCOL_SCHEMA_VERSION,
    TUPLE_FIELDS, UNIQUENESS,
};
pub use version::ToolVersion;

/// 이 검증기 크레이트의 바이너리 버전. 영수증 `toolVersion` 과 대조한다.
pub const VERIFIER_BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_fresh_axis() {
        assert_eq!(PROTOCOL_SCHEMA_VERSION, "v-fresh.1.0");
        assert_eq!(CLAIM_ID, "V-fresh");
        assert_eq!(KIND, "toolVersionGate");
        assert_eq!(VERIFIER_BINARY_VERSION, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn forbidden_keys_block_other_axes() {
        assert!(FORBIDDEN_KEYS.contains(&"bestOfN"));
        assert!(FORBIDDEN_KEYS.contains(&"holisticScore"));
        assert!(FORBIDDEN_KEYS.contains(&"processReward"));
        assert!(FORBIDDEN_KEYS.contains(&"plan"));
        assert!(FORBIDDEN_KEYS.contains(&"expectSha"));
    }
}
