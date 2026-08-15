//! 스캐폴드 명세([`ScaffoldSpec`]) → [`Document`] IR 빌더.
//!
//! `build-from-ingest`(`src/document_core/builders/exam_paper.rs`)와 같은 계열의
//! **무(無)에서 생성** 경로다. 문서 제목·개요 제목·본문 문단·단순 표를 표준 layout 의
//! `Document` IR 로 조립하며, 출력은 `serialize_hwpx` 로 직렬화한다.
//!
//! 왕복 정직성: 여기서 만드는 모든 요소는 rhwp 파서로 되읽었을 때 그대로 복원된다
//! (텍스트는 바이트 그대로, 제목은 개요 수준, 표는 치수+셀 텍스트). 검증은
//! `mod.rs` 의 테스트가 `roundtrip_ir_diff` 와 파서 재파싱으로 수행한다.

use crate::model::control::Control;
use crate::model::document::{Document, Section};
use crate::model::page::PageDef;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::shape::{common_obj_offsets, CommonObjAttr};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::model::Padding;
use crate::scaffold::schema::{Block, PageSize, ScaffoldSpec};

// 글자 모양 ID (doc_info.char_shapes 인덱스).
const CS_NORMAL: u32 = 0;
const CS_TITLE: u32 = 1;
const CS_HEADING: u32 = 2;

// 문단 모양 ID (doc_info.para_shapes 인덱스).
const PS_NORMAL: u16 = 0;
const PS_TITLE: u16 = 1;
/// 개요 수준 L(1~7) → para_shape_id = `PS_HEADING_BASE + (L-1)` (2~8).
const PS_HEADING_BASE: u16 = 2;

// 테두리/채우기 ID (1-based, doc_info.border_fills 인덱스 + 1).
const BF_NONE: u16 = 1; // border_fills[0] — 무테두리 (글자/문단 참조)
const BF_SOLID: u16 = 2; // border_fills[1] — 실선 (표/셀 참조)

/// [`ScaffoldSpec`] → [`Document`] IR 변환.
pub fn build_scaffold(spec: &ScaffoldSpec) -> Document {
    let mut doc = Document::default();
    init_doc_info(&mut doc, &spec.font);

    let mut section = Section::default();
    section.section_def.page_def = page_def_from_spec(spec.page_size);
    section.section_def.page_border_fill.border_fill_id = 1;
    section.section_def.page_border_fill.spacing_left = 1417;
    section.section_def.page_border_fill.spacing_right = 1417;
    section.section_def.page_border_fill.spacing_top = 1417;
    section.section_def.page_border_fill.spacing_bottom = 1417;
    doc.sections.push(section);

    let content_width = content_width_of(&doc.sections[0].section_def.page_def);

    // 문서 제목 — 가운데 정렬 제목 문단.
    if let Some(title) = spec
        .title
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        doc.sections[0]
            .paragraphs
            .push(make_text_para(title, PS_TITLE, CS_TITLE));
    }

    for block in &spec.blocks {
        match block {
            Block::Heading { level, text } => {
                let level = (*level).clamp(1, 7);
                let ps_id = PS_HEADING_BASE + (level as u16 - 1);
                doc.sections[0]
                    .paragraphs
                    .push(make_text_para(text, ps_id, CS_HEADING));
            }
            Block::Paragraph { text } => {
                doc.sections[0]
                    .paragraphs
                    .push(make_text_para(text, PS_NORMAL, CS_NORMAL));
            }
            Block::Table { rows } => {
                if let Some(table_para) = build_table_paragraph(rows, content_width) {
                    doc.sections[0].paragraphs.push(table_para);
                    // 표 문단 뒤에는 평문 문단이 온다(한컴 표준 구조 + 다음 표와의 경계).
                    doc.sections[0].paragraphs.push(Paragraph::new_empty());
                }
            }
        }
    }

    // 본문은 최소 1개의 문단으로 끝나야 한다.
    if doc.sections[0].paragraphs.is_empty() {
        doc.sections[0].paragraphs.push(Paragraph::new_empty());
    }

    doc
}

