//! 단 첫 여백 원본의 쪽 테두리 중심·저장 간격·Studio 쪽 정보 관계.
//! 같은 원본의 독립 2024 Print와 전64쪽 Native/fresh WASM 최저 91.99347%.
//! 이중선의 잉크 끝을 중심 좌표로 고정한 과거 절대 px 검사를 교체한다.
use rhwp::document_core::DocumentCore;
use rhwp::model::page::PageBorderBasis;
use rhwp::renderer::render_tree::RenderNodeType;

#[test]
fn page_border_centers_preserve_source_gaps_and_match_the_public_page_information() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample16-hwp5.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let section = &core.document().sections[0].section_def;
    assert_eq!(section.page_border_fill.basis, PageBorderBasis::BodyBased);
    let fill: serde_json::Value =
        serde_json::from_str(&core.get_page_border_fill_native(0).unwrap()).unwrap();
    assert_eq!(fill["basis"], "page", "원본 UI 기준 보존");
    assert_eq!(
        fill["spacingTop"].as_i64(),
        Some(i64::from(section.page_border_fill.spacing_top))
    );
    let source_gap =
        f64::from(section.page_border_fill.spacing_top) / f64::from(section.page_def.height);
    let source_footer_gap =
        f64::from(section.page_border_fill.spacing_bottom) / f64::from(section.page_def.height);
    let tree = core.build_page_render_tree(0).unwrap();
    let bodies: Vec<_> = tree
        .root
        .children
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .collect();
    assert_eq!(bodies.len(), 1, "한 쪽 본문 영역 소유·누락·중복");
    let body = bodies[0].bbox;
    let mut horizontal: Vec<_> = tree
        .root
        .children
        .iter()
        .filter_map(|n| {
            if let RenderNodeType::Line(line) = &n.node_type {
                (line.y1 == line.y2 && line.x1 < line.x2).then_some(line)
            } else {
                None
            }
        })
        .collect();
    horizontal.sort_by(|a, b| a.y1.total_cmp(&b.y1));
    assert_eq!(horizontal.len(), 4, "위·아래 이중 테두리 선의 보존");
    let top = (horizontal[0].y1 + horizontal[1].y1) / 2.0;
    let bottom = (horizontal[2].y1 + horizontal[3].y1) / 2.0;
    assert!(
        top < body.y && bottom > body.y + body.height,
        "본문 바깥에 같은 저장 간격을 소비한 선 묶음이 있어야 함"
    );
    let equal_ratio = |actual: f64, expected: f64| {
        assert!(
            (actual - expected).abs() <= f64::EPSILON * 128.0,
            "선 묶음 중심이 본문과 원본 간격의 같은 축을 소비해야 함: {actual}/{expected}"
        );
    };
    equal_ratio((body.y - top) / tree.root.bbox.height, source_gap);
    equal_ratio(
        (bottom - body.y - body.height) / tree.root.bbox.height,
        source_footer_gap,
    );
    let ui: serde_json::Value =
        serde_json::from_str(&core.get_page_info_native(0).unwrap()).unwrap();
    // API는 소수 한 자리로 직렬화한다. 기대 화면 좌표·px 허용치를 고정하지 않는다.
    // 실제 선 묶음 중심을 동일한 공개 직렬화 계약으로 표시해야 한다.
    assert_eq!(
        format!("{:.1}", ui["pageBorderTop"].as_f64().unwrap()),
        format!("{top:.1}")
    );
    let bottom_gap = tree.root.bbox.height - bottom;
    assert_eq!(
        format!("{:.1}", ui["pageBorderBottom"].as_f64().unwrap()),
        format!("{bottom_gap:.1}")
    );
}
