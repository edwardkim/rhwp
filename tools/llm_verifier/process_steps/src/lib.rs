//! V-step: 편집 한 스텝마다 기존 기계 검사를 돌려 과정 보상을 남긴다.
//!
//! 합격/불합격은 기존 rhwp 명령(`verify`, `layout-anomaly`, `info` 의
//! `pageCount`, `edit fill-fields --verify`)의 종료코드와 `--json` 봉투
//! 필드만으로 결정한다. 새 rhwp CLI 를 만들지 않는다.
//! Best-of-N 순위(V-bon)는 여기 없다.

pub mod check;
pub mod envelope;
pub mod exit_class;
pub mod loader;
pub mod reward;
pub mod score;
pub mod step_kind;
pub mod trace;

pub use check::{CheckKind, CheckObservation, CheckVerdict};
pub use envelope::{extract_check_fields, CheckFields};
pub use exit_class::ExitClass;
pub use loader::{
    corpus_dir, list_shard_paths, load_manifest, load_shard_path, load_shards, load_step_bytes,
    CorpusManifest, CorpusShard, LoadError,
};
pub use reward::ProcessReward;
pub use score::{score_check, score_step, scored_reward};
pub use step_kind::StepKind;
pub use trace::{ProcessStep, UniquenessKey};

pub const SCHEMA_VERSION: &str = "v-step.1.0";
