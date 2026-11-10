//! 관계형: 참조 번호의 공통 paint 전진 칸·뒤 본문 소속과 본문 자간의 비간섭을 검사한다.
//! 주석 갱신: 2026-11-10T10:07:17+09:00.
//! report-628 95쪽의 최소 글꼴 Native97.64481% / fresh WASM97.64831% (96dpi/PDFium/raw2px).
//! 원본 글꼴 대조99.99111% 및 독립 Print Gulim 공급 대조99.99487%와 최소 조건을 섞지 않는다.
//! 독립 Print는 표시 Gulim의 자연 전진에 TJ 보정을 적용한다. 대체 face의 비양자화 폭을
//! 고정하지 않고, 실제 출력에서 확인한 공통 전진 배열·칸·다음 본문의 비겹침을 검사한다.
//! 설치 글꼴을 요구하지 않으며 절대 px를 고정하지 않는다.

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

/// 검사 유형: 관계형. 모든 paint 소비자에게 전달한 원본 전진 칸에 뒤 본문이 침범하지 않는다.
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
    let run = m.source_run.as_deref().expect("공통 참조 paint 전진");
    assert_eq!(run.text, m.text);
    assert_eq!(run.section_index, Some(m.section_index));
    assert_eq!(run.para_index, Some(m.para_index));
    assert_eq!(
        run.char_start, None,
        "생성 세 글자를 본문 세 글자로 중복 소유하지 않는다"
    );
    let positions = run.layout_positions.as_ref().expect("확정된 글리프 전진");
    assert_eq!(positions.len(), m.text.chars().count() + 1);
    assert_eq!(positions.first(), Some(&0.0));
    assert!(positions.windows(2).all(|pair| pair[1] >= pair[0]));
    let paint_end = *positions.last().unwrap();
    assert!(
        (marker.bbox.width - paint_end).abs() < 1.0 / 75.0,
        "측정 칸과 모든 소비자에게 전달한 paint 칸이 다르다"
    );
    let following:Vec<_>=all.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextRun(r)if r.para_index==Some(472)&&r.text.starts_with("이 공무상 재해"))).collect();
    assert_eq!(following.len(), 1);
    assert!(
        following[0].bbox.x - marker.bbox.x >= paint_end - 1.0 / 75.0,
        "원본 전진 배열의 paint 칸을 뒤 본문이 침범한다: 실제 {} / paint 끝 {}",
        following[0].bbox.x - marker.bbox.x,
        paint_end
    );
    assert!(
        marker.bbox.y <= following[0].bbox.y + following[0].bbox.height,
        "같은 줄의 참조 소속을 유지한다"
    );
}
