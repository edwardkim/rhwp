//! Issue #4968 W9-Q3-2..Q3-4: capability, run gate, and pair candidate must fail closed.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;

use kerning::{
    compose_kerning_paragraph_measurement, compute_kerning_pair_candidate,
    compute_kerning_run_measurement, decide_kerning_run_gate, inspect_exact_font_kerning,
    identify_exact_font_source, measure_kerning_paragraph_segments, prepare_kerning_pair_engine,
    resolve_exact_font_source,
    ExactFontRegistryError, ExactFontRegistryRegistration, ExactFontSlot, ExactFontSource,
    ExactFontSourceHandle, ExactFontSourceProvider, ExactFontSourceRegistry,
    ExactFontSourceResolutionReason, KerningCapability, KerningCapabilityFallbackReason,
    KerningLayoutSession, KerningPairCandidateFallbackReason, KerningPairCandidateStatus,
    KerningParagraphMeasurementDisposition, KerningParagraphMeasurementFallbackReason,
    KerningParagraphScalarStyle, KerningParagraphSegmentMeasurement, KerningRequest,
    KerningRunFallbackReason, KerningRunGate, KerningRunMeasurementDisposition,
    KerningRunMeasurementFallbackReason, KerningSourceSession, KerningSourceSessionStatus,
    MAX_KERNING_ADJACENT_PAIRS, MAX_KERNING_FONT_BYTES, MAX_KERNING_PARAGRAPH_SEGMENTS,
    MAX_KERNING_REGISTRY_FACES, MAX_KERNING_RUN_CODE_POINTS, MAX_KERNING_RUN_GLYPHS,
};
use std::cell::Cell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const NOTO_REGULAR: &[u8] =
    include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const NO_PAIR_TABLE: &[u8] =
    include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");
const EXACT_KERNING_SMOKE: &[u8] =
    include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");

#[test]
fn issue_4968_exact_slot_registry_roundtrips_provider_and_session() {
    let slot = ExactFontSlot::new(7, 1);
    let mut registry = ExactFontSourceRegistry::default();
    assert_eq!(
        registry.register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        ),
        Ok(ExactFontRegistryRegistration::Registered)
    );
    assert_eq!(registry.slot_count(), 1);
    assert_eq!(registry.source_count(), 1);
    assert_eq!(registry.total_source_bytes(), EXACT_KERNING_SMOKE.len());

    let handle = registry.handle_for_slot(slot).expect("slot handle").clone();
    let mut session = KerningSourceSession::new(&registry);
    let trace = session.prepare(&handle);
    assert_eq!(trace.status, KerningSourceSessionStatus::Ready);
    assert!(session.engine(&handle).is_some());
    drop(session);

    assert_eq!(
        registry.register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        ),
        Ok(ExactFontRegistryRegistration::AlreadyRegistered)
    );
    assert_eq!(registry.source_count(), 1);
}

#[test]
fn issue_4968_exact_slot_registry_fails_closed_on_conflict_and_face_limit() {
    let mut registry = ExactFontSourceRegistry::default();
    let slot = ExactFontSlot::new(1, 1);
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        )
        .expect("first source");
    assert_eq!(
        registry.register(
            slot,
            ExactFontSource {
                bytes: NO_PAIR_TABLE,
                face_index: 0,
            },
        ),
        Err(ExactFontRegistryError::SlotConflict)
    );

    let mut bounded = ExactFontSourceRegistry::default();
    for face_index in 0..MAX_KERNING_REGISTRY_FACES as u32 {
        bounded
            .register(
                ExactFontSlot::new(face_index, 0),
                ExactFontSource {
                    bytes: b"bounded",
                    face_index,
                },
            )
            .expect("bounded face");
    }
    assert_eq!(
        bounded.register(
            ExactFontSlot::new(MAX_KERNING_REGISTRY_FACES as u32, 0),
            ExactFontSource {
                bytes: b"bounded",
                face_index: MAX_KERNING_REGISTRY_FACES as u32,
            },
        ),
        Err(ExactFontRegistryError::FaceLimitExceeded)
    );
}

#[test]
fn issue_4968_external_exact_source_registration_keeps_k0_svg_identical() {
    const BLANK: &[u8] = include_bytes!("../../saved/blank2010.hwp");
    let mut core = rhwp::DocumentCore::from_bytes(BLANK).expect("blank document");
    let before = core.render_page_svg_native(0).expect("baseline SVG");
    let registration = core
        .register_exact_font_source_native(0, 1, EXACT_KERNING_SMOKE, 0)
        .expect("external exact source registration");
    assert!(registration.contains("\"status\":\"registered\""));
    let after = core.render_page_svg_native(0).expect("registered SVG");
    assert_eq!(after, before);
}

struct BorrowedSourceProvider<'a> {
    source: Option<ExactFontSource<'a>>,
}

impl ExactFontSourceProvider for BorrowedSourceProvider<'_> {
    fn source_for_handle<'a>(
        &'a self,
        _handle: &ExactFontSourceHandle,
    ) -> Option<ExactFontSource<'a>> {
        self.source
    }
}

struct CountingSourceProvider<'a> {
    source: Option<ExactFontSource<'a>>,
    calls: Cell<usize>,
}

