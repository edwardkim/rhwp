//! [Issue #6895] HWPX 채움 그림의 `bright`/`contrast` 가 저장할 때마다 뒤바뀌지 않는다.
//!
//! 공통 `ImageFill` 의 두 필드는 **이진 HWP5 `FILL_INFO` 의 저장 순서**를 담는다. 그 순서는
//! HWPX 속성 이름과 반대다 — 한/글 오라클로 확인했다(같은 문서를 한/글이 HWPX→HWP5→HWPX 로
//! 저장하면 `bright`/`contrast` 가 보존되고, 그 중간 HWP5 의 이진 1번 바이트는 화면
//! `contrast` 다).
//!
//! 그래서 읽을 때 정규화하고(`parser/hwpx/header.rs` · `section.rs`) 쓸 때 되돌려야 한다
//! (`serializer/hwpx/shape.rs`). 종전에는 쓰는 쪽이 되돌리지 않아 **HWPX 를 열어 다시 저장할
//! 때마다 두 값이 뒤집혔다**(두 번 저장하면 원상 복귀).

#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::PathBuf;

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

fn sample() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("samples/HWP5-nopassword-123456.hwpx")
}

fn header_xml(hwpx: &[u8]) -> String {
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX 는 zip 이어야 한다");
    let mut file = zip
        .by_name("Contents/header.xml")
        .expect("Contents/header.xml");
    let mut xml = String::new();
    file.read_to_string(&mut xml).expect("header.xml 읽기");
    xml
}

/// `header.xml` 의 `<hc:img …>` 에서 `(bright, contrast)` 를 문서 순서대로 모은다.
fn img_bright_contrast(xml: &str) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for chunk in xml.split("<hc:img ").skip(1) {
        let tag = match chunk.find('>') {
            Some(end) => &chunk[..end],
            None => continue,
        };
        let attr = |name: &str| -> Option<i32> {
            let needle = format!("{name}=\"");
            let start = tag.find(&needle)? + needle.len();
            let rest = &tag[start..];
            let end = rest.find('"')?;
            rest[..end].parse().ok()
        };
        if let (Some(bright), Some(contrast)) = (attr("bright"), attr("contrast")) {
            out.push((bright, contrast));
        }
    }
    out
}

/// 쪽 배경 그림(`hh:borderFill` 의 `hc:imgBrush`)이 왕복에서 그대로 남는다.
#[test]
fn issue6895_page_background_img_fill_survives_hwpx_roundtrip() {
    let original = std::fs::read(sample()).expect("표본 읽기");
    let before = img_bright_contrast(&header_xml(&original));
    assert_eq!(
        before,
        vec![(50, -15)],
        "표본의 쪽 배경은 bright=50 contrast=-15 이다"
    );

    let doc = parse_hwpx(&original).expect("HWPX 파싱");
    let saved = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let after = img_bright_contrast(&header_xml(&saved));

    assert_eq!(
        after, before,
        "HWPX 왕복에서 채움 그림의 bright/contrast 가 바뀌었다"
    );

    // 두 번 저장해도 같다 — 뒤집힘은 저장마다 부호가 도는 형태였다.
    let doc2 = parse_hwpx(&saved).expect("HWPX 재파싱");
    let saved2 = serialize_hwpx(&doc2).expect("HWPX 재직렬화");
    assert_eq!(
        img_bright_contrast(&header_xml(&saved2)),
        before,
        "두 번째 저장에서 다시 뒤집혔다"
    );
}

/// 모델이 담는 순서는 **이진 HWP5** 순서다 — 화면·HWPX 로 나갈 때만 되돌린다.
///
/// 이 계약이 깨지면 HWP5↔HWPX 변환이 비대칭이 된다(HWP5 이진 파서는 순서 그대로 읽는다).
#[test]
fn issue6895_image_fill_keeps_binary_storage_order() {
    let original = std::fs::read(sample()).expect("표본 읽기");
    let doc = parse_hwpx(&original).expect("HWPX 파싱");

    let fills: Vec<_> = doc
        .doc_info
        .border_fills
        .iter()
        .filter_map(|bf| bf.fill.image.as_ref())
        .filter(|img| (img.brightness, img.contrast) != (0, 0))
        .collect();
    assert!(
        !fills.is_empty(),
        "표본에는 bright/contrast 가 0 이 아닌 채움 그림이 있어야 한다"
    );

    for img in fills {
        assert_eq!(
            img.display_brightness_contrast(),
            (50, -15),
            "화면 순서는 (bright, contrast) = (50, -15)"
        );
        assert_eq!(img.brightness, -15, "이진 1번 바이트 = 화면 contrast");
        assert_eq!(img.contrast, 50, "이진 2번 바이트 = 화면 bright");
    }
}
