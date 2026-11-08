//! 기존 스타일 해소의 계약 검사. 함수·검사 내용을 유지한다.

use super::*;
use crate::model::document::DocInfo;
use crate::model::style::*;
use crate::renderer::DEFAULT_DPI;

fn make_doc_info_with_font() -> DocInfo {
    DocInfo {
        font_faces: vec![
            // 한글(lang=0) 폰트
            vec![
                Font {
                    name: "함초롬돋움".to_string(),
                    ..Default::default()
                },
                Font {
                    name: "함초롬바탕".to_string(),
                    ..Default::default()
                },
            ],
        ],
        char_shapes: vec![
            CharShape {
                font_ids: [0, 0, 0, 0, 0, 0, 0], // 함초롬돋움
                base_size: 2400,                 // 24pt = 2400 HWPUNIT (1pt = 100 HWPUNIT)
                bold: true,
                italic: false,
                text_color: 0x00000000, // 검정
                ratios: [100, 100, 100, 100, 100, 100, 100],
                spacings: [0, 0, 0, 0, 0, 0, 0],
                ..Default::default()
            },
            CharShape {
                font_ids: [1, 1, 1, 1, 1, 1, 1], // 함초롬바탕
                base_size: 1000,                 // 10pt
                bold: false,
                italic: true,
                text_color: 0x00FF0000, // 파란색 (BGR)
                ratios: [80, 80, 80, 80, 80, 80, 80],
                spacings: [-5, -5, -5, -5, -5, -5, -5],
                underline_type: UnderlineType::Bottom,
                underline_color: 0x00000000,
                ..Default::default()
            },
        ],
        para_shapes: vec![
            ParaShape {
                alignment: Alignment::Center,
                line_spacing: 160,
                line_spacing_type: LineSpacingType::Percent,
                margin_left: 0,
                margin_right: 0,
                indent: 0,
                spacing_before: 0,
                spacing_after: 400, // 400 HWPUNIT
                ..Default::default()
            },
            ParaShape {
                alignment: Alignment::Justify,
                line_spacing: 1200, // 1200 HWPUNIT (고정)
                line_spacing_type: LineSpacingType::Fixed,
                margin_left: 1000,
                margin_right: 500,
                indent: 800,
                spacing_before: 200,
                spacing_after: 200,
                ..Default::default()
            },
        ],
        border_fills: vec![BorderFill {
            borders: [
                BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                },
                BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                },
                BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                },
                BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                },
            ],
            fill: Fill {
                fill_type: FillType::Solid,
                solid: Some(SolidFill {
                    background_color: 0x00FFFFFF,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn test_resolve_char_style_font_name() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert_eq!(styles.char_styles.len(), 2);
    assert_eq!(styles.char_styles[0].font_family, "함초롬돋움");
    assert_eq!(styles.char_styles[1].font_family, "함초롬바탕");
}

#[test]
fn test_resolve_char_style_size() {
    let mut doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    // 2400 HWPUNIT * 96 / 7200 = 32.0 px
    let expected_24pt = 2400.0 * DEFAULT_DPI / 7200.0;
    assert!((styles.char_styles[0].font_size - expected_24pt).abs() < 0.01);

    // 1000 HWPUNIT * 96 / 7200 ≈ 13.33 px
    let expected_10pt = 1000.0 * DEFAULT_DPI / 7200.0;
    assert!((styles.char_styles[1].font_size - expected_10pt).abs() < 0.01);
    // 100%는 모든 언어 슬롯에서 원래 크기와 정확히 같아야 한다.
    // 미세한 증가도 저장 0 간격을 재계산하는 분기를 잘못 발동시킨다.
    for style in &styles.char_styles {
        for lang in 0..LANG_COUNT {
            assert_eq!(style.font_size_for_lang(lang), style.font_size);
        }
    }
    // 실제 상대 크기 변경은 계속 반영한다.
    doc_info.char_shapes[1].relative_sizes[1] = 80;
    doc_info.char_shapes[1].relative_sizes[2] = 125;
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);
    assert_eq!(
        styles.char_styles[1].font_size_for_lang(1),
        expected_10pt * 0.8
    );
    assert_eq!(
        styles.char_styles[1].font_size_for_lang(2),
        expected_10pt * 1.25
    );
}

#[test]
fn test_resolve_char_style_bold_italic() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert!(styles.char_styles[0].bold);
    assert!(!styles.char_styles[0].italic);
    assert!(!styles.char_styles[1].bold);
    assert!(styles.char_styles[1].italic);
}

