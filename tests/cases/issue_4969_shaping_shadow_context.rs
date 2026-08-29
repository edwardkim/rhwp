//! Issue #4969 W10-Q2-B: exact-source horizontal shadow measurement는 cluster-aware하고 bounded하다.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_context.rs"]
mod shaping_context;

// Product symbols stay crate-private. This source integration case includes
// kerning.rs directly, so mirror only the paint surface that module consumes.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
}

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use shaping::MAX_SHAPING_VARIATION_AXES;
use shaping::{ShapingFeature, ShapingRejectReason, ShapingVariation, TerminalShapingDisposition};
use shaping_context::{
    HorizontalShapingContext, HorizontalShapingInstanceRequestError,
    HorizontalShapingInstanceRequestRegistration, HorizontalShapingInstanceRequestRegistry,
    HorizontalShapingRequest, MAX_HORIZONTAL_SHAPING_CACHE_CLUSTERS,
    MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES, MAX_HORIZONTAL_SHAPING_CACHE_GLYPHS,
    MAX_HORIZONTAL_SHAPING_CACHE_TEXT_BYTES, MAX_HORIZONTAL_SHAPING_CLUSTERS,
    MAX_HORIZONTAL_SHAPING_INSTANCE_REQUESTS,
};
use std::collections::HashSet;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const NOTO: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const HAPPINESS: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");

const NOTO_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 1,
};
const SOURCE_HAN_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};
const HAPPINESS_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 2,
};

fn registry() -> ExactFontSourceRegistry {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            NOTO_SLOT,
            ExactFontSource {
                bytes: NOTO,
                face_index: 0,
            },
        )
        .expect("register Noto exact source");
    registry
        .register(
            SOURCE_HAN_SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register Source Han exact source");
    registry
        .register(
            HAPPINESS_SLOT,
            ExactFontSource {
                bytes: HAPPINESS,
                face_index: 0,
            },
        )
        .expect("register Happiness Sans variable source");
    registry
}

fn request<'a>(
    attempt_id: u32,
    slot: ExactFontSlot,
    text: &'a str,
    script: &'a str,
    language: &'a str,
    features: &'a [ShapingFeature],
) -> HorizontalShapingRequest<'a> {
    HorizontalShapingRequest {
        attempt_id,
        slot,
        text,
        effective_font_size_px: 10.0,
        width_ratio: 0.8,
        script: Some(script),
        language: Some(language),
        features,
        variations: &[],
    }
}

fn variable_request<'a>(
    attempt_id: u32,
    text: &'a str,
    script: &'a str,
    language: &'a str,
    variations: &'a [ShapingVariation],
) -> HorizontalShapingRequest<'a> {
    let mut request = request(attempt_id, HAPPINESS_SLOT, text, script, language, &[]);
    request.variations = variations;
    request
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_a_default_and_title_are_distinct_instance_cache_entries() {
    let defaults = [
        ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 400.0,
        },
    ];
    let title = [
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
    ];
    let context = HorizontalShapingContext::new(registry());
    let mut transaction = context.transaction();
    let empty_default =
        transaction.shadow_measure(&variable_request(20, "가변", "Hang", "ko", &[]));
    let explicit_default =
        transaction.shadow_measure(&variable_request(21, "가변", "Hang", "ko", &defaults));
    let title = transaction.shadow_measure(&variable_request(22, "가변", "Hang", "ko", &title));

    let empty_default = empty_default
        .measurement
        .expect("empty default measurement");
    let explicit_default = explicit_default
        .measurement
        .expect("explicit default measurement");
    let title = title.measurement.expect("title measurement");
    assert!(Arc::ptr_eq(&empty_default, &explicit_default));
    assert!(!Arc::ptr_eq(&empty_default, &title));
    assert_ne!(empty_default.total_advance_px, title.total_advance_px);
    assert_eq!(context.cached_result_count(), 2);
    assert_eq!(transaction.prepared_source_count(), 1);
    assert_eq!(transaction.parsed_face_count(), 2);
}

