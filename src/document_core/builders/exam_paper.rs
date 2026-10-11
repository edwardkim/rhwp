//! 시험문제 Document IR 빌더 (Task #660 본 작업 1단계).
//!
//! [`IngestDocument`]를 시험문제 표준 layout의 [`Document`] IR로 변환한다.
//!
//! 본 단계(#660)는 **텍스트 위주**:
//! - 지문(stem) + 선택지(①~⑤) 텍스트 직접 포함 (spike #654 결정 정책)
//! - 이미지는 `[이미지: <ref>]` placeholder 텍스트로 대체
//! - 공유 지문과 `<보기>` 박스는 schema 의미가 보존되도록 텍스트/문단 스타일로 매핑
//!
//! 후속 단계:
//! - #661: placement 4모드 IR 매핑 + Picture/BinData 빌드 (단 출력은 #182 의존)
//! - #182: HWPX writer Picture 직렬화 분기 추가 (별도 작업자)

use std::collections::{HashMap, HashSet};

use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::page::PageDef;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::parser::ingest::schema::{IngestDocument, Passage, StemBlock};

struct ExamStyleIds {
    normal_para_shape_id: u16,
    boxed_para_shape_id: u16,
}

/// [`IngestDocument`] → [`Document`] IR 변환.
///
/// 시험지 layout:
/// - 각 문제: `{번호}. {지문}` + (이미지 placeholder) + ① ~ ⑤ 선택지 + 빈 문단(다음 문제 간격)
/// - 마지막 문제는 끝 빈 문단 없음
pub fn build_exam_paper(ingest: &IngestDocument) -> Document {
    let mut doc = Document::default();
    let style_ids = init_exam_doc_info(&mut doc, &ingest.default_font);

    let mut section = Section::default();
    section.section_def.page_def = page_def_from_ingest(ingest.page_size);
    section.section_def.page_border_fill.border_fill_id = 1;
    section.section_def.page_border_fill.spacing_left = 1417;
    section.section_def.page_border_fill.spacing_right = 1417;
    section.section_def.page_border_fill.spacing_top = 1417;
    section.section_def.page_border_fill.spacing_bottom = 1417;
    doc.sections.push(section);

    let passage_map: HashMap<&str, &Passage> = ingest
        .passages
        .iter()
        .map(|passage| (passage.id.as_str(), passage))
        .collect();
    let mut emitted_passages = HashSet::new();
    let total_questions = ingest.questions.len();
    for (q_idx, q) in ingest.questions.iter().enumerate() {
        if let Some(passage_ref) = q.passage_ref.as_deref() {
            if emitted_passages.insert(passage_ref.to_string()) {
                if let Some(passage) = passage_map.get(passage_ref) {
                    render_blocks(
                        &passage.blocks,
                        &mut doc.sections[0].paragraphs,
                        &style_ids,
                        style_ids.normal_para_shape_id,
                    );
                } else {
                    doc.sections[0].paragraphs.push(make_text_para_with_shape(
                        &format!("[공유 지문 없음: {passage_ref}]"),
                        style_ids.normal_para_shape_id,
                    ));
                }
            }
        }

        // 1. stem
        if q.stem_blocks.is_empty() {
            // stem_blocks 미제공 시 stem 한 줄 사용
            doc.sections[0].paragraphs.push(make_text_para_with_shape(
                &apply_number_prefix(q, &q.stem),
                style_ids.normal_para_shape_id,
            ));
        } else {
            for (b_idx, block) in q.stem_blocks.iter().enumerate() {
                render_question_block(
                    block,
                    q,
                    b_idx == 0,
                    &mut doc.sections[0].paragraphs,
                    &style_ids,
                );
            }
        }

        // 2. 선택지 ①~⑤ — spike #654 결정 정책: 텍스트 직접 포함
        for choice in &q.choices {
            let line = format!("{} {}", choice.label, choice.text);
            doc.sections[0].paragraphs.push(make_text_para_with_shape(
                &line,
                style_ids.normal_para_shape_id,
            ));
        }

        // 3. 문제 간 빈 문단 (마지막 문제 제외)
        if q_idx + 1 < total_questions {
            doc.sections[0].paragraphs.push(Paragraph::new_empty());
        }
    }

    attach_exam_header_footer(&mut doc, ingest, &style_ids);

    doc
}

