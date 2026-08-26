//! LLM-as-verifier axis 3: CLAIM ↔ document coordinate bind.
//!
//! 자연어 주장 한 줄마다 기존 `search` / `extract-data` 봉투가 준
//! `section` · `paragraph` · `page` · `charOffset` 가 있어야 통과한다.
//! 좌표가 없거나 불완전하거나 봉투 밖 키를 지어내면 실패다.
//!
//! 새 rhwp CLI 를 만들지 않는다. 전략가·출처 스킬을 다시 쓰지 않는다.

pub mod bind;
pub mod claim;
pub mod coords;
pub mod envelope;
pub mod loader;
pub mod row;
pub mod schema;
pub mod verdict;

pub use bind::{bind_claim, bind_claim_to_envelope, bind_row};
pub use claim::NaturalClaim;
pub use coords::{
    field_set_of, DocumentCoords, ALLOWED_COORD_FIELDS, INVENTED_COORD_FIELDS,
    REQUIRED_COORD_FIELDS,
};
pub use envelope::{EnvelopeHit, EnvelopeKind, SearchExtractEnvelope};
pub use loader::{
    corpus_dir, load_manifest, load_shard_rows, load_shards, CorpusManifest, LoadError, ShardMeta,
};
pub use row::ClaimBindRow;
pub use schema::{
    BIND_SCHEMA_VERSION, CLAIM_BIND_ROW_SCHEMA, COORD_BIND_SCHEMA, SEARCH_EXTRACT_ENVELOPE_SCHEMA,
};
pub use verdict::{BindDecision, FailKind, Verdict};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_axis_three() {
        assert_eq!(BIND_SCHEMA_VERSION, "v-bind.1.0");
    }

    #[test]
    fn required_coord_fields_are_the_four_envelope_keys() {
        assert_eq!(
            REQUIRED_COORD_FIELDS,
            ["section", "paragraph", "page", "charOffset"]
        );
    }
}
