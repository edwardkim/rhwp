//! 자동 쪽번호는 아래쪽 여백 선과 최종 글자 기준선을 공유한다.
//! 원본 및 여백 한 속성만 바꾼 생성 대조군의 한컴 PDF를 독립 기준으로 쓴다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn open(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

fn number_baseline(core: &DocumentCore, page: u32) -> f64 {
    let tree = core.build_page_render_tree(page).expect("쪽번호 실제 배치");
    let footer = tree
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Footer))
        .expect("꼬리말 영역");
    fn find(node: &RenderNode) -> Option<f64> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.para_index.is_none()
                && run.text.contains('-')
                && run.text.chars().any(|ch| ch.is_ascii_digit())
            {
                return Some(node.bbox.y + run.baseline);
            }
        }
        node.children.iter().find_map(find)
    }
    find(footer).expect("자동 쪽번호의 최종 기준선")
}

fn assert_original(sample: &str) {
    let core = open(sample);
    assert_eq!(core.page_count(), 3, "원본의 쪽 소유 유지");
    for page in 0..3 {
        let actual = number_baseline(&core, page);
        // 같은 입력의 한컴 PDF 세 쪽에서 관측한 기준선(96dpi)이다.
        assert!(
            (actual - 1_080.419_270_833_333_3).abs() <= 0.6,
            "{sample} {}쪽: 실제 쪽번호 기준선{actual}, 독립 PDF1080.41927",
            page + 1
        );
    }
}

#[test]
fn native_page_number_uses_bottom_margin_line() {
    assert_original("samples/issue_1133.hwp");
}

#[test]
fn hwpx_page_number_uses_bottom_margin_line() {
    assert_original("samples/hwpx/issue_1133.hwpx");
}

#[test]
fn page_number_follows_bottom_margin_but_not_footer_margin() {
    let original = open("samples/hwpx/issue_1133.hwpx");
    let footer = open("samples/issue1133/page-number-footer15.hwpx");
    let bottom = open("samples/issue1133/page-number-bottom15.hwpx");
    for page in 0..3 {
        let base = number_baseline(&original, page);
        let footer_y = number_baseline(&footer, page);
        let bottom_y = number_baseline(&bottom, page);
        // 한컴에서 꼬리말 여백 변경은0px, 아래쪽 여백 변경은−18.859375px다.
        assert!(
            (footer_y - base).abs() <= 0.15,
            "꼬리말 여백을 바꿔 쪽번호가 움직임: {base} → {footer_y}"
        );
        assert!(
            (bottom_y - base + 18.859_375).abs() <= 0.15,
            "아래쪽 여백에 따른 쪽번호 이동이 다름: {base} → {bottom_y}"
        );
        assert!(
            (bottom_y - 1_061.559_895_833_333_3).abs() <= 0.6,
            "대조군의 실제 기준선{bottom_y}, 독립 PDF1061.55990"
        );
    }
}

#[test]
fn page_number_style_control_keeps_independent_baseline() {
    let core = open("samples/issue7336/nested_table_fragment_cut.hwp");
    let actual = number_baseline(&core, 0);
    // 독립 한컴2020 PDF1쪽의 HCRDotum 기준선이다. 기존5쪽 공차는 변경하지 않는다.
    assert!(
        (actual - 1_061.559_895_833_333_3).abs() <= 0.6,
        "기존 쪽번호 스타일 대조군의 실제 기준선{actual}, 독립 PDF1061.55990"
    );
}

#[test]
fn footnote_page_number_keeps_the_same_bottom_margin_anchor() {
    let core = open("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    let tree = core.build_page_render_tree(24).expect("각주가 있는 25쪽");
    assert!(
        tree.root
            .children
            .iter()
            .any(|node| matches!(node.node_type, RenderNodeType::FootnoteArea)),
        "각주 없는 경로를 각주 대조군으로 대신하지 않음"
    );
    let actual = number_baseline(&core, 24);
    // 같은 원본의 독립 한컴 PDF25쪽, HCRDotum 쪽번호 기준선이다.
    assert!(
        (actual - 1_061.719_726_562_5).abs() <= 0.6,
        "각주 쪽의 실제 쪽번호 기준선{actual}, 독립 PDF1061.71973"
    );
}

#[test]
fn zero_footer_margin_uses_the_empty_bottom_band_center() {
    let core = open("samples/hwp3-sample5-hwp5.hwp");
    assert_eq!(core.page_count(), 64);
    for page in [4, 5] {
        let actual = number_baseline(&core, page);
        // 꼬리말 여백0인 같은 원본의 한컴2022 PDF5·6쪽 기준선이다.
        // PDF/실제 글자 기준선의 잔여 약1.8px를 포함해 96dpi 두 픽셀로 비교한다.
        assert!(
            (actual - 1_091.519_938_151_041_7).abs() <= 2.0,
            "꼬리말 밴드가 없는 {}쪽: 실제{actual}, 독립 PDF1091.519938",
            page + 1
        );
    }
}

#[test]
fn zero_footer_margin_number_does_not_move_into_release_body() {
    let core = open("samples/issue6575/156489219_satellite_pm_release.hwp");
    assert_eq!(core.page_count(), 8);
    for page in [5, 7] {
        let actual = number_baseline(&core, page);
        // 같은 입력의 한컴2020 PDF6·8쪽 자동 쪽번호 기준선이다.
        assert!(
            (actual - 1_082.079_996_744_791_7).abs() <= 2.0,
            "꼬리말 밴드가 없는 {}쪽: 실제{actual}, 독립 PDF1082.079997",
            page + 1
        );
    }
}
