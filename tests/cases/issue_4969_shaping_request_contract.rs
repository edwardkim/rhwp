//! Issue #4969 W10-Q1/Q4-A~C: exact-source shaping request는 bounded·fail-closed하고,
//! vertical output과 dormant geometry는 하나의 결정적 glyph/position/bbox owner를 가져야 한다.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_vertical.rs"]
mod shaping_vertical;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use shaping::{
    canonicalize_shaping_request, shape_bounded_request, shape_canonical_request_with_face,
    terminal_shaping_attempt, validate_shaping_request, BoundedShapingAttemptLedger,
    ShapingAttemptLedgerStatus, ShapingDirection, ShapingDisposition, ShapingExactSource,
    ShapingFeature, ShapingRejectReason, ShapingRequest, ShapingVariation, ShapingWritingMode,
    TerminalShapingDisposition, MAX_SHAPING_ATTEMPT_TRACE_RECORDS, MAX_SHAPING_FEATURES,
    MAX_SHAPING_FONT_BYTES, MAX_SHAPING_TEXT_CODE_POINTS, MAX_SHAPING_VARIATION_AXES,
};
use shaping_vertical::{
    adapt_hwp5_vertical_intent, adapt_hwpx_vertical_intent,
    prepare_bounded_vertical_glyph_publication_shadow,
    prepare_dormant_vertical_shaping_transaction, BoundedVerticalHwp5TableCellSidecar,
    DormantVerticalShapingRejectReason, DormantVerticalShapingRequest, TypedVerticalIntent,
    VerticalGlyphPublicationLeafInput, VerticalGlyphPublicationShadowRejectReason,
    VerticalGlyphTransform, VerticalIntentDisposition, VerticalIntentSurface,
    VerticalLatinOrientation, VerticalLegacyGeometry, VerticalPoint, VerticalRect,
    VerticalRunClass, VerticalShapingContext, VerticalShapingContextRejectReason,
    VerticalShapingContextRequest, VerticalShapingPageSidecars, VerticalShapingSidecarRejectReason,
    NOTO_SANS_KR_REGULAR_SHA256,
};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const NOTO: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const HAPPINESS: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");

fn request<'a>(bytes: &'a [u8], text: &'a str) -> ShapingRequest<'a> {
    ShapingRequest {
        source: Some(ShapingExactSource {
            bytes,
            face_index: 0,
            portable: true,
        }),
        text,
        direction: ShapingDirection::LeftToRight,
        writing_mode: ShapingWritingMode::HorizontalTb,
        script: Some("Hang"),
        language: Some("ko"),
        features: &[],
        variations: &[],
    }
}

fn vertical_request<'a>(
    bytes: &'a [u8],
    text: &'a str,
    script: &'a str,
    language: &'a str,
    features: &'a [ShapingFeature],
) -> ShapingRequest<'a> {
    ShapingRequest {
        source: Some(ShapingExactSource {
            bytes,
            face_index: 0,
            portable: true,
        }),
        text,
        direction: ShapingDirection::TopToBottom,
        writing_mode: ShapingWritingMode::VerticalRl,
        script: Some(script),
        language: Some(language),
        features,
        variations: &[],
    }
}

fn assert_rejected(
    request: &ShapingRequest<'_>,
    disposition: ShapingDisposition,
    reason: ShapingRejectReason,
) {
    let decision = validate_shaping_request(request);
    assert_eq!(decision.disposition, disposition);
    assert_eq!(decision.reason, Some(reason));
}

#[test]
fn issue_4969_exact_sources_expose_only_proven_capabilities() {
    let noto = validate_shaping_request(&request(NOTO, "office"));
    assert_eq!(noto.disposition, ShapingDisposition::Requested);
    assert!(noto.has_gsub && noto.has_gpos && noto.has_vertical_metrics);
    assert!(!noto.has_variations);

    let mut rtl = request(NOTO, "abc");
    rtl.direction = ShapingDirection::RightToLeft;
    assert_eq!(
        validate_shaping_request(&rtl).disposition,
        ShapingDisposition::Requested
    );

    let mut source_han = request(SOURCE_HAN, "ᄒᆞᆫ글");
    source_han.direction = ShapingDirection::TopToBottom;
    source_han.writing_mode = ShapingWritingMode::VerticalRl;
    let source_han = validate_shaping_request(&source_han);
    assert_eq!(source_han.disposition, ShapingDisposition::Requested);
    assert!(source_han.has_vertical_metrics && source_han.has_vorg);

    let axes = [
        ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let mut happiness = request(HAPPINESS, "가변 Typography");
    happiness.variations = &axes;
    let happiness = validate_shaping_request(&happiness);
    assert_eq!(happiness.disposition, ShapingDisposition::Requested);
    assert!(happiness.has_variations);
    assert!(!happiness.has_vertical_metrics);
}

#[test]
fn issue_4969_source_and_orientation_fail_closed() {
    let mut missing = request(NOTO, "가");
    missing.source = None;
    assert_rejected(
        &missing,
        ShapingDisposition::Unsupported,
        ShapingRejectReason::SourceUnavailable,
    );

    assert_rejected(
        &request(b"not-a-font", "가"),
        ShapingDisposition::Malformed,
        ShapingRejectReason::MalformedSfnt,
    );

    let mut local = request(NOTO, "가");
    local.source.as_mut().expect("source").portable = false;
    assert_rejected(
        &local,
        ShapingDisposition::NonPortable,
        ShapingRejectReason::NonPortableSource,
    );

    let mut mismatched = request(NOTO, "가");
    mismatched.direction = ShapingDirection::TopToBottom;
    assert_rejected(
        &mismatched,
        ShapingDisposition::Malformed,
        ShapingRejectReason::DirectionWritingModeMismatch,
    );

    let mut no_vertical = request(HAPPINESS, "가");
    no_vertical.direction = ShapingDirection::TopToBottom;
    no_vertical.writing_mode = ShapingWritingMode::VerticalLr;
    assert_rejected(
        &no_vertical,
        ShapingDisposition::Unsupported,
        ShapingRejectReason::VerticalMetricsUnavailable,
    );
}

#[test]
fn issue_4969_tags_and_axes_are_canonical_and_unambiguous() {
    let mut malformed_script = request(NOTO, "가");
    malformed_script.script = Some("Korean");
    assert_rejected(
        &malformed_script,
        ShapingDisposition::Malformed,
        ShapingRejectReason::MalformedScriptTag,
    );

    let mut malformed_language = request(NOTO, "가");
    malformed_language.language = Some("ko_kr");
    assert_rejected(
        &malformed_language,
        ShapingDisposition::Malformed,
        ShapingRejectReason::MalformedLanguageTag,
    );

    let duplicate_features = [
        ShapingFeature {
            tag: "liga".into(),
            value: 1,
        },
        ShapingFeature {
            tag: "liga".into(),
            value: 0,
        },
    ];
    let mut duplicate_feature = request(NOTO, "office");
    duplicate_feature.features = &duplicate_features;
    assert_rejected(
        &duplicate_feature,
        ShapingDisposition::Malformed,
        ShapingRejectReason::DuplicateFeatureTag,
    );

    let malformed_features = [ShapingFeature {
        tag: "lig".into(),
        value: 1,
    }];
    let mut malformed_feature = request(NOTO, "office");
    malformed_feature.features = &malformed_features;
    assert_rejected(
        &malformed_feature,
        ShapingDisposition::Malformed,
        ShapingRejectReason::MalformedFeatureTag,
    );

    for (axes, disposition, reason) in [
        (
            vec![
                ShapingVariation {
                    tag: "wght".into(),
                    value: 400.0,
                },
                ShapingVariation {
                    tag: "wght".into(),
                    value: 900.0,
                },
            ],
            ShapingDisposition::Malformed,
            ShapingRejectReason::DuplicateVariationAxis,
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: f32::NAN,
            }],
            ShapingDisposition::Malformed,
            ShapingRejectReason::VariationValueNonFinite,
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: f32::INFINITY,
            }],
            ShapingDisposition::Malformed,
            ShapingRejectReason::VariationValueNonFinite,
        ),
        (
            vec![ShapingVariation {
                tag: "wgt".into(),
                value: 400.0,
            }],
            ShapingDisposition::Malformed,
            ShapingRejectReason::MalformedVariationTag,
        ),
        (
            vec![ShapingVariation {
                tag: "wdth".into(),
                value: 100.0,
            }],
            ShapingDisposition::Unsupported,
            ShapingRejectReason::VariationAxisUnsupported,
        ),
        (
            vec![ShapingVariation {
                tag: "wght".into(),
                value: 901.0,
            }],
            ShapingDisposition::Malformed,
            ShapingRejectReason::VariationValueOutOfRange,
        ),
    ] {
        let mut invalid = request(HAPPINESS, "가변");
        invalid.variations = &axes;
        assert_rejected(&invalid, disposition, reason);
    }
}

