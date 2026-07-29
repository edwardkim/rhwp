//! 각주/미주(+미주 수식) native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
    fn find_note_equation_ref(
        &self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<&crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let note_para = match para.controls.get(note_control_idx) {
            Some(Control::Footnote(note)) if kind == "footnote" => {
                note.paragraphs.get(note_para_idx)
            }
            Some(Control::Endnote(note)) if kind == "endnote" => note.paragraphs.get(note_para_idx),
            _ => None,
        }
        .ok_or_else(|| {
            HwpError::RenderError(format!(
                "각주/미주 문단을 찾을 수 없습니다: kind={} sec={} para={} ctrl={} note_para={}",
                kind, section_idx, parent_para_idx, note_control_idx, note_para_idx
            ))
        })?;
        match note_para.controls.get(inner_control_idx) {
            Some(Control::Equation(eq)) => Ok(eq),
            _ => Err(HwpError::RenderError(format!(
                "각주/미주 내부 컨트롤 {}은 수식이 아닙니다",
                inner_control_idx
            ))),
        }
    }
    fn find_note_equation_mut(
        &mut self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<&mut crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let note_para = match para.controls.get_mut(note_control_idx) {
            Some(Control::Footnote(note)) if kind == "footnote" => {
                note.paragraphs.get_mut(note_para_idx)
            }
            Some(Control::Endnote(note)) if kind == "endnote" => {
                note.paragraphs.get_mut(note_para_idx)
            }
            _ => None,
        }
        .ok_or_else(|| {
            HwpError::RenderError(format!(
                "각주/미주 문단을 찾을 수 없습니다: kind={} sec={} para={} ctrl={} note_para={}",
                kind, section_idx, parent_para_idx, note_control_idx, note_para_idx
            ))
        })?;
        match note_para.controls.get_mut(inner_control_idx) {
            Some(Control::Equation(eq)) => Ok(eq),
            _ => Err(HwpError::RenderError(format!(
                "각주/미주 내부 컨트롤 {}은 수식이 아닙니다",
                inner_control_idx
            ))),
        }
    }
    pub fn get_note_equation_properties_native(
        &self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let eq = self.find_note_equation_ref(
            kind,
            section_idx,
            parent_para_idx,
            note_control_idx,
            note_para_idx,
            inner_control_idx,
        )?;
        Ok(Self::equation_properties_json(eq))
    }
    pub fn set_note_equation_properties_native(
        &mut self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let dpi = self.dpi;
        let eq = self.find_note_equation_mut(
            kind,
            section_idx,
            parent_para_idx,
            note_control_idx,
            note_para_idx,
            inner_control_idx,
        )?;
        Self::apply_equation_properties(eq, dpi, props_json);

        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(crate::document_core::helpers::json_ok())
    }
    pub(crate) fn footnote_numbering_name(
        numbering: crate::model::footnote::FootnoteNumbering,
    ) -> &'static str {
        use crate::model::footnote::FootnoteNumbering;
        match numbering {
            FootnoteNumbering::Continue => "continue",
            FootnoteNumbering::RestartSection => "restartSection",
            FootnoteNumbering::RestartPage => "restartPage",
        }
    }
    pub(crate) fn footnote_numbering_from_str(
        value: &str,
        fallback: crate::model::footnote::FootnoteNumbering,
    ) -> crate::model::footnote::FootnoteNumbering {
        use crate::model::footnote::FootnoteNumbering;
        match value {
            "continue" | "CONTINUOUS" | "continuous" => FootnoteNumbering::Continue,
            "restartSection" | "ON_SECTION" | "RESTART_SECTION" | "onSection" => {
                FootnoteNumbering::RestartSection
            }
            "restartPage" | "ON_PAGE" | "RESTART_PAGE" | "onPage" => FootnoteNumbering::RestartPage,
            _ => fallback,
        }
    }
    pub(crate) fn footnote_placement_name(
        placement: crate::model::footnote::FootnotePlacement,
    ) -> &'static str {
        use crate::model::footnote::FootnotePlacement;
        match placement {
            FootnotePlacement::EachColumn => "documentEnd",
            FootnotePlacement::BelowText => "sectionEnd",
            FootnotePlacement::RightColumn => "rightColumn",
        }
    }
    pub(crate) fn footnote_placement_from_str(
        value: &str,
        fallback: crate::model::footnote::FootnotePlacement,
    ) -> crate::model::footnote::FootnotePlacement {
        use crate::model::footnote::FootnotePlacement;
        match value {
            "documentEnd" | "eachColumn" => FootnotePlacement::EachColumn,
            "sectionEnd" | "belowText" => FootnotePlacement::BelowText,
            "rightColumn" => FootnotePlacement::RightColumn,
            _ => fallback,
        }
    }
    pub(crate) fn json_escape_note_char(ch: char) -> String {
        if ch == '\0' {
            String::new()
        } else {
            crate::document_core::helpers::json_escape(&ch.to_string())
        }
    }
    fn make_note_inner_paragraph(
        number_type: crate::model::control::AutoNumberType,
        number: u16,
        format: u8,
        prefix_char: char,
        suffix_char: char,
        default_char_shape_id: u32,
        para_shape_id: u16,
        style_id: u8,
    ) -> crate::model::paragraph::Paragraph {
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        let auto_num = crate::model::control::AutoNumber {
            number_type,
            format,
            superscript: false,
            number,
            assigned_number: number,
            user_symbol: '\0',
            prefix_char,
            suffix_char,
        };

        Paragraph {
            text: "  ".to_string(),
            char_count: 10,
            char_count_msb: true,
            control_mask: 1u32 << 0x12,
            char_offsets: vec![0, 8],
            para_shape_id,
            style_id,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            controls: vec![crate::model::control::Control::AutoNumber(auto_num)],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 0,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: true,
            ..Default::default()
        }
    }
    fn endnote_style_defaults(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> (u32, u16, u8) {
        let section = &self.document.sections[section_idx];

        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Endnote(en) = ctrl {
                    if let Some(ep) = en.paragraphs.first() {
                        let char_shape_id = ep
                            .char_shapes
                            .first()
                            .map(|cs| cs.char_shape_id)
                            .unwrap_or(0);
                        return (char_shape_id, ep.para_shape_id, ep.style_id);
                    }
                }
            }
        }

        for (idx, style) in self.document.doc_info.styles.iter().enumerate() {
            if style.local_name == "미주" || style.english_name.eq_ignore_ascii_case("Endnote") {
                return (
                    style.char_shape_id as u32,
                    style.para_shape_id,
                    idx.min(u8::MAX as usize) as u8,
                );
            }
        }

        // 폴백 — 커서 offset 의 글자모양 기준 (첫 엔트리 아님).
        let current_para = &section.paragraphs[para_idx];
        (
            current_para.char_shape_id_at(char_offset).unwrap_or(0),
            current_para.para_shape_id,
            current_para.style_id,
        )
    }
    /// 각주를 삽입한다.
    /// 커서 위치에 각주 컨트롤을 추가하고 빈 문단 1개를 생성한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N, "footnoteNumber":N}`
    pub fn insert_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::footnote::Footnote;
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }

        // 각주 번호: 삽입 위치 이전의 모든 각주 수 + 1
        // 본문 문단 + 표 셀 + 글상자 내부의 각주를 모두 포함
        let footnote_number = {
            let mut count = 0u16;
            let section = &self.document.sections[section_idx];
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let is_before = pi < para_idx;
                let is_same = pi == para_idx;
                // 본문 문단의 각주
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Footnote(_) => {
                            if is_before {
                                count += 1;
                            } else if is_same {
                                let positions =
                                    crate::document_core::helpers::find_control_text_positions(
                                        para,
                                    );
                                let pos = positions.get(ci).copied().unwrap_or(usize::MAX);
                                if pos <= char_offset {
                                    count += 1;
                                }
                            }
                        }
                        // 표 셀 내 각주
                        // [Task #825 후속] is_same(커서와 같은 문단)일 때, 이 Table
                        // 컨트롤 자체가 커서보다 뒤(char_offset 이후)에 있으면 그
                        // 안의 각주는 아직 삽입 시점 기준으로 "이전" 각주가 아니다.
                        // 기존 코드는 위치를 확인하지 않고 무조건 카운트해
                        // 본문에 텍스트 - 표(각주 포함) - 커서 순서가 아니라
                        // 커서 - 텍스트 - 표(각주 포함) 순서일 때도 표 각주를
                        // 선행으로 오산해 신규 각주 번호가 부풀려졌다.
                        Control::Table(table)
                            if is_before
                                || (is_same && {
                                    let positions =
                                        crate::document_core::helpers::find_control_text_positions(
                                            para,
                                        );
                                    positions.get(ci).copied().unwrap_or(usize::MAX) <= char_offset
                                }) =>
                        {
                            for cell in &table.cells {
                                for cp in &cell.paragraphs {
                                    count +=
                                        cp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Footnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        // 글상자 내 각주 (표와 동일한 위치 검사)
                        Control::Shape(shape)
                            if is_before
                                || (is_same && {
                                    let positions =
                                        crate::document_core::helpers::find_control_text_positions(
                                            para,
                                        );
                                    positions.get(ci).copied().unwrap_or(usize::MAX) <= char_offset
                                }) =>
                        {
                            if let Some(text_box) =
                                shape.drawing().and_then(|d| d.text_box.as_ref())
                            {
                                for tp in &text_box.paragraphs {
                                    count +=
                                        tp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Footnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            count + 1
        };

        // 각주 내부 문단 생성: 기존 각주의 스타일을 참조하여 동일한 스타일 적용
        // 기존 각주가 없으면 본문 문단 스타일 사용
        let (default_char_shape_id, default_para_shape_id) = {
            let section = &self.document.sections[section_idx];
            let mut found = None;
            // 본문 문단의 각주에서 스타일 참조
            'outer: for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Footnote(fn_) = ctrl {
                        if let Some(fp) = fn_.paragraphs.first() {
                            found = Some((
                                fp.char_shapes
                                    .first()
                                    .map(|cs| cs.char_shape_id)
                                    .unwrap_or(0),
                                fp.para_shape_id,
                            ));
                            break 'outer;
                        }
                    }
                    // 표 셀 내 각주에서도 참조
                    if let Control::Table(table) = ctrl {
                        for cell in &table.cells {
                            for cp in &cell.paragraphs {
                                for cc in &cp.controls {
                                    if let Control::Footnote(fn_) = cc {
                                        if let Some(fp) = fn_.paragraphs.first() {
                                            found = Some((
                                                fp.char_shapes
                                                    .first()
                                                    .map(|cs| cs.char_shape_id)
                                                    .unwrap_or(0),
                                                fp.para_shape_id,
                                            ));
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            found.unwrap_or_else(|| {
                // 폴백 — 커서 offset 의 글자모양 기준 (첫 엔트리 아님).
                let current_para = &section.paragraphs[para_idx];
                (
                    current_para.char_shape_id_at(char_offset).unwrap_or(0),
                    current_para.para_shape_id,
                )
            })
        };

        // [Task #1058 reopen Round 5] 신규 각주 inner paragraph 한컴 contract 정합:
        //   - style_id = 11 (각주 style, 한컴 DocInfo 기본 각주 style ID)
        //   - para_shape_id = 0 (각주 default ParaShape)
        //   - controls = [AutoNumber] (각주 번호 inline 컨트롤, char index 0 위치)
        //   - text = "  " (placeholder space ×2, AutoNumber 가 두 space 사이 8 cu 차지)
        //   - char_offsets = [0, 8] (첫 space pos 0, AutoNumber anchor 점유 pos 0~7, 두 번째 space pos 8)
        //   - char_count = 10 (2 placeholder + 8 AutoNumber inline ctrl)
        //   - has_para_text = true
        // 한컴 정답지 samples/footnote-01.hwp 의 각주 inner_para 와 동일한 contract.
        // 사용자 입력은 두 placeholder 뒤 (char_offset=2) 부터 시작 — insert_text_at 의
        // 일반 분기가 char_offsets[i] = base + sum(widths) 시프트 (jump 8 보존).
        let auto_num = crate::model::control::AutoNumber {
            number_type: crate::model::control::AutoNumberType::Footnote,
            format: 0, // Digit
            superscript: false,
            number: footnote_number,
            assigned_number: footnote_number,
            user_symbol: '\0',
            prefix_char: '\0',
            suffix_char: ')',
        };
        let inner_para = Paragraph {
            text: "  ".to_string(), // placeholder space ×2 (정답지 정합)
            char_count: 10,         // 2 placeholder + 8 (AutoNumber inline ctrl)
            char_count_msb: true,
            control_mask: 1u32 << 0x12, // bit 18 (AutoNumber)
            char_offsets: vec![0, 8],   // AutoNumber 가 두 space 사이 8 cu 차지
            para_shape_id: 0,
            style_id: 11, // 각주 style
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            controls: vec![crate::model::control::Control::AutoNumber(auto_num)],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 0,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: true,
            ..Default::default()
        };
        // default_para_shape_id 변수가 위에서 unused 가 되지 않도록 (caller paragraph 의 ps 정보는
        // 본 본문 paragraph 의 contract 보존 — 각주 본문은 ps_id=0 사용)
        let _ = default_para_shape_id;

        let footnote = Footnote {
            number: footnote_number,
            paragraphs: vec![inner_para],
            // [Task #1050] HWP5 CTRL_FOOTNOTE 한컴 default
            after_decoration_letter: 0x0029, // ')'
            ..Default::default()
        };

        // 문단에 각주 컨트롤 삽입
        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        // 삽입 위치 결정 (char_offset 기준)
        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::Footnote(Box::new(footnote)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        // char_offsets 조정: char_offset 위치에 8바이트 갭 생성
        // char_offsets[i]는 텍스트 i번째 문자의 UTF-16 오프셋 (컨트롤은 갭으로 표현)
        // 주의: char_offset은 텍스트 기준 인덱스이지만, char_offsets 배열 길이는 text.chars().count()
        // text에 포함되지 않는 제어 문자(cc - text_len 차이)가 있을 수 있으므로 범위 확인
        paragraph.shift_for_inline_control_insert(char_offset);
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0011; // 각주/미주 비트
        paragraph.has_para_text = true;

        // 전체 각주 순서 번호 재계산 (1부터 순차)
        // 본문 문단 + 표 셀 + 글상자 내부의 각주를 모두 포함
        {
            let mut num = 1u16;
            for pi in 0..self.document.sections[section_idx].paragraphs.len() {
                for ci in 0..self.document.sections[section_idx].paragraphs[pi]
                    .controls
                    .len()
                {
                    match &mut self.document.sections[section_idx].paragraphs[pi].controls[ci] {
                        Control::Footnote(ref mut fn_) => {
                            fn_.number = num;
                            num += 1;
                        }
                        Control::Table(ref mut table) => {
                            for cell in &mut table.cells {
                                for cp in &mut cell.paragraphs {
                                    for cc in &mut cp.controls {
                                        if let Control::Footnote(ref mut fn_) = cc {
                                            fn_.number = num;
                                            num += 1;
                                        }
                                    }
                                }
                            }
                        }
                        Control::Shape(ref mut shape) => {
                            if let Some(text_box) =
                                shape.drawing_mut().and_then(|d| d.text_box.as_mut())
                            {
                                for tp in &mut text_box.paragraphs {
                                    for tc in &mut tp.controls {
                                        if let Control::Footnote(ref mut fn_) = tc {
                                            fn_.number = num;
                                            num += 1;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 각주 내부 문단 리플로우
        self.reflow_footnote_paragraph(section_idx, para_idx, insert_idx, 0);

        // 본문 문단 리플로우 (각주 마커 폭으로 인한 줄넘김 변경 반영)
        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        // 리플로우 + 페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{},\"footnoteNumber\":{}}}",
            para_idx, insert_idx, footnote_number
        ))
    }
    /// 미주를 삽입한다.
    /// 커서 위치에 미주 컨트롤을 추가하고 빈 미주 문단 1개를 생성한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N, "endnoteNumber":N}`
    pub fn insert_endnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::footnote::Endnote;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }

        let shape = self.document.sections[section_idx]
            .section_def
            .endnote_shape
            .clone();
        let start_number = shape.start_number.max(1);
        let number_format_code = Self::footnote_shape_number_format_code(shape.number_format);
        let endnote_number = {
            let mut count = 0u16;
            let section = &self.document.sections[section_idx];
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let is_before = pi < para_idx;
                let is_same = pi == para_idx;
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Endnote(_) => {
                            if is_before {
                                count += 1;
                            } else if is_same {
                                let positions =
                                    crate::document_core::helpers::find_control_text_positions(
                                        para,
                                    );
                                let pos = positions.get(ci).copied().unwrap_or(usize::MAX);
                                if pos <= char_offset {
                                    count += 1;
                                }
                            }
                        }
                        // [Task #825 후속] 각주와 동일한 결함 — is_same 일 때 Table 위치가
                        // char_offset 보다 뒤면 그 안의 미주는 아직 선행이 아니다.
                        Control::Table(table)
                            if is_before
                                || (is_same && {
                                    let positions =
                                        crate::document_core::helpers::find_control_text_positions(
                                            para,
                                        );
                                    positions.get(ci).copied().unwrap_or(usize::MAX) <= char_offset
                                }) =>
                        {
                            for cell in &table.cells {
                                for cp in &cell.paragraphs {
                                    count +=
                                        cp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Endnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        Control::Shape(shape)
                            if is_before
                                || (is_same && {
                                    let positions =
                                        crate::document_core::helpers::find_control_text_positions(
                                            para,
                                        );
                                    positions.get(ci).copied().unwrap_or(usize::MAX) <= char_offset
                                }) =>
                        {
                            if let Some(text_box) =
                                shape.drawing().and_then(|d| d.text_box.as_ref())
                            {
                                for tp in &text_box.paragraphs {
                                    count +=
                                        tp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Endnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            start_number.saturating_add(count)
        };

        let (default_char_shape_id, para_shape_id, style_id) =
            self.endnote_style_defaults(section_idx, para_idx, char_offset);
        let prefix_char = if shape.prefix_char == '\0' {
            '\0'
        } else {
            shape.prefix_char
        };
        let suffix_char = if shape.suffix_char == '\0' {
            ')'
        } else {
            shape.suffix_char
        };

        let inner_para = Self::make_note_inner_paragraph(
            crate::model::control::AutoNumberType::Endnote,
            endnote_number,
            number_format_code,
            prefix_char,
            suffix_char,
            default_char_shape_id,
            para_shape_id,
            style_id,
        );

        let endnote = Endnote {
            number: endnote_number,
            paragraphs: vec![inner_para],
            before_decoration_letter: prefix_char as u16,
            after_decoration_letter: suffix_char as u16,
            number_shape: number_format_code as u32,
            ..Default::default()
        };

        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::Endnote(Box::new(endnote)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        paragraph.shift_for_inline_control_insert(char_offset);
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0011;
        paragraph.has_para_text = true;

        let mut next_number = start_number;
        Self::renumber_paragraph_endnotes_with_shape(
            &mut self.document.sections[section_idx].paragraphs,
            &mut next_number,
            number_format_code,
            prefix_char,
            suffix_char,
        );

        self.reflow_footnote_paragraph(section_idx, para_idx, insert_idx, 0);

        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{},\"endnoteNumber\":{}}}",
            para_idx, insert_idx, endnote_number
        ))
    }
}

#[cfg(test)]
mod char_shape_inherit_tests {
    use crate::document_core::DocumentCore;
    use crate::model::control::Control;
    use crate::model::paragraph::{CharShapeRef, Paragraph};

    /// 혼합 글자모양 문단: 텍스트 20자, 글자 인덱스 0~9 는 34, 10~ 는 37.
    fn core_with_mixed_shape_paragraph() -> DocumentCore {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "0123456789abcdefghij")
            .unwrap();
        let para = &mut core.document.sections[0].paragraphs[0];
        // 컨트롤(SectionDef 등)이 UTF-16 앞자리를 차지하므로 경계는 char_offsets 로 계산.
        let boundary = para.char_offsets[10];
        para.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 34,
            },
            CharShapeRef {
                start_pos: boundary,
                char_shape_id: 37,
            },
        ];
        core
    }

    /// 기존 각주가 없는 문서의 폴백 — 커서 offset 글자모양(37) 상속.
    #[test]
    fn insert_footnote_fallback_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        core.insert_footnote_native(0, 0, 10).unwrap();

        let inner = core.document.sections[0]
            .paragraphs
            .iter()
            .find_map(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::Footnote(f) => f.paragraphs.first(),
                    _ => None,
                })
            })
            .expect("각주 내부 문단");
        assert_eq!(
            inner.char_shapes.first().map(|cs| cs.char_shape_id),
            Some(37),
            "각주 내부 문단이 커서 offset 글자모양(37)이 아닌 값을 상속"
        );
    }

    /// [Task #825 후속] 같은 문단 안, 커서보다 "뒤"(char_offset 이후)에 있는 표
    /// 안 각주는 신규 각주 번호 계산에서 선행 각주로 세면 안 된다.
    ///
    /// 문단 구성: 텍스트 "AB"(char_offset 0~2) 뒤에 표(각주 1개 포함)를 삽입한다.
    /// 그 상태에서 문단 맨 앞(char_offset=0), 즉 표보다 앞에 신규 각주를 삽입하면
    /// 표 각주는 아직 "이전" 각주가 아니므로 신규 각주 번호는 1이어야 한다.
    /// 버그가 있으면 같은 문단(is_same)이라는 이유만으로 표 각주를 무조건 세어
    /// 번호가 2로 부풀려진다.
    #[test]
    fn insert_footnote_ignores_table_footnote_positioned_after_cursor_in_same_paragraph() {
        use crate::model::footnote::Footnote;
        use crate::model::table::{Cell, Table};

        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "AB").unwrap();

        // 표(각주 1개 포함 셀)를 "AB" 텍스트 뒤 sibling control 로 직접 배치한다
        // (create_table_native 는 문단 끝 삽입 시 별도 문단으로 분리하므로, 같은
        // 문단 안 텍스트-뒤 컨트롤 배치를 재현하려면 IR 을 직접 구성해야 한다).
        let mut cell = Cell::default();
        cell.paragraphs.push(Paragraph::default());
        cell.paragraphs[0]
            .controls
            .push(Control::Footnote(Box::<Footnote>::default()));
        let mut table = Table::default();
        table.cells.push(cell);

        let para = &mut core.document.sections[0].paragraphs[0];
        para.controls.push(Control::Table(Box::new(table)));
        para.ctrl_data_records.push(None);
        // "AB"(2) 뒤에 표(8 코드유닛) 갭을 표현하는 char_offsets 항목 추가.
        para.char_offsets.push(2);
        para.char_count += 8;

        // 문단 맨 앞(표보다 앞)에 신규 각주 삽입.
        let result = core
            .insert_footnote_native(0, 0, 0)
            .expect("insert footnote before table");
        let footnote_number: u16 = result
            .split("\"footnoteNumber\":")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("missing footnoteNumber in {result}"));

        assert_eq!(
            footnote_number, 1,
            "표 안 각주가 커서보다 뒤에 있으므로 신규 각주는 1번이어야 한다: {result}"
        );
    }

    /// 기존 미주도 미주 스타일도 없는 문서의 폴백 — 커서 offset 글자모양(37) 상속.
    #[test]
    fn insert_endnote_fallback_inherits_char_shape_at_cursor_offset() {
        let mut core = core_with_mixed_shape_paragraph();
        core.document.doc_info.styles.clear();
        core.insert_endnote_native(0, 0, 10).unwrap();

        let inner = core.document.sections[0]
            .paragraphs
            .iter()
            .find_map(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::Endnote(e) => e.paragraphs.first(),
                    _ => None,
                })
            })
            .expect("미주 내부 문단");
        assert_eq!(
            inner.char_shapes.first().map(|cs| cs.char_shape_id),
            Some(37),
            "미주 내부 문단이 커서 offset 글자모양(37)이 아닌 값을 상속"
        );
    }

    /// [index-axis 회귀] 인라인 각주 삽입이 char_offsets 는 +8 시프트하면서
    /// char_shapes.start_pos 는 시프트하지 않아 글자모양 run 경계가 텍스트와 어긋나던 결함.
    /// text="abcd", char_offsets=[0,1,2,3], run [{0→id5},{2→id6}]. offset 1 에 각주 삽입 후
    /// 원래 'b'(char idx 1)는 여전히 id 5 여야 한다(결함 시 run1(id6)이 덮어 6 반환).
    #[test]
    fn insert_footnote_shifts_char_shapes_with_char_offsets() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "abcd").unwrap();
        {
            let para = &mut core.document.sections[0].paragraphs[0];
            para.text = "abcd".to_string();
            para.char_offsets = vec![0, 1, 2, 3];
            para.char_shapes = vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 5,
                },
                CharShapeRef {
                    start_pos: 2,
                    char_shape_id: 6,
                },
            ];
        }
        core.insert_footnote_native(0, 0, 1).unwrap();
        let para = &core.document.sections[0].paragraphs[0];
        assert_eq!(
            para.char_shape_id_at(1),
            Some(5),
            "각주 삽입 후 원래 'b'(char idx 1) 글자모양이 오염됨 — char_shapes.start_pos 미시프트"
        );
    }
}
