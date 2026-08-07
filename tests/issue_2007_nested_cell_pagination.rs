//! Issue #2007: 1×1 단일 셀 중첩 표의 셀 콘텐츠 페이지 분할(intra-cell pagination).
//!
//! `samples/basic/issue2007_nested_cell_pagination_42065.hwp` (규제영향분석서)는
//! 1×1 RowBreak 표(자리차지) 안에 중첩 1×1 표가 있고, 그 중첩 셀에 135+문단(약 8164px,
//! 8쪽 분량)이 담긴다.
//!
//! 회귀 (수정 전 버그, rhwp 6p vs 한글 17p):
//! - per-중첩행 유닛 분해(`cell_units`)는 중첩 표 `row_count >= 2` 에만 적용 →
//!   1×1(단일 행) 중첩 표는 atomic 유닛 1개로 취급 → 8164px 콘텐츠가 한 페이지에 통째
//!   배치(오버플로/크램) → under-pagination.
//!
//! 정정: 1×1 중첩 표의 셀 콘텐츠가 한 페이지를 명백히 초과(>1000px)하면 기존
//! `nested_table_mixed_fragment_heights`(텍스트+중첩표 문단에 쓰던 페이지 분할 fragment)
//! 를 빈-텍스트 문단에도 적용해 splittable 유닛으로 분해 → 페이지 경계로 분할.
//! 한컴 2020 PDF = 17페이지. #4069의 완료 계약은 중첩 표를 하위 행·셀
//! 흐름까지 분할해 빠짐·중복 없이 17페이지에 정확히 수렴하는 것이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn page_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        page_text(child, out);
    }
}

fn normalized_page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("render tree p{}: {error:?}", page + 1));
    let mut text = String::new();
    page_text(&tree.root, &mut text);
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn terminal_bottom_lines_with_cell_clips(
    node: &RenderNode,
    clip_ancestors: &mut Vec<BoundingBox>,
    found: &mut Vec<(BoundingBox, Vec<BoundingBox>)>,
) {
    let pushes_clip = matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.clip);
    if pushes_clip {
        clip_ancestors.push(node.bbox);
    }

    if matches!(node.node_type, RenderNodeType::Line(_))
        && node.bbox.y > 820.0
        && node.bbox.width > 500.0
        && node.bbox.height <= 2.0
    {
        found.push((node.bbox, clip_ancestors.clone()));
    }
    for child in &node.children {
        terminal_bottom_lines_with_cell_clips(child, clip_ancestors, found);
    }

    if pushes_clip {
        clip_ancestors.pop();
    }
}

fn svg_number_attr(tag: &str, name: &str) -> f64 {
    let marker = format!("{name}=\"");
    let value = tag
        .split_once(&marker)
        .and_then(|(_, tail)| tail.split_once('"'))
        .map(|(value, _)| value)
        .unwrap_or_else(|| panic!("SVG attribute {name} missing: {tag}"));
    value
        .parse::<f64>()
        .unwrap_or_else(|error| panic!("SVG attribute {name}={value}: {error}"))
}

#[test]
fn issue_2007_nested_cell_content_paginates() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // 한컴 2020 기준 PDF는 17페이지다. `>= 12`는 24페이지 공백 회귀와
    // 23페이지 중복 회귀를 모두 통과시켜 #4069를 보호하지 못했다.
    let pages = doc.page_count();
    assert_eq!(
        pages, 17,
        "#4069 중첩 흐름 분할 회귀 — 페이지 수 {pages} (한컴 2020 기준 17)"
    );
}

#[test]
fn issue_2007_nested_cell_cursor_has_no_boundary_duplication() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page2 = normalized_page_text(&core, 1);
    let page3 = normalized_page_text(&core, 2);

    const FIRST_ITEM: &str = "1.출석요구및진술청취또는진술서제출요구";
    const SECOND_ITEM: &str = "2.신고사항과관련이있다고인정되는자료등의제출요구";
    assert!(
        page2.contains(FIRST_ITEM),
        "2쪽에 조문 대비표 제1호가 없다 — 첫 child cursor 누락"
    );
    assert!(
        !page2.contains(SECOND_ITEM),
        "3쪽 소속 조문 대비표 제2호가 2쪽에 미리 노출됐다 — 비종료 clip 회귀"
    );
    assert!(
        !page3.contains(FIRST_ITEM),
        "3쪽에 조문 대비표 제1호가 반복됐다 — continuation cursor 중복"
    );
    assert!(
        page3.contains(SECOND_ITEM),
        "3쪽에 조문 대비표 제2호가 없다 — continuation cursor 누락"
    );
    assert!(
        page3.contains("④제1항부터제3항까지"),
        "3쪽에 조문 대비표 마지막 개정 조항이 없다 — terminal cursor 누락"
    );
}

