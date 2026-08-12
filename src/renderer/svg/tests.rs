use super::*;

#[test]
fn test_svg_begin_end_page() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.end_page();
    let output = renderer.output();
    assert!(output.starts_with("<svg"));
    assert!(output.contains("width=\"800\""));
    assert!(output.ends_with("</svg>\n"));
}

#[test]
fn test_svg_draw_text() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text(
        "안녕하세요",
        10.0,
        20.0,
        &TextStyle {
            font_size: 16.0,
            bold: true,
            ..Default::default()
        },
    );
    let output = renderer.output();
    assert!(output.contains("<text"));
    assert!(output.contains("font-weight=\"bold\""));
}

#[test]
fn test_svg_draw_text_medium_weight() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text(
        "중고딕",
        10.0,
        20.0,
        &TextStyle {
            font_size: 16.0,
            font_family: "HY중고딕".to_string(),
            bold: false,
            ..Default::default()
        },
    );
    let output = renderer.output();
    assert!(
        output.contains("font-weight=\"500\""),
        "중고딕 계열은 font-weight 500이어야 함"
    );
    assert!(!output.contains("font-weight=\"bold\""));
}

#[test]
fn legacy_hanyang_faces_have_portable_local_aliases() {
    // HWPX가 저장한 legacy face와 한컴 2020 PDF가 실제로 대체해 출력한 family가
    // 다르다. `--font-style` SVG가 이 순서를 잃으면 검증 host에서 무관한 폴백 또는
    // 두부(□)로 rasterize되어 PDF 대조 증적이 무효해진다.
    assert_eq!(
        font_local_aliases("한양중고딕"),
        vec![
            "HCR Dotum",
            "함초롬돋움",
            "한양중고딕",
            "HY중고딕",
            "HYGothic-Medium",
        ]
    );
    assert_eq!(
        font_local_aliases("휴먼명조"),
        vec![
            "HCR Batang",
            "함초롬바탕",
            "Batang",
            "바탕",
            "AppleMyungjo",
            "Noto Serif CJK KR",
            "휴먼명조",
            "HumanMyeongJo",
        ],
        "Chrome에서 bitmap HMKMM을 먼저 고르면 한글이 두부가 되므로 outline 대체가 우선이어야 함"
    );
    assert_eq!(
        font_local_aliases("휴먼고딕"),
        vec![
            "Malgun Gothic",
            "맑은 고딕",
            "Apple SD Gothic Neo",
            "Noto Sans KR ExtraLight",
            "Noto Sans KR",
            "Pretendard",
            "휴먼고딕",
        ],
        "Chrome에서 HMKMG를 먼저 고르면 한글이 두부가 되므로 outline 대체가 우선이어야 함"
    );
    assert_eq!(
        font_local_aliases("한양신명조"),
        vec!["한양신명조", "HY신명조", "HYSinMyeongJo-Medium"],
        "정상 outline인 HY신명조는 원 face를 먼저 유지해야 함"
    );
    assert_eq!(
        known_font_filenames("한양중고딕").first(),
        Some(&"HANDotum.ttf"),
        "한컴 2020 PDF와 같은 HCR Dotum을 full embed에서 먼저 찾아야 함"
    );
    assert_eq!(
        known_font_filenames("휴먼명조").first(),
        Some(&"HANBatang.ttf"),
        "한컴 2020 PDF와 같은 HCR Batang을 휴먼명조보다 먼저 찾아야 함"
    );
    assert_eq!(
        known_font_filenames("한양신명조").first(),
        Some(&"H2MJSM.TTF"),
        "Windows 설치본의 실제 한양신명조 파일을 먼저 찾아야 함"
    );
    assert_eq!(
        font_local_bold_aliases("휴먼명조").first(),
        Some(&"HCR Batang Bold"),
        "휴먼명조의 Bold는 browser synthetic bold가 아니라 한컴 HCR Bold face여야 함"
    );
    assert_eq!(
        known_bold_font_filenames("한양중고딕").first(),
        Some(&"HANDotumB.ttf"),
        "한양중고딕의 Bold full embed는 한컴 HCR Dotum Bold 파일을 먼저 찾아야 함"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn style_font_face_css_orders_broken_bitmap_faces_after_outline_fallbacks() {
    let mut renderer = SvgRenderer::new();
    renderer.font_embed_mode = FontEmbedMode::Style;
    for family in ["휴먼명조", "휴먼고딕", "한양중고딕", "한양신명조"] {
        renderer
            .font_codepoints
            .entry(family.to_string())
            .or_default()
            .extend("한글".chars());
    }
    renderer.font_bold_families.insert("휴먼명조".to_string());
    renderer.font_bold_families.insert("한양중고딕".to_string());

    let css = generate_font_style(
        &renderer,
        &[],
        &std::collections::HashMap::<String, Vec<u8>>::new(),
    );
    let human_myeongjo = css
        .lines()
        .find(|line| line.contains("font-family: \"휴먼명조\""))
        .expect("휴먼명조 style rule");
    assert!(
        human_myeongjo
            .find("local(\"HCR Batang\")")
            .expect("HCR Batang local")
            < human_myeongjo
                .find("local(\"휴먼명조\")")
                .expect("휴먼명조 local"),
        "실제 CSS에서도 한컴 PDF 대체 명조가 깨진 HMKMM local보다 앞서야 함"
    );
    let hanyang_jung = css
        .lines()
        .find(|line| line.contains("font-family: \"한양중고딕\""))
        .expect("한양중고딕 style rule");
    assert!(
        hanyang_jung.contains("local(\"HCR Dotum\"), local(\"함초롬돋움\"), local(\"한양중고딕\")"),
        "한컴 PDF 대체 고딕이 legacy local face보다 앞서야 함"
    );
    let human_gothic = css
        .lines()
        .find(|line| line.contains("font-family: \"휴먼고딕\""))
        .expect("휴먼고딕 style rule");
    assert!(
        human_gothic
            .find("local(\"Malgun Gothic\")")
            .expect("Malgun Gothic local")
            < human_gothic
                .find("local(\"휴먼고딕\")")
                .expect("휴먼고딕 local"),
        "실제 CSS에서도 outline 고딕이 깨진 HMKMG local보다 앞서야 함"
    );
    let hanyang = css
        .lines()
        .find(|line| line.contains("font-family: \"한양신명조\""))
        .expect("한양신명조 style rule");
    assert!(
        hanyang.contains(
            "local(\"한양신명조\"), local(\"HY신명조\"), local(\"HYSinMyeongJo-Medium\")"
        ),
        "정상 outline 한양신명조의 local 우선순위를 보존해야 함"
    );
    let human_myeongjo_bold = css
        .lines()
        .find(|line| {
            line.contains("font-family: \"휴먼명조\"") && line.contains("font-weight: bold")
        })
        .expect("휴먼명조 bold style rule");
    assert!(
        human_myeongjo_bold.contains("local(\"HCR Batang Bold\")"),
        "휴먼명조 bold는 HCR Batang Bold를 명시해 synthetic bold를 피해야 함"
    );
    let hanyang_jung_bold = css
        .lines()
        .find(|line| {
            line.contains("font-family: \"한양중고딕\"") && line.contains("font-weight: bold")
        })
        .expect("한양중고딕 bold style rule");
    assert!(
        hanyang_jung_bold.contains("local(\"HCR Dotum Bold\")"),
        "한양중고딕 bold는 HCR Dotum Bold를 명시해 synthetic bold를 피해야 함"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn full_font_embed_uses_real_bold_face_when_document_uses_bold() {
    let dir = std::env::temp_dir().join(format!("rhwp-svg-bold-font-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temporary font directory");
    std::fs::write(dir.join("HANBatang.ttf"), b"regular").expect("regular test font");
    std::fs::write(dir.join("HANBatangB.ttf"), b"bold").expect("bold test font");

    let mut renderer = SvgRenderer::new();
    renderer.font_embed_mode = FontEmbedMode::Full;
    renderer
        .font_codepoints
        .entry("휴먼명조".to_string())
        .or_default()
        .extend("한글".chars());
    renderer.font_bold_families.insert("휴먼명조".to_string());

    let css = generate_font_style(
        &renderer,
        std::slice::from_ref(&dir),
        &std::collections::HashMap::<String, Vec<u8>>::new(),
    );
    std::fs::remove_dir_all(&dir).expect("temporary font directory cleanup");

    assert!(
        css.contains("font-family: \"휴먼명조\"; src: url(\"data:font/opentype;base64,Ym9sZA==\") format(\"opentype\"); font-weight: bold;"),
        "full embed는 별도 Bold TTF를 font-weight: bold face로 선언해야 함"
    );
}

#[test]
fn test_svg_draw_text_superscript_adjusts_baseline_and_size() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text(
        "1",
        10.0,
        100.0,
        &TextStyle {
            font_size: 20.0,
            superscript: true,
            ..Default::default()
        },
    );
    let output = renderer.output();
    assert!(output.contains("font-size=\"14\""));
    assert!(output.contains("y=\"94\""));
}

/// 주어진 글리프를 담은 `<text>` 줄에서 `textLength` 값을 뽑아낸다.
fn text_length_of(output: &str, glyph: &str) -> f64 {
    let needle = format!(">{glyph}</text>");
    let line = output
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("SVG 에 `{glyph}` <text> 가 있어야 함"));
    let value = line
        .split("textLength=\"")
        .nth(1)
        .unwrap_or_else(|| panic!("`{glyph}` 는 textLength 를 가져야 함: {line}"))
        .split('"')
        .next()
        .unwrap_or_else(|| panic!("textLength 값이 닫히지 않음: {line}"));
    value
        .parse()
        .unwrap_or_else(|_| panic!("textLength 는 수치여야 함: {line}"))
}

fn text_x_of(output: &str, glyph: &str) -> f64 {
    let needle = format!(">{glyph}</text>");
    let line = output
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("SVG 에 `{glyph}` <text> 가 있어야 함"));
    let value = line
        .split(" x=\"")
        .nth(1)
        .unwrap_or_else(|| panic!("`{glyph}` 는 x 좌표를 가져야 함: {line}"))
        .split('"')
        .next()
        .unwrap_or_else(|| panic!("x 값이 닫히지 않음: {line}"));
    value
        .parse()
        .unwrap_or_else(|_| panic!("x 좌표는 수치여야 함: {line}"))
}

#[test]
fn test_svg_draw_text_script_scales_text_length_by_glyph_size() {
    // [#2771] 첨자 글리프는 본문의 0.7 배 크기로 그려진다. 그런데 폭 맞춤에 쓰는
    // textLength 가 본문(base) advance 그대로면 lengthAdjust="spacingAndGlyphs"
    // 가 0.7 배 글리프를 본문 폭까지 되늘려 1/0.7 ≈ 1.43 배 가로 확대가 난다.
    // → textLength 도 0.7 배여야 한다.
    let base_style = TextStyle {
        font_size: 20.0,
        font_family: "돋움".to_string(),
        ..Default::default()
    };
    let mut base_renderer = SvgRenderer::new();
    base_renderer.begin_page(800.0, 600.0);
    base_renderer.draw_text("1", 10.0, 100.0, &base_style);
    let base_length = text_length_of(base_renderer.output(), "1");
    assert!(base_length > 0.0, "본문 숫자는 textLength 를 가져야 함");

    for style in [
        TextStyle {
            superscript: true,
            ..base_style.clone()
        },
        TextStyle {
            subscript: true,
            ..base_style.clone()
        },
    ] {
        let mut renderer = SvgRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.draw_text("1", 10.0, 100.0, &style);
        let script_length = text_length_of(renderer.output(), "1");
        assert!(
            (script_length - base_length * 0.7).abs() < 0.001,
            "첨자 textLength 는 본문의 0.7 배여야 함: base={base_length}, script={script_length}"
        );
    }
}

#[test]
fn test_svg_draw_text_non_script_text_length_is_unchanged() {
    // [#2771] 배율 인자는 비첨자에서 **정확히 1.0** 이라 기존 golden textLength
    // 값이 비트 단위로 보존된다. 레이아웃 advance 자체는 첨자에서도 본문 기준을
    // 유지하므로(그리기 크기만 축소) 두 run 의 char_positions 는 동일하다.
    let style = TextStyle {
        font_size: 20.0,
        font_family: "돋움".to_string(),
        ..Default::default()
    };
    assert_eq!(style.script_advance_scale(), 1.0);

    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text("1", 10.0, 100.0, &style);
    let length = text_length_of(renderer.output(), "1");
    // `x * 1.0` 은 IEEE-754 상 반올림 없는 항등 연산이다.
    assert_eq!(length * style.script_advance_scale(), length);
}

#[test]
fn extra_char_spacing_does_not_expand_ascii_glyph_width() {
    let base_style = TextStyle {
        font_size: 20.0,
        font_family: "돋움".to_string(),
        ..Default::default()
    };

    let mut base_renderer = SvgRenderer::new();
    base_renderer.begin_page(800.0, 600.0);
    base_renderer.draw_text("A1", 10.0, 100.0, &base_style);
    let base_output = base_renderer.output();

    let mut spaced_renderer = SvgRenderer::new();
    spaced_renderer.begin_page(800.0, 600.0);
    spaced_renderer.draw_text(
        "A1",
        10.0,
        100.0,
        &TextStyle {
            extra_char_spacing: 20.0,
            ..base_style
        },
    );
    let spaced_output = spaced_renderer.output();

    for glyph in ["A", "1"] {
        let base_length = text_length_of(base_output, glyph);
        let spaced_length = text_length_of(spaced_output, glyph);
        assert!(
            (spaced_length - base_length).abs() < 0.001,
            "배분 자간은 영문·숫자 glyph 폭을 늘리면 안 됨: \
             glyph={glyph}, base={base_length}, spaced={spaced_length}"
        );
    }

    let base_second_x = text_x_of(base_output, "1");
    let spaced_second_x = text_x_of(spaced_output, "1");
    assert!(
        (spaced_second_x - base_second_x - 20.0).abs() < 0.001,
        "glyph 폭은 고정해도 배분 간격은 다음 글자의 위치에 유지되어야 함: \
         base_x={base_second_x}, spaced_x={spaced_second_x}"
    );
}

#[test]
fn negative_extra_char_spacing_preserves_existing_svg_glyph_fit() {
    let base_style = TextStyle {
        font_size: 20.0,
        font_family: "돋움".to_string(),
        ..Default::default()
    };
    let mut base_renderer = SvgRenderer::new();
    base_renderer.begin_page(800.0, 600.0);
    base_renderer.draw_text("A1", 10.0, 100.0, &base_style);
    let base_output = base_renderer.output();

    let mut spaced_renderer = SvgRenderer::new();
    spaced_renderer.begin_page(800.0, 600.0);
    spaced_renderer.draw_text(
        "A1",
        10.0,
        100.0,
        &TextStyle {
            extra_char_spacing: -2.0,
            ..base_style
        },
    );
    let spaced_output = spaced_renderer.output();

    for glyph in ["A", "1"] {
        let base_length = text_length_of(base_output, glyph);
        let spaced_length = text_length_of(spaced_output, glyph);
        assert!(
            (spaced_length - (base_length - 2.0)).abs() < 0.001,
            "음수 셀 보정은 기존 SVG glyph-fit 폭을 유지해야 함: \
             glyph={glyph}, base={base_length}, spaced={spaced_length}"
        );
    }

    let base_second_x = text_x_of(base_output, "1");
    let spaced_second_x = text_x_of(spaced_output, "1");
    assert!(
        (spaced_second_x - base_second_x + 2.0).abs() < 0.001,
        "음수 배분 간격은 다음 글자의 위치에는 유지되어야 함: \
         base_x={base_second_x}, spaced_x={spaced_second_x}"
    );
}

#[test]
fn test_svg_draw_text_corner_quote_uses_halfwidth_text_length() {
    let render = |extra_char_spacing| {
        let mut renderer = SvgRenderer::new();
        renderer.begin_page(800.0, 600.0);
        renderer.draw_text(
            "「여",
            10.0,
            100.0,
            &TextStyle {
                font_size: 13.333,
                font_family: "돋움체".to_string(),
                extra_char_spacing,
                ..Default::default()
            },
        );
        renderer.output().to_string()
    };
    let base_output = render(0.0);
    let negative_output = render(-2.0);

    for output in [&base_output, &negative_output] {
        let quote_line = output
            .lines()
            .find(|line| line.contains(">「</text>"))
            .expect("SVG must emit the opening corner quote");
        let hangul_line = output
            .lines()
            .find(|line| line.contains(">여</text>"))
            .expect("SVG must emit the following Hangul character");
        assert!(
            quote_line.contains("textLength="),
            "`「` glyph 는 음수 셀 보정에서도 textLength 를 가져야 함: {quote_line}"
        );
        assert!(
            !hangul_line.contains("textLength="),
            "일반 한글 glyph 는 낫표 보정의 영향을 받으면 안 됨: {hangul_line}"
        );
    }

    let base_quote_length = text_length_of(&base_output, "「");
    let negative_quote_length = text_length_of(&negative_output, "「");
    assert!(
        (negative_quote_length - (base_quote_length - 2.0)).abs() < 0.001,
        "음수 셀 보정은 낫표의 기존 SVG glyph-fit 폭을 유지해야 함: \
         base={base_quote_length}, negative={negative_quote_length}"
    );
    assert!(
        (text_x_of(&negative_output, "여") - text_x_of(&base_output, "여") + 2.0).abs() < 0.001,
        "음수 배분 간격은 낫표 다음 글자의 위치에는 유지되어야 함"
    );
}

#[test]
fn test_svg_draw_rect() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_rect(
        10.0,
        20.0,
        100.0,
        50.0,
        0.0,
        &ShapeStyle {
            fill_color: Some(0x00FF0000),
            stroke_color: Some(0x00000000),
            stroke_width: 2.0,
            ..Default::default()
        },
    );
    let output = renderer.output();
    assert!(output.contains("<rect"));
    assert!(output.contains("fill=\"#0000ff\"")); // BGR → RGB
}

#[test]
fn test_svg_draw_path() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    let commands = vec![
        PathCommand::MoveTo(0.0, 0.0),
        PathCommand::LineTo(100.0, 0.0),
        PathCommand::ClosePath,
    ];
    renderer.draw_path(&commands, &ShapeStyle::default());
    let output = renderer.output();
    assert!(output.contains("<path"));
    assert!(output.contains("M0 0"));
    assert!(output.contains("L100 0"));
    assert!(output.contains("Z"));
}

