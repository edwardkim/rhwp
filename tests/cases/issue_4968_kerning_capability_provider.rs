//! Issue #4968 W9-Q3-2..Q3-4: capability, run gate, and pair candidate must fail closed.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;

use kerning::{
    compute_kerning_pair_candidate, decide_kerning_run_gate, inspect_exact_font_kerning,
    identify_exact_font_source, prepare_kerning_pair_engine, resolve_exact_font_source,
    ExactFontSource, ExactFontSourceHandle, ExactFontSourceProvider,
    ExactFontSourceResolutionReason, KerningCapability, KerningCapabilityFallbackReason,
    KerningPairCandidateFallbackReason, KerningPairCandidateStatus, KerningRequest,
    KerningRunFallbackReason, KerningRunGate, KerningSourceSession,
    KerningSourceSessionStatus, MAX_KERNING_ADJACENT_PAIRS, MAX_KERNING_FONT_BYTES,
    MAX_KERNING_RUN_CODE_POINTS, MAX_KERNING_RUN_GLYPHS,
};
use std::cell::Cell;

const NOTO_REGULAR: &[u8] =
    include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const NO_PAIR_TABLE: &[u8] =
    include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

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
