//! 단 정의가 없는 파일의 유효 용지와 본문 흐름을 로드 단계에서 함께 해석한다.
use crate::model::{
    control::Control, document::Document, page::PageDef, paragraph::ColumnBreakType,
};
use crate::parser::FileFormat;
use crate::renderer::{px_to_hwpunit, style_resolver::ResolvedStyleSet};

/// 저수준 파서는 선언 속성을 보존한다. 한컴이 실제로 활성화하는 기본 용지는
/// 파일을 여는 공통 경로에서만 적용하며, 셀 안의 단 정의를 본문 단으로 보지 않는다.
pub(super) fn activate_page_geometry(document: &mut Document, format: FileFormat) -> Vec<usize> {
    if !matches!(format, FileFormat::Hwp | FileFormat::Hwpx)
        || document.layout_profile().hwp3_layout()
    {
        return Vec::new();
    }
    let mut activated = Vec::new();
    for (index, section) in document.sections.iter_mut().enumerate() {
        let controls = || {
            section
                .paragraphs
                .iter()
                .flat_map(|paragraph| &paragraph.controls)
        };
        if !controls().any(|control| matches!(control, Control::SectionDef(_)))
            || controls().any(|control| matches!(control, Control::ColumnDef(_)))
        {
            continue;
        }
        let page = PageDef::a4_default();
        section.section_def.page_def = page.clone();
        for paragraph in &mut section.paragraphs {
            for control in &mut paragraph.controls {
                if let Control::SectionDef(definition) = control {
                    definition.page_def = page.clone();
                }
            }
        }
        activated.push(index);
    }
    activated
}

/// 활성 단이 없었던 구역의 옛 쪽 원점을 새 용지의 저장 경계로 재사용하지 않는다.
/// 줄 상자·문자 분할·표와 셀은 보존하고 본문 전진 축만 공유 스타일 간격으로 잇는다.
/// 실제 쪽 수용과 표 이월은 기존 paginator가 결정하며 명시적 나누기는 보존한다.
pub(super) fn rebase_body_frames(
    document: &mut Document,
    activated: &[usize],
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    for &index in activated {
        let mut next = 0_i32;
        for (paragraph_index, paragraph) in
            document.sections[index].paragraphs.iter_mut().enumerate()
        {
            if paragraph_index > 0 && paragraph.column_type != ColumnBreakType::None {
                next = 0;
            }
            let style = styles.para_styles.get(paragraph.para_shape_id as usize);
            let before = style.map_or(0.0, |style| style.spacing_before);
            let after = style.map_or(0.0, |style| style.spacing_after);
            // 본문 폭의 소유자가 없는 저장 문자 분할은 현재 상자에서 다시 구성한다.
            // 개체만 담는 줄과 표 셀의 물리 메트릭은 본문 문자 분할과 별개다.
            if paragraph
                .text
                .chars()
                .any(|ch| !ch.is_whitespace() && !ch.is_control() && ch != '\u{FFFC}')
            {
                paragraph.stored_text_partition_dirty = true;
            }
            next = next.saturating_add(px_to_hwpunit(before, dpi));
            // 원본 바이트는 raw provenance가 보존한다. 무효한 옛 쪽 좌표를
            // paint가 별도 source 축으로 다시 가져오지 않도록 같은 결과를 발행한다.
            paragraph.source_line_seg_vertical_pos = None;
            for line in &mut paragraph.line_segs {
                line.vertical_pos = next;
                next = next
                    .saturating_add(line.line_height)
                    .saturating_add(line.line_spacing);
            }
            next = next.saturating_add(px_to_hwpunit(after, dpi));
        }
    }
}