fn init_doc_info(doc: &mut Document, font_name: &str) {
    use crate::model::style::{
        Alignment, BorderFill, BorderLine, BorderLineType, CharShape, Font, HeadType, ParaShape,
        Style, TabDef,
    };

    let font_name = if font_name.trim().is_empty() {
        "함초롬바탕"
    } else {
        font_name.trim()
    };
    let font = Font {
        name: font_name.to_string(),
        alt_type: 1,
        ..Default::default()
    };
    doc.doc_info.font_faces = vec![vec![font]; 7];

    // [#3355] `BorderFill::default()` 는 무테두리가 아니다(Rust 기본 BorderLineType=Solid).
    // 글자/문단이 참조할 무테두리 fill 을 명시 생성한다. 표/셀용 실선 fill 은 별도.
    let no_border = BorderFill {
        borders: [BorderLine {
            line_type: BorderLineType::None,
            width: 0,
            color: 0,
        }; 4],
        ..Default::default()
    };
    let solid = BorderFill {
        borders: [BorderLine {
            line_type: BorderLineType::Solid,
            width: 1,
            color: 0,
        }; 4],
        ..Default::default()
    };
    doc.doc_info.border_fills = vec![no_border, solid];
    doc.doc_info.tab_defs = vec![TabDef::default()];

    // 글자 모양: CharShape::default() 를 기반으로 만들어 #4141(relative_sizes=100)·
    // #4155(shade_color=NONE) 스펙 기본값을 그대로 보존한다.
    let base_char_shape = || {
        let mut cs = CharShape::default();
        cs.ratios = [100; 7];
        cs.base_size = 1000; // 10pt
        cs.border_fill_id = BF_NONE; // 무테두리(#3355 검정 상자 회피)
        cs.text_color = 0x000000;
        cs
    };
    let normal_cs = base_char_shape();
    let mut title_cs = base_char_shape();
    title_cs.base_size = 1800; // 18pt
    title_cs.bold = true;
    let mut heading_cs = base_char_shape();
    heading_cs.base_size = 1200; // 12pt
    heading_cs.bold = true;
    doc.doc_info.char_shapes = vec![normal_cs, title_cs, heading_cs];

    // 문단 모양.
    let normal_ps = ParaShape {
        attr1: (1 << 7) | (1 << 8),
        line_spacing: 160,
        border_fill_id: BF_NONE,
        tab_def_id: 0,
        ..Default::default()
    };
    let title_ps = ParaShape {
        alignment: Alignment::Center,
        spacing_before: 300,
        spacing_after: 300,
        ..normal_ps.clone()
    };
    doc.doc_info.para_shapes = vec![normal_ps.clone(), title_ps];
    // 개요 수준 1~7 → para_shape_id 2~8. head_type=Outline + para_level=L-1 이면
    // export-structure 가 개요 노드(level = para_level+1)로 인식한다.
    for level in 1u8..=7 {
        doc.doc_info.para_shapes.push(ParaShape {
            head_type: HeadType::Outline,
            para_level: level - 1,
            spacing_before: 200,
            spacing_after: 100,
            ..normal_ps.clone()
        });
    }

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
}

