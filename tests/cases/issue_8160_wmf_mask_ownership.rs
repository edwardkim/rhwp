//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_6865_wmf_monochrome_mask_rop.rs
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

fn to_svg(bytes: &[u8]) -> String {
    let bytes = WMFConverter::new(bytes, SVGPlayer::new())
        .run()
        .expect("WMF 변환은 성공해야 한다");
    String::from_utf8(bytes).expect("SVG 는 UTF-8 이다")
}

/// 사용자 제공 한컴2024 Print64쪽과 Windows GDI+에서 확인한 분홍색 패널이다.
/// 좌표를 고정하지 않고 실물 WMF의 브러시와 역흑백 mask 소유를 검사한다.
#[test]
fn issue_8160_sample16_pink_panels_preserve_their_original_brush_and_contained_mask() {
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwp"),
    )
    .expect("원본 HWP");
    let document = rhwp::parser::parse_hwp(&input).expect("원본 파싱");
    let image = document
        .bin_data_content
        .iter()
        .find(|data| data.id == 3 && data.extension.eq_ignore_ascii_case("wmf"))
        .expect("원본 서버 도해 BIN0003.WMF");
    let svg = to_svg(&image.data.load());
    let xml = roxmltree::Document::parse(&svg).expect("변환 SVG");
    let paints: Vec<_> = xml
        .descendants()
        .filter(|node| {
            node.attribute("fill") == Some("#FFCCFF")
                && !node.ancestors().any(|owner| owner.has_tag_name("defs"))
        })
        .collect();
    assert!(!paints.is_empty(), "독립 출력의 원본 분홍색 paint 보존");
    for paint in paints {
        // A contained ROP may apply the mask once to a viewport that owns both
        // the black backing and the original brush. SVG masks are inherited
        // through that ancestor's compositing, not copied onto every paint.
        let reference = paint
            .ancestors()
            .find_map(|owner| owner.attribute("mask"))
            .expect("분홍색 paint의 실제 합성 소유자가 역마스크를 적용한다");
        let id = reference
            .strip_prefix("url(#")
            .and_then(|value| value.strip_suffix(')'))
            .expect("소유 mask 참조");
        let mask = xml
            .descendants()
            .find(|node| node.attribute("id") == Some(id))
            .expect("같은 SVG의 mask");
        assert!(mask.has_tag_name("mask"));
        assert_eq!(mask.attribute("mask-type"), Some("luminance"));
        assert!(
            mask.descendants()
                .any(|node| node.attribute("filter").is_some()),
            "검정 비트가 원본 브러시를 선택하도록 역마스크 보존"
        );
    }
}