#[test]
fn issue_4969_payload_counts_are_bounded_before_shaping() {
    let oversized_font = vec![0_u8; MAX_SHAPING_FONT_BYTES + 1];
    assert_rejected(
        &request(&oversized_font, "가"),
        ShapingDisposition::BoundedLimit,
        ShapingRejectReason::FontByteLimitExceeded,
    );

    let oversized_text = "가".repeat(MAX_SHAPING_TEXT_CODE_POINTS + 1);
    assert_rejected(
        &request(NOTO, &oversized_text),
        ShapingDisposition::BoundedLimit,
        ShapingRejectReason::TextCodePointLimitExceeded,
    );

    let features = (0..=MAX_SHAPING_FEATURES)
        .map(|_| ShapingFeature {
            tag: "liga".into(),
            value: 1,
        })
        .collect::<Vec<_>>();
    let mut feature_limit = request(NOTO, "office");
    feature_limit.features = &features;
    assert_rejected(
        &feature_limit,
        ShapingDisposition::BoundedLimit,
        ShapingRejectReason::FeatureLimitExceeded,
    );

    let axes = (0..=MAX_SHAPING_VARIATION_AXES)
        .map(|_| ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        })
        .collect::<Vec<_>>();
    let mut axis_limit = request(HAPPINESS, "가변");
    axis_limit.variations = &axes;
    assert_rejected(
        &axis_limit,
        ShapingDisposition::BoundedLimit,
        ShapingRejectReason::VariationAxisLimitExceeded,
    );
}

#[test]
fn issue_4969_identity_sorts_axes_but_preserves_feature_order() {
    let axes_forward = [
        ShapingVariation {
            tag: "wght".into(),
            value: 650.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let axes_reverse = [axes_forward[1].clone(), axes_forward[0].clone()];
    let features_forward = [
        ShapingFeature {
            tag: "liga".into(),
            value: 1,
        },
        ShapingFeature {
            tag: "kern".into(),
            value: 0,
        },
    ];
    let features_reverse = [features_forward[1].clone(), features_forward[0].clone()];

    let mut forward = request(HAPPINESS, "가변 Typography");
    forward.language = Some("KO-KR");
    forward.features = &features_forward;
    forward.variations = &axes_forward;
    let forward = canonicalize_shaping_request(&forward).expect("canonical request");

    let mut reordered_axes = request(HAPPINESS, "다른 원문");
    reordered_axes.language = Some("ko-kr");
    reordered_axes.features = &features_forward;
    reordered_axes.variations = &axes_reverse;
    let reordered_axes = canonicalize_shaping_request(&reordered_axes).expect("canonical request");
    assert_eq!(forward, reordered_axes);
    assert_eq!(
        forward.settings_sha256,
        "e62f3422b5da9dd24f7e88504440e916ce4808dcd10dc7ea7d24d4a2616e38be"
    );
    assert_eq!(forward.language.as_deref(), Some("ko-kr"));
    assert_eq!(forward.variations[0].tag, "opsz");
    assert_eq!(forward.variations[1].tag, "wght");

    let mut reordered_features = request(HAPPINESS, "가변 Typography");
    reordered_features.language = Some("ko-kr");
    reordered_features.features = &features_reverse;
    reordered_features.variations = &axes_forward;
    let reordered_features =
        canonicalize_shaping_request(&reordered_features).expect("canonical request");
    assert_ne!(forward.settings_sha256, reordered_features.settings_sha256);
    assert_eq!(reordered_features.features[0].tag, "kern");
    assert_eq!(reordered_features.features[1].tag, "liga");
}

#[test]
fn issue_4969_q3_a_effective_default_axes_share_the_empty_identity() {
    let explicit_defaults = [
        ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 400.0,
        },
    ];
    let reversed_defaults = [explicit_defaults[1].clone(), explicit_defaults[0].clone()];
    let wght_default = [explicit_defaults[0].clone()];
    let opsz_default = [explicit_defaults[1].clone()];

    let empty =
        canonicalize_shaping_request(&request(HAPPINESS, "가변")).expect("empty default instance");
    assert!(empty.variations.is_empty());

    for axes in [
        explicit_defaults.as_slice(),
        reversed_defaults.as_slice(),
        wght_default.as_slice(),
        opsz_default.as_slice(),
    ] {
        let mut candidate = request(HAPPINESS, "가변");
        candidate.variations = axes;
        let candidate = canonicalize_shaping_request(&candidate).expect("effective default");
        assert_eq!(candidate, empty, "explicit fvar defaults must be omitted");
        assert!(candidate.variations.is_empty());
    }
}

#[test]
fn issue_4969_q3_a_explicit_defaults_shape_as_the_empty_instance() {
    let defaults = [
        ShapingVariation {
            tag: "opsz".into(),
            value: 400.0,
        },
        ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        },
    ];
    let empty_request = request(HAPPINESS, "Typography");
    let expected = shape_bounded_request(&empty_request);
    let mut explicit_request = request(HAPPINESS, "Typography");
    explicit_request.variations = &defaults;
    let actual = shape_bounded_request(&explicit_request);

    assert_eq!(actual.disposition, ShapingDisposition::Applied);
    assert_eq!(actual.identity, expected.identity);
    assert_eq!(actual.glyphs, expected.glyphs);
}

fn shape_with_shared_face(
    face: &mut rustybuzz::Face<'_>,
    request: &ShapingRequest<'_>,
) -> shaping::ShapingOutputDecision {
    let identity = canonicalize_shaping_request(request).expect("canonical request");
    shape_canonical_request_with_face(request, identity, face)
}