impl ExactFontSourceProvider for CountingSourceProvider<'_> {
    fn source_for_handle<'a>(
        &'a self,
        _handle: &ExactFontSourceHandle,
    ) -> Option<ExactFontSource<'a>> {
        self.calls.set(self.calls.get() + 1);
        self.source
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn replace_table_with_legacy_kern(font: &[u8], replace_tag: &[u8; 4]) -> Vec<u8> {
    // OpenType kern v0, horizontal format 0, one pair: glyph 1/glyph 2 => -70.
    let kern_table: [u8; 24] = [
        0, 0, 0, 1, // table version, subtable count
        0, 0, 0, 20, 0, 1, // subtable version, length, horizontal format 0
        0, 1, 0, 6, 0, 0, 0, 0, // nPairs, searchRange, entrySelector, rangeShift
        0, 1, 0, 2, 0xff, 0xba, // glyph pair and -70
    ];
    let mut output = font.to_vec();
    let table_count = usize::from(read_u16(&output, 4));
    let record = (0..table_count)
        .map(|index| 12 + index * 16)
        .find(|offset| &output[*offset..*offset + 4] == replace_tag)
        .expect("replaceable optional table");
    let table_offset = read_u32(&output, record + 8) as usize;
    let table_len = read_u32(&output, record + 12) as usize;
    assert!(table_len >= kern_table.len(), "replacement table is too small");
    output[record..record + 4].copy_from_slice(b"kern");
    output[record + 4..record + 8].fill(0); // parser does not consume checksums
    output[record + 12..record + 16].copy_from_slice(&(kern_table.len() as u32).to_be_bytes());
    output[table_offset..table_offset + kern_table.len()].copy_from_slice(&kern_table);
    output
}

#[test]
fn issue_4968_capability_precedence_and_failure_reasons_are_structured() {
    let gpos = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    }));
    assert_eq!(gpos.capability, KerningCapability::GposKern);
    assert_eq!(gpos.fallback_reason, None);
    assert_eq!(gpos.units_per_em, Some(1000));
    assert_eq!(
        gpos.font_source_sha256.as_deref(),
        Some("6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74")
    );

    let legacy_font = replace_table_with_legacy_kern(NO_PAIR_TABLE, b"sbix");
    let legacy = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &legacy_font,
        face_index: 0,
    }));
    assert_eq!(legacy.capability, KerningCapability::LegacyKern);
    assert_eq!(legacy.fallback_reason, None);

    let both_font = replace_table_with_legacy_kern(NOTO_REGULAR, b"BASE");
    let both = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &both_font,
        face_index: 0,
    }));
    assert_eq!(both.capability, KerningCapability::GposKern);

    let unsupported = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NO_PAIR_TABLE,
        face_index: 0,
    }));
    assert_eq!(unsupported.capability, KerningCapability::Unsupported);
    assert_eq!(
        unsupported.fallback_reason,
        Some(KerningCapabilityFallbackReason::PairTableUnsupported)
    );
    let unsupported_json = serde_json::to_value(&unsupported).expect("capability JSON");
    assert_eq!(unsupported_json["capability"], "unsupported");
    assert_eq!(
        unsupported_json["fallbackReason"],
        "pair-table-unsupported"
    );
    assert_eq!(unsupported_json["fontBytes"], NO_PAIR_TABLE.len());
    assert_eq!(unsupported_json["faceIndex"], 0);

    let missing = inspect_exact_font_kerning(None);
    assert_eq!(
        missing.fallback_reason,
        Some(KerningCapabilityFallbackReason::FontSourceUnavailable)
    );

    let malformed = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: b"not-an-sfnt",
        face_index: 0,
    }));
    assert_eq!(
        malformed.fallback_reason,
        Some(KerningCapabilityFallbackReason::MalformedSfnt)
    );
    assert!(malformed.font_source_sha256.is_some());

    let oversized_bytes = vec![0; MAX_KERNING_FONT_BYTES + 1];
    let oversized = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: &oversized_bytes,
        face_index: 0,
    }));
    assert_eq!(
        oversized.fallback_reason,
        Some(KerningCapabilityFallbackReason::FontByteLimitExceeded)
    );
    assert_eq!(oversized.font_source_sha256, None);
}

#[test]
fn issue_4968_exact_source_handle_resolves_without_carrying_font_payload() {
    let source = ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    };
    let handle = identify_exact_font_source(source).expect("bounded exact source identity");
    assert_eq!(handle.font_bytes, NOTO_REGULAR.len());
    assert_eq!(handle.face_index, 0);
    assert_eq!(
        handle.font_source_sha256,
        "6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74"
    );

    let handle_json = serde_json::to_value(&handle).expect("source handle JSON");
    assert_eq!(handle_json["fontBytes"], NOTO_REGULAR.len());
    assert_eq!(handle_json["faceIndex"], 0);
    assert!(handle_json.get("bytes").is_none());
    assert!(handle_json.get("path").is_none());
    assert!(handle_json.get("fontFamily").is_none());

    let provider = BorrowedSourceProvider {
        source: Some(source),
    };
    let resolved = resolve_exact_font_source(&provider, &handle).expect("exact source resolves");
    assert_eq!(resolved.bytes.as_ptr(), NOTO_REGULAR.as_ptr());
    assert_eq!(resolved.bytes.len(), NOTO_REGULAR.len());

    let missing = BorrowedSourceProvider { source: None };
    assert_eq!(
        resolve_exact_font_source(&missing, &handle).unwrap_err(),
        ExactFontSourceResolutionReason::SourceUnavailable
    );

    let wrong_face = BorrowedSourceProvider {
        source: Some(ExactFontSource {
            bytes: NOTO_REGULAR,
            face_index: 1,
        }),
    };
    assert_eq!(
        resolve_exact_font_source(&wrong_face, &handle).unwrap_err(),
        ExactFontSourceResolutionReason::FaceIndexMismatch
    );

    let shorter = BorrowedSourceProvider {
        source: Some(ExactFontSource {
            bytes: NO_PAIR_TABLE,
            face_index: 0,
        }),
    };
    assert_eq!(
        resolve_exact_font_source(&shorter, &handle).unwrap_err(),
        ExactFontSourceResolutionReason::ByteLengthMismatch
    );

    let small_source = ExactFontSource {
        bytes: NO_PAIR_TABLE,
        face_index: 0,
    };
    let small_handle = identify_exact_font_source(small_source).expect("small source identity");
    let mut same_length_different_bytes = NO_PAIR_TABLE.to_vec();
    same_length_different_bytes[0] ^= 0x01;
    let wrong_digest = BorrowedSourceProvider {
        source: Some(ExactFontSource {
            bytes: &same_length_different_bytes,
            face_index: 0,
        }),
    };
    assert_eq!(
        resolve_exact_font_source(&wrong_digest, &small_handle).unwrap_err(),
        ExactFontSourceResolutionReason::Sha256Mismatch
    );

    let oversized = vec![0; MAX_KERNING_FONT_BYTES + 1];
    assert_eq!(
        identify_exact_font_source(ExactFontSource {
            bytes: &oversized,
            face_index: 0,
        })
        .unwrap_err(),
        ExactFontSourceResolutionReason::FontByteLimitExceeded
    );
}

