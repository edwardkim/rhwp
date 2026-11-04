//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8434의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:41+09:00 (`78826613113ec6de341686832ce875f6e2aa4d0d`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8432/156690773-declared-zero-textbox.hwp`; 독립 Print `pdf/issue8432/156690773-declared-zero-textbox-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.13% / fresh WASM 100.00%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8432/156761223-declared-zero-textbox.hwp`; 독립 Print `pdf/issue8432/156761223-declared-zero-textbox-2022.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.97% / fresh WASM 99.99%.
//! 주장 3쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8434_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-01T20:35:46+09:00 (새 파일).
//! 2026-11-01 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(3쪽):
//! - 156761223: 글상자 3쪽 Native 99.98% / fresh WASM 100.00%, 전 3쪽 최저 Native 97.78% / WASM 98.87%.
//! - 156690773: 글상자 2쪽 Native 99.99% / fresh WASM 100.00%, 전 3쪽 최저 Native 99.34% / WASM 99.87%.
//!   수정 전에는 두 문서 모두 열리지 않았다.
//!
//! 증거: mydocs/pr/assets/issue_8432/{native,wasm}/*_silhouette.tsv.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::DocumentCore;

fn verify(sample: &str, host: usize, text_box_controls: usize) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("한/글이 정상으로 여는 원본");
    let core = DocumentCore::from_bytes(&bytes)
        .unwrap_or_else(|error| panic!("{sample}: 문서를 열어야 한다: {error}"));
    assert_eq!(
        core.page_count(),
        3,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );
    let paragraph = &core.document().sections[0].paragraphs[host];
    let Some(Control::Shape(shape)) = paragraph.controls.first() else {
        panic!("{sample}: 문단 {host} 의 사각형");
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_ref() else {
        panic!("{sample}: 사각형 개체");
    };
    let text_box = rectangle
        .drawing
        .text_box
        .as_ref()
        .expect("소유 글상자 문단 보존");
    // 선언 문단 수는 0 이지만 실제 소유 문단 하나와 그 안의 개체를 그대로 읽는다.
    assert_eq!(text_box.paragraphs.len(), 1, "{sample}: 실제 소유 문단");
    assert_eq!(
        text_box.paragraphs[0].controls.len(),
        text_box_controls,
        "{sample}: 글상자 문단의 개체 보존"
    );
}

#[test]
fn declared_zero_rectangle_list_with_owned_paragraph_opens() {
    verify(
        "samples/issue8432/156761223-declared-zero-textbox.hwp",
        25,
        2,
    );
    verify(
        "samples/issue8432/156690773-declared-zero-textbox.hwp",
        30,
        1,
    );
}