#[test]
fn issue_4969_q3_a_default_after_title_does_not_inherit_axes() {
    let title_axes = [
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let mut title = request(HAPPINESS, "가변");
    title.variations = &title_axes;
    let default_probe = request(HAPPINESS, "Typography");
    let expected_default = shape_bounded_request(&default_probe);
    let mut shared_face = rustybuzz::Face::from_slice(HAPPINESS, 0).expect("variable face");

    let title_output = shape_with_shared_face(&mut shared_face, &title);
    assert_eq!(title_output.disposition, ShapingDisposition::Applied);
    let actual_default = shape_with_shared_face(&mut shared_face, &default_probe);

    assert_eq!(actual_default.disposition, ShapingDisposition::Applied);
    assert_eq!(actual_default.identity, expected_default.identity);
    assert_eq!(actual_default.glyphs, expected_default.glyphs);
}

#[test]
fn issue_4969_q3_a_partial_axis_requests_do_not_leak_between_instances() {
    let weight_axes = [ShapingVariation {
        tag: "wght".into(),
        value: 650.0,
    }];
    let optical_axes = [ShapingVariation {
        tag: "opsz".into(),
        value: 650.0,
    }];
    let mut weight = request(HAPPINESS, "가변");
    weight.variations = &weight_axes;
    let mut optical_probe = request(HAPPINESS, "Typography");
    optical_probe.variations = &optical_axes;
    let expected_optical = shape_bounded_request(&optical_probe);
    let mut shared_face = rustybuzz::Face::from_slice(HAPPINESS, 0).expect("variable face");

    let weight_output = shape_with_shared_face(&mut shared_face, &weight);
    assert_eq!(weight_output.disposition, ShapingDisposition::Applied);
    let actual_optical = shape_with_shared_face(&mut shared_face, &optical_probe);

    assert_eq!(actual_optical.disposition, ShapingDisposition::Applied);
    assert_eq!(actual_optical.identity, expected_optical.identity);
    assert_eq!(actual_optical.glyphs, expected_optical.glyphs);
}

fn glyph_ids(output: &shaping::ShapingOutputDecision) -> Vec<u32> {
    output.glyphs.iter().map(|glyph| glyph.glyph_id).collect()
}

fn clusters(output: &shaping::ShapingOutputDecision) -> Vec<u32> {
    output
        .glyphs
        .iter()
        .map(|glyph| glyph.cluster_utf8)
        .collect()
}

fn x_advances(output: &shaping::ShapingOutputDecision) -> Vec<i32> {
    output.glyphs.iter().map(|glyph| glyph.x_advance).collect()
}

fn glyph_records(output: &shaping::ShapingOutputDecision) -> Vec<(u32, u32, i32, i32, i32, i32)> {
    output
        .glyphs
        .iter()
        .map(|glyph| {
            (
                glyph.glyph_id,
                glyph.cluster_utf8,
                glyph.x_advance,
                glyph.y_advance,
                glyph.x_offset,
                glyph.y_offset,
            )
        })
        .collect()
}

#[test]
fn issue_4969_q4_a_exact_vertical_metrics_and_origins_are_deterministic() {
    let noto_cjk = shape_bounded_request(&vertical_request(NOTO, "한글", "Hang", "ko", &[]));
    assert_eq!(noto_cjk.disposition, ShapingDisposition::Applied);
    assert_eq!(
        glyph_records(&noto_cjk),
        [
            (11232, 0, 0, -1000, -460, -880),
            (1156, 3, 0, -1000, -460, -880),
        ]
    );
    let noto_identity = noto_cjk.identity.as_ref().expect("Noto vertical identity");
    assert_eq!(noto_identity.direction, "ttb");
    assert_eq!(noto_identity.writing_mode, "vertical-rl");
    assert_eq!(
        noto_identity.settings_sha256,
        "8777a129be5e352b4727cf247b63de2b9457b46be09074fcc1eb6b6bfa9d4808"
    );

    let noto_latin = shape_bounded_request(&vertical_request(NOTO, "AB", "Latn", "en", &[]));
    assert_eq!(noto_latin.disposition, ShapingDisposition::Applied);
    assert_eq!(
        glyph_records(&noto_latin),
        [(34, 0, 0, -1000, -304, -880), (35, 1, 0, -1000, -328, -880),]
    );

    let source_han = shape_bounded_request(&vertical_request(
        SOURCE_HAN,
        "\u{1112}\u{119e}\u{11ab}글",
        "Hang",
        "ko",
        &[],
    ));
    assert_eq!(source_han.disposition, ShapingDisposition::Applied);
    assert_eq!(
        glyph_records(&source_han),
        [
            (614, 0, 0, -1000, -483, -880),
            (1230, 9, 0, -1000, -483, -880),
            (1497, 9, 0, 0, 483, 120),
            (2085, 9, 0, 0, 483, 120),
        ]
    );
    assert_eq!(
        source_han
            .identity
            .as_ref()
            .expect("Source Han vertical identity")
            .settings_sha256,
        "e41dd5fc7b332e7367802c508cc7b07cb193bd92903731b3fe4637071ac3de6f"
    );
}

#[test]
fn issue_4969_q4_a_writing_mode_identity_does_not_rewrite_vertical_glyphs() {
    let rl = shape_bounded_request(&vertical_request(NOTO, "한글", "Hang", "ko", &[]));
    let mut lr_request = vertical_request(NOTO, "한글", "Hang", "ko", &[]);
    lr_request.writing_mode = ShapingWritingMode::VerticalLr;
    let lr = shape_bounded_request(&lr_request);

    assert_eq!(rl.glyphs, lr.glyphs);
    let rl_identity = rl.identity.expect("vertical-rl identity");
    let lr_identity = lr.identity.expect("vertical-lr identity");
    assert_eq!(rl_identity.writing_mode, "vertical-rl");
    assert_eq!(lr_identity.writing_mode, "vertical-lr");
    assert_eq!(
        lr_identity.settings_sha256,
        "0f9548dfa3d703f2aa253bb36545791438d785d8c164552de9fbfd83deb461c9"
    );
    assert_ne!(rl_identity.settings_sha256, lr_identity.settings_sha256);
}

#[test]
fn issue_4969_q4_a_vert_and_vrt2_feature_policy_is_explicit() {
    let vertical_off = [
        ShapingFeature {
            tag: "vert".into(),
            value: 0,
        },
        ShapingFeature {
            tag: "vrt2".into(),
            value: 0,
        },
    ];
    let vertical_on = [
        ShapingFeature {
            tag: "vert".into(),
            value: 1,
        },
        ShapingFeature {
            tag: "vrt2".into(),
            value: 1,
        },
    ];
    let default = shape_bounded_request(&vertical_request(NOTO, "—…", "Hani", "ja", &[]));
    let off = shape_bounded_request(&vertical_request(NOTO, "—…", "Hani", "ja", &vertical_off));
    let on = shape_bounded_request(&vertical_request(NOTO, "—…", "Hani", "ja", &vertical_on));

    assert_eq!(
        glyph_records(&default),
        [
            (197, 0, 0, -1000, -447, -880),
            (11826, 3, 0, -1000, -500, -880),
        ]
    );
    assert_eq!(default.glyphs, on.glyphs);
    assert_eq!(
        glyph_records(&off),
        [
            (197, 0, 0, -1000, -447, -880),
            (206, 3, 0, -1000, -500, -880),
        ]
    );
    assert_ne!(default.glyphs, off.glyphs);
    assert_eq!(
        default
            .identity
            .expect("default feature identity")
            .settings_sha256,
        "ff220fc653825ac2e1219cbedff9c9949a4bcc55aefcacc6ce0e42e0335b6c15"
    );
    assert_eq!(
        off.identity
            .expect("disabled feature identity")
            .settings_sha256,
        "63a70ea1144669216d62c33be5cf2f3d53bd7eeff4dfa45f8b46c7b32b1e5169"
    );
    assert_eq!(
        on.identity
            .expect("enabled feature identity")
            .settings_sha256,
        "c37b7bef80d887899ee27592f2a0872644988f086b6b67225b15ee45fd321a6e"
    );
}

#[test]
fn issue_4969_bounded_output_matches_horizontal_and_old_hangul_oracles() {
    let liga_off = [ShapingFeature {
        tag: "liga".into(),
        value: 0,
    }];
    let mut office_off = request(NOTO, "office");
    office_off.script = Some("Latn");
    office_off.language = Some("en");
    office_off.features = &liga_off;
    let office_off = shape_bounded_request(&office_off);
    assert_eq!(office_off.disposition, ShapingDisposition::Applied);
    assert_eq!(glyph_ids(&office_off), [80, 71, 71, 74, 68, 70]);
    assert_eq!(clusters(&office_off), [0, 1, 2, 3, 4, 5]);
    assert_eq!(x_advances(&office_off), [601, 325, 325, 275, 483, 554]);

    let liga_on = [ShapingFeature {
        tag: "liga".into(),
        value: 1,
    }];
    let mut office_on = request(NOTO, "office");
    office_on.script = Some("Latn");
    office_on.language = Some("en");
    office_on.features = &liga_on;
    let office_on = shape_bounded_request(&office_on);
    assert_eq!(office_on.disposition, ShapingDisposition::Applied);
    assert_eq!(glyph_ids(&office_on), [80, 11819, 68, 70]);
    assert_eq!(clusters(&office_on), [0, 1, 4, 5]);
    assert_eq!(x_advances(&office_on), [606, 918, 483, 554]);
    assert!(!serde_json::to_string(&office_on)
        .expect("serialize output")
        .contains("office"));

    let source_han = shape_bounded_request(&request(SOURCE_HAN, "ᄒᆞᆫ글"));
    assert_eq!(source_han.disposition, ShapingDisposition::Applied);
    assert_eq!(glyph_ids(&source_han), [614, 1230, 1497, 2085]);
    assert_eq!(clusters(&source_han), [0, 9, 9, 9]);
    assert_eq!(x_advances(&source_han), [966, 966, 0, 0]);
}

#[test]
fn issue_4969_variation_changes_advances_without_changing_glyph_ids() {
    let axes_400 = [
        ShapingVariation {
            tag: "wght".into(),
            value: 400.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 400.0,
        },
    ];
    let axes_900 = [
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
    ];
    let mut request_400 = request(HAPPINESS, "가변 Typography");
    request_400.variations = &axes_400;
    let output_400 = shape_bounded_request(&request_400);
    let mut request_900 = request(HAPPINESS, "가변 Typography");
    request_900.variations = &axes_900;
    let output_900 = shape_bounded_request(&request_900);

    let expected_glyphs = [221, 1359, 1, 53, 90, 81, 80, 72, 83, 66, 81, 73, 90];
    assert_eq!(output_400.disposition, ShapingDisposition::Applied);
    assert_eq!(output_900.disposition, ShapingDisposition::Applied);
    assert_eq!(glyph_ids(&output_400), expected_glyphs);
    assert_eq!(glyph_ids(&output_900), expected_glyphs);
    assert_eq!(
        x_advances(&output_400),
        [920, 920, 230, 579, 496, 578, 597, 536, 462, 561, 578, 586, 496]
    );
    assert_eq!(
        x_advances(&output_900),
        [930, 930, 240, 598, 563, 614, 633, 611, 500, 612, 614, 608, 563]
    );
    assert_ne!(
        output_400
            .identity
            .as_ref()
            .expect("identity")
            .settings_sha256,
        output_900
            .identity
            .as_ref()
            .expect("identity")
            .settings_sha256
    );
}

#[test]
fn issue_4969_output_preserves_structured_rejection() {
    let mut missing = request(NOTO, "가");
    missing.source = None;
    let output = shape_bounded_request(&missing);
    assert_eq!(output.disposition, ShapingDisposition::Unsupported);
    assert_eq!(output.reason, Some(ShapingRejectReason::SourceUnavailable));
    assert!(output.identity.is_none());
    assert_eq!(output.glyph_count, 0);
    assert!(output.glyphs.is_empty());
}

#[test]
fn issue_4969_utf8_cluster_offset_is_bounded_by_code_point_limit() {
    let max_width_text = "\u{10ffff}".repeat(MAX_SHAPING_TEXT_CODE_POINTS);
    assert_eq!(max_width_text.len(), MAX_SHAPING_TEXT_CODE_POINTS * 4);
    let output = shape_bounded_request(&request(NOTO, &max_width_text));
    assert_eq!(output.disposition, ShapingDisposition::Applied);
    assert_eq!(output.glyph_count, MAX_SHAPING_TEXT_CODE_POINTS);
    assert_eq!(
        output
            .glyphs
            .last()
            .expect("last bounded glyph")
            .cluster_utf8,
        16_380
    );
}

#[test]
fn issue_4969_terminal_attempt_owns_applied_output_without_text_trace() {
    let attempt = terminal_shaping_attempt(17, &request(SOURCE_HAN, "ᄒᆞᆫ글"));
    assert!(attempt.is_applied());
    assert_eq!(attempt.trace.attempt_id, 17);
    assert_eq!(
        attempt.trace.disposition,
        TerminalShapingDisposition::Applied
    );
    assert_eq!(attempt.trace.reason, None);
    assert_eq!(attempt.trace.glyph_count, 4);
    assert!(attempt.trace.settings_sha256.is_some());
    assert_eq!(
        attempt.trace.font_source_sha256.as_deref(),
        Some("2f86ef9a52acb6d1dad9d915843239123b635d97edd88fd0573a88ffcb4e16f1")
    );
    let applied = attempt.applied.as_ref().expect("applied payload");
    assert_eq!(applied.glyphs.len(), attempt.trace.glyph_count);
    assert_eq!(applied.glyphs[0].glyph_id, 614);
    assert_eq!(
        applied.identity.settings_sha256,
        attempt
            .trace
            .settings_sha256
            .as_deref()
            .expect("settings hash")
    );

    let trace_json = serde_json::to_string(&attempt.trace).expect("terminal trace JSON");
    for forbidden in ["ᄒᆞᆫ글", "SourceHanSerif", "fontBytes", "glyphs", "/home/"] {
        assert!(!trace_json.contains(forbidden), "trace leaked {forbidden}");
    }
}

#[test]
fn issue_4969_terminal_rejections_keep_typed_disposition_and_optional_hashes() {
    let mut missing = request(NOTO, "가");
    missing.source = None;
    let missing = terminal_shaping_attempt(1, &missing);
    assert_eq!(
        missing.trace.disposition,
        TerminalShapingDisposition::Unsupported
    );
    assert_eq!(
        missing.trace.reason,
        Some(ShapingRejectReason::SourceUnavailable)
    );
    assert!(missing.trace.settings_sha256.is_none());
    assert!(missing.trace.font_source_sha256.is_none());
    assert!(missing.applied.is_none());

    let malformed = terminal_shaping_attempt(2, &request(b"not-a-font", "가"));
    assert_eq!(
        malformed.trace.disposition,
        TerminalShapingDisposition::Malformed
    );
    assert_eq!(
        malformed.trace.reason,
        Some(ShapingRejectReason::MalformedSfnt)
    );

    let mut non_portable_request = request(NOTO, "가");
    non_portable_request
        .source
        .as_mut()
        .expect("source")
        .portable = false;
    let non_portable = terminal_shaping_attempt(3, &non_portable_request);
    assert_eq!(
        non_portable.trace.disposition,
        TerminalShapingDisposition::NonPortable
    );

    let oversized_text = "가".repeat(MAX_SHAPING_TEXT_CODE_POINTS + 1);
    let bounded = terminal_shaping_attempt(4, &request(NOTO, &oversized_text));
    assert_eq!(
        bounded.trace.disposition,
        TerminalShapingDisposition::BoundedLimit
    );
    assert_eq!(
        bounded.trace.reason,
        Some(ShapingRejectReason::TextCodePointLimitExceeded)
    );
    assert_eq!(bounded.trace.glyph_count, 0);
}

#[test]
fn issue_4969_attempt_ledger_truncates_without_retaining_applied_payload() {
    let attempt = terminal_shaping_attempt(9, &request(NOTO, "office"));
    let mut ledger = BoundedShapingAttemptLedger::default();
    for _ in 0..MAX_SHAPING_ATTEMPT_TRACE_RECORDS {
        ledger.record(&attempt.trace);
    }
    assert_eq!(ledger.status(), ShapingAttemptLedgerStatus::Complete);
    assert_eq!(ledger.record_count(), MAX_SHAPING_ATTEMPT_TRACE_RECORDS);
    ledger.record(&attempt.trace);
    ledger.record(&attempt.trace);
    assert_eq!(ledger.status(), ShapingAttemptLedgerStatus::Truncated);
    assert_eq!(ledger.record_count(), MAX_SHAPING_ATTEMPT_TRACE_RECORDS);
    assert_eq!(ledger.omitted_record_count(), 2);
    assert_eq!(ledger.record_limit(), MAX_SHAPING_ATTEMPT_TRACE_RECORDS);

    let mut one = BoundedShapingAttemptLedger::default();
    one.record(&attempt.trace);
    let ledger_json = serde_json::to_string(&one).expect("ledger JSON");
    assert!(!ledger_json.contains("office"));
    assert!(!ledger_json.contains("glyphs"));
    assert!(ledger_json.contains("\"status\":\"complete\""));
}

fn table_cell_directions(doc: &rhwp::model::document::Document) -> Vec<u8> {
    use rhwp::model::control::Control;

    doc.sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Table(table) => Some(table.cells.iter().map(|cell| cell.text_direction)),
            _ => None,
        })
        .flatten()
        .collect()
}

