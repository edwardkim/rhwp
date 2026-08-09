//! 웹한글컨트롤 ParameterSet 값 — `CharShape`·`ParaShape` (규격 §8.2.2, §8.2.11).
//!
//! 한글은 서식을 **파라미터셋**으로 돌려준다. 항목 이름과 단위가 rhwp 모델과 다르므로
//! (`Height` 는 HWPUNIT, `AlignType` 은 코드값) 그 번역을 데이터 곁인 여기에 둔다.
//! 좌표는 한글 커서 좌표(list/para/pos)를 쓴다 — 호출 측이 구역·문단으로 옮기려면
//! 리스트 표를 다시 만들어야 한다.

use crate::document_core::queries::field_query::{
    caret_stops, cell_path_to_list, char_idx_at_stream_pos, cursor_paragraph, json_escape,
    leading_anchor_pos, root_para_location, root_para_of, select_start_pos, stream_len, stream_pos,
    word_end_from, word_starts, ListEntry, EXTENDED_CTRL_UNITS, ROOT_LIST_ID,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{AutoNumber, AutoNumberType, Control};
use crate::model::page::ColumnDef;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::style::{Alignment, HeadType, LineSpacingType, UnderlineType};

/// 언어 일곱 갈래 — 항목 이름 접미사 순서가 모델 배열 순서와 같다.
const LANGS: [&str; 7] = [
    "Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User",
];

fn bit(value: bool) -> u8 {
    u8::from(value)
}

/// 문단 켜고끄기 비트 — 원본은 `attr1`, 5.0.1.7 이후 문서는 `attr2` 에 같은 뜻을 싣는다.
fn para_flag(shape: &crate::model::style::ParaShape, attr1_bit: u32, attr2_bit: u32) -> bool {
    (shape.attr1 >> attr1_bit) & 1 != 0 || (shape.attr2 >> attr2_bit) & 1 != 0
}

/// 한글 `AlignType` 코드 — 0 양쪽혼합 · 1 왼쪽 · 2 오른쪽 · 3 가운데 · 4 배분 · 5 나눔.
fn align_code(alignment: Alignment) -> u8 {
    match alignment {
        Alignment::Justify => 0,
        Alignment::Left => 1,
        Alignment::Right => 2,
        Alignment::Center => 3,
        Alignment::Distribute => 4,
        Alignment::Split => 5,
    }
}

/// 한글 `LineSpacingType` 코드 — 0 글자에 따라(%) · 1 고정값 · 2 여백만 지정.
fn line_spacing_code(kind: LineSpacingType) -> u8 {
    match kind {
        LineSpacingType::Percent => 0,
        LineSpacingType::Fixed => 1,
        LineSpacingType::SpaceOnly => 2,
        LineSpacingType::Minimum => 3,
    }
}

/// 한글 `HeadingType` 코드 — 0 없음 · 1 개요 · 2 번호 · 3 불릿.
fn heading_code(kind: HeadType) -> u8 {
    match kind {
        HeadType::None => 0,
        HeadType::Outline => 1,
        HeadType::Number => 2,
        HeadType::Bullet => 3,
    }
}

/// 한글 `UnderlineType` 코드 — 0 없음 · 1 아래 · 2 위.
fn underline_code(kind: UnderlineType) -> u8 {
    match kind {
        UnderlineType::None => 0,
        UnderlineType::Bottom => 1,
        UnderlineType::Top => 2,
    }
}

/// 컨트롤 하나의 `(CtrlID, CtrlCh, UserDesc)` — 한글이 주는 값 그대로다(실측).
///
/// `CtrlCh` 는 스트림에서의 글자 코드다: 구역·단 정의처럼 문단에 붙는 표식은 **2**, 표·그리기
/// 같은 개체는 **11**. 아직 못 본 갈래는 짐작으로 채우지 않고 빈 이름으로 둔다 — 없는 값을
/// 그럴듯하게 채우면 "모른다"가 사라진다.
fn control_identity(ctrl: &Control) -> (&'static str, u32, &'static str) {
    match ctrl {
        Control::SectionDef(_) => ("secd", 2, "구역 정의"),
        Control::ColumnDef(_) => ("cold", 2, "단 정의"),
        Control::Table(_) => ("tbl", 11, "표"),
        // 그리기 개체의 이름은 갈래마다 다르다("사각형"·"타원" …). rhwp 가 이미 같은 이름을
        // 들고 있어서 그대로 쓴다(오라클 실측과 일치).
        Control::Shape(shape) => ("gso", 11, shape.shape_name()),
        Control::Picture(_) => ("gso", 11, "그림"),
        Control::Equation(_) => ("eqed", 11, "수식"),
        Control::Header(_) => ("head", 2, "머리말"),
        Control::Footer(_) => ("foot", 2, "꼬리말"),
        Control::Footnote(_) => ("fn", 2, "각주"),
        Control::Endnote(_) => ("en", 2, "미주"),
        Control::AutoNumber(_) => ("atno", 2, "자동 번호"),
        Control::NewNumber(_) => ("nwno", 2, "새 번호 지정"),
        Control::PageNumberPos(_) => ("pgnp", 2, "쪽 번호 위치"),
        Control::PageHide(_) => ("pghd", 2, "감추기"),
        Control::Bookmark(_) => ("bokm", 2, "책갈피"),
        Control::HiddenComment(_) => ("tcmt", 2, "숨은 설명"),
        _ => ("", 0, ""),
    }
}

/// 문단 하나가 담은 컨트롤을 사슬에 넣는다 — **자기 다음에 자기 속을** 넣는 깊이 우선이다.
///
/// 셀 안의 표도 사슬에 든다(한글 실측: 중첩 표가 있는 문서에서 `tbl` 이 하나 더 나온다).
/// 리스트 번호를 매기는 규칙(§4.9)과 같은 걸음이라 둘이 어긋나지 않는다.
fn collect_controls(
    para: &Paragraph,
    at: (u32, usize),
    lists: &[ListEntry],
    items: &mut Vec<String>,
) {
    let (list_id, para_in_list) = at;
    let control_positions = para.control_text_positions();
    for (ci, ctrl) in para.controls.iter().enumerate() {
        let (id, ch, desc) = control_identity(ctrl);
        // 앵커 자리는 그 컨트롤이 **스트림에서 서 있는 자리**다(실측: 본문 첫 문단의 셋이
        // 0·8·16, 셀 안의 표는 그 문단의 글자 자리 그대로 `3/14/2`).
        //
        // `stream_pos` 로는 못 구한다 — 자리표 글자를 남기는 문단과 안 남기는 문단이 섞여
        // 있어 그 함수 하나로 갈리지 않는다. 여기서는 **앞선 것들만** 세면 된다:
        // 앞의 맨 글자 수 + 8 × 앞의 컨트롤 수.
        let pos = control_positions
            .get(ci)
            .map(|char_idx| {
                let placeholders_before = control_positions[..ci]
                    .iter()
                    .filter(|p| *p < char_idx)
                    .count();
                let plain_before = char_idx.saturating_sub(placeholders_before);
                plain_before + ci * EXTENDED_CTRL_UNITS
            })
            .unwrap_or(0);
        items.push(format!(
            "{{\"ctrlId\":{},\"ctrlCh\":{},\"userDesc\":{},\"list\":{},\"para\":{},\"pos\":{},\"controlIndex\":{},\"props\":{}}}",
            json_escape(id),
            ch,
            json_escape(desc),
            list_id,
            para_in_list,
            pos,
            ci,
            control_props_json(ctrl),
        ));

        // 자기 다음에 자기 속으로 — 리스트 번호를 매기는 걸음(§4.9)과 같은 순서다.
        let child_of = |cell_index: usize| {
            lists
                .iter()
                .find(|l| {
                    l.host_list_id == list_id
                        && l.host_para_index == para_in_list
                        && l.control_index == ci
                        && l.cell_index == cell_index
                })
                .map(|l| l.list_id)
        };
        match ctrl {
            Control::Table(table) => {
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    let Some(child) = child_of(cell_index) else {
                        continue;
                    };
                    for (pi, cell_para) in cell.paragraphs.iter().enumerate() {
                        collect_controls(cell_para, (child, pi), lists, items);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(text_box) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    let Some(child) = child_of(0) else { continue };
                    for (pi, box_para) in text_box.paragraphs.iter().enumerate() {
                        collect_controls(box_para, (child, pi), lists, items);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 컨트롤의 `Properties` 파라미터셋 — 채울 수 있는 항목만 낸다.
///
/// `Lock` 이 특히 중요하다: **잠긴 개체는 `SelectCtrlFront` 가 건너뛴다**(실측 — 이 표본의
/// 표 열둘 중 잠긴 셋만 정확히 빠진다). §4.34 가 못 풀던 "왜 어떤 개체는 안 골리는가"의 답이다.
///
/// `TextWrap`(본문과의 배치)은 `attr` **비트 21‥22** 다 — 짐작이 아니라 실측으로 뽑았다:
/// 오라클이 준 값 `0`·`1`·`3` 여섯 짝을 두 문서에서 모아 맞는 비트 구간을 찾았다.
/// 한때 이 값이 개체 고르기를 가른다고 봤으나 **반증됐다**(§4.44) — 다른 표본이 어긋났고,
/// 실제로는 그 개체가 캐럿보다 앞이라 안 걸린 것뿐이다. 값은 그대로 싣되 규칙에는 안 쓴다.
///
/// `VertRelTo`·`HorzRelTo` 같은 나머지는 아직 **넣지 않는다** — 짐작으로 채우면 "모른다"가
/// 사라진다(`CharShape` 와 같은 규칙).
fn control_props_json(ctrl: &Control) -> String {
    let common = match ctrl {
        Control::Table(t) => Some(&t.common),
        Control::Shape(s) => Some(s.common()),
        Control::Picture(p) => Some(&p.common),
        _ => None,
    };
    let Some(c) = common else {
        return "{}".to_string();
    };
    format!(
        "{{\"Lock\":{},\"TreatAsChar\":{},\"AllowOverlap\":{},\"TextWrap\":{},\"Width\":{},\"Height\":{}}}",
        u8::from(c.locked),
        u8::from(c.treat_as_char),
        u8::from(c.allow_overlap),
        (c.attr >> 21) & 0x03,
        c.width,
        c.height,
    )
}

impl DocumentCore {
    /// `HwpCtrl.CharShape` 가 돌려줄 값들 (규격 §8.2.2).
    ///
    /// 아직 못 채우는 항목(`FontType*`·`SmallCaps`·`BorderFill`)은 **넣지 않는다** —
    /// 없는 값을 0 으로 채우면 "모른다"와 "0이다"가 구별되지 않는다.
    pub fn char_shape_set_json(&self, list_id: u32, para_in_list: usize, pos: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        let char_idx = char_idx_at_stream_pos(para, pos);
        let shape_id = para.char_shape_id_at(char_idx).unwrap_or(0);
        let Some(raw) = self.document.doc_info.char_shapes.get(shape_id as usize) else {
            return "{}".to_string();
        };
        let style = self.styles.char_styles.get(shape_id as usize);

        let mut items: Vec<String> = Vec::new();
        for (i, lang) in LANGS.iter().enumerate() {
            if let Some(cs) = style {
                let raw_name = cs.font_family_for_lang(i);
                let name = crate::renderer::style_resolver::primary_font_name(&raw_name);
                items.push(format!("\"FaceName{}\":{}", lang, json_escape(name)));
            }
            items.push(format!("\"Size{}\":{}", lang, raw.relative_sizes[i]));
            items.push(format!("\"Ratio{}\":{}", lang, raw.ratios[i]));
            items.push(format!("\"Spacing{}\":{}", lang, raw.spacings[i]));
            items.push(format!("\"Offset{}\":{}", lang, raw.char_offsets[i]));
        }
        items.push(format!("\"Height\":{}", raw.base_size));
        items.push(format!("\"Bold\":{}", bit(raw.bold)));
        items.push(format!("\"Italic\":{}", bit(raw.italic)));
        items.push(format!("\"Emboss\":{}", bit(raw.emboss)));
        items.push(format!("\"Engrave\":{}", bit(raw.engrave)));
        items.push(format!("\"SuperScript\":{}", bit(raw.superscript)));
        items.push(format!("\"SubScript\":{}", bit(raw.subscript)));
        items.push(format!(
            "\"UnderlineType\":{}",
            underline_code(raw.underline_type)
        ));
        items.push(format!("\"UnderlineShape\":{}", raw.underline_shape));
        items.push(format!("\"OutlineType\":{}", raw.outline_type));
        items.push(format!("\"ShadowType\":{}", raw.shadow_type));
        items.push(format!("\"ShadowOffsetX\":{}", raw.shadow_offset_x));
        items.push(format!("\"ShadowOffsetY\":{}", raw.shadow_offset_y));
        items.push(format!("\"StrikeOutType\":{}", bit(raw.strikethrough)));
        items.push(format!("\"DiacSymMark\":{}", raw.emphasis_dot));
        items.push(format!("\"UseFontSpace\":{}", bit(raw.use_font_space)));
        items.push(format!("\"UseKerning\":{}", bit(raw.kerning)));
        items.push(format!("\"TextColor\":{}", raw.text_color));
        items.push(format!("\"ShadeColor\":{}", raw.shade_color));
        items.push(format!("\"UnderlineColor\":{}", raw.underline_color));
        items.push(format!("\"ShadowColor\":{}", raw.shadow_color));
        format!("{{{}}}", items.join(","))
    }

    /// `HwpCtrl.ParaShape` 가 돌려줄 값들 (규격 §8.2.11).
    pub fn para_shape_set_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        let Some(shape) = self
            .document
            .doc_info
            .para_shapes
            .get(para.para_shape_id as usize)
        else {
            return "{}".to_string();
        };
        let items = [
            format!("\"LeftMargin\":{}", shape.margin_left),
            format!("\"RightMargin\":{}", shape.margin_right),
            format!("\"Indentation\":{}", shape.indent),
            format!("\"PrevSpacing\":{}", shape.spacing_before),
            format!("\"NextSpacing\":{}", shape.spacing_after),
            format!("\"LineSpacing\":{}", shape.line_spacing),
            format!(
                "\"LineSpacingType\":{}",
                line_spacing_code(shape.line_spacing_type)
            ),
            format!("\"AlignType\":{}", align_code(shape.alignment)),
            format!("\"HeadingType\":{}", heading_code(shape.head_type)),
            format!("\"Level\":{}", shape.para_level),
            // 켜고 끄는 비트들 — attr1 이 원본이고 attr2 는 5.0.1.7 이후 확장이라 둘 다 본다.
            format!("\"WidowOrphan\":{}", bit(para_flag(shape, 16, 5))),
            format!("\"KeepWithNext\":{}", bit(para_flag(shape, 17, 6))),
            format!("\"KeepLinesTogether\":{}", bit(para_flag(shape, 18, 7))),
            format!("\"PagebreakBefore\":{}", bit(para_flag(shape, 19, 8))),
        ];
        format!("{{{}}}", items.join(","))
    }

    /// 커서 좌표(list/para/pos)로 글자 서식을 건다 — 웹한글컨트롤 `Run("CharShape*")` 용.
    ///
    /// `end_pos` 가 문단 길이를 넘으면 끝까지로 자른다(셀 블록처럼 "이 문단 전부"를 뜻할 때
    /// `u32::MAX` 를 주면 된다). `pos` 는 코드 유닛, rhwp 서식 API 는 글자 번호라 여기서 옮긴다.
    pub fn apply_char_format_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        start_pos: usize,
        end_pos: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let (start_char, end_char) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let last = para.text.chars().count();
            (
                char_idx_at_stream_pos(para, start_pos).min(last),
                char_idx_at_stream_pos(para, end_pos).min(last),
            )
        };
        if start_char >= end_char {
            return Ok(r#"{"ok":false,"reason":"빈 범위"}"#.to_string());
        }

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.apply_char_format_native(sec, para, start_char, end_char, props_json);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.apply_char_format_in_cell_by_path(
            section_index,
            host_para,
            &path,
            start_char,
            end_char,
            props_json,
        )
    }

    /// 커서 좌표(list/para/pos)에 글자를 끼운다 — 웹한글컨트롤 `Run("Insert*Space")` 용.
    ///
    /// 빈칸 세 가지가 스트림에서 저마다 다른 글자다(전부 한 칸): 보통 빈칸 `U+0020`,
    /// 묶음 빈칸 `U+001E`, 고정폭 빈칸 `U+001F`. 탭은 여기 없다 — 확장 컨트롤(8칸)이라
    /// 글자 끼우기로 다룰 수 없다.
    pub fn insert_text_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.insert_text_native(sec, para, char_idx, text);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.insert_text_in_cell_by_path(section_index, host_para, &path, char_idx, text)
    }

    /// 문서가 담은 컨트롤을 **문서 순서로** 늘어놓는다 — `HeadCtrl`·`LastCtrl` 과 `Next`·`Prev`
    /// 가 딛는 사슬이다(규격 §8.4 `CtrlCode`).
    ///
    /// 한글이 주는 값 셋을 그대로 낸다(실측): `CtrlID` 는 네 글자 코드, `CtrlCh` 는 그 컨트롤이
    /// 스트림에서 갖는 글자 코드(구역·단 정의 같은 표식은 2, 개체는 11), `UserDesc` 는 사람이
    /// 읽는 이름("구역 정의"·"표"·"사각형").
    ///
    /// 개체 목록([`objects_json`](Self::objects_json))과 달리 **표식까지 전부** 담는다 —
    /// 한글의 사슬이 그렇다.
    pub fn controls_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<String> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                collect_controls(para, (ROOT_LIST_ID, para_in_body), &lists, &mut items);
                para_in_body += 1;
            }
        }
        format!("[{}]", items.join(","))
    }

    /// 컨트롤 하나를 지운다 — 웹한글컨트롤 `DeleteCtrl`.
    ///
    /// 자리는 컨트롤 사슬이 준 `(list, para, controlIndex)` 다. 본문만 다룬다 — 셀·글상자 안의
    /// 컨트롤은 아래 삭제 API 가 `(구역, 문단, 컨트롤)` 셋만 받아 짚지 못한다.
    pub fn delete_control_at(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        control_index: usize,
    ) -> Result<String, HwpError> {
        if list_id != ROOT_LIST_ID {
            return Ok(
                r#"{"ok":false,"reason":"본문 밖 컨트롤은 아직 다루지 않는다"}"#.to_string(),
            );
        }
        let (sec, para) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        self.delete_control_native(sec, para, control_index)
    }

    /// 개체의 잠금을 켜고 끈다 — 웹한글컨트롤 `ShapeObjLock`·`ShapeObjUnlockAll`.
    ///
    /// `control_index` 가 `None` 이면 **본문 전체**를 푼다(`ShapeObjUnlockAll`). 잠금은 HWP5
    /// `attr` **비트 30** 이라 파서가 읽고 직렬화기가 도로 쓴다 — 값 하나만 뒤집으면 된다.
    ///
    /// 잠금은 고르기를 가른다(잠긴 개체는 `SelectCtrlFront` 가 건너뛴다). 그래서 이 뮤테이터는
    /// 고르기 규칙과 짝이고, 하니스는 잠근 뒤 고르기가 달라지는 것으로 둘을 한꺼번에 검증한다.
    pub fn set_control_lock(
        &mut self,
        para_in_list: Option<usize>,
        control_index: Option<usize>,
        locked: bool,
    ) -> Result<String, HwpError> {
        let target = match para_in_list {
            Some(p) => Some(
                root_para_location(self, p)
                    .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", p)))?,
            ),
            None => None,
        };
        let mut touched = 0usize;
        let mut touched_sections: Vec<usize> = Vec::new();
        for (si, section) in self.document.sections.iter_mut().enumerate() {
            for (pi, para) in section.paragraphs.iter_mut().enumerate() {
                if let Some((ts, tp)) = target {
                    if si != ts || pi != tp {
                        continue;
                    }
                }
                for (ci, ctrl) in para.controls.iter_mut().enumerate() {
                    if let Some(want) = control_index {
                        if ci != want {
                            continue;
                        }
                    }
                    let common = match ctrl {
                        Control::Table(t) => Some(&mut t.common),
                        Control::Shape(s) => Some(s.common_mut()),
                        Control::Picture(p) => Some(&mut p.common),
                        _ => None,
                    };
                    let Some(c) = common else { continue };
                    touched_sections.push(si);
                    c.locked = locked;
                    // 저장으로도 살아남게 원본 비트까지 맞춘다(파서가 읽는 자리와 같다).
                    if locked {
                        c.attr |= 1 << 30;
                    } else {
                        c.attr &= !(1u32 << 30);
                    }
                    touched += 1;
                }
            }
        }
        // 잠금은 `attr` 비트라 저장기가 다시 써야 한다 — 손댄 구역의 원본 스트림을 버린다.
        for si in touched_sections {
            if let Some(section) = self.document.sections.get_mut(si) {
                section.raw_stream = None;
            }
        }
        Ok(format!("{{\"ok\":true,\"touched\":{}}}", touched))
    }

    /// 커서가 든 필드의 상태 — 웹한글컨트롤 `CurFieldState`(규격 §8.2).
    ///
    /// 값은 **갈래 + 필드 비트**다(오라클 실측 넷으로 세운 규칙):
    ///
    /// | 자리 | 값 | 풀이 |
    /// | --- | --- | --- |
    /// | 본문 | 0 | 갈래 0, 필드 밖 |
    /// | 표 셀 안(필드 아님) | 1 | 갈래 1 |
    /// | 셀 필드 안 | 17 | 갈래 1 + `0x10` |
    /// | 누름틀 안 | 18 | 갈래 2 + `0x10` |
    ///
    /// 셀 필드는 **셀 전체가 필드**라 그 셀 안이면 어디든 해당한다(`start_pos`·`end_pos` 가 0).
    pub fn cur_field_state(&self, list_id: u32, para_in_list: usize, pos: usize) -> u32 {
        const IN_FIELD: u32 = 0x10;
        const KIND_CELL: u32 = 1;
        const KIND_CLICK_HERE: u32 = 2;

        let (fields, lists) = self.collect_fields_and_lists();
        let in_cell = lists.iter().any(|l| l.list_id == list_id && l.is_cell);
        let mut state = if in_cell { KIND_CELL } else { 0 };
        for f in &fields {
            if f.list_id != list_id {
                continue;
            }
            // 셀 필드는 범위가 없다 — 그 셀 안이면 어디든 필드 안이다.
            if f.start_pos == 0 && f.end_pos == 0 {
                state = IN_FIELD | KIND_CELL;
                continue;
            }
            if f.para_in_list == para_in_list && pos >= f.start_pos && pos <= f.end_pos {
                return IN_FIELD | KIND_CLICK_HERE;
            }
        }
        state
    }

    /// 커서가 든 셀의 모양 — 웹한글컨트롤 `CellShape` 파라미터셋(규격 §8.2).
    ///
    /// 오라클이 답하는 항목은 `Width`·`Height`·`VertAlign` 이다(나머지 이름은 전부 `null` —
    /// `CellWidth`·`MarginLeft` 따위는 이 컨트롤에 없다). 셀이 아니면 빈 셋이다.
    pub fn cell_shape_set_json(&self, list_id: u32) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let Some(entry) = lists.iter().find(|l| l.list_id == list_id) else {
            return "{}".to_string();
        };
        if !entry.is_cell {
            return "{}".to_string();
        }
        let Some(section) = self.document.sections.get(entry.section_index) else {
            return "{}".to_string();
        };
        let Some(para) = section.paragraphs.get(entry.host_para_index) else {
            return "{}".to_string();
        };
        let Some(Control::Table(table)) = para.controls.get(entry.control_index) else {
            return "{}".to_string();
        };
        let Some(cell) = table.cells.get(entry.cell_index) else {
            return "{}".to_string();
        };
        format!(
            "{{\"Width\":{},\"Height\":{},\"VertAlign\":{}}}",
            cell.width, cell.height, cell.vertical_align as u8,
        )
    }

    /// 본문에 놓인 **개체** 목록 — `Run("ShapeObjNextObject")` 따위가 딛는다.
    ///
    /// 개체는 그림·그리기·수식과 **표**다. 표를 빼면 한글이 고르는 자리와 안 맞는다 —
    /// 실측: 오라클은 `0/0/16`(그리기)뿐 아니라 `0/1/0`·`0/4/0` 도 고르는데 그 둘이 표다.
    /// 개체를 고르면 캐럿이 `(문단, 8 × 컨트롤 번호)` 에 선다.
    ///
    /// `listId` 는 그 개체가 글자를 담는 리스트가 있을 때만 있다(글상자). `ShapeObjTextBoxEdit`
    /// 가 그 안으로 들어간다.
    pub fn objects_json(&self) -> String {
        let (_, lists) = self.collect_fields_and_lists();
        let mut items: Vec<String> = Vec::new();
        // 본문은 구역을 가로질러 이어진다 — 문단 번호도 이어서 센다.
        let mut para_base = 0usize;
        for (sec_idx, section) in self.document.sections.iter().enumerate() {
            for (para_off, para) in section.paragraphs.iter().enumerate() {
                let para_idx = para_base + para_off;
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let kind = match ctrl {
                        Control::Shape(_) => "shape",
                        Control::Picture(_) => "picture",
                        Control::Equation(_) => "equation",
                        Control::Table(_) => "table",
                        _ => continue,
                    };
                    // 리스트 표의 `host_para_index` 는 **구역 안 번호**다(본문 번호가 아니다).
                    let list_id = lists
                        .iter()
                        .find(|l| {
                            !l.is_cell
                                && l.host_list_id == ROOT_LIST_ID
                                && l.section_index == sec_idx
                                && l.host_para_index == para_off
                                && l.control_index == ci
                        })
                        .map(|l| l.list_id.to_string())
                        .unwrap_or_else(|| "null".to_string());
                    // 자리차지(어울림)인지 글자처럼 다루는지 — 한글이 개체로 고르는 것과 갈릴 수 있다.
                    let anchored = match ctrl {
                        Control::Table(t) => !t.common.treat_as_char,
                        Control::Shape(s) => !s.common().treat_as_char,
                        _ => true,
                    };
                    items.push(format!(
                    "{{\"para\":{},\"controlIndex\":{},\"kind\":\"{}\",\"listId\":{},\"anchored\":{}}}",
                    para_idx, ci, kind, list_id, anchored
                ));
                }
            }
            para_base += section.paragraphs.len();
        }
        format!("[{}]", items.join(","))
    }

    /// 커서 자리에서 문단을 가른다 — 웹한글컨트롤 `Run("BreakPara")`.
    ///
    /// 캐럿은 새 문단의 처음으로 간다(실측: 6/0/1 에서 걸면 6/1/0).
    pub fn split_para_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
    ) -> Result<String, HwpError> {
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.split_paragraph_native(sec, para, char_idx, None);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.split_paragraph_in_cell_by_path(section_index, host_para, &path, char_idx, None)
    }

    /// 나누기 — 웹한글컨트롤 `Run("BreakPage"·"BreakColumn"·"BreakColDef"·"BreakSection")`.
    ///
    /// 넷은 **한 규칙**이다(실측 15건, 계획서 §4.45). 표식이 앉는 문단은 캐럿이 문단의 어디에
    /// 있느냐로 갈린다 — 문단을 가르는 것은 **한가운데일 때뿐**이다:
    ///
    /// | 캐럿 | 하는 일 | 표식이 앉는 문단 |
    /// | --- | --- | --- |
    /// | 문단 끝 | 안 가름 | 다음 문단 |
    /// | 문단 처음 | 안 가름 | 그 문단 |
    /// | 한가운데 | 가름 | 뒤쪽 문단 |
    ///
    /// 시작과 끝이 같은 문단(자리차지 하나뿐인 문단)은 한글이 **끝으로 친다** — 그래서 끝 가지를
    /// 먼저 본다. 표식은 갈래마다 크기가 다르다: 쪽·단은 문단 속성이라 0칸, `BreakColDef` 는
    /// `ColumnDef` 하나로 8칸, `BreakSection` 은 `SectionDef`+`ColumnDef` 로 16칸이다.
    ///
    /// 캐럿은 `max(표식칸, 대상 문단의 원래 시작)` 에 선다. 이 `max` 는 **맞춘 식**이지 밝힌
    /// 기전이 아니다 — 액션 넷 × 자리 셋 + 판별 자리 셋, 열다섯 관측에 전부 맞는다.
    ///
    /// 처음엔 빈 문단에서 재다가 앞의 둘이 "아무 일도 안 한다"고 볼 뻔했다.
    /// **자를 빈 곳에 대면 눈금이 안 보인다.**
    pub fn break_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        kind: &str,
    ) -> Result<String, HwpError> {
        let marker_units = match kind {
            "page" | "column" => 0usize,
            "colDef" => EXTENDED_CTRL_UNITS,
            "section" => 2 * EXTENDED_CTRL_UNITS,
            other => return Err(HwpError::InvalidField(format!("모르는 나누기 {}", other))),
        };
        let (start, end) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            (leading_anchor_pos(para), stream_len(para))
        };

        // 대상 문단을 고른다. 한가운데일 때만 가른다.
        //
        // 시작과 끝이 같은 문단은 `end > 0` 으로 갈린다: 자리차지가 하나라도 있으면(끝 8) 끝으로
        // 치고, **아무것도 없는 빈 문단**(끝 0)은 자기가 표식을 진다. 이 갈림은 오라클이 캐럿을
        // 어디 두는지로만 보인다 — 쪽 나눔은 0칸이라 문단 자에는 아무 자국도 안 남는다.
        let at_end = pos >= end && end > 0;
        let target_in_list = if at_end {
            para_in_list + 1
        } else if pos <= start {
            para_in_list
        } else {
            let raw = self.split_para_at_cursor(list_id, para_in_list, pos)?;
            if raw.contains("\"ok\":false") {
                return Ok(raw);
            }
            para_in_list + 1
        };
        let at_start = !at_end && target_in_list == para_in_list;

        // 대상 문단이 **이미 표식을 지고 있으면** 겹쳐 얹을 수 없다 — 한글은 그 자리에 빈 문단을
        // 새로 끼운다(실측: 쪽 나눔이 앉은 문단에 단 나눔을 걸면 문단이 하나 는다). 그래서
        // "가르지 않는다"는 앞의 표는 **대상이 비어 있을 때** 이야기다.
        let occupied = self
            .cursor_paragraph_ref(list_id, target_in_list)
            .map(|p| {
                p.column_type != ColumnBreakType::None
                    || matches!(
                        p.controls.first(),
                        Some(Control::SectionDef(_) | Control::ColumnDef(_))
                    )
            })
            .unwrap_or(false);
        if occupied {
            self.insert_empty_paragraph_at(list_id, target_in_list)?;
        }

        // 캐럿은 **글을 따라간다.** 문단 처음에서 걸어 빈 문단을 끼웠다면 캐럿이 있던 글은
        // 한 칸 뒤로 밀렸으니 캐럿도 따라간다. 문단 끝에서 끼웠다면 새 문단이 캐럿 뒤에 오므로
        // 캐럿이 그 안으로 들어간다(실측: 앞은 9/8, 뒤는 7/0).
        let caret_para = if at_start && occupied {
            target_in_list + 1
        } else {
            target_in_list
        };
        // 표식을 얹기 **전에** 캐럿이 설 문단의 시작을 재 둔다.
        let caret_start = self
            .cursor_paragraph_ref(list_id, caret_para)
            .map(leading_anchor_pos)
            .unwrap_or(0);
        let caret = marker_units.max(caret_start);

        if list_id != ROOT_LIST_ID {
            // 본문 밖은 구역·단 정의를 둘 곳이 없다 — 표식은 안 단다.
            return Ok(format!(
                "{{\"ok\":true,\"para\":{},\"pos\":{}}}",
                caret_para, caret
            ));
        }
        let Some((sec, para)) = root_para_location(self, target_in_list) else {
            // 마지막 문단 끝에서 걸면 다음 문단이 없다 — 아직 다루지 않는다.
            return Ok(r#"{"ok":false,"reason":"대상 문단이 없다"}"#.to_string());
        };
        let target = self
            .document
            .sections
            .get_mut(sec)
            .and_then(|s| s.paragraphs.get_mut(para))
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", target_in_list)))?;
        match kind {
            "page" => target.column_type = ColumnBreakType::Page,
            "column" => target.column_type = ColumnBreakType::Column,
            // 표식 컨트롤은 글자를 안 남기고 `char_count` 로만 8칸을 센다(모델 규약).
            "colDef" => {
                target.column_type = ColumnBreakType::MultiColumn;
                target
                    .controls
                    .insert(0, Control::ColumnDef(ColumnDef::default()));
                target.char_count += EXTENDED_CTRL_UNITS as u32;
            }
            _ => {
                target.column_type = ColumnBreakType::Section;
                target
                    .controls
                    .insert(0, Control::ColumnDef(ColumnDef::default()));
                target
                    .controls
                    .insert(0, Control::SectionDef(Box::default()));
                target.char_count += 2 * EXTENDED_CTRL_UNITS as u32;
            }
        }
        // 표식은 문단 레코드에 얹히므로 그 구역의 원본 스트림을 버려야 저장에 실린다.
        if let Some(section) = self.document.sections.get_mut(sec) {
            section.raw_stream = None;
        }
        Ok(format!(
            "{{\"ok\":true,\"para\":{},\"pos\":{}}}",
            caret_para, caret
        ))
    }

    /// 자동 번호를 캐럿 자리에 끼운다 — 웹한글컨트롤 `InsertPageNum`·`InsertCpNo`·`InsertTpNo`.
    ///
    /// 셋 다 사슬에 `atno` 하나를 더하고 스트림에서 **8칸**을 차지한다(실측: 문단 끝 7 → 15).
    /// 갈래는 `page`(쪽 번호)·`current`(현재 쪽)·`total`(전체 쪽수)이고, 컨트롤 아이디로는
    /// 안 갈린다 — 셋이 같은 `atno` 다.
    ///
    /// 파서가 이 컨트롤에 **자리표 글자 한 칸**을 남기므로(`parse_para_text` 의 `0x0012` 가지)
    /// 여기서도 그 한 칸을 함께 넣는다. 그래야 저장·조판이 파일에서 온 문서와 같은 꼴이 된다.
    pub fn insert_auto_number_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        pos: usize,
        kind: &str,
    ) -> Result<String, HwpError> {
        let number_type = match kind {
            "page" | "current" => AutoNumberType::Page,
            "total" => AutoNumberType::TotalPage,
            other => {
                return Err(HwpError::InvalidField(format!(
                    "모르는 번호 갈래 {}",
                    other
                )))
            }
        };
        if list_id != ROOT_LIST_ID {
            return Ok(r#"{"ok":false,"reason":"본문 밖은 아직 다루지 않는다"}"#.to_string());
        }
        let char_idx = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            char_idx_at_stream_pos(para, pos).min(para.text.chars().count())
        };
        let (sec, para_idx) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::InvalidField(format!("문단 {} 없음", para_idx)))?;

        // 컨트롤은 문단 안에서 **글자 차례대로** 놓인다 — 앞선 컨트롤 수가 끼울 자리다.
        let control_index = para
            .control_text_positions()
            .iter()
            .filter(|p| **p < char_idx)
            .count();
        // 자리표 글자는 문단이 스스로 넣게 둔다 — `char_offsets`·`char_shapes`·`line_segs` 를
        // 함께 갱신해 준다. 여기서 지우거나 직접 만지면 `control_text_positions` 가 이 컨트롤을
        // 문단 맨 앞으로 오해해 캐럿 클램프까지 어긋난다(실제로 한 번 그랬다).
        para.insert_text_at(char_idx, " ");
        para.controls.insert(
            control_index,
            Control::AutoNumber(AutoNumber {
                number_type,
                ..Default::default()
            }),
        );
        // 글자 한 칸은 `insert_text_at` 이 이미 셌다 — 컨트롤 몫 일곱만 더한다(합 8칸).
        para.char_count += (EXTENDED_CTRL_UNITS - 1) as u32;
        section.raw_stream = None;
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 구역마다 **첫 본문 문단 번호** — `MoveSectionUp`·`MoveSectionDown` 이 딛는 자리다.
    ///
    /// 본문 리스트는 구역을 가로질러 이어지므로([`root_para_location`]) 구역 경계는 이 표로만
    /// 안다. 구역이 셋인 문서면 `[0, 8, 15]` 꼴이다.
    ///
    /// 경계는 `document.sections` 의 칸막이가 아니라 **문단이 진 구역 표식**으로 센다.
    /// `BreakSection` 이 만든 구역은 문단에 `SectionDef` 를 얹을 뿐 `sections` 를 안 가르는데,
    /// 한글은 그 문단부터 새 구역으로 본다(실측: 나눈 뒤 `MoveSectionDown` 이 그 문단을 짚는다).
    /// 칸막이로 세면 방금 만든 구역이 안 보인다.
    pub fn section_starts_json(&self) -> String {
        let mut starts: Vec<String> = Vec::new();
        let mut para_in_body = 0usize;
        for section in self.document.sections.iter() {
            for para in section.paragraphs.iter() {
                let marks_section = para_in_body == 0
                    || matches!(para.controls.first(), Some(Control::SectionDef(_)));
                if marks_section {
                    starts.push(para_in_body.to_string());
                }
                para_in_body += 1;
            }
        }
        format!("[{}]", starts.join(","))
    }

    /// 빈 문단 하나를 그 자리에 끼운다 — 나누기가 표식을 놓을 자리를 만들 때 쓴다.
    ///
    /// 서식은 **뒤 이웃**에서 물려받는다. 나누기로 생기는 문단은 뒤따르는 글의 앞머리라
    /// 그쪽을 닮는 것이 맞다.
    fn insert_empty_paragraph_at(
        &mut self,
        list_id: u32,
        para_in_list: usize,
    ) -> Result<(), HwpError> {
        if list_id != ROOT_LIST_ID {
            return Err(HwpError::InvalidField(
                "본문 밖에는 아직 문단을 끼우지 않는다".into(),
            ));
        }
        let (sec, para) = root_para_location(self, para_in_list)
            .ok_or_else(|| HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list)))?;
        let section = self
            .document
            .sections
            .get_mut(sec)
            .ok_or_else(|| HwpError::InvalidField(format!("구역 {} 없음", sec)))?;
        let fresh = match section.paragraphs.get(para) {
            Some(neighbor) => Paragraph::new_empty_like(neighbor),
            None => Paragraph::new_empty(),
        };
        section.paragraphs.insert(para, fresh);
        section.raw_stream = None;
        Ok(())
    }

    /// 커서 좌표(list/para/pos)로 글자를 지운다 — 웹한글컨트롤 `Run("Delete*")` 용.
    ///
    /// [`apply_char_format_at_cursor`](Self::apply_char_format_at_cursor) 와 같은 자를 쓴다 —
    /// 인자는 코드 유닛이고 여기서 글자 번호로 옮긴다. 빈 범위면 아무 일도 하지 않는다.
    pub fn delete_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        start_pos: usize,
        end_pos: usize,
    ) -> Result<String, HwpError> {
        let (start_char, end_char) = {
            let para = self
                .cursor_paragraph_ref(list_id, para_in_list)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let last = para.text.chars().count();
            (
                char_idx_at_stream_pos(para, start_pos).min(last),
                char_idx_at_stream_pos(para, end_pos).min(last),
            )
        };
        if start_char >= end_char {
            return Ok(r#"{"ok":false,"reason":"빈 범위"}"#.to_string());
        }

        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.delete_text_native(sec, para, start_char, end_char - start_char);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.delete_range_in_cell_by_path(
            section_index,
            host_para,
            &path,
            para_in_list,
            start_char,
            para_in_list,
            end_char,
        )
    }

    /// 커서가 든 셀을 기준으로 표를 고친다 — 웹한글컨트롤 `Run("TableInsert*"·"TableDelete*")`.
    ///
    /// 리스트 아이디만 주면 구역·문단·컨트롤·행·열을 여기서 풀어 준다. 캐럿을 어디로 옮길지는
    /// 호출 측(호환 층)이 정한다 — 표가 바뀐 **뒤의** 격자를 봐야 알 수 있기 때문이다.
    ///
    /// 중첩 표는 아직 다루지 않는다. 아래 표 편집 API 가 `(구역, 문단, 컨트롤)` 세 값만 받아서
    /// 셀 안의 표까지 짚지 못한다.
    pub fn table_edit_at_cursor(&mut self, list_id: u32, op: &str) -> Result<String, HwpError> {
        let (section, host_para, control_index, row, col) = {
            let (_, lists) = self.collect_fields_and_lists();
            let entry = lists
                .iter()
                .find(|l| l.list_id == list_id)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let grid = entry
                .grid
                .ok_or_else(|| HwpError::InvalidField("표 셀이 아니다".into()))?;
            if entry.host_list_id != ROOT_LIST_ID {
                return Ok(r#"{"ok":false,"reason":"중첩 표는 아직 다루지 않는다"}"#.to_string());
            }
            (
                entry.section_index,
                entry.host_para_index,
                entry.control_index,
                grid.row,
                grid.col,
            )
        };
        match op {
            "insertRowAbove" => {
                self.insert_table_row_native(section, host_para, control_index, row, false)
            }
            // `TableAppendRow` 도 같은 자리에 끼운다 — 다른 것은 캐럿이 새 줄로 간다는 점뿐이고
            // 그 판단은 호환 층이 한다.
            "insertRowBelow" | "appendRow" | "appendRowAtEnd" => {
                self.insert_table_row_native(section, host_para, control_index, row, true)
            }
            "insertColLeft" => {
                self.insert_table_column_native(section, host_para, control_index, col, false)
            }
            "insertColRight" => {
                self.insert_table_column_native(section, host_para, control_index, col, true)
            }
            "deleteRow" => self.delete_table_row_native(section, host_para, control_index, row),
            "deleteCol" => self.delete_table_column_native(section, host_para, control_index, col),
            // 셀을 두 줄·두 칸으로 나눈다. 한글의 `TableSplitCellRow2`·`Col2` 는 대화상자 없이
            // 곧바로 반씩 나눈다(실측: 셀 하나가 늘고 캐럿은 제자리).
            "splitRow2" => self.split_table_cell_into_native(
                section,
                host_para,
                control_index,
                row,
                col,
                2,
                1,
                true,
                false,
            ),
            "splitCol2" => self.split_table_cell_into_native(
                section,
                host_para,
                control_index,
                row,
                col,
                1,
                2,
                true,
                false,
            ),
            _ => Err(HwpError::InvalidField(format!("모르는 표 편집 '{}'", op))),
        }
    }

    /// 커서가 든 셀에서 `(end_row, end_col)` 까지를 하나로 합친다 — `Run("TableMergeCell")`.
    ///
    /// 셀 블록의 범위는 호환 층이 들고 있다. 오라클에서 블록은 `GetSelectedPos` 로 안 보이니
    /// (글자 범위가 아니다) 이 층이 기억한 범위를 그대로 넘겨받는다.
    pub fn table_merge_at_cursor(
        &mut self,
        list_id: u32,
        end_row: u16,
        end_col: u16,
    ) -> Result<String, HwpError> {
        let (section, host_para, control_index, row, col) = {
            let (_, lists) = self.collect_fields_and_lists();
            let entry = lists
                .iter()
                .find(|l| l.list_id == list_id)
                .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
            let grid = entry
                .grid
                .ok_or_else(|| HwpError::InvalidField("표 셀이 아니다".into()))?;
            if entry.host_list_id != ROOT_LIST_ID {
                return Ok(r#"{"ok":false,"reason":"중첩 표는 아직 다루지 않는다"}"#.to_string());
            }
            (
                entry.section_index,
                entry.host_para_index,
                entry.control_index,
                grid.row,
                grid.col,
            )
        };
        self.merge_table_cells_native(
            section,
            host_para,
            control_index,
            row.min(end_row),
            col.min(end_col),
            row.max(end_row),
            col.max(end_col),
        )
    }

    /// 문단 하나의 캐럿 경계 — 웹한글컨트롤 `MoveParaBegin`·`MoveParaEnd` 가 가는 자리.
    ///
    /// `start` 는 앞머리 자리차지 컨트롤을 건너뛴 자리다(본문 첫 문단은 0 이 아니다).
    /// `end` 는 문단 부호를 뺀 코드 유닛 길이다. 없는 자리면 `{}`.
    pub fn para_bounds_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "{}".to_string();
        };
        format!(
            "{{\"start\":{},\"end\":{},\"selectStart\":{}}}",
            leading_anchor_pos(para),
            stream_len(para),
            select_start_pos(para),
        )
    }

    /// 줄이 시작하는 자리들 — `MoveLineBegin`·`MoveLineEnd` 가 딛는 값.
    ///
    /// `LineSeg::text_start` 는 파일이 그대로 준 **코드 유닛** 위치라 한글 좌표와 같은 자다.
    /// 옮길 것이 없다.
    pub fn line_starts_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let starts: Vec<String> = para
            .line_segs
            .iter()
            .map(|seg| seg.text_start.to_string())
            .collect();
        format!("[{}]", starts.join(","))
    }

    /// 지금 단어의 끝 — `MoveWordEnd` 가 가는 자리(다음 공백 글자의 자리).
    pub fn word_end_json(&self, list_id: u32, para_in_list: usize, pos: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "null".to_string();
        };
        word_end_from(para, pos).to_string()
    }

    /// 단어가 시작하는 자리들 — `MoveNextWord`·`MovePrevWord`·`MoveWordBegin/End` 가 딛는 눈금.
    pub fn word_starts_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let starts: Vec<String> = word_starts(para).iter().map(|p| p.to_string()).collect();
        format!("[{}]", starts.join(","))
    }

    /// 캐럿이 설 수 있는 자리들 — 한 글자 이동이 딛는 눈금(`MoveNextChar` 류).
    pub fn caret_stops_json(&self, list_id: u32, para_in_list: usize) -> String {
        let Some(para) = self.cursor_paragraph_ref(list_id, para_in_list) else {
            return "[]".to_string();
        };
        let stops: Vec<String> = caret_stops(para).iter().map(|p| p.to_string()).collect();
        format!("[{}]", stops.join(","))
    }

    /// 커서 좌표(list/para)로 문단 서식을 건다 — 웹한글컨트롤 `Run("ParagraphShape*")` 용.
    ///
    /// 문단 서식은 셀 경로가 깊으면 걸 수 없다 — 코어에 by-path 짝이 아직 없다. 그 경우
    /// 조용히 넘기지 않고 오류로 알린다.
    pub fn apply_para_format_at_cursor(
        &mut self,
        list_id: u32,
        para_in_list: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        if list_id == ROOT_LIST_ID {
            let (sec, para) = root_para_location(self, para_in_list).ok_or_else(|| {
                HwpError::InvalidField(format!("본문 문단 {} 없음", para_in_list))
            })?;
            return self.apply_para_format_native(sec, para, props_json);
        }
        let (_, lists) = self.collect_fields_and_lists();
        let entry = lists
            .iter()
            .find(|l| l.list_id == list_id)
            .ok_or_else(|| HwpError::InvalidField(format!("리스트 {} 없음", list_id)))?;
        let path = cell_path_to_list(&lists, list_id, para_in_list)
            .ok_or_else(|| HwpError::InvalidField("셀 경로를 세울 수 없음".into()))?;
        if path.len() != 1 {
            return Err(HwpError::InvalidField(
                "중첩 셀의 문단 서식은 아직 다루지 않는다".into(),
            ));
        }
        let (control_idx, cell_idx, cell_para_idx) = path[0];
        let section_index = entry.section_index;
        let host_para = root_para_of(&lists, entry);
        self.apply_para_format_in_cell_native(
            section_index,
            host_para,
            control_idx,
            cell_idx,
            cell_para_idx,
            props_json,
        )
    }

    /// 커서 좌표가 가리키는 문단 — 리스트 표를 한 번만 만든다.
    fn cursor_paragraph_ref(&self, list_id: u32, para_in_list: usize) -> Option<&Paragraph> {
        if list_id == ROOT_LIST_ID {
            // 본문은 구역을 가로질러 이어진다 — `root_para_location` 주석 참고.
            let (sec, para) = root_para_location(self, para_in_list)?;
            return self.document.sections.get(sec)?.paragraphs.get(para);
        }
        let (_, lists) = self.collect_fields_and_lists();
        cursor_paragraph(self, &lists, list_id, para_in_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Section;
    use crate::model::style::ParaShape;

    /// 문단 모양이 한글 코드값과 단위로 나가는지 — 이름만 같고 값이 rhwp 내부 표현이면
    /// 오라클과 어긋난다.
    #[test]
    fn para_shape_set_uses_hwp_codes() {
        let mut core = DocumentCore::new_empty();
        core.document.doc_info.para_shapes = vec![ParaShape {
            alignment: Alignment::Center,
            line_spacing_type: LineSpacingType::Percent,
            line_spacing: 160,
            margin_left: 100,
            head_type: HeadType::Outline,
            para_level: 2,
            ..Default::default()
        }];
        core.document.sections.push(Section {
            paragraphs: vec![Paragraph {
                para_shape_id: 0,
                ..Default::default()
            }],
            ..Default::default()
        });

        let json = core.para_shape_set_json(ROOT_LIST_ID, 0);

        // 가운데 정렬은 3, 글자에 따라(%)는 0 — rhwp 열거형 순서(Justify=0, Center=3)와
        // 우연히 같은 자리가 아니라 한글 코드표를 따른 값이다.
        assert!(json.contains("\"AlignType\":3"), "{json}");
        assert!(json.contains("\"LineSpacingType\":0"), "{json}");
        assert!(json.contains("\"LineSpacing\":160"), "{json}");
        assert!(json.contains("\"LeftMargin\":100"), "{json}");
        assert!(json.contains("\"HeadingType\":1"), "{json}");
        assert!(json.contains("\"Level\":2"), "{json}");
    }

    /// 없는 자리를 물으면 빈 셋이다 — 0 으로 채우면 "모른다"와 "0이다"가 뭉개진다.
    #[test]
    fn missing_cursor_gives_empty_set() {
        let core = DocumentCore::new_empty();
        assert_eq!(core.para_shape_set_json(ROOT_LIST_ID, 99), "{}");
        assert_eq!(core.char_shape_set_json(ROOT_LIST_ID, 99, 0), "{}");
    }
}
