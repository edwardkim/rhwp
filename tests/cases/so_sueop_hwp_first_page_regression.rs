#![cfg(not(target_arch = "wasm32"))]
//! SO-SUEOP 원본 HWP의 1쪽 제목 영역과 글자 전진 보존(#7807).
//!
//! 독립 한컴 Print의 제목은 원본 개체의 폭과 높이를 모두 채운다. 미리보기의
//! 종횡비로 높이를 축소해 중앙에 놓으면 제목의 위·아래 끝이 개체 영역에서 떨어진다.
//! 절대 픽셀 좌표나 전체 SVG 문자열을 고정하지 않고, 최종 SVG를 독립 파서로
//! 해석한 미리보기의 투영 영역과 같은 원본 개체의 소유 영역을 대조한다.
//! 학교명과 학년 숫자의 전진은 독립 한컴 Print의 비율 구간으로 검사한다.
//! 모든 검사는 원본 HWP의 첫 쪽만 사용하며 다른 쪽의 잠정 배치를 고정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

fn collect_ole_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::RawSvg(raw)
        if raw.control_ref.as_ref().is_some_and(|owner| owner.kind == "ole"))
    {
        out.push(node);
    }
    for child in &node.children {
        collect_ole_nodes(child, out);
    }
}

fn collect_vector_images<'a>(group: &'a usvg::Group, out: &mut Vec<&'a usvg::Image>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect_vector_images(group, out),
            usvg::Node::Image(image) if matches!(image.kind(), usvg::ImageKind::SVG(_)) => {
                out.push(image);
            }
            _ => {}
        }
    }
}

fn assert_title_fills_owned_extent(bytes: &[u8]) {
    let core = DocumentCore::from_bytes(bytes).expect("SO-SUEOP 원본 열기");
    let page = core.build_page_render_tree(0).expect("첫 쪽 배치");
    let mut owners = Vec::new();
    collect_ole_nodes(&page.root, &mut owners);
    assert_eq!(
        owners.len(),
        1,
        "첫 쪽 제목은 원본 OLE 개체 하나가 소유한다"
    );
    let owned = owners[0].bbox;
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("실제 Print SVG");
    let parsed = usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("최종 SVG 해석");
    let mut images = Vec::new();
    collect_vector_images(parsed.root(), &mut images);
    assert_eq!(images.len(), 1, "제목 벡터 미리보기는 한 번만 출력한다");
    let projected = images[0]
        .bounding_box()
        .transform(images[0].abs_transform())
        .expect("제목 투영 영역");
    let normalized = [
        (f64::from(projected.x()) - owned.x) / owned.width,
        (f64::from(projected.y()) - owned.y) / owned.height,
        f64::from(projected.width()) / owned.width - 1.0,
        f64::from(projected.height()) / owned.height - 1.0,
    ];
    // SVG 소수점 직렬화의 상대 오차만 허용한다. 실물의 픽셀 위치를 고정하지 않는다.
    assert!(
        normalized.iter().all(|value| value.abs() < 1e-4),
        "제목 미리보기는 원본 개체의 두 축을 채워야 한다: 상대 위치/크기 차이 {normalized:?}"
    );
}

fn sample(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(name),
    )
    .expect("SO-SUEOP 원본 파일")
}

#[test]
fn so_sueop_hwp_first_page_ole_title_fills_its_owned_extent() {
    assert_title_fills_owned_extent(&sample("SO-SUEOP.hwp"));
}

fn collect_school_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.display_or_text().contains("협성고등학교"))
    {
        out.push(node);
    }
    for child in &node.children {
        collect_school_runs(child, out);
    }
}