fn rectangle_textbox_directions(doc: &rhwp::model::document::Document) -> Vec<(u8, bool)> {
    use rhwp::model::control::Control;
    use rhwp::model::shape::ShapeObject;

    doc.sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|para| &para.controls)
        .filter_map(|control| match control {
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rect) => rect
                    .drawing
                    .text_box
                    .as_ref()
                    .map(|text_box| ((text_box.list_attr & 0x07) as u8, text_box.vertical_all)),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

#[test]
fn issue_4969_q4_b_surface_values_map_without_collapsing_latin_orientation() {
    let horizontal = VerticalIntentDisposition::Supported(TypedVerticalIntent::horizontal());
    let sideways = VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
        VerticalLatinOrientation::Sideways,
    ));
    let upright = VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
        VerticalLatinOrientation::Upright,
    ));

    for surface in [
        VerticalIntentSurface::Hwp5TableCell,
        VerticalIntentSurface::Hwp5TextBox,
    ] {
        assert_eq!(adapt_hwp5_vertical_intent(surface, 0), horizontal);
        assert_eq!(adapt_hwp5_vertical_intent(surface, 1), sideways);
        assert_eq!(adapt_hwp5_vertical_intent(surface, 2), upright);
        for raw in 3..=7 {
            assert_eq!(
                adapt_hwp5_vertical_intent(surface, raw),
                VerticalIntentDisposition::UnsupportedRaw
            );
        }
    }

    for surface in [
        VerticalIntentSurface::HwpxTableCell,
        VerticalIntentSurface::HwpxTextBox,
        VerticalIntentSurface::HwpxSection,
        VerticalIntentSurface::HwpxMasterPage,
    ] {
        assert_eq!(
            adapt_hwpx_vertical_intent(surface, "HORIZONTAL"),
            horizontal
        );
        assert_eq!(adapt_hwpx_vertical_intent(surface, "VERTICAL"), sideways);
        assert_eq!(adapt_hwpx_vertical_intent(surface, "VERTICALALL"), upright);
        assert_eq!(
            adapt_hwpx_vertical_intent(surface, "SIDEWAYS-RL"),
            VerticalIntentDisposition::UnsupportedRaw
        );
    }

    assert_eq!(
        adapt_hwp5_vertical_intent(VerticalIntentSurface::HwpxTableCell, 1),
        VerticalIntentDisposition::UnsupportedRaw
    );
    assert_eq!(
        adapt_hwpx_vertical_intent(VerticalIntentSurface::Hwp5TableCell, "VERTICAL"),
        VerticalIntentDisposition::UnsupportedRaw
    );
}