#[test]
fn issue_2007_intra_paragraph_saved_frame_break_is_preserved() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page10 = normalized_page_text(&core, 9);
    let page11 = normalized_page_text(&core, 10);

    const FRAME_START: &str = "제50조의2(조사권의남용금지)";
    const FRAME_CONTINUATION: &str = "행하여야하며,다른목적등을위하여조사권을남용하여서는아니된다.";
    const NEXT_ARTICLE: &str = "제50조의4(이행강제금등)";

    assert!(
        page10.contains(FRAME_START),
        "10쪽에 저장 프레임 말미 조항이 없다"
    );
    assert!(
        !page10.contains(FRAME_CONTINUATION),
        "10쪽에 다음 저장 프레임이 겹쳤다 — 문단 내부 vpos reset 소실"
    );
    assert!(
        page11.contains(FRAME_CONTINUATION),
        "11쪽에 문단 내부 vpos reset 이후 줄이 없다"
    );
    assert!(
        page11.contains(NEXT_ARTICLE),
        "11쪽에 후속 조항이 없다 — child cursor 누락"
    );
}

#[test]
fn issue_2007_saved_frame_tail_nested_table_starts_before_next_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page15 = normalized_page_text(&core, 14);
    let page16 = normalized_page_text(&core, 15);

    const NESTED_TABLE_START: &str = "조달사업에관한법률";
    const NESTED_TABLE_TAIL: &str = "제4항에따라시정요구를받은계약상대자";
    const NEXT_FRAME: &str = "<이해관계자협의>:입법예고‧기관협의중";

    assert!(
        page15.contains(NESTED_TABLE_START),
        "15쪽 조달청 제목 뒤 자식 표가 다음 쪽으로 통째로 밀렸다"
    );
    assert!(
        page15.contains(NESTED_TABLE_TAIL),
        "15쪽 저장 프레임 말미까지 조달청 자식 표가 이어지지 않았다"
    );
    assert!(
        !page15.contains(NEXT_FRAME),
        "다음 저장 프레임의 이해관계자 협의 제목이 15쪽에 흡수됐다"
    );
    assert!(
        page16.contains(NEXT_FRAME),
        "16쪽이 이해관계자 협의 저장 프레임에서 재개하지 않았다"
    );
}

#[test]
fn issue_4159_terminal_nested_bottom_border_is_inside_all_cell_clips() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let preceding_tree = core
        .build_page_render_tree(1)
        .expect("render physical page 2");
    let mut premature = Vec::new();
    terminal_bottom_lines_with_cell_clips(&preceding_tree.root, &mut Vec::new(), &mut premature);
    assert!(
        premature.is_empty(),
        "비종료 물리 2쪽에 종료 bottom 선이 미리 노출됐다: {premature:?}"
    );

    let tree = core
        .build_page_render_tree(2)
        .expect("render physical page 3");

    let mut found = Vec::new();
    terminal_bottom_lines_with_cell_clips(&tree.root, &mut Vec::new(), &mut found);
    assert_eq!(
        found.len(),
        1,
        "물리 3쪽의 폭 500px 이상 종료 bottom 선을 하나만 찾아야 한다: {found:?}"
    );

    let (line, clips) = &found[0];
    assert!(
        !clips.is_empty(),
        "종료 nested bottom 선에 clip=true TableCell 조상이 없다"
    );
    let line_bottom = line.y + line.height;
    for clip in clips {
        let clip_bottom = clip.y + clip.height;
        assert!(
            clip_bottom + 0.01 >= line_bottom,
            "종료 nested bottom stroke가 조상 셀 clip에 잘린다: line_bottom={line_bottom:.3}, clip_bottom={clip_bottom:.3}, line={line:?}, clip={clip:?}"
        );
    }
}

#[test]
fn issue_4159_svg_terminal_bottom_border_is_visible_inside_outer_cell_clip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("fixture parse");
    let svg = doc
        .render_page_svg_native(2)
        .expect("render physical page 3 SVG");

    let outer_clip = svg
        .lines()
        .filter(|line| line.contains("<clipPath id=\"cell-clip"))
        .find(|line| {
            let x = svg_number_attr(line, "x");
            let width = svg_number_attr(line, "width");
            x < 80.0 && width > 650.0
        })
        .expect("physical page 3 outer split cell clip");
    let bottom_line = svg
        .lines()
        .filter(|line| line.starts_with("<line "))
        .find(|line| {
            let x1 = svg_number_attr(line, "x1");
            let x2 = svg_number_attr(line, "x2");
            let y1 = svg_number_attr(line, "y1");
            let y2 = svg_number_attr(line, "y2");
            y1 > 820.0 && (y1 - y2).abs() < 0.01 && x2 - x1 > 500.0
        })
        .expect("physical page 3 terminal nested bottom SVG line");

    let clip_bottom = svg_number_attr(outer_clip, "y") + svg_number_attr(outer_clip, "height");
    let line_bottom =
        svg_number_attr(bottom_line, "y1") + svg_number_attr(bottom_line, "stroke-width");
    assert!(
        clip_bottom + 0.01 >= line_bottom,
        "SVG bottom stroke가 outer cell clip에 잘린다: line_bottom={line_bottom:.3}, clip_bottom={clip_bottom:.3}\n{outer_clip}\n{bottom_line}"
    );
}
