//! 실제 WMF 재생 경로의 SVG 자원 직렬화 계약이다. 문서 시각 일치는 별도 Print로 대조한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

fn record(function: u16, parameters: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(&(3 + parameters.len() as u32 / 2).to_le_bytes());
    output.extend_from_slice(&function.to_le_bytes());
    output.extend_from_slice(parameters);
}

fn header() -> Vec<u8> {
    // WMF placeable header와 표준 메타파일 헤더. 좌표는 화면 핀이 아닌 생성 입력의 계약이다.
    let mut bytes = 0x9AC6_CDD7u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for value in [0i16, 0, 100, 100] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&96u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for value in [1u16, 9, 0x0300] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes
}

fn diagram(items: &[(i16, i16, [u8; 3])]) -> String {
    let mut bytes = header();
    let pair = [100i16.to_le_bytes(), 100i16.to_le_bytes()].concat();
    record(0x020C, &pair, &mut bytes);
    // 같은 clip을 가진 두 그림도 각각의 표시 위치와 순서를 보존한다.
    // 실제 SVG clip을 발행하는 CreateRegion → SelectClipRegion 경로를 사용한다.
    // IntersectClipRect의 DC 기록만으로 출력 clip까지 구현됐다고 추정하지 않는다.
    let mut region = vec![];
    for value in [0u16, 6] {
        region.extend_from_slice(&value.to_le_bytes());
    }
    region.extend_from_slice(&1u32.to_le_bytes());
    for value in [34i16, 1, 2, 0, 0, 90, 90, 2, 0, 90, 0, 90, 2] {
        region.extend_from_slice(&value.to_le_bytes());
    }
    record(0x06FF, &region, &mut bytes);
    record(0x012C, &0u16.to_le_bytes(), &mut bytes);
    for &(x, width, bgr) in items {
        let mut parameters = 0x00CC_0020u32.to_le_bytes().to_vec(); // SRCCOPY
        for value in [2i16, 2, 0, 0, 30, width, 7, x] {
            parameters.extend_from_slice(&value.to_le_bytes());
        }
        parameters.extend_from_slice(&40u32.to_le_bytes());
        parameters.extend_from_slice(&2i32.to_le_bytes());
        parameters.extend_from_slice(&2i32.to_le_bytes());
        parameters.extend_from_slice(&1u16.to_le_bytes());
        parameters.extend_from_slice(&24u16.to_le_bytes());
        parameters.extend_from_slice(&[0; 24]);
        for _ in 0..2 {
            parameters.extend_from_slice(&bgr);
            parameters.extend_from_slice(&bgr);
            parameters.extend_from_slice(&[0; 2]);
        }
        record(0x0B41, &parameters, &mut bytes);
    }
    record(0, &[], &mut bytes);
    String::from_utf8(
        WMFConverter::new(bytes.as_slice(), SVGPlayer::new())
            .run()
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn wmf_repeated_png_serialization_keeps_placement_order_clip_and_ids() {
    let svg = diagram(&[(11, 20, [20, 40, 80]), (31, 20, [20, 40, 80])]);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let images: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("image"))
        .collect();
    let uses: Vec<_> = doc
        .descendants()
        .filter(|n| n.has_tag_name("use"))
        .collect();
    assert_eq!(images.len(), 1, "동일 PNG는 한 번만 직렬화한다");
    assert_eq!(uses.len(), 2, "각 원래 그림의 소유·순서는 보존한다");
    let source = images[0];
    assert!(source.parent().unwrap().has_tag_name("defs"));
    assert_eq!(source.attribute("width"), Some("20"));
    assert_eq!(source.attribute("height"), Some("30"));
    assert_eq!(uses[0].attribute("x"), Some("11"));
    assert_eq!(uses[1].attribute("x"), Some("31"));
    assert_ne!(
        uses[0].parent().unwrap().attribute("id"),
        uses[1].parent().unwrap().attribute("id")
    );
    for item in uses {
        assert_eq!(
            item.attribute("href").unwrap().strip_prefix('#'),
            source.attribute("id")
        );
        assert!(item.parent().unwrap().attribute("clip-path").is_some());
        assert!(item.attribute("clip-path").is_none());
        assert_eq!(item.attribute("y"), Some("7"));
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn wmf_shared_png_stays_inside_the_original_device_clip() {
    // 90이라는 값은 위 생성 WMF의 clip 우단이다. 실물 문서 화면 좌표를 고정하지 않는다.
    let svg = diagram(&[(80, 20, [20, 40, 80]), (85, 20, [20, 40, 80])]);
    let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default()).unwrap();
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let mut painted = 0;
    for (index, pixel) in pixmap.pixels().iter().enumerate() {
        if pixel.alpha() != 0 {
            painted += 1;
            let x = index % size.width() as usize;
            assert!(
                x < 90,
                "그림 이동이 원래 clip 우단을 이동시키면 안 된다: {x}"
            );
        }
    }
    assert!(painted > 0, "전부 지워서 클립 관계를 통과시키면 안 된다");
}

#[test]
fn wmf_png_serialization_keeps_distinct_size_and_bitmap_payloads() {
    let svg = diagram(&[
        (1, 20, [20, 40, 80]),
        (2, 40, [20, 40, 80]),
        (3, 20, [90, 60, 30]),
    ]);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    assert_eq!(
        doc.descendants()
            .filter(|n| n.has_tag_name("image"))
            .count(),
        3
    );
    assert!(!doc.descendants().any(|n| n.has_tag_name("use")));
}

fn font_diagram(face: &[u8], charset: u8) -> String {
    font_diagram_with_family(face, charset, 0)
}

/// `pitch_and_family`: LOGFONT 상위 니블이 FamilyFont(0x10 ROMAN, 0x20 SWISS, 0x30 MODERN).
fn font_diagram_with_family(face: &[u8], charset: u8, pitch_and_family: u8) -> String {
    let mut bytes = header();
    let mut font = Vec::new();
    for value in [-12i16, 0, 0, 0, 400] {
        font.extend_from_slice(&value.to_le_bytes());
    }
    font.extend_from_slice(&[0, 0, 0, charset, 0, 0, 0, pitch_and_family]);
    let mut name = [0; 32];
    name[..face.len()].copy_from_slice(face);
    font.extend_from_slice(&name);
    record(0x02FB, &font, &mut bytes);
    record(0x012D, &0u16.to_le_bytes(), &mut bytes);
    let mut text = 3u16.to_le_bytes().to_vec();
    text.extend_from_slice(b"ABC\0");
    for value in [20i16, 10] {
        text.extend_from_slice(&value.to_le_bytes());
    }
    record(0x0521, &text, &mut bytes);
    record(0, &[], &mut bytes);
    String::from_utf8(
        WMFConverter::new(bytes.as_slice(), SVGPlayer::new())
            .run()
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn wmf_korean_font_keeps_installed_family_before_generic_fallback() {
    // 굴림체의 CP949 이름과 설치 글꼴의 영문 family는 같은 face의 이름이다.
    let svg = font_diagram(&[0xB1, 0xBC, 0xB8, 0xB2, 0xC3, 0xBC], 129);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let text = doc.descendants().find(|n| n.has_tag_name("text")).unwrap();
    let families: Vec<_> = text.attribute("font-family").unwrap().split(',').collect();
    let original = families.iter().position(|f| *f == "'굴림체'").unwrap();
    let installed = families.iter().position(|f| *f == "'GulimChe'").unwrap();
    let fallback = families
        .iter()
        .position(|f| *f == "'Malgun Gothic'")
        .unwrap();
    assert!(original < installed && installed < fallback);
    assert_eq!(text.text(), Some("ABC"));
    assert_eq!(text.attribute("font-size"), Some("12"));
    assert_eq!(text.attribute("font-weight"), Some("400"));
}

#[test]
fn wmf_font_without_alias_keeps_its_original_family_and_text() {
    let svg = font_diagram(b"Arial", 0);
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let text = doc.descendants().find(|n| n.has_tag_name("text")).unwrap();
    // 별칭은 넣지 않는다: 원래 이름이 맨 앞에 한 번, 그 뒤는 계열 대체 사슬뿐이다.
    let families: Vec<_> = text.attribute("font-family").unwrap().split(',').collect();
    assert_eq!(families.first(), Some(&"'Arial'"));
    assert_eq!(families.iter().filter(|f| **f == "'Arial'").count(), 1);
    assert_eq!(families.last(), Some(&"'sans-serif'"));
    assert_eq!(text.text(), Some("ABC"));
}

#[test]
fn wmf_latin_only_face_falls_back_by_logfont_family() {
    // 차트 WMF 의 "Calibri" 만 지정된 글자(1790387 35쪽 "2018년")는 미설치 환경에서 renderer
    // 기본 명조로 떨어지면 안 된다 — 한/글 PDF 는 고딕. SWISS 는 고딕 사슬, ROMAN 은 명조 사슬.
    let families_of = |svg: String| -> Vec<String> {
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let text = doc.descendants().find(|n| n.has_tag_name("text")).unwrap();
        text.attribute("font-family")
            .unwrap()
            .split(',')
            .map(str::to_owned)
            .collect()
    };
    let swiss = families_of(font_diagram_with_family(b"Calibri", 0, 0x22));
    assert_eq!(swiss.first().map(String::as_str), Some("'Calibri'"));
    assert!(swiss.iter().any(|f| f == "'Malgun Gothic'"));
    assert_eq!(swiss.last().map(String::as_str), Some("'sans-serif'"));

    let roman = families_of(font_diagram_with_family(b"Cambria", 0, 0x12));
    assert_eq!(roman.first().map(String::as_str), Some("'Cambria'"));
    assert_eq!(roman.last().map(String::as_str), Some("'serif'"));
}