#[test]
fn issue_4969_q4_b_public_pairs_adjudicate_verticalall_as_hwp5_code_2() {
    let hwpx = std::fs::read("samples/hwpx/tbox-v-flow-01.hwpx").expect("HWPX fixture");
    let hwp =
        std::fs::read("samples/hwpx/hancom-hwp/tbox-v-flow-01.hwp").expect("paired HWP fixture");
    let hwpx_doc = rhwp::parser::parse_document(&hwpx).expect("HWPX parse");
    let hwp_doc = rhwp::parser::parse_document(&hwp).expect("paired HWP parse");

    assert_eq!(rectangle_textbox_directions(&hwpx_doc), vec![(1, true)]);
    assert_eq!(rectangle_textbox_directions(&hwp_doc), vec![(2, false)]);
    let hwpx_roundtrip = rhwp::serializer::serialize_hwpx(&hwpx_doc).expect("HWPX serialize");
    let hwpx_roundtrip_doc =
        rhwp::parser::parse_document(&hwpx_roundtrip).expect("HWPX roundtrip parse");
    assert_eq!(
        rectangle_textbox_directions(&hwpx_roundtrip_doc),
        vec![(1, true)]
    );
    let hwp_roundtrip = rhwp::serializer::serialize_hwp(&hwp_doc).expect("HWP5 serialize");
    let hwp_roundtrip_doc =
        rhwp::parser::parse_document(&hwp_roundtrip).expect("HWP5 roundtrip parse");
    assert_eq!(
        rectangle_textbox_directions(&hwp_roundtrip_doc),
        vec![(2, false)]
    );
    assert_eq!(
        adapt_hwpx_vertical_intent(VerticalIntentSurface::HwpxTextBox, "VERTICALALL"),
        adapt_hwp5_vertical_intent(VerticalIntentSurface::Hwp5TextBox, 2)
    );
}

#[test]
fn issue_4969_q4_b_public_table_controls_preserve_direction_values_on_roundtrip() {
    let hwp = std::fs::read("samples/table-004.hwp").expect("HWP5 table fixture");
    let hwp_doc = rhwp::parser::parse_document(&hwp).expect("HWP5 parse");
    let hwp_direction_2 = table_cell_directions(&hwp_doc)
        .into_iter()
        .filter(|direction| *direction == 2)
        .count();
    assert_eq!(hwp_direction_2, 3);
    let hwp_roundtrip = rhwp::serializer::serialize_hwp(&hwp_doc).expect("HWP5 serialize");
    let hwp_roundtrip_doc =
        rhwp::parser::parse_document(&hwp_roundtrip).expect("HWP5 roundtrip parse");
    assert_eq!(
        table_cell_directions(&hwp_roundtrip_doc)
            .into_iter()
            .filter(|direction| *direction == 2)
            .count(),
        3
    );

    let hwpx =
        std::fs::read("samples/issue6029/3200477_icao_procedure.hwpx").expect("HWPX table fixture");
    let hwpx_doc = rhwp::parser::parse_document(&hwpx).expect("HWPX parse");
    let hwpx_direction_1 = table_cell_directions(&hwpx_doc)
        .into_iter()
        .filter(|direction| *direction == 1)
        .count();
    assert_eq!(hwpx_direction_1, 3);
    let hwpx_roundtrip = rhwp::serializer::serialize_hwpx(&hwpx_doc).expect("HWPX serialize");
    let hwpx_roundtrip_doc =
        rhwp::parser::parse_document(&hwpx_roundtrip).expect("HWPX roundtrip parse");
    assert_eq!(
        table_cell_directions(&hwpx_roundtrip_doc)
            .into_iter()
            .filter(|direction| *direction == 1)
            .count(),
        3
    );
}

fn vertical_fallback() -> VerticalLegacyGeometry {
    VerticalLegacyGeometry {
        bbox: VerticalRect {
            x: 91.0,
            y: 199.0,
            width: 18.0,
            height: 22.0,
        },
        next_inline_origin: VerticalPoint { x: 100.0, y: 222.0 },
        next_column_origin: VerticalPoint { x: 88.0, y: 200.0 },
    }
}

fn dormant_vertical_request<'a>(
    bytes: &'a [u8],
    text: &'a str,
    script: &'a str,
    language: &'a str,
    features: &'a [ShapingFeature],
    intent: TypedVerticalIntent,
) -> DormantVerticalShapingRequest<'a> {
    DormantVerticalShapingRequest {
        attempt_id: 4969,
        shaping: ShapingRequest {
            source: Some(ShapingExactSource {
                bytes,
                face_index: 0,
                portable: true,
            }),
            text,
            direction: ShapingDirection::TopToBottom,
            writing_mode: intent.writing_mode(),
            script: Some(script),
            language: Some(language),
            features,
            variations: &[],
        },
        intent,
        font_size_px: 10.0,
        origin: VerticalPoint { x: 100.0, y: 200.0 },
        column_pitch_px: 12.0,
        fallback_geometry: vertical_fallback(),
    }
}

fn assert_vertical_point(actual: VerticalPoint, expected: VerticalPoint) {
    assert!((actual.x - expected.x).abs() <= 1.0e-9, "x: {actual:?}");
    assert!((actual.y - expected.y).abs() <= 1.0e-9, "y: {actual:?}");
}