fn school_glyph_advances(name: &str) -> Vec<f64> {
    let core = DocumentCore::from_bytes(&sample(name)).expect("원본 열기");
    let page = core.build_page_render_tree(0).expect("첫 쪽 배치");
    let mut runs = Vec::new();
    collect_school_runs(&page.root, &mut runs);
    assert_eq!(runs.len(), 1, "학교명은 하나의 원본 문단이 소유한다");
    let RenderNodeType::TextRun(run) = &runs[0].node_type else {
        unreachable!()
    };
    assert!(run.para_index.is_some(), "학교명의 문단 소유를 보존한다");
    let declared_em = run.style.font_size * run.style.ratio;
    assert!(declared_em > 0.0);
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("최종 Print SVG");
    let parsed = roxmltree::Document::parse(&svg).expect("SVG XML 해석");
    let glyphs: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .map(|n| (n.text().unwrap_or(""), n))
        .collect();
    let school: Vec<_> = "협성고등학교".chars().map(|c| c.to_string()).collect();
    let matches: Vec<_> = glyphs
        .windows(school.len())
        .filter(|window| {
            window
                .iter()
                .zip(&school)
                .all(|((text, _), expected)| *text == expected)
        })
        .collect();
    assert_eq!(matches.len(), 1, "학교명 여섯 글자의 순서·중복 없음");
    let starts: Vec<f64> = matches[0]
        .iter()
        .map(|(_, node)| {
            let transform = node.attribute("transform").expect("최종 글리프 변환");
            let translate = transform.strip_prefix("translate(").expect("글리프 원점");
            translate
                .split([',', ' ', ')'])
                .next()
                .unwrap()
                .parse()
                .expect("x 원점")
        })
        .collect();
    starts
        .windows(2)
        .map(|pair| (pair[1] - pair[0]) / declared_em)
        .collect()
}

#[test]
fn so_sueop_hwp_first_page_preserves_independent_hft_advances() {
    // 독립 Print의 30/32/52pt HFT 굵은 한글은 장평 적용 em 대비1.037~1.041.
    // 힌팅 반올림만 포함한 비율 구간이며
    // 실물의 절대 픽셀 좌표나 구현 helper의 계산값을 기대값으로 재사용하지 않는다.
    let hwp = school_glyph_advances("SO-SUEOP.hwp");
    assert!(
        hwp.iter().all(|a| (1.035..=1.045).contains(a)),
        "HFT 독립 전진: {hwp:?}"
    );
}

fn collect_digit_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.display_or_text() == "3") {
        out.push(node);
    }
    for child in &node.children {
        collect_digit_runs(child, out);
    }
}

#[test]
fn so_sueop_hwp_first_page_digit_uses_its_declared_program_advance() {
    let name = "SO-SUEOP.hwp";
    let core = DocumentCore::from_bytes(&sample(name)).expect("원본 열기");
    let page = core.build_page_render_tree(0).expect("첫쪽 배치");
    let mut runs = Vec::new();
    collect_digit_runs(&page.root, &mut runs);
    assert_eq!(runs.len(), 1, "학년 숫자의 원본 소유와 중복 없음");
    let RenderNodeType::TextRun(run) = &runs[0].node_type else {
        unreachable!()
    };
    let declared_em = run.style.font_size * run.style.ratio;
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .unwrap();
    let parsed = roxmltree::Document::parse(&svg).unwrap();
    let glyphs: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect();
    let pairs: Vec<_> = glyphs
        .windows(2)
        .filter(|p| p[0].text() == Some("3") && p[1].text() == Some("학"))
        .collect();
    assert_eq!(pairs.len(), 1, "학년 숫자 다음 한글 순서를 보존한다");
    let x = |node: roxmltree::Node<'_, '_>| -> f64 {
        node.attribute("transform")
            .unwrap()
            .strip_prefix("translate(")
            .unwrap()
            .split([',', ' ', ')'])
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    let ratio = (x(pairs[0][1]) - x(pairs[0][0])) / declared_em;
    // 독립 Print의 HCI Tulip 숫자 Widths498/1000과 같은 글줄의 실제 원점
    // 차이는 장평 적용 em의 약0.493이다. 절대 위치를 고정하지 않는다.
    assert!(
        (0.489..=0.497).contains(&ratio),
        "{name} 숫자 프로그램 전진: {ratio}"
    );
}