fn init_exam_doc_info(doc: &mut Document, default_font: &str) -> ExamStyleIds {
    use crate::model::style::{
        BorderFill, BorderLine, BorderLineType, CharShape, Fill, FillType, Font, ParaShape,
        SolidFill, Style, TabDef,
    };

    let font_name = if default_font.trim().is_empty() {
        "함초롬바탕"
    } else {
        default_font.trim()
    };
    let font = Font {
        name: font_name.to_string(),
        alt_type: 1,
        ..Default::default()
    };
    doc.doc_info.font_faces = vec![vec![font]; 7];
    let boxed_border_fill = BorderFill {
        borders: [BorderLine {
            line_type: BorderLineType::Solid,
            width: 1,
            color: 0,
        }; 4],
        fill: Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: 0x00F7F7F7,
                pattern_color: 0,
                pattern_type: -1,
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    // [#3355] `BorderFill::default()` 는 무테두리가 아니다 — `BorderLineType` 의 Rust
    // 기본값이 Solid(1) 라서 4면 실선이 되고, 이를 기본 글자모양이 참조해 모든 텍스트
    // 런에 검정 상자가 그려졌다. 무테두리를 명시 생성한다.
    let no_border_fill = BorderFill {
        borders: [BorderLine {
            line_type: BorderLineType::None,
            width: 0,
            color: 0,
        }; 4],
        ..Default::default()
    };
    doc.doc_info.border_fills = vec![no_border_fill, boxed_border_fill];
    doc.doc_info.tab_defs = vec![TabDef::default()];

    let mut char_shape = CharShape {
        font_ids: [0; 7],
        ratios: [100; 7],
        relative_sizes: [100; 7],
        base_size: 1000,
        text_color: 0x000000,
        underline_color: 0x000000,
        shadow_color: 0x000000,
        strike_color: 0x000000,
        border_fill_id: 1,
        ..Default::default()
    };
    char_shape.spacings = [0; 7];
    char_shape.char_offsets = [0; 7];
    doc.doc_info.char_shapes = vec![char_shape];

    let normal_para_shape = ParaShape {
        attr1: (1 << 7) | (1 << 8),
        line_spacing: 160,
        border_fill_id: 1,
        tab_def_id: 0,
        ..Default::default()
    };
    let boxed_para_shape = ParaShape {
        border_fill_id: 2,
        border_spacing: [300, 300, 180, 180],
        spacing_before: 120,
        spacing_after: 120,
        ..normal_para_shape.clone()
    };
    doc.doc_info.para_shapes = vec![normal_para_shape, boxed_para_shape];
    doc.doc_info.styles = vec![Style {
        local_name: "바탕글".to_string(),
        english_name: "Normal".to_string(),
        style_type: 0,
        next_style_id: 0,
        lang_id: 1042,
        para_shape_id: 0,
        char_shape_id: 0,
        ..Default::default()
    }];

    ExamStyleIds {
        normal_para_shape_id: 0,
        boxed_para_shape_id: 1,
    }
}

fn page_def_from_ingest(page_size: crate::parser::ingest::schema::PageSize) -> PageDef {
    let mut page_def = PageDef::a4_default();
    if (page_size.width_mm - 210.0).abs() > f32::EPSILON
        || (page_size.height_mm - 297.0).abs() > f32::EPSILON
    {
        page_def.width = mm_to_hwpunit(page_size.width_mm);
        page_def.height = mm_to_hwpunit(page_size.height_mm);
    }
    page_def
}

fn mm_to_hwpunit(mm: f32) -> u32 {
    ((mm.max(0.0) as f64) * 7200.0 / 25.4).round() as u32
}

/// 첫 stem 텍스트에 `{q.number}. ` 접두어를 적용하는 정책 결정.
///
/// **v2 정책 (#660 후속, code review 권고 반영)**: 휴리스틱(`text.starts_with('[')` 등)
/// 제거. `Question.auto_number` 명시 필드(default `true`)에 따라 결정한다.
/// - `auto_number == true` (default): `{n}. ` 자동 prepend
/// - `auto_number == false`: Skill이 명시적으로 prefix 또는 그룹 지시문을 작성한 것으로
///   간주해 그대로 사용
fn apply_number_prefix(q: &crate::parser::ingest::schema::Question, text: &str) -> String {
    if q.auto_number {
        format!("{}. {}", q.number, text)
    } else {
        text.to_string()
    }
}

fn render_question_block(
    block: &StemBlock,
    q: &crate::parser::ingest::schema::Question,
    apply_prefix: bool,
    out: &mut Vec<Paragraph>,
    style_ids: &ExamStyleIds,
) {
    match block {
        StemBlock::Text { text } => {
            let rendered = if apply_prefix {
                apply_number_prefix(q, text)
            } else {
                text.clone()
            };
            out.push(make_text_para_with_shape(
                &rendered,
                style_ids.normal_para_shape_id,
            ));
        }
        StemBlock::Image { ref_, .. } => {
            // Picture/BinData 본격 빌드는 별도 작업이다. schema 의미 손실을 막기 위해
            // 현재 단계에서는 참조 placeholder를 문서에 남긴다.
            out.push(make_text_para_with_shape(
                &format!("[이미지: {ref_}]"),
                style_ids.normal_para_shape_id,
            ));
        }
        StemBlock::Boxed { title, blocks } => {
            if let Some(title) = title.as_deref().filter(|s| !s.trim().is_empty()) {
                out.push(make_text_para_with_shape(
                    title,
                    style_ids.boxed_para_shape_id,
                ));
            }
            render_blocks(blocks, out, style_ids, style_ids.boxed_para_shape_id);
        }
    }
}

fn render_blocks(
    blocks: &[StemBlock],
    out: &mut Vec<Paragraph>,
    style_ids: &ExamStyleIds,
    para_shape_id: u16,
) {
    for block in blocks {
        match block {
            StemBlock::Text { text } => out.push(make_text_para_with_shape(text, para_shape_id)),
            StemBlock::Image { ref_, .. } => out.push(make_text_para_with_shape(
                &format!("[이미지: {ref_}]"),
                para_shape_id,
            )),
            StemBlock::Boxed { title, blocks } => {
                if let Some(title) = title.as_deref().filter(|s| !s.trim().is_empty()) {
                    out.push(make_text_para_with_shape(
                        title,
                        style_ids.boxed_para_shape_id,
                    ));
                }
                render_blocks(blocks, out, style_ids, style_ids.boxed_para_shape_id);
            }
        }
    }
}

fn attach_exam_header_footer(
    doc: &mut Document,
    ingest: &IngestDocument,
    style_ids: &ExamStyleIds,
) {
    let header_text =
        combine_header_text(ingest.form_label.as_deref(), ingest.header_text.as_deref());
    let footer_text = ingest
        .footer_text
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if header_text.is_none() && footer_text.is_none() {
        return;
    }
    if doc.sections.is_empty() {
        return;
    }
    if doc.sections[0].paragraphs.is_empty() {
        doc.sections[0].paragraphs.push(Paragraph::new_empty());
    }

    let page_def = doc.sections[0].section_def.page_def.clone();
    let text_width = page_def
        .width
        .saturating_sub(page_def.margin_left)
        .saturating_sub(page_def.margin_right)
        .saturating_sub(page_def.margin_gutter);
    let header_height = page_def.margin_header.max(1000);
    let footer_height = page_def.margin_footer.max(1000);

    if let Some(text) = header_text {
        let header = Header {
            apply_to: HeaderFooterApply::Both,
            paragraphs: vec![make_text_para_with_shape(
                &text,
                style_ids.normal_para_shape_id,
            )],
            raw_attr: HeaderFooterApply::Both as u32,
            text_width,
            text_height: header_height,
            ..Default::default()
        };
        doc.sections[0].paragraphs[0]
            .controls
            .push(Control::Header(Box::new(header)));
        doc.sections[0].paragraphs[0].char_count += 8;
    }

    if let Some(text) = footer_text {
        let footer = Footer {
            apply_to: HeaderFooterApply::Both,
            paragraphs: vec![make_text_para_with_shape(
                text,
                style_ids.normal_para_shape_id,
            )],
            raw_attr: HeaderFooterApply::Both as u32,
            text_width,
            text_height: footer_height,
            ..Default::default()
        };
        doc.sections[0].paragraphs[0]
            .controls
            .push(Control::Footer(Box::new(footer)));
        doc.sections[0].paragraphs[0].char_count += 8;
    }
}

fn combine_header_text(form_label: Option<&str>, header_text: Option<&str>) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(s) = form_label.map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(s);
    }
    if let Some(s) = header_text.map(str::trim).filter(|s| !s.is_empty()) {
        parts.push(s);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

fn make_text_para_with_shape(text: &str, para_shape_id: u16) -> Paragraph {
    let utf16_len: u32 = text.encode_utf16().count() as u32;
    Paragraph {
        text: text.to_string(),
        char_count: utf16_len + 1, // +1: 문단 끝 마커
        char_offsets: (0..utf16_len).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            line_spacing: 600,
            segment_width: 50000,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        para_shape_id,
        style_id: 0,
        has_para_text: true,
        ..Default::default()
    }
}
