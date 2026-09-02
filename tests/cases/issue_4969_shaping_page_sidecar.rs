//! Issue #4969 W10-Q2-D0: page-local shaping decisions are bounded and unpublished.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_context.rs"]
mod shaping_context;
#[path = "../../src/renderer/shaping_publication.rs"]
mod shaping_publication;

// Product symbols stay crate-private. This source integration case includes
// kerning.rs directly, so mirror only the paint surface that module consumes.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
}

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use rhwp::renderer::render_tree::PageRenderTree;
use shaping::{ShapingAttemptTrace, ShapingRejectReason, TerminalShapingDisposition};
use shaping_context::{HorizontalShapingContext, HorizontalShapingRequest};
use shaping_publication::{
    HorizontalShapingPageSidecars, HorizontalShapingRunDecision, HorizontalShapingRunRange,
    HorizontalShapingSidecarRejectReason, MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const SOURCE_HAN_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};

fn old_hangul_range() -> HorizontalShapingRunRange {
    HorizontalShapingRunRange {
        scalar_start: 0,
        scalar_end: 4,
        utf8_start: 0,
        utf8_end: 12,
        utf16_start: 0,
        utf16_end: 4,
    }
}

fn rejected_trace(attempt_id: u32) -> ShapingAttemptTrace {
    ShapingAttemptTrace {
        attempt_id,
        disposition: TerminalShapingDisposition::Unsupported,
        reason: Some(ShapingRejectReason::SourceUnavailable),
        settings_sha256: None,
        font_source_sha256: None,
        glyph_count: 0,
    }
}

fn applied_decision() -> Arc<HorizontalShapingRunDecision> {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            SOURCE_HAN_SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register Source Han exact source");
    let context = HorizontalShapingContext::new(registry);
    let outcome = context
        .transaction()
        .shadow_measure(&HorizontalShapingRequest {
            attempt_id: 1,
            slot: SOURCE_HAN_SLOT,
            text: "ᄒᆞᆫ글",
            effective_font_size_px: 10.0,
            width_ratio: 0.8,
            script: Some("Hang"),
            language: Some("ko"),
            features: &[],
            variations: &[],
        });
    let measurement = outcome.measurement.expect("old Hangul measurement");
    Arc::new(HorizontalShapingRunDecision::applied(
        old_hangul_range(),
        outcome.trace,
        measurement,
    ))
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d0_attach_and_clone_share_the_owned_measurement() {
    let decision = applied_decision();
    let measurement = Arc::clone(decision.measurement().expect("applied measurement"));
    let mut sidecars = HorizontalShapingPageSidecars::default();

    sidecars
        .attach(17, old_hangul_range(), Arc::clone(&decision))
        .expect("attach applied sidecar");
    assert_eq!(sidecars.len(), 1);
    assert_eq!(
        sidecars.registry_generation(),
        Some(decision.registry_generation())
    );
    assert!(Arc::ptr_eq(
        sidecars.get(17).expect("node sidecar"),
        &decision
    ));
    assert!(Arc::ptr_eq(
        sidecars
            .get(17)
            .and_then(|owned| owned.measurement())
            .expect("owned measurement"),
        &measurement
    ));

    let cloned = sidecars.clone();
    assert!(Arc::ptr_eq(
        cloned.get(17).expect("cloned node sidecar"),
        &decision
    ));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d0_range_duplicate_and_generation_mismatches_fail_closed() {
    let decision = applied_decision();
    let generation = decision.registry_generation();
    let mut sidecars = HorizontalShapingPageSidecars::default();
    let mut wrong_range = old_hangul_range();
    wrong_range.utf16_end += 1;

    assert_eq!(
        sidecars.attach(1, wrong_range, Arc::clone(&decision)),
        Err(HorizontalShapingSidecarRejectReason::RangeMismatch)
    );
    assert!(sidecars.is_empty());
    sidecars
        .attach(1, old_hangul_range(), Arc::clone(&decision))
        .expect("first node attach");
    assert_eq!(
        sidecars.attach(1, old_hangul_range(), Arc::clone(&decision)),
        Err(HorizontalShapingSidecarRejectReason::DuplicateNode)
    );

    let stale = Arc::new(HorizontalShapingRunDecision::rejected(
        generation + 1,
        old_hangul_range(),
        rejected_trace(2),
    ));
    assert_eq!(
        sidecars.attach(2, old_hangul_range(), stale),
        Err(HorizontalShapingSidecarRejectReason::StaleRegistryGeneration)
    );
    assert_eq!(sidecars.len(), 1);

    let mut wrong_trace = decision.trace().clone();
    wrong_trace.settings_sha256 = Some("not-the-owned-settings".into());
    let wrong_identity = Arc::new(HorizontalShapingRunDecision::applied(
        old_hangul_range(),
        wrong_trace,
        Arc::clone(decision.measurement().expect("applied measurement")),
    ));
    let mut identity_sidecars = HorizontalShapingPageSidecars::default();
    assert_eq!(
        identity_sidecars.attach(2, old_hangul_range(), wrong_identity),
        Err(HorizontalShapingSidecarRejectReason::AttemptIdentityMismatch)
    );

    let mut rejected_sidecars = HorizontalShapingPageSidecars::default();
    let rejected = Arc::new(HorizontalShapingRunDecision::rejected(
        generation,
        old_hangul_range(),
        rejected_trace(3),
    ));
    rejected_sidecars
        .attach(3, old_hangul_range(), Arc::clone(&rejected))
        .expect("attach terminal rejection");
    assert!(rejected.measurement().is_none());
    assert_eq!(
        rejected.trace().disposition,
        TerminalShapingDisposition::Unsupported
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d0_page_table_enforces_the_hard_entry_limit() {
    assert_eq!(MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS, 4_096);
    let decision = applied_decision();
    let mut sidecars = HorizontalShapingPageSidecars::default();
    for node_id in 1..=MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS as u32 {
        sidecars
            .attach(node_id, old_hangul_range(), Arc::clone(&decision))
            .expect("bounded node attach");
    }
    assert_eq!(
        sidecars.attach(
            MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS as u32 + 1,
            old_hangul_range(),
            decision,
        ),
        Err(HorizontalShapingSidecarRejectReason::EntryLimitExceeded)
    );
    assert_eq!(sidecars.len(), MAX_HORIZONTAL_SHAPING_PAGE_SIDECARS);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d0_default_render_tree_json_contract_is_unchanged() {
    let tree = PageRenderTree::new(7, 793.7, 1122.5);
    let cloned = tree.clone();
    let serialized = serde_json::to_string(&tree).expect("serialize page render tree");

    assert_eq!(serialized, serde_json::to_string(&cloned).unwrap());
    assert_eq!(tree.root.to_json(), cloned.root.to_json());
    assert!(!serialized.contains("horizontalShaping"));
    assert!(!tree.root.to_json().contains("horizontalShaping"));
}