fn assert_vertical_rect(actual: VerticalRect, expected: VerticalRect) {
    assert!((actual.x - expected.x).abs() <= 1.0e-9, "x: {actual:?}");
    assert!((actual.y - expected.y).abs() <= 1.0e-9, "y: {actual:?}");
    assert!(
        (actual.width - expected.width).abs() <= 1.0e-9,
        "width: {actual:?}"
    );
    assert!(
        (actual.height - expected.height).abs() <= 1.0e-9,
        "height: {actual:?}"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_c_old_hangul_geometry_has_one_owner_and_exact_origins() {
    let transaction = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        SOURCE_HAN,
        "ᄒᆞᆫ글",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    ))
    .expect("old Hangul vertical transaction");
    assert!(!transaction.product_published());
    assert_eq!(transaction.fallback_geometry(), vertical_fallback());
    assert_eq!(transaction.trace().attempt_id, 4969);
    assert_eq!(
        transaction.trace().font_source_sha256.as_deref(),
        Some("2f86ef9a52acb6d1dad9d915843239123b635d97edd88fd0573a88ffcb4e16f1")
    );
    assert!(std::sync::Arc::ptr_eq(
        transaction.line_geometry(),
        transaction.bbox_geometry()
    ));
    assert!(std::sync::Arc::ptr_eq(
        transaction.line_geometry(),
        transaction.next_origin_geometry()
    ));

    let geometry = transaction.line_geometry();
    assert_eq!(geometry.run_class, VerticalRunClass::CjkUpright);
    assert_eq!(geometry.writing_mode, ShapingWritingMode::VerticalRl);
    assert_eq!(
        transaction
            .applied()
            .glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![614, 1230, 1497, 2085]
    );
    assert_eq!(
        geometry
            .glyphs
            .iter()
            .map(|glyph| glyph.cluster_utf8_range.clone())
            .collect::<Vec<_>>(),
        vec![0..9, 9..12, 9..12, 9..12]
    );
    for (glyph, expected) in geometry.glyphs.iter().zip([
        (
            VerticalPoint { x: 95.17, y: 208.8 },
            VerticalRect {
                x: 96.43,
                y: 200.41,
                width: 6.98,
                height: 9.18,
            },
        ),
        (
            VerticalPoint { x: 95.17, y: 218.8 },
            VerticalRect {
                x: 96.83,
                y: 211.19,
                width: 6.24,
                height: 3.85,
            },
        ),
        (
            VerticalPoint {
                x: 104.83,
                y: 218.8,
            },
            VerticalRect {
                x: 95.42,
                y: 214.74,
                width: 9.2,
                height: 0.9,
            },
        ),
        (
            VerticalPoint {
                x: 104.83,
                y: 218.8,
            },
            VerticalRect {
                x: 97.12,
                y: 216.59,
                width: 5.98,
                height: 3.05,
            },
        ),
    ]) {
        assert_vertical_point(glyph.origin, expected.0);
        assert_vertical_rect(glyph.bbox, expected.1);
    }
    assert_vertical_rect(
        geometry.bbox,
        VerticalRect {
            x: 95.42,
            y: 200.41,
            width: 9.2,
            height: 19.23,
        },
    );
    assert_eq!(geometry.inline_advance_px, 20.0);
    assert_eq!(
        geometry.next_inline_origin,
        VerticalPoint { x: 100.0, y: 220.0 }
    );
    assert_eq!(
        geometry.next_column_origin,
        VerticalPoint { x: 88.0, y: 200.0 }
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_c_latin_orientation_changes_transform_not_shaping_output() {
    let sideways = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        "AB",
        "Latn",
        "en",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Sideways),
    ))
    .expect("sideways Latin transaction");
    let upright = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        "AB",
        "Latn",
        "en",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    ))
    .expect("upright Latin transaction");
    assert_eq!(sideways.applied().as_ref(), upright.applied().as_ref());
    assert_eq!(
        sideways.line_geometry().run_class,
        VerticalRunClass::LatinSideways
    );
    assert_eq!(
        upright.line_geometry().run_class,
        VerticalRunClass::LatinUpright
    );
    assert!(sideways
        .line_geometry()
        .glyphs
        .iter()
        .all(|glyph| glyph.transform == VerticalGlyphTransform::RotateClockwise90));
    assert!(upright
        .line_geometry()
        .glyphs
        .iter()
        .all(|glyph| glyph.transform == VerticalGlyphTransform::Upright));
    assert_ne!(sideways.line_geometry().bbox, upright.line_geometry().bbox);
    assert_vertical_rect(
        sideways.line_geometry().bbox,
        VerticalRect {
            x: 96.72,
            y: 208.84,
            width: 7.57,
            height: 16.08,
        },
    );
    assert_vertical_rect(
        upright.line_geometry().bbox,
        VerticalRect {
            x: 97.0,
            y: 201.47,
            width: 6.0,
            height: 17.33,
        },
    );
    assert_eq!(
        sideways.line_geometry().next_inline_origin,
        upright.line_geometry().next_inline_origin
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_c_punctuation_and_column_progression_are_explicit() {
    let rl = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        "—…",
        "Hani",
        "ja",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    ))
    .expect("RL punctuation transaction");
    let lr = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        "—…",
        "Hani",
        "ja",
        &[],
        TypedVerticalIntent::vertical_lr(VerticalLatinOrientation::Upright),
    ))
    .expect("LR punctuation transaction");
    assert_eq!(
        rl.line_geometry().run_class,
        VerticalRunClass::CjkPunctuation
    );
    assert_eq!(
        lr.line_geometry().run_class,
        VerticalRunClass::CjkPunctuation
    );
    assert_eq!(
        rl.applied()
            .glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![197, 11826]
    );
    assert_eq!(
        lr.applied()
            .glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![197, 11826]
    );
    assert_vertical_rect(
        rl.line_geometry().bbox,
        VerticalRect {
            x: 95.99,
            y: 205.68,
            width: 8.01,
            height: 13.31,
        },
    );
    assert_eq!(
        rl.line_geometry().next_column_origin,
        VerticalPoint { x: 88.0, y: 200.0 }
    );
    assert_eq!(
        lr.line_geometry().next_column_origin,
        VerticalPoint { x: 112.0, y: 200.0 }
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_c_mixed_and_missing_vertical_source_return_pristine_fallback() {
    let mixed = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        "한A",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Sideways),
    ))
    .expect_err("mixed run must remain dormant unsupported");
    assert_eq!(
        mixed.reason(),
        DormantVerticalShapingRejectReason::MixedRunUnsupported
    );
    assert_eq!(mixed.fallback_geometry(), vertical_fallback());
    assert!(!mixed.product_published());

    let missing_metrics = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        HAPPINESS,
        "가변",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    ))
    .expect_err("font without vhea/vmtx must fail closed");
    assert_eq!(
        missing_metrics.reason(),
        DormantVerticalShapingRejectReason::ShapingRejected(
            ShapingRejectReason::VerticalMetricsUnavailable
        )
    );
    assert_eq!(missing_metrics.fallback_geometry(), vertical_fallback());
    assert!(!missing_metrics.product_published());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q4_c_intent_and_geometry_inputs_fail_before_publication() {
    let mut horizontal = dormant_vertical_request(
        NOTO,
        "한글",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::horizontal(),
    );
    horizontal.shaping.direction = ShapingDirection::LeftToRight;
    let horizontal = prepare_dormant_vertical_shaping_transaction(horizontal)
        .expect_err("horizontal intent is outside Q4-C");
    assert_eq!(
        horizontal.reason(),
        DormantVerticalShapingRejectReason::HorizontalIntentUnsupported
    );

    let mut mismatch = dormant_vertical_request(
        NOTO,
        "한글",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    );
    mismatch.shaping.writing_mode = ShapingWritingMode::VerticalLr;
    let mismatch = prepare_dormant_vertical_shaping_transaction(mismatch)
        .expect_err("source intent and shaping mode must agree");
    assert_eq!(
        mismatch.reason(),
        DormantVerticalShapingRejectReason::DirectionIntentMismatch
    );

    let mut invalid_pitch = dormant_vertical_request(
        NOTO,
        "한글",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    );
    invalid_pitch.column_pitch_px = f64::NAN;
    let invalid_pitch = prepare_dormant_vertical_shaping_transaction(invalid_pitch)
        .expect_err("non-finite column pitch must fail closed");
    assert_eq!(
        invalid_pitch.reason(),
        DormantVerticalShapingRejectReason::ColumnPitchInvalid
    );

    let axes = [ShapingVariation {
        tag: "wght".into(),
        value: 400.0,
    }];
    let mut variation = dormant_vertical_request(
        HAPPINESS,
        "가변",
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    );
    variation.shaping.variations = &axes;
    let variation = prepare_dormant_vertical_shaping_transaction(variation)
        .expect_err("variable outline bbox is outside Q4-C");
    assert_eq!(
        variation.reason(),
        DormantVerticalShapingRejectReason::VariationGeometryUnsupported
    );
}

