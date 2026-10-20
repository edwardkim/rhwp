//! 독립 한컴 Print와 공식 TextLayout의 가운데 정렬·범례 위쪽 정렬.
//! 분리 원본 양형식의 Native/fresh 전체1쪽 선행 증거는99.83461% 이상이다.
//! 절대 픽셀 원점 대신 최종 SVG의 공통 정렬선과 마커/글자의 관계를 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use std::path::Path;

fn coordinate(node: roxmltree::Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name)
        .expect("출력 좌표")
        .parse()
        .expect("유한 수치 좌표")
}

fn assert_text_layout(svg: &str) {
    let xml = roxmltree::Document::parse(svg).expect("독립 SVG 파서");
    let chart = xml
        .descendants()
        .find(|n| {
            n.has_tag_name("g")
                && n.attribute("class")
                    .is_some_and(|s| s.split_whitespace().any(|c| c == "hwp-ole-chart"))
        })
        .expect("실제 OLE 차트");
    let elements: Vec<_> = chart.children().filter(|n| n.is_element()).collect();
    let mut centers = Vec::new();
    for value in ["0", "500", "1000", "1500", "2000"] {
        let labels: Vec<_> = elements
            .iter()
            .filter(|n| n.has_tag_name("text") && n.text() == Some(value))
            .collect();
        assert_eq!(labels.len(), 1, "값축 숫자의 누락·중복: {value}");
        let label = *labels[0];
        assert_eq!(
            label.attribute("text-anchor"),
            Some("middle"),
            "독립 Print의 값축은 서로 길이가 다른 숫자를 가운데 정렬한다"
        );
        centers.push(coordinate(label, "x"));
    }
    assert!(
        centers.windows(2).all(|pair| pair[0] == pair[1]),
        "값축 숫자는 같은 중심선을 공유한다"
    );
    for name in ["적립금", "수입", "지출"] {
        let index = elements
            .iter()
            .position(|n| n.has_tag_name("text") && n.text() == Some(name))
            .expect("원본 범례 문구");
        let label = elements[index];
        let marker = elements[index - 1];
        assert!(marker.has_tag_name("rect"), "범례 마커 보존");
        assert_eq!(label.attribute("dominant-baseline"), Some("central"));
        assert!(
            coordinate(label, "x") > coordinate(marker, "x") + coordinate(marker, "width"),
            "범례 글자는 자기 마커 오른쪽에 놓인다"
        );
        let center = coordinate(label, "y");
        let top = coordinate(marker, "y");
        assert!(
            center >= top && center < top + coordinate(marker, "height") / 2.0,
            "독립 Print의 범례 글자는 마커 위쪽 반에 정렬된다"
        );
    }
}

#[test]
fn hwp_and_hwpx_stored_chart_text_uses_owned_alignment() {
    for name in ["143E433F503322BD33-ole.hwp", "143E433F503322BD33-ole.hwpx"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/hwpx")
            .join(name);
        let core = DocumentCore::from_bytes(&std::fs::read(path).expect("사용자 분리 원본"))
            .expect("분리 차트 열기");
        let svg = core
            .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
            .expect("최종 Print SVG");
        assert_text_layout(&svg);
    }
}
