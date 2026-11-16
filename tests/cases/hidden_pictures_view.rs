//! 한컴 보기 > 그림을 끈 편집 화면(`setShowPictures`).
//!
//! 한컴 12.30은 그림을 끄면 그림 개체 자리에 회색 파선 상자와 두 대각선만 그린다. 다시 켜면
//! 그림을 그대로 그린다. 처음 값은 켜짐이다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::wasm_api::HwpDocument;

fn doc_with_picture() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        include_bytes!("../../assets/logo/logo-16.png"),
        4000,
        4000,
        16,
        16,
        "png",
        "그림",
        None,
        None,
    )
    .unwrap();
    doc
}

#[test]
fn hidden_pictures_draw_a_placeholder_box_instead_of_the_image() {
    let mut doc = doc_with_picture();
    assert!(doc.get_show_pictures());
    let shown = doc.render_page_svg_native(0).unwrap();
    assert!(shown.contains("<image"), "그림을 그린다");

    doc.set_show_pictures(false);
    assert!(!doc.get_show_pictures());
    let hidden = doc.render_page_svg_native(0).unwrap();
    assert!(!hidden.contains("<image"), "그림 파일을 그리지 않는다");
    // 자리 상자의 네 변과 두 대각선.
    assert!(
        hidden.matches("<line").count() >= shown.matches("<line").count() + 6,
        "그림 자리에 파선 상자와 대각선을 그린다"
    );

    doc.set_show_pictures(true);
    assert_eq!(doc.render_page_svg_native(0).unwrap(), shown);
}
