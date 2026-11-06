//! 관계형 및 고정형 회귀: 원본 모양 순서·표시 문자·전각 한 칸의 글자 크기 비율을 보존한다. 절대 배치 좌표는 고정하지 않는다. 같은 원본의 독립 한컴 Print 전체 1쪽과 대응 내용 보존을 함께 검사한다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! hancom-mac-shapes.hwp (8616-hancom-mac-shapes-hwp-2022): 전 1쪽, Native 최저 99.2128%, fresh WASM 최저 99.22558%.
//! hancom-mac-shapes.hwpx (8616-hancom-mac-shapes-hwpx-2022): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 한컴 Mac 12.30 글자 겹치기 창의 14개 모양을 읽고 그리고 저장한다.
//!
//! `samples/char_overlap/hancom-mac-shapes.*`는 한컴이 만든 같은 문서의 HWP·HWPX·Print PDF다.
//! 첫 문단은 "12"를 창의 모양 순서(원, 글자끼리, 반전 원, 사각형 … 굵은 순환 삼각형)로 넣었다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const ORDER: [u8; 14] = [1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

fn open(ext: &str) -> DocumentCore {
    let path = format!(
        "{}/samples/char_overlap/hancom-mac-shapes.{ext}",
        env!("CARGO_MANIFEST_DIR")
    );
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap()
}

fn first_row_borders(core: &DocumentCore) -> Vec<u8> {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::CharOverlap(overlap) => Some(overlap.border_type),
            _ => None,
        })
        .collect()
}

fn page_text(core: &DocumentCore) -> String {
    let svg = core.render_page_svg_native(0).unwrap();
    let mut text = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text ") {
        rest = &rest[start..];
        let end = rest.find('>').unwrap();
        let close = rest.find("</text>").unwrap();
        text.push_str(&rest[end + 1..close]);
        rest = &rest[close + 7..];
    }
    text
}

#[test]
fn hwpx_keeps_all_fourteen_shapes_through_save() {
    let core = open("hwpx");
    assert_eq!(first_row_borders(&core), ORDER);
    let saved = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(first_row_borders(&saved), ORDER);
}

#[test]
fn hancom_border_glyphs_are_painted_for_every_shape() {
    for ext in ["hwpx", "hwp"] {
        let core = open(ext);
        assert_eq!(core.page_count(), 1, "{ext}: 독립 한컴 Print 전체 쪽수");
        let text = page_text(&core);
        for glyph in ['△', '▲', '☼', '◇', '◆', '▢', '♲', '♺', '♻'] {
            assert!(text.contains(glyph), "{ext}: {glyph} 테두리가 없다");
        }
    }
}

/// 한컴 HWP는 반전 원·사각형·반전 사각형 안의 두 자리 숫자를 자릿수별 PUA로 저장한다.
#[test]
fn hwp_two_digit_pua_shapes_draw_the_number() {
    let text = page_text(&open("hwp"));
    assert!(
        !text
            .chars()
            .any(|ch| (0xF0288..=0xF02E9).contains(&u32::from(ch))),
        "PUA 숫자가 그대로 남았다"
    );
    assert!(text.matches("12").count() >= 4, "{text}");
}

fn overlap_widths(node: &rhwp::renderer::render_tree::RenderNode, out: &mut Vec<(f64, f64)>) {
    if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
        if run.char_overlap.is_some() && run.para_index == Some(0) {
            out.push((node.bbox.width, run.style.font_size));
        }
    }
    for child in &node.children {
        overlap_widths(child, out);
    }
}

/// 한컴 Print는 14모양 모두 전각 한 칸(10pt 9.66pt)에 그린다. PUA 원 숫자를 표시 글자로
/// 다시 재 1.5칸이 되거나, 펼친 글자끼리 겹치기가 반각 숫자 폭이 되면 뒤 글자가 밀린다.
#[test]
fn every_shape_takes_one_full_width_cell() {
    for ext in ["hwp", "hwpx"] {
        let core = open(ext);
        let tree = core.build_page_render_tree(0).unwrap();
        let mut widths = Vec::new();
        overlap_widths(&tree.root, &mut widths);
        assert_eq!(widths.len(), 14, "{ext}");
        for (width, size) in widths {
            assert!(
                (width - size).abs() < 1.0,
                "{ext}: 폭 {width} / 글자 크기 {size}"
            );
        }
    }
}