#[derive(Debug, Clone, Copy)]
struct Q4D0ActivationSurface<'a> {
    hwp5_semantic_provenance: bool,
    text_direction: u8,
    cell_count: usize,
    paragraph_count: usize,
    composed_line_count: usize,
    text_run_count: usize,
    column_count: usize,
    text: &'a str,
    exact_source_registered: bool,
    variation_requested: bool,
}

fn q4_d0_activation_surface_is_candidate(surface: Q4D0ActivationSurface<'_>) -> bool {
    surface.hwp5_semantic_provenance
        && surface.text_direction == 2
        && surface.cell_count == 1
        && surface.paragraph_count == 1
        && surface.composed_line_count == 1
        && surface.text_run_count == 1
        && surface.column_count == 1
        && !surface.text.is_empty()
        && surface.text.chars().all(|ch| {
            matches!(
                u32::from(ch),
                0x1100..=0x11ff
                    | 0x3130..=0x318f
                    | 0x3400..=0x4dbf
                    | 0x4e00..=0x9fff
                    | 0xac00..=0xd7af
                    | 0xf900..=0xfaff
            )
        })
        && surface.exact_source_registered
        && !surface.variation_requested
}

#[test]
fn issue_4969_q4_d0_target_policy_is_hwp5_code2_one_column_pure_cjk_only() {
    let target = Q4D0ActivationSurface {
        hwp5_semantic_provenance: true,
        text_direction: 2,
        cell_count: 1,
        paragraph_count: 1,
        composed_line_count: 1,
        text_run_count: 1,
        column_count: 1,
        text: "한글",
        exact_source_registered: true,
        variation_requested: false,
    };
    assert!(q4_d0_activation_surface_is_candidate(target));
    assert_eq!(
        adapt_hwp5_vertical_intent(VerticalIntentSurface::Hwp5TableCell, 2),
        VerticalIntentDisposition::Supported(TypedVerticalIntent::vertical_rl(
            VerticalLatinOrientation::Upright
        ))
    );
    let transaction = prepare_dormant_vertical_shaping_transaction(dormant_vertical_request(
        NOTO,
        target.text,
        "Hang",
        "ko",
        &[],
        TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
    ))
    .expect("D0 target must already qualify in the Q4-C dormant owner");
    assert!(!transaction.product_published());

    assert!(!q4_d0_activation_surface_is_candidate(
        Q4D0ActivationSurface {
            hwp5_semantic_provenance: false,
            ..target
        }
    ));
    assert!(!q4_d0_activation_surface_is_candidate(
        Q4D0ActivationSurface {
            text_direction: 1,
            ..target
        }
    ));
    assert!(!q4_d0_activation_surface_is_candidate(
        Q4D0ActivationSurface {
            text: "한A",
            ..target
        }
    ));
    assert!(!q4_d0_activation_surface_is_candidate(
        Q4D0ActivationSurface {
            paragraph_count: 2,
            ..target
        }
    ));
    assert!(!q4_d0_activation_surface_is_candidate(
        Q4D0ActivationSurface {
            exact_source_registered: false,
            ..target
        }
    ));
}

fn q4_d1_context_request(slot: ExactFontSlot, text: &str) -> VerticalShapingContextRequest<'_> {
    VerticalShapingContextRequest {
        attempt_id: 4969,
        slot,
        text,
        intent: TypedVerticalIntent::vertical_rl(VerticalLatinOrientation::Upright),
        font_size_px: 10.0,
        origin: VerticalPoint { x: 100.0, y: 200.0 },
        column_pitch_px: 12.0,
        fallback_geometry: vertical_fallback(),
        script: Some("Hang"),
        language: Some("ko"),
        features: &[],
        variations: &[],
    }
}

#[test]
fn issue_4969_q4_d1_exact_source_context_certifies_dormant_owner_without_publication() {
    let slot = ExactFontSlot::new(7, 0);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: NOTO,
                face_index: 0,
            },
        )
        .expect("register exact Noto slot");
    let context = VerticalShapingContext::new(registry.clone());
    let first = context
        .prepare_dormant(q4_d1_context_request(slot, "한글"))
        .expect("certified D1 dormant transaction");
    let second = context
        .prepare_dormant(q4_d1_context_request(slot, "세로"))
        .expect("same exact source must reuse the registry Arc");

    assert_eq!(context.registry_generation(), registry.generation());
    assert_eq!(context.slot_count(), 1);
    assert_eq!(context.source_count(), 1);
    assert_eq!(first.certificate().slot(), slot);
    assert_eq!(
        first.certificate().registry_generation(),
        registry.generation()
    );
    assert_eq!(first.certificate().font_bytes(), NOTO.len());
    assert_eq!(first.certificate().face_index(), 0);
    assert_eq!(first.certificate().units_per_em(), 1_000);
    assert_eq!(
        first.certificate().font_source_sha256(),
        "6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74"
    );
    assert!(std::sync::Arc::ptr_eq(
        first.certificate().source_bytes_arc(),
        second.certificate().source_bytes_arc()
    ));
    assert_eq!(first.transaction().line_geometry().glyphs.len(), 2);
    assert!(!first.product_published());
    assert!(!second.product_published());

    let diagnostic = format!("{context:?} {:?}", first.certificate());
    for forbidden in ["한글", "세로", "NotoSans", "fontBytes", "/home/"] {
        assert!(
            !diagnostic.contains(forbidden),
            "D1 diagnostic leaked {forbidden}"
        );
    }

    registry
        .register(
            ExactFontSlot::new(8, 0),
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("mutate the original registry after the D1 snapshot");
    let stale_slot = context
        .prepare_dormant(q4_d1_context_request(ExactFontSlot::new(8, 0), "ᄒᆞᆫ글"))
        .expect_err("immutable D1 snapshot must not observe a later slot");
    assert_eq!(
        stale_slot.reason(),
        VerticalShapingContextRejectReason::SourceUnavailable
    );
    assert_eq!(stale_slot.fallback_geometry(), vertical_fallback());
    assert!(!stale_slot.product_published());
}

#[test]
fn issue_4969_q4_d1_context_preserves_typed_rejections_and_pristine_fallback() {
    let malformed_slot = ExactFontSlot::new(9, 0);
    let noto_slot = ExactFontSlot::new(10, 0);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            malformed_slot,
            ExactFontSource {
                bytes: b"not-a-font",
                face_index: 0,
            },
        )
        .expect("registry identity can retain malformed bounded bytes");
    registry
        .register(
            noto_slot,
            ExactFontSource {
                bytes: NOTO,
                face_index: 0,
            },
        )
        .expect("register exact Noto control");
    let context = VerticalShapingContext::new(registry);

    let malformed = context
        .prepare_dormant(q4_d1_context_request(malformed_slot, "한글"))
        .expect_err("malformed exact bytes must fail closed");
    assert_eq!(
        malformed.reason(),
        VerticalShapingContextRejectReason::Dormant(
            DormantVerticalShapingRejectReason::ShapingRejected(ShapingRejectReason::MalformedSfnt)
        )
    );
    assert_eq!(malformed.fallback_geometry(), vertical_fallback());
    assert!(!malformed.product_published());

    let mixed = context
        .prepare_dormant(q4_d1_context_request(noto_slot, "한A"))
        .expect_err("mixed run must preserve the Q4-C typed reason");
    assert_eq!(
        mixed.reason(),
        VerticalShapingContextRejectReason::Dormant(
            DormantVerticalShapingRejectReason::MixedRunUnsupported
        )
    );
    assert_eq!(mixed.fallback_geometry(), vertical_fallback());

    let axes = [ShapingVariation {
        tag: "wght".into(),
        value: 400.0,
    }];
    let mut variation = q4_d1_context_request(noto_slot, "한글");
    variation.variations = &axes;
    let variation = context
        .prepare_dormant(variation)
        .expect_err("variable bbox is outside D1");
    assert_eq!(
        variation.reason(),
        VerticalShapingContextRejectReason::Dormant(
            DormantVerticalShapingRejectReason::VariationGeometryUnsupported
        )
    );
    assert_eq!(variation.fallback_geometry(), vertical_fallback());

    let mut malformed_fallback = q4_d1_context_request(ExactFontSlot::new(99, 0), "한글");
    malformed_fallback.fallback_geometry.bbox.width = f64::NAN;
    let malformed_fallback = context
        .prepare_dormant(malformed_fallback)
        .expect_err("legacy geometry is validated before source lookup");
    assert_eq!(
        malformed_fallback.reason(),
        VerticalShapingContextRejectReason::Dormant(
            DormantVerticalShapingRejectReason::LegacyGeometryMalformed
        )
    );
    assert!(!malformed_fallback.product_published());
}

