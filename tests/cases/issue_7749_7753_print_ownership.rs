//! 관계형·고정형 회귀 — 저장 줄과 표·그림·각주의 쪽 소속 및 독립 Print 전체 쪽수.
//! 주석 갱신 2026-11-21T09:21:39+09:00.
//! 직전 파일 변경 `1722836b78a343b34cd6e15423c4e4b923678c6d` (2026-11-21T07:55:25+09:00).
//! 측정 source `ec8515f1d8d52353f1e5092eb080ab2619de55e7`. 실루엣 96 DPI, 색 경계 보정 전 2px raw.
//! 아래 결과는 Native / fresh WASM 순서다.
//!
//! regulatory-2020 전체 31쪽 최저 98.70816% / 98.69136%.
//! - 14쪽 99.39991% / 99.40249%.
//! - 15쪽 99.45641% / 99.46676%.
//! - 29쪽 99.95252% / 99.95355%.
//! - 30쪽 99.86025% / 99.86250%.
//! - 31쪽 100.00000% / 99.98889%.
//!
//! parameter-2022 전체 74쪽 최저 100.00000% / 100.00000%.
//! - 1쪽 100.00000% / 100.00000%.
//! - 43쪽 100.00000% / 100.00000%.
//! - 44쪽 100.00000% / 100.00000%.
//! - 45쪽 100.00000% / 100.00000%.
//!
//! research-hwp 전체 215쪽 최저 98.96557% / 98.86514%.
//! - 24쪽 99.99490% / 99.99495%.
//! - 66쪽 99.97426% / 99.97063%.
//! - 67쪽 99.93854% / 99.94001%.
//! - 90쪽 99.37392% / 99.40817%.
//! - 91쪽 100.00000% / 100.00000%.
//!
//! research-hwpx 전체 215쪽 최저 99.00639% / 99.01378%.
//! - 11쪽 99.56904% / 99.54601%.
//! - 24쪽 99.99490% / 99.99495%.
//! - 66쪽 99.97426% / 99.97063%.
//! - 67쪽 99.93854% / 99.94001%.
//! - 90쪽 99.37392% / 99.40814%.
//! - 91쪽 100.00000% / 100.00000%.
//!
//! Print 원본 SHA·전쪽 TSV·대표 확대 증적은 `mydocs/pr/assets/issues7749_7753_20261121`에 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn visible_nodes<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    if !node.visible || node.editor_only {
        return;
    }
    nodes.push(node);
    for child in &node.children {
        visible_nodes(child, nodes);
    }
}

fn find_node(
    node: &RenderNode,
    matches: impl Fn(&RenderNode) -> bool + Copy,
) -> Option<&RenderNode> {
    if !node.visible || node.editor_only {
        return None;
    }
    if matches(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, matches))
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if !node.visible || node.editor_only {
        return;
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(run.display_or_text());
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn compact_text(node: &RenderNode) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

const REGULATORY: &[u8] =
    include_bytes!("../../samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx");

fn regulatory() -> DocumentCore {
    let mut core = DocumentCore::from_bytes(REGULATORY).expect("80550 정식 원본");
    core.set_hangul2024_compat(false);
    core
}

#[test]
fn cell_host_title_remains_on_the_page_that_consumed_its_text_line() {
    let core = regulatory();
    let title = "향후10년간폐농업용지게차해체수익계산";
    assert_eq!(core.page_count(), 31, "독립 한컴 2020 Print의 전체 쪽수");
    for (page, expected_count) in [(29, 0), (30, 1), (31, 0)] {
        let _ = core.take_overflow_cell_lines();
        let tree = core.build_page_render_tree(page - 1).expect("렌더 트리");
        let text = compact_text(&tree.root);
        assert_eq!(
            text.matches(title).count(),
            expected_count,
            "{page}쪽 제목 소속"
        );
        let carried_table = find_node(&tree.root, |node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.para_index == Some(40)
                    && table.row_count == 13
                    && table.col_count == 7
                    && table.cell_context.as_ref().is_some_and(|context|
                        context.parent_para_index == 208))
        });
        assert_eq!(carried_table.is_some(), page == 31, "{page}쪽 자식 표 소속");
        assert_eq!(
            text.contains("연간폐기처리건수(건,A)"),
            page == 31,
            "자식 표의 첫 행 소속"
        );
        assert_eq!(core.take_overflow_cell_lines(), 0, "{page}쪽 셀 초과 줄");
    }
}

#[test]
fn closing_wrapper_keeps_the_saved_empty_line_after_the_nested_table() {
    let core = regulatory();
    let source = rhwp::parse_document(REGULATORY).expect("80550 원본 프레임");
    let [Control::Table(stored)] = source.sections[0].paragraphs[111].controls.as_slice() else {
        panic!("바깥 표의 원본 호스트")
    };
    let margin = f64::from(stored.outer_margin_top) * 96.0 / 7200.0;
    for page in [14, 15] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("이어받기 렌더 트리");
        let column = find_node(&tree.root, |node| {
            matches!(node.node_type, RenderNodeType::Column(_))
        })
        .expect("본문 단");
        let wrapper = find_node(column, |node| {
            matches!(&node.node_type, RenderNodeType::Table(table)
                if table.cell_context.is_none() && table.para_index == Some(111))
        })
        .expect("이어받은 바깥 표");
        assert!(
            ((wrapper.bbox.y - column.bbox.y) - margin).abs() < 0.05,
            "{page}쪽 조각은 원본 바깥 위 여백부터 시작한다"
        );
        if page == 15 {
            let child = find_node(wrapper, |node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                    if table.cell_context.is_some() && table.row_count == 13 && table.col_count == 2)
            }).expect("이어받은 자식 표");
            let blank_line = 1400.0 * 96.0 / 7200.0;
            assert!(
                wrapper.bbox.y + wrapper.bbox.height
                    >= child.bbox.y + child.bbox.height + blank_line,
                "닫는 셀이 자식 아래에 저장 빈 줄을 담는다"
            );
        }
    }
}

