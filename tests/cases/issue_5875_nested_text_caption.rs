//! Issue #5875: 셀 안 중첩 표의 글자 캡션을 그리지 않아 표 제목이 사라진다.
//!
//! ## 근거
//!
//! `should_render_table_caption` 은 `depth == 0` 이거나 `depth == 1 + 캡션 안
//! TopAndBottom 그림`(#1590)일 때만 캡션을 그렸다. 글자만 든 중첩 표 캡션은
//! 통째로 버려지고(`2181727` 7·8쪽 `<표 1·2·3·5·7>`), 캡션이 차지했어야 할 띠가
//! 표 아래 빈칸으로 남는다. 파서는 캡션을 정상적으로 읽는다(`table.caption`).
//! 분할 경로(`table_partial`)는 애초에 depth 가드가 없으므로 전체 경로도 캡션이
//! 붙어 있으면 무조건 그린다.
//!
//! 검증 방법: 최상위 표의 첫 셀 안에 **글자만 든 캡션**을 단 중첩 표를 심고,
//! 렌더 트리에서 캡션 센티널(`cell_index = 65534`) TextRun 으로 제목 문구가
//! 방출되는지 단언한다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{Caption, CaptionDirection};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwpx/hy-001.hwpx";
const CAPTION_CELL_SENTINEL: usize = 65534;
const CAPTION_TEXT: &str = "<표 1> 공급전압 차단";

/// 본문에서 비어 있지 않은 첫 문단을 찾는다(캡션 내용 재료).
fn first_nonempty_para(paragraphs: &[Paragraph]) -> Option<Paragraph> {
    for para in paragraphs {
        if !para.text.trim().is_empty() {
            return Some(para.clone());
        }
        for ctrl in para.controls.iter() {
            if let Control::Table(table) = ctrl {
                for cell in &table.cells {
                    if let Some(found) = first_nonempty_para(&cell.paragraphs) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

/// 캡션 문단의 텍스트를 고정 문구로 바꿔 판정을 결정적으로 만든다.
fn caption_para_with_fixed_text(mut para: Paragraph) -> Paragraph {
    para.text = CAPTION_TEXT.to_string();
    para.char_count = para.text.chars().count() as u32;
    para.controls.clear();
    para
}

fn top_caption(caption_para: Paragraph) -> Caption {
    Caption {
        direction: CaptionDirection::Top,
        width: 10_000,
        spacing: 0,
        max_width: 50_000,
        paragraphs: vec![caption_para],
        ..Default::default()
    }
}

/// 최상위 표의 첫 셀에 "글자 캡션을 단 중첩 표"를 심는다.
fn attach_nested_text_caption_table(paragraphs: &mut [Paragraph], caption: Caption) -> bool {
    let mut nested_table =
        clone_first_table(paragraphs).expect("fixture must contain a cloneable table");
    nested_table.caption = Some(caption);
    nested_table.common.treat_as_char = true;

    for para in paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Table(table) = ctrl {
                let Some(cell) = table.cells.first_mut() else {
                    return false;
                };
                let Some(cell_para) = cell.paragraphs.first_mut() else {
                    return false;
                };
                cell_para.text.clear();
                cell_para.char_offsets.clear();
                cell_para.char_count = 0;
                cell_para.controls.clear();
                cell_para
                    .controls
                    .push(Control::Table(Box::new(nested_table)));
                return true;
            }
        }
    }
    false
}

fn clone_first_table(paragraphs: &[Paragraph]) -> Option<rhwp::model::table::Table> {
    for para in paragraphs {
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                return Some((**table).clone());
            }
        }
    }
    None
}

/// 캡션 센티널(`cell_index = 65534`)이 붙은 TextRun 텍스트를 모은다.
fn collect_caption_texts(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let is_caption_run = run
            .cell_context
            .as_ref()
            .and_then(|ctx| ctx.path.last())
            .is_some_and(|entry| entry.cell_index == CAPTION_CELL_SENTINEL);
        if is_caption_run && !run.display_or_text().trim().is_empty() {
            out.push(run.display_or_text().to_string());
        }
    }
    for child in &node.children {
        collect_caption_texts(child, out);
    }
}

fn load_doc() -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn nested_table_text_only_caption_is_rendered() {
    let mut doc = load_doc();
    let source_para = first_nonempty_para(&doc.document().sections[0].paragraphs)
        .expect("fixture must contain a non-empty paragraph");
    let caption = top_caption(caption_para_with_fixed_text(source_para));

    assert!(
        attach_nested_text_caption_table(&mut doc.document_mut().sections[0].paragraphs, caption,),
        "fixture must allow inserting a nested caption table"
    );

    // 시각 증적용: 합성 문서를 저장해 베이스/수정 바이너리 export-svg A/B 에 쓴다.
    if let Ok(dir) = std::env::var("RHWP_5875_EVIDENCE_DIR") {
        let bytes = rhwp::serializer::cfb_writer::serialize_hwp(doc.document())
            .expect("serialize synthesized document");
        let out = Path::new(&dir).join("issue_5875_nested_text_caption.hwp");
        fs::write(&out, &bytes).expect("save synthesized document");
    }

    let tree = doc.build_page_render_tree(0).expect("render page 1");
    let mut caption_texts = Vec::new();
    collect_caption_texts(&tree.root, &mut caption_texts);

    let joined = caption_texts.concat();
    assert!(
        joined.contains(CAPTION_TEXT),
        "nested table text-only caption must be rendered on the page tree, got {caption_texts:?}"
    );
}
