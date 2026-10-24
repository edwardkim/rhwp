//! #7084 A: 측정 공급자의 합성 계약 검사다. 실물 한컴 출력 수용을 주장하지 않는다.
//! 전진 폭은 독립 공급자 입력값이며 이모지의 보편적인 폭을 가정하지 않는다.
use std::sync::Arc;

use rhwp::renderer::canvas_text_font::CanvasTextFont;
use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use rhwp::renderer::supplemental_metrics::{
    MetricBackend, MetricContext, MetricError, MetricEvidence, SupplementalMetric,
    SupplementalMetricStore, MAX_SUPPLEMENTAL_ENTRIES, MAX_SUPPLEMENTAL_KEY_BYTES,
};
use rhwp::renderer::TextStyle;

fn context() -> MetricContext {
    MetricContext {
        document_generation: 1,
        font_generation: 2,
        backend: MetricBackend::Canvas2d,
    }
}

fn style() -> TextStyle {
    TextStyle {
        font_family: "HCR Batang".into(),
        font_size: 20.0,
        ..Default::default()
    }
}

fn measured(style: &TextStyle, ch: &str, width: f64) -> SupplementalMetric {
    SupplementalMetric::backend_measured(style, ch, "20px test-provider".into(), width).unwrap()
}

fn bound(style: &mut TextStyle, entries: Vec<SupplementalMetric>) -> SupplementalMetricStore {
    let mut store = SupplementalMetricStore::new(context());
    store.replace(context(), entries).unwrap();
    store.bind_style(context(), style).unwrap();
    store
}

fn positions(text: &str, style: &TextStyle) -> Vec<f64> {
    EmbeddedTextMeasurer.compute_char_positions(text, style)
}

fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-8, "actual={a}, expected={b}");
}

#[test]
fn canvas_declared_size_and_horizontal_ratio_survive_script_setup() {
    let s = TextStyle {
        bold: true,
        italic: true,
        ratio: 0.81,
        superscript: true,
        ..style()
    };
    let font = CanvasTextFont::for_positioned_text(&s, 100.0);
    let family = rhwp::renderer::canvas_font_family_chain("HCR Batang");
    let script_size = s.font_size * 0.7;
    assert_eq!(
        font.descriptor(),
        format!("italic bold {script_size:.3}px {family}")
    );
    assert_eq!(
        font.old_hangul_descriptor(),
        format!("italic bold {script_size:.3}px 'RHWP Old Hangul', {family}")
    );
    near(font.draw_size(), script_size);
    near(font.horizontal_scale(), s.ratio);
    near(font.baseline() - 100.0, -s.font_size * 0.3);
    let sub = CanvasTextFont::for_positioned_text(
        &TextStyle {
            superscript: false,
            subscript: true,
            ..s
        },
        100.0,
    );
    near(sub.baseline() - 100.0, s.font_size * 0.15);
    assert_eq!(sub.descriptor(), font.descriptor());
}

#[test]
fn canvas_measured_advance_receives_declared_horizontal_ratio_once() {
    // Provider outputs are independent inputs. Print Tf/CTM keeps vertical size;
    // Tz applies the declared horizontal ratio, including narrow condensation.
    for ratio in [0.64, 0.81, 0.999, 1.0, 1.2, 0.0, -1.0] {
        let paint_scale = if ratio > 0.0 { ratio } else { 1.0 };
        for script in [false, true] {
            let mut s = TextStyle {
                ratio,
                superscript: script,
                ..style()
            };
            let font = CanvasTextFont::for_positioned_text(&s, 0.0);
            near(
                font.draw_size(),
                s.font_size * if script { 0.7 } else { 1.0 },
            );
            let entry = SupplementalMetric::from_canvas_measurement(
                &s,
                "😀",
                font.descriptor(),
                "browser-resolved font".into(),
                27.531,
            )
            .unwrap();
            let _store = bound(&mut s, vec![entry]);
            near(positions("😀", &s)[1], 27.531 * paint_scale);
            // CSS local()의 형식이 미확인이어도 실제 Canvas 공급자가 잰 폭은
            // 같은 backend에서 소비한다. 가족 이름으로 TrueType을 선언하지 않는다.
            for punctuation in ["‘", "’", "·"] {
                let mut local = TextStyle {
                    font_family: "HYGothic-Medium".into(),
                    supplemental_metrics: None,
                    ..s.clone()
                };
                assert!(!local.font_metric_trusted);
                let setup = CanvasTextFont::for_positioned_text(&local, 0.0);
                let measured = setup.draw_size(); // 독립 공급자 입력: 한글과 같은 1em.
                let entry = SupplementalMetric::from_canvas_measurement(
                    &local,
                    punctuation,
                    setup.descriptor(),
                    setup.descriptor().into(),
                    measured,
                )
                .unwrap();
                let _store = bound(&mut local, vec![entry]);
                near(positions(punctuation, &local)[1], measured * paint_scale);
            }
        }
    }
}

