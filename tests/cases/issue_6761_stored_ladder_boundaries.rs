//! [#6761 · #7345 · #7351] 1480000-201900042 88·89쪽의 저장 사다리 경계 계약.
//!
//! ## 입력과 독립 기준
//!
//! - 입력: `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp`
//!   (HWP5 한컴 2010 저장본, 원본 그대로 — 저장 줄 정보를 손대지 않았다),
//!   `samples/issue2004_cell_image_stack.hwp`(#7351).
//! - 기준: 저장소 추적 한컴 PDF `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf`
//!   (103쪽, sha256 `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79`)와
//!   `pdf/issue2004_cell_image_stack-hwp-2020.pdf`.
//!
//! ## 계약
//!
//! 1. 빈 그림 호스트(문단 4.200)의 저장 프레임은 **vpos 스냅을 마친** 실제 커서로 대조한다.
//!    스냅 전 커서에는 앞 문단들의 sb·trailing_ls drift(21.6px)가 남아 프레임이 기각되고,
//!    그림 아래에 호스트 빈 글줄 25.6px 를 더 쌓아 `<그림 4-5>` 캡션이 89쪽으로 밀렸다.
//!    정본 88쪽은 그 캡션으로 끝나고 89쪽은 문단 4.203 으로 시작한다.
//! 2. 글자 있는 캡션(문단 4.205)이 빈 그림 호스트(4.204)를 바로 이을 때도 세 독립 기록 —
//!    그림 하단 = 캡션 저장 줄, 캡션 끝 + 다음 앞 간격 = 다음 저장 줄, 스냅된 실제 커서 =
//!    그림 앵커 — 이 모두 맞으면 저장 프레임을 따른다. #7048 의 반례(실제 커서 증거 없이
//!    기하적으로만 닿는 글자 후속 줄)는 그대로 기각된다.
//! 3. [#7345] 쪽 머리 문단의 첫 vpos 가 자기 앞 간격이고 다음 문단도 자기 앞 간격만큼
//!    전진하면, 배치의 쪽 저장 기준점은 쪽 원점이다(HWPX #7406 과 같은 판정).
//! 4. [#7351] 표 조각(PartialTable) 항목의 스냅은 host 앞 간격을 사전 차감하지 않는다.
//! 5. 한 줄의 글자처럼 그림이 높이가 다르면 낮은 그림은 가장 높은 상자가 정한 기준선에
//!    85/15 로 앉는다(저장 줄 bl = 0.85 × lh).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
/// 저장 단위의 4 HU 격자 양자화만 허용한다. 절대 픽셀 좌표 허용치가 아니다.
fn source_grid_slack() -> f64 {
    hwpunit_to_px(4, 96.0)
}

fn saved_blank_pitch(core: &DocumentCore, para: usize) -> f64 {
    let line = &core.document().sections[4].paragraphs[para].line_segs[0];
    hwpunit_to_px(line.line_height + line.line_spacing, 96.0)
}

fn core() -> DocumentCore {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("issue6782 원본이 저장소에 있어야 한다");
    DocumentCore::from_bytes(&bytes).expect("문서 파싱")
}

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

/// 구역 4 문단 `para` 의 첫 글자 줄(TextRun) 노드.
fn run_of<'a>(nodes: &[&'a RenderNode], para: usize, needle: &str) -> Option<&'a RenderNode> {
    nodes.iter().copied().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.section_index == Some(4) && run.para_index == Some(para) && run.text.contains(needle))
    })
}

fn image_of<'a>(nodes: &[&'a RenderNode], para: usize) -> Option<&'a RenderNode> {
    nodes.iter().copied().find(|node| {
        matches!(&node.node_type, RenderNodeType::Image(image)
            if image.section_index == Some(4) && image.para_index == Some(para))
    })
}

