//! [#4062] 텍스트 EPS/AI 아트워크의 SVG 변환·표시 계약.

use rhwp::eps::convert_ai_artwork_to_svg;
use rhwp::renderer::image_resolver::{eps_renderable_bytes, is_displayable_image_data};

fn eps_with(setup: &str, body: &str) -> Vec<u8> {
    format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%BeginSetup\n{setup}\n\
         %%EndSetup\n{body}\n%%Trailer\n"
    )
    .into_bytes()
}

fn eps(body: &str) -> Vec<u8> {
    eps_with("", body)
}

fn svg_of(body: &str) -> String {
    String::from_utf8(convert_ai_artwork_to_svg(&eps(body)).expect("변환")).unwrap()
}

#[test]
fn fills_a_closed_path_and_flips_the_y_axis() {
    let svg = svg_of("0 0 0 1 k\n10 10 m\n90 10 L\n90 40 L\n10 40 L\nf");
    assert!(svg.contains("viewBox=\"0 0 100 50\""), "{svg}");
    assert!(svg.contains("M10 40L90 40L90 10L10 10Z"), "{svg}");
    assert!(svg.contains("fill=\"#000000\""), "{svg}");
}

#[test]
fn cmyk_uses_the_ai_prolog_formula() {
    let svg = svg_of("1 0.72 0 0.38 k\n0 0 m 10 0 L 10 10 L f");
    assert!(svg.contains("fill=\"#00009e\""), "{svg}");
}

#[test]
fn custom_color_tint_is_one_minus_gray() {
    let full = svg_of("1 1 0 0 (PANTONE X) 0 x\n0 0 m 10 0 L 10 10 L f");
    assert!(full.contains("fill=\"#0000ff\""), "{full}");
    let none = svg_of("1 1 0 0 (PANTONE X) 1 x\n0 0 m 10 0 L 10 10 L f");
    assert!(none.contains("fill=\"#ffffff\""), "{none}");
}

#[test]
fn rgb_and_generic_custom_color_operators() {
    let rgb = svg_of("1 0.5 0 Xa\n0 0 m 10 0 L 10 10 L f");
    assert!(rgb.contains("fill=\"#ff8000\""), "{rgb}");
    let spot = svg_of("1 0 0 (Spot) 0.5 1 Xx\n0 0 m 10 0 L 10 10 L f");
    assert!(spot.contains("fill=\"#ff8080\""), "{spot}");
}

#[test]
fn compound_path_paints_once_so_holes_survive() {
    let svg = svg_of("*u\n0 0 m 40 0 L 40 40 L 0 40 L h\n10 10 m 30 10 L 30 30 L 10 30 L h\nf\n*U");
    assert_eq!(svg.matches("<path").count(), 1, "{svg}");
    assert_eq!(svg.matches('M').count(), 2, "{svg}");
}

#[test]
fn even_odd_flag_selects_fill_rule() {
    let svg = svg_of("1 XR\n0 0 m 10 0 L 10 10 L f");
    assert!(svg.contains("fill-rule=\"evenodd\""), "{svg}");
}

#[test]
fn stroke_carries_width_dash_and_caps() {
    let svg = svg_of("0 G\n2 w\n1 J 1 j\n[3 2]0 d\n0 0 m 10 10 L S");
    assert!(svg.contains("stroke-width=\"2\""), "{svg}");
    assert!(svg.contains("stroke-dasharray=\"3 2\""), "{svg}");
    assert!(svg.contains("stroke-linecap=\"round\""), "{svg}");
    assert!(svg.contains("stroke-linejoin=\"round\""), "{svg}");
}

#[test]
fn clip_path_scope_closes_at_matching_q() {
    let svg =
        svg_of("q\n0 0 m 10 0 L 10 10 L h W n\n0 0 m 5 0 L 5 5 L f\nQ\n20 20 m 30 20 L 30 30 L f");
    assert!(svg.contains("<clipPath"), "{svg}");
    let after_close = svg.rsplit("</g>").next().unwrap_or("");
    assert!(after_close.contains("M20 30"), "{svg}");
}