#[test]
fn canvas_rounded_css_size_keeps_provider_advance_in_its_measured_frame() {
    let mut s = TextStyle {
        font_size: 13.333333,
        ratio: 0.8,
        ..style()
    };
    let font = CanvasTextFont::for_positioned_text(&s, 0.0);
    assert!(font
        .descriptor()
        .starts_with(&format!("{:.3}px ", s.font_size)));
    // The provider may have nonlinear optical metrics at this CSS size. Keep
    // its measured result; do not infer it from nominal or unrounded font size.
    let entry = SupplementalMetric::from_canvas_measurement(
        &s,
        "😀",
        font.descriptor(),
        format!("{:.3}px resolved-face", s.font_size),
        16.731,
    )
    .unwrap();
    let _store = bound(&mut s, vec![entry]);
    near(positions("😀", &s)[1], 16.731 * s.ratio);
}

#[test]
fn canvas_descriptor_must_match_requested_style_before_registration() {
    let s = style();
    let font = CanvasTextFont::for_positioned_text(&s, 0.0);
    let changed = TextStyle {
        bold: true,
        ..s.clone()
    };
    assert_eq!(
        SupplementalMetric::from_canvas_measurement(
            &changed,
            "😀",
            font.descriptor(),
            "resolved-face".into(),
            27.5,
        ),
        Err(MetricError::InvalidDescriptor)
    );
    // Browser substitution/canonicalization may legitimately differ from input.
    let entry = SupplementalMetric::from_canvas_measurement(
        &s,
        "😀",
        font.descriptor(),
        "20px browser-fallback".into(),
        27.5,
    )
    .unwrap();
    assert_eq!(
        entry.evidence(),
        &MetricEvidence::BackendMeasured {
            descriptor: "20px browser-fallback".into()
        }
    );
}

#[test]
fn measured_descriptor_cannot_leak_to_a_different_document_ratio() {
    let mut s = TextStyle {
        ratio: 0.8,
        ..style()
    };
    let font = CanvasTextFont::for_positioned_text(&s, 0.0);
    let entry = SupplementalMetric::from_canvas_measurement(
        &s,
        "😀",
        font.descriptor(),
        "resolved-face".into(),
        25.0,
    )
    .unwrap();
    let _store = bound(&mut s, vec![entry]);
    let changed = TextStyle {
        ratio: 0.9,
        ..s.clone()
    };
    assert!(s
        .supplemental_metrics
        .as_ref()
        .unwrap()
        .lookup(context(), &changed, '😀')
        .is_none());
    let plain = TextStyle {
        supplemental_metrics: None,
        ..changed.clone()
    };
    assert_eq!(positions("😀", &changed), positions("😀", &plain));
}

#[test]
fn canvas_registration_rejects_invalid_units_without_changing_paint_defaults() {
    let s = style();
    let font = CanvasTextFont::for_positioned_text(&s, 0.0);
    for width in [f64::NAN, f64::INFINITY, -1.0] {
        assert_eq!(
            SupplementalMetric::from_canvas_measurement(
                &s,
                "😀",
                font.descriptor(),
                "resolved-face".into(),
                width,
            ),
            Err(MetricError::InvalidAdvance)
        );
    }
    for ratio in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let invalid = TextStyle { ratio, ..s.clone() };
        assert_eq!(
            SupplementalMetric::from_canvas_measurement(
                &invalid,
                "😀",
                font.descriptor(),
                "resolved-face".into(),
                27.5,
            ),
            Err(MetricError::InvalidStyle)
        );
    }
    let invalid = TextStyle {
        font_size: 0.0,
        ..s.clone()
    };
    let fallback = CanvasTextFont::for_positioned_text(&invalid, 0.0);
    assert!(fallback.descriptor().starts_with("12.000px "));
    assert_eq!(
        SupplementalMetric::from_canvas_measurement(
            &invalid,
            "😀",
            fallback.descriptor(),
            "resolved-face".into(),
            27.5,
        ),
        Err(MetricError::InvalidStyle)
    );
    let tiny = TextStyle {
        font_size: 0.00001,
        ..s
    };
    let tiny_font = CanvasTextFont::for_positioned_text(&tiny, 0.0);
    assert_eq!(
        SupplementalMetric::from_canvas_measurement(
            &tiny,
            "😀",
            tiny_font.descriptor(),
            "resolved-face".into(),
            27.5,
        ),
        Err(MetricError::InvalidAdvance)
    );
}

