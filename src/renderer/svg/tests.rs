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
    renderer.draw_text("안녕하세요", 10.0, 20.0, &TextStyle {
        font_size: 16.0,
        bold: true,
        ..Default::default()
    });
    let output = renderer.output();
    assert!(output.contains("<text"));
    assert!(output.contains("font-weight=\"bold\""));
}

#[test]
fn test_svg_draw_rect() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    renderer.draw_rect(10.0, 20.0, 100.0, 50.0, 0.0, &ShapeStyle {
        fill_color: Some(0x00FF0000),
        stroke_color: Some(0x00000000),
        stroke_width: 2.0,
        ..Default::default()
    });
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
    renderer.draw_text("밑줄", 10.0, 20.0, &TextStyle {
        font_size: 16.0,
        underline: UnderlineType::Bottom,
        ..Default::default()
    });
    renderer.draw_text("취소", 10.0, 40.0, &TextStyle {
        font_size: 16.0,
        strikethrough: true,
        ..Default::default()
    });
    let output = renderer.output();
    // 밑줄: <line> 요소로 출력
    let underline_count = output.matches("y1=\"22\"").count(); // y + 2.0
    assert!(underline_count > 0, "밑줄 <line> 요소가 있어야 함");
    // 취소선: <line> 요소로 출력
    let strike_count = output.matches("stroke=\"#000000\" stroke-width=\"1\"").count();
    assert!(strike_count >= 2, "취소선과 밑줄 <line> 요소가 있어야 함");
}

#[test]
fn test_svg_text_ratio() {
    let mut renderer = SvgRenderer::new();
    renderer.begin_page(800.0, 600.0);
    // ratio 80%: 문자별 transform 적용
    renderer.draw_text("장평", 50.0, 100.0, &TextStyle {
        font_size: 16.0,
        ratio: 0.8,
        ..Default::default()
    });
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
    renderer.draw_text("기본", 50.0, 100.0, &TextStyle {
        font_size: 16.0,
        ratio: 1.0,
        ..Default::default()
    });
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
        0xFF, 0x00, 0x00, 0xFF,  0x00, 0xFF, 0x00, 0xFF, // row 0 (아래→위 저장)
        0x00, 0x00, 0xFF, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF, // row 1
    ];
    let file_size: u32 = 14 + 40 + 16;
    let mut v = Vec::new();
    v.extend_from_slice(b"BM");
    v.extend_from_slice(&file_size.to_le_bytes());
    v.extend_from_slice(&[0, 0, 0, 0]);
    v.extend_from_slice(&54u32.to_le_bytes());
    v.extend_from_slice(&40u32.to_le_bytes());          // DIB size
    v.extend_from_slice(&2i32.to_le_bytes());           // width
    v.extend_from_slice(&2i32.to_le_bytes());           // height
    v.extend_from_slice(&1u16.to_le_bytes());           // planes
    v.extend_from_slice(&32u16.to_le_bytes());          // bpp
    v.extend_from_slice(&0u32.to_le_bytes());           // BI_RGB
    v.extend_from_slice(&16u32.to_le_bytes());          // image size
    v.extend_from_slice(&[0, 0, 0, 0]);                 // x ppm
    v.extend_from_slice(&[0, 0, 0, 0]);                 // y ppm
    v.extend_from_slice(&[0, 0, 0, 0]);                 // colors used
    v.extend_from_slice(&[0, 0, 0, 0]);                 // important colors
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
    assert!(def.contains("slope=\"1.0000\""), "slope expected 1.0000: {def}");
    assert!(def.contains("intercept=\"0.5000\""), "intercept expected 0.5000: {def}");
}

/// 순수 대비 (b=0, c=50) → slope=1.5, intercept=-0.25
#[test]
fn test_brightness_contrast_filter_pure_contrast() {
    let mut renderer = SvgRenderer::new();
    renderer.ensure_brightness_contrast_filter(0, 50);
    let def = &renderer.defs[0];
    assert!(def.contains("slope=\"1.5000\""), "slope expected 1.5000: {def}");
    assert!(def.contains("intercept=\"-0.2500\""), "intercept expected -0.2500: {def}");
}

/// HWP 범위 외 입력은 -100..=100 으로 clamp — i8 max/min → 100/-100
#[test]
fn test_brightness_contrast_filter_clamp_out_of_range() {
    let mut renderer = SvgRenderer::new();
    let id = renderer.ensure_brightness_contrast_filter(127, -128).expect("clamp 후 nonzero");
    assert_eq!(id, "rhwp-img-bc-b100c-100");
    assert_eq!(renderer.defs.len(), 1);
}

#[test]
fn test_compute_image_crop_src_exam_kor_header() {
    // exam_kor.hwp bin_id=27: 원본 174000×26580 HU(=2320×354 px @ 75 HU/px),
    // crop=(0, 0, 102366, 26580) → 좌측 1364.88px × 354px (= "국어 영역")
    let (sx, sy, sw, sh) = compute_image_crop_src(
        (0, 0, 102366, 26580),
        Some((174000, 26580)),
        2320.0, 354.0,
    );
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 1364.88).abs() < 0.01);
    assert!((sh - 354.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_no_crop_full_image() {
    // crop이 원본 전체를 가리키면 src도 이미지 전체와 일치
    let (sx, sy, sw, sh) = compute_image_crop_src(
        (0, 0, 174000, 26580),
        Some((174000, 26580)),
        2320.0, 354.0,
    );
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 2320.0).abs() < 0.01);
    assert!((sh - 354.0).abs() < 0.01);
}

#[test]
fn test_compute_image_crop_src_offset_top_left() {
    // 좌·상단을 잘라낸 케이스: top=ow/4, left=ow/4 → 우하단 75% 영역
    let (sx, sy, sw, sh) = compute_image_crop_src(
        (1000, 500, 4000, 2500),
        Some((4000, 2500)),
        400.0, 250.0,
    );
    // scale_x = 4000/400 = 10 HU/px, scale_y = 2500/250 = 10 HU/px
    assert!((sx - 100.0).abs() < 0.01);   // 1000/10
    assert!((sy - 50.0).abs() < 0.01);    // 500/10
    assert!((sw - 300.0).abs() < 0.01);   // 3000/10
    assert!((sh - 200.0).abs() < 0.01);   // 2000/10
}

#[test]
fn test_compute_image_crop_src_fallback_when_original_size_missing() {
    // original_size_hu가 None이면 폴백 (cr/img_w 스케일)
    // 이 폴백은 항상 src_w == img_w 가 되는 과거 동작과 동일
    let (sx, sy, sw, sh) = compute_image_crop_src(
        (0, 0, 102366, 26580),
        None,
        2320.0, 354.0,
    );
    // 폴백 스케일 = 102366/2320 = 44.12 HU/px (X·Y 동일)
    assert!((sx - 0.0).abs() < 0.01);
    assert!((sy - 0.0).abs() < 0.01);
    assert!((sw - 2320.0).abs() < 0.01); // (cr-cl)/scale = cr/(cr/img_w) = img_w
    // 폴백 src_h는 src_w와 같은 비율이라 정확하지 않음 — 호환성 유지일 뿐
    assert!(sh > 0.0);
}
