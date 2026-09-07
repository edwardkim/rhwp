//! [Issue #6840] `extract-pages` 가 참조 없는 `BinData` 를 하나도 버리지 않아, 쪽을 한
//! 장만 남겨도 파일이 거의 줄지 않던 결함의 가드.
//!
//! ## 왜 중요한가
//!
//! 이 명령이 내건 용도는 **발췌·부분 제출**과 결함 이분법이다. 그림이 많은 문서는 본문을
//! 88% 걷어내도 파일이 20% 밖에 줄지 않았고, **버리려던 쪽의 그림 원본이 산출물에 그대로
//! 남았다** — 부분 제출에서는 유출 경로다.
//!
//! ```text
//!   104쪽 6.5MB 문서 → --from 77 --to 77
//!     수정 전   BinData 204개 그대로   파일 5,225,984
//!     수정 후   BinData  36개          파일 1,364,992
//! ```
//!
//! ## 무엇을 잠그나 — 뒤바뀜이 진짜 위험이다
//!
//! `ImageAttr.bin_data_id` 는 **1 기준 순번(위치)** 이라 항목을 지우면 뒤 참조가 한 칸씩
//! 밀려 **그림이 조용히 뒤바뀐다.** 개수만 보는 시험은 그걸 못 잡으므로, 남은 그림의
//! **좌표와 이미지 바이트**를 정리 전후로 대조한다(`render_page_svg_native` 의 data URI).
//!
//! ⚠ 내장 글꼴은 순번이 아니라 **storage id** 로 참조한다
//! (`load_bounded_embedded_font_bytes` 가 `content.id == font_id` 로 찾는다). 그 축을
//! 빠뜨리면 발췌본에서 글꼴이 통째로 사라지므로 음성 통제군으로 함께 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rhwp::DocumentCore;

/// 43개 중 7개만 실제로 쓰는 문서 — 칸 안 다문단 float 그림.
const PICTURES: &str = "samples/issue5833/cell_multi_para_float_pics.hwp";
/// 그림은 없고 `BinData` 가 **내장 글꼴 하나**뿐인 문서.
const EMBEDDED_FONT: &str = "samples/render-p35-font-native-bitmap.hwpx";
/// 34개를 모두 쓰는 문서 — 버릴 것이 없으면 건드리지 않아야 한다.
const ALL_REFERENCED: &str = "samples/basic/Worldcup_FIFA2010_32.hwp";

fn load(rel: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#6840 공개 fixture 읽기 {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("{rel} 파싱: {error:?}"))
}

/// 1쪽 SVG 의 `<image>` 를 `(x, y, 바이트 해시)` 로 모은다.
///
/// data URI 를 통째로 비교하지 않고 해시로 줄인다 — 실패 메시지에 수십 KB 가 실리면
/// 무엇이 달라졌는지 오히려 안 보인다.
fn page_images(core: &DocumentCore) -> Vec<(String, String, u64)> {
    let svg = core.render_page_svg_native(0).expect("1쪽 SVG");
    let mut out = Vec::new();
    for chunk in svg.split("<image ").skip(1) {
        let attr = |name: &str| -> String {
            chunk
                .split_once(&format!("{name}=\""))
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(v, _)| v.to_string())
                .unwrap_or_default()
        };
        let href = attr("href");
        let mut hasher = DefaultHasher::new();
        href.hash(&mut hasher);
        out.push((attr("x"), attr("y"), hasher.finish()));
    }
    out.sort();
    out
}

/// 양성 계약 — 참조 없는 항목은 버리고, 남은 그림은 **한 장도 바뀌지 않는다.**
#[test]
fn unreferenced_bin_data_is_dropped_without_swapping_pictures() {
    let mut core = load(PICTURES);
    let before = page_images(&core);
    assert_eq!(
        before.len(),
        7,
        "표본이 어긋났다 — 1쪽에 그림 7장이 있어야 한다"
    );

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (43, 7),
        "참조 없는 36개가 버려져야 한다 — 회귀 시 43개가 그대로 남아 파일이 줄지 않는다"
    );
    assert_eq!(
        page_images(&core),
        before,
        "정리 뒤 그림의 좌표 또는 바이트가 달라졌다 — 순번 재번호매김이 어긋나면 \
         그림이 조용히 뒤바뀐다"
    );
}

/// 음성 통제군 ① — 내장 글꼴은 **순번 참조가 없어도** 남아야 한다.
///
/// 이 문서는 그림이 0장이고 `BinData` 가 글꼴 하나(`font-native-smoke.ttf`)뿐이다.
/// 순번만 보고 버리면 여기서 `1 → 0` 이 되어 발췌본의 글꼴이 사라진다.
#[test]
fn embedded_font_bin_data_survives_even_with_no_picture_reference() {
    let mut core = load(EMBEDDED_FONT);
    assert!(
        page_images(&core).is_empty(),
        "표본이 어긋났다 — 이 문서에는 그림이 없어야 글꼴 축만 남는다"
    );

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (1, 1),
        "내장 글꼴이 쓰는 항목을 버렸다 — storage id 축을 빠뜨리면 글꼴이 사라진다"
    );
}

/// 음성 통제군 ② — 모두 참조되는 문서는 건드리지 않는다.
#[test]
fn fully_referenced_bin_data_is_left_alone() {
    let mut core = load(ALL_REFERENCED);
    let before = page_images(&core);

    let report = core.extract_page_range(1, 1).expect("1쪽 추출");

    assert_eq!(
        (report.bin_data_before, report.bin_data_after),
        (34, 34),
        "버릴 것이 없는데 항목이 줄었다"
    );
    assert_eq!(page_images(&core), before, "그림이 바뀌었다");
}
