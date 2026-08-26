//! LLM-as-verifier axis: repeated evaluation (V-repeat).
//!
//! 같은 산출을 K번 기계 검사한다. 종료코드와 기존 `--json` 봉투 필드만
//! 읽고, 범주는 다수결·수치는 평균으로 줄여 분산을 낮춘다.
//! 새 rhwp CLI 를 만들지 않는다.
//! V-bon(후보 순위) 과 V-decomp(기준 분해) 를 구현하지 않는다.

pub mod artifact;
pub mod check;
pub mod command;
pub mod envelope;
pub mod exit_class;
pub mod loader;
pub mod reduce;
pub mod report;
pub mod row;
pub mod schema;
pub mod trial;
pub mod variance;
pub mod vote;

pub use artifact::Artifact;
pub use check::{CheckKind, CheckSpec, ValueKind};
pub use command::CommandFamily;
pub use envelope::{read_path, Observed};
pub use exit_class::ExitClass;
pub use loader::{
    corpus_dir, load_manifest, load_shard_path, load_shards, CorpusManifest, CorpusShard,
    LoadError, ShardMeta,
};
pub use reduce::{reduce_row, reduce_trials};
pub use report::{FinalValue, ReduceKind, ReduceReport};
pub use row::{RepeatRow, UniquenessKey};
pub use schema::{
    CLAIM_ID, FORBIDDEN_KEYS, KIND, PROTOCOL_SCHEMA_VERSION, REDUCE_REPORT_SCHEMA,
    REPEAT_ROW_SCHEMA, TRIAL_SCHEMA,
};
pub use trial::Trial;
pub use variance::VarianceStats;
pub use vote::VoteTally;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_repeat_axis() {
        assert_eq!(PROTOCOL_SCHEMA_VERSION, "v-repeat.1.0");
        assert_eq!(CLAIM_ID, "V-repeat");
        assert_eq!(KIND, "repeatEvaluation");
    }

    #[test]
    fn forbidden_keys_block_other_axes() {
        assert!(FORBIDDEN_KEYS.contains(&"bestOfN"));
        assert!(FORBIDDEN_KEYS.contains(&"holisticScore"));
        assert!(FORBIDDEN_KEYS.contains(&"processReward"));
        assert!(FORBIDDEN_KEYS.contains(&"expectedRank"));
        assert!(FORBIDDEN_KEYS.contains(&"atomPass"));
    }
}
