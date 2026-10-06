//! [Issue #7172] HWP3 내어쓰기 문단의 왼쪽 여백이 부풀려져 한/글이 쪽을 더 쓴다.
//!
//! HWP3 은 내어쓰기(음수 들여쓰기) 문단에서 **후속 줄** 기준 여백을 저장한다.
//! HWP5 `ParaShape.margin_left` 는 **첫 줄** 기준이므로 들여쓰기를 더해 옮겨야
//! 한다. 이 정규화가 암호 HWP3 fixture 에만 걸려 있어서, 일반 문서는 저장 여백을
//! 그대로 실어 줄 폭이 좁아졌다.
//!
//! # 실측
//!
//! 264쪽 문서(`1170000-200500003_D0150004-1-001`)를 한/글 자신의 HWP5 변환본과
//! 전 문단 대조하면 `음수 들여쓰기 -> 여백 + 들여쓰기(0 하한)` 이 **3,699/3,699**
//! 전건 성립하고 반례가 0 이다. 들여쓰기가 0 이상인 문단은 여백이 이미 전건
//! 일치한다. 한/글로 열었을 때 쪽수는 **272 -> 263**(정본 264)이 된다.
//!
//! 참고문헌 문단의 렌더 x 좌표도 정본과 같아진다 — 정본 `100.7`, 수정 전 `157.0`,
//! 수정 후 `100.7`.
//!
//! # 상쇄되어 있던 두 번째 결함
//!
//! 여백만 고치면 `SO-SUEOP` 의 줄 상자 18개가 어긋난다. `hwp3_para_line_box` 가
//! 여백에 음수 들여쓰기를 **한 번 더** 더하고 있었기 때문이다. 종전에는 여백이
//! 정확히 `|들여쓰기|` 만큼 부풀어 두 오류가 상쇄됐다. 정본 실측:
//!
//! ```text
//!   여백 6000 · 들여쓰기 -2000 -> 줄 상자 (3000, 39520) = 여백/2
//! ```
//!
//! 두 곳을 함께 고치면 `SO-SUEOP` 의 여백 불일치가 17 -> 0, 줄 상자는 기준선과
//! 같은 4건(이 축과 무관한 기존 차이)으로 돌아온다.
//!
//! 이 시험은 저장소 표본만으로 그 계약을 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::parse_document;
use std::path::Path;

fn load(rel: &str) -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱")
}

/// 내어쓰기 문단의 여백은 첫 줄 기준이다 — 줄 상자가 들여쓰기를 다시 빼지 않는다.
#[test]
fn hanging_indent_margin_is_first_line_based_and_not_double_counted() {
    let doc = load("samples/SO-SUEOP.hwp");
    let section = &doc.sections[0];

    let mut checked = 0usize;
    for para in &section.paragraphs {
        let Some(ps) = doc.doc_info.para_shapes.get(para.para_shape_id as usize) else {
            continue;
        };
        if ps.indent >= 0 {
            continue;
        }
        let Some(seg) = para.line_segs.first() else {
            continue;
        };
        // 줄 상자의 단 시작은 **여백만** 반영한다. 들여쓰기를 한 번 더 빼면
        // 같은 문단에서 `여백/2 + 들여쓰기/2` 가 되어 정본과 어긋난다.
        assert_eq!(
            seg.column_start,
            ps.margin_left / 2,
            "내어쓰기 문단의 줄 상자는 여백만 반영해야 한다 \
             (여백 {} 들여쓰기 {})",
            ps.margin_left,
            ps.indent
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "SO-SUEOP 에 음수 들여쓰기 문단이 없다 — 표본이 바뀌었다면 전제부터 다시 세워야 한다"
    );
}
