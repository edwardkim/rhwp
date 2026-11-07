//! 관계형: 글상자의 현재 글줄·안 여백, 고정 크기와 아래 정렬의 소유를 보호한다.
//! 주석 갱신: 2026-11-06T23:29:08+09:00.
//! 2026-11-06 raw(96DPI, PDFium, 2px): 원본·목록 선언·실사용 선언 각1쪽,
//! 고정 크기·빈 글줄·공백 대조군 각1쪽 모두 Native/fresh WASM 100.00000%.
//! 전쪽 최저도 각100.00000%; 부모4b03792e56에 글상자 최소 높이 보정을 적용한 source다.
//! 설치 글꼴 없이 검사하며 독립 Print 좌표는 별도 고정형 검사로 분리한다.
//! 근거: mydocs/pr/assets/issue7375/paper-textbox-minimum-20261106/README.md.

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.display_or_text().to_owned();
    }
    node.children.iter().map(text).collect()
}

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

fn check(input: &str, protected: bool) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(root.join(input)).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let mut source = None;
    for section in &core.document().sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                if let Control::Footer(footer) = control {
                    for paragraph in &footer.paragraphs {
                        for control in &paragraph.controls {
                            if let Control::Shape(shape) = control {
                                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref())
                                {
                                    assert!(source.is_none(), "표본의 독립 글상자 소유");
                                    source = Some((
                                        tb.margin_top,
                                        tb.margin_bottom,
                                        shape.common().height,
                                    ));
                                    assert_eq!(shape.common().size_protect, protected);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let (top, bottom, declared_height) = source.expect("꼬리말의 원본 글상자");
    let before = core.export_hwpx_native().unwrap();
    let page = core.build_page_render_tree(0).unwrap();
    let shape = find(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Rectangle(_))
            && find(node, &|child| {
                matches!(child.node_type, RenderNodeType::TextBox)
            })
            .is_some()
    })
    .expect("꼬리말 글상자 사각형");
    let tb = find(shape, &|node| {
        matches!(node.node_type, RenderNodeType::TextBox)
    })
    .unwrap();
    let line = find(tb, &|node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
    .unwrap();
    let px = |value: f64| value * 96.0 / 7200.0;
    if protected {
        assert!(
            (shape.bbox.height - px(f64::from(declared_height))).abs() < 0.01,
            "크기 고정은 넘친 내용으로 선언 높이를 늘리지 않는다"
        );
    } else {
        assert!(
            line.bbox.y >= shape.bbox.y + px(f64::from(top)) - 0.01,
            "안쪽 위 여백의 소유"
        );
        assert!(
            line.bbox.y + line.bbox.height + px(f64::from(bottom))
                <= shape.bbox.y + shape.bbox.height + 0.01,
            "빈 글줄을 포함한 실제 글줄과 두 여백이 물리 상자 안에 들어간다"
        );
    }
    let footer = find(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Table(_)) && text(node) == "FOOTER C"
    })
    .expect("같은 아래 기준을 쓰는 꼬리말 표");
    assert!(
        ((shape.bbox.y + shape.bbox.height) - (footer.bbox.y + footer.bbox.height)).abs() < 0.02,
        "내용 하한으로 높이를 늘려도 꼬리말 틀의 아랫변은 보존한다"
    );
    assert_eq!(
        core.export_hwpx_native().unwrap(),
        before,
        "그리기용 최소 상자로 원본 선언·저장 글줄을 변경하지 않는다"
    );
}

#[test]
fn paper_textbox_contains_its_current_line_and_both_margins() {
    check("samples/header-footer-paper-frame.hwpx", false);
}

#[test]
fn unused_list_font_keeps_the_same_textbox_contract() {
    check("pdf/issue8687/font_list_name_fallback.hwpx", false);
}

#[test]
fn used_list_font_keeps_the_same_textbox_contract() {
    check("pdf/issue8687/font_list_name_used_fallback.hwpx", false);
}

#[test]
fn size_protection_preserves_the_declared_box() {
    check("pdf/issue7375/paper-textbox-protected.hwpx", true);
}

#[test]
fn empty_published_line_still_owns_its_font_strut_and_margins() {
    check("pdf/issue7375/paper-textbox-empty.hwpx", false);
}

#[test]
fn whitespace_published_line_still_owns_its_font_strut_and_margins() {
    check("pdf/issue7375/paper-textbox-space.hwpx", false);
}