#[test]
fn test_svg_text_decoration() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text(
        "밑줄",
        10.0,
        20.0,
        &TextStyle {
            font_size: 16.0,
            underline: UnderlineType::Bottom,
            ..Default::default()
        },
    );
    renderer.draw_text(
        "취소",
        10.0,
        40.0,
        &TextStyle {
            font_size: 16.0,
            strikethrough: true,
            ..Default::default()
        },
    );
    let output = renderer.output();
    // 밑줄: <line> 요소로 출력
    let underline_count = output.matches("y1=\"22\"").count(); // y + 2.0
    assert!(underline_count > 0, "밑줄 <line> 요소가 있어야 함");
    // 취소선: <line> 요소로 출력
    let strike_count = output
        .matches("stroke=\"#000000\" stroke-width=\"1\"")
        .count();
    assert!(strike_count >= 2, "취소선과 밑줄 <line> 요소가 있어야 함");
}

#[test]
fn test_svg_text_ratio() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    // ratio 80%: 문자별 transform 적용
    renderer.draw_text(
        "장평",
        50.0,
        100.0,
        &TextStyle {
            font_size: 16.0,
            ratio: 0.8,
            ..Default::default()
        },
    );
    let output = renderer.output();
    // 첫 문자 '장': translate(50,100) scale(0.8000,1)
    assert!(output.contains("transform=\"translate(50,100) scale(0.8000,1)\""));
    // 문자별 렌더링이므로 각 문자가 개별 <text> 요소
    let text_count = output.matches("<text ").count();
    assert_eq!(text_count, 2, "2개 문자 = 2개 <text> 요소");
}

