//! LLM-as-verifier axis: criteria decomposition (V-decomp).
//!
//! 검증을 한 덩어리 점수로 주지 않는다. 기존 rhwp `--json` 봉투 필드마다
//! 원자 기준을 두고, 각 원자가 통과했는지와 총점 채점이 그 실패를
//! 가릴 수 있는지만 본다. 새 rhwp CLI 를 만들지 않는다.
//! V-bon(Best-of-N 순위) 과 V-step(과정 보상) 을 구현하지 않는다.

pub mod atom;
pub mod decomp;
pub mod envelope;
pub mod field;
pub mod holistic;
pub mod loader;
pub mod row;
pub mod schema;
pub mod task;
pub mod verdict;

pub use atom::{AtomSpec, Expected};
pub use decomp::{decompose_bundle, evaluate_atom, evaluate_row};
pub use envelope::{read_field, Observed};
pub use field::{
    is_allowed_envelope_field, parse_envelope_field, EnvelopeField, ALLOWED_ENVELOPE_FIELDS,
    INVENTED_FIELDS,
};
pub use holistic::{holistic_would_hide, HOLISTIC_HIDE_DEN, HOLISTIC_HIDE_NUM};
pub use loader::{
    corpus_dir, load_manifest, load_shard_rows, load_shards, CorpusManifest, LoadError, ShardMeta,
};
pub use row::DecompRow;
pub use schema::{
    ATOMIC_CRITERION_SCHEMA, DECOMP_ROW_SCHEMA, DECOMP_SCHEMA_VERSION, ENVELOPE_ATOM_SCHEMA,
};
pub use task::TaskBundle;
pub use verdict::{AtomVerdict, DecompReport, FailKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_decomp_axis() {
        assert_eq!(DECOMP_SCHEMA_VERSION, "v-decomp.1.0");
    }

    #[test]
    fn allowed_fields_come_from_existing_envelopes() {
        assert!(is_allowed_envelope_field("identical"));
        assert!(is_allowed_envelope_field("filledCount"));
        assert!(is_allowed_envelope_field("verify.identical"));
        assert!(is_allowed_envelope_field("untrustedContent"));
        assert!(!is_allowed_envelope_field("holisticScore"));
        assert!(!is_allowed_envelope_field("bestOfN"));
        assert!(!is_allowed_envelope_field("processReward"));
    }
}
