//! Issue #5599: 한컴 전용 PUA 기호가 표시 대체표에 없어 공개 글꼴로 raw PUA(두부)가
//! 그려지던 문제 — 한컴 PDF 로 확정한 항목의 회귀 가드.
//!
//! 이번에 확정한 두 묶음은 `samples/hwp3-sample11-hwp5.hwp` 와 그 한컴 출력
//! `pdf/hwp3-sample11-2020.pdf` 대조로 얻었다.
//!
//! - 罫線 조각 `U+F0806·F0807·F0808·F080C·F080E·F0810` → `┌ ┬ ┐ └ ┘ │`
//!   (p6 세로 묶음 `━┐ │ │ ━┘`, p22 `━━━┬━━━` 와 `└─>`, p129 `┌ │ └`)
//! - 원숫자 `U+F0288·F0289·F028A·F028C..F0291` → `⓪ ① ② ④ ⑤ ⑥ ⑦ ⑧ ⑨`
//!   (p23 NVRAM 바이트 라벨 줄이 한컴 출력에서 ⓪①②③④⑤⑥⑦⑧⑨ⓐⓑ 로 이어진다)
//!
//! 이 테스트는 그 두 쪽을 렌더해 (1) 대체 문자가 실제로 그려지고 (2) 매핑된
//! 코드포인트의 raw PUA 가 남지 않는 것을 잠근다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwp3-sample11-hwp5.hwp";

/// 이번에 확정해 표에 넣은 코드포인트.
const MAPPED: &[u32] = &[
    0xF0288, 0xF0289, 0xF028A, 0xF028C, 0xF028D, 0xF028E, 0xF028F, 0xF0290, 0xF0291, 0xF0806,
    0xF0807, 0xF0808, 0xF080C, 0xF080E, 0xF0810,
];

fn render(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    doc.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("render page {page}: {e}"))
}

fn assert_no_mapped_raw_pua(svg: &str, page: u32) {
    for &code_point in MAPPED {
        let ch = char::from_u32(code_point).expect("valid code point");
        assert!(
            !svg.contains(ch),
            "p{page}: 매핑된 U+{code_point:05X} 가 raw PUA 로 남았다 — 대체표 투영이 끊겼는지 확인"
        );
    }
}

/// p6(0-based 5): 세로 묶음 `━┐ │ │ ━┘`.
#[test]
fn issue_5599_box_drawing_pua_is_substituted() {
    let svg = render(5);
    for ch in ['┐', '│', '┘'] {
        assert!(svg.contains(ch), "p6 에 {ch} 가 그려져야 한다");
    }
    assert_no_mapped_raw_pua(&svg, 6);
}

/// p23(0-based 22): NVRAM 바이트 라벨 ⓪..⑨.
#[test]
fn issue_5599_circled_digit_pua_is_substituted() {
    let svg = render(22);
    for ch in ['⓪', '①', '②', '④', '⑨'] {
        assert!(svg.contains(ch), "p23 에 {ch} 가 그려져야 한다");
    }
    assert_no_mapped_raw_pua(&svg, 23);
}