#[test]
fn test_resolve_char_style_color() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert_eq!(styles.char_styles[0].text_color, 0x00000000);
    assert_eq!(styles.char_styles[1].text_color, 0x00FF0000);
}

#[test]
fn test_resolve_char_style_underline() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert_eq!(styles.char_styles[0].underline, UnderlineType::None);
    assert_eq!(styles.char_styles[1].underline, UnderlineType::Bottom);
}

#[test]
fn test_resolve_char_style_ratio() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert!((styles.char_styles[0].ratio - 1.0).abs() < 0.01);
    assert!((styles.char_styles[1].ratio - 0.8).abs() < 0.01);
}

#[test]
fn test_resolve_char_style_letter_spacing() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    // 첫 번째: spacing=0 → 0.0 px
    assert!((styles.char_styles[0].letter_spacing - 0.0).abs() < 0.01);

    // 두 번째: spacing=-5, font_size ≈ 13.33 → -5% * 13.33 ≈ -0.67
    let expected = styles.char_styles[1].font_size * -5.0 / 100.0;
    assert!((styles.char_styles[1].letter_spacing - expected).abs() < 0.01);
}

#[test]
fn test_resolve_para_style_alignment() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert_eq!(styles.para_styles.len(), 2);
    assert_eq!(styles.para_styles[0].alignment, Alignment::Center);
    assert_eq!(styles.para_styles[1].alignment, Alignment::Justify);
}

#[test]
fn test_resolve_para_style_line_spacing() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    // 퍼센트 타입: 그대로 160.0
    assert!((styles.para_styles[0].line_spacing - 160.0).abs() < 0.01);
    assert_eq!(
        styles.para_styles[0].line_spacing_type,
        LineSpacingType::Percent
    );

    // 고정 타입: 저장값 1200 HWPUNIT 은 유효값의 2배(여백·문단간격·탭과 동일
    // 규약, #2070/#2197) → resolve 시 /2 후 px 변환
    let expected = hwpunit_to_px(1200, DEFAULT_DPI) / 2.0;
    assert!((styles.para_styles[1].line_spacing - expected).abs() < 0.01);
    assert_eq!(
        styles.para_styles[1].line_spacing_type,
        LineSpacingType::Fixed
    );
}

#[test]
fn test_resolve_para_style_margins() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    // ParaShape의 여백은 2배 값으로 저장되므로 resolve 시 2로 나눈다
    let margin_left = hwpunit_to_px(1000, DEFAULT_DPI) / 2.0;
    let margin_right = hwpunit_to_px(500, DEFAULT_DPI) / 2.0;
    let indent = hwpunit_to_px(800, DEFAULT_DPI) / 2.0;

    assert!((styles.para_styles[1].margin_left - margin_left).abs() < 0.01);
    assert!((styles.para_styles[1].margin_right - margin_right).abs() < 0.01);
    assert!((styles.para_styles[1].indent - indent).abs() < 0.01);
}

#[test]
fn test_resolve_border_style() {
    let doc_info = make_doc_info_with_font();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert_eq!(styles.border_styles.len(), 1);
    assert_eq!(styles.border_styles[0].fill_color, Some(0x00FFFFFF));
    assert_eq!(
        styles.border_styles[0].borders[0].line_type,
        BorderLineType::Solid
    );
}

