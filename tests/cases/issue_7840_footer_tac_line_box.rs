//! 저장 줄(LINE_SEG)이 없는 꼬리말 문단도 글자처럼 취급한 그림을 줄 상자에 담는다.
//!
//! 156465345 꼬리말은 BOTTOM 정렬(밴드 3401 HU)에 PARA_LINE_SEG 가 없고, 글자처럼 취급한
//! 로고 그림 셋(3685/3685/3430 HU)과 공백 58·3·2 로 이루어진다. 독립 기준은 같은 입력의
//! 한컴 Print PDF(engine 2020, 9쪽)다. 한컴은 로고 셋을 한 줄에 놓고 밴드 위끝에서 그린다.
//! - 줄 상자는 로고까지 품어야 한다(종전: 글자 12px 줄에 실려 BOTTOM 정렬이 33.3px 아래로 밀었다).
//! - 셋째 로고 끝은 48112 HU 로 48190 상자 안이고, 넘치는 것은 뒤 공백뿐이다(종전: 둘째 줄로 내렸다).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7840/156465345_footer_tac_logos.hwp";

fn open() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

/// 꼬리말 안 줄마다 (줄 상자, 그 줄의 그림 상자들).
fn footer_lines(
    core: &DocumentCore,
    page: u32,
) -> (BoundingBox, Vec<(BoundingBox, Vec<BoundingBox>)>) {
    let tree = core.build_page_render_tree(page).expect("쪽 배치");
    let footer = tree
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Footer))
        .expect("꼬리말 영역");
    fn images(node: &RenderNode, out: &mut Vec<BoundingBox>) {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            out.push(node.bbox);
        }
        for child in &node.children {
            images(child, out);
        }
    }
    let lines = footer
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .map(|line| {
            let mut found = Vec::new();
            images(line, &mut found);
            (line.bbox, found)
        })
        .filter(|(_, found)| !found.is_empty())
        .collect();
    (footer.bbox, lines)
}

#[test]
fn footer_without_lineseg_holds_tac_logos_in_one_line_box() {
    let core = open();
    assert_eq!(core.page_count(), 9, "한컴 PDF 쪽수");
    for page in 0..core.page_count() {
        let (footer, lines) = footer_lines(&core, page);
        assert_eq!(
            lines.len(),
            1,
            "{}쪽: 로고 셋은 한 줄이다(넘치는 것은 셋째 로고 뒤 공백뿐)",
            page + 1
        );
        let (line, logos) = &lines[0];
        assert_eq!(logos.len(), 3, "{}쪽: 로고 셋 보존", page + 1);
        for logo in logos {
            assert!(
                logo.y >= line.y - 0.01 && logo.y + logo.height <= line.y + line.height + 0.01,
                "{}쪽: 줄 상자 {line:?} 가 글자처럼 취급한 로고 {logo:?} 를 품지 않는다",
                page + 1
            );
            assert!(
                logo.y >= footer.y - 0.01,
                "{}쪽: 로고가 꼬리말 영역 위로 나갔다",
                page + 1
            );
        }
        assert!(
            logos
                .windows(2)
                .all(|pair| pair[0].x + pair[0].width <= pair[1].x + 0.01),
            "{}쪽: 로고 순서·겹침",
            page + 1
        );
    }
}

/// 8쪽은 Native/fresh WASM 2px 관용 실루엣 100%/100% 이고 확대 판독을 마친 쪽이다.
/// 한컴 PDF 8쪽을 384dpi 로 래스터해 잰 로고 잉크 좌표(96dpi 환산)를 ±2px 로 고정한다.
#[test]
fn footer_logos_match_hancom_pdf_on_completed_page() {
    let core = open();
    let (_, lines) = footer_lines(&core, 7);
    let logos = &lines[0].1;
    // (로고, 잉크 왼끝 x, 잉크 위끝 y) — OPEN 1020.25, 해외감염병 1019.25 / x 537.25, 1339 x 632.5
    let expected = [
        (0, 76.25, Some(1_020.25)),
        (1, 537.25, Some(1_019.25)),
        (2, 632.5, None),
    ];
    for (index, x, y) in expected {
        let logo = logos[index];
        assert!(
            (logo.x - x).abs() <= 2.0,
            "로고 {index} x {} 가 한컴 {x} 와 2px 넘게 다르다",
            logo.x
        );
        if let Some(y) = y {
            assert!(
                (logo.y - y).abs() <= 2.0,
                "로고 {index} y {} 가 한컴 {y} 와 2px 넘게 다르다",
                logo.y
            );
        }
    }
}
