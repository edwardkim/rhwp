//! 관계형 — [#9379] KAF 회의자료 부록: 글꼴에 어울리는 줄 높이 글 상자의 줄 피치, 원래 크기 전체를 자른 WMF 그림.
//!
//! 주석 갱신: 2026-11-25 23:53 KST(+09:00), 한/글 2020 재저장·크기 보호 검증 반영.
//!
//! - 입력: `samples/issue9379/kaf_appendix.hwp` — 10k 공개 코퍼스의 KAF 총회 회의자료(docs#881,
//!   HWP 5.0.2.2, SHA-256 `8014ae9b…f6c9e5`, 66쪽, 저장소에 넣지 않음)에서 rhwp `edit delete-paragraph`
//!   로 구역 1 문단 32(33~44쪽의 11×6 표, #9000)만 지운 파생본(54쪽). 원본 45~66쪽이 33~54쪽이 된다.
//! - 기준: `pdf/issue9379/kaf_appendix-2020.pdf`(한/글 2020 11.0.0.1623, `Hancom PDF` Print,
//!   `hancom2020_pdf_driver_one_up`, 54쪽).
//!
//! 검사:
//! - 관계형(줄 피치): 36·40·42쪽 1×1 글 상자 문단(문단 모양 bit 22 글꼴에 어울리는 줄 높이, 함초롬바탕
//!   13pt, 퍼센트 192%)의 이웃 줄 사이 거리가 한/글 2020 이 같은 원본을 다시 저장한 줄 피치 3243 HU
//!   (43.24px; 줄 높이 1691 = 13pt × HCRBatang hhea (1070+230)/1000)와 0.5px 안에서 같다. 원본 저장
//!   줄 피치는 2496 HU(33.28px)로 옛 한/글이 상자를 적용하지 않은 값이며, 수정 전 rhwp 는 이 값을 그렸다.
//! - 관계형(그림 자르기): 38·39·52쪽 WMF 그림(표 116 기준 이미지 크기 없음, 자르기 = 메타파일 원래
//!   크기 전체 21180×17100·18000×6660·28980×13200 HU)은 자르기 창 없이 그림 틀 전체에 그려진다.
//!   한/글 2020 Print 는 세 그림 모두 비트맵 전체를 틀에 채운다. 수정 전 rhwp 는 15000×15000 범위로
//!   읽어 자르기 창(중첩 `<svg viewBox>`)으로 그림을 줄이거나 잘랐다.
//! - 반례(관계형): 36쪽 띠 그림(자르기 위 48977 / 아래 49260, 원래 크기 297900×49260 의 일부)은 여전히
//!   자르기 창으로 그린다 — 부분 자르기는 기존 계약을 유지한다.
//! - 줄 피치·그림 자르기 검사는 수정 전(devel `8a26c00869`·`0fbee2c04e`)에 실패한다 — 36쪽 피치 33.28px,
//!   38쪽 자르기 창 1개. 반례는 수정 전후 모두 통과한다.
//!
//! 측정(2026-11-25 00:51~02:32 KST, `scripts/visual_sweep.py --silhouette-only`, 96 DPI,
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 내용 실루엣 일치율, 전체 54쪽 = 한컴 PDF 54쪽):
//!
//! | 쪽 | Native base | Native head | fresh WASM base | fresh WASM head |
//! |---|---|---|---|---|
//! | 36 | 82.48 | 91.77 | 82.17 | 92.09 |
//! | 38 | 33.76 | 97.87 | 35.86 | 97.87 |
//! | 39 | 30.67 | 100.00 | 33.72 | 100.00 |
//! | 40 | 72.85 | 100.00 | 73.83 | 99.95 |
//! | 42 | 79.06 | 100.00 | 79.68 | 99.99 |
//! | 52 | 50.13 | 100.00 | 51.04 | 99.99 |
//!
//! - 전쪽 최저: Native head 91.77(36쪽), fresh WASM head 92.09(36쪽); base 는 Native 30.67·WASM 33.72(39쪽).
//!   그 밖의 쪽은 base 와 head 가 같다. base = devel `8a26c00869`, head = 이 수정(source `e6df26160c`).
//!   devel `0fbee2c04e` 와 그 위로 rebase 한 수정(source `26518dbadc`)의 재측정 TSV 4개가 바이트 단위로 같다.
//! - 36쪽은 내용에 맞춰 늘어난 인쇄 상자에 셋째 줄과 안 여백이 들어가는지 확인한다(#9395).
//! - 관계형(#9395): 한/글 2020(11.0.0.9172) 재저장본의 개체 높이 7855 HU와 현재 요소 높이
//!   8743 HU가 다르다. 저장 글줄과 안 여백이 현재 높이를 설명하는 종축 확대에서는 인쇄 클립이
//!   현재 높이를 따른다. 크기를 보호한 대조본은 같은 글줄을 저장해도 요소 높이 7855 HU를 유지한다.
//!   테스트는 해당 관계를 원본 모델에 재현해 클립과 저장 치수를 대조한다.
//!   새 한컴 2020 Print에서 `such as`는 둘째 줄, `and Cadastre`는 셋째 줄에 있으며 위 검사로 보존한다.
//! - 증거: `mydocs/pr/assets/issue_9379/`(전쪽 Native·fresh WASM·base TSV, 36·38·39·40·42·52쪽 review·overlay).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use base64::Engine;
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::paint::RenderProfile;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9379/kaf_appendix.hwp";
/// 한컴 PDF 의 전체 쪽수.
const HANCOM_PAGES: u32 = 54;
/// 한/글 2020 재저장 줄 피치 3243 HU 를 96 DPI 픽셀로.
const HANCOM_PITCH_PX: f64 = 3243.0 * 96.0 / 7200.0;
/// (쪽, 글 상자 줄 첫머리 글들). 넘친 36쪽은 한컴 Print의 다시 나눈 경계를 확인한다.
const TEXT_BOX_LINES: &[(u32, &[&str])] = &[
    (36, &["In each", "such as", "and Cadastre"]),
    (
        40,
        &[
            "We categories",
            "components that",
            "listed below",
            "delivery area",
        ],
    ),
    (42, &["Parcel based", "Survey", "addresses system"]),
];
/// 원래 크기 전체를 자른 WMF 그림이 있는 쪽.
const FULL_CROP_PICTURE_PAGES: &[u32] = &[38, 39, 52];
/// 부분 자르기 띠 그림이 있는 쪽.
const PARTIAL_CROP_PICTURE_PAGE: u32 = 36;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");
    core
}