#[test]
fn guide_paths_are_discarded_not_merged() {
    let svg = svg_of("(N) *\n0 0 m 1000 0 L 1000 1000 L (N) *\n10 10 m 20 10 L 20 20 L f");
    assert_eq!(svg.matches("<path").count(), 1, "{svg}");
    assert!(!svg.contains("1000"), "{svg}");
}

#[test]
fn non_printing_layer_is_skipped() {
    let visible = svg_of("1 1 1 1 0 0 0 0 0 0 Lb\n0 0 m 10 0 L 10 10 L f\nLB");
    assert_eq!(visible.matches("<path").count(), 1, "{visible}");
    let hidden =
        convert_ai_artwork_to_svg(&eps("1 1 1 0 0 0 0 0 0 0 Lb\n0 0 m 10 0 L 10 10 L f\nLB"));
    assert!(
        hidden.is_none(),
        "인쇄 안 하는 레이어만 있으면 그릴 것이 없다"
    );
}

#[test]
fn linear_gradient_instance_becomes_svg_gradient() {
    let setup = "1 Bn\n%AI5_BeginGradient: (Red & Yellow)\n(Red & Yellow) 0 2 Bd\n[\n\
                 0 1 1 0 1 50 0 %_Bs\n0 0 1 0 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
    let body = "0 0 m 100 0 L 100 50 L 0 50 L\nBb\n\
                1 (Red & Yellow) 0 0 0 100 1 0 0 1 0 0 Bg\nf\n0 BB";
    let svg =
        String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
    assert!(svg.contains("<linearGradient"), "{svg}");
    assert!(svg.contains("stop-color=\"#ff0000\""), "{svg}");
    assert!(svg.contains("stop-color=\"#ffff00\""), "{svg}");
    assert!(svg.contains("fill=\"url(#aigrad"), "{svg}");
}

#[test]
fn radial_gradient_uses_hilight_as_focus() {
    let setup = "1 Bn\n%AI5_BeginGradient: (Ball)\n(Ball) 1 2 Bd\n[\n\
                 0 0 0 0 1 50 0 %_Bs\n0 0 0 1 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
    let body = "0 0 m 50 0 L 50 50 L\nBb\n0 0 0 0 Bh\n\
                1 (Ball) 25 25 0 25 1 0 0 1 0 0 Bg\nf\n0 BB";
    let svg =
        String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
    assert!(svg.contains("<radialGradient"), "{svg}");
    assert!(svg.contains("r=\"25\""), "{svg}");
}

#[test]
fn gradient_midpoint_adds_an_interpolated_stop() {
    let setup = "1 Bn\n%AI5_BeginGradient: (Mid)\n(Mid) 0 2 Bd\n[\n\
                 0 0 0 0 1 25 0 %_Bs\n0 0 0 1 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
    let body = "0 0 m 10 0 L 10 10 L\nBb\n1 (Mid) 0 0 0 10 1 0 0 1 0 0 Bg\nf\n0 BB";
    let svg =
        String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
    assert_eq!(svg.matches("<stop").count(), 3, "{svg}");
    assert!(svg.contains("offset=\"25%\""), "{svg}");
}

#[test]
fn text_is_placed_by_its_matrix_and_upright() {
    let svg = svg_of("0 To\n1 0 0 1 10 20 0 Tp\nTP\n0 Tr\n/_Helvetica-Bold 12 0 0 Tf\n(Hi) Tx\nTO");
    assert!(svg.contains("<text"), "{svg}");
    assert!(svg.contains(">Hi<"), "{svg}");
    assert!(svg.contains("font-size=\"12\""), "{svg}");
    assert!(svg.contains("font-weight=\"bold\""), "{svg}");
    assert!(svg.contains("matrix(1 0 0 1 10 30)"), "{svg}");
}

#[test]
fn invisible_text_render_modes_are_dropped() {
    let hidden = convert_ai_artwork_to_svg(&eps(
        "0 To\n1 0 0 1 0 0 0 Tp\nTP\n3 Tr\n/_Helvetica 12 0 0 Tf\n(Hi) Tx\nTO",
    ));
    assert!(hidden.is_none(), "render 3 은 안 보이는 글자다");
}

