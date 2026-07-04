//! Task #1562 회귀 테스트 — HWPX 폼 컨트롤 caption `&&` 표시 정합.
//!
//! #1534는 저장/roundtrip 계층에서 caption XML escape 누적 손상을 해결했다.
//! 이 테스트는 별도 표시 계층 문제를 고정한다. `samples/hwpx/form-002.hwpx`의
//! 저장값은 `R&&D`를 유지해야 하지만, 사용자에게 보이는 폼 caption은 한컴처럼
//! `R&D`로 표시되어야 한다.

use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwpx/form-002.hwpx";

fn render_form_002_page_0_svg() -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("form-002 fixture read failed {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("form-002 parse failed");
    doc.render_page_svg_native(0)
        .expect("form-002 page 0 SVG render failed")
}

#[test]
fn form_caption_double_ampersand_displays_as_single_ampersand_in_svg() {
    let svg = render_form_002_page_0_svg();

    for expected in [
        "IP R&amp;D연계",
        "R&amp;D 자율성트랙(일반)",
        "R&amp;D 자율성트랙(지정)",
    ] {
        assert!(
            svg.contains(expected),
            "폼 caption 이 한컴 표시 문자열로 렌더링되어야 함: expected={expected}"
        );
    }

    assert!(
        !svg.contains("R&amp;&amp;D"),
        "폼 caption 표시 문자열에 `R&&D`가 그대로 남아 있음"
    );
}