#[test]
fn issue_4968_layout_session_caches_exact_engine_without_payload_trace() {
    let source = ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    };
    let handle = identify_exact_font_source(source).expect("bounded exact source identity");
    let provider = CountingSourceProvider {
        source: Some(source),
        calls: Cell::new(0),
    };
    let mut session = KerningSourceSession::new(&provider);

    let first = session.prepare(&handle);
    assert_eq!(first.status, KerningSourceSessionStatus::Ready);
    assert!(!first.cache_hit);
    assert_eq!(first.capability.capability, KerningCapability::GposKern);
    assert_eq!(first.resolution_reason, None);
    assert_eq!(first.pair_engine_reason, None);
    assert_eq!(provider.calls.get(), 1);

    let second = session.prepare(&handle);
    assert_eq!(second.status, KerningSourceSessionStatus::Ready);
    assert!(second.cache_hit);
    assert_eq!(provider.calls.get(), 1, "cache hit must not query provider");

    let text = "AV To WA HH";
    let gate = decide_kerning_run_gate(
        true,
        text,
        text.chars().count(),
        &second.capability,
    );
    let candidate = compute_kerning_pair_candidate(
        text,
        session.engine(&handle).expect("cached pair engine"),
        &gate,
    );
    assert_eq!(
        candidate.status,
        KerningPairCandidateStatus::AdjustmentCandidate
    );
    assert_eq!(candidate.total_x_advance_delta, -94);

    let trace = serde_json::to_value(&second).expect("session trace JSON");
    assert_eq!(trace["status"], "ready");
    assert_eq!(trace["cacheHit"], true);
    for forbidden in ["bytes", "path", "text", "fontFamily"] {
        assert!(trace.get(forbidden).is_none(), "trace leaked {forbidden}");
        assert!(
            trace["handle"].get(forbidden).is_none(),
            "handle leaked {forbidden}"
        );
        assert!(
            trace["capability"].get(forbidden).is_none(),
            "capability leaked {forbidden}"
        );
    }
}

#[test]
fn issue_4968_layout_session_caches_resolution_failures_closed() {
    let handle = identify_exact_font_source(ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    })
    .expect("bounded exact source identity");

    let missing_provider = CountingSourceProvider {
        source: None,
        calls: Cell::new(0),
    };
    let mut missing_session = KerningSourceSession::new(&missing_provider);
    let missing_first = missing_session.prepare(&handle);
    let missing_second = missing_session.prepare(&handle);
    assert_eq!(
        missing_first.status,
        KerningSourceSessionStatus::FailClosed
    );
    assert!(!missing_first.cache_hit);
    assert!(missing_second.cache_hit);
    assert_eq!(
        missing_first.resolution_reason,
        Some(ExactFontSourceResolutionReason::SourceUnavailable)
    );
    assert_eq!(
        missing_first.capability.fallback_reason,
        Some(KerningCapabilityFallbackReason::FontSourceUnavailable)
    );
    assert_eq!(missing_provider.calls.get(), 1);
    assert!(missing_session.engine(&handle).is_none());

    let mismatch_provider = CountingSourceProvider {
        source: Some(ExactFontSource {
            bytes: NO_PAIR_TABLE,
            face_index: 0,
        }),
        calls: Cell::new(0),
    };
    let mut mismatch_session = KerningSourceSession::new(&mismatch_provider);
    let mismatch_first = mismatch_session.prepare(&handle);
    let mismatch_second = mismatch_session.prepare(&handle);
    assert_eq!(
        mismatch_first.resolution_reason,
        Some(ExactFontSourceResolutionReason::ByteLengthMismatch)
    );
    assert!(mismatch_second.cache_hit);
    assert_eq!(mismatch_provider.calls.get(), 1);
    assert!(mismatch_session.engine(&handle).is_none());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_public_exact_source_fixture_is_native_wasm_portable() {
    let source = ExactFontSource {
        bytes: EXACT_KERNING_SMOKE,
        face_index: 0,
    };
    let handle = identify_exact_font_source(source).expect("bounded public fixture identity");
    assert_eq!(handle.font_bytes, 1_236);
    assert_eq!(
        handle.font_source_sha256,
        "775667d1980cd734e331f01e9390e02191bc35d669325291c842968cb0a4a9fc"
    );

    let provider = BorrowedSourceProvider {
        source: Some(source),
    };
    let mut session = KerningSourceSession::new(&provider);
    let trace = session.prepare(&handle);
    assert_eq!(trace.status, KerningSourceSessionStatus::Ready);
    assert_eq!(trace.capability.capability, KerningCapability::GposKern);

    let text = "AV To WA HH";
    let gate = decide_kerning_run_gate(
        true,
        text,
        text.chars().count(),
        &trace.capability,
    );
    let candidate = compute_kerning_pair_candidate(
        text,
        session.engine(&handle).expect("public fixture pair engine"),
        &gate,
    );
    assert_eq!(
        candidate.status,
        KerningPairCandidateStatus::AdjustmentCandidate
    );
    assert_eq!(candidate.total_x_advance_delta, -120);
    assert_eq!(candidate.fallback_reason, None);
}