#[test]
fn raster_image_becomes_an_embedded_png() {
    let body = "[ 10 0 0 10 5 5 ] 0 0 2 2 2 2 8 3 0 0 0 0 XI\n\
                %FF0000 00FF00\n%0000FF FFFFFF\n%AI5_EndRaster";
    let svg = svg_of(body);
    assert!(svg.contains("<image"), "{svg}");
    assert!(svg.contains("data:image/png;base64,"), "{svg}");
}

#[test]
fn pattern_definition_becomes_an_svg_pattern() {
    let setup = "%AI3_BeginPattern: (dots)\n(dots) 0 0 10 10 [\n%AI3_Tile\n\
                 (0 O 0 R 1 0 0 0 k) @\n(0 0 m 10 0 L 10 10 L 0 10 L f) &\n] E\n\
                 %AI3_EndPattern";
    let body = "(dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 0 m 50 0 L 50 50 L 0 50 L f";
    let svg =
        String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
    assert!(svg.contains("<pattern"), "{svg}");
    assert!(svg.contains("fill=\"url(#aipat"), "{svg}");
}

#[test]
fn prolog_is_not_interpreted() {
    let data = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n\
                /f { closepath F } def\n0 0 m 5 5 L 5 0 L f\n%%EndSetup\n%%Trailer\n";
    assert!(convert_ai_artwork_to_svg(data.as_bytes()).is_none());
}

#[test]
fn text_only_postscript_yields_nothing() {
    let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n";
    assert!(convert_ai_artwork_to_svg(data).is_none());
}

#[test]
fn missing_bounding_box_is_rejected() {
    let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%EndSetup\n0 0 m 10 10 L f\n";
    assert!(convert_ai_artwork_to_svg(data).is_none());
}

#[test]
fn hires_bounding_box_wins() {
    let data = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n\
                 %%HiResBoundingBox: 0 0 100.5 50.25\n%%EndSetup\n0 0 m 10 0 L 10 10 L f\n";
    let svg = String::from_utf8(convert_ai_artwork_to_svg(data).unwrap()).unwrap();
    assert!(svg.contains("viewBox=\"0 0 100.5 50.25\""), "{svg}");
}

#[test]
fn truncated_gradient_stop_does_not_panic() {
    let setup = "1 Bn\n%AI5_BeginGradient: (Broken)\n(Broken) 0 2 Bd\n[\n\
                 0 1 50 0 %_Bs\n0 0 1 0 1 50 100 %_Bs\nBD\n%AI5_EndGradient";
    let body = "0 0 m 100 0 L 100 50 L 0 50 L\nBb\n\
                1 (Broken) 0 0 0 100 1 0 0 1 0 0 Bg\nf\n0 BB";
    let svg =
        String::from_utf8(convert_ai_artwork_to_svg(&eps_with(setup, body)).unwrap()).unwrap();
    assert!(svg.contains("<linearGradient"), "{svg}");
}

#[test]
fn text_eps_artwork_is_emitted_as_svg_and_displayable() {
    let artwork = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 50\n%%EndSetup\n\
                    0 0 0 1 k\n10 10 m\n90 10 L\n90 40 L\n10 40 L\nf\n%%Trailer\n";
    let (mime, bytes) = eps_renderable_bytes(artwork).expect("아트워크는 SVG 로 변환된다");
    assert_eq!(mime, "image/svg+xml");
    assert!(bytes.starts_with(b"<svg"), "SVG 문서가 나온다");
    assert!(is_displayable_image_data(artwork), "그릴 수 있다고 본다");
}

#[test]
fn postscript_without_artwork_stays_undecodable() {
    let text_only = b"%!PS-Adobe-3.0\n%%BoundingBox: 0 0 10 10\n%%EndSetup\n%%Trailer\n";
    assert!(
        eps_renderable_bytes(text_only).is_none(),
        "변환 실패면 원본 경로로 되돌린다"
    );
    assert!(!is_displayable_image_data(text_only));
}
