//! LLM-as-verifier 단일 프로토콜.
//!
//! 합격/불합격은 기존 rhwp 종료코드(0/1/2/3/4)와 `--json` 봉투 필드만으로
//! 결정한다. 새 rhwp CLI 를 만들지 않는다.

pub mod classify;
pub mod command;
pub mod exit_class;
pub mod extract;
pub mod judgment;
pub mod loader;
pub mod observation;
pub mod schema;

pub use classify::{classify, MachineVerdict, ProtocolDecision};
pub use command::CommandFamily;
pub use exit_class::ExitClass;
pub use extract::extract_judgment;
pub use judgment::{JudgmentFields, Reproduced, VerifyBlock};
pub use loader::{
    corpus_dir, list_shard_paths, load_manifest, load_observation_bytes, load_observation_path,
    load_shard_path, load_shards, CorpusManifest, CorpusShard, LoadError,
};
pub use observation::{Observation, UniquenessKey};
pub use schema::PROTOCOL_SCHEMA_VERSION;