#[test]
fn master_picture_and_continuing_parameter_table_keep_their_pages() {
    let source = include_bytes!("../../samples/hwpctl_ParameterSetID_Item_v1.2.hwp");
    let mut core = DocumentCore::from_bytes(source).expect("표 원본");
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 74, "한컴 2022 Print의 전체 쪽수");

    let first = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let master = find_node(&first.root, |node| {
        matches!(node.node_type, RenderNodeType::MasterPage)
    })
    .expect("바탕쪽");
    let body = find_node(&first.root, |node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 상자");
    let picture = find_node(master, |node| {
        matches!(node.node_type, RenderNodeType::Image(_)) && node.bbox.y < body.bbox.y
    })
    .expect("본문 시작을 가로지르는 바탕쪽 그림");
    assert!(
        picture.bbox.y >= first.root.bbox.y,
        "바탕쪽 그림 위가 용지에 포함된다"
    );
    assert!(picture.bbox.y + picture.bbox.height > body.bbox.y);

    let snippet = "_dset.SetItem(\"TopMargin\"";
    for (page, expected_count) in [(43, 0), (44, 1), (45, 0)] {
        let tree = core
            .build_page_render_tree(page - 1)
            .expect("표 이어받기 쪽");
        let compact = compact_text(&tree.root);
        assert_eq!(
            compact.matches(snippet).count(),
            expected_count,
            "{page}쪽 코드 소속"
        );
    }
}

fn report(extension: &str) -> DocumentCore {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).{extension}"
    ));
    let mut core = DocumentCore::from_bytes(&std::fs::read(source).expect("보고서 원본"))
        .expect("보고서 읽기");
    core.set_hangul2024_compat(true);
    core
}

#[test]
fn diagnostic_figure_and_footnote_successors_stay_in_print_order() {
    for extension in ["hwp", "hwpx"] {
        let core = report(extension);
        assert_eq!(
            core.page_count(),
            215,
            "{extension} 독립 한컴 2024 Print 쪽수"
        );

        let figures = core.build_page_render_tree(23).expect("24쪽 렌더 트리");
        let mut nodes = Vec::new();
        visible_nodes(&figures.root, &mut nodes);
        let mut images: Vec<&RenderNode> = nodes
            .iter()
            .copied()
            .filter(|node| matches!(node.node_type, RenderNodeType::Image(_)))
            .collect();
        images.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
        assert_eq!(images.len(), 2, "두 그림을 빠짐없이 한 번씩 출력한다");
        let line = |search: &str| {
            nodes
                .iter()
                .copied()
                .find(|node| {
                    if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                        return false;
                    }
                    let mut text = String::new();
                    collect_text(node, &mut text);
                    text.contains(search)
                })
                .expect(search)
        };
        let first_caption = line("그림 23.");
        let paragraph = line("EU에서 생존 장기기증이 상승하고 있으나");
        let second_caption = line("그림 24.");
        let bottom = |node: &RenderNode| node.bbox.y + node.bbox.height;
        assert!(bottom(images[0]) <= first_caption.bbox.y);
        assert!(bottom(first_caption) <= paragraph.bbox.y);
        assert!(bottom(paragraph) <= images[1].bbox.y);
        assert!(bottom(images[1]) <= second_caption.bbox.y);

        for (page, has_next_footnote) in [(90, false), (91, true)] {
            let tree = core
                .build_page_render_tree(page - 1)
                .expect("각주 이어받기 쪽");
            let mut nodes = Vec::new();
            visible_nodes(&tree.root, &mut nodes);
            let note_area = nodes
                .into_iter()
                .find(|node| matches!(node.node_type, RenderNodeType::FootnoteArea))
                .expect("각주 영역");
            let compact = compact_text(note_area);
            assert_eq!(
                compact.matches("유럽28개국과노르웨이분석").count(),
                usize::from(has_next_footnote),
                "{extension} {page}쪽 각주 소속"
            );
        }
    }
}

#[test]
fn table_and_reserved_footnotes_follow_their_print_pages() {
    for extension in ["hwp", "hwpx"] {
        let core = report(extension);
        for (page, first_fragment) in [(66, true), (67, false)] {
            let tree = core
                .build_page_render_tree(page - 1)
                .expect("표와 각주의 이어받기 쪽");
            let body = find_node(&tree.root, |node| {
                matches!(node.node_type, RenderNodeType::Body { .. })
            })
            .expect("본문 영역");
            let note = find_node(&tree.root, |node| {
                matches!(node.node_type, RenderNodeType::FootnoteArea)
            })
            .expect("각주 영역");
            let body = compact_text(body);
            let note = compact_text(note);
            for (phrase, owned) in [
                ("OrganDonationandRecoveryImprovementAct", first_fragment),
                ("StephanieTubbsJonesGiftofLifeMedalAct", !first_fragment),
            ] {
                assert_eq!(
                    body.matches(phrase).count(),
                    usize::from(owned),
                    "{extension} {page}쪽 표 행 소속 {phrase}"
                );
            }
            assert_eq!(
                note.matches("본단원은질병관리본부").count(),
                usize::from(first_fragment),
                "{extension} {page}쪽 각주 76 소속"
            );
            assert_eq!(
                note.contains("PUBLICHEALTHSERVICE"),
                !first_fragment,
                "{extension} {page}쪽 각주 78 소속"
            );
        }
    }
}
