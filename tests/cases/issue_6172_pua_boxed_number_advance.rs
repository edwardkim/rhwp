//! #6172: 네모 숫자는 원본 글자 크기의 전각 전진폭을 사용한다.
//! 같은 소유 줄의 `200□-□□□` 내용과 순서를 보존하고 이웃 런과 겹치지 않는다.
//! 한컴 Print 대조: stage651/652, source8d8a2c415a, Native/fresh WASM 96.70382%.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const BOXED_RUN: &str = "\u{F02B2}\u{F02B0}\u{F02B0}";

fn owner_lines<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextLine { .. })
        && node.children.iter().any(|child| {
            matches!(&child.node_type,
            RenderNodeType::TextRun(run) if run.text == BOXED_RUN)
        })
    {
        out.push(node);
    }
    for child in &node.children {
        owner_lines(child, out);
    }
}

#[test]
fn issue_6172_boxed_pua_number_advances_like_white_square() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6172/2599643_port_call_form.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("정식 입력")).expect("문서 열기");
    let page = core.build_page_render_tree(0).expect("입항 횟수의 소유 쪽");
    let mut owners = Vec::new();
    owner_lines(&page.root, &mut owners);
    assert_eq!(owners.len(), 1, "네모 숫자의 소유 줄은 하나다");
    let runs: Vec<_> = owners[0]
        .children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                Some((node, run))
            } else {
                None
            }
        })
        .collect();
    let content: String = runs.iter().map(|(_, run)| run.text.as_str()).collect();
    let expected = format!("{BOXED_RUN}□-□□□");
    assert_eq!(
        content.matches(&expected).count(),
        1,
        "번호와 뒤 빈칸을 누락·중복하지 않는다"
    );
    let index = runs
        .iter()
        .position(|(_, run)| run.text == BOXED_RUN)
        .expect("네모 숫자");
    let (boxed, run) = runs[index];
    let advance = boxed.bbox.width / run.text.chars().count() as f64;
    let em = run.style.font_size;
    assert!(em > 0.0 && em.is_finite());
    let tolerance = em * 0.02;
    assert!(
        (advance - em).abs() <= tolerance,
        "원본 글자 크기의 전각 전진폭을 보존한다"
    );
    let following = runs.get(index + 1).expect("뒤 빈칸 런").0;
    assert!(
        following.bbox.x >= boxed.bbox.x + boxed.bbox.width - tolerance,
        "글꼴별 런이 나뉘어도 뒤 빈칸은 숫자 프레임과 겹치지 않는다"
    );
    for pair in runs[index..].windows(2) {
        assert!(
            pair[0].0.bbox.x + pair[0].0.bbox.width <= pair[1].0.bbox.x + tolerance,
            "소유 줄의 뒤 내용은 같은 순서로 겹침 없이 전진한다"
        );
    }
}