#[test]
fn missing_advance_is_shared_by_total_and_following_positions() {
    let original = style();
    // observed generic 0.5em path on the Hancom device grid (#7702): 187.5 → 188 units × 4/75
    let fallback = 188.0 * 4.0 / 75.0;
    near(positions("😀", &original)[1], fallback);
    let mut prepared = original.clone();
    let _store = bound(&mut prepared, vec![measured(&original, "😀", 27.5)]);
    let old = positions("😀A한", &original);
    let new = positions("😀A한", &prepared);
    near(new[1], 27.5);
    for i in 1..new.len() {
        near(new[i] - old[i], 27.5 - fallback);
    }
    // The public estimate intentionally retains its pre-existing integer rounding.
    let total = EmbeddedTextMeasurer.estimate_text_width("😀A한", &prepared);
    near(total, new.last().unwrap().round());
    let snapshot = prepared.supplemental_metrics.as_ref().unwrap();
    assert_eq!(
        snapshot
            .lookup(context(), &prepared, '😀')
            .unwrap()
            .width_source(),
        "supplementalBackendMeasured"
    );
}

#[test]
fn unknown_non_emoji_glyph_uses_provider_width_not_square_rule() {
    let mut prepared = style();
    let entry = measured(&prepared, "𝄞", 11.25);
    let _store = bound(&mut prepared, vec![entry]);
    near(positions("𝄞", &prepared)[1], 11.25);
    // 같은 검증 regular 프로그램의 faux bold도 cmap 소유가 바뀌지 않는다.
    // 검증한 TrueType 환경은 결측 기호 공급자의 자연 폭을 소비한다.
    // 형식 미확인의 한글 슬롯은 독립 sample16 Print의 전각 호환 폭을
    // 보존한다. 글리프 프로그램이 있다는 사실로 폭 계약을 승격하지 않는다.
    for (bold, font_metric_trusted, glyph_fallback_advance_explicit) in [
        (false, false, false),
        (false, true, false),
        (true, false, false),
        (true, true, false),
        (false, false, true),
        (true, false, true),
    ] {
        let declared = TextStyle {
            font_family: "휴먼명조,Haansoft Batang".into(),
            font_metric_trusted,
            glyph_fallback_advance_explicit,
            bold,
            ..style()
        };
        let p = positions("용․편", &declared);
        if font_metric_trusted || glyph_fallback_advance_explicit {
            assert!(p[2] - p[1] > 0.0 && p[2] - p[1] < (p[3] - p[2]) / 2.0);
        } else {
            near(p[2] - p[1], p[3] - p[2]);
        }
        for (hft_hangul_face, hft_fullwidth_dot, italic) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let protected = TextStyle {
                hft_hangul_face,
                hft_fullwidth_dot,
                italic,
                ..declared.clone()
            };
            let p = positions("용․편", &protected);
            assert!(
                p[2] - p[1] >= (p[3] - p[2]) / 2.0,
                "HFT 또는 확인하지 않은 style 프로그램의 glyph 부재를 추정하지 않음"
            );
        }
    }
}

