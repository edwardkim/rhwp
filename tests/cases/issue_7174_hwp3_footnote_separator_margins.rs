//! [Issue #7174] HWP3 각주 모양의 구분선 여백이 HWP5 저장본에서 0 이 된다.
//!
//! `FootnoteShape` 는 구분선 여백을 포맷별 슬롯으로 나눠 갖는다(모델 주석).
//!
//! ```text
//!   separator_margin_top      HWPX 원본 슬롯
//!   separator_margin_bottom   HWP5 원본 슬롯: 구분선 위 여백   -> 저장 offset 16
//!   note_spacing              구분선 아래 여백                 -> 저장 offset 18
//! ```
//!
//! HWP3 경로는 이 값을 미주 기본값(`hwp3_default_endnote_shape`)에서 **HWPX 슬롯**
//! 에만 넣었고, 각주 모양(`section_def.footnote_shape`)에는 배선이 아예 없었다.
//! 그래서 HWP5 로 저장하면 각주 구분선 여백이 통째로 0 으로 나갔다.
//!
//! # 기대값의 출처
//!
//! 한/글 네이티브 HWP5 정본(264쪽 문서의 한컴 변환본)은 `FOOTNOTE_SHAPE[0]` 에
//! 구분선 위 `852` · 아래 `568` 을 쓴다. 이는 `doc_info` 의 hunit 값 `213`·`142` 에
//! ×4 한 값과 정확히 같다.
//!
//! 렌더로도 확인된다 — 각주가 있는 쪽에서 본문 마지막 줄과 각주 첫 줄 사이 간격이
//! 정본 `38.1px`, 수정 전 `17.9px`, 수정 후 `38.5px` 다. 수정 전에는 구분선이
//! 본문과 각주에 붙었다.
//!
//! # 잠그지 않는 것
//!
//! 각주 번호의 `)` 가 본문 글자로 남는 축은 이 시험의 범위가 아니다. 현재 렌더는
//! 정본과 같고(이중 괄호 0건), 모양의 뒤 장식 문자를 채우려면 본문 글자를 동시에
//! 지워야 해서 별도로 다룬다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::parse_document;
use std::path::Path;

fn load(rel: &str) -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱")
}

/// HWP3 각주 구분선 여백이 HWP5 슬롯에 실린다.
#[test]
fn hwp3_footnote_separator_margins_reach_the_hwp5_slots() {
    for rel in [
        "samples/SO-SUEOP.hwp",
        "samples/hwp3-sample10.hwp",
        "samples/hwp3-sample16.hwp",
    ] {
        let doc = load(rel);
        let fs = &doc.sections[0].section_def.footnote_shape;
        assert_ne!(
            fs.separator_margin_bottom, 0,
            "{rel}: 구분선 위 여백이 HWP5 슬롯에 실리지 않았다 — 저장본에서 0 이 된다"
        );
        assert_ne!(
            fs.note_spacing, 0,
            "{rel}: 구분선 아래 여백이 HWP5 슬롯에 실리지 않았다"
        );
    }
}
