//! 임시 저장 원점이 물리 쪽 경계로 승격되어 본문 꼬리를 이월하지 않도록 한다.
//! 전쪽 Native/fresh WASM 90% 및 41쪽 양쪽 98% 선행 증거와 독립 PDF의
//! 좌표는 mydocs/working/task_m100_7642_stage3508.md에 기록한다.
//! 실제 공급 이름 경로의 재검증은 #7445 stage34에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const TAIL: &str = "듯하니 불쌍코 가련하다";

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.display_or_text().to_owned();
    }
    node.children.iter().map(text).collect()
}

fn contained(inner: &BoundingBox, outer: &BoundingBox) {
    let roundoff = f64::EPSILON * outer.width.max(outer.height).max(1.0) * 16.0;
    assert!(inner.x + roundoff >= outer.x);
    assert!(inner.y + roundoff >= outer.y);
    assert!(inner.x + inner.width <= outer.x + outer.width + roundoff);
    assert!(inner.y + inner.height <= outer.y + outer.height + roundoff);
}

fn collect<'a>(
    node: &'a RenderNode,
    owner: (usize, usize),
    body: Option<BoundingBox>,
    lines: &mut Vec<&'a RenderNode>,
    tails: &mut Vec<&'a RenderNode>,
) {
    if !node.visible || node.editor_only {
        return;
    }
    let body = if matches!(node.node_type, RenderNodeType::Body { .. }) {
        Some(node.bbox)
    } else {
        body
    };
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if (line.section_index, line.para_index) == (Some(owner.0), Some(owner.1)) {
            contained(&node.bbox, &body.expect("본문 줄의 소유 영역"));
            lines.push(node);
        }
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if (run.section_index, run.para_index) == (Some(owner.0), Some(owner.1))
            && run.display_or_text().starts_with(TAIL)
        {
            contained(&node.bbox, &body.expect("마지막 줄의 본문 소유"));
            tails.push(node);
        }
    }
    for child in &node.children {
        collect(child, owner, body, lines, tails);
    }
}

#[test]
fn provisional_source_origin_keeps_fitting_tail_on_its_print_page() {
    for (extension, compat2024, supplied_programs) in [
        ("hwp", false, false),
        ("hwpx", true, false),
        ("hwp", false, true),
        ("hwpx", true, true),
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(format!("[2027] 온새미로 1 본교재.{extension}"));
        let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
        core.set_hangul2024_compat(compat2024);
        if supplied_programs {
            // Native 호스트가 실제 공급하는 것과 같은 전체 이름/name6 요청을
            // 결정적인 호출자 snapshot으로 전달한다. 설치 유무에 기대지 않는다.
            let mut programs = Vec::new();
            for prefix in ["KoPub", "KoPubWorld"] {
                for family in ["Batang", "Dotum"] {
                    for weight in ["Light", "Medium", "Bold"] {
                        programs.push(serde_json::json!({
                            "query": format!("{prefix}{family}{weight}"),
                            "family": format!("{prefix}{family} {weight}"),
                            "programId": format!("supplied-{prefix}-{family}-{weight}"),
                            "trueType": true
                        }));
                    }
                }
            }
            for family in ["HCR Batang", "Haansoft Batang"] {
                programs.push(serde_json::json!({
                    "query": family, "family": family,
                    "programId": format!("supplied-{family}"), "trueType": true
                }));
            }
            let environment =
                rhwp::renderer::font_environment::FontEnvironment::from_available_programs(
                    &core.document().doc_info,
                    &serde_json::to_string(&programs).unwrap(),
                )
                .unwrap();
            core.set_font_environment(environment).unwrap();
        }
        let owners: Vec<_> = core
            .document()
            .sections
            .iter()
            .enumerate()
            .flat_map(|(si, section)| {
                section
                    .paragraphs
                    .iter()
                    .enumerate()
                    .filter_map(move |(pi, p)| p.text.contains(TAIL).then_some((si, pi)))
            })
            .collect();
        assert_eq!(owners.len(), 1, "원본 꼬리 문단은 하나다");
        let owner = owners[0];
        let source = &core.document().sections[owner.0].paragraphs[owner.1];
        assert_eq!(core.page_count(), 46, "독립 Print 전쪽 수와 내용 소유");
        let mut actual_text = String::new();
        let mut line_indices = Vec::new();
        let mut tail_positions = Vec::new();
        let mut previous_bottom = None;
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let (mut lines, mut tails) = (Vec::new(), Vec::new());
            collect(&tree.root, owner, None, &mut lines, &mut tails);
            for node in lines {
                assert_eq!(page + 1, 41, "독립 Print의 네 줄은 같은 본문 쪽 소유다");
                if let Some(bottom) = previous_bottom {
                    assert!(node.bbox.y >= bottom, "문단 줄 순서·겹침 방지");
                }
                previous_bottom = Some(node.bbox.y + node.bbox.height);
                let RenderNodeType::TextLine(line) = &node.node_type else {
                    unreachable!()
                };
                line_indices.push(line.line_index.unwrap() as usize);
                actual_text.push_str(&text(node));
            }
            for node in tails {
                let RenderNodeType::TextRun(run) = &node.node_type else {
                    unreachable!()
                };
                tail_positions.push((page + 1, node.bbox.x, node.bbox.y + run.baseline));
            }
        }
        assert_eq!(
            line_indices,
            (0..source.line_segs.len()).collect::<Vec<_>>()
        );
        assert_eq!(
            normalized(&actual_text),
            normalized(&source.text),
            "내용 누락·중복 방지"
        );
        assert_eq!(tail_positions.len(), 1, "꼬리 내용은 한 번만 출력한다");
        let (page, x, baseline) = tail_positions[0];
        assert_eq!(page, 41);
        // 독립 PDF의 유일한 전체 꼬리 구절에서 측정. 96dpi CSS px, 왼쪽 위 원점.
        assert!((x - 119.200_002_034_505_2).abs() <= 2.0);
        assert!((baseline - 997.439_941_406_25).abs() <= 2.0);
    }
}