#[test]
fn test_resolve_empty_doc_info() {
    let doc_info = DocInfo::default();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    assert!(styles.char_styles.is_empty());
    assert!(styles.para_styles.is_empty());
    assert!(styles.border_styles.is_empty());
}

#[test]
fn test_lookup_font_missing() {
    let doc_info = DocInfo::default();
    let name = lookup_font_name(&doc_info, 0, 0);
    assert!(name.is_empty());
}

#[test]
fn test_lookup_font_preserves_non_embedded_document_substitute() {
    let doc_info = DocInfo {
        font_faces: vec![vec![Font {
            name: "정부상징 부처명_16040911".to_string(),
            alt_type: 1,
            subst_font: Some(SubstFont {
                face: "한컴바탕".to_string(),
                font_type: 1,
                ..Default::default()
            }),
            ..Default::default()
        }]],
        ..Default::default()
    };

    assert_eq!(
        lookup_font_name(&doc_info, 0, 0),
        "정부상징 부처명_16040911,한컴바탕"
    );

    let decision = lookup_font_name_decision(&doc_info, 0, 0);
    assert_eq!(decision.language_slot, 0);
    assert_eq!(decision.font_id, 0);
    assert_eq!(
        decision.requested_face.as_deref(),
        Some("정부상징 부처명_16040911")
    );
    assert_eq!(
        decision.normalized_face.as_deref(),
        decision.requested_face.as_deref()
    );
    assert_eq!(decision.subst_font.as_deref(), Some("한컴바탕"));
    assert_eq!(
        decision.css_family_chain,
        ["정부상징 부처명_16040911", "한컴바탕"]
    );
}

#[test]
fn test_resolve_border_no_fill() {
    let doc_info = DocInfo {
        border_fills: vec![BorderFill::default()],
        ..Default::default()
    };
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);
    assert_eq!(styles.border_styles[0].fill_color, None);
}

#[test]
fn test_resolve_border_image_fill_preserves_watermark_attrs() {
    let mut fill = Fill::default();
    fill.fill_type = FillType::Image;
    fill.image = Some(ImageFill {
        fill_mode: ImageFillMode::Center,
        brightness: -50,
        contrast: 70,
        effect: 1,
        bin_data_id: 3,
    });
    let doc_info = DocInfo {
        border_fills: vec![BorderFill {
            fill,
            ..Default::default()
        }],
        ..Default::default()
    };

    let styles = resolve_styles(&doc_info, DEFAULT_DPI);
    let image_fill = styles.border_styles[0]
        .image_fill
        .as_ref()
        .expect("image fill");

    assert_eq!(image_fill.bin_data_id, 3);
    assert_eq!(image_fill.fill_mode, ImageFillMode::Center);
    assert_eq!(image_fill.brightness, -50);
    assert_eq!(image_fill.contrast, 70);
    assert_eq!(image_fill.effect, ImageEffect::GrayScale);
}

// === 언어 판별 테스트 ===

#[test]
fn test_detect_lang_category_korean() {
    assert_eq!(detect_lang_category('가'), 0);
    assert_eq!(detect_lang_category('힣'), 0);
    assert_eq!(detect_lang_category('ㄱ'), 0); // Compatibility Jamo
    assert_eq!(detect_lang_category('ㅎ'), 0);
}

#[test]
fn test_detect_lang_category_english() {
    assert_eq!(detect_lang_category('A'), 1);
    assert_eq!(detect_lang_category('z'), 1);
    assert_eq!(detect_lang_category('0'), 1);
    assert_eq!(detect_lang_category('9'), 1);
    assert_eq!(detect_lang_category('é'), 1); // Latin Extended
}

#[test]
fn test_detect_lang_category_cjk() {
    assert_eq!(detect_lang_category('中'), 2);
    assert_eq!(detect_lang_category('漢'), 2);
}