#[test]
fn issue_4968_run_gate_is_bounded_and_does_not_claim_pair_application() {
    let gpos = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    }));
    let eligible = decide_kerning_run_gate(true, "AV To WA HH", 11, &gpos);
    assert_eq!(eligible.request, KerningRequest::Enabled);
    assert_eq!(eligible.capability, KerningCapability::GposKern);
    assert_eq!(eligible.gate, KerningRunGate::Eligible);
    assert_eq!(eligible.candidate_pair_count, 10);
    assert_eq!(eligible.fallback_reason, None);
    let eligible_json = serde_json::to_value(&eligible).expect("run gate JSON");
    assert_eq!(eligible_json["request"], "enabled");
    assert_eq!(eligible_json["capability"], "gpos-kern");
    assert_eq!(eligible_json["gate"], "eligible");
    assert!(eligible_json.get("text").is_none(), "trace must omit source text");

    let disabled = decide_kerning_run_gate(false, "AV", 2, &gpos);
    assert_eq!(disabled.request, KerningRequest::Disabled);
    assert_eq!(disabled.gate, KerningRunGate::NotRequested);

    let unsupported_capability = inspect_exact_font_kerning(Some(ExactFontSource {
        bytes: NO_PAIR_TABLE,
        face_index: 0,
    }));
    let unsupported = decide_kerning_run_gate(true, "AV", 2, &unsupported_capability);
    assert_eq!(unsupported.gate, KerningRunGate::FailClosed);
    assert_eq!(
        unsupported.fallback_reason,
        Some(KerningRunFallbackReason::PairTableUnsupported)
    );

    let oversized_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS + 50_000);
    let code_points = decide_kerning_run_gate(
        true,
        &oversized_text,
        MAX_KERNING_RUN_GLYPHS,
        &gpos,
    );
    assert_eq!(code_points.gate, KerningRunGate::FailClosed);
    assert_eq!(code_points.code_point_count, MAX_KERNING_RUN_CODE_POINTS);
    assert!(code_points.code_point_limit_exceeded);
    assert_eq!(code_points.glyph_count, MAX_KERNING_RUN_GLYPHS);
    assert_eq!(
        code_points.candidate_pair_count,
        MAX_KERNING_ADJACENT_PAIRS
    );
    assert_eq!(
        code_points.fallback_reason,
        Some(KerningRunFallbackReason::RunCodePointLimitExceeded)
    );

    let glyphs = decide_kerning_run_gate(true, "AV", MAX_KERNING_RUN_GLYPHS + 1, &gpos);
    assert_eq!(glyphs.gate, KerningRunGate::FailClosed);
    assert_eq!(glyphs.glyph_count, MAX_KERNING_RUN_GLYPHS);
    assert!(glyphs.glyph_limit_exceeded);
    assert_eq!(glyphs.candidate_pair_count, MAX_KERNING_ADJACENT_PAIRS);
    assert_eq!(
        glyphs.fallback_reason,
        Some(KerningRunFallbackReason::RunGlyphLimitExceeded)
    );
}

#[test]
fn issue_4968_pair_candidate_is_exact_bounded_and_not_applied() {
    let source = ExactFontSource {
        bytes: NOTO_REGULAR,
        face_index: 0,
    };
    let capability = inspect_exact_font_kerning(Some(source));
    let engine = prepare_kerning_pair_engine(source, &capability).expect("exact Noto engine");

    let pair_text = "AV To WA HH";
    let pair_gate = decide_kerning_run_gate(true, pair_text, pair_text.chars().count(), &capability);
    let pair = compute_kerning_pair_candidate(pair_text, &engine, &pair_gate);
    assert_eq!(
        pair.status,
        KerningPairCandidateStatus::AdjustmentCandidate
    );
    assert_eq!(pair.capability, KerningCapability::GposKern);
    assert_eq!(pair.glyph_count, 11);
    assert_eq!(pair.examined_pair_count, 10);
    assert_eq!(pair.total_x_advance_delta, -94);
    assert!(pair.adjusted_position_count > 0);
    assert_eq!(pair.fallback_reason, None);
    let pair_json = serde_json::to_value(&pair).expect("pair candidate JSON");
    assert_eq!(pair_json["status"], "adjustment-candidate");
    assert!(pair_json.get("text").is_none(), "trace must omit source text");
    assert!(
        pair_json.get("applied").is_none(),
        "candidate must not claim application"
    );

    let no_pair_text = "HH";
    let no_pair_gate =
        decide_kerning_run_gate(true, no_pair_text, no_pair_text.chars().count(), &capability);
    let no_pair = compute_kerning_pair_candidate(no_pair_text, &engine, &no_pair_gate);
    assert_eq!(
        no_pair.status,
        KerningPairCandidateStatus::NoAdjustmentCandidate
    );
    assert_eq!(no_pair.total_x_advance_delta, 0);
    assert!(no_pair.position_deltas.is_empty());

    let disabled_gate = decide_kerning_run_gate(false, "AV", 2, &capability);
    let disabled = compute_kerning_pair_candidate("AV", &engine, &disabled_gate);
    assert_eq!(disabled.status, KerningPairCandidateStatus::NotEligible);
    assert_eq!(
        disabled.fallback_reason,
        Some(KerningPairCandidateFallbackReason::RunGateNotEligible)
    );

    let mismatched = prepare_kerning_pair_engine(
        ExactFontSource {
            bytes: NO_PAIR_TABLE,
            face_index: 0,
        },
        &capability,
    );
    assert_eq!(
        mismatched.err(),
        Some(KerningPairCandidateFallbackReason::FontSourceMismatch)
    );

    let stale_gate = decide_kerning_run_gate(true, "AV", 2, &capability);
    let stale = compute_kerning_pair_candidate("AVA", &engine, &stale_gate);
    assert_eq!(stale.status, KerningPairCandidateStatus::FailClosed);
    assert_eq!(
        stale.fallback_reason,
        Some(KerningPairCandidateFallbackReason::RunGateInputMismatch)
    );

    let rtl_text = "אב";
    let rtl_gate =
        decide_kerning_run_gate(true, rtl_text, rtl_text.chars().count(), &capability);
    let rtl = compute_kerning_pair_candidate(rtl_text, &engine, &rtl_gate);
    assert_eq!(rtl.status, KerningPairCandidateStatus::FailClosed);
    assert_eq!(
        rtl.fallback_reason,
        Some(KerningPairCandidateFallbackReason::UnsupportedDirection)
    );

    let ligature_text = "ffi";
    let ligature_gate = decide_kerning_run_gate(
        true,
        ligature_text,
        ligature_text.chars().count(),
        &capability,
    );
    let ligature = compute_kerning_pair_candidate(ligature_text, &engine, &ligature_gate);
    assert_eq!(ligature.status, KerningPairCandidateStatus::FailClosed);
    assert_eq!(
        ligature.fallback_reason,
        Some(KerningPairCandidateFallbackReason::NominalGlyphIdentityChanged)
    );

    let bounded_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS);
    let bounded_gate = decide_kerning_run_gate(
        true,
        &bounded_text,
        MAX_KERNING_RUN_GLYPHS,
        &capability,
    );
    let bounded = compute_kerning_pair_candidate(&bounded_text, &engine, &bounded_gate);
    assert_ne!(bounded.status, KerningPairCandidateStatus::FailClosed);
    assert_eq!(bounded.glyph_count, MAX_KERNING_RUN_GLYPHS);
    assert_eq!(bounded.examined_pair_count, MAX_KERNING_ADJACENT_PAIRS);
    assert!(bounded.adjusted_position_count <= MAX_KERNING_RUN_GLYPHS);
}

