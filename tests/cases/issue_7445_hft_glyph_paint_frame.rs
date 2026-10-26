//! 독립 Print와 원본 HFT·TrueType·치환 HWP3의 좌표 계약을 대조한다.
//! C263: HWPX115+row18, Native/fresh WASM 전266쪽90% 이상.
//! C256: PII66×2환경×2backend 전264쪽90% 이상.
//! C264: sample11 원본151×2backend 전302쪽 최저90.08879/90.09923%.
//! 독립 Print의 원본 HFT 하부 글리프 원점은 논리 원점보다 위에 있다.
//! 미완료 위치를 고정 px로 굳히지 않고 paint와 source의 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{paint::RenderProfile, DocumentCore};
use serde_json::Value;

fn load(path: &str) -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(root.join(path)).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

fn texts<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    if value.get("type").and_then(Value::as_str) == Some("textRun") {
        out.push(value);
    }
    match value {
        Value::Array(values) => values.iter().for_each(|value| texts(value, out)),
        Value::Object(values) => values.values().for_each(|value| texts(value, out)),
        _ => {}
    }
}

fn selected(value: &Value, text: &str, family: &str) -> Value {
    let mut runs = vec![];
    texts(value, &mut runs);
    let selected: Vec<_> = runs
        .into_iter()
        .filter(|run| {
            run["text"].as_str().is_some_and(|s| s.contains(text))
                && run["style"]["fontFamily"]
                    .as_str()
                    .is_some_and(|s| s.contains(family))
        })
        .collect();
    assert_eq!(
        selected.len(),
        1,
        "원본 줄과 실제 표시 글꼴을 한 번 선택: {text}/{family}"
    );
    selected[0].clone()
}

fn origin(run: &Value) -> (f64, f64) {
    let placement = &run["placement"]["runToPage"];
    let x = placement["e"].as_f64().unwrap();
    let y = placement["f"].as_f64().unwrap();
    let transform = &run["paintTransform"];
    if transform.is_null() {
        return (x, y);
    }
    (
        transform["a"].as_f64().unwrap() * x
            + transform["c"].as_f64().unwrap() * y
            + transform["e"].as_f64().unwrap(),
        transform["b"].as_f64().unwrap() * x
            + transform["d"].as_f64().unwrap() * y
            + transform["f"].as_f64().unwrap(),
    )
}

fn pair(core: &DocumentCore, page: u32, text: &str, family: &str) -> (Value, Value) {
    let screen: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(page, RenderProfile::Screen)
            .unwrap(),
    )
    .unwrap();
    let print: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(page, RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let (source, paint) = (
        selected(&screen, text, family),
        selected(&print, text, family),
    );
    assert_eq!(source["text"], paint["text"], "내용 순서·누락·중복 보존");
    assert_eq!(
        source["source"]["stableSourceKey"], paint["source"]["stableSourceKey"],
        "원본 소유 보존"
    );
    assert_eq!(
        source["bbox"], paint["bbox"],
        "paint 프레임으로 논리 상자를 덮어쓰지 않는다"
    );
    (source, paint)
}

