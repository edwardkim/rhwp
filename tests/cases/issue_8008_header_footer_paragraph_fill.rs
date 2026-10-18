//! [#8008] 머리말·꼬리말 문단의 테두리/배경 채우기를 그 영역 안에서 그린다.
//!
//! `samples/한글문서파일형식_5.0_revision1.3.hwp` 의 머리말 문단은 문단 모양이 선 없음 + 채움
//! `#C0C0C0` 인 테두리/배경을 가리키고 글자색이 흰색이다. 정본
//! (`pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`)은 회색 띠 (92.1, 44.1)–(701.1, 65.0)px
//! 위에 흰 글씨를 그린다. 수정 전 rhwp 는 문단 배치가 모은 배경 범위를 머리말 영역에서 그리지 않고
//! 본문 단을 시작할 때 비워, 띠가 없고 흰 글씨가 보이지 않았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::wasm_api::HwpDocument;
use rhwp::DocumentCore;
use serde_json::Value;

const SAMPLE: &str = "samples/한글문서파일형식_5.0_revision1.3.hwp";

fn bytes() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("read")
}

fn rect(node: &Value) -> (f64, f64, f64, f64) {
    let b = &node["bbox"];
    let (x, y) = (b["x"].as_f64().unwrap(), b["y"].as_f64().unwrap());
    (
        x,
        y,
        x + b["w"].as_f64().unwrap(),
        y + b["h"].as_f64().unwrap(),
    )
}

fn find_header(node: &Value) -> Option<&Value> {
    if node["type"].as_str() == Some("Header") {
        return Some(node);
    }
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(find_header)
}

#[test]
fn header_paragraph_fill_is_painted_behind_the_header_text() {
    let doc = HwpDocument::from_bytes(&bytes()).expect("표본 파싱");
    // 4쪽(차례, 정본과 쪽이 어긋나지 않는 쪽)과 19쪽(본문).
    for page in [3u32, 18] {
        let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
        let header =
            find_header(&tree).unwrap_or_else(|| panic!("{}쪽에 머리말이 있어야 한다", page + 1));
        let children = header["children"].as_array().expect("머리말 자식");
        let fill_at = children
            .iter()
            .position(|c| c["type"].as_str() == Some("Rect"))
            .unwrap_or_else(|| {
                panic!(
                    "{}쪽 머리말에 문단 배경 채움이 있어야 한다: {children:?}",
                    page + 1
                )
            });
        let line_at = children
            .iter()
            .position(|c| c["type"].as_str() == Some("TextLine"))
            .expect("머리말 글줄");
        assert!(
            fill_at < line_at,
            "{}쪽: 배경은 글줄보다 먼저(뒤에) 그린다",
            page + 1
        );
        let (fx0, fy0, fx1, fy1) = rect(&children[fill_at]);
        let (lx0, ly0, lx1, ly1) = rect(&children[line_at]);
        assert!(
            fx0 <= lx0 && fy0 <= ly0 && fx1 >= lx1 && fy1 >= ly1,
            "{}쪽: 배경 ({fx0:.1},{fy0:.1})-({fx1:.1},{fy1:.1}) 은 머리말 글줄 ({lx0:.1},{ly0:.1})-({lx1:.1},{ly1:.1}) 을 덮어야 한다",
            page + 1
        );
    }
}

#[test]
fn header_fill_color_reaches_the_svg() {
    let core = DocumentCore::from_bytes(&bytes()).expect("표본 파싱");
    let svg = core
        .render_page_svg_layer_with_profile_native(3, RenderProfile::Print)
        .expect("SVG")
        .to_ascii_lowercase();
    assert!(
        svg.contains("#c0c0c0"),
        "4쪽 SVG 에 머리말 배경 채움 #C0C0C0 이 있어야 한다"
    );
}