fn text_runs<'a>(node: &'a RenderNode, out: &mut Vec<(&'a str, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((run.text.as_str(), node.bbox.y));
    }
    for child in &node.children {
        text_runs(child, out);
    }
}

#[test]
fn font_line_height_text_box_lines_use_the_font_line_box() {
    let core = open();
    for (page, firsts) in TEXT_BOX_LINES {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        let mut runs = Vec::new();
        text_runs(&tree.root, &mut runs);
        let line_tops: Vec<f64> = firsts
            .iter()
            .map(|word| {
                runs.iter()
                    .find(|(text, _)| text.trim_start().starts_with(word))
                    .unwrap_or_else(|| panic!("{page}쪽 글 상자 줄 `{word}`"))
                    .1
            })
            .collect();
        for pair in line_tops.windows(2) {
            let pitch = pair[1] - pair[0];
            assert!(
                (pitch - HANCOM_PITCH_PX).abs() <= 0.5,
                "{page}쪽 글 상자 줄 피치 {pitch:.2}px ≠ 한/글 2020 {HANCOM_PITCH_PX:.2}px \
                 — 글꼴에 어울리는 줄 높이 문단은 글꼴 세로 상자(1691 HU)에 퍼센트 간격을 건다"
            );
        }
    }
}

fn find_textbox_with_line<'a>(
    node: &'a RenderNode,
    first_word: &str,
) -> Option<(&'a RenderNode, &'a RenderNode)> {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let matching_line = node.children.iter().find(|line| {
            matches!(line.node_type, RenderNodeType::TextLine(_))
                && line.children.iter().any(|child| {
                    matches!(&child.node_type, RenderNodeType::TextRun(run)
                        if run.text.trim_start().starts_with(first_word))
                })
        });
        if let Some(line) = matching_line {
            return Some((node, line));
        }
    }
    node.children
        .iter()
        .find_map(|child| find_textbox_with_line(child, first_word))
}

#[test]
fn restored_font_line_height_keeps_the_last_line_inside_its_print_frame() {
    let core = open();
    let tree = core.build_page_render_tree(35).unwrap();
    let (textbox, last_line) =
        find_textbox_with_line(&tree.root, "and Cadastre").expect("36쪽 글 상자의 마지막 줄");
    let clip = textbox.print_clip_bbox.expect("글 상자 인쇄 경계");
    let line_bottom = last_line.bbox.y + last_line.bbox.height;
    let clip_bottom = clip.y + clip.height;
    assert!(
        line_bottom < clip_bottom,
        "마지막 글줄 뒤에 아래 안 여백이 있어야 한다 ({line_bottom:.2} < {clip_bottom:.2})"
    );
}

fn terminology_textbox(core: &mut DocumentCore) -> &mut rhwp::model::shape::RectangleShape {
    let Control::Shape(shape) = &mut core.document_mut().sections[1].paragraphs[46].controls[0]
    else {
        panic!("36쪽 용어 글 상자 도형");
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
        panic!("36쪽 용어 글 상자 사각형");
    };
    rectangle
}