#[test]
fn original_hft_glyph_paint_has_its_print_frame_without_moving_source() {
    let core = load("samples/issue1949_giant_cell_nested_tables_perf.hwpx");
    assert_eq!(core.page_count(), 115);
    let (source, paint) = pair(&core, 14, "충분한 부력과", "한양신명조");
    assert!(paint["source"]["stableSourceKey"]
        .as_str()
        .unwrap()
        .contains("/para:335/"));
    assert!(
        origin(&paint).1 < origin(&source).1 - 1e-6,
        "독립 Print의 원본 HFT 글리프는 물리 인쇄 프레임을 소비한다: source={:?}, paint={:?}",
        origin(&source),
        origin(&paint)
    );

    // C308: 독립 HFT 대조군의 셀 BorderFill은 이후 쪽의 정밀 CTM을
    // 바꾸지 않는다. 도형 컨트롤의 polygon fill(C307), 글자 shade(C275)와
    // 구별한다. 로드맵 원본의 앞쪽 노란 셀 배경 뒤 본문도 같은 계약이다.
    // 시각 전쪽 재검증 전에는 이 보완을 완료 회귀로 커밋하지 않는다.
    let core = load("samples/issue5941/1490000-201600081_roadmap_research.hwp");
    assert_eq!(core.page_count(), 302);
    let (source, paint) = pair(&core, 61, "파견, 용역, 특수형태근로", "한양신명조");
    assert!(
        origin(&paint).1 < origin(&source).1 - 1e-6,
        "셀 배경을 도형 polygon fill로 간주해 후속 HFT 종이 프레임을 없애지 않는다: source={:?}, paint={:?}",
        origin(&source),
        origin(&paint)
    );

    // C317: 독립 Print의 42쪽 첫 표 프레임은 저장 호스트 원점과
    // 원본 offset·위 바깥여백을 함께 소유한다. 절대 px 위치 대신 같은
    // 소스 소유자의 선언과 최종 배치 사이 관계를 확인한다.
    // 이 보완도 전쪽 Native/fresh WASM 검증 전에는 커밋하지 않는다.
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn find<'a>(node: &'a RenderNode, f: &impl Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
        if f(node) {
            return Some(node);
        }
        node.children.iter().find_map(|child| find(child, f))
    }
    let (section, para_index, control_index, host, table) = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, s)| {
            s.paragraphs.iter().enumerate().flat_map(move |(pi, p)| {
                p.controls.iter().enumerate().filter_map(move |(ci, c)| {
                    let Control::Table(t) = c else { return None };
                    t.cells
                        .first()
                        .and_then(|cell| cell.paragraphs.first())
                        .is_some_and(|p| p.text.starts_with("◦ 고용형태:"))
                        .then_some((section, pi, ci, p, t.as_ref()))
                })
            })
        })
        .next()
        .expect("저장 줄과 바깥여백이 있는 원본 표");
    let is_owner = |node: &RenderNode| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.section_index == Some(section) && t.para_index == Some(para_index)
                && t.control_index == Some(control_index))
    };
    let tree = core.build_page_render_tree(41).unwrap();
    let column = find(&tree.root, &|n| {
        matches!(n.node_type, RenderNodeType::Column(_)) && find(n, &is_owner).is_some()
    })
    .unwrap();
    let table_node = find(column, &is_owner).unwrap();
    let hu_to_css = 96.0 / 7200.0;
    let source_top = column.bbox.y
        + f64::from(
            host.line_segs[0].vertical_pos
                + table.common.vertical_offset as i32
                + i32::from(table.outer_margin_top),
        ) * hu_to_css;
    assert!(
        (table_node.bbox.y - source_top).abs() <= hu_to_css,
        "확정 첫 프레임의 위 바깥여백을 한 번 보존: source={source_top}, paint={}",
        table_node.bbox.y
    );
}

#[test]
fn true_type_program_keeps_its_existing_origin() {
    let core = load("samples/task2097/75544_pii_bunseok.hwpx");
    assert_eq!(core.page_count(), 66);
    let (source, paint) = pair(&core, 3, "① ---------", "한양신명조");
    assert_eq!(
        origin(&source),
        origin(&paint),
        "명시 TrueType 원점에 HFT 프레임을 중복 적용하지 않는다"
    );
}