fn instance_axes(wght: f32, opsz: f32) -> Vec<ShapingVariation> {
    vec![
        ShapingVariation {
            tag: "wght".into(),
            value: wght,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: opsz,
        },
    ]
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_b_public_instance_matrix_is_deterministic_for_hangul_and_latin() {
    let matrix = [
        instance_axes(400.0, 400.0),
        instance_axes(650.0, 400.0),
        instance_axes(900.0, 400.0),
        instance_axes(400.0, 650.0),
        instance_axes(900.0, 900.0),
    ];
    let context = HorizontalShapingContext::new(registry());
    let mut transaction = context.transaction();
    let mut settings = HashSet::new();
    let mut hangul_ids = Vec::new();
    let mut latin_ids = Vec::new();
    let mut hangul_advances = Vec::new();
    let mut latin_advances = Vec::new();

    for (index, axes) in matrix.iter().enumerate() {
        let hangul = transaction.shadow_measure(&variable_request(
            30 + index as u32 * 2,
            "가변",
            "Hang",
            "ko",
            axes,
        ));
        let latin = transaction.shadow_measure(&variable_request(
            31 + index as u32 * 2,
            "Typography",
            "Latn",
            "en",
            axes,
        ));
        let hangul = hangul.measurement.expect("Hangul matrix measurement");
        let latin = latin.measurement.expect("Latin matrix measurement");
        settings.insert(hangul.applied.identity.settings_sha256.clone());
        hangul_ids.push(
            hangul
                .glyphs_px
                .iter()
                .map(|glyph| glyph.glyph_id)
                .collect::<Vec<_>>(),
        );
        latin_ids.push(
            latin
                .glyphs_px
                .iter()
                .map(|glyph| glyph.glyph_id)
                .collect::<Vec<_>>(),
        );
        hangul_advances.push(hangul.total_advance_px);
        latin_advances.push(latin.total_advance_px);
    }

    assert_eq!(settings.len(), matrix.len());
    assert!(hangul_ids.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(latin_ids.windows(2).all(|pair| pair[0] == pair[1]));
    println!("q3-b public matrix Hangul={hangul_advances:?} Latin={latin_advances:?}");
    for (actual, expected) in hangul_advances
        .iter()
        .zip([14.72, 14.72, 14.72, 14.72, 14.88])
    {
        assert!((actual - expected).abs() <= 1.0e-9);
    }
    for (actual, expected) in latin_advances
        .iter()
        .zip([43.752, 44.448, 45.136, 43.752, 47.328])
    {
        assert!((actual - expected).abs() <= 1.0e-9);
    }
    assert_ne!(hangul_advances[0], hangul_advances[4]);
    assert_ne!(latin_advances[0], latin_advances[4]);
    assert_eq!(transaction.prepared_source_count(), 1);
    assert_eq!(transaction.parsed_face_count(), matrix.len());
    assert_eq!(transaction.result_cache_miss_count(), matrix.len() * 2);
    assert_eq!(transaction.result_cache_hit_count(), 0);
    assert_eq!(context.cached_result_count(), matrix.len() * 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_b_instance_and_run_matrix_has_bounded_cache_ownership() {
    let matrix = [
        instance_axes(400.0, 400.0),
        instance_axes(650.0, 400.0),
        instance_axes(900.0, 400.0),
        instance_axes(400.0, 650.0),
        instance_axes(900.0, 900.0),
        instance_axes(650.0, 650.0),
        instance_axes(400.0, 900.0),
        instance_axes(650.0, 900.0),
    ];

    for instance_count in [1_usize, 2, 8] {
        for runs_per_instance in [1_usize, 2, 8] {
            let context = HorizontalShapingContext::new(registry());
            let mut transaction = context.transaction();
            for (instance_index, axes) in matrix.iter().take(instance_count).enumerate() {
                let mut first = None;
                for run_index in 0..runs_per_instance {
                    let outcome = transaction.shadow_measure(&variable_request(
                        100 + (instance_index * runs_per_instance + run_index) as u32,
                        "Typography",
                        "Latn",
                        "en",
                        axes,
                    ));
                    let measurement = outcome.measurement.expect("bounded matrix measurement");
                    if let Some(first) = first.as_ref() {
                        assert!(Arc::ptr_eq(first, &measurement));
                        assert!(outcome.cache_hit);
                    } else {
                        assert!(!outcome.cache_hit);
                        first = Some(measurement);
                    }
                }
            }

            assert_eq!(transaction.prepared_source_count(), 1);
            assert_eq!(transaction.parsed_face_count(), instance_count);
            assert_eq!(transaction.result_cache_miss_count(), instance_count);
            assert_eq!(
                transaction.result_cache_hit_count(),
                instance_count * (runs_per_instance - 1)
            );
            assert_eq!(context.cached_result_count(), instance_count);
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_b_invalid_variations_keep_structured_terminal_reasons() {
    let cases = [
        (
            vec![ShapingVariation {
                tag: "wdth".into(),
                value: 100.0,
            }],
            TerminalShapingDisposition::Unsupported,
            ShapingRejectReason::VariationAxisUnsupported,
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: f32::NAN,
            }],
            TerminalShapingDisposition::Malformed,
            ShapingRejectReason::VariationValueNonFinite,
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: 901.0,
            }],
            TerminalShapingDisposition::Malformed,
            ShapingRejectReason::VariationValueOutOfRange,
        ),
        (
            (0..=MAX_SHAPING_VARIATION_AXES)
                .map(|_| ShapingVariation {
                    tag: "wght".into(),
                    value: 400.0,
                })
                .collect(),
            TerminalShapingDisposition::BoundedLimit,
            ShapingRejectReason::VariationAxisLimitExceeded,
        ),
    ];
    let context = HorizontalShapingContext::new(registry());
    let mut transaction = context.transaction();

    for (index, (axes, disposition, reason)) in cases.iter().enumerate() {
        let outcome = transaction.shadow_measure(&variable_request(
            200 + index as u32,
            "가변",
            "Hang",
            "ko",
            axes,
        ));
        assert_eq!(outcome.trace.disposition, *disposition);
        assert_eq!(outcome.trace.reason, Some(*reason));
        assert!(outcome.measurement.is_none());
    }

    assert_eq!(transaction.prepared_source_count(), 1);
    assert_eq!(transaction.parsed_face_count(), 0);
    assert_eq!(transaction.result_cache_hit_count(), 0);
    assert_eq!(transaction.result_cache_miss_count(), 0);
    assert_eq!(context.cached_result_count(), 0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_d_explicit_slot_request_is_canonical_and_generation_owned() {
    let sources = registry();
    let reversed_title = [
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let title = [reversed_title[1].clone(), reversed_title[0].clone()];
    let mut requests = HorizontalShapingInstanceRequestRegistry::default();
    assert_eq!(MAX_HORIZONTAL_SHAPING_INSTANCE_REQUESTS, 4_096);
    assert_eq!(requests.generation(), 0);
    assert_eq!(
        requests.set_verified(&sources, HAPPINESS_SLOT, &reversed_title),
        Ok(HorizontalShapingInstanceRequestRegistration::Registered)
    );
    assert_eq!(requests.generation(), 1);
    assert_eq!(requests.request_count(), 1);
    assert_eq!(
        requests.set_verified(&sources, HAPPINESS_SLOT, &title),
        Ok(HorizontalShapingInstanceRequestRegistration::AlreadyRegistered)
    );
    assert_eq!(
        requests.generation(),
        1,
        "canonical no-op must not invalidate"
    );

    let context = HorizontalShapingContext::with_instance_requests(sources, requests);
    assert_eq!(context.instance_request_generation(), 1);
    assert_eq!(context.instance_request_count(), 1);
    let baseline = context
        .transaction()
        .shadow_measure(&request(
            230,
            HAPPINESS_SLOT,
            "Typography",
            "Latn",
            "en",
            &[],
        ))
        .measurement
        .expect("default measurement");
    let mut explicit = context
        .explicit_instance_transaction(HAPPINESS_SLOT)
        .expect("explicit instance transaction");
    assert_eq!(
        explicit.registry_generation(),
        context.registry_generation()
    );
    assert_eq!(explicit.request_generation(), 1);
    let selected = explicit
        .shadow_measure(&request(
            231,
            HAPPINESS_SLOT,
            "Typography",
            "Latn",
            "en",
            &[],
        ))
        .measurement
        .expect("explicit Title measurement");

    assert!((baseline.total_advance_px - 43.752).abs() <= 1.0e-9);
    assert!((selected.total_advance_px - 47.328).abs() <= 1.0e-9);
    assert_ne!(
        baseline.applied.identity.settings_sha256,
        selected.applied.identity.settings_sha256
    );
    assert_eq!(baseline.instance_request, None);
    assert_eq!(
        selected
            .instance_request
            .expect("explicit request provenance")
            .request_generation,
        1
    );
    assert_eq!(
        selected
            .instance_request
            .expect("explicit request provenance")
            .slot,
        HAPPINESS_SLOT
    );
    assert_eq!(context.cached_result_count(), 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_e1_per_slot_clear_preserves_other_requests() {
    let sources = registry();
    let mut requests = HorizontalShapingInstanceRequestRegistry::default();
    requests
        .set_verified(&sources, HAPPINESS_SLOT, &instance_axes(900.0, 900.0))
        .expect("register variable instance");
    requests
        .set_verified(&sources, NOTO_SLOT, &[])
        .expect("register independent default instance");
    assert_eq!(requests.request_count(), 2);
    assert_eq!(requests.generation(), 2);

    assert!(!requests.remove(ExactFontSlot::new(u32::MAX, 6)));
    assert_eq!(requests.generation(), 2, "missing clear must be idempotent");
    assert!(requests.remove(HAPPINESS_SLOT));
    assert_eq!(requests.request_count(), 1);
    assert_eq!(requests.generation(), 3);
    assert!(requests.request_slice_for_slot(HAPPINESS_SLOT).is_none());
    assert!(requests.request_slice_for_slot(NOTO_SLOT).is_some());

    assert!(
        requests.clear(),
        "existing whole-registry clear remains valid"
    );
    assert_eq!(requests.request_count(), 0);
    assert_eq!(requests.generation(), 4);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_d_invalid_requests_do_not_mutate_the_owner_snapshot() {
    let sources = registry();
    let mut requests = HorizontalShapingInstanceRequestRegistry::default();
    let cases = [
        (
            vec![ShapingVariation {
                tag: "wdth".into(),
                value: 100.0,
            }],
            HorizontalShapingInstanceRequestError::ShapingRejected(
                ShapingRejectReason::VariationAxisUnsupported,
            ),
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: f32::NAN,
            }],
            HorizontalShapingInstanceRequestError::ShapingRejected(
                ShapingRejectReason::VariationValueNonFinite,
            ),
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: 901.0,
            }],
            HorizontalShapingInstanceRequestError::ShapingRejected(
                ShapingRejectReason::VariationValueOutOfRange,
            ),
        ),
    ];
    for (axes, expected) in cases {
        assert_eq!(
            requests.set_verified(&sources, HAPPINESS_SLOT, &axes),
            Err(expected)
        );
        assert_eq!(requests.request_count(), 0);
        assert_eq!(requests.generation(), 0);
    }
    assert_eq!(
        requests.set_verified(
            &sources,
            ExactFontSlot::new(u32::MAX, 6),
            &instance_axes(900.0, 900.0),
        ),
        Err(HorizontalShapingInstanceRequestError::SourceUnavailable)
    );
    assert_eq!(
        HorizontalShapingContext::with_instance_requests(sources, requests)
            .explicit_instance_transaction(HAPPINESS_SLOT)
            .err(),
        Some(HorizontalShapingInstanceRequestError::RequestUnavailable)
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_d_slot_mismatch_and_adhoc_override_fail_before_cache_mutation() {
    let sources = registry();
    let mut requests = HorizontalShapingInstanceRequestRegistry::default();
    requests
        .set_verified(&sources, HAPPINESS_SLOT, &instance_axes(900.0, 900.0))
        .expect("register explicit Title request");
    let context = HorizontalShapingContext::with_instance_requests(sources, requests);
    let mut transaction = context
        .explicit_instance_transaction(HAPPINESS_SLOT)
        .expect("explicit instance transaction");

    let mismatch =
        transaction.shadow_measure(&request(240, NOTO_SLOT, "Typography", "Latn", "en", &[]));
    assert_eq!(
        mismatch.trace.reason,
        Some(ShapingRejectReason::ExplicitInstanceSlotMismatch)
    );
    let override_axes = instance_axes(650.0, 650.0);
    let override_attempt = transaction.shadow_measure(&variable_request(
        241,
        "Typography",
        "Latn",
        "en",
        &override_axes,
    ));
    assert_eq!(
        override_attempt.trace.reason,
        Some(ShapingRejectReason::ExplicitInstanceOverrideNotAllowed)
    );
    assert_eq!(context.cached_result_count(), 0);

    assert!(transaction
        .shadow_measure(&request(
            242,
            HAPPINESS_SLOT,
            "Typography",
            "Latn",
            "en",
            &[],
        ))
        .is_applied());
    assert_eq!(context.cached_result_count(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_d_explicit_default_preserves_q2_output_but_not_cache_provenance() {
    let sources = registry();
    let mut requests = HorizontalShapingInstanceRequestRegistry::default();
    requests
        .set_verified(&sources, HAPPINESS_SLOT, &instance_axes(900.0, 900.0))
        .expect("register Title");
    assert_eq!(
        requests.set_verified(&sources, HAPPINESS_SLOT, &instance_axes(400.0, 400.0)),
        Ok(HorizontalShapingInstanceRequestRegistration::Updated)
    );
    assert_eq!(requests.generation(), 2);
    let context = HorizontalShapingContext::with_instance_requests(sources, requests);
    let base_request = request(250, HAPPINESS_SLOT, "Typography", "Latn", "en", &[]);
    let q2 = context
        .transaction()
        .shadow_measure(&base_request)
        .measurement
        .expect("Q2 default");
    let explicit_default = context
        .explicit_instance_transaction(HAPPINESS_SLOT)
        .expect("explicit default transaction")
        .shadow_measure(&HorizontalShapingRequest {
            attempt_id: 251,
            ..base_request
        })
        .measurement
        .expect("explicit default");

    assert_eq!(q2.total_advance_px, explicit_default.total_advance_px);
    assert_eq!(
        q2.applied.identity.settings_sha256,
        explicit_default.applied.identity.settings_sha256
    );
    assert!(!Arc::ptr_eq(&q2, &explicit_default));
    assert_eq!(q2.instance_request, None);
    assert_eq!(
        explicit_default
            .instance_request
            .expect("request provenance")
            .request_generation,
        2
    );
    assert_eq!(context.cached_result_count(), 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_b_transaction_parses_each_source_once_and_reuses_owned_result() {
    let context = HorizontalShapingContext::new(registry());
    assert!(context.registry_generation() > 0);
    assert_eq!(context.cached_result_count(), 0);
    let liga = [ShapingFeature {
        tag: "liga".into(),
        value: 1,
    }];
    let mut transaction = context.transaction();
    assert_eq!(transaction.prepared_source_count(), 0);
    assert_eq!(
        transaction.registry_generation(),
        context.registry_generation()
    );

    let first = transaction.shadow_measure(&request(1, NOTO_SLOT, "office", "Latn", "en", &liga));
    assert!(first.is_applied());
    assert!(!first.cache_hit);
    assert_eq!(transaction.prepared_source_count(), 1);
    assert_eq!(transaction.parsed_face_count(), 1);
    assert_eq!(transaction.result_cache_miss_count(), 1);

    let second = transaction.shadow_measure(&request(2, NOTO_SLOT, "office", "Latn", "en", &liga));
    assert!(second.is_applied());
    assert!(second.cache_hit);
    assert_eq!(second.trace.attempt_id, 2);
    assert_eq!(transaction.parsed_face_count(), 1);
    assert_eq!(transaction.result_cache_hit_count(), 1);
    assert!(Arc::ptr_eq(
        first.measurement.as_ref().expect("first measurement"),
        second.measurement.as_ref().expect("cached measurement")
    ));

    let third = transaction.shadow_measure(&request(3, NOTO_SLOT, "AV", "Latn", "en", &[]));
    assert!(third.is_applied());
    assert!(!third.cache_hit);
    assert_eq!(transaction.prepared_source_count(), 1);
    assert_eq!(transaction.parsed_face_count(), 1);
    assert_eq!(transaction.result_cache_miss_count(), 2);
    assert_eq!(context.cached_result_count(), 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_b_latin_ligature_has_cluster_aligned_px_widths() {
    let context = HorizontalShapingContext::new(registry());
    let liga_off = [ShapingFeature {
        tag: "liga".into(),
        value: 0,
    }];
    let liga_on = [ShapingFeature {
        tag: "liga".into(),
        value: 1,
    }];
    let mut transaction = context.transaction();
    let baseline =
        transaction.shadow_measure(&request(4, NOTO_SLOT, "office", "Latn", "en", &liga_off));
    let outcome =
        transaction.shadow_measure(&request(5, NOTO_SLOT, "office", "Latn", "en", &liga_on));
    let baseline = baseline.measurement.expect("non-ligature measurement");
    let measurement = outcome.measurement.expect("liga measurement");

    assert_eq!(measurement.units_per_em, 1_000);
    assert_eq!(measurement.glyphs_px.len(), 4);
    assert_eq!(measurement.clusters.len(), 4);
    assert_eq!(
        measurement
            .glyphs_px
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        [80, 11819, 68, 70]
    );
    assert_eq!(
        measurement.clusters[1].utf8_start..measurement.clusters[1].utf8_end,
        1..4
    );
    assert_eq!(
        measurement.clusters[1].scalar_start..measurement.clusters[1].scalar_end,
        1..4
    );
    assert!((measurement.range_width(1, 4).expect("liga width") - 7.344).abs() < 1.0e-9);
    assert!(measurement.range_width(2, 3).is_none());
    assert!((measurement.range_width(0, 6).expect("run width") - 20.488).abs() < 1.0e-9);
    assert!((measurement.total_advance_px - 20.488).abs() < 1.0e-9);
    assert!((baseline.total_advance_px - 20.504).abs() < 1.0e-9);
    assert!((measurement.total_advance_px - baseline.total_advance_px + 0.016).abs() < 1.0e-9);
    assert_eq!(transaction.parsed_face_count(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_b_old_hangul_cluster_cannot_be_split_into_fake_scalar_advances() {
    let context = HorizontalShapingContext::new(registry());
    let outcome = context.transaction().shadow_measure(&request(
        6,
        SOURCE_HAN_SLOT,
        "ᄒᆞᆫ글",
        "Hang",
        "ko",
        &[],
    ));
    let measurement = outcome.measurement.expect("old Hangul measurement");

    assert_eq!(measurement.glyphs_px.len(), 4);
    assert_eq!(measurement.clusters.len(), 2);
    assert_eq!(
        measurement.clusters[0].utf8_start..measurement.clusters[0].utf8_end,
        0..9
    );
    assert_eq!(
        measurement.clusters[0].scalar_start..measurement.clusters[0].scalar_end,
        0..3
    );
    assert!(measurement.range_width(1, 3).is_none());
    assert!((measurement.range_width(0, 3).expect("jamo cluster width") - 7.728).abs() < 1.0e-9);
    assert!((measurement.total_advance_px - 15.456).abs() < 1.0e-9);

    let face = ttf_parser::Face::parse(SOURCE_HAN, 0).expect("nominal old Hangul face");
    let nominal_advances = "ᄒᆞᆫ글"
        .chars()
        .map(|scalar| {
            face.glyph_index(scalar)
                .and_then(|glyph| face.glyph_hor_advance(glyph))
        })
        .collect::<Vec<_>>();
    assert!(nominal_advances.iter().any(Option::is_none));
    assert_eq!(measurement.code_point_count, 4);
    assert_eq!(measurement.clusters.len(), 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_b_kern_feature_delta_is_single_owner_shadow_data() {
    let context = HorizontalShapingContext::new(registry());
    let kern_off = [ShapingFeature {
        tag: "kern".into(),
        value: 0,
    }];
    let kern_on = [ShapingFeature {
        tag: "kern".into(),
        value: 1,
    }];
    let mut transaction = context.transaction();
    let off = transaction.shadow_measure(&request(7, NOTO_SLOT, "AV", "Latn", "en", &kern_off));
    let on = transaction.shadow_measure(&request(8, NOTO_SLOT, "AV", "Latn", "en", &kern_on));
    let off = off.measurement.expect("kern off measurement");
    let on = on.measurement.expect("kern on measurement");

    println!(
        "q2-b AV kern-off={:.6} kern-on={:.6} delta={:.6}",
        off.total_advance_px,
        on.total_advance_px,
        on.total_advance_px - off.total_advance_px
    );
    assert_eq!(off.glyphs_px.len(), on.glyphs_px.len());
    assert!((off.total_advance_px - 9.464).abs() < 1.0e-9);
    assert!((on.total_advance_px - 9.320).abs() < 1.0e-9);
    assert!((on.total_advance_px - off.total_advance_px + 0.144).abs() < 1.0e-9);
    assert!(!Arc::ptr_eq(&off, &on));
    assert!(on.total_advance_px < off.total_advance_px);
    assert_eq!(transaction.parsed_face_count(), 1);
    assert_eq!(transaction.result_cache_miss_count(), 2);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_b_limits_and_invalid_scale_fail_closed_without_partial_measurement() {
    assert_eq!(MAX_HORIZONTAL_SHAPING_CACHE_ENTRIES, 4_096);
    assert_eq!(MAX_HORIZONTAL_SHAPING_CLUSTERS, 4_096);
    assert_eq!(MAX_HORIZONTAL_SHAPING_CACHE_TEXT_BYTES, 1024 * 1024);
    assert_eq!(MAX_HORIZONTAL_SHAPING_CACHE_GLYPHS, 262_144);
    assert_eq!(MAX_HORIZONTAL_SHAPING_CACHE_CLUSTERS, 262_144);
    let context = HorizontalShapingContext::with_cache_limit(registry(), 1);
    let mut transaction = context.transaction();
    assert!(transaction
        .shadow_measure(&request(9, NOTO_SLOT, "office", "Latn", "en", &[]))
        .is_applied());
    let cache_limit = transaction.shadow_measure(&request(10, NOTO_SLOT, "AV", "Latn", "en", &[]));
    assert_eq!(
        cache_limit.trace.disposition,
        TerminalShapingDisposition::BoundedLimit
    );
    assert_eq!(
        cache_limit.trace.reason,
        Some(ShapingRejectReason::CacheEntryLimitExceeded)
    );
    assert!(cache_limit.measurement.is_none());

    let mut invalid = request(11, NOTO_SLOT, "가", "Hang", "ko", &[]);
    invalid.width_ratio = f64::NAN;
    let invalid = transaction.shadow_measure(&invalid);
    assert_eq!(
        invalid.trace.disposition,
        TerminalShapingDisposition::Malformed
    );
    assert_eq!(
        invalid.trace.reason,
        Some(ShapingRejectReason::InvalidHorizontalScale)
    );
    assert!(invalid.measurement.is_none());

    let missing = transaction.shadow_measure(&request(
        12,
        ExactFontSlot::new(u32::MAX, 6),
        "가",
        "Hang",
        "ko",
        &[],
    ));
    assert_eq!(
        missing.trace.disposition,
        TerminalShapingDisposition::Unsupported
    );
    assert_eq!(
        missing.trace.reason,
        Some(ShapingRejectReason::SourceUnavailable)
    );
    assert!(missing.measurement.is_none());
}

const Q3_E_DOCUMENT_COMMAND_SOURCE: &str =
    include_str!("../../src/document_core/commands/document.rs");
const Q3_E_WASM_API_SOURCE: &str = include_str!("../../src/wasm_api.rs");
const Q3_E_SHAPING_PARAGRAPH_SOURCE: &str = include_str!("../../src/renderer/shaping_paragraph.rs");
const Q3_E_SHAPING_GLYPH_SOURCE: &str = include_str!("../../src/paint/shaping_glyph.rs");

#[test]
fn issue_4969_q3_e0_red_reversible_exact_instance_owner_is_absent() {
    assert!(
        Q3_E_DOCUMENT_COMMAND_SOURCE.contains("pub fn clear_exact_font_instance_native"),
        "Q3-E1 red: the per-slot reversible native owner is absent"
    );
}

#[test]
fn issue_4969_q3_e0_red_public_instance_adapters_are_absent() {
    assert!(
        Q3_E_WASM_API_SOURCE.contains("js_name = setExactFontInstance")
            && Q3_E_WASM_API_SOURCE.contains("js_name = clearExactFontInstance"),
        "Q3-E2 red: the strict set/clear WASM adapters are absent"
    );
}

#[test]
fn issue_4969_q3_e0_red_explicit_only_candidate_session_is_absent() {
    assert!(
        Q3_E_SHAPING_PARAGRAPH_SOURCE.contains("is_bounded_explicit_instance_candidate_text")
            && Q3_E_SHAPING_PARAGRAPH_SOURCE
                .contains("run_bounded_explicit_instance_line_transaction"),
        "Q3-E3 red: the request-gated modern-Hangul/Latin single-run policy is absent"
    );
}

#[test]
fn issue_4969_q3_e0_red_atomic_variable_outline_publication_is_absent() {
    assert!(
        Q3_E_SHAPING_GLYPH_SOURCE.contains("report.glyph_outline")
            && Q3_E_SHAPING_GLYPH_SOURCE.contains("PaintOp::GlyphOutline"),
        "Q3-E4 red: product lowering does not atomically publish the qualified variable outline"
    );
}