#[test]
fn issue_4968_pair_candidate_honors_legacy_and_gpos_precedence() {
    let legacy_font = replace_table_with_legacy_kern(NO_PAIR_TABLE, b"sbix");
    let legacy_source = ExactFontSource {
        bytes: &legacy_font,
        face_index: 0,
    };
    let legacy_capability = inspect_exact_font_kerning(Some(legacy_source));
    let legacy_engine =
        prepare_kerning_pair_engine(legacy_source, &legacy_capability).expect("legacy engine");
    let legacy_text = "\u{e100}\u{e101}";
    let legacy_gate = decide_kerning_run_gate(true, legacy_text, 2, &legacy_capability);
    let legacy = compute_kerning_pair_candidate(legacy_text, &legacy_engine, &legacy_gate);
    assert_eq!(legacy.capability, KerningCapability::LegacyKern);
    assert_eq!(
        legacy.status,
        KerningPairCandidateStatus::AdjustmentCandidate
    );
    assert_eq!(legacy.total_x_advance_delta, -70);

    let both_font = replace_table_with_legacy_kern(NOTO_REGULAR, b"BASE");
    let both_source = ExactFontSource {
        bytes: &both_font,
        face_index: 0,
    };
    let both_capability = inspect_exact_font_kerning(Some(both_source));
    let both_engine =
        prepare_kerning_pair_engine(both_source, &both_capability).expect("GPOS engine");
    let both_gate = decide_kerning_run_gate(true, "AV", 2, &both_capability);
    let both = compute_kerning_pair_candidate("AV", &both_engine, &both_gate);
    assert_eq!(both.capability, KerningCapability::GposKern);
    assert_eq!(both.total_x_advance_delta, -18);
}

fn linear_positions(code_points: usize, advance: f64) -> Vec<f64> {
    (0..=code_points)
        .map(|index| index as f64 * advance)
        .collect()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_common_run_measurement_applies_pair_delta_after_ratio_and_spacing() {
    let text = "AV To WA HH";
    let code_points = text.chars().count();
    let mut registry = ExactFontSourceRegistry::default();
    let slot = ExactFontSlot::new(4968, 1);
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        )
        .expect("public exact source");
    let handle = registry
        .handle_for_slot(slot)
        .expect("registered handle")
        .clone();
    let mut session = KerningSourceSession::new(&registry);

    for (ratio, letter_spacing) in [(1.0, 0.0), (0.9, -1.0), (0.8, -2.0)] {
        // base advance는 기존 측정 계층이 이미 장평과 glyph-relative 자간을
        // 적용한 값이다. pair 계층에는 자간 자체를 넘기지 않는다.
        let base_advance = 10.0 * ratio + letter_spacing;
        let base_positions = linear_positions(code_points, base_advance);
        let base_width = *base_positions.last().expect("base width");
        let measurement = compute_kerning_run_measurement(
            text,
            true,
            base_positions.clone(),
            20.0,
            ratio,
            Some(&handle),
            &mut session,
        );

        assert_eq!(
            measurement.disposition,
            KerningRunMeasurementDisposition::PairAdjusted
        );
        assert_eq!(measurement.fallback_reason, None);
        assert_eq!(measurement.base_positions, base_positions);
        assert_eq!(measurement.code_point_count, code_points);
        assert!(!measurement.code_point_limit_exceeded);
        assert_eq!(measurement.bounded_segment_count, 1);
        assert_eq!(measurement.advance_deltas.len(), code_points);
        let expected_pair_delta = -120.0 * 20.0 * ratio / 1_000.0;
        let actual_pair_delta: f64 = measurement.advance_deltas.iter().sum();
        assert!((actual_pair_delta - expected_pair_delta).abs() < 1e-12);
        let replay_pair_delta: f64 = measurement
            .glyph_position_deltas
            .iter()
            .map(|delta| delta.x_advance)
            .sum();
        assert!((replay_pair_delta - expected_pair_delta).abs() < 1e-12);
        assert!(measurement.glyph_position_deltas.iter().all(|delta| {
            delta.x_advance.is_finite()
                && delta.y_advance.is_finite()
                && delta.x_offset.is_finite()
                && delta.y_offset.is_finite()
        }));
        assert!((measurement.total_width - (base_width + expected_pair_delta)).abs() < 1e-12);
        assert_eq!(
            measurement.positions(),
            measurement
                .pair_adjusted_positions
                .as_deref()
                .expect("adjusted positions")
        );
        assert_eq!(
            measurement
                .candidate
                .as_ref()
                .expect("candidate trace")
                .total_x_advance_delta,
            -120
        );
    }

    let trace = serde_json::to_value(compute_kerning_run_measurement(
        text,
        true,
        linear_positions(code_points, 10.0),
        20.0,
        1.0,
        Some(&handle),
        &mut session,
    ))
    .expect("measurement JSON");
    for forbidden in ["text", "bytes", "path", "fontFamily"] {
        assert!(trace.get(forbidden).is_none(), "trace leaked {forbidden}");
    }
}

