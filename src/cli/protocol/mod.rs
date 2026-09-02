//! Agent work protocol CLI adapters.
//!
//! #5511 P1 moves replayable capsule, trust, exchange, harness, and plan
//! execution responsibilities out of the process root without changing their
//! observable contracts.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::cli::commands::edit::parse_field_key;
use crate::cli::commands::edit::runtime::{
    edit_output_format, edit_serialize, edit_verify_report, EditOutputFormat,
};
use crate::cli::integrity::{
    cas_test_mark_checked_and_wait, cas_test_synchronize_before_lock, sha256_hex_of, CasPathLock,
};
use crate::{
    anchor_log, audit_standard, capsule_sign, disclose, lineage_bundle, paths_refer_to_same_file,
    policy_gate, recolor_cell_text_black, resolve_table_cell, settle, CellResolveError, EXIT_OK,
    EXIT_RUNTIME, EXIT_USAGE,
};

mod capsule;
mod exchange;
mod harness;
mod plan;
mod trust;

pub(crate) use capsule::{
    cmd_audit, cmd_keygen, cmd_lineage, cmd_replay, cmd_verify_signature, collect_audit_capsules,
    is_sha256_hex, replay_execute_to_temp, replay_scratch_dir, replay_sha256_hex,
    validated_capsule_plan, with_replay_input_snapshot,
};
pub(crate) use exchange::{cmd_bundle, cmd_disclose, cmd_settle};
pub(crate) use harness::{cmd_harness, cmd_harness_status};
pub(crate) use plan::{cmd_run_plan, run_plan_engine};
pub(crate) use trust::{cmd_anchor, cmd_audit_report, cmd_conformance, cmd_gate, cmd_recall_scope};
