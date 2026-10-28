//! [#7130] `META_TEXTOUT` 의 WORD 정렬용 채움 바이트를 글자로 그리지 않는다.
//!
//! MS-WMF §2.3.5.6 에서 `StringLength` 는 글자 수이고, `String` 필드가 차지하는
//! 자리만 짝수로 올림된다. 홀수 길이일 때 뒤에 붙는 1바이트는 정렬용 채움이며
//! **0 으로 정해져 있지 않다**.
//!
//! 재현체 `tests/fixtures/issue_7130/transistor_circuit_ole.wmf` 는 합성물이 아니라
//! 한/글 2020 이 만든 문서(#7105 첨부 `실험4 Transistor-MOSFET.hwp`)의
//! `BinData/BIN0004.OLE` 안 `CONTENTS` 스트림에서 잘라낸 WMF 원본 4,534바이트다.
//! 이 한 파일이 세 가지를 모두 담고 있어 양쪽을 같이 잠근다.
//!
//! | 라벨              | 길이 | 채움 바이트 | 종전 결과      |
//! |-------------------|-----:|-------------|----------------|
//! | `Emitter`         |    7 | `0x6F`(`o`) | `Emittero`     |
//! | `Collector`       |    9 | `0x00`      | `Collector\0`  |
//! | `Base`·`IB`·`PNP Transistor` | 짝수 | 없음 | 정상(반례) |

use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

const FIXTURE: &str = "tests/fixtures/issue_7130/transistor_circuit_ole.wmf";

fn fixture_svg() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("재현체를 읽지 못했습니다 {}: {e}", path.display()));
    let svg = WMFConverter::new(&bytes[..], SVGPlayer::new())
        .run()
        .expect("WMF→SVG 변환이 실패했습니다");
    String::from_utf8_lossy(&svg).into_owned()
}

/// 텍스트 노드에 담긴 글자만 모은다(마크업 제거).
fn text_nodes(svg: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        rest = &rest[open..];
        let Some(gt) = rest.find('>') else { break };
        let body = &rest[gt + 1..];
        let Some(close) = body.find("</text>") else {
            break;
        };
        let raw = &body[..close];
        // `<tspan>` 등 내부 태그를 걷어낸다.
        let mut text = String::new();
        let mut in_tag = false;
        for c in raw.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => text.push(c),
                _ => {}
            }
        }
        out.push(
            text.replace("&amp;", "&")
                .replace("&lt;", "<")
                .replace("&gt;", ">"),
        );
        rest = &body[close..];
    }
    out
}

#[test]
fn odd_length_label_does_not_carry_its_pad_byte() {
    let svg = fixture_svg();
    let nodes = text_nodes(&svg);
    assert!(
        !nodes.is_empty(),
        "재현체에서 텍스트 노드를 하나도 찾지 못했습니다 — 검사 대상이 0건입니다"
    );

    // `Emitter`(7자) 뒤 채움 바이트는 `0x6F` = `o` 다.
    assert!(
        nodes.iter().any(|t| t.trim() == "Emitter"),
        "`Emitter` 가 그대로 나와야 합니다: {nodes:?}"
    );
    assert!(
        !nodes.iter().any(|t| t.contains("Emittero")),
        "채움 바이트 `0x6F` 가 글자로 붙었습니다: {nodes:?}"
    );

    // `Collector`(9자) 뒤 채움 바이트는 `0x00` — 수정 전에도 화면에는 안 드러났다.
    assert!(
        nodes.iter().any(|t| t.trim() == "Collector"),
        "`Collector` 가 그대로 나와야 합니다: {nodes:?}"
    );
    assert!(
        !nodes.iter().any(|t| t.contains('\u{0}')),
        "채움 바이트 `0x00` 이 글자로 남았습니다: {nodes:?}"
    );
}

#[test]
fn even_length_labels_are_untouched() {
    let nodes = text_nodes(&fixture_svg());
    // 같은 재현체의 짝수 길이 라벨 — 채움 바이트가 없으므로 이 수정의 영향을 받지 않는다.
    for expected in ["PNP Transistor", "NPN Transistor", "Base", "IB", "IC", "IE"] {
        assert!(
            nodes.iter().any(|t| t.trim() == expected),
            "짝수 길이 라벨 `{expected}` 이 사라졌습니다: {nodes:?}"
        );
    }
}