#[test]
fn issue_4968_common_run_measurement_preserves_k0_and_fail_closed_positions() {
    let text = "AV";
    let base_positions = vec![0.0, 9.25, 18.5];
    let source = ExactFontSource {
        bytes: EXACT_KERNING_SMOKE,
        face_index: 0,
    };
    let handle = identify_exact_font_source(source).expect("exact source handle");
    let provider = CountingSourceProvider {
        source: Some(source),
        calls: Cell::new(0),
    };
    let mut session = KerningSourceSession::new(&provider);

    let k0 = compute_kerning_run_measurement(
        text,
        false,
        base_positions.clone(),
        f64::NAN,
        f64::NAN,
        Some(&handle),
        &mut session,
    );
    assert_eq!(
        k0.disposition,
        KerningRunMeasurementDisposition::ExistingPositions
    );
    assert_eq!(k0.positions(), base_positions);
    assert!(k0.pair_adjusted_positions.is_none());
    assert!(k0.glyph_position_deltas.is_empty());
    assert!(k0.session.is_none());
    assert!(k0.candidate.is_none());
    assert_eq!(provider.calls.get(), 0, "K0 must not resolve source");

    let missing = compute_kerning_run_measurement(
        text,
        true,
        base_positions.clone(),
        20.0,
        1.0,
        None,
        &mut session,
    );
    assert_eq!(
        missing.disposition,
        KerningRunMeasurementDisposition::ExactSourceUnavailable
    );
    assert_eq!(
        missing.fallback_reason,
        Some(KerningRunMeasurementFallbackReason::ExactSourceUnavailable)
    );
    assert_eq!(missing.positions(), base_positions);
    assert_eq!(provider.calls.get(), 0, "missing handle must not query provider");

    let oversized_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS + 1);
    let oversized_positions = linear_positions(oversized_text.len(), 5.0);
    let oversized = compute_kerning_run_measurement(
        &oversized_text,
        true,
        oversized_positions.clone(),
        20.0,
        1.0,
        Some(&handle),
        &mut session,
    );
    assert_eq!(
        oversized.disposition,
        KerningRunMeasurementDisposition::FailClosed
    );
    assert_eq!(
        oversized.fallback_reason,
        Some(KerningRunMeasurementFallbackReason::RunCodePointLimitExceeded)
    );
    assert_eq!(oversized.positions(), oversized_positions);
    assert!(oversized.advance_deltas.is_empty());
    assert_eq!(provider.calls.get(), 0, "oversized run must stop before source");
}

#[test]
fn issue_4968_layout_transaction_pins_slot_generation_and_reuses_face_cache() {
    let slot = ExactFontSlot::new(4968, 1);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        )
        .expect("public exact source");
    let generation = registry.generation();
    let expected_handle = registry
        .handle_for_slot(slot)
        .expect("registered handle")
        .clone();

    let mut transaction = KerningLayoutSession::new(&registry);
    assert_eq!(transaction.registry_generation(), generation);
    assert_eq!(transaction.source_handle(slot), Some(&expected_handle));

    let first = transaction.measure_run(
        slot,
        "AV",
        true,
        vec![0.0, 10.0, 20.0],
        20.0,
        1.0,
    );
    assert_eq!(
        first.disposition,
        KerningRunMeasurementDisposition::PairAdjusted
    );
    assert!(!first.session.as_ref().expect("first trace").cache_hit);

    let second = transaction.measure_run(
        slot,
        "AV",
        true,
        vec![0.0, 10.0, 20.0],
        20.0,
        1.0,
    );
    assert_eq!(
        second.disposition,
        KerningRunMeasurementDisposition::PairAdjusted
    );
    assert!(second.session.as_ref().expect("cached trace").cache_hit);

    // K0는 존재하지 않는 slot과 유효하지 않은 scale을 주어도 source 계층에
    // 진입하지 않고 기존 위치를 그대로 보존한다.
    let k0 = transaction.measure_run(
        ExactFontSlot::new(9999, 1),
        "AV",
        false,
        vec![0.0, 9.25, 18.5],
        f64::NAN,
        f64::NAN,
    );
    assert_eq!(
        k0.disposition,
        KerningRunMeasurementDisposition::ExistingPositions
    );
    assert_eq!(k0.positions(), [0.0, 9.25, 18.5]);
    assert!(k0.source_handle.is_none());
    assert!(k0.session.is_none());
}

