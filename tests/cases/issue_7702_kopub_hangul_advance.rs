//! #7702: KoPubWorld 계열은 KoPub 전진폭 표로 재고, 바탕체 한글은 936/1000 이다.
//!
//! 정본 `pdf/hwpx/hancom-hwp/[2027] 온새미로 1 본교재-hwp-2020.pdf` 에 내장된
//! `KoPubWorldBatangLight` CIDFont `/W` 의 한글 전진폭은 936, `KoPubWorldDotumLight` 는 872
//! 다. 12쪽 `…족보를 이루었는데`(장평 95%·자간 −1%) 의 한글 평균 전진 878.9HU = 936×0.95 − 10.
//! 종전 rhwp 는 `KoPubWorld…` 이름을 KoPub 으로 읽지 못해 일반 폴백 0.953em 을 썼고,
//! KoPub 바탕체 계수도 0.94 였다.
//!
//! 계약: 10pt 한글 20자 줄의 글자 간격 = 계수 × 13.333px(±0.05px/자).

use rhwp::document_core::DocumentCore;

fn advances(font_name: &str, text: &str) -> Vec<f64> {
    use rhwp::model::paragraph::{CharShapeRef, Paragraph};
    use rhwp::model::style::Font;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank template");
    let mut document = core.document().clone();
    let mut font_ids = [0_u16; 7];
    for (language, faces) in document.doc_info.font_faces.iter_mut().take(7).enumerate() {
        font_ids[language] = u16::try_from(faces.len()).expect("font id");
        faces.push(Font {
            name: font_name.to_string(),
            alt_type: 1,
            ..Default::default()
        });
    }
    let mut char_shape = document
        .doc_info
        .char_shapes
        .first()
        .cloned()
        .expect("char shape");
    char_shape.raw_data = None;
    char_shape.font_ids = font_ids;
    char_shape.base_size = 1000;
    char_shape.ratios = [100; 7];
    char_shape.spacings = [0; 7];
    char_shape.relative_sizes = [100; 7];
    let char_shape_id = u32::try_from(document.doc_info.char_shapes.len()).expect("id");
    document.doc_info.char_shapes.push(char_shape);
    let mut paragraph = Paragraph::new_empty();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    document.sections[0].paragraphs = vec![paragraph];
    core.set_document(document);
    core.insert_text_native(0, 0, 0, text).expect("text");

    let svg = core.render_page_svg_native(0).expect("svg");
    let mut xs: Vec<f64> = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let head = cap.split('>').next().unwrap_or("");
        let spaced = format!(" {head}");
        let x = spaced
            .split("translate(")
            .nth(1)
            .and_then(|t| t.split([',', ' ']).next())
            .or_else(|| {
                spaced
                    .split(" x=\"")
                    .nth(1)
                    .and_then(|t| t.split('"').next())
            })
            .and_then(|v| v.parse::<f64>().ok());
        if let Some(x) = x {
            xs.push(x);
        }
    }
    xs.sort_by(f64::total_cmp);
    xs.windows(2).map(|w| w[1] - w[0]).collect()
}

fn assert_hangul_advance(font_name: &str, units: f64) {
    let text = "가나다라마바사아자차카타파하가나다라마바";
    let gaps = advances(font_name, text);
    assert!(
        gaps.len() >= 19,
        "{font_name}: 글자 위치를 찾지 못했다 — 전제 불성립 {gaps:?}"
    );
    let expected = 1000.0 * 96.0 / 7200.0 * units / 1000.0;
    let mean = gaps.iter().take(19).sum::<f64>() / 19.0;
    assert!(
        (mean - expected).abs() <= 0.05,
        "{font_name}: 한글 전진 {mean:.3}px, 기대 {expected:.3}px ({units}/1000) (#7702)"
    );
}

#[test]
fn issue_7702_kopubworld_batang_uses_kopub_hangul_advance() {
    assert_hangul_advance("KoPubWorld바탕체 Light", 936.0);
}

#[test]
fn issue_7702_kopubworld_dotum_uses_kopub_hangul_advance() {
    assert_hangul_advance("KoPubWorld돋움체 Light", 872.0);
}

#[test]
fn issue_7702_kopub_batang_hangul_advance_is_936() {
    assert_hangul_advance("KoPub바탕체 Light", 936.0);
}

#[test]
fn issue_7702_kopub_dotum_hangul_advance_unchanged() {
    assert_hangul_advance("KoPub돋움체 Light", 872.0);
}