#[test]
fn substituted_hwp3_program_keeps_its_existing_origin() {
    let core = load("samples/hwp3-sample11.hwp");
    assert_eq!(core.page_count(), 151);
    let (source, paint) = pair(&core, 1, "SUN Product", "돋움");
    assert_eq!(
        origin(&source),
        origin(&paint),
        "치환 프로그램은 검증된 인쇄 원점을 보존한다"
    );

    // 동일 Print의 자동 대체 세션과 실제 원 프로그램 선택을 구분한다.
    // source 메트릭을 보존하는 공급의 윤곽을 자연 폭으로 승격하지 않는다.
    fn supplied_runs(node: &rhwp::paint::layer_tree::LayerNode, preserves_source: bool) -> usize {
        use rhwp::paint::{layer_tree::LayerNodeKind, paint_op::PaintOp};
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children
                .iter()
                .map(|n| supplied_runs(n, preserves_source))
                .sum(),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                supplied_runs(child, preserves_source)
            }
            LayerNodeKind::Leaf { ops } => {
                let mut count = 0;
                for op in ops {
                    if let PaintOp::TextRun { run, .. } = op {
                        if run
                            .style
                            .font_family
                            .split(',')
                            .next()
                            .unwrap_or_default()
                            .trim()
                            == "RHWP Public Batang"
                            && !run.text.is_empty()
                        {
                            assert_eq!(
                                run.style.preserve_glyph_ink, !preserves_source,
                                "자동 표시용 공급은 기존 glyph fit, 명시 원 프로그램은 자연 윤곽"
                            );
                            count += 1;
                        }
                    }
                }
                count
            }
        }
    }
    for preserves_source in [true, false] {
        let mut program = load("samples/hwp3-sample11.hwp");
        program
            .set_font_environment(Some(
                rhwp::renderer::font_environment::FontEnvironment::from_json(
                    &serde_json::json!({
                        "id": "original-or-display-only-program-contract",
                        "substitutions": {"하이텔울릉도제목체": "RHWP Public Batang"},
                        "trueTypeFaces": ["RHWP Public Batang"],
                        "preserveSourceMetrics": preserves_source
                    })
                    .to_string(),
                )
                .unwrap(),
            ))
            .unwrap();
        let paint = program
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        assert!(
            supplied_runs(&paint.root, preserves_source) > 0,
            "실제 공급 런 없이 검사 통과 금지"
        );
    }
}

/// C278: 같은 원본/독립 Print의 Native/fresh WASM 전18쪽 최저92.87548/92.86964%.
/// 9쪽은99.67453/99.64590%이며 양쪽 review+overlay를 직접 판독했다.
/// 첫 쪽 polygon fill 뒤에는 독립 Print의 device CTM이0.120/-0.120이다.
/// C275 단일 shadeColor 인과 대조에서도 그 뒤6쪽까지 같은 전환이 지속됐다.
/// 독립 PDF 9쪽 첫 '<' 원점(182.08,113.60)CSSpx을±2px로 함께 보호한다.
#[test]
fn earlier_source_fill_preserves_later_hft_print_device_frame() {
    let mut core = load("samples/rowbreak-problem-pages.hwp");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 18, "독립 한컴 Print 전쪽수 보존");
    // 앞쪽 Print를 호출하지 않고9쪽부터 요청해도 source fill 이력을 소비한다.
    let (source, paint) = pair(
        &core,
        8,
        "<클라우드컴퓨팅 발전 및 이용자 보호에 관한 법률 시행령>",
        "한양중고딕",
    );
    let frame = &paint["paintTransform"];
    let (sx, sy) = if frame.is_null() {
        (1.0, 1.0)
    } else {
        (frame["a"].as_f64().unwrap(), frame["d"].as_f64().unwrap())
    };
    assert!(
        (sx - 1.0).abs() < 1e-6 && (sy - 1.0).abs() < 1e-6,
        "독립 Print에서 fill 이후600DPI CTM은0.120이며 HFT라고 정밀 종이 축척을 재적용하지 않는다: {sx}/{sy}"
    );
    let (x, y) = origin(&paint);
    assert!((x - 182.08).abs() <= 2.0, "9쪽 독립 glyph 원점x: {x}");
    assert!((y - 113.60).abs() <= 2.0, "9쪽 독립 glyph 원점y: {y}");
    assert_eq!(
        source["source"], paint["source"],
        "정밀도 전환은 원본 줄 소속을 바꾸지 않는다"
    );
}