#[test]
fn issue_4968_paragraph_measurement_commits_segments_to_one_position_map() {
    let slot = ExactFontSlot::new(4968, 1);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: EXACT_KERNING_SMOKE,
                face_index: 0,
            },
        )
        .expect("public exact source");
    let mut transaction = KerningLayoutSession::new(&registry);
    let base_positions = vec![0.0, 10.0, 20.0, 30.0, 40.0];
    let left = transaction.measure_run(
        slot,
        "AV",
        true,
        vec![0.0, 10.0, 20.0],
        20.0,
        1.0,
    );
    let right = transaction.measure_run(
        slot,
        "AV",
        true,
        vec![0.0, 10.0, 20.0],
        20.0,
        1.0,
    );
    let left_width = left.total_width;
    let right_width = right.total_width;
    let paragraph = compose_kerning_paragraph_measurement(
        4,
        base_positions.clone(),
        vec![
            KerningParagraphSegmentMeasurement {
                start_index: 0,
                end_index: 2,
                slot,
                measurement: left,
            },
            KerningParagraphSegmentMeasurement {
                start_index: 2,
                end_index: 4,
                slot,
                measurement: right,
            },
        ],
    );

    assert_eq!(
        paragraph.disposition,
        KerningParagraphMeasurementDisposition::PairAdjusted
    );
    assert_eq!(paragraph.fallback_reason, None);
    assert_eq!(paragraph.bounded_segment_count, 2);
    assert_eq!(paragraph.base_positions, base_positions);
    assert_eq!(paragraph.positions().len(), 5);
    assert!((paragraph.range_width(0, 2).expect("left width") - left_width).abs() < 1e-12);
    assert!((paragraph.range_width(2, 4).expect("right width") - right_width).abs() < 1e-12);
    assert!(
        (paragraph.range_width(0, 4).expect("whole width") - left_width - right_width).abs()
            < 1e-12
    );
    assert_eq!(paragraph.range_width(4, 3), None);
    assert_eq!(paragraph.range_width(0, 5), None);

    let trace = serde_json::to_string(&paragraph).expect("paragraph measurement JSON");
    for forbidden in ["AV", "fontFamily", "fontPath", "sourcePath"] {
        assert!(!trace.contains(forbidden), "paragraph trace leaked {forbidden}");
    }
}

#[test]
fn issue_4968_paragraph_measurement_rolls_back_k0_and_segment_limit() {
    let slot = ExactFontSlot::new(4968, 1);
    let registry = ExactFontSourceRegistry::default();
    let mut transaction = KerningLayoutSession::new(&registry);
    let k0_run = transaction.measure_run(
        slot,
        "AV",
        false,
        vec![0.0, 9.25, 18.5],
        f64::NAN,
        f64::NAN,
    );
    let segment = KerningParagraphSegmentMeasurement {
        start_index: 0,
        end_index: 2,
        slot,
        measurement: k0_run,
    };
    let k0 = compose_kerning_paragraph_measurement(
        2,
        vec![0.0, 9.25, 18.5],
        vec![segment.clone()],
    );
    assert_eq!(
        k0.disposition,
        KerningParagraphMeasurementDisposition::ExistingPositions
    );
    assert_eq!(k0.positions(), [0.0, 9.25, 18.5]);
    assert!(k0.pair_adjusted_positions.is_none());

    let over_limit = compose_kerning_paragraph_measurement(
        2,
        vec![0.0, 9.25, 18.5],
        vec![segment; MAX_KERNING_PARAGRAPH_SEGMENTS + 1],
    );
    assert_eq!(
        over_limit.disposition,
        KerningParagraphMeasurementDisposition::FailClosed
    );
    assert_eq!(
        over_limit.fallback_reason,
        Some(KerningParagraphMeasurementFallbackReason::SegmentLimitExceeded)
    );
    assert!(over_limit.segment_limit_exceeded);
    assert_eq!(over_limit.bounded_segment_count, MAX_KERNING_PARAGRAPH_SEGMENTS);
    assert_eq!(over_limit.positions(), [0.0, 9.25, 18.5]);
    assert!(over_limit.pair_adjusted_positions.is_none());
}

fn paragraph_scalar_style(slot: ExactFontSlot, requested: bool) -> KerningParagraphScalarStyle {
    KerningParagraphScalarStyle {
        slot,
        requested,
        effective_font_size_px: 20.0,
        width_ratio: 1.0,
    }
}

#[test]
fn issue_4968_paragraph_segmentation_honors_slot_control_and_inline_boundaries() {
    let slot_a = ExactFontSlot::new(4968, 1);
    let slot_b = ExactFontSlot::new(4969, 1);
    let mut registry = ExactFontSourceRegistry::default();
    for slot in [slot_a, slot_b] {
        registry
            .register(
                slot,
                ExactFontSource {
                    bytes: EXACT_KERNING_SMOKE,
                    face_index: 0,
                },
            )
            .expect("public exact source");
    }
    let mut transaction = KerningLayoutSession::new(&registry);
    let text = "AVAV\tAV\nAVAV";
    let code_points = text.chars().count();
    let mut scalar_styles = vec![paragraph_scalar_style(slot_a, true); code_points];
    scalar_styles[2] = paragraph_scalar_style(slot_b, true);
    scalar_styles[3] = paragraph_scalar_style(slot_b, true);
    let mut hard_boundaries = vec![false; code_points + 1];
    hard_boundaries[10] = true; // visible text 사이의 inline control 위치
    let base_positions = linear_positions(code_points, 10.0);

    let paragraph = measure_kerning_paragraph_segments(
        text,
        base_positions.clone(),
        &scalar_styles,
        &hard_boundaries,
        &mut transaction,
    );

    assert_eq!(
        paragraph.disposition,
        KerningParagraphMeasurementDisposition::PairAdjusted
    );
    assert_eq!(paragraph.fallback_reason, None);
    assert_eq!(paragraph.bounded_segment_count, 5);
    assert_eq!(paragraph.attempted_segment_count, 5);
    assert_eq!(paragraph.whitespace_fallback_run_count, 0);
    let ranges: Vec<(usize, usize, ExactFontSlot)> = paragraph
        .segments
        .iter()
        .map(|segment| (segment.start_index, segment.end_index, segment.slot))
        .collect();
    assert_eq!(
        ranges,
        vec![
            (0, 2, slot_a),
            (2, 4, slot_b),
            (5, 7, slot_a),
            (8, 10, slot_a),
            (10, 12, slot_a),
        ]
    );
    assert_eq!(paragraph.base_positions, base_positions);
    assert!(paragraph
        .segments
        .iter()
        .all(|segment| segment.measurement.disposition
            == KerningRunMeasurementDisposition::PairAdjusted));
    assert!(!paragraph.segments[0]
        .measurement
        .session
        .as_ref()
        .expect("first source trace")
        .cache_hit);
    assert!(paragraph.segments[1..].iter().all(|segment| segment
        .measurement
        .session
        .as_ref()
        .expect("cached source trace")
        .cache_hit));
}

