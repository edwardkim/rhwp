//! [#7905] 글 뒤로(BehindText) 그림만 얹은 빈 문단의 줄 계약 — 저장 줄 배치(LINE_SEG)가 없는 문서.
//!
//! 한/글이 저장한 `samples/복학원서.hwpx` 의 첫 문단은 글자 없이 로고(BehindText, 문단 기준)만 얹은
//! 빈 문단이다. 그 저장 줄 배치는 이 문단이 흐름에서 **정확히 한 줄**(vertpos 0, 줄 높이 1000 +
//! 줄 간격 400 = 1400HU)을 차지하고 다음 문단이 vertpos 1400 에서 시작한다고 증언한다 — 로고의 높이와
//! 무관하다. 법제처 국가법령정보센터 HWPX 처럼 저장 줄 배치가 없는 기계 생성 문서에서도 본문은 같은
//! 자리에 놓여야 하는데, 그 한 줄이 사라지면 본문 전체가 한 줄 위로 올라와 host 줄과 겹친다.
//!
//! 공개 API 로 만든 합성 문서를 **같은 내용에 저장 줄 배치를 선언한 짝**과 대조한다. 짝의 위치는 선언한
//! `vertical_pos`(HWPUNIT)에서 오므로 구현의 계산값을 빌려 쓰지 않는다. 아래 셋이 양쪽을 잠근다 —
//! 저장 줄 배치가 있는 문서까지 이 규칙이 번지면 ①이, 저장 줄 배치가 없는 문서에서 host 줄이 빠지면 ②가,
//! 줄이 아니라 그림 높이만큼 흐름을 밀면 ③이 깨진다.
//!
//! ```text
//!   ① 저장 줄 배치 있음  — 본문·그림이 선언한 vertical_pos 에 놓인다 (정상 대조군)
//!   ② 저장 줄 배치 없음  — host 가 첫 문단·가운데·연달아 있어도 본문이 ①과 같은 y 에 놓인다
//!   ③ 저장 줄 배치 없음  — 그림 높이가 줄 높이의 30배여도 본문은 ①과 같은 y 에 놓인다
//! ```
//!
//! 범위: 글자 없음 + 글자처럼 취급 아님 + BehindText + 문단 기준 그림 + 저장 줄 배치 없음. 이 밖의
//! 입력(쪽·종이 기준 그림, 표 host 등)은 이 검사가 근거를 확보하지 않았으므로 기대값을 두지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::style::LineSpacingType;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 1 HWPUNIT 이 96dpi 에서 차지하는 픽셀 수(7200 HWPUNIT = 1inch = 96px).
const PX_PER_HWPUNIT: f64 = 96.0 / 7200.0;
/// 선언 단위에서 온 기대값과 비교하는 허용 오차 — 부동소수 오차보다 크고 반 줄보다 훨씬 작다.
const TOLERANCE_PX: f64 = 0.5;

/// 10pt 글자(1000HU)에 160% 줄 간격이 붙은 한 줄의 저장 값: 줄 높이 1000 + 줄 간격 600 = 1600HU.
const STORED_LINE_HEIGHT: i32 = 1000;
const STORED_BASELINE_DISTANCE: i32 = 850;
const STORED_LINE_SPACING: i32 = 600;
const STORED_LINE_PITCH: i32 = STORED_LINE_HEIGHT + STORED_LINE_SPACING;

/// 2cm 정사각형 그림(법제처 QR 과 같은 5668HU).
const ORDINARY_PICTURE_SIZE: u32 = 5668;
/// 줄 높이(1000HU)의 30배 — 쪽 안에 들어가되 한 줄과는 비교할 수 없이 큰 그림.
const TALL_PICTURE_HEIGHT: u32 = 30000;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
    0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 합성 문서의 문단 하나 — 모두 한 줄(1600HU)을 차지한다.
