//! Issue #4968 W9-Q3-2..Q3-4: capability, run gate, and pair candidate must fail closed.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;

use kerning::{
    compose_kerning_paragraph_measurement, compute_kerning_pair_candidate,
    compute_kerning_run_measurement, decide_kerning_run_gate, identify_exact_font_source,
    inspect_exact_font_kerning, measure_kerning_paragraph_segments, prepare_kerning_pair_engine,
    resolve_exact_font_source, ExactFontRegistryError, ExactFontRegistryRegistration,
    ExactFontSlot, ExactFontSource, ExactFontSourceHandle, ExactFontSourceProvider,
    ExactFontSourceRegistry, ExactFontSourceResolutionReason, KerningCapability,
    KerningCapabilityFallbackReason, KerningLayoutSession, KerningPairCandidateFallbackReason,
    KerningPairCandidateStatus, KerningParagraphBreakSession,
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

const NOTO_REGULAR: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const NO_PAIR_TABLE: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");
const EXACT_KERNING_SMOKE: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
#[cfg(not(target_arch = "wasm32"))]
const R4E_RUNTIME_FIXTURE: &[u8] = include_bytes!(
    "../../mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.hwpx"
);
#[cfg(not(target_arch = "wasm32"))]
const R4E_RUNTIME_MANIFEST: &str = include_str!(
    "../../mydocs/tech/investigations/issue-4968/fixtures/kerning_runtime_fixture.manifest.json"
);

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

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_runtime_snapshot(core: &rhwp::document_core::DocumentCore) -> serde_json::Value {
    let render_tree = core.build_page_render_tree(0).expect("R4E render tree");
    let layer_tree = core.build_page_layer_tree(0).expect("R4E layer tree");
    let svg = core.render_page_svg_native(0).expect("R4E SVG");
    let canvas_command_count = core
        .render_page_canvas_native(0)
        .expect("R4E Canvas commands");
    let canvaskit: serde_json::Value = serde_json::from_str(
        &core
            .get_canvaskit_replay_plan_native(0, "default")
            .expect("R4E CanvasKit plan"),
    )
    .expect("R4E CanvasKit JSON");

    serde_json::json!({
        "pageCount": core.page_count(),
        "renderTree": serde_json::from_str::<serde_json::Value>(&render_tree.root.to_json())
            .expect("R4E render tree JSON"),
        "layerTree": serde_json::from_str::<serde_json::Value>(&layer_tree.to_json())
            .expect("R4E layer tree JSON"),
        "svg": {
            "bytes": svg.len(),
            "sha256": issue_4968_r4e_sha256_hex(svg.as_bytes()),
        },
        "canvasCommandCount": canvas_command_count,
        "canvasKit": canvaskit,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_positions_for_key(
    value: &serde_json::Value,
    stable_source_key: &str,
) -> Option<Vec<serde_json::Value>> {
    if value.get("type").and_then(serde_json::Value::as_str) == Some("textRun")
        && value
            .pointer("/source/stableSourceKey")
            .and_then(serde_json::Value::as_str)
            == Some(stable_source_key)
    {
        return value
            .get("positions")
            .and_then(serde_json::Value::as_array)
            .cloned();
    }
    if let Some(array) = value.as_array() {
        for child in array {
            if let Some(positions) = issue_4968_r4e_positions_for_key(child, stable_source_key) {
                return Some(positions);
            }
        }
    } else if let Some(object) = value.as_object() {
        for child in object.values() {
            if let Some(positions) = issue_4968_r4e_positions_for_key(child, stable_source_key) {
                return Some(positions);
            }
        }
    }
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_registration_attempt(
    core: &mut rhwp::document_core::DocumentCore,
    char_shape_id: u32,
    language_index: usize,
    font_bytes: &[u8],
    face_index: u32,
) -> serde_json::Value {
    match core.register_exact_font_source_native(
        char_shape_id,
        language_index,
        font_bytes,
        face_index,
    ) {
        Ok(value) => serde_json::json!({
            "ok": true,
            "value": serde_json::from_str::<serde_json::Value>(&value)
                .expect("R4E registration JSON"),
        }),
        Err(error) => serde_json::json!({
            "ok": false,
            "error": error.to_string(),
        }),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_registration_failure_case(
    name: &str,
    font_bytes: &[u8],
    face_index: u32,
    language_index: usize,
) -> serde_json::Value {
    let mut core = rhwp::document_core::DocumentCore::from_bytes(R4E_RUNTIME_FIXTURE)
        .expect("R4E failure fixture");
    let before = issue_4968_r4e_runtime_snapshot(&core);
    let registration =
        issue_4968_r4e_registration_attempt(&mut core, 8, language_index, font_bytes, face_index);
    let after = issue_4968_r4e_runtime_snapshot(&core);
    serde_json::json!({
        "case": name,
        "registration": registration,
        "before": before,
        "after": after,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn issue_4968_r4e_registration_failure_matrix() -> Vec<serde_json::Value> {
    let oversized = vec![0; MAX_KERNING_FONT_BYTES + 1];
    let mut cases = vec![
        issue_4968_r4e_registration_failure_case("malformed-sfnt", b"not-an-sfnt", 0, 1),
        issue_4968_r4e_registration_failure_case("pair-table-unsupported", NO_PAIR_TABLE, 0, 1),
        issue_4968_r4e_registration_failure_case(
            "unavailable-face-index",
            EXACT_KERNING_SMOKE,
            1,
            1,
        ),
        issue_4968_r4e_registration_failure_case(
            "invalid-language-index",
            EXACT_KERNING_SMOKE,
            0,
            7,
        ),
        issue_4968_r4e_registration_failure_case("font-byte-limit-exceeded", &oversized, 0, 1),
    ];

    let mut conflict_core = rhwp::document_core::DocumentCore::from_bytes(R4E_RUNTIME_FIXTURE)
        .expect("R4E conflict fixture");
    issue_4968_r4e_registration_attempt(&mut conflict_core, 8, 1, EXACT_KERNING_SMOKE, 0);
    let before = issue_4968_r4e_runtime_snapshot(&conflict_core);
    let registration =
        issue_4968_r4e_registration_attempt(&mut conflict_core, 8, 1, NO_PAIR_TABLE, 0);
    let after = issue_4968_r4e_runtime_snapshot(&conflict_core);
    cases.push(serde_json::json!({
        "case": "slot-conflict",
        "registration": registration,
        "before": before,
        "after": after,
    }));

    for case in &cases {
        assert_eq!(
            case.get("before"),
            case.get("after"),
            "failure matrix must preserve the pre-attempt render state: {}",
            case.get("case")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
        );
    }
    for index in 0..3 {
        assert_eq!(
            cases[index].pointer("/registration/ok"),
            Some(&serde_json::json!(true))
        );
    }
    for (index, reason) in [
        (3, "invalid-language-index"),
        (4, "font-byte-limit-exceeded"),
        (5, "slot-conflict"),
    ] {
        assert_eq!(
            cases[index].pointer("/registration/ok"),
            Some(&serde_json::json!(false))
        );
        assert!(cases[index]
            .pointer("/registration/error")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|error| error.contains(reason)));
    }
    cases
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_r4e_native_runtime_probe_registers_exact_slots_and_changes_only_k1() {
    let manifest: serde_json::Value =
        serde_json::from_str(R4E_RUNTIME_MANIFEST).expect("R4E manifest JSON");
    let fixture_sha256 = issue_4968_r4e_sha256_hex(R4E_RUNTIME_FIXTURE);
    let font_sha256 = issue_4968_r4e_sha256_hex(EXACT_KERNING_SMOKE);
    assert_eq!(
        manifest
            .get("inputSha256")
            .and_then(serde_json::Value::as_str),
        Some(fixture_sha256.as_str())
    );
    assert_eq!(
        manifest
            .pointer("/semantic/fontSource/sha256")
            .and_then(serde_json::Value::as_str),
        Some(font_sha256.as_str())
    );

    let mut core = rhwp::document_core::DocumentCore::from_bytes(R4E_RUNTIME_FIXTURE)
        .expect("R4E runtime fixture");
    let k0 = issue_4968_r4e_runtime_snapshot(&core);
    let slots = manifest
        .pointer("/semantic/exactSourceRegistration/slots")
        .and_then(serde_json::Value::as_array)
        .expect("R4E exact slots");
    let mut registrations = Vec::with_capacity(slots.len());
    for slot in slots {
        let char_shape_id = slot
            .get("charShapeId")
            .and_then(serde_json::Value::as_u64)
            .expect("R4E char shape id") as u32;
        let language_index = slot
            .get("languageIndex")
            .and_then(serde_json::Value::as_u64)
            .expect("R4E language index") as usize;
        let registration = core
            .register_exact_font_source_native(
                char_shape_id,
                language_index,
                EXACT_KERNING_SMOKE,
                0,
            )
            .expect("R4E exact source registration");
        registrations.push(
            serde_json::from_str::<serde_json::Value>(&registration)
                .expect("R4E registration JSON"),
        );
    }
    let k1 = issue_4968_r4e_runtime_snapshot(&core);

    assert_eq!(slots.len(), 18);
    assert_eq!(k0.get("pageCount"), Some(&serde_json::json!(1)));
    assert_eq!(k1.get("pageCount"), Some(&serde_json::json!(1)));
    assert_eq!(k0.get("canvasCommandCount"), k1.get("canvasCommandCount"));
    assert_eq!(
        registrations
            .last()
            .and_then(|value| value.pointer("/registry/slotCount")),
        Some(&serde_json::json!(18))
    );
    assert_eq!(
        registrations
            .last()
            .and_then(|value| value.pointer("/registry/sourceCount")),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        registrations
            .last()
            .and_then(|value| value.pointer("/registry/totalSourceBytes")),
        Some(&serde_json::json!(EXACT_KERNING_SMOKE.len()))
    );

    let k0_layer = k0.get("layerTree").expect("R4E K0 layer tree");
    let k1_layer = k1.get("layerTree").expect("R4E K1 layer tree");
    let off_key = "section:0/para:1/char:0";
    let on_key = "section:0/para:2/char:0";
    assert_eq!(
        issue_4968_r4e_positions_for_key(k0_layer, off_key),
        issue_4968_r4e_positions_for_key(k1_layer, off_key),
        "K0 row must remain identical after exact registration"
    );
    assert_ne!(
        issue_4968_r4e_positions_for_key(k0_layer, on_key),
        issue_4968_r4e_positions_for_key(k1_layer, on_key),
        "K1 row must consume the registered exact pair positions"
    );
    assert_ne!(k0.get("svg"), k1.get("svg"));
    assert_eq!(
        k1.pointer("/canvasKit/summary/hiddenOverlayViolations"),
        Some(&serde_json::json!(0))
    );

    let probe = serde_json::json!({
        "schemaVersion": 1,
        "issue": 4968,
        "stage": "W9-Q3-5R4E-1",
        "projectionContractSha256": manifest.get("projectionContractSha256"),
        "registration": registrations,
        "k0": k0,
        "k1": k1,
        "failureMatrix": issue_4968_r4e_registration_failure_matrix(),
    });
    if let Some(path) = std::env::var_os("RHWP_4968_R4E_NATIVE_PROBE") {
        let mut bytes = serde_json::to_vec_pretty(&probe).expect("R4E native probe JSON");
        bytes.push(b'\n');
        std::fs::write(path, bytes).expect("write R4E native probe");
    }
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
    assert!(
        table_len >= kern_table.len(),
        "replacement table is too small"
    );
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
    assert_eq!(unsupported_json["fallbackReason"], "pair-table-unsupported");
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
    let gate = decide_kerning_run_gate(true, text, text.chars().count(), &second.capability);
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
    assert_eq!(missing_first.status, KerningSourceSessionStatus::FailClosed);
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
    let gate = decide_kerning_run_gate(true, text, text.chars().count(), &trace.capability);
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
    assert!(
        eligible_json.get("text").is_none(),
        "trace must omit source text"
    );

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
    let code_points = decide_kerning_run_gate(true, &oversized_text, MAX_KERNING_RUN_GLYPHS, &gpos);
    assert_eq!(code_points.gate, KerningRunGate::FailClosed);
    assert_eq!(code_points.code_point_count, MAX_KERNING_RUN_CODE_POINTS);
    assert!(code_points.code_point_limit_exceeded);
    assert_eq!(code_points.glyph_count, MAX_KERNING_RUN_GLYPHS);
    assert_eq!(code_points.candidate_pair_count, MAX_KERNING_ADJACENT_PAIRS);
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
    let pair_gate =
        decide_kerning_run_gate(true, pair_text, pair_text.chars().count(), &capability);
    let pair = compute_kerning_pair_candidate(pair_text, &engine, &pair_gate);
    assert_eq!(pair.status, KerningPairCandidateStatus::AdjustmentCandidate);
    assert_eq!(pair.capability, KerningCapability::GposKern);
    assert_eq!(pair.glyph_count, 11);
    assert_eq!(pair.examined_pair_count, 10);
    assert_eq!(pair.total_x_advance_delta, -94);
    assert!(pair.adjusted_position_count > 0);
    assert_eq!(pair.fallback_reason, None);
    let pair_json = serde_json::to_value(&pair).expect("pair candidate JSON");
    assert_eq!(pair_json["status"], "adjustment-candidate");
    assert!(
        pair_json.get("text").is_none(),
        "trace must omit source text"
    );
    assert!(
        pair_json.get("applied").is_none(),
        "candidate must not claim application"
    );

    let no_pair_text = "HH";
    let no_pair_gate = decide_kerning_run_gate(
        true,
        no_pair_text,
        no_pair_text.chars().count(),
        &capability,
    );
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
    let rtl_gate = decide_kerning_run_gate(true, rtl_text, rtl_text.chars().count(), &capability);
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
    let bounded_gate =
        decide_kerning_run_gate(true, &bounded_text, MAX_KERNING_RUN_GLYPHS, &capability);
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
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
    assert_eq!(
        provider.calls.get(),
        0,
        "missing handle must not query provider"
    );

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
    assert_eq!(
        provider.calls.get(),
        0,
        "oversized run must stop before source"
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
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

    let first = transaction.measure_run(slot, "AV", true, vec![0.0, 10.0, 20.0], 20.0, 1.0);
    assert_eq!(
        first.disposition,
        KerningRunMeasurementDisposition::PairAdjusted
    );
    assert!(!first.session.as_ref().expect("first trace").cache_hit);

    let second = transaction.measure_run(slot, "AV", true, vec![0.0, 10.0, 20.0], 20.0, 1.0);
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
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
    let left = transaction.measure_run(slot, "AV", true, vec![0.0, 10.0, 20.0], 20.0, 1.0);
    let right = transaction.measure_run(slot, "AV", true, vec![0.0, 10.0, 20.0], 20.0, 1.0);
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
        assert!(
            !trace.contains(forbidden),
            "paragraph trace leaked {forbidden}"
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_paragraph_measurement_rolls_back_k0_and_segment_limit() {
    let slot = ExactFontSlot::new(4968, 1);
    let registry = ExactFontSourceRegistry::default();
    let mut transaction = KerningLayoutSession::new(&registry);
    let k0_run =
        transaction.measure_run(slot, "AV", false, vec![0.0, 9.25, 18.5], f64::NAN, f64::NAN);
    let segment = KerningParagraphSegmentMeasurement {
        start_index: 0,
        end_index: 2,
        slot,
        measurement: k0_run,
    };
    let k0 = compose_kerning_paragraph_measurement(2, vec![0.0, 9.25, 18.5], vec![segment.clone()]);
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
    assert_eq!(
        over_limit.bounded_segment_count,
        MAX_KERNING_PARAGRAPH_SEGMENTS
    );
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
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
    assert!(
        !paragraph.segments[0]
            .measurement
            .session
            .as_ref()
            .expect("first source trace")
            .cache_hit
    );
    assert!(paragraph.segments[1..].iter().all(|segment| segment
        .measurement
        .session
        .as_ref()
        .expect("cached source trace")
        .cache_hit));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_paragraph_segmentation_rolls_back_execution_budget_atomically() {
    let text = "A".repeat(MAX_KERNING_PARAGRAPH_SEGMENTS + 1);
    let code_points = text.chars().count();
    let slot_a = ExactFontSlot::new(4968, 1);
    let slot_b = ExactFontSlot::new(4969, 1);
    let scalar_styles: Vec<_> = (0..code_points)
        .map(|index| paragraph_scalar_style(if index % 2 == 0 { slot_a } else { slot_b }, false))
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_long_word_and_line_boundary_consume_one_paragraph_measurement() {
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
    let text = "AVAV";
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

    let initial_prefix_width = paragraph.range_width(0, 3).expect("initial prefix");
    let mut line_measurement = KerningParagraphBreakSession::new(
        text,
        &scalar_styles,
        &hard_boundaries,
        &paragraph,
        &mut transaction,
    )
    .expect("same paragraph transaction");
    assert_eq!(
        line_measurement.range_width(0, 4),
        paragraph.range_width(0, 4),
        "token total must read the paragraph-owned positions"
    );
    assert_eq!(
        line_measurement.boundary_width(0, 1),
        Some(10.0),
        "a pair split after A must remove the crossing adjustment"
    );
    assert_eq!(line_measurement.boundary_pair_adjustment(0, 1), Some(0.0));
    let boundary_prefix_width = line_measurement
        .boundary_width(0, 3)
        .expect("boundary-safe prefix");
    assert!(boundary_prefix_width > initial_prefix_width);
    let available_width = (initial_prefix_width + boundary_prefix_width) / 2.0;

    let decision = line_measurement
        .find_fitting_end(0, 4, available_width)
        .expect("bounded line decision");
    assert_eq!(decision.initial_end_index, 3);
    assert_eq!(decision.final_end_index, 2);
    assert!(!decision.overflow_forced);
    assert!(decision.final_width <= available_width);
    assert_eq!(
        decision.final_width,
        line_measurement.boundary_width(0, 2).unwrap()
    );
    assert_eq!(line_measurement.failed_reason(), None);

    let trace = serde_json::to_string(&decision).expect("boundary decision JSON");
    for forbidden in ["AVAV", "fontFamily", "fontPath", "sourcePath"] {
        assert!(
            !trace.contains(forbidden),
            "boundary trace leaked {forbidden}"
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn issue_4968_line_boundary_remeasurement_shares_the_segment_budget() {
    let text = "A".repeat(MAX_KERNING_PARAGRAPH_SEGMENTS);
    let code_points = text.chars().count();
    let slot_a = ExactFontSlot::new(4968, 1);
    let slot_b = ExactFontSlot::new(4969, 1);
    let scalar_styles: Vec<_> = (0..code_points)
        .map(|index| paragraph_scalar_style(if index % 2 == 0 { slot_a } else { slot_b }, false))
        .collect();
    let hard_boundaries = vec![false; code_points + 1];
    let registry = ExactFontSourceRegistry::default();
    let mut transaction = KerningLayoutSession::new(&registry);
    let paragraph = measure_kerning_paragraph_segments(
        &text,
        linear_positions(code_points, 10.0),
        &scalar_styles,
        &hard_boundaries,
        &mut transaction,
    );
    assert_eq!(
        paragraph.attempted_segment_count,
        MAX_KERNING_PARAGRAPH_SEGMENTS
    );
    assert_eq!(
        paragraph.disposition,
        KerningParagraphMeasurementDisposition::ExistingPositions
    );

    let mut line_measurement = KerningParagraphBreakSession::new(
        &text,
        &scalar_styles,
        &hard_boundaries,
        &paragraph,
        &mut transaction,
    )
    .expect("bounded paragraph measurement");
    assert_eq!(line_measurement.boundary_width(0, 1), None);
    assert_eq!(
        line_measurement.failed_reason(),
        Some(KerningParagraphMeasurementFallbackReason::SegmentExecutionLimitExceeded)
    );
    assert_eq!(
        line_measurement.attempted_segment_count(),
        MAX_KERNING_PARAGRAPH_SEGMENTS
    );
    assert_eq!(line_measurement.find_fitting_end(0, 2, 20.0), None);
}

#[cfg(not(target_arch = "wasm32"))]
fn public_edit_reflow_line_starts(with_exact_source: bool, body_width_hwp: u32) -> Vec<u32> {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::paragraph::{CharShapeRef, Paragraph};

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.kerning = true;
    char_shape.base_size = 1_500;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);
    let mut paragraph = Paragraph::new_empty();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    document.sections[0].paragraphs = vec![paragraph];
    document.sections[0].section_def.page_def.width = body_width_hwp + 2_000;
    document.sections[0].section_def.page_def.height = 200_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);
    if with_exact_source {
        core.register_exact_font_source_native(char_shape_id, 1, EXACT_KERNING_SMOKE, 0)
            .expect("register exact public face");
    }
    core.insert_text_native(0, 0, 0, &"AV".repeat(40))
        .expect("edit reflow");
    core.document().sections[0].paragraphs[0]
        .line_segs
        .iter()
        .map(|segment| segment.text_start)
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_edit_reflow_consumes_exact_boundary_measurement() {
    let (body_width_hwp, k0, k1) = (7_000..=13_000)
        .step_by(100)
        .find_map(|body_width_hwp| {
            let k0 = public_edit_reflow_line_starts(false, body_width_hwp);
            let k1 = public_edit_reflow_line_starts(true, body_width_hwp);
            (k1 != k0).then_some((body_width_hwp, k0, k1))
        })
        .expect("bounded public width ladder must expose an exact AV boundary change");
    assert_eq!(
        k1,
        public_edit_reflow_line_starts(true, body_width_hwp),
        "same exact generation must produce deterministic edit boundaries"
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_public_av_run_lengths(core: &mut rhwp::document_core::DocumentCore) -> Vec<usize> {
    fn collect(node: &serde_json::Value, lengths: &mut Vec<usize>) {
        if node.get("type").and_then(|value| value.as_str()) == Some("TextRun") {
            if let Some(text) = node.get("text").and_then(|value| value.as_str()) {
                let av_count = text
                    .chars()
                    .filter(|character| matches!(character, 'A' | 'V'))
                    .count();
                if av_count > 0 {
                    lengths.push(av_count);
                }
            }
        }
        if let Some(children) = node.get("children").and_then(|value| value.as_array()) {
            for child in children {
                collect(child, lengths);
            }
        }
    }

    let mut lengths = Vec::new();
    for page_number in 0..core.page_count() {
        let page = core
            .build_page_render_tree(page_number as u32)
            .expect("fresh public page tree");
        let root: serde_json::Value =
            serde_json::from_str(&page.root.to_json()).expect("page tree JSON");
        collect(&root, &mut lengths);
    }
    lengths
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
struct PublicAvLayoutRun {
    text: String,
    bbox_x: f64,
    scalar_count: usize,
    bbox_width: f64,
    layout_positions: Option<Vec<f64>>,
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_public_av_layout_runs(
    core: &mut rhwp::document_core::DocumentCore,
) -> Vec<PublicAvLayoutRun> {
    fn collect(node: &rhwp::renderer::render_tree::RenderNode, runs: &mut Vec<PublicAvLayoutRun>) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            let replay_text = run.display_or_text();
            let scalar_count = replay_text
                .chars()
                .filter(|character| matches!(character, 'A' | 'V'))
                .count();
            if scalar_count > 0 {
                runs.push(PublicAvLayoutRun {
                    text: replay_text.to_string(),
                    bbox_x: node.bbox.x,
                    scalar_count,
                    bbox_width: node.bbox.width,
                    layout_positions: run.layout_positions.clone(),
                });
            }
        }
        for child in &node.children {
            collect(child, runs);
        }
    }

    let mut runs = Vec::new();
    for page_number in 0..core.page_count() {
        let page = core
            .build_page_render_tree(page_number as u32)
            .expect("fresh public page tree");
        collect(&page.root, &mut runs);
    }
    runs
}

#[cfg(not(target_arch = "wasm32"))]
fn set_public_av_paragraph(paragraph: &mut rhwp::model::paragraph::Paragraph, char_shape_id: u32) {
    use rhwp::model::paragraph::CharShapeRef;

    let text = "AV".repeat(40);
    paragraph.text = text.clone();
    paragraph.char_offsets = (0..text.chars().count() as u32).collect();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    paragraph.line_segs.clear();
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_render_av_core(
    with_exact_source: bool,
    body_width_hwp: u32,
) -> rhwp::document_core::DocumentCore {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::paragraph::{CharShapeRef, Paragraph};

    let text = "AV".repeat(40);
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.kerning = true;
    char_shape.base_size = 1_500;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = text.clone();
    paragraph.char_offsets = (0..text.chars().count() as u32).collect();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    paragraph.line_segs.clear();
    document.sections[0].paragraphs = vec![paragraph];
    document.sections[0].section_def.page_def.width = body_width_hwp + 2_000;
    document.sections[0].section_def.page_def.height = 200_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);
    if with_exact_source {
        core.register_exact_font_source_native(char_shape_id, 1, EXACT_KERNING_SMOKE, 0)
            .expect("register exact public face");
    }

    core
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_render_av_run_lengths(with_exact_source: bool, body_width_hwp: u32) -> Vec<usize> {
    let mut core = public_fresh_render_av_core(with_exact_source, body_width_hwp);
    collect_public_av_run_lengths(&mut core)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_fresh_pagination_and_page_tree_share_exact_boundaries() {
    let (body_width_hwp, k0, k1) = (7_000..=13_000)
        .step_by(100)
        .find_map(|body_width_hwp| {
            let k0 = public_fresh_render_av_run_lengths(false, body_width_hwp);
            let k1 = public_fresh_render_av_run_lengths(true, body_width_hwp);
            (!k0.is_empty() && k1 != k0).then_some((body_width_hwp, k0, k1))
        })
        .expect("bounded public width ladder must expose a fresh exact boundary change");
    assert!(!k0.is_empty() && !k1.is_empty());
    assert_eq!(
        k1,
        public_fresh_render_av_run_lengths(true, body_width_hwp),
        "fresh exact page-tree boundaries must be deterministic"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_final_emitted_runs_publish_exact_positions_once() {
    let body_width_hwp = (7_000..=13_000)
        .step_by(100)
        .find(|body_width_hwp| {
            public_fresh_render_av_run_lengths(false, *body_width_hwp)
                != public_fresh_render_av_run_lengths(true, *body_width_hwp)
        })
        .expect("bounded public width ladder must expose an exact AV boundary change");

    let mut k0_core = public_fresh_render_av_core(false, body_width_hwp);
    let k0_runs = collect_public_av_layout_runs(&mut k0_core);
    assert!(!k0_runs.is_empty());
    assert!(k0_runs.iter().all(|run| run.layout_positions.is_none()));

    let mut k1_core = public_fresh_render_av_core(true, body_width_hwp);
    let k1_runs = collect_public_av_layout_runs(&mut k1_core);
    let adjusted: Vec<&PublicAvLayoutRun> = k1_runs
        .iter()
        .filter(|run| run.layout_positions.is_some())
        .collect();
    assert!(!adjusted.is_empty(), "K1 must publish final run positions");
    for run in adjusted {
        let positions = run.layout_positions.as_deref().expect("K1 positions");
        assert_eq!(positions.len(), run.scalar_count + 1);
        assert_eq!(positions.first().copied(), Some(0.0));
        assert!(positions.windows(2).all(|pair| pair[0] <= pair[1]));
        let final_position = positions.last().copied().expect("final position");
        assert!((final_position - run.bbox_width).abs() < 1e-9);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_visual_consumers_replay_the_published_positions() {
    use rhwp::renderer::canvas::{CanvasCommand, CanvasRenderer};
    use rhwp::renderer::html::HtmlRenderer;
    use rhwp::renderer::svg::SvgRenderer;
    use rhwp::renderer::{Renderer, TextStyle};

    let mut malformed_canvas = CanvasRenderer::new();
    malformed_canvas.draw_text_positioned(
        "AV",
        0.0,
        0.0,
        &TextStyle::default(),
        Some(&[0.0, f64::NAN, 12.0]),
    );
    assert!(matches!(
        malformed_canvas.commands(),
        [CanvasCommand::FillText(text, 0.0, 0.0)] if text == "AV"
    ));

    let body_width_hwp = 10_000;
    let mut k0_core = public_fresh_render_av_core(false, body_width_hwp);
    let k0_tree = k0_core
        .build_page_render_tree(0)
        .expect("K0 public page tree");
    let mut k0_canvas = CanvasRenderer::new();
    k0_canvas.render_tree(&k0_tree);
    assert!(k0_canvas
        .commands()
        .iter()
        .all(|command| !matches!(command, CanvasCommand::FillTextPositioned(..))));

    let mut k1_core = public_fresh_render_av_core(true, body_width_hwp);
    let k1_runs = collect_public_av_layout_runs(&mut k1_core);
    let adjusted = k1_runs
        .iter()
        .find(|run| run.layout_positions.is_some())
        .expect("K1 adjusted public run");
    let expected_positions = adjusted
        .layout_positions
        .as_deref()
        .expect("K1 published positions");

    let k1_tree = k1_core
        .build_page_render_tree(0)
        .expect("K1 public page tree");
    let mut k1_canvas = CanvasRenderer::new();
    k1_canvas.render_tree(&k1_tree);
    assert!(k1_canvas.commands().iter().any(|command| matches!(
        command,
        CanvasCommand::FillTextPositioned(text, _, _, positions)
            if text == &adjusted.text && positions == expected_positions
    )));

    let mut svg = SvgRenderer::new();
    svg.render_tree(&k1_tree);
    let second_scalar_x = adjusted.bbox_x + expected_positions[1];
    assert!(
        svg.output().contains(&format!("x=\"{second_scalar_x}\"")),
        "SVG must consume the second published scalar position"
    );

    let mut html = HtmlRenderer::new();
    html.render_tree(&k1_tree);
    assert!(html.output().contains("text-run-positioned"));
    let second_scalar_left = adjusted.bbox_x + expected_positions[1];
    assert!(
        html.output()
            .contains(&format!("left:{second_scalar_left}px;")),
        "HTML must consume the second published scalar position"
    );

    let k1_layer = k1_core
        .build_page_layer_tree(0)
        .expect("K1 public layer tree");
    let mut layer_canvas = CanvasRenderer::new();
    layer_canvas.render_layer_tree(&k1_layer);
    assert!(layer_canvas.commands().iter().any(|command| matches!(
        command,
        CanvasCommand::FillTextPositioned(text, _, _, positions)
            if text == &adjusted.text && positions == expected_positions
    )));

    let expected_json_positions = expected_positions
        .iter()
        .map(|position| format!("{position:.3}"))
        .collect::<Vec<_>>()
        .join(",");
    assert!(k1_layer
        .to_json()
        .contains(&format!("\"positions\":[{expected_json_positions}]")));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_bounded_positions_replay_and_canvaskit_work_are_fail_closed() {
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile, TextDecorationKind};
    use rhwp::renderer::canvas::{CanvasCommand, CanvasRenderer};
    use rhwp::renderer::canvaskit_policy::{
        analyze_canvaskit_document_preflight, estimate_canvaskit_page_lowering_work,
        CanvasKitBoundedWorkCount, CanvasKitPreflightPageBuild, CanvasKitReplayMode,
    };
    use rhwp::renderer::render_tree::{
        BoundingBox, FieldMarkerType, PageRenderTree, RenderNode, RenderNodeType, TextRunNode,
    };
    use rhwp::renderer::{Renderer, TextStyle};

    fn text_run(text: String, layout_positions: Option<Vec<f64>>) -> TextRunNode {
        TextRunNode {
            text,
            style: TextStyle {
                font_family: "RHWP bounded replay fixture".to_string(),
                font_size: 12.0,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 12.0,
            field_marker: FieldMarkerType::None,
            layout_positions,
            display_text: None,
        }
    }

    fn page_tree(run: TextRunNode) -> PageRenderTree {
        let bbox = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let mut tree = PageRenderTree::new(0, 100.0, 100.0);
        tree.root
            .children
            .push(RenderNode::new(1, RenderNodeType::TextRun(run), bbox));
        tree
    }

    fn layer_tree(run: TextRunNode, decoration_only: bool) -> PageLayerTree {
        let bbox = BoundingBox::new(0.0, 0.0, 100.0, 20.0);
        let op = if decoration_only {
            PaintOp::text_decoration(bbox, run, TextDecorationKind::Underline)
        } else {
            PaintOp::text_run(bbox, run)
        };
        PageLayerTree::new(100.0, 100.0, LayerNode::leaf(bbox, None, vec![op]))
    }

    let k0_json =
        serde_json::to_string(&text_run("AV".to_string(), None)).expect("serialize K0 text run");
    assert!(!k0_json.contains("layout_positions"));
    let valid_positions = vec![0.0, 7.5, 14.0];
    let k1_json = serde_json::to_string(&text_run("AV".to_string(), Some(valid_positions.clone())))
        .expect("serialize K1 text run");
    assert!(k1_json.contains("\"layout_positions\":[0.0,7.5,14.0]"));

    let mut valid_tree_canvas = CanvasRenderer::new();
    valid_tree_canvas.render_tree(&page_tree(text_run(
        "AV".to_string(),
        Some(valid_positions.clone()),
    )));
    assert!(valid_tree_canvas.commands().iter().any(|command| matches!(
        command,
        CanvasCommand::FillTextPositioned(text, _, _, positions)
            if text == "AV" && positions == &valid_positions
    )));

    for malformed in [
        vec![1.0, 7.5, 14.0],
        vec![0.0, f64::NAN, 14.0],
        vec![0.0, 8.0, 7.0],
        vec![0.0, 7.5],
    ] {
        let mut malformed_tree_canvas = CanvasRenderer::new();
        malformed_tree_canvas.render_tree(&page_tree(text_run("AV".to_string(), Some(malformed))));
        assert!(malformed_tree_canvas
            .commands()
            .iter()
            .any(|command| matches!(command, CanvasCommand::FillText(text, _, _) if text == "AV")));
        assert!(malformed_tree_canvas
            .commands()
            .iter()
            .all(|command| !matches!(command, CanvasCommand::FillTextPositioned(..))));
    }

    let mut mismatched_run = text_run("AV".to_string(), Some(valid_positions));
    mismatched_run.display_text = Some("A".to_string());
    let mut mismatched_tree_canvas = CanvasRenderer::new();
    mismatched_tree_canvas.render_tree(&page_tree(mismatched_run));
    assert!(mismatched_tree_canvas
        .commands()
        .iter()
        .any(|command| matches!(command, CanvasCommand::FillText(text, _, _) if text == "A")));
    assert!(mismatched_tree_canvas
        .commands()
        .iter()
        .all(|command| !matches!(command, CanvasCommand::FillTextPositioned(..))));

    let max_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS);
    let max_positions = (0..=MAX_KERNING_RUN_CODE_POINTS)
        .map(|value| value as f64)
        .collect::<Vec<_>>();
    let mut max_canvas = CanvasRenderer::new();
    max_canvas.draw_text_positioned(
        &max_text,
        0.0,
        0.0,
        &TextStyle::default(),
        Some(&max_positions),
    );
    assert!(matches!(
        max_canvas.commands(),
        [CanvasCommand::FillTextPositioned(_, 0.0, 0.0, positions)]
            if positions.len() == MAX_KERNING_RUN_CODE_POINTS + 1
    ));

    let over_text = "A".repeat(MAX_KERNING_RUN_CODE_POINTS + 1);
    let over_positions = (0..=MAX_KERNING_RUN_CODE_POINTS + 1)
        .map(|value| value as f64)
        .collect::<Vec<_>>();
    let malformed_positions = [
        vec![0.0, f64::INFINITY, 2.0],
        vec![0.0, 2.0, 1.0],
        vec![0.0, 1.0],
    ];
    let mut over_canvas = CanvasRenderer::new();
    over_canvas.draw_text_positioned(
        &over_text,
        0.0,
        0.0,
        &TextStyle::default(),
        Some(&over_positions),
    );
    assert!(matches!(
        over_canvas.commands(),
        [CanvasCommand::FillText(_, 0.0, 0.0)]
    ));
    for malformed in &malformed_positions {
        let mut canvas = CanvasRenderer::new();
        canvas.draw_text_positioned("AV", 0.0, 0.0, &TextStyle::default(), Some(malformed));
        assert!(matches!(
            canvas.commands(),
            [CanvasCommand::FillText(_, 0.0, 0.0)]
        ));
    }

    let baseline_tree = page_tree(text_run(max_text.clone(), None));
    let positioned_tree = page_tree(text_run(max_text.clone(), Some(max_positions.clone())));
    let CanvasKitBoundedWorkCount::Complete(baseline_work) =
        estimate_canvaskit_page_lowering_work(&baseline_tree, u32::MAX)
    else {
        panic!("bounded baseline pre-lowering work");
    };
    let CanvasKitBoundedWorkCount::Complete(positioned_work) =
        estimate_canvaskit_page_lowering_work(&positioned_tree, u32::MAX)
    else {
        panic!("bounded positioned pre-lowering work");
    };
    assert!(positioned_work > baseline_work);
    assert_eq!(
        estimate_canvaskit_page_lowering_work(&positioned_tree, baseline_work),
        CanvasKitBoundedWorkCount::Exceeded
    );

    let oversized_tree = page_tree(text_run("A".to_string(), Some(vec![0.0; 140_000])));
    assert_eq!(
        estimate_canvaskit_page_lowering_work(&oversized_tree, u32::MAX),
        CanvasKitBoundedWorkCount::Exceeded
    );

    let baseline_layer = layer_tree(text_run(max_text.clone(), None), false);
    let positioned_layer = layer_tree(
        text_run(max_text.clone(), Some(max_positions.clone())),
        false,
    );
    let baseline_preflight = analyze_canvaskit_document_preflight(
        1,
        CanvasKitReplayMode::Default,
        RenderProfile::Screen,
        move |_, _| {
            Ok::<_, &'static str>(CanvasKitPreflightPageBuild::Complete {
                tree: Box::new(baseline_layer.clone()),
                prelower_work_units: 0,
            })
        },
    );
    let positioned_preflight = analyze_canvaskit_document_preflight(
        1,
        CanvasKitReplayMode::Default,
        RenderProfile::Screen,
        move |_, _| {
            Ok::<_, &'static str>(CanvasKitPreflightPageBuild::Complete {
                tree: Box::new(positioned_layer.clone()),
                prelower_work_units: 0,
            })
        },
    );
    assert!(positioned_preflight.scanned_work_units > baseline_preflight.scanned_work_units);

    let bounded_json = layer_tree(text_run(max_text, Some(max_positions)), true).to_json();
    let bounded_value: serde_json::Value =
        serde_json::from_str(&bounded_json).expect("bounded positions JSON");
    let bounded_decoration = &bounded_value["root"]["ops"][0]["decoration"];
    assert_eq!(bounded_decoration["positionsComplete"], true);
    assert_eq!(
        bounded_decoration["positions"]
            .as_array()
            .expect("bounded positions")
            .len(),
        MAX_KERNING_RUN_CODE_POINTS + 1
    );
    assert_eq!(
        bounded_decoration["positions"][MAX_KERNING_RUN_CODE_POINTS],
        MAX_KERNING_RUN_CODE_POINTS as f64
    );

    let oversized_json = layer_tree(text_run(over_text, Some(over_positions)), true).to_json();
    let oversized_value: serde_json::Value =
        serde_json::from_str(&oversized_json).expect("oversized positions JSON");
    let oversized_decoration = &oversized_value["root"]["ops"][0]["decoration"];
    assert_eq!(oversized_decoration["positionsComplete"], false);
    assert_eq!(
        oversized_decoration["positions"]
            .as_array()
            .expect("bounded oversized positions")
            .len(),
        MAX_KERNING_RUN_CODE_POINTS + 1
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_portable_glyph_run_reuses_the_published_positions() {
    use rhwp::paint::{
        lower_font_native_glyph_sidecars, EmbeddedFontFace, LayerNode, LayerNodeKind, PaintOp,
        ResourceArena,
    };
    use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
    use rhwp::renderer::TextStyle;

    let published = vec![0.0, 7.0, 13.0];
    let bbox = BoundingBox::new(10.0, 20.0, 13.0, 16.0);
    let run = TextRunNode {
        text: "AV".to_string(),
        style: TextStyle {
            font_family: "RHWP Exact Kerning Smoke".to_string(),
            font_size: 16.0,
            ..Default::default()
        },
        char_shape_id: Some(7),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: None,
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 12.0,
        field_marker: FieldMarkerType::None,
        layout_positions: Some(published.clone()),
        display_text: None,
    };
    let mut root = LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]);
    let mut resources = ResourceArena::default();
    let report = lower_font_native_glyph_sidecars(
        &mut root,
        &mut resources,
        &[EmbeddedFontFace {
            char_shape_id: 7,
            language_index: 1,
            family: "RHWP Exact Kerning Smoke",
            alternate_family: None,
            bytes: EXACT_KERNING_SMOKE,
            face_index: 0,
        }],
    );
    assert_eq!(report.emitted_glyph_runs, 1);

    let LayerNodeKind::Leaf { ops } = root.kind else {
        panic!("expected portable leaf");
    };
    let PaintOp::GlyphRun { run, .. } = &ops[1] else {
        panic!("expected portable GlyphRun");
    };
    assert_eq!(run.positions[0].x, published[0]);
    assert_eq!(run.positions[1].x, published[1]);
    let advances = run.advances.as_ref().expect("portable advances");
    assert_eq!(advances[0].dx, published[1] - published[0]);
    assert_eq!(advances[1].dx, published[2] - published[1]);
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_table_cell_av_core(
    with_exact_source: bool,
    content_width_hwp: u32,
) -> rhwp::document_core::DocumentCore {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::CharShapeRef;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.kerning = true;
    char_shape.base_size = 1_500;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);
    document.sections[0].paragraphs[0].char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    document.sections[0].section_def.page_def.width = 50_000;
    document.sections[0].section_def.page_def.height = 200_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);

    let cell_width = content_width_hwp + 1_020;
    core.create_table_ex_native(0, 0, 0, 1, 1, false, Some(&[cell_width]), None)
        .expect("public one-cell table");
    let mut document = core.document().clone();
    let table = document.sections[0]
        .paragraphs
        .iter_mut()
        .find_map(|paragraph| {
            paragraph
                .controls
                .iter_mut()
                .find_map(|control| match control {
                    Control::Table(table) => Some(table.as_mut()),
                    _ => None,
                })
        })
        .expect("created public table");
    let cell = table.cells.first_mut().expect("created public cell");
    cell.width = cell_width;
    set_public_av_paragraph(
        cell.paragraphs.first_mut().expect("created cell paragraph"),
        char_shape_id,
    );
    core.set_document(document);
    if with_exact_source {
        core.register_exact_font_source_native(char_shape_id, 1, EXACT_KERNING_SMOKE, 0)
            .expect("register exact public face");
    }
    core
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_table_cell_av_run_lengths(
    with_exact_source: bool,
    content_width_hwp: u32,
) -> Vec<usize> {
    let mut core = public_fresh_table_cell_av_core(with_exact_source, content_width_hwp);
    collect_public_av_run_lengths(&mut core)
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_text_box_av_core(
    with_exact_source: bool,
    content_width_hwp: u32,
) -> rhwp::document_core::DocumentCore {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::CharShapeRef;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();
    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.kerning = true;
    char_shape.base_size = 1_500;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);
    document.sections[0].paragraphs[0].char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    document.sections[0].section_def.page_def.width = 50_000;
    document.sections[0].section_def.page_def.height = 200_000;
    document.sections[0].section_def.page_def.margin_left = 1_000;
    document.sections[0].section_def.page_def.margin_right = 1_000;
    document.sections[0].section_def.page_def.margin_top = 1_000;
    document.sections[0].section_def.page_def.margin_bottom = 1_000;
    core.set_document(document);

    core.create_shape_control_native(
        0,
        0,
        0,
        content_width_hwp + 1_020,
        80_000,
        0,
        0,
        false,
        "InFrontOfText",
        "textbox",
        false,
        false,
        &[],
    )
    .expect("public text box");
    let mut document = core.document().clone();
    let text_box = document.sections[0]
        .paragraphs
        .iter_mut()
        .find_map(|paragraph| {
            paragraph
                .controls
                .iter_mut()
                .find_map(|control| match control {
                    Control::Shape(shape) => shape
                        .drawing_mut()
                        .and_then(|drawing| drawing.text_box.as_mut()),
                    _ => None,
                })
        })
        .expect("created public text box");
    set_public_av_paragraph(
        text_box
            .paragraphs
            .first_mut()
            .expect("created text-box paragraph"),
        char_shape_id,
    );
    core.set_document(document);
    if with_exact_source {
        core.register_exact_font_source_native(char_shape_id, 1, EXACT_KERNING_SMOKE, 0)
            .expect("register exact public face");
    }
    core
}

#[cfg(not(target_arch = "wasm32"))]
fn public_fresh_text_box_av_run_lengths(
    with_exact_source: bool,
    content_width_hwp: u32,
) -> Vec<usize> {
    let mut core = public_fresh_text_box_av_core(with_exact_source, content_width_hwp);
    collect_public_av_run_lengths(&mut core)
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn issue_4968_fresh_table_cell_and_text_box_share_exact_boundaries() {
    type ContainerRunLengths = fn(bool, u32) -> Vec<usize>;
    type ContainerCore = fn(bool, u32) -> rhwp::document_core::DocumentCore;
    let containers: [(&str, ContainerRunLengths, ContainerCore); 2] = [
        (
            "table-cell",
            public_fresh_table_cell_av_run_lengths,
            public_fresh_table_cell_av_core,
        ),
        (
            "text-box",
            public_fresh_text_box_av_run_lengths,
            public_fresh_text_box_av_core,
        ),
    ];

    for (label, run_lengths, build_core) in containers {
        let (content_width_hwp, k0, k1) = (7_000..=13_000)
            .step_by(100)
            .find_map(|content_width_hwp| {
                let k0 = run_lengths(false, content_width_hwp);
                let k1 = run_lengths(true, content_width_hwp);
                (!k0.is_empty() && k1 != k0).then_some((content_width_hwp, k0, k1))
            })
            .unwrap_or_else(|| panic!("{label} width ladder must expose an exact AV boundary"));
        assert!(!k0.is_empty() && !k1.is_empty(), "{label} must render text");
        assert_eq!(
            k1,
            run_lengths(true, content_width_hwp),
            "{label} exact boundary must be deterministic"
        );
        let mut k1_core = build_core(true, content_width_hwp);
        let k1_layout_runs = collect_public_av_layout_runs(&mut k1_core);
        let adjusted: Vec<&PublicAvLayoutRun> = k1_layout_runs
            .iter()
            .filter(|run| run.layout_positions.is_some())
            .collect();
        assert!(!adjusted.is_empty(), "{label} must publish exact positions");
        for run in adjusted {
            let positions = run.layout_positions.as_deref().expect("K1 positions");
            assert_eq!(positions.len(), run.scalar_count + 1, "{label}");
            assert!(positions.windows(2).all(|pair| pair[0] <= pair[1]));
            assert!(
                (positions.last().copied().expect("end") - run.bbox_width).abs() < 1e-9,
                "{label} bbox must consume final positions"
            );
        }
    }
}