#[test]
fn issue_4968_paragraph_segmentation_retries_nominal_identity_failure_by_word() {
    let slot = ExactFontSlot::new(4968, 1);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: NOTO_REGULAR,
                face_index: 0,
            },
        )
        .expect("public exact source");
    let mut transaction = KerningLayoutSession::new(&registry);
    let text = "ffi AV";
    let code_points = text.chars().count();
    let scalar_styles = vec![paragraph_scalar_style(slot, true); code_points];
    let hard_boundaries = vec![false; code_points + 1];

    let paragraph = measure_kerning_paragraph_segments(
        text,
        linear_positions(code_points, 10.0),
        &scalar_styles,
        &hard_boundaries,
        &mut transaction,
    );

    assert_eq!(
        paragraph.disposition,
        KerningParagraphMeasurementDisposition::PairAdjusted
    );
    assert_eq!(paragraph.attempted_segment_count, 3);
    assert_eq!(paragraph.whitespace_fallback_run_count, 1);
    assert_eq!(paragraph.bounded_segment_count, 2);
    assert_eq!(
        paragraph
            .segments
            .iter()
            .map(|segment| (segment.start_index, segment.end_index))
            .collect::<Vec<_>>(),
        vec![(0, 3), (4, 6)]
    );
    assert_eq!(
        paragraph.segments[0].measurement.disposition,
        KerningRunMeasurementDisposition::FailClosed
    );
    assert_eq!(
        paragraph.segments[0]
            .measurement
            .candidate
            .as_ref()
            .expect("nominal identity trace")
            .fallback_reason,
        Some(KerningPairCandidateFallbackReason::NominalGlyphIdentityChanged)
    );
    assert_eq!(
        paragraph.segments[1].measurement.disposition,
        KerningRunMeasurementDisposition::PairAdjusted
    );
    assert_eq!(paragraph.range_width(0, 3), Some(30.0));
    assert!(paragraph.range_width(4, 6).expect("AV width") < 20.0);
}

#[test]
fn issue_4968_paragraph_segmentation_rolls_back_execution_budget_atomically() {
    let text = "A".repeat(MAX_KERNING_PARAGRAPH_SEGMENTS + 1);
    let code_points = text.chars().count();
    let slot_a = ExactFontSlot::new(4968, 1);
    let slot_b = ExactFontSlot::new(4969, 1);
    let scalar_styles: Vec<_> = (0..code_points)
        .map(|index| {
            paragraph_scalar_style(if index % 2 == 0 { slot_a } else { slot_b }, false)
        })
        .collect();
    let hard_boundaries = vec![false; code_points + 1];
    let base_positions = linear_positions(code_points, 10.0);
    let registry = ExactFontSourceRegistry::default();
    let mut transaction = KerningLayoutSession::new(&registry);

    let paragraph = measure_kerning_paragraph_segments(
        &text,
        base_positions.clone(),
        &scalar_styles,
        &hard_boundaries,
        &mut transaction,
    );

    assert_eq!(
        paragraph.disposition,
        KerningParagraphMeasurementDisposition::FailClosed
    );
    assert_eq!(
        paragraph.fallback_reason,
        Some(KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded)
    );
    assert!(paragraph.segment_limit_exceeded);
    assert_eq!(
        paragraph.attempted_segment_count,
        MAX_KERNING_PARAGRAPH_SEGMENTS
    );
    assert_eq!(paragraph.bounded_segment_count, 0);
    assert!(paragraph.segments.is_empty(), "partial K0/K1 result leaked");
    assert_eq!(paragraph.positions(), base_positions);
    assert!(paragraph.pair_adjusted_positions.is_none());

    let oversized_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS + 1);
    let oversized_styles =
        vec![paragraph_scalar_style(slot_a, true); MAX_KERNING_RUN_CODE_POINTS + 1];
    let oversized_boundaries = vec![false; MAX_KERNING_RUN_CODE_POINTS + 2];
    let oversized_positions = linear_positions(MAX_KERNING_RUN_CODE_POINTS + 1, 10.0);
    let oversized = measure_kerning_paragraph_segments(
        &oversized_text,
        oversized_positions.clone(),
        &oversized_styles,
        &oversized_boundaries,
        &mut transaction,
    );
    assert_eq!(
        oversized.fallback_reason,
        Some(KerningParagraphMeasurementFallbackReason::CodePointLimitExceeded)
    );
    assert!(oversized.code_point_limit_exceeded);
    assert_eq!(oversized.code_point_count, MAX_KERNING_RUN_CODE_POINTS);
    assert_eq!(oversized.attempted_segment_count, 0);
    assert_eq!(oversized.positions(), oversized_positions);
}