/// C290 전18쪽 N/W≥90%. 7쪽은98% 미만이므로 절대 px 핀 대신
/// 저장 선언·바깥여백·점유 끝과 실제 셀 첫 줄 사이의 가운데 정렬 관계를 검사한다.
#[test]
fn stored_mixed_opening_centers_the_source_frame_and_preserves_continuation() {
    use rhwp::model::control::Control;
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn find<'a>(
        node: &'a RenderNode,
        predicate: &impl Fn(&RenderNode) -> bool,
    ) -> Option<&'a RenderNode> {
        if predicate(node) {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find(child, predicate))
    }
    fn all_text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        node.children.iter().for_each(|child| all_text(child, out));
    }
    fn owned_text(
        node: &RenderNode,
        owner: &rhwp::renderer::render_tree::TextRunNode,
        out: &mut String,
    ) {
        if !node.visible || node.editor_only {
            return;
        }
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.cell_context == owner.cell_context && run.para_index == owner.para_index {
                out.push_str(&run.text);
            }
        }
        node.children
            .iter()
            .for_each(|child| owned_text(child, owner, out));
    }
    let mut core = load("samples/rowbreak-problem-pages.hwp");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 18);
    let first_text = "이용자는 클라우드컴퓨팅서비스 제공자에게 이용자 정보가 저장되는";
    let (table, cell) = core
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .find_map(|table| {
            table
                .cells
                .iter()
                .find(|cell| cell.paragraphs.iter().any(|p| p.text.contains(first_text)))
                .map(|cell| (table, cell))
        })
        .unwrap();
    let host = cell
        .paragraphs
        .iter()
        .find(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .unwrap();
    let Control::Table(child) = &host.controls[0] else {
        panic!("source child table")
    };
    let padding = cell.effective_padding(&table.padding);
    let source_end = host.line_segs.last().unwrap().vertical_pos
        + child.common.vertical_offset as i32
        + child.common.margin.top as i32
        + child.common.height as i32
        + child.common.margin.bottom as i32;
    let unused_hu = cell.height as i32 - padding.top as i32 - padding.bottom as i32 - source_end;
    assert!(
        unused_hu > 0,
        "독립 Print와 대조한 원본 첫 프레임의 정렬 여백"
    );
    let tree = core.build_page_render_tree(6).unwrap();
    let owner = find(&tree.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row==cell.row && c.col==cell.col)
            && find(node, &|n| matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text.contains(first_text))).is_some()
    }).unwrap();
    let line = find(
        owner,
        &|n| matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text.contains(first_text)),
    )
    .unwrap();
    let hu_to_css = 96.0 / 7200.0;
    let top_free = line.bbox.y - owner.bbox.y - f64::from(padding.top) * hu_to_css;
    let bottom_free = owner.bbox.height
        - f64::from(padding.top + padding.bottom) * hu_to_css
        - f64::from(source_end) * hu_to_css
        - top_free;
    assert!(
        top_free > 0.0 && (top_free - bottom_free).abs() < 1e-6,
        "source 점유 끝의 앞뒤 빈 밴드가 같은 Center: top={top_free}, bottom={bottom_free}"
    );
    let mut page7 = String::new();
    all_text(&tree.root, &mut page7);
    let mut page8 = String::new();
    all_text(&core.build_page_render_tree(7).unwrap().root, &mut page8);
    assert_eq!(
        page7.matches(first_text).count(),
        1,
        "첫 source 줄을 한 번 표시"
    );
    assert!(
        !page8.contains(first_text),
        "소비한 첫 줄을 다음 쪽에 복제하지 않는다"
    );
    assert!(
        page7.contains("통신사업자와 영리를"),
        "앞 조각의 마지막 source 줄 보존"
    );
    assert!(
        !page8.contains("통신사업자와 영리를"),
        "앞 조각의 마지막 source 줄 중복 없음"
    );
    // 같은 인용문 안의 제2호와 제3호는 '하여 정보를 제공하거나…'를
    // 반복한다. 전역 문자열 검색 대신 마지막 줄의 source 문단만 대조한다.
    let tail = find(owner, &|node| {
        matches!(&node.node_type,
        RenderNodeType::TextRun(run) if run.text.contains("통신사업자와 영리를"))
    })
    .unwrap();
    let RenderNodeType::TextRun(tail) = &tail.node_type else {
        unreachable!()
    };
    assert!(tail.cell_context.is_some(), "중첩 셀의 실제 source 경로");
    let source_paragraph = child.cells[0]
        .paragraphs
        .iter()
        .position(|p| p.text.contains("통신사업자와 영리를"))
        .unwrap();
    assert_eq!(tail.para_index, Some(source_paragraph));
    let mut owned7 = String::new();
    let mut owned8 = String::new();
    owned_text(&tree.root, tail, &mut owned7);
    owned_text(
        &core.build_page_render_tree(7).unwrap().root,
        tail,
        &mut owned8,
    );
    let continuation = "하여 정보를 제공하거나 정보의 제공을";
    assert!(
        !owned7.contains(continuation),
        "다음 조각의 첫 줄을 앞당겨 소비하지 않는다"
    );
    assert_eq!(
        owned8.matches(continuation).count(),
        1,
        "다음 조각의 첫 source 줄을 누락·복제하지 않는다"
    );
    let content = |s: &str| {
        s.chars()
            .filter(|ch| !ch.is_whitespace() && !ch.is_control())
            .collect::<String>()
    };
    assert_eq!(
        content(&(owned7 + &owned8)),
        content(&child.cells[0].paragraphs[source_paragraph].text),
        "앞 조각과 다음 조각을 합치면 같은 source 문단이 순서대로 한 번 보존된다"
    );
}