#[test]
fn test_detect_lang_category_japanese() {
    assert_eq!(detect_lang_category('あ'), 3); // Hiragana
    assert_eq!(detect_lang_category('ア'), 3); // Katakana
}

#[test]
fn test_detect_lang_category_symbol() {
    assert_eq!(detect_lang_category('→'), 5); // 화살표
    assert_eq!(detect_lang_category('★'), 5); // 도형
    assert_eq!(detect_lang_category('①'), 5); // 원숫자
}

#[test]
fn test_detect_lang_category_default() {
    // 공백, 구두점 등은 기본값(한국어=0)
    assert_eq!(detect_lang_category(' '), 0);
    assert_eq!(detect_lang_category('.'), 0);
    assert_eq!(detect_lang_category(','), 0);
}

// === 언어별 폰트 해소 테스트 ===

fn make_doc_info_with_multilang_fonts() -> DocInfo {
    DocInfo {
        font_faces: vec![
            // lang=0 (한국어)
            vec![Font {
                name: "함초롬돋움".to_string(),
                ..Default::default()
            }],
            // lang=1 (영어)
            vec![Font {
                name: "Arial".to_string(),
                ..Default::default()
            }],
            // lang=2 (한자)
            vec![Font {
                name: "SimSun".to_string(),
                ..Default::default()
            }],
            // lang=3~6 (나머지) - 비어있을 수 있음
        ],
        char_shapes: vec![CharShape {
            font_ids: [0, 0, 0, 0, 0, 0, 0], // 모든 언어에서 0번 폰트
            base_size: 1000,
            ratios: [100, 80, 90, 100, 100, 100, 100],
            spacings: [0, -5, 0, 0, 0, 0, 0],
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn test_resolve_char_style_font_families() {
    let doc_info = make_doc_info_with_multilang_fonts();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    let cs = &styles.char_styles[0];
    assert_eq!(cs.font_families.len(), 7);
    assert_eq!(cs.font_families[0], "함초롬돋움"); // 한국어
    assert_eq!(cs.font_families[1], "Arial"); // 영어
    assert_eq!(cs.font_families[2], "SimSun"); // 한자
    assert_eq!(cs.font_families[3], ""); // 일본어 (없음)
    assert_eq!(cs.font_family, "함초롬돋움"); // 기본값 = 한국어
}

#[test]
fn test_resolve_char_style_lang_ratios() {
    let doc_info = make_doc_info_with_multilang_fonts();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    let cs = &styles.char_styles[0];
    assert!((cs.ratios[0] - 1.0).abs() < 0.01); // 한국어 100%
    assert!((cs.ratios[1] - 0.8).abs() < 0.01); // 영어 80%
    assert!((cs.ratios[2] - 0.9).abs() < 0.01); // 한자 90%
    assert!((cs.ratio - 1.0).abs() < 0.01); // 기본값 = 한국어
}

#[test]
fn test_resolve_char_style_lang_spacings() {
    let doc_info = make_doc_info_with_multilang_fonts();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    let cs = &styles.char_styles[0];
    assert!((cs.letter_spacings[0] - 0.0).abs() < 0.01); // 한국어 spacing=0
    let expected_en = cs.font_size * -5.0 / 100.0;
    assert!((cs.letter_spacings[1] - expected_en).abs() < 0.01); // 영어 spacing=-5
}

#[test]
fn test_font_family_for_lang_fallback() {
    let doc_info = make_doc_info_with_multilang_fonts();
    let styles = resolve_styles(&doc_info, DEFAULT_DPI);

    let cs = &styles.char_styles[0];
    assert_eq!(cs.font_family_for_lang(0), "함초롬돋움");
    assert_eq!(cs.font_family_for_lang(1), "Arial");
    assert_eq!(cs.font_family_for_lang(3), "함초롬돋움"); // 빈 문자열 → 한국어 폴백
    assert_eq!(cs.font_family_for_lang(99), "함초롬돋움"); // 범위 초과 → 한국어 폴백
}