#[test]
fn existing_metrics_and_explicit_hwp_rules_are_not_overridden() {
    let original = style();
    // ASCII DB hit, Hangul, HWP figure space, ordinary space, halfwidth form,
    // narrow punctuation, synthetic PUA number and inline object placeholder.
    let text = "A가\u{2007} ｢(\u{F02B0}\u{FFFC}";
    let entries = text
        .chars()
        .map(|ch| measured(&original, &ch.to_string(), 111.0))
        .collect();
    let mut prepared = original.clone();
    let _store = bound(&mut prepared, entries);
    assert_eq!(positions(text, &original), positions(text, &prepared));
    assert_eq!(positions("A\t가", &original), positions("A\t가", &prepared));
    // 확인된 실제 프로그램과 고정폭 원 글리프, 명시적 반각 규칙은
    // 임의 공급자 폭으로 바꾸지 않는다. 형식 미확인의 좁힘 추정과 구분한다.
    for (family, trusted, punctuation) in [
        ("HYGothic-Medium", true, "‘’·"),
        ("굴림체", false, "‘’"),
        ("HYGothic-Medium", false, "“”"),
    ] {
        let original = TextStyle {
            font_family: family.into(),
            font_metric_trusted: trusted,
            ..style()
        };
        let entries = punctuation
            .chars()
            .map(|ch| measured(&original, &ch.to_string(), original.font_size * 3.0))
            .collect();
        let mut prepared = original.clone();
        let _store = bound(&mut prepared, entries);
        assert_eq!(
            positions(punctuation, &original),
            positions(punctuation, &prepared)
        );
    }
}

#[test]
fn ratio_and_spacing_are_applied_once_after_natural_advance() {
    for (spacing, expected) in [(0.0, 24.0), (2.0, 26.4), (-2.0, 21.6), (-40.0, 12.0)] {
        let mut prepared = TextStyle {
            ratio: 0.8,
            letter_spacing: spacing,
            ..style()
        };
        let entry = measured(&prepared, "😀", 30.0);
        let _store = bound(&mut prepared, vec![entry]);
        near(positions("😀", &prepared)[1], expected);
    }
}

#[test]
fn dash_leader_keeps_explicit_advance_with_missing_family() {
    let original = TextStyle {
        font_family: "unknown-test-face".into(),
        ..style()
    };
    let mut prepared = original.clone();
    let _store = bound(&mut prepared, vec![measured(&original, "-", 1.0)]);
    assert_eq!(positions("---", &prepared), positions("---", &original));
}

#[test]
fn replacing_valid_batch_invalidates_old_styles_until_rebound() {
    let mut prepared = style();
    let original = prepared.clone();
    let entry = measured(&prepared, "😀", 27.5);
    let mut store = bound(&mut prepared, vec![entry]);
    store
        .replace(context(), vec![measured(&original, "😀", 31.0)])
        .unwrap();
    assert_eq!(positions("😀", &prepared), positions("😀", &original));
    store.bind_style(context(), &mut prepared).unwrap();
    near(positions("😀", &prepared)[1], 31.0);
}

#[test]
fn effective_size_and_face_style_are_part_of_lookup_identity() {
    let original = style();
    let mut prepared = original.clone();
    let _store = bound(&mut prepared, vec![measured(&original, "😀", 27.5)]);
    for variant in [
        TextStyle {
            bold: true,
            ..prepared.clone()
        },
        TextStyle {
            italic: true,
            ..prepared.clone()
        },
        TextStyle {
            font_size: 21.0,
            ..prepared.clone()
        },
        TextStyle {
            font_family: "other face".into(),
            ..prepared.clone()
        },
        TextStyle {
            superscript: true,
            ..prepared.clone()
        },
    ] {
        let mut legacy = variant.clone();
        legacy.supplemental_metrics = None;
        assert_eq!(positions("😀", &variant), positions("😀", &legacy));
    }
    for (superscript, subscript) in [(true, false), (false, true)] {
        let mut scripted = TextStyle {
            superscript,
            subscript,
            ..original.clone()
        };
        let entry = measured(&scripted, "😀", 19.25); // already measured at 14px
        let _store = bound(&mut scripted, vec![entry]);
        near(positions("😀", &scripted)[1], 19.25); // no second 0.7 multiplication
    }
}

#[test]
fn multi_scalar_graphemes_are_never_partially_overridden() {
    let original = style();
    let mut prepared = original.clone();
    let _store = bound(&mut prepared, vec![measured(&original, "😀", 27.5)]);
    for text in ["😀\u{FE0F}", "😀\u{200D}😀", "😀\u{301}"] {
        assert_eq!(
            positions(text, &original),
            positions(text, &prepared),
            "{text}"
        );
    }
    let mixed = positions("😀\u{FE0F}😀", &prepared);
    near(mixed[3] - mixed[2], 27.5);
}