#[test]
fn primary_program_availability_keeps_the_source_print_frame() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let mut source = core.document().clone();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.name = "휴먼명조".into();
        font.alt_type = 2;
        font.native_program = None;
    }
    for shape in &mut source.doc_info.char_shapes {
        shape.bold = false;
        shape.base_size = 7500;
        shape.ratios = [100; 7];
        shape.spacings = [0; 7];
        shape.relative_sizes = [100; 7];
    }
    core.set_document(source);
    let original = core.export_hwpx_native().unwrap();
    let (_, before) = pair(&core, 0, "가", "휴먼명조");
    assert!(
        before["paintTransform"].is_object(),
        "원 HFT Print 프레임이 실제 활성인 대조군"
    );
    let snapshot = serde_json::json!([{
        "query":"휴먼명조", "family":"휴먼명조", "programId":"primary-availability-contract", "trueType":true
    }]);
    let available =
        FontEnvironment::from_available_programs(&core.document().doc_info, &snapshot.to_string())
            .unwrap()
            .expect("같은 주 프로그램의 실제 공급 선언");
    core.set_font_environment(Some(available)).unwrap();
    let (_, after) = pair(&core, 0, "가", "휴먼명조");
    assert_eq!(before["source"], after["source"]);
    assert_eq!(before["bbox"], after["bbox"], "일반 한글의 같은 논리 상자");
    assert_eq!(
        before["paintTransform"], after["paintTransform"],
        "실제 주 프로그램 인증은 원문이 소유한 물리 종이 프레임의 변경 요청이 아니다"
    );
    assert_eq!(core.export_hwpx_native().unwrap(), original);

    // 호출자가 TTF 전진·프레임을 명시적으로 선택하는 기존 override와 구분한다.
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"explicit-ttf-frame-control","substitutions":{},"trueTypeFaces":["휴먼명조"]}"#,
    ).unwrap())).unwrap();
    let (_, explicit) = pair(&core, 0, "가", "휴먼명조");
    assert!(explicit["paintTransform"].is_null());
}