#[test]
fn test_svg_text_ratio_default() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    // ratio 100%: transform 미적용, 문자별 x좌표
    renderer.draw_text(
        "기본",
        50.0,
        100.0,
        &TextStyle {
            font_size: 16.0,
            ratio: 1.0,
            ..Default::default()
        },
    );
    let output = renderer.output();
    assert!(!output.contains("transform="));
    // 첫 문자는 x=50
    assert!(output.contains("x=\"50\""));
    // 두 번째 문자는 x > 50 (font_size=16 기준)
    let text_count = output.matches("<text ").count();
    assert_eq!(text_count, 2, "2개 문자 = 2개 <text> 요소");
}

#[test]
fn test_svg_text_char_positions() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    // 자간이 있는 경우 문자별 위치가 정확한지 확인
    let style = TextStyle {
        font_size: 16.0,
        letter_spacing: 2.0,
        ..Default::default()
    };
    renderer.draw_text("AB", 10.0, 20.0, &style);
    let output = renderer.output();
    // letter-spacing SVG 속성은 없어야 함 (좌표에 반영됨)
    assert!(!output.contains("letter-spacing="));
    // 2개 문자 = 2개 <text> 요소
    let text_count = output.matches("<text ").count();
    assert_eq!(text_count, 2);
}

#[test]
fn test_xml_escape() {
    assert_eq!(escape_xml("<test>&\"'"), "&lt;test&gt;&amp;&quot;&apos;");
}

