//! 관계형: 구형 HFT 수식의 본체·아래첨자 기준선과 자식 프레임 포함 관계를 보호한다.
//! 주석 갱신: 2026-11-07T02:49:39+09:00.
//! 현재 raw(Native/fresh WASM): 44쪽 97.97440/98.01130%,
//! 129쪽 94.76447/94.83414%, 130쪽 95.98723/95.92003% (192DPI, PDFium, 2px).
//! 45쪽은 98.94138/98.95601%지만 Q_n 식의 가로 배치 차이가 남아 완료 페이지로 판정하지 않는다.
//! 미완료 쪽에는 절대 px 좌표를 고정하지 않고 아래첨자 글줄의 소유 관계를 검사한다.
//! 독립 한컴2024 Print와 6/9/12/18pt 대조의 기준선 차이는 약 0.28em이며,
//! 현재 자식 font_size·ascent에서 잘못 유도한 0.032em과 구분한다.
//! 근거: mydocs/pr/assets/issue4138/legacy-subscript-frame-20261107/README.md.

use rhwp::{
    renderer::{
        equation::layout::{LayoutBox, LayoutKind},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn text(value: &LayoutBox) -> String {
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => run.glyphs.clone(),
        LayoutKind::Row(children) => children.iter().map(text).collect(),
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => text(body),
        _ => String::new(),
    }
}

fn subscript<'a>(value: &'a LayoutBox, base_text: &str, sub_text: &str) -> Option<&'a LayoutBox> {
    match &value.kind {
        LayoutKind::Subscript { base, sub } | LayoutKind::SubSup { base, sub, .. }
            if text(base) == base_text && text(sub) == sub_text =>
        {
            Some(value)
        }
        LayoutKind::Row(children) => children
            .iter()
            .find_map(|child| subscript(child, base_text, sub_text)),
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
            subscript(body, base_text, sub_text)
        }
        _ => None,
    }
}

fn equation<'a>(node: &'a RenderNode, prefix: &str) -> Option<&'a LayoutBox> {
    if let RenderNodeType::Equation(value) = &node.node_type {
        if value.script.starts_with(prefix) {
            return Some(&value.layout_box);
        }
    }
    node.children
        .iter()
        .find_map(|child| equation(child, prefix))
}

fn check(page: u32, prefix: &str, base_text: &str, sub_text: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(page - 1).unwrap();
    let root_box = equation(&tree.root, prefix).expect("독립 Print와 같은 쪽의 실제 수식");
    let value = subscript(root_box, base_text, sub_text).expect("같은 본체와 아래첨자의 소유");
    let (base, sub) = match &value.kind {
        LayoutKind::Subscript { base, sub } | LayoutKind::SubSup { base, sub, .. } => (base, sub),
        _ => unreachable!(),
    };
    let em = (sub.y + sub.baseline - base.y - base.baseline) / base.font_size;
    assert!(
        (0.265..=0.295).contains(&em),
        "독립 원본 글줄의 아래첨자 기준선: {em}em"
    );
    assert!((sub.font_size / base.font_size - 0.64).abs() < 1e-8);
    assert!(value.height + 1e-8 >= base.y + base.height);
    assert!(
        value.height + 1e-8 >= sub.y + sub.height,
        "실제 자식 끝을 같은 측정 프레임이 포함한다"
    );
}

#[test]
fn beam_formula_keeps_its_subscript_on_the_original_legacy_line() {
    check(44, "d_B =", "d", "B");
}

#[test]
fn collision_formula_keeps_the_complete_subscript_word_on_its_line() {
    check(129, "g_call", "g", "call");
}

#[test]
fn length_factor_keeps_its_subscript_inside_the_same_measured_frame() {
    check(130, "C_L =", "C", "L");
}

#[test]
fn flow_formula_keeps_the_subscript_line_with_its_measured_parent() {
    check(45, "Q_n", "Q", "n");
}
