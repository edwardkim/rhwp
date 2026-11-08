//! 독립 한컴 Print와 대조한 왼쪽 그림·오른쪽 본문의 저장 줄 소유 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}

fn visible(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn verify(sample: &str) {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples")
            .join(sample),
    )
    .expect("시각 검증한 원본");
    let source = rhwp::parser::parse_document(&bytes).expect("원본 저장 줄 파싱");
    let paragraphs = &source.sections[0].paragraphs;
    let (paragraph_index, paragraph) = paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("하드디스크는 하나 이상의"))
        .expect("그림 오른쪽 본문");
    assert!(!paragraph.line_segs.is_empty());
    let mut core = DocumentCore::from_bytes(&bytes).expect("동일 원본 배치");
    core.set_hangul2024_compat(sample.contains("v2024"));
    assert_eq!(
        core.page_count(),
        64,
        "독립 Print의 전체 쪽수와 대응 내용 유지"
    );
    let page = core.build_page_render_tree(15).expect("독립 Print 16쪽");
    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes);
    let rows: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(
                &node.node_type, RenderNodeType::TextLine(line)
                    if line.section_index == Some(0) && line.para_index == Some(paragraph_index)
            )
        })
        .collect();
    assert_eq!(
        rows.len(),
        paragraph.line_segs.len(),
        "그림 옆 원본 줄을 전폭으로 다시 나누면 안 된다"
    );
    let picture = nodes.iter().copied().find(|node| matches!(
        &node.node_type, RenderNodeType::Image(image)
            if image.section_index == Some(0) && image.para_index == Some(paragraph_index - 1)
    )).expect("같은 쪽의 실제 어울림 그림");
    assert!(rows
        .iter()
        .any(|row| row.bbox.y < picture.bbox.y + picture.bbox.height
            && picture.bbox.y < row.bbox.y + row.bbox.height));
    let mut emitted = String::new();
    let mut prior_start = 0;
    for (index, row) in rows.iter().enumerate() {
        assert!(
            row.bbox.x >= picture.bbox.x + picture.bbox.width,
            "그림의 실제 오른쪽에 본문 레인이 있어야 한다"
        );
        if index > 0 {
            assert!(rows[index - 1].bbox.y < row.bbox.y, "원본 순서의 줄 배치");
        }
        for child in &row.children {
            let RenderNodeType::TextRun(run) = &child.node_type else {
                continue;
            };
            assert_eq!(run.section_index, Some(0));
            assert_eq!(run.para_index, Some(paragraph_index));
            let start = run.char_start.expect("원본 문자 소유");
            assert!(start >= prior_start, "글꼴별 런 분할 후에도 원문 순서 유지");
            prior_start = start + run.text.chars().count();
            assert!(
                child.bbox.x >= row.bbox.x
                    && child.bbox.x + child.bbox.width <= page.root.bbox.width,
                "런을 좁은 원점에 놓고 전폭 전진량을 남기면 내용이 페이지 밖으로 나간다"
            );
            emitted.push_str(&run.text);
        }
    }
    assert_eq!(
        visible(&emitted),
        visible(&paragraph.text),
        "본문 누락·중복 방지"
    );
    let following = core
        .extract_page_text_native(15)
        .expect("같은 쪽의 후속 내용");
    assert!(
        following.contains("Figure 4-1"),
        "본문 아래 그림 설명도 보존한다"
    );
}

#[test]
fn print_verified_right_lane_preserves_saved_rows_and_source_ownership() {
    for sample in [
        "hwp3-sample5-hwp5.hwp",
        "hwp3-sample5-hwp5-v2018.hwp",
        "hwp3-sample5-hwp5-v2024.hwp",
    ] {
        verify(sample);
    }
}