#[test]
fn test_color_to_svg() {
    assert_eq!(color_to_svg(0x000000FF), "#ff0000");
    assert_eq!(color_to_svg(0x00FFFFFF), "#ffffff");
}

/// 최소 2x2 BI_RGB 32-bit BMP를 생성한다 (테스트용).
fn make_minimal_bmp_2x2() -> Vec<u8> {
    // BMP 파일 헤더 (14B): "BM" + file_size + 0 + data_offset(54)
    // DIB 헤더 (BITMAPINFOHEADER 40B): w=2, h=2, planes=1, bpp=32, BI_RGB, size=16
    // 픽셀 데이터: 2*2*4 = 16B (BGRA)
    let pixels: [u8; 16] = [
        0xFF, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, // row 0 (아래→위 저장)
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // row 1
    ];
    let file_size: u32 = 14 + 40 + 16;
    let mut v = Vec::new();
    v.extend_from_slice(b"BM");
    v.extend_from_slice(&file_size.to_le_bytes());
    v.extend_from_slice(&[0, 0, 0, 0]);
    v.extend_from_slice(&54u32.to_le_bytes());
    v.extend_from_slice(&40u32.to_le_bytes()); // DIB size
    v.extend_from_slice(&2i32.to_le_bytes()); // width
    v.extend_from_slice(&2i32.to_le_bytes()); // height
    v.extend_from_slice(&1u16.to_le_bytes()); // planes
    v.extend_from_slice(&32u16.to_le_bytes()); // bpp
    v.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    v.extend_from_slice(&16u32.to_le_bytes()); // image size
    v.extend_from_slice(&[0, 0, 0, 0]); // x ppm
    v.extend_from_slice(&[0, 0, 0, 0]); // y ppm
    v.extend_from_slice(&[0, 0, 0, 0]); // colors used
    v.extend_from_slice(&[0, 0, 0, 0]); // important colors
    v.extend_from_slice(&pixels);
    v
}