#[derive(Debug, Clone, Copy)]
enum ParagraphSpec {
    /// 글자 없이 글 뒤로·문단 기준 그림 하나만 얹은 host 문단. 값은 그림 높이(HWPUNIT).
    HostWithBehindTextPicture { picture_height: u32 },
    /// 글줄 하나로 끝나는 본문 문단.
    TextLine(&'static str),
}

/// 첫 쪽 렌더 트리에서 문단별로 모은 위쪽 y(px).
#[derive(Debug)]
struct RenderedParagraphTops {
    body_top: f64,
    first_text_line_top_by_paragraph: BTreeMap<usize, f64>,
    picture_top_by_paragraph: BTreeMap<usize, f64>,
}

fn behind_text_picture(picture_height: u32) -> Picture {
    let mut picture = Picture::default();
    picture.common = CommonObjAttr {
        treat_as_char: false,
        text_wrap: TextWrap::BehindText,
        vert_rel_to: VertRelTo::Para,
        horz_rel_to: HorzRelTo::Para,
        width: ORDINARY_PICTURE_SIZE,
        height: picture_height,
        ..Default::default()
    };
    picture.image_attr.bin_data_id = 1;
    picture
}

/// 한 줄 문단의 저장 줄 배치. `paragraph_index` 번째 문단은 앞 문단이 모두 한 줄이라 그 줄 수만큼 내려온 자리다.
fn stored_single_line(paragraph_index: usize, column_width: i32) -> LineSeg {
    let paragraph_number = i32::try_from(paragraph_index).expect("문단 번호");
    LineSeg {
        vertical_pos: paragraph_number * STORED_LINE_PITCH,
        line_height: STORED_LINE_HEIGHT,
        text_height: STORED_LINE_HEIGHT,
        baseline_distance: STORED_BASELINE_DISTANCE,
        line_spacing: STORED_LINE_SPACING,
        segment_width: column_width,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        ..Default::default()
    }
}

fn paragraph_from_spec(
    paragraph_index: usize,
    spec: ParagraphSpec,
    para_shape_id: u16,
    column_width: i32,
    declare_stored_line_layout: bool,
) -> Paragraph {
    let line_segs = if declare_stored_line_layout {
        vec![stored_single_line(paragraph_index, column_width)]
    } else {
        Vec::new()
    };
    let common_paragraph = Paragraph {
        para_shape_id,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs,
        ..Default::default()
    };
    match spec {
        ParagraphSpec::HostWithBehindTextPicture { picture_height } => Paragraph {
            char_count: 1,
            controls: vec![Control::Picture(Box::new(behind_text_picture(
                picture_height,
            )))],
            ..common_paragraph
        },
        ParagraphSpec::TextLine(text) => {
            let character_count = u32::try_from(text.chars().count()).expect("글자 수");
            Paragraph {
                text: text.to_string(),
                char_count: character_count + 1,
                char_offsets: (0..character_count).collect(),
                ..common_paragraph
            }
        }
    }
}

/// 빈 문서에서 출발해 문단 목록만 갈아 끼운 문서를 만든다.
///
/// Pre-condition: 빈 문서의 첫 글자 모양이 10pt, 첫 문단 모양이 줄 간격 160%·문단 간격 0 이어야
/// 한 줄이 1600HU 라는 선언이 성립한다 — 어긋나면 조용히 어긋난 값으로 비교하지 않고 즉시 실패한다.
fn build_document_core(specs: &[ParagraphSpec], declare_stored_line_layout: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서 생성");
    let mut document = core.document().clone();
    let para_shape_id = document.sections[0].paragraphs[0].para_shape_id;
    let para_shape = &document.doc_info.para_shapes[usize::from(para_shape_id)];
    assert_eq!(
        document.doc_info.char_shapes[0].base_size, STORED_LINE_HEIGHT,
        "전제: 빈 문서 기본 글자는 10pt"
    );
    assert!(
        para_shape.line_spacing == 160
            && matches!(para_shape.line_spacing_type, LineSpacingType::Percent)
            && para_shape.spacing_before == 0
            && para_shape.spacing_after == 0,
        "전제: 빈 문서 기본 문단은 줄 간격 160%·문단 간격 0 ({:?})",
        para_shape
    );
    let page = &document.sections[0].section_def.page_def;
    let column_width = i32::try_from(page.width).expect("쪽 너비")
        - i32::try_from(page.margin_left + page.margin_right).expect("좌우 여백");
    document.sections[0].paragraphs = specs
        .iter()
        .enumerate()
        .map(|(paragraph_index, spec)| {
            paragraph_from_spec(
                paragraph_index,
                *spec,
                para_shape_id,
                column_width,
                declare_stored_line_layout,
            )
        })
        .collect();
    document.bin_data_content = vec![BinDataContent {
        id: 1,
        data: PNG_1X1.to_vec().into(),
        extension: "png".to_string(),
    }];
    core.set_document(document);
    core
}

fn collect_rendered_paragraph_tops(core: &DocumentCore) -> RenderedParagraphTops {
    fn walk(node: &RenderNode, tops: &mut RenderedParagraphTops, body_top: &mut Option<f64>) {
        match &node.node_type {
            RenderNodeType::Body { .. } => {
                body_top.get_or_insert(node.bbox.y);
            }
            RenderNodeType::TextRun(run) if !run.text.trim().is_empty() => {
                if let Some(paragraph_index) = run.para_index {
                    tops.first_text_line_top_by_paragraph
                        .entry(paragraph_index)
                        .or_insert(node.bbox.y);
                }
            }
            RenderNodeType::Image(image) => {
                if let Some(paragraph_index) = image.para_index {
                    tops.picture_top_by_paragraph
                        .entry(paragraph_index)
                        .or_insert(node.bbox.y);
                }
            }
            _ => {}
        }
        for child in &node.children {
            walk(child, tops, body_top);
        }
    }
    let page_root = core.build_page_render_tree(0).expect("첫 쪽 렌더").root;
    let mut tops = RenderedParagraphTops {
        body_top: f64::NAN,
        first_text_line_top_by_paragraph: BTreeMap::new(),
        picture_top_by_paragraph: BTreeMap::new(),
    };
    let mut body_top = None;
    walk(&page_root, &mut tops, &mut body_top);
    tops.body_top = body_top.expect("렌더 트리에 본문(Body) 노드가 있어야 한다");
    tops
}

fn render_document(
    specs: &[ParagraphSpec],
    declare_stored_line_layout: bool,
) -> RenderedParagraphTops {
    let core = build_document_core(specs, declare_stored_line_layout);
    assert_eq!(core.page_count(), 1, "합성 문서는 한 쪽에 들어간다");
    collect_rendered_paragraph_tops(&core)
}

/// 선언한 저장 줄 배치가 가리키는 위쪽 y(px): 본문 위쪽 + 앞선 줄 수 × 1600HU.
fn declared_top(body_top: f64, paragraph_index: usize) -> f64 {
    body_top + (paragraph_index as f64) * f64::from(STORED_LINE_PITCH) * PX_PER_HWPUNIT
}

fn host(picture_height: u32) -> ParagraphSpec {
    ParagraphSpec::HostWithBehindTextPicture { picture_height }
}

/// 검사 대상 문단 구성 — 이름은 실패 메시지에서 어느 구성이 깨졌는지 알려 준다.
fn paragraph_layouts() -> Vec<(&'static str, Vec<ParagraphSpec>)> {
    vec![
        (
            "host 가 첫 문단 (법제처 형상)",
            vec![
                host(ORDINARY_PICTURE_SIZE),
                ParagraphSpec::TextLine("본문 첫째 문단"),
                ParagraphSpec::TextLine("본문 둘째 문단"),
            ],
        ),
        (
            "host 가 본문 가운데",
            vec![
                ParagraphSpec::TextLine("앞 문단"),
                host(ORDINARY_PICTURE_SIZE),
                ParagraphSpec::TextLine("뒤 문단"),
            ],
        ),
        (
            "host 가 연달아 둘",
            vec![
                host(ORDINARY_PICTURE_SIZE),
                host(ORDINARY_PICTURE_SIZE),
                ParagraphSpec::TextLine("뒤 문단"),
            ],
        ),
        (
            "그림 높이가 줄 높이의 30배",
            vec![
                host(TALL_PICTURE_HEIGHT),
                ParagraphSpec::TextLine("뒤 문단"),
            ],
        ),
    ]
}

/// ① 저장 줄 배치가 있는 문서는 선언한 자리를 그대로 따른다 — 규칙이 번지지 않았다는 정상 대조군.
#[test]
fn stored_line_layout_keeps_text_and_pictures_at_declared_positions() {
    for (case_name, specs) in paragraph_layouts() {
        let stored = render_document(&specs, true);
        for (paragraph_index, spec) in specs.iter().enumerate() {
            let expected_top = declared_top(stored.body_top, paragraph_index);
            let (kind, actual_top) = match spec {
                ParagraphSpec::TextLine(_) => (
                    "본문",
                    stored.first_text_line_top_by_paragraph[&paragraph_index],
                ),
                ParagraphSpec::HostWithBehindTextPicture { .. } => {
                    ("그림", stored.picture_top_by_paragraph[&paragraph_index])
                }
            };
            println!(
                "[저장 줄 배치 있음] {case_name} / {paragraph_index}번 문단 {kind}: {actual_top:.2}px (선언 {expected_top:.2}px)"
            );
            assert!(
                (actual_top - expected_top).abs() < TOLERANCE_PX,
                "{case_name} / {paragraph_index}번 문단 {kind}: {actual_top:.2}px, 선언한 저장 줄 배치는 {expected_top:.2}px"
            );
        }
    }
}

/// ②③ 저장 줄 배치가 없어도 본문은 저장 줄 배치가 있을 때와 같은 자리에 놓인다 — host 문단이 한 줄을 차지한다.
#[test]
fn host_paragraph_without_stored_line_layout_occupies_exactly_one_line() {
    for (case_name, specs) in paragraph_layouts() {
        let stored = render_document(&specs, true);
        let regenerated = render_document(&specs, false);
        for (paragraph_index, spec) in specs.iter().enumerate() {
            let (kind, stored_top, regenerated_top) = match spec {
                ParagraphSpec::TextLine(_) => (
                    "본문",
                    stored.first_text_line_top_by_paragraph[&paragraph_index],
                    regenerated.first_text_line_top_by_paragraph[&paragraph_index],
                ),
                ParagraphSpec::HostWithBehindTextPicture { .. } => (
                    "그림",
                    stored.picture_top_by_paragraph[&paragraph_index],
                    regenerated.picture_top_by_paragraph[&paragraph_index],
                ),
            };
            println!(
                "[저장 줄 배치 없음] {case_name} / {paragraph_index}번 문단 {kind}: {regenerated_top:.2}px (저장 줄 배치 있음 {stored_top:.2}px, 차 {:+.2}px)",
                regenerated_top - stored_top
            );
            assert!(
                (regenerated_top - stored_top).abs() < TOLERANCE_PX,
                "{case_name} / {paragraph_index}번 문단 {kind}: 저장 줄 배치 없음 {regenerated_top:.2}px, 있음 {stored_top:.2}px — host 문단의 한 줄이 흐름에서 빠지면 뒤 문단이 그만큼({:.2}px) 위에 놓인다",
                f64::from(STORED_LINE_PITCH) * PX_PER_HWPUNIT
            );
        }
    }
}