fn page_def_from_spec(page_size: PageSize) -> PageDef {
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

/// 표/본문이 차지할 수 있는 편집 영역 폭(HWPUNIT).
fn content_width_of(pd: &PageDef) -> u32 {
    let outer_margin_lr: i32 = 283 * 2;
    (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr).max(7200)
        as u32
}

/// UTF-16 코드 단위 오프셋 배열과 총 길이(비-BMP 문자 포함 정확 계산).
fn utf16_offsets(text: &str) -> (Vec<u32>, u32) {
    let mut offsets = Vec::new();
    let mut acc = 0u32;
    for c in text.chars() {
        offsets.push(acc);
        acc += c.len_utf16() as u32;
    }
    (offsets, acc)
}

/// 평문 텍스트 문단을 만든다(제목/제목/본문 공용).
fn make_text_para(text: &str, para_shape_id: u16, char_shape_id: u32) -> Paragraph {
    let (char_offsets, utf16_len) = utf16_offsets(text);
    Paragraph {
        text: text.to_string(),
        char_count: utf16_len + 1, // +1: 문단 끝 마커
        char_offsets,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id,
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

/// 셀 내부 문단을 만든다. `create_table_native` 의 셀 문단 보정과 정합
/// (char_count_msb=true, raw_header_extra 10바이트, seg_width=셀폭-좌우패딩).
fn make_cell_para(text: &str, col_width: u32) -> Paragraph {
    let (char_offsets, utf16_len) = utf16_offsets(text);
    let seg_w = (col_width as i32) - 141 - 141; // 셀 폭 - 좌우 패딩
    let mut raw_header_extra = vec![0u8; 10];
    raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes()); // n_char_shapes=1
    raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes()); // n_line_segs=1
    Paragraph {
        text: text.to_string(),
        char_count: utf16_len + 1,
        char_count_msb: true, // 셀 문단은 항상 MSB 설정
        char_offsets,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: CS_NORMAL,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            line_spacing: 600,
            segment_width: seg_w,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        para_shape_id: PS_NORMAL,
        style_id: 0,
        has_para_text: !text.is_empty(),
        raw_header_extra,
        ..Default::default()
    }
}

/// 단순 R×C 표를 담은 문단을 만든다. 행마다 열 수가 달라도 최대 열 수에 맞춰
/// 빈 셀로 채운다(직사각 정규화). 빈 표(행 0)면 `None`.
///
/// 구조 조립은 `DocumentCore::create_table_native`
/// (`src/document_core/commands/object_ops/table.rs`)의 균일 그리드 경로와 정합한다.
fn build_table_paragraph(rows: &[Vec<String>], content_width: u32) -> Option<Paragraph> {
    let row_count = rows.len();
    if row_count == 0 {
        return None;
    }
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0).max(1);
    let row_count_u16 = row_count.min(u16::MAX as usize) as u16;
    let col_count_u16 = col_count.min(u16::MAX as usize) as u16;

    let cell_pad = Padding {
        left: 510,
        right: 510,
        top: 141,
        bottom: 141,
    };
    let col_width = (content_width / col_count as u32).max(1);
    let cell_height: u32 = (cell_pad.top + cell_pad.bottom) as u32;
    let rendered_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
    let total_width = col_width * col_count as u32;
    let total_height = rendered_row_height * row_count as u32;

    // 셀 조립 (행 우선).
    let mut cells: Vec<Cell> = Vec::with_capacity(row_count * col_count);
    for r in 0..row_count_u16 {
        for c in 0..col_count_u16 {
            let text = rows
                .get(r as usize)
                .and_then(|row| row.get(c as usize))
                .map(String::as_str)
                .unwrap_or("");
            let mut cell = Cell::new_empty(c, r, col_width, cell_height, BF_SOLID);
            cell.padding = cell_pad;
            cell.vertical_align = VerticalAlign::Center;
            cell.paragraphs = vec![make_cell_para(text, col_width)];
            cell.raw_list_extra = Vec::new();
            cells.push(cell);
        }
    }

    let row_sizes: Vec<i16> = (0..row_count_u16).map(|_| col_count_u16 as i16).collect();

    // raw_ctrl_data: CommonObjAttr 바이너리 (파서 호환, 38바이트).
    let flags: u32 = (2 << 3) | (3 << 8) | (4 << 15) | (2 << 18) | (1 << 21);
    let outer_margin: i16 = 283; // ~1mm
    let mut raw_ctrl_data = vec![0u8; 38];
    raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT].copy_from_slice(&outer_margin.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
    raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM].copy_from_slice(&outer_margin.to_le_bytes());
    let instance_id: u32 = {
        let mut h: u32 = 0x7c15_0000;
        h = h.wrapping_add(row_count_u16 as u32 * 0x1000);
        h = h.wrapping_add(col_count_u16 as u32 * 0x100);
        h = h.wrapping_add(total_width);
        h = h.wrapping_add(total_height.wrapping_mul(0x1b));
        if h == 0 {
            h = 0x7c15_4b69;
        }
        h
    };
    raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());

    let mut table = Table {
        attr: 0x082A_2210,
        row_count: row_count_u16,
        col_count: col_count_u16,
        cell_spacing: 0,
        padding: cell_pad,
        row_sizes,
        border_fill_id: BF_SOLID,
        zones: Vec::new(),
        cells,
        cell_grid: Vec::new(),
        page_break: TablePageBreak::None,
        repeat_header: false,
        caption: None,
        common: CommonObjAttr {
            treat_as_char: false,
            text_wrap: crate::model::shape::TextWrap::TopAndBottom,
            vert_rel_to: crate::model::shape::VertRelTo::Para,
            horz_rel_to: crate::model::shape::HorzRelTo::Para,
            vert_align: crate::model::shape::VertAlign::Top,
            horz_align: crate::model::shape::HorzAlign::Left,
            width: total_width,
            height: total_height,
            ..Default::default()
        },
        outer_margin_left: outer_margin,
        outer_margin_right: outer_margin,
        outer_margin_top: outer_margin,
        outer_margin_bottom: outer_margin,
        raw_ctrl_data,
        raw_table_record_attr: 0x0000_0006, // bit1=셀분리금지, bit2=repeat_header
        raw_table_record_extra: Vec::new(),
        dirty: true,
        text_reflowed_after_edit: false,
        local_resize_rows: Vec::new(),
        local_resize_cols: Vec::new(),
        local_resize_cell_widths: Vec::new(),
        local_resize_cell_heights: Vec::new(),
    };
    table.rebuild_grid();

    let mut table_raw_header_extra = vec![0u8; 10];
    table_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
    table_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());

    Some(Paragraph {
        text: String::new(),
        char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1)
        control_mask: 0x0000_0800,
        char_offsets: vec![],
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: CS_NORMAL,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            line_height: 1000,
            text_height: 1000,
            baseline_distance: 850,
            line_spacing: 600,
            segment_width: 0, // 한컴 표준: 표 문단 segment_width=0
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        para_shape_id: PS_NORMAL,
        style_id: 0,
        controls: vec![Control::Table(Box::new(table))],
        ctrl_data_records: vec![None],
        has_para_text: true,
        raw_header_extra: table_raw_header_extra,
        char_count_msb: false,
        ..Default::default()
    })
}