#[test]
fn issue_6761_empty_picture_host_keeps_its_caption_on_the_picture_page() {
    let core = core();
    // 문서 전체의 잔여 차이는 #7445로 분리한다. 여기서는 그림·캡션의 쪽 소유를 검사한다.
    let mut picture_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        let Some(picture) = image_of(&nodes, 200) else {
            continue;
        };
        picture_pages.push(page);
        // 캡션은 그림과 같은 쪽, 그림 아래, 본문 하단 안에 있어야 한다(정본 물리 88쪽).
        let caption = run_of(&nodes, 202, "상 최종 표시도안(")
            .expect("<그림 4-5> 캡션은 그림과 같은 쪽에 있어야 한다(정본 88쪽)");
        let picture_bottom = picture.bbox.y + picture.bbox.height;
        assert!(
            caption.bbox.y + source_grid_slack() >= picture_bottom,
            "캡션이 그림과 겹친다: picture {:?} / caption {:?}",
            picture.bbox,
            caption.bbox
        );
        assert!(
            caption.bbox.y + caption.bbox.height <= {
                let page = &core.document().sections[4].section_def.page_def;
                hwpunit_to_px(
                    (page.height - page.margin_bottom - page.margin_footer) as i32,
                    96.0,
                ) + source_grid_slack()
            },
            "캡션이 본문 하단을 넘는다: {:?}",
            caption.bbox
        );
        // 그림 하단과 캡션 사이에 호스트의 빈 글줄(25.6px)을 끼우지 않는다.
        // 저장 사다리: 그림 하단 → 빈 후속 줄 4.0px + 줄간격 1.2px → 캡션.
        assert!(
            ((caption.bbox.y - picture_bottom) - saved_blank_pitch(&core, 201)).abs() <= source_grid_slack(),
            "그림 아래에 빈 글줄이 한 번 더 쌓였다: picture bottom {picture_bottom:.1}, caption {:?}",
            caption.bbox
        );
        // 다음 문단(203)은 저장 되감김(vpos 600)이 가리키는 다음 쪽 머리에 있다.
        assert!(
            run_of(&nodes, 203, "표시면의").is_none(),
            "문단 203 은 다음 쪽이어야 한다(정본 89쪽 머리)"
        );
        let next = core.build_page_render_tree(page + 1).unwrap();
        let mut next_nodes = Vec::new();
        collect(&next.root, &mut next_nodes);
        assert!(
            run_of(&next_nodes, 202, "상 최종 표시도안(").is_none(),
            "캡션이 다음 쪽으로 밀려났다"
        );
        let first_text = next_nodes.iter().find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.section_index == Some(4) && !run.text.trim().is_empty() =>
            {
                Some(run.para_index)
            }
            _ => None,
        });
        assert_eq!(first_text, Some(Some(203)), "다음 쪽 첫 본문은 문단 203");
    }
    assert_eq!(
        picture_pages.len(),
        1,
        "문단 200 그림은 한 쪽에만 있어야 한다"
    );
}

/// 조건을 만족하는 쪽의 render tree 들.
fn pages_where<F>(core: &DocumentCore, pred: F) -> Vec<RenderNode>
where
    F: Fn(&[&RenderNode]) -> bool,
{
    let mut found = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        if pred(&nodes) {
            found.push(tree.root.clone());
        }
    }
    found
}

fn line_top(nodes: &[&RenderNode], section: usize, para: usize) -> Option<f64> {
    nodes.iter().find_map(|node| match &node.node_type {
        RenderNodeType::TextLine(line)
            if line.section_index == Some(section) && line.para_index == Some(para) =>
        {
            Some(node.bbox.y)
        }
        _ => None,
    })
}

/// [#6761] 글자 있는 캡션이 빈 그림 호스트를 바로 잇는 형상(정본 89쪽 `<그림 4-6>`).
/// 저장 사다리가 세 기록으로 같은 경계를 말한다: 그림 하단 = 후속 저장 줄, 캡션 끝 + 다음
/// 앞 간격 = 다음 저장 줄, 스냅된 실제 커서 = 그림 앵커. 정본은 캡션 글자를 그림 하단에
/// 붙인다(그림 4-6 하단 ≈486.7px · 캡션 글자 상단 486.65px). 수정 전에는 호스트 빈 글줄
/// 25.6px 를 그림 아래에 한 번 더 쌓았다. 뒤 그림(문단 209)은 앞 그림 흐름이 저장 프레임으로
/// 닫혀야 같은 계약을 받는다(#7048 의 "앞 소제목을 덮지 않는다" 조건 유지).
#[test]
fn issue_6761_text_caption_successor_follows_stored_picture_frame() {
    let core = core();
    let pages = pages_where(&core, |nodes| image_of(nodes, 204).is_some());
    assert_eq!(pages.len(), 1, "문단 204 그림은 한 쪽에만 있어야 한다");
    let mut nodes = Vec::new();
    collect(&pages[0], &mut nodes);
    let picture = image_of(&nodes, 204).unwrap();
    let caption = run_of(&nodes, 205, "상 최종 표시도안(").expect("<그림 4-6> 캡션");
    let gap = caption.bbox.y - (picture.bbox.y + picture.bbox.height);
    assert!(
        gap.abs() <= source_grid_slack(),
        "캡션은 그림 하단 바로 아래(저장 프레임)여야 한다: gap {gap:.1}"
    );
    let second = image_of(&nodes, 209).expect("문단 209 그림");
    let heading = run_of(&nodes, 208, "미만인 제품").expect("앞 소제목");
    assert!(
        second.bbox.y >= heading.bbox.y + heading.bbox.height,
        "그림이 앞 소제목을 덮으면 안 된다"
    );
    let second_caption = run_of(&nodes, 211, "상 최종 표시도안(").expect("<그림 4-7> 캡션");
    let second_gap = second_caption.bbox.y - (second.bbox.y + second.bbox.height);
    assert!(
        (second_gap - saved_blank_pitch(&core, 210)).abs() <= source_grid_slack(),
        "<그림 4-7> 캡션은 빈 후속 줄 하나 뒤여야 한다: gap {second_gap:.1}"
    );
}