#[test]
fn malformed_measurements_are_rejected_and_zero_is_valid() {
    for width in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert_eq!(
            SupplementalMetric::backend_measured(&style(), "😀", "20px x".into(), width),
            Err(MetricError::InvalidAdvance)
        );
    }
    for cluster in ["", "ab", "😀\u{FE0F}", "😀\u{200D}😀", "\n"] {
        assert_eq!(
            SupplementalMetric::backend_measured(&style(), cluster, "20px x".into(), 20.0),
            Err(MetricError::UnsupportedCluster)
        );
    }
    for descriptor in ["", "20px\nx"] {
        assert_eq!(
            SupplementalMetric::backend_measured(&style(), "😀", descriptor.into(), 20.0),
            Err(MetricError::InvalidDescriptor)
        );
    }
    for size in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let invalid = TextStyle {
            font_size: size,
            ..style()
        };
        assert_eq!(
            SupplementalMetric::backend_measured(&invalid, "😀", "20px x".into(), 20.0),
            Err(MetricError::InvalidStyle)
        );
    }
    let mut prepared = style();
    let entry = measured(&prepared, "😀", 0.0);
    let _store = bound(&mut prepared, vec![entry]);
    near(positions("😀", &prepared)[1], 0.0);
}

#[test]
fn rejected_or_identical_batches_leave_current_snapshot_intact() {
    let mut prepared = style();
    let entry = measured(&prepared, "😀", 27.5);
    let mut store = bound(&mut prepared, vec![entry.clone()]);
    let old = store.snapshot().unwrap();
    store.replace(context(), vec![entry.clone()]).unwrap();
    assert!(Arc::ptr_eq(&old, &store.snapshot().unwrap()));
    let wrong = MetricContext {
        document_generation: 9,
        ..context()
    };
    assert_eq!(
        store.replace(wrong, vec![entry.clone()]),
        Err(MetricError::ContextMismatch)
    );
    assert_eq!(
        store.replace(context(), vec![entry.clone(), entry.clone()]),
        Err(MetricError::DuplicateKey)
    );
    assert_eq!(
        store.replace(context(), vec![entry; MAX_SUPPLEMENTAL_ENTRIES + 1]),
        Err(MetricError::LimitExceeded)
    );
    let oversized = "x".repeat(MAX_SUPPLEMENTAL_KEY_BYTES);
    assert_eq!(
        SupplementalMetric::backend_measured(&prepared, "😀", oversized, 20.0),
        Err(MetricError::LimitExceeded)
    );
    assert!(Arc::ptr_eq(&old, &store.snapshot().unwrap()));
    near(positions("😀", &prepared)[1], 27.5);
}

#[test]
fn total_key_budget_is_atomic_not_just_per_entry() {
    let mut prepared = style();
    let entry = measured(&prepared, "😀", 27.5);
    let mut store = bound(&mut prepared, vec![entry]);
    let descriptor = "x".repeat(MAX_SUPPLEMENTAL_KEY_BYTES / 2);
    let entries = ['😀', '𝄞']
        .iter()
        .map(|ch| {
            SupplementalMetric::backend_measured(
                &prepared,
                &ch.to_string(),
                descriptor.clone(),
                20.0,
            )
            .unwrap()
        })
        .collect();
    assert_eq!(
        store.replace(context(), entries),
        Err(MetricError::LimitExceeded)
    );
    near(positions("😀", &prepared)[1], 27.5);
}

#[test]
fn session_changes_invalidate_retained_styles_and_reject_late_responses() {
    for next in [
        MetricContext {
            document_generation: 2,
            ..context()
        },
        MetricContext {
            font_generation: 3,
            ..context()
        },
        MetricContext {
            backend: MetricBackend::NativeSkia,
            ..context()
        },
    ] {
        let original = style();
        let mut prepared = original.clone();
        let entry = measured(&prepared, "😀", 27.5);
        let mut store = bound(&mut prepared, vec![entry.clone()]);
        let snapshot = store.snapshot().unwrap();
        assert!(snapshot.lookup(next, &prepared, '😀').is_none());
        store.reset_context(next);
        assert!(!snapshot.is_active());
        assert_eq!(positions("😀", &prepared), positions("😀", &original));
        assert_eq!(
            store.replace(context(), vec![entry]),
            Err(MetricError::ContextMismatch)
        );
    }
}

