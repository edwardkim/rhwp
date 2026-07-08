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

#[test]
fn test_svg_draw_text_corner_quote_uses_halfwidth_text_length() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_text(
        "「여",
        10.0,
        100.0,
        &TextStyle {
            font_size: 13.333,
            font_family: "돋움체".to_string(),
            ..Default::default()
        },
    );
    let output = renderer.output();
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
        "`「` glyph 는 반각 advance 에 맞춰 textLength 를 가져야 함: {quote_line}"
    );
    assert!(
        !hangul_line.contains("textLength="),
        "일반 한글 glyph 는 낫표 보정의 영향을 받으면 안 됨: {hangul_line}"
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
        output.contains("rhwp-img-bc-b-50c70"),
        "non-RealPic PageBackground watermark should keep the brightness/contrast filter: {output}"
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
    // 26580 / 75 = 354.4 (≈ 354 image height)
    assert!((sh - 354.4).abs() < 0.01);
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
    assert!((sh - 354.4).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_offset_top_left() {
    // 좌·상단을 잘라낸 케이스: top=ow/4, left=ow/4 → 우하단 75% 영역
    let (sx, sy, sw, sh) =
        compute_image_crop_src((1000, 500, 4000, 2500), Some((4000, 2500)), 400.0, 250.0);
    // [Task #477] 75 HU/px 룰
    // src_x = 1000/75 = 13.33, src_y = 500/75 = 6.67
    // src_w = 3000/75 = 40, src_h = 2000/75 = 26.67
    assert!((sx - 13.333).abs() < 0.01);
    assert!((sy - 6.667).abs() < 0.01);
    assert!((sw - 40.0).abs() < 0.01);
    assert!((sh - 26.667).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_kwater_pi31() {
    // [Task #477] k-water-rfp.hwp pi=31 케이스 (회귀 정정 검증):
    // PNG (169 × 93 px) 가 이미 crop 적용 후 image — viewBox 가 image 전체와
    // 매칭해야 (좌측 일부만 보이는 결함 정정).
    // crop=(0, 0, 12660, 6960), original 14119×7766 HU.
    let (sx, sy, sw, sh) =
        compute_image_crop_src((0, 0, 12660, 6960), Some((14119, 7766)), 169.0, 93.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    // 12660 / 75 = 168.8 (≈ image width 169)
    assert!((sw - 168.8).abs() < 0.01);
    // 6960 / 75 = 92.8 (≈ image height 93)
    assert!((sh - 92.8).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_fallback_when_original_size_missing() {
    // original_size_hu가 None 이어도 [Task #477] 75 HU/px 룰을 동일하게 적용.
    let (sx, sy, sw, sh) = compute_image_crop_src((0, 0, 102366, 26580), None, 2320.0, 354.0);
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 1364.88).abs() < 0.01);
    assert!((sh - 354.4).abs() < 0.01);
}