#[test]
fn resaved_textbox_uses_the_current_element_height_for_its_print_frame() {
    let mut core = open();
    let rectangle = terminology_textbox(&mut core);
    let attr = &mut rectangle.drawing.shape_attr;
    let stored_height = attr.original_height;
    let text_box = rectangle.drawing.text_box.as_mut().unwrap();
    text_box.font_line_heights_refreshed = false;
    let last = text_box.paragraphs[0].line_segs.last().unwrap();
    let content_height = last.vertical_pos
        + last.line_height
        + i32::from(text_box.margin_top + text_box.margin_bottom);
    attr.current_height = u32::try_from(content_height).unwrap();
    attr.render_sy = f64::from(attr.current_height) / f64::from(stored_height);
    core.set_document(core.document().clone());
    let tree = core.build_page_render_tree(35).unwrap();
    let (textbox, first_line) =
        find_textbox_with_line(&tree.root, "In each").expect("36쪽 용어 글 상자");
    let clip = textbox.print_clip_bbox.expect("글 상자 인쇄 경계");
    let last_line = textbox
        .children
        .iter()
        .rev()
        .find(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .expect("마지막 글줄");
    assert!(last_line.bbox.y + last_line.bbox.height < clip.y + clip.height);
    assert!(first_line.bbox.y >= clip.y);
    assert!((clip.height - f64::from(content_height) * 96.0 / 7200.0).abs() < 1e-6);
}

#[test]
fn protected_textbox_keeps_its_declared_frame_when_lines_overflow() {
    let mut core = open();
    let rectangle = terminology_textbox(&mut core);
    rectangle.common.size_protect = true;
    let declared_height = f64::from(rectangle.common.height) * 96.0 / 7200.0;
    core.set_document(core.document().clone());
    let tree = core.build_page_render_tree(35).unwrap();
    let (textbox, last_line) =
        find_textbox_with_line(&tree.root, "and Cadastre").expect("크기 보호 상태의 셋째 줄");
    let (second_line_box, _) =
        find_textbox_with_line(&tree.root, "such as").expect("크기 보호 상태의 둘째 줄");
    assert_eq!(second_line_box.id, textbox.id);
    let clip = textbox.print_clip_bbox.expect("글 상자 인쇄 경계");
    assert!((clip.height - declared_height).abs() < 1e-6);
    assert!(last_line.bbox.y + last_line.bbox.height > clip.y + clip.height);
}

#[test]
fn resized_inline_textbox_refolds_inside_the_stored_rows() {
    let tree = open().build_page_render_tree(23).unwrap();
    let (textbox, first) = find_textbox_with_line(&tree.root, "그동안").expect("24쪽 설명 글상자");
    let rows: Vec<_> = textbox
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)))
        .collect();
    assert_eq!(rows.len(), 4, "높이를 소유한 저장 글줄 수를 유지한다");
    assert_eq!(rows[0].id, first.id);
    let mut runs = Vec::new();
    text_runs(rows[2], &mut runs);
    assert!(runs
        .first()
        .is_some_and(|(text, _)| text.starts_with("법률")));
    runs.clear();
    text_runs(rows[3], &mut runs);
    assert!(runs
        .first()
        .is_some_and(|(text, _)| text.starts_with("도로명주소로")));
}

#[test]
fn full_single_bitmap_wmf_replays_the_full_print_frame() {
    let core = open();
    let svg = core
        .render_page_svg_layer_with_profile_native(37, RenderProfile::Print)
        .unwrap();
    let page = roxmltree::Document::parse(&svg).unwrap();
    let data = page
        .descendants()
        .filter(|node| node.has_tag_name("image"))
        .find_map(|node| {
            node.attribute("href")?
                .strip_prefix("data:image/svg+xml;base64,")
        })
        .expect("38쪽 WMF 인쇄 자원");
    let embedded = base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap();
    let wmf = roxmltree::Document::parse(std::str::from_utf8(&embedded).unwrap()).unwrap();
    let outer = wmf
        .root_element()
        .attribute("data-rhwp-print-wmf-frame")
        .unwrap();
    let inner = wmf
        .descendants()
        .find_map(|node| node.attribute("data-rhwp-print-wmf-image"))
        .unwrap();
    // 버전과 용지 치수에 이어 나오는 장치 좌표·크기·클립은 같은 틀이다.
    let frame = |marker: &str| {
        marker
            .split_ascii_whitespace()
            .skip(3)
            .take(6)
            .map(|value| value.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(frame(inner), frame(outer));
}

fn nested_crop_windows(svg: &str) -> usize {
    svg.matches("<svg x=").count()
}

#[test]
fn full_crop_wmf_pictures_fill_their_frames() {
    let core = open();
    for page in FULL_CROP_PICTURE_PAGES {
        let svg = core.render_page_svg_native(page - 1).unwrap();
        assert!(svg.contains("<image"), "{page}쪽: 그림이 그려져야 한다");
        assert_eq!(
            nested_crop_windows(&svg),
            0,
            "{page}쪽: 자르기가 WMF 원래 크기 전체와 같은 그림은 자르기 창 없이 틀 전체에 그린다"
        );
    }
}

#[test]
fn partial_crop_wmf_picture_keeps_its_crop_window() {
    let core = open();
    let svg = core
        .render_page_svg_native(PARTIAL_CROP_PICTURE_PAGE - 1)
        .unwrap();
    assert_eq!(
        nested_crop_windows(&svg),
        1,
        "{PARTIAL_CROP_PICTURE_PAGE}쪽: 원래 크기의 일부만 자른 띠 그림은 자르기 창으로 그린다"
    );
}