#[test]
fn issue_4969_q4_d0_red_product_vertical_module_registration_is_absent() {
    let renderer_mod = include_str!("../../src/renderer/mod.rs");
    assert!(
        renderer_mod.contains("pub(crate) mod shaping_vertical;"),
        "Q4-D1 red: shaping_vertical must not be a product module before approval"
    );
}

#[test]
fn issue_4969_q4_d0_red_exact_source_bound_vertical_owner_is_absent() {
    let vertical = include_str!("../../src/renderer/shaping_vertical.rs");
    assert!(
        vertical.contains("pub(crate) struct VerticalShapingContext"),
        "Q4-D1 red: exact-source-bound vertical context is not implemented"
    );
}

#[test]
fn issue_4969_q4_d0_red_atomic_table_cell_layout_commit_is_absent() {
    let layout = include_str!("../../src/renderer/layout/table_cell_content.rs");
    assert!(
        layout.contains("commit_bounded_vertical_hwp5_table_cell"),
        "Q4-D2 red: atomic HWP5 table-cell layout commit is not implemented"
    );
}

#[test]
fn issue_4969_q4_d2_vertical_sidecar_is_atomic_and_keeps_one_geometry_owner() {
    let slot = ExactFontSlot::new(71, 0);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: NOTO,
                face_index: 0,
            },
        )
        .expect("register public Noto exact source");
    let context = VerticalShapingContext::new(registry);
    let certified = Arc::new(
        context
            .prepare_dormant(q4_d1_context_request(slot, "한글"))
            .expect("certify bounded vertical owner"),
    );
    let geometry = certified.transaction().line_geometry();
    assert!(Arc::ptr_eq(
        geometry,
        certified.transaction().bbox_geometry()
    ));
    assert!(Arc::ptr_eq(
        geometry,
        certified.transaction().next_origin_geometry()
    ));

    let mut sidecars = VerticalShapingPageSidecars::default();
    let sidecar = Arc::new(BoundedVerticalHwp5TableCellSidecar::new(
        41,
        Arc::clone(&certified),
        "한글",
    ));
    sidecars
        .attach_bounded_hwp5_table_cell_atomic(Arc::clone(&sidecar))
        .expect("first atomic attach");
    assert_eq!(sidecars.len(), 1);
    assert!(Arc::ptr_eq(
        sidecars.get(41).expect("attached owner"),
        &sidecar
    ));
    let generation = sidecars.registry_generation();

    let duplicate = sidecars
        .attach_bounded_hwp5_table_cell_atomic(sidecar)
        .expect_err("duplicate node must fail before mutation");
    assert_eq!(duplicate, VerticalShapingSidecarRejectReason::DuplicateNode);
    assert_eq!(sidecars.len(), 1);
    assert_eq!(sidecars.registry_generation(), generation);
}

#[test]
fn issue_4969_q4_d2_non_noto_exact_source_leaves_vertical_sidecar_pristine() {
    let slot = ExactFontSlot::new(72, 0);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register non-target exact source");
    let context = VerticalShapingContext::new(registry);
    let certified = Arc::new(
        context
            .prepare_dormant(q4_d1_context_request(slot, "ᄒᆞᆫ글"))
            .expect("source is shape-capable but outside D2 hash gate"),
    );
    let sidecar = Arc::new(BoundedVerticalHwp5TableCellSidecar::new(
        51,
        certified,
        "ᄒᆞᆫ글",
    ));
    let mut sidecars = VerticalShapingPageSidecars::default();
    assert_eq!(
        sidecars
            .attach_bounded_hwp5_table_cell_atomic(sidecar)
            .expect_err("D2 accepts only the approved public Noto bytes"),
        VerticalShapingSidecarRejectReason::SourceIdentityMismatch
    );
    assert_eq!(sidecars.len(), 0);
    assert_eq!(sidecars.registry_generation(), None);
}

#[test]
fn issue_4969_q4_d3_a_maps_one_certified_line_to_leaf_scoped_sources_atomically() {
    let slot = ExactFontSlot::new(73, 0);
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            slot,
            ExactFontSource {
                bytes: NOTO,
                face_index: 0,
            },
        )
        .expect("register public Noto exact source");
    let context = VerticalShapingContext::new(registry);
    let certified = Arc::new(
        context
            .prepare_dormant(q4_d1_context_request(slot, "한글"))
            .expect("certify bounded vertical owner"),
    );
    let geometry = certified.transaction().line_geometry();
    assert_eq!(geometry.glyphs.len(), 2);
    let sidecar = Arc::new(BoundedVerticalHwp5TableCellSidecar::new(
        81,
        Arc::clone(&certified),
        "한글",
    ));
    let leaves = [
        VerticalGlyphPublicationLeafInput {
            source_node_id: 82,
            text_source_id: 11,
            text: "한",
            is_vertical: true,
            bbox: geometry.glyphs[0].bbox,
        },
        VerticalGlyphPublicationLeafInput {
            source_node_id: 83,
            text_source_id: 12,
            text: "글",
            is_vertical: true,
            bbox: geometry.glyphs[1].bbox,
        },
    ];

    let shadow = prepare_bounded_vertical_glyph_publication_shadow(&sidecar, &leaves)
        .expect("D3-A must prepare a read-only all-leaf mapping");
    assert_eq!(shadow.line_node_id(), 81);
    assert_eq!(shadow.leaves().len(), 2);
    assert_eq!(shadow.font_source_sha256(), NOTO_SANS_KR_REGULAR_SHA256);
    assert_eq!(shadow.font_bytes(), NOTO.len());
    assert_eq!(shadow.leaves()[0].source_node_id(), 82);
    assert_eq!(shadow.leaves()[0].text_source_id(), 11);
    assert_eq!(shadow.leaves()[0].source_utf8_range(), 0..3);
    assert_eq!(shadow.leaves()[0].source_utf16_range(), 0..1);
    assert_eq!(shadow.leaves()[0].glyph_id(), geometry.glyphs[0].glyph_id);
    assert_eq!(shadow.leaves()[1].source_node_id(), 83);
    assert_eq!(shadow.leaves()[1].text_source_id(), 12);
    assert_eq!(shadow.leaves()[1].source_utf8_range(), 0..3);
    assert_eq!(shadow.leaves()[1].source_utf16_range(), 0..1);
    assert_eq!(shadow.leaves()[1].glyph_id(), geometry.glyphs[1].glyph_id);
    assert!(!shadow.product_published());

    let mut malformed = leaves;
    malformed[1].text = "가";
    assert_eq!(
        prepare_bounded_vertical_glyph_publication_shadow(&sidecar, &malformed)
            .expect_err("one mismatched leaf must reject the whole line"),
        VerticalGlyphPublicationShadowRejectReason::ClusterTextMismatch
    );
}

#[test]
fn issue_4969_q4_d0_red_vertical_glyph_run_publication_is_absent() {
    let builder = include_str!("../../src/paint/builder.rs");
    assert!(
        builder.contains("lower_vertical_shaping_page_sidecars"),
        "Q4-D3 red: vertical sidecar publication is not implemented"
    );
}

#[test]
fn issue_4969_q4_d0_red_canvaskit_vertical_feature_detection_is_absent() {
    let rust_policy = include_str!("../../src/renderer/layer_renderer.rs");
    let studio_replay = include_str!("../../rhwp-studio/src/view/canvaskit/glyph-run-fonts.ts");
    assert!(
        rust_policy.contains("boundedVerticalHwp5TableCellV1")
            && studio_replay.contains("boundedVerticalHwp5TableCellV1"),
        "Q4-D4 red: Rust and Studio CanvasKit feature detection are not implemented"
    );
}