/// 최소 2x2 RGB TIFF를 생성한다 (브라우저가 data URI를 직접 decode하지 못하는 회귀용).
fn make_minimal_tiff_2x2() -> Vec<u8> {
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
    use std::io::Cursor;

    let mut image = RgbImage::new(2, 2);
    for y in 0..2 {
        for x in 0..2 {
            image.put_pixel(x, y, Rgb([32 + x as u8, 96 + y as u8, 160]));
        }
    }
    let mut tiff = Vec::new();
    DynamicImage::ImageRgb8(image)
        .write_to(&mut Cursor::new(&mut tiff), ImageFormat::Tiff)
        .expect("TIFF fixture encode");
    tiff
}

#[test]
fn test_bmp_to_png_success() {
    let bmp = make_minimal_bmp_2x2();
    let png = bmp_bytes_to_png_bytes(&bmp).expect("BMP->PNG 변환 실패");
    // PNG 시그니처: 89 50 4E 47 0D 0A 1A 0A
    assert!(png.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
}

#[test]
fn test_bmp_to_png_invalid_returns_none() {
    let junk = vec![0u8; 32];
    assert!(bmp_bytes_to_png_bytes(&junk).is_none());
}

/// 최소 2x1 8-bit paletted PCX를 생성한다 (테스트용).
fn make_minimal_pcx_2x1() -> Vec<u8> {
    let mut header = [0u8; 128];
    header[0] = 0x0A; // PCX manufacturer
    header[1] = 0x05; // version 3.0+
    header[2] = 0x01; // RLE
    header[3] = 0x08; // bits per pixel per plane
    header[4..6].copy_from_slice(&0u16.to_le_bytes()); // xmin
    header[6..8].copy_from_slice(&0u16.to_le_bytes()); // ymin
    header[8..10].copy_from_slice(&1u16.to_le_bytes()); // xmax = width - 1
    header[10..12].copy_from_slice(&0u16.to_le_bytes()); // ymax = height - 1
    header[65] = 1; // color planes
    header[66..68].copy_from_slice(&2u16.to_le_bytes()); // bytes per line
    header[68..70].copy_from_slice(&1u16.to_le_bytes()); // color palette type

    let mut pcx = Vec::from(header);
    pcx.extend_from_slice(&[0, 1]); // white pixel, black pixel
    pcx.push(0x0C); // 256-color palette marker
    let mut palette = vec![0u8; 256 * 3];
    palette[0..3].copy_from_slice(&[255, 255, 255]);
    palette[3..6].copy_from_slice(&[0, 0, 0]);
    pcx.extend_from_slice(&palette);
    pcx
}

#[test]
fn test_pcx_to_png_maps_white_to_transparent() {
    let pcx = make_minimal_pcx_2x1();
    let png = pcx_bytes_to_png_bytes(&pcx).expect("PCX->PNG 변환 실패");
    let img = image::load_from_memory(&png)
        .expect("PNG decode")
        .to_rgba8();

    assert_eq!(img.dimensions(), (2, 1));
    assert_eq!(img.get_pixel(0, 0).0, [255, 255, 255, 0]);
    assert_eq!(img.get_pixel(1, 0).0, [0, 0, 0, 255]);
}

#[test]
fn test_page_background_image_pcx_converts_to_png() {
    let image = PageBackgroundImage {
        data: make_minimal_pcx_2x1(),
        fill_mode: ImageFillMode::FitToSize,
        brightness: 0,
        contrast: 0,
        effect: crate::model::image::ImageEffect::RealPic,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(output.contains("data:image/png;base64,iVBORw0KGgo"));
    assert!(!output.contains("data:image/x-pcx"));
}

#[test]
fn test_image_node_tiff_converts_to_png_for_svg() {
    let image = ImageNode::new(1, Some(make_minimal_tiff_2x2()));
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_image_node(&image, &bbox);

    let output = renderer.output();
    assert!(
        output.contains("data:image/png;base64,iVBORw0KGgo"),
        "SVG picture TIFF must be browser-compatible PNG: {output}"
    );
    assert!(!output.contains("data:image/tiff"));
}

#[test]
fn test_page_background_image_fit_to_size_preserves_bbox_output() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let image = PageBackgroundImage {
        data: png,
        fill_mode: ImageFillMode::FitToSize,
        brightness: 0,
        contrast: 0,
        effect: crate::model::image::ImageEffect::RealPic,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(
        output.contains(
            "<image x=\"10\" y=\"20\" width=\"100\" height=\"50\" preserveAspectRatio=\"none\""
        ),
        "FitToSize PageBackground image should keep bbox output: {output}"
    );
}

#[test]
fn test_page_background_image_center_uses_original_image_size() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let image = PageBackgroundImage {
        data: png,
        fill_mode: ImageFillMode::Center,
        brightness: 0,
        contrast: 0,
        effect: crate::model::image::ImageEffect::RealPic,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(
        output.contains(
            "<g clip-path=\"url(#fill-clip-1)\"><image x=\"59\" y=\"44\" width=\"2\" height=\"2\" preserveAspectRatio=\"none\""
        ),
        "Center PageBackground image should render at original size in bbox center: {output}"
    );
    assert!(
        !output.contains(
            "<image x=\"10\" y=\"20\" width=\"100\" height=\"50\" preserveAspectRatio=\"none\""
        ),
        "Center PageBackground image must not stretch to the full bbox: {output}"
    );
}

#[test]
fn test_page_background_image_realpic_watermark_preserves_color_with_opacity() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let image = PageBackgroundImage {
        data: png,
        fill_mode: ImageFillMode::Center,
        brightness: -50,
        contrast: 70,
        effect: crate::model::image::ImageEffect::RealPic,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(
        !output.contains("rhwp-img-bc-b-50c70"),
        "RealPic PageBackground watermark should preserve source color without brightness/contrast filter: {output}"
    );
    assert!(
        !output.contains("rhwp-realpic-watermark-tone"),
        "RealPic PageBackground watermark should bake the shared tone transform into image pixels: {output}"
    );
    assert!(
        output.contains("data:image/png;base64,"),
        "RealPic PageBackground watermark should render as a tone-baked PNG: {output}"
    );
    assert!(
        output.contains(&format!(
            "<g opacity=\"{}\">",
            REAL_PICTURE_WATERMARK_PAGE_OPACITY
        )),
        "PageBackground watermark preset should apply page watermark opacity: {output}"
    );
    assert!(
        output.contains(
            "<g clip-path=\"url(#fill-clip-1)\"><image x=\"59\" y=\"44\" width=\"2\" height=\"2\" preserveAspectRatio=\"none\""
        ),
        "PageBackground watermark should still preserve Center placement: {output}"
    );
}

#[test]
fn test_page_background_image_non_realpic_watermark_uses_legacy_opacity() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let image = PageBackgroundImage {
        data: png,
        fill_mode: ImageFillMode::FitToSize,
        brightness: -50,
        contrast: 70,
        effect: crate::model::image::ImageEffect::GrayScale,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(
        output.contains(&format!(
            "<g opacity=\"{}\">",
            LEGACY_IMAGE_WATERMARK_OPACITY
        )),
        "non-RealPic PageBackground watermark should apply legacy watermark opacity: {output}"
    );
    assert!(
        output.contains("rhwp-img-grayscale"),
        "non-RealPic PageBackground watermark should keep the image effect filter: {output}"
    );
    assert!(
        output.contains("rhwp-img-bc-b70c-50"),
        "non-RealPic PageBackground watermark should keep the display brightness/contrast filter: {output}"
    );
}

#[test]
fn test_page_background_image_uses_display_brightness_contrast_order() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let image = PageBackgroundImage {
        data: png,
        fill_mode: ImageFillMode::Center,
        // HWP5 공통 IR의 raw storage order. 화면에서는 bright=50, contrast=-15다.
        brightness: -15,
        contrast: 50,
        effect: crate::model::image::ImageEffect::RealPic,
    };
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_page_background_image(&image, &bbox);

    let output = renderer.output();
    assert!(
        output.contains("rhwp-img-bc-b50c-15"),
        "쪽 배경은 raw ImageFill 순서가 아닌 화면 bright/contrast를 써야 한다: {output}"
    );
    assert!(
        !output.contains("rhwp-img-bc-b-15c50"),
        "raw 저장 순서를 화면 필터에 직접 넘기면 안 된다: {output}"
    );
}

#[test]
fn test_background_image_realpic_watermark_fill_preserves_color_with_opacity() {
    let png = bmp_bytes_to_png_bytes(&make_minimal_bmp_2x2()).expect("BMP->PNG 변환 실패");
    let mut image = ImageNode::new(1, Some(png));
    image.fill_mode = Some(ImageFillMode::FitToSize);
    image.brightness = -50;
    image.contrast = 70;
    image.effect = crate::model::image::ImageEffect::RealPic;
    let bbox = BoundingBox::new(10.0, 20.0, 100.0, 50.0);
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(200.0, 100.0);

    renderer.render_image_node(&image, &bbox);

    let output = renderer.output();
    assert!(
        !output.contains("rhwp-img-bc-b-50c70"),
        "RealPic background watermark fill should preserve source color without brightness/contrast filter: {output}"
    );
    assert!(
        !output.contains("rhwp-realpic-watermark-tone"),
        "RealPic background watermark fill should bake the shared tone transform into image pixels: {output}"
    );
    assert!(
        output.contains(&format!(
            "<g opacity=\"{}\">",
            REAL_PICTURE_WATERMARK_FILL_OPACITY
        )),
        "RealPic background watermark fill should apply fill watermark opacity: {output}"
    );
}

#[test]
fn test_brightness_contrast_filter_zero_returns_none() {
    let mut renderer = SvgRenderer::new();
    assert!(renderer.ensure_brightness_contrast_filter(0, 0).is_none());
    assert!(renderer.defs.is_empty());
}

#[test]
fn test_brightness_contrast_filter_nonzero_adds_defs() {
    let mut renderer = SvgRenderer::new();
    let id = renderer.ensure_brightness_contrast_filter(30, -20);
    assert!(id.is_some());
    let id = id.unwrap();
    assert_eq!(id, "rhwp-img-bc-b30c-20");
    assert_eq!(renderer.defs.len(), 1);
    let def = &renderer.defs[0];
    assert!(def.contains(&format!("id=\"{}\"", id)));
    assert!(def.contains("<feComponentTransfer>"));
    assert!(def.contains("feFuncR"));
}

#[test]
fn test_brightness_contrast_filter_dedup() {
    let mut renderer = SvgRenderer::new();
    renderer.ensure_brightness_contrast_filter(50, 50);
    renderer.ensure_brightness_contrast_filter(50, 50);
    assert_eq!(renderer.defs.len(), 1);
}

/// 순수 밝기 (b=50, c=0) → slope=1.0, intercept=0.5
#[test]
fn test_brightness_contrast_filter_pure_brightness() {
    let mut renderer = SvgRenderer::new();
    renderer.ensure_brightness_contrast_filter(50, 0);
    let def = &renderer.defs[0];
    assert!(
        def.contains("slope=\"1.0000\""),
        "slope expected 1.0000: {def}"
    );
    assert!(
        def.contains("intercept=\"0.5000\""),
        "intercept expected 0.5000: {def}"
    );
}

/// 순수 대비 (b=0, c=50) → slope=1.5, intercept=-0.25
#[test]
fn test_brightness_contrast_filter_pure_contrast() {
    let mut renderer = SvgRenderer::new();
    renderer.ensure_brightness_contrast_filter(0, 50);
    let def = &renderer.defs[0];
    assert!(
        def.contains("slope=\"1.5000\""),
        "slope expected 1.5000: {def}"
    );
    assert!(
        def.contains("intercept=\"-0.2500\""),
        "intercept expected -0.2500: {def}"
    );
}

/// HWP 범위 외 입력은 -100..=100 으로 clamp — i8 max/min → 100/-100
#[test]
fn test_brightness_contrast_filter_clamp_out_of_range() {
    let mut renderer = SvgRenderer::new();
    let id = renderer
        .ensure_brightness_contrast_filter(127, -128)
        .expect("clamp 후 nonzero");
    assert_eq!(id, "rhwp-img-bc-b100c-100");
    assert_eq!(renderer.defs.len(), 1);
}

#[test]
fn test_compute_image_crop_src_exam_kor_header() {
    // [Task #477] HWP 표준 75 HU/px 룰 적용.
    // exam_kor.hwp bin_id=27: image 픽셀 2320×354 (= 174000/75 × 26580/75 HU),
    // crop=(0, 0, 102366, 26580) → 좌측 1364.88px × 354px (= "국어 영역")
    let (sx, sy, sw, sh) =
        compute_image_crop_src((0, 0, 102366, 26580), Some((174000, 26580)), 2320.0, 354.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    // 102366 / 75 = 1364.88
    assert!((sw - 1364.88).abs() < 0.01);
    // imgDim 세로 범위가 디코딩 이미지 전체 높이에 대응한다.
    assert!((sh - 354.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_no_crop_full_image() {
    // crop이 원본 전체를 가리키면 src도 이미지 전체와 일치
    let (sx, sy, sw, sh) =
        compute_image_crop_src((0, 0, 174000, 26580), Some((174000, 26580)), 2320.0, 354.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    // 174000 / 75 = 2320 (= image width)
    assert!((sw - 2320.0).abs() < 0.01);
    assert!((sh - 354.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_offset_top_left() {
    // 좌·상단을 잘라낸 케이스: top=oh/5, left=ow/4 → 우하단 영역.
    // imgDim 부재 → 적응 폴백(#3239): right/bottom(4000, 2500)이 전체 좌표
    // 범위 = 디코딩 400×250px 에 대응 (10 HU/px).
    let (sx, sy, sw, sh) = compute_image_crop_src((1000, 500, 4000, 2500), None, 400.0, 250.0);
    // src_x = 1000/10 = 100, src_y = 500/10 = 50
    // src_w = 3000/10 = 300, src_h = 2000/10 = 200
    assert!((sx - 100.0).abs() < 0.01);
    assert!((sy - 50.0).abs() < 0.01);
    assert!((sw - 300.0).abs() < 0.01);
    assert!((sh - 200.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_kwater_pi31() {
    // [Task #477] k-water-rfp.hwp pi=31 케이스 (회귀 정정 검증):
    // PNG (169 × 93 px) 가 이미 crop 적용 후 image — viewBox 가 image 전체와
    // 매칭해야 (좌측 일부만 보이는 결함 정정).
    // crop=(0, 0, 12660, 6960), imgDim=12660×6960.
    let (sx, sy, sw, sh) =
        compute_image_crop_src((0, 0, 12660, 6960), Some((12660, 6960)), 169.0, 93.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 169.0).abs() < 0.01);
    assert!((sh - 93.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_issue2817_img_dim_scale() {
    // issue2817 image2.png: 192×108 px, imgDim/crop 전체 범위 144000×81000.
    // 고정 75 HU/px를 적용하면 1920×1080으로 계산되어 그림이 1/10만 표시된다.
    let (sx, sy, sw, sh) =
        compute_image_crop_src((0, 0, 144000, 81000), Some((144000, 81000)), 192.0, 108.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 192.0).abs() < 0.01);
    assert!((sh - 108.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_fallback_when_original_size_missing() {
    // original_size_hu(imgDim) 부재 시 적응 폴백(#3239): crop right/bottom
    // (102366, 26580)이 전체 좌표 범위 = 디코딩 2320×354px 에 대응한다고 본다.
    // pre-#2990 skia 경로(image_conv.rs)와 동일한 해석 — crop 이 전체 범위를
    // 가리키는 그림(대부분의 무-crop 저장)은 단위와 무관하게 정확하다.
    let (sx, sy, sw, sh) = compute_image_crop_src((0, 0, 102366, 26580), None, 2320.0, 354.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 2320.0).abs() < 0.01);
    assert!((sh - 354.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_issue3239_non_96dpi_scan_fallback() {
    // #3239 r22 회귀 재현 실측값: samples/issue3239 평가결과서 BIN0001.TIF —
    // 200dpi 스캔(36 HU/px), 디코딩 1654×2340px, crop=(0,0,59520,84240),
    // raw_picture_extra 9바이트로 imgDim 부재.
    // 고정 75 룰이면 src=793.6×1123.2 로 과소 계산되어 좌상단만 2.08배
    // 확대·절단 렌더된다. 적응 폴백은 전체 이미지를 그대로 돌려준다.
    let (sx, sy, sw, sh) = compute_image_crop_src((0, 0, 59520, 84240), None, 1654.0, 2340.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 1654.0).abs() < 0.01);
    assert!((sh - 2340.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_last_resort_hu_rule() {
    // crop right/bottom 이 무효(≤0)이고 imgDim 도 없으면 최후 폴백으로
    // [Task #477] 75 HU/px 룰을 유지한다.
    let (sx, sy, sw, sh) = compute_image_crop_src((-300, -150, 0, 0), None, 400.0, 250.0);
    assert!((sx - -4.0).abs() < 0.01);
    assert!((sy - -2.0).abs() < 0.01);
    assert!((sw - 4.0).abs() < 0.01);
    assert!((sh - 2.0).abs() < 0.01);
}

/// [#4085] 테두리 없는 글자겹침(`border_type=0`)은 `charSz` 축소를 받지 않고
/// 본문과 같은 글자 크기로 나가야 한다.
///
/// 관세청 월간 수출입 현황 p1 의 절 제목 번호(`charSz=-4`)가
/// 한컴 PDF 에서 본문과 같은 `101 Tf`, 같은 baseline 으로 나온다. 축소를 걸면
/// 본문 대비 60% 로 렌더돼 다른 폰트처럼 보인다.
#[test]
fn char_overlap_without_border_keeps_body_font_size() {
    let style = TextStyle {
        font_size: 22.67,
        ..Default::default()
    };
    let overlap = CharOverlapInfo {
        border_type: 0,
        inner_char_size: -4,
    };

    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_char_overlap("1", &style, &overlap, 10.0, 20.0, 22.67, 22.67);
    let output = renderer.output();

    assert!(
        output.contains("font-size=\"22.67\""),
        "테두리 없는 글자겹침은 본문 크기 유지: {output}"
    );
    assert!(
        !output.contains("<ellipse"),
        "border_type=0 은 테두리를 그리지 않는다: {output}"
    );
}

/// [#4158] 실제 CharOverlap의 U+F02B1은 raw border가 0이어도 문자 자체의 의미에 따라
/// 사각형과 숫자 1로 합성해야 한다. raw PUA를 폰트에 맡기면 backend별 tofu가 생긴다.
#[test]
fn boxed_pua_char_overlap_draws_square_and_number() {
    let style = TextStyle {
        font_size: 22.67,
        ..Default::default()
    };
    let overlap = CharOverlapInfo {
        border_type: 0,
        inner_char_size: 0,
    };

    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_char_overlap("\u{F02B1}", &style, &overlap, 10.0, 20.0, 22.67, 22.67);
    let output = renderer.output();

    assert!(output.contains("<rect"), "사각 테두리 필요: {output}");
    assert!(output.contains(">1</text>"), "숫자 1 합성 필요: {output}");
    assert!(
        !output.contains('\u{F02B1}'),
        "raw PUA를 렌더 출력에 남기지 않음: {output}"
    );
}

/// [#4085] 회귀 금지 — 테두리가 있으면 PR #1101 의 실측 축소 규칙을 유지한다.
/// `samples/hwpx/k-water-rfp.hwpx` p13 의 반전 사각형(`charSz=-2`) 이 근거다.
#[test]
fn char_overlap_with_border_keeps_charsz_reduction() {
    let style = TextStyle {
        font_size: 22.66,
        ..Default::default()
    };
    let overlap = CharOverlapInfo {
        border_type: 4,
        inner_char_size: -2,
    };

    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_char_overlap("3", &style, &overlap, 10.0, 20.0, 22.66, 22.66);
    let output = renderer.output();

    // 22.66 × 0.80 = 18.128 → "18.13"
    assert!(
        output.contains("font-size=\"18.13\""),
        "반전 사각형은 charSz 축소 유지: {output}"
    );
    assert!(
        output.contains("<rect"),
        "border_type=4 는 사각 테두리를 그린다: {output}"
    );
}
