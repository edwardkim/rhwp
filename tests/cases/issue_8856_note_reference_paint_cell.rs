//! 관계형: 원본 참조 번호의 자연 paint 칸·뒤 본문 소속과 본문 자간의 비간섭을 검사한다.
//! 주석 갱신: 2026-11-10T09:01:00+09:00.
//! report-628 95쪽의 최소 글꼴 Native97.30697% / fresh WASM97.33682% (96dpi/PDFium/raw2px).
//! 원본 글꼴 대조는 Native99.96475% / fresh WASM99.96474%이며 최소 조건과 섞지 않는다.
//! 독립 Print403.52CSSpx와 수정 전401.344CSSpx의 진단은 별도 증적이며 여기서는 절대 px를 고정하지 않는다.
//! 설치 글꼴 없이 원본 source owner·번호와 검증된 프로그램 전진의 포함 관계를 검사한다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn nodes<'a>(n: &'a RenderNode, v: &mut Vec<&'a RenderNode>) {
    v.push(n);
    for c in &n.children {
        nodes(c, v)
    }
}
fn core() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("mydocs/pr/assets/planet6897_20261106/pr8642/report-628-original.hwp"),
        )
        .unwrap(),
    )
    .unwrap()
}
/// 검사 유형: 관계형. 본문 자간만 바뀌고 paint face·크기·문자열이 같으면 참조 칸은 같다.
#[test]
fn marker_cell_does_not_inherit_body_tracking_that_paint_omits() {
    let mut observed = vec![];
    for spacing in [-12, 0] {
        let mut c = core();
        let mut d = c.document().clone();
        d.doc_info.char_shapes[75].spacings = [spacing; 7];
        c.set_document(d);
        let t = c.build_page_render_tree(94).unwrap();
        let mut v = vec![];
        nodes(&t.root, &mut v);
        let n=v.iter().find(|n|matches!(&n.node_type,RenderNodeType::FootnoteMarker(m)if m.section_index==3&&m.para_index==472)).unwrap();
        let RenderNodeType::FootnoteMarker(m) = &n.node_type else {
            unreachable!()
        };
        observed.push((
            n.bbox.width,
            m.font_family.clone(),
            m.base_font_size,
            m.text.clone(),
        ));
    }
    assert_eq!(observed[0].1, observed[1].1);
    assert_eq!(observed[0].2, observed[1].2);
    assert_eq!(observed[0].3, observed[1].3);
    assert!(
        (observed[0].0 - observed[1].0).abs() < 0.001,
        "같은 paint 조건의 참조 칸은 본문 자간만 바꿔도 달라지지 않는다: {:?}",
        observed
    );
}

/// 검사 유형: 관계형. paint가 사용하는 face·축소 크기의 자연 전진 칸에 뒤 본문이 침범하지 않는다.
#[test]
fn following_text_stays_outside_the_reference_program_cell() {
    let c = core();
    let tree = c.build_page_render_tree(94).unwrap();
    let mut all = vec![];
    nodes(&tree.root, &mut all);
    let markers:Vec<_>=all.iter().filter(|n|matches!(&n.node_type,RenderNodeType::FootnoteMarker(m)if m.section_index==3&&m.para_index==472)).collect();
    assert_eq!(markers.len(), 1);
    let marker = markers[0];
    let RenderNodeType::FootnoteMarker(m) = &marker.node_type else {
        unreachable!()
    };
    assert_eq!(m.text, "56)");
    let metric = rhwp::renderer::font_metrics_data::find_metric(&m.font_family, false, false)
        .expect("원본 paint face의 확인된 프로그램 전진")
        .metric;
    let natural = m
        .text
        .chars()
        .map(|ch| {
            f64::from(metric.get_width(ch).expect("각 참조 문자의 원본 전진"))
                / f64::from(metric.em_size)
        })
        .sum::<f64>()
        * m.base_font_size
        * 0.75;
    let following:Vec<_>=all.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextRun(r)if r.para_index==Some(472)&&r.text.starts_with("이 공무상 재해"))).collect();
    assert_eq!(following.len(), 1);
    assert!(
        following[0].bbox.x - marker.bbox.x >= natural - 1.0 / 75.0,
        "같은 원본 face의 자연 paint 칸을 뒤 본문이 침범한다: 실제 {} / 원본 프로그램 {}",
        following[0].bbox.x - marker.bbox.x,
        natural
    );
    assert!(
        marker.bbox.y <= following[0].bbox.y + following[0].bbox.height,
        "같은 줄의 참조 소속을 유지한다"
    );
}
