//! [Issue #5678] 자간이 있는 문단의 fit 판정을 공개 조판 경로에서 구속한다.
//!
//! 이것은 현재 구현의 양방향 특성화다. 양수 자간에서 줄 끝 잉크가 한컴 정답지와
//! 일치한다는 판정은 포함하지 않으며, 그 오라클 과제와 per-character allocation은
//! #5678에 남긴다. 내부 helper를 공개하지 않고 `DocumentCore`의 실제 조판 결과만 본다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp};

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const TEXT: &str = "가나다라마바사아자차";

fn document_with_spacing(spacing: i8, content_width_hwp: u32) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("public blank template");
    let mut document = core.document().clone();

    let mut char_shape = document.doc_info.char_shapes[0].clone();
    char_shape.raw_data = None;
    char_shape.base_size = 1_000;
    char_shape.ratios = [100; 7];
    char_shape.spacings = [spacing; 7];
    char_shape.bold = false;
    char_shape.italic = false;
    char_shape.kerning = false;
    char_shape.border_fill_id = 0;
    let char_shape_id = document.doc_info.char_shapes.len() as u32;
    document.doc_info.char_shapes.push(char_shape);

    let mut paragraph = Paragraph::new_empty();
    paragraph.text = TEXT.to_string();
    paragraph.char_count = TEXT.encode_utf16().count() as u32;
    paragraph.char_offsets = (0..TEXT.chars().count() as u32).collect();
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id,
    }];
    paragraph.line_segs.clear();
    paragraph.invalidate_layout_inputs();
    document.sections[0].paragraphs = vec![paragraph];
    let page = &mut document.sections[0].section_def.page_def;
    page.width = content_width_hwp + 2_000;
    page.height = 100_000;
    page.margin_left = 1_000;
    page.margin_right = 1_000;
    page.margin_top = 1_000;
    page.margin_bottom = 1_000;
    core.set_document(document);
    core.register_exact_font_source_native(char_shape_id, 0, SOURCE_HAN, 0)
        .expect("register exact test font");
    core
}

fn collect_text_runs<'a>(node: &'a LayerNode, runs: &mut Vec<(f64, &'a str)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_text_runs(child, runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_text_runs(child, runs),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { bbox, run, .. } = op {
                    runs.push((bbox.y, run.text.as_str()));
                }
            }
        }
    }
}

fn first_line(spacing: i8, content_width_hwp: u32) -> String {
    let tree = document_with_spacing(spacing, content_width_hwp)
        .build_page_layer_tree(0)
        .expect("render synthetic paragraph");
    let mut runs = Vec::new();
    collect_text_runs(&tree.root, &mut runs);
    runs.first()
        .expect("at least one rendered line")
        .1
        .to_string()
}

#[test]
fn positive_spacing_characterizes_the_current_public_line_break() {
    assert_eq!(
        first_line(20, 4_500),
        "가나다라",
        "양수 자간의 후보 끝 보정을 제거하면 첫 줄은 '가나다'로 줄어든다"
    );
}

/// 🔴 음수 자간의 기대값을 `가나다라` → `가나다라마` 로 옮긴다.
///
/// 이 파일 머리말이 남겨 둔 **오라클 과제**(한컴 정답지 대조)를 실제로 재 본 결과다.
/// 후보 끝 보정은 자간을 그대로 빼는데, 음수면 `w - (-x) = w + x` 라 후보가 |자간| 만큼
/// **넓어져** 판정만 엄격해진다(13pt·−8% 에서 +103 HWPUNIT). 마지막 글자의 전진폭에는 축소분이
/// 이미 접혀 있고 한컴 저장 `lineseg` 도 그 접힌 폭으로 담는다(실측: 41자 전진폭 38,613 HU ≤
/// `horzsize` 38,640). 그래서 이 항 때문에 줄마다 마지막 한 글자가 다음 줄로 밀렸다.
///
/// 실사용 문서(11쪽·표 15·그림 8)를 한컴 저장 조판과 줄 단위로 대조하면
/// 이 보정을 남길 때 **57.8%**, 없앨 때 **92.9%** 다. 양수 자간은 `max(0)` 이 항등이라
/// 위 시험(`positive_spacing_...`)의 기대값은 그대로다.
#[test]
fn negative_spacing_characterizes_the_current_public_line_break() {
    assert_eq!(
        first_line(-20, 3_950),
        "가나다라마",
        "음수 자간의 후보 끝 보정을 되살리면 첫 줄은 '가나다라'로 줄어든다"
    );
}
