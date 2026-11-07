//! 고정형: 그림과 뒤 가사가 공유하는 첫 저장 셀 프레임의 독립 Print 그림 원점을 ±2px로 보호한다.
//! 주석 갱신: 2026-11-07 16:08:23 KST.
//! 수정 전 1쪽 raw: Native 98.59756% / fresh WASM 98.54350% 이상인 전4쪽 비교의 대상이다.
//! 수정 후 1쪽: Native 99.86093% / fresh WASM 99.83671%; 전4쪽 98% 이상.
//! 그림 세로 원점은 직접 판독했고, 별도 가사 자간 차이는 계속 보정한다.
//! 근거: mydocs/pr/assets/issue1994/print-frame-20261107/README.md.
//! 기준: pdf/semanticist21-20261008/basic-issue1994_behindtext_table_20200830-8e7a95cf5919-2020.pdf.
use rhwp::{
    model::control::Control,
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
fn picture(node: &RenderNode, id: u16) -> Option<&RenderNode> {
    if matches!(&node.node_type,RenderNodeType::Image(image)if image.bin_data_id==id) {
        return Some(node);
    }
    node.children.iter().find_map(|n| picture(n, id))
}
#[test]
fn first_picture_and_following_lyrics_keep_the_independent_print_origin() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes =
        std::fs::read(root.join("samples/basic/issue1994_behindtext_table_20200830.hwp")).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    let json =
        std::fs::read_to_string(root.join(
            "mydocs/pr/assets/issue1994/print-frame-20261107/reference-font-environment.json",
        ))
        .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&json).unwrap()))
        .unwrap();
    let Control::Table(outer) = &core.document().sections[0].paragraphs[5].controls[0] else {
        panic!("원본 바깥 표")
    };
    let parent = outer
        .cells
        .iter()
        .find(|c| c.row == 2 && c.col == 0)
        .unwrap();
    let Control::Table(table) = &parent.paragraphs[0].controls[0] else {
        panic!("원본 예배 순서 표")
    };
    let cell = table
        .cells
        .iter()
        .find(|c| c.row == 6 && c.col == 2)
        .unwrap();
    let Control::Picture(source) = &cell.paragraphs[0].controls[0] else {
        panic!("원본 악보")
    };
    let id = source.image_attr.bin_data_id;
    let page = core.build_page_render_tree(0).unwrap();
    let image = picture(&page.root, id).expect("원본 악보의 실제 출력");
    // 독립 Print p1 Im4의 실제 그림 좌표(pt)를 96DPI CSSpx로 변환한다.
    for (actual, pt, label) in [
        (image.bbox.x, 50.82400131225586, "x"),
        (image.bbox.y, 374.79901123046875, "y"),
    ] {
        let expected = pt * 96.0 / 72.0;
        assert!(
            (actual - expected).abs() <= 2.0,
            "악보 {label}: actual={actual}, independent={expected}"
        );
    }
}