/// [#7345] 쪽 머리 문단이 앞 간격을 유지한 저장 증거(첫 vpos = 앞 간격, 다음 문단도 자기 앞
/// 간격만큼 전진)가 있으면 쪽 기준점은 쪽 원점이다. 첫 줄과 다음 문단이 같은 저장 축에
/// 놓여야 한다(정본 88쪽: 문단 4.194 → 4.195 저장 간격 6060HU = 80.8px).
#[test]
fn issue_7345_page_top_spacing_keeps_following_items_on_the_stored_axis() {
    let core = core();
    let pages = pages_where(&core, |nodes| image_of(nodes, 200).is_some());
    assert_eq!(pages.len(), 1);
    let mut nodes = Vec::new();
    collect(&pages[0], &mut nodes);
    let first = line_top(&nodes, 4, 194).expect("쪽 머리 문단 194");
    let next = line_top(&nodes, 4, 195).expect("문단 195");
    assert!(
        {
            let ps = &core.document().sections[4].paragraphs;
            let saved_delta = ps[195].line_segs[0].vertical_pos - ps[194].line_segs[0].vertical_pos;
            ((next - first) - hwpunit_to_px(saved_delta, 96.0)).abs() <= source_grid_slack()
        },
        "쪽 머리 앞 간격만큼 뒤 항목이 위로 갔다: {:.2}",
        next - first
    );
}

/// [#7351] 표 조각(PartialTable) 항목의 저장 vpos 스냅은 host 문단 앞 간격을 사전 차감하면
/// 안 된다 — 조각은 그 앞 간격을 그리는 문단 경로를 타지 않는다. `samples/issue2004_cell_image_stack.hwp`
/// 4쪽은 쪽 머리 문단(41, 저장 vpos 500 = 앞 간격)의 증거로 쪽 기준점이 쪽 원점이 되면(#7345)
/// 이 스냅이 수용되는데, 차감하면 표가 6.67px 위(121.1)로 간다. 한컴 2020 정본
/// `pdf/issue2004_cell_image_stack-hwp-2020.pdf` 4쪽 표 위 괘선 127.84px.
#[test]
fn issue_7351_table_fragment_snap_keeps_host_spacing() {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue2004_cell_image_stack.hwp"),
    )
    .expect("issue2004 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 파싱");
    let tree = core.build_page_render_tree(3).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(t) if t.para_index == Some(42) && t.cell_context.is_none() => {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .expect("4쪽 문단 42 표 조각");
    let previous = nodes
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TextLine(line) if line.para_index == Some(41))
        })
        .expect("표 앞 문단의 소유 글줄");
    let paragraphs = &core.document().sections[0].paragraphs;
    let saved_spacing = paragraphs[41].line_segs.last().unwrap().line_spacing;
    let Control::Table(source_table) = &paragraphs[42].controls[0] else {
        panic!("원본 문단 42 표")
    };
    let source_gap = hwpunit_to_px(saved_spacing + source_table.common.margin.top as i32, 96.0);
    assert!(
        ((table - previous.bbox.y - previous.bbox.height) - source_gap).abs()
            <= source_grid_slack(),
        "표 조각이 앞 문단의 저장 줄간격·표 바깥여백 관계를 잃었다"
    );
}

/// [#6761] 한 줄에 높이가 다른 글자처럼 그림 둘(정본 88쪽 문단 4.195: 138.1px · 149.4px).
/// 저장 줄의 기준선은 가장 큰 개체 높이의 0.85 배(lh 11206 · bl 9525)이고, 한/글은 낮은
/// 그림도 그 기준선에 85/15 로 앉힌다 — 정본 그림 위 끝 226.7 / 216.9px(차 9.8).
#[test]
fn issue_6761_tac_pictures_share_the_object_baseline() {
    let core = core();
    let pages = pages_where(&core, |nodes| image_of(nodes, 195).is_some());
    assert_eq!(pages.len(), 1);
    let mut nodes = Vec::new();
    collect(&pages[0], &mut nodes);
    let pictures: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(image)
                if image.section_index == Some(4) && image.para_index == Some(195))
        })
        .collect();
    assert_eq!(pictures.len(), 2);
    let (low, tall) = if pictures[0].bbox.height < pictures[1].bbox.height {
        (pictures[0], pictures[1])
    } else {
        (pictures[1], pictures[0])
    };
    let stored_line = &core.document().sections[4].paragraphs[195].line_segs[0];
    let baseline_fraction = stored_line.baseline_distance as f64 / stored_line.line_height as f64;
    let expected = (tall.bbox.height - low.bbox.height) * baseline_fraction;
    assert!(
        ((low.bbox.y - tall.bbox.y) - expected).abs() <= source_grid_slack(),
        "낮은 그림이 기준선에 앉지 않았다: {:.2} (기대 {expected:.2})",
        low.bbox.y - tall.bbox.y
    );
}