#[test]
fn canvas_measurements_cannot_be_registered_for_other_backends() {
    for backend in [
        MetricBackend::CanvasKit,
        MetricBackend::NativeSkia,
        MetricBackend::Svg,
    ] {
        let target = MetricContext {
            backend,
            ..context()
        };
        let mut store = SupplementalMetricStore::new(target);
        assert_eq!(
            store.replace(target, vec![measured(&style(), "😀", 27.5)]),
            Err(MetricError::ContextMismatch)
        );
        assert!(store.snapshot().is_none());
    }
}

#[test]
fn closing_session_invalidates_styles_without_serializing_resources() {
    let original = style();
    let mut prepared = original.clone();
    let store = bound(&mut prepared, vec![measured(&original, "😀", 27.5)]);
    assert_eq!(
        serde_json::to_value(&prepared).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    drop(store);
    assert_eq!(positions("😀", &prepared), positions("😀", &original));
}

#[test]
fn verified_source_uses_real_hmtx_and_preserves_source_identity() {
    let bytes: Arc<[u8]> = std::fs::read("ttfs/opensource/NotoSansKR-Regular.ttf")
        .unwrap()
        .into();
    let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
    let glyph = face.glyph_index('A').unwrap();
    let expected =
        f64::from(face.glyph_hor_advance(glyph).unwrap()) * 20.0 / f64::from(face.units_per_em());
    let entry = SupplementalMetric::from_font(&style(), "A", bytes.clone(), 0).unwrap();
    near(entry.natural_advance_px(), expected);
    assert_eq!(entry.width_source(), "supplementalVerifiedSource");
    match entry.evidence() {
        MetricEvidence::VerifiedSource {
            bytes: source,
            face_index,
            glyph_id,
            ..
        } => {
            assert!(Arc::ptr_eq(source, &bytes));
            assert_eq!(*face_index, 0);
            assert_eq!(*glyph_id, glyph.0);
        }
        _ => panic!("font bytes must not be mislabeled as browser measurement"),
    }
    assert_eq!(
        SupplementalMetric::from_font(&style(), "A", Arc::from(&b"invalid"[..]), 0),
        Err(MetricError::MalformedFont)
    );
    assert_eq!(
        SupplementalMetric::from_font(&style(), "\u{10FFFF}", bytes, 0),
        Err(MetricError::MissingGlyph)
    );
}

/// 독립 한컴 Print의 DotumChe 합성 굵게는 같은 정규 프로그램의 채움+획이다.
/// 획은 advance를 바꾸지 않으며, 실제 Bold face의 요청은 별도로 남아야 한다.
#[test]
fn faux_bold_uses_the_regular_canvas_measurement_without_changing_metric_ownership() {
    let regular = TextStyle {
        font_family: "DotumChe".into(),
        ..style()
    };
    let bold = TextStyle {
        bold: true,
        ..regular.clone()
    };
    let regular_font = CanvasTextFont::for_positioned_text(&regular, 0.0);
    let bold_font = CanvasTextFont::for_positioned_text(&bold, 0.0);
    assert_eq!(
        regular_font.descriptor(),
        bold_font.descriptor(),
        "PDF의 같은 정규 face를 측정하며 굵은 획은 paint가 담당한다"
    );
    let entry = SupplementalMetric::from_canvas_measurement(
        &bold,
        "😀",
        regular_font.descriptor(),
        regular_font.descriptor().into(),
        27.5,
    )
    .expect("실제 그리는 정규 프로그램의 advance 등록");
    let mut measured_bold = bold.clone();
    let _store = bound(&mut measured_bold, vec![entry]);
    near(positions("😀", &measured_bold)[1], 27.5);
    assert_ne!(
        positions("😀", &regular)[1],
        27.5,
        "동일 face를 사용해도 보충 메트릭의 원본 style 소유는 분리한다"
    );
    let real_bold = TextStyle {
        font_family: "Malgun Gothic".into(),
        bold: true,
        ..regular.clone()
    };
    let real_regular = TextStyle {
        bold: false,
        ..real_bold.clone()
    };
    assert_ne!(
        CanvasTextFont::for_positioned_text(&real_bold, 0.0).descriptor(),
        CanvasTextFont::for_positioned_text(&real_regular, 0.0).descriptor(),
        "실제 Bold 메트릭이 있는 face는 bold 요청을 유지한다"
    );
}
