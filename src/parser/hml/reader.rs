use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::model::page::PageDef;
use crate::model::paragraph::{CharShapeRef, ColumnBreakType};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, ShapeComponentAttr, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::{
    border_width_index, Alignment, BorderFill, BorderLineType, CharShape, Fill, FillType, Font,
    LineSpacingType, ParaShape, ShapeBorderLine, SolidFill, Style, TabDef,
};
use crate::model::Padding;

use super::envelope::PreservedFragment;
use super::error::HmlError;
use super::warnings::{HmlWarning, HmlWarningCode};

const LANGUAGE_NAMES: [&str; 7] = [
    "Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User",
];

const MAX_EQUATION_DIAGNOSTIC_CHARS: usize = 256;

fn bounded_equation_semantics(name: &str, value: &str) -> String {
    let semantics = format!("{name}={value}");
    if semantics.chars().count() <= MAX_EQUATION_DIAGNOSTIC_CHARS {
        return semantics;
    }
    let mut bounded = semantics
        .chars()
        .take(MAX_EQUATION_DIAGNOSTIC_CHARS - 1)
        .collect::<String>();
    bounded.push('…');
    bounded
}

#[derive(Debug, Clone)]
pub struct HmlLimits {
    pub max_xml_bytes: usize,
    pub max_depth: usize,
    pub max_attributes: usize,
    pub max_text_node_bytes: usize,
}

impl Default for HmlLimits {
    fn default() -> Self {
        Self {
            max_xml_bytes: 100 * 1024 * 1024,
            max_depth: 256,
            max_attributes: 256,
            max_text_node_bytes: 8 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct HmlSource {
    pub version: String,
    pub sub_version: Option<String>,
    pub style: Option<String>,
    pub resource_count: usize,
    pub warnings: Vec<HmlWarning>,
    pub preserved_fragments: Vec<PreservedFragment>,
    pub font_faces: Vec<Vec<Font>>,
    pub border_fills: Vec<BorderFill>,
    pub char_shapes: Vec<CharShape>,
    pub para_shapes: Vec<ParaShape>,
    pub tab_defs: Vec<TabDef>,
    pub styles: Vec<Style>,
    pub sections: Vec<HmlSection>,
}

#[derive(Debug, Default)]
pub(crate) struct HmlSection {
    pub page_def: Option<PageDef>,
    pub paragraphs: Vec<HmlParagraph>,
}

#[derive(Debug, Default)]
pub(crate) struct HmlParagraph {
    pub para_shape_id: u16,
    pub style_id: u8,
    pub column_type: ColumnBreakType,
    pub raw_break_type: u8,
    pub text: String,
    pub char_offsets: Vec<u32>,
    pub char_shapes: Vec<CharShapeRef>,
    pub controls: Vec<HmlControl>,
    pub raw_pos: u32,
}

#[derive(Debug)]
pub(crate) enum HmlControl {
    Equation(HmlEquation),
    Rectangle(HmlRectangle),
    Table(HmlTable),
}

#[derive(Debug, Default)]
pub(crate) struct HmlEquation {
    pub script: String,
    pub font_size: u32,
    pub color: u32,
    pub baseline: i16,
    pub version_info: String,
    pub font_name: String,
    script_count: usize,
}

#[derive(Debug, Default)]
pub(crate) struct HmlRectangle {
    pub width: u32,
    pub height: u32,
    pub x_coords: [i32; 4],
    pub y_coords: [i32; 4],
    pub horizontal_offset: i32,
    pub vertical_offset: i32,
    pub treat_as_char: bool,
    pub flow_with_text: bool,
    pub allow_overlap: bool,
    pub vert_rel_to: VertRelTo,
    pub vert_align: VertAlign,
    pub horz_rel_to: HorzRelTo,
    pub horz_align: HorzAlign,
    pub text_wrap: TextWrap,
    pub shape_attr: ShapeComponentAttr,
    pub border_line: ShapeBorderLine,
    pub fill: Fill,
    pub text_margin: Padding,
    pub text_box: Vec<HmlParagraph>,
}

#[derive(Debug, Default)]
pub(crate) struct HmlTable {
    pub common: CommonObjAttr,
    pub row_count: u16,
    pub col_count: u16,
    pub cell_spacing: i16,
    pub border_fill_id: u16,
    pub padding: Padding,
    pub cells: Vec<HmlCell>,
}

#[derive(Debug, Default)]
pub(crate) struct HmlCell {
    pub col: u16,
    pub row: u16,
    pub col_span: u16,
    pub row_span: u16,
    pub width: u32,
    pub height: u32,
    pub padding: Padding,
    pub border_fill_id: u16,
    pub paragraphs: Vec<HmlParagraph>,
}

/// 캡처 시작 지점을 기록해두었다가 대응하는 종료 태그에서 원문을 잘라내기 위한 대기 상태.
struct PendingCapture {
    /// `end()`가 이 서브트리를 닫을 때 스택 깊이가 이 값과 같아야 한다 (push 직후 깊이).
    target_depth: usize,
    /// 시작 태그의 첫 바이트 오프셋 (디코딩된 XML 텍스트 기준).
    start_offset: u64,
    parent: String,
    xml_path: String,
    modeled_siblings_before: usize,
}

struct ReadState<'a> {
    xml: &'a str,
    stack: Vec<String>,
    source: HmlSource,
    pending_capture: Option<PendingCapture>,
    paragraphs: Vec<HmlParagraph>,
    equations: Vec<HmlEquation>,
    accepted_script_depth: Option<usize>,
    rectangles: Vec<HmlRectangle>,
    tables: Vec<HmlTable>,
    cells: Vec<HmlCell>,
    font_language: Option<usize>,
    current_border_fill: Option<usize>,
    head_modeled_children: usize,
    body_modeled_children: usize,
    saw_head: bool,
    saw_body: bool,
}

impl<'a> ReadState<'a> {
    fn new(xml: &'a str) -> Self {
        Self {
            xml,
            stack: Vec::new(),
            source: HmlSource::default(),
            pending_capture: None,
            paragraphs: Vec::new(),
            equations: Vec::new(),
            accepted_script_depth: None,
            rectangles: Vec::new(),
            tables: Vec::new(),
            cells: Vec::new(),
            font_language: None,
            current_border_fill: None,
            head_modeled_children: 0,
            body_modeled_children: 0,
            saw_head: false,
            saw_body: false,
        }
    }

    fn start(
        &mut self,
        element: &BytesStart<'_>,
        limits: &HmlLimits,
        start_pos: u64,
    ) -> Result<(), HmlError> {
        let name = element_name(element)?;
        validate_attributes(element, limits.max_attributes)?;
        if self.pending_capture.is_some() {
            self.stack.push(name);
            return Ok(());
        }
        let modeled_siblings_before = self.modeled_siblings_before();
        let preserve_target = self.warn_if_unsupported(&name, element)?;
        self.note_modeled_child(&name);
        if self.is_unsupported_inline(&name) {
            self.reserve_control_slot()?;
        }
        self.stack.push(name.clone());
        if let Some((parent, xml_path)) = preserve_target {
            self.pending_capture = Some(PendingCapture {
                target_depth: self.stack.len(),
                start_offset: start_pos,
                parent,
                xml_path,
                modeled_siblings_before,
            });
        }
        self.capture_start(&name, element)
    }

    fn empty(
        &mut self,
        element: &BytesStart<'_>,
        limits: &HmlLimits,
        start_pos: u64,
        end_pos: u64,
    ) -> Result<(), HmlError> {
        let name = element_name(element)?;
        validate_attributes(element, limits.max_attributes)?;
        if self.pending_capture.is_some() {
            self.stack.push(name);
            self.stack.pop();
            return Ok(());
        }
        let modeled_siblings_before = self.modeled_siblings_before();
        let preserve_target = self.warn_if_unsupported(&name, element)?;
        self.note_modeled_child(&name);
        if self.is_unsupported_inline(&name) {
            self.reserve_control_slot()?;
        }
        self.stack.push(name.clone());
        if let Some((parent, xml_path)) = preserve_target {
            self.push_preserved_fragment(
                parent,
                xml_path,
                modeled_siblings_before,
                start_pos,
                end_pos,
            );
        }
        self.capture_start(&name, element)?;
        self.finish_element(&name)?;
        self.stack.pop();
        Ok(())
    }

    fn end(&mut self, name: &[u8], end_pos: u64) -> Result<(), HmlError> {
        let actual = std::str::from_utf8(name)
            .map_err(|_| HmlError::InvalidXml("non-UTF-8 element name".to_string()))?;
        let expected = self
            .stack
            .last()
            .ok_or_else(|| HmlError::InvalidXml("unexpected closing element".to_string()))?;
        if actual != expected {
            return Err(HmlError::InvalidXml(format!(
                "closing element {actual} does not match {expected}"
            )));
        }
        if self
            .pending_capture
            .as_ref()
            .is_some_and(|pending| pending.target_depth < self.stack.len())
        {
            self.stack.pop();
            return Ok(());
        }
        if self
            .pending_capture
            .as_ref()
            .is_some_and(|pending| pending.target_depth == self.stack.len())
        {
            let pending = self
                .pending_capture
                .take()
                .expect("pending_capture presence checked above");
            self.push_preserved_fragment(
                pending.parent,
                pending.xml_path,
                pending.modeled_siblings_before,
                pending.start_offset,
                end_pos,
            );
        }
        self.finish_element(actual)?;
        self.stack.pop();
        Ok(())
    }

    /// 부모/경로/바이트 구간으로부터 보존 캡슐 하나를 만들어 저장한다.
    fn push_preserved_fragment(
        &mut self,
        parent: String,
        xml_path: String,
        modeled_siblings_before: usize,
        start_offset: u64,
        end_offset: u64,
    ) {
        let order = self
            .source
            .preserved_fragments
            .iter()
            .filter(|fragment| fragment.parent == parent)
            .count();
        let raw_xml = self.xml[start_offset as usize..end_offset as usize].to_string();
        self.source.preserved_fragments.push(PreservedFragment {
            parent,
            order,
            modeled_siblings_before,
            xml_path,
            raw_xml,
        });
    }

    fn modeled_siblings_before(&self) -> usize {
        match self.stack.last().map(String::as_str) {
            Some("HEAD") => self.head_modeled_children,
            Some("BODY") => self.body_modeled_children,
            _ => 0,
        }
    }

    fn note_modeled_child(&mut self, name: &str) {
        if self.stack.last().map(String::as_str) == Some("HEAD") && name == "MAPPINGTABLE" {
            self.head_modeled_children += 1;
        }
        if self.stack.last().map(String::as_str) == Some("BODY") && name == "SECTION" {
            self.body_modeled_children += 1;
        }
    }

    fn capture_start(&mut self, name: &str, element: &BytesStart<'_>) -> Result<(), HmlError> {
        match name {
            "HWPML" => self.capture_root(element),
            "HEAD" if self.stack.len() == 2 => {
                self.saw_head = true;
                Ok(())
            }
            "BODY" if self.stack.len() == 2 => {
                self.saw_body = true;
                Ok(())
            }
            "SECTION" => {
                self.source.sections.push(HmlSection::default());
                Ok(())
            }
            "FONTFACE" => self.start_font_face(element),
            "FONT" if self.font_language.is_some() => self.capture_font(element),
            "BORDERFILL" => self.capture_border_fill(element),
            "LEFTBORDER" | "RIGHTBORDER" | "TOPBORDER" | "BOTTOMBORDER" => {
                self.capture_border_line(name, element)
            }
            "CHARSHAPE" => self.capture_char_shape(element),
            "FONTID" | "RATIO" | "CHARSPACING" | "RELSIZE" | "CHAROFFSET" => {
                self.capture_char_shape_array(name, element)
            }
            "PARASHAPE" => self.capture_para_shape(element),
            "PARAMARGIN" => self.capture_para_margin(element),
            "PARABORDER" => self.capture_para_border(element),
            "TABDEF" => self.capture_tab_def(element),
            "STYLE" => self.capture_style(element),
            "PAGEDEF" => self.capture_page_def(element),
            "PAGEMARGIN" => self.capture_page_margin(element),
            "P" => self.start_paragraph(element),
            "TEXT" => self.start_text_run(element),
            "EQUATION" if self.stack.iter().rev().nth(1).map(String::as_str) == Some("TEXT") => {
                self.start_equation(element)
            }
            "SCRIPT" => self.start_equation_script(element),
            "RECTANGLE" => self.start_rectangle(element),
            "SHAPEOBJECT" => self.capture_shape_object(element),
            "SHAPECOMPONENT" => self.capture_shape_component(element),
            "LINESHAPE" => self.capture_line_shape(element),
            "WINDOWBRUSH" => self.capture_window_brush(element),
            "TEXTMARGIN" => self.capture_text_margin(element),
            "TABLE" => self.start_table(element),
            "CELL" => self.start_cell(element),
            "SIZE" => self.capture_object_size(element),
            "POSITION" => self.capture_object_position(element),
            "INSIDEMARGIN" => self.capture_table_padding(element),
            "CELLMARGIN" => self.capture_cell_padding(element),
            "BINDATA" => {
                self.source.resource_count += 1;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn finish_element(&mut self, name: &str) -> Result<(), HmlError> {
        match name {
            "FONTFACE" => self.font_language = None,
            "BORDERFILL" => self.current_border_fill = None,
            "P" => self.finish_paragraph()?,
            "EQUATION" if self.stack.iter().rev().nth(1).map(String::as_str) == Some("TEXT") => {
                self.finish_equation()?
            }
            "SCRIPT" => self.finish_equation_script(),
            "RECTANGLE" => self.finish_rectangle()?,
            "CELL" => self.finish_cell()?,
            "TABLE" => self.finish_table()?,
            _ => {}
        }
        Ok(())
    }

    fn capture_root(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if self.stack.len() != 1 || !self.source.version.is_empty() {
            return Err(HmlError::InvalidXml("multiple HWPML roots".to_string()));
        }
        let version = attribute(element, b"Version")?.unwrap_or_default();
        if !matches!(version.as_str(), "2.9" | "2.91") {
            return Err(HmlError::UnsupportedVersion(version));
        }
        self.source.version = version;
        self.source.sub_version = attribute(element, b"SubVersion")?;
        self.source.style = attribute(element, b"Style")?;
        Ok(())
    }

    fn start_font_face(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let lang = attribute(element, b"Lang")?.unwrap_or_default();
        self.font_language = LANGUAGE_NAMES
            .iter()
            .position(|candidate| *candidate == lang);
        if self.font_language.is_some() && self.source.font_faces.len() < 7 {
            self.source.font_faces.resize_with(7, Vec::new);
        }
        Ok(())
    }

    fn capture_font(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let Some(language) = self.font_language else {
            return Ok(());
        };
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(0);
        let font = Font {
            name: attribute(element, b"Name")?.unwrap_or_default(),
            alt_type: match attribute(element, b"Type")?.as_deref() {
                Some("ttf") => 1,
                Some("hft") => 2,
                _ => 0,
            },
            ..Default::default()
        };
        set_indexed(&mut self.source.font_faces[language], id, font);
        Ok(())
    }

    fn capture_border_fill(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(1);
        if id == 0 {
            return Err(HmlError::InvalidReference("BORDERFILL Id=0".to_string()));
        }
        set_indexed(&mut self.source.border_fills, id - 1, BorderFill::default());
        self.current_border_fill = Some(id - 1);
        Ok(())
    }

    fn capture_border_line(
        &mut self,
        name: &str,
        element: &BytesStart<'_>,
    ) -> Result<(), HmlError> {
        if self.stack.iter().rev().nth(1).map(String::as_str) != Some("BORDERFILL") {
            return Ok(());
        }
        let Some(border_fill_index) = self.current_border_fill else {
            return Ok(());
        };
        let side = match name {
            "LEFTBORDER" => 0,
            "RIGHTBORDER" => 1,
            "TOPBORDER" => 2,
            "BOTTOMBORDER" => 3,
            _ => return Ok(()),
        };
        let line_type = match attribute(element, b"Type")?.as_deref() {
            Some("None") => BorderLineType::None,
            Some("Solid") | None => BorderLineType::Solid,
            Some(value) => {
                self.source.warnings.push(HmlWarning::unsupported_attribute(
                    format!("/{}", self.stack.join("/")),
                    &format!("Type={value}"),
                ));
                BorderLineType::Solid
            }
        };
        let width = parse_border_width(element)?;
        let border_fill = self
            .source
            .border_fills
            .get_mut(border_fill_index)
            .ok_or_else(|| {
                HmlError::InvalidReference(format!("BORDERFILL index {}", border_fill_index + 1))
            })?;
        border_fill.borders[side].line_type = line_type;
        border_fill.borders[side].width = width;
        Ok(())
    }

    fn capture_char_shape(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(0);
        let shape = CharShape {
            base_size: parse_attribute(element, b"Height")?.unwrap_or(1000),
            border_fill_id: parse_attribute(element, b"BorderFillId")?.unwrap_or(0),
            text_color: parse_attribute(element, b"TextColor")?.unwrap_or(0),
            shade_color: parse_attribute(element, b"ShadeColor")?.unwrap_or(0),
            ..Default::default()
        };
        set_indexed(&mut self.source.char_shapes, id, shape);
        Ok(())
    }

    fn capture_char_shape_array(
        &mut self,
        name: &str,
        element: &BytesStart<'_>,
    ) -> Result<(), HmlError> {
        let Some(shape) = self.source.char_shapes.last_mut() else {
            return Ok(());
        };
        match name {
            "FONTID" => shape.font_ids = language_array(element)?,
            "RATIO" => shape.ratios = language_array(element)?,
            "CHARSPACING" => shape.spacings = language_array(element)?,
            "RELSIZE" => shape.relative_sizes = language_array(element)?,
            "CHAROFFSET" => shape.char_offsets = language_array(element)?,
            _ => {}
        }
        Ok(())
    }

    fn capture_para_shape(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(0);
        let shape = ParaShape {
            alignment: parse_alignment(attribute(element, b"Align")?.as_deref()),
            tab_def_id: parse_attribute(element, b"TabDef")?.unwrap_or(0),
            para_level: parse_attribute(element, b"Level")?.unwrap_or(0),
            ..Default::default()
        };
        set_indexed(&mut self.source.para_shapes, id, shape);
        Ok(())
    }

    fn capture_para_margin(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let Some(shape) = self.source.para_shapes.last_mut() else {
            return Ok(());
        };
        shape.margin_left = parse_attribute(element, b"Left")?.unwrap_or(0);
        shape.margin_right = parse_attribute(element, b"Right")?.unwrap_or(0);
        shape.indent = parse_attribute(element, b"Indent")?.unwrap_or(0);
        shape.spacing_before = parse_attribute(element, b"Prev")?.unwrap_or(0);
        shape.spacing_after = parse_attribute(element, b"Next")?.unwrap_or(0);
        shape.line_spacing = parse_attribute(element, b"LineSpacing")?.unwrap_or(160);
        shape.line_spacing_type = match attribute(element, b"LineSpacingType")?.as_deref() {
            Some("Fixed") => LineSpacingType::Fixed,
            Some("BetweenLines") => LineSpacingType::SpaceOnly,
            Some("AtLeast") => LineSpacingType::Minimum,
            _ => LineSpacingType::Percent,
        };
        Ok(())
    }

    fn capture_para_border(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(shape) = self.source.para_shapes.last_mut() {
            shape.border_fill_id = parse_attribute(element, b"BorderFill")?.unwrap_or(0);
        }
        Ok(())
    }

    fn capture_tab_def(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(0);
        set_indexed(&mut self.source.tab_defs, id, TabDef::default());
        Ok(())
    }

    fn capture_style(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let id = parse_attribute::<usize>(element, b"Id")?.unwrap_or(0);
        let style = Style {
            local_name: attribute(element, b"Name")?.unwrap_or_default(),
            english_name: attribute(element, b"EngName")?.unwrap_or_default(),
            style_type: u8::from(attribute(element, b"Type")?.as_deref() == Some("Char")),
            next_style_id: parse_attribute(element, b"NextStyle")?.unwrap_or(0),
            lang_id: parse_attribute(element, b"LangId")?.unwrap_or(1042),
            para_shape_id: parse_attribute(element, b"ParaShape")?.unwrap_or(0),
            char_shape_id: parse_attribute(element, b"CharShape")?.unwrap_or(0),
            ..Default::default()
        };
        set_indexed(&mut self.source.styles, id, style);
        Ok(())
    }

    fn capture_page_def(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let page = self.current_page_def()?;
        page.width = parse_attribute(element, b"Width")?.unwrap_or(page.width);
        page.height = parse_attribute(element, b"Height")?.unwrap_or(page.height);
        page.landscape = parse_attribute::<u8>(element, b"Landscape")?.unwrap_or(0) != 0;
        Ok(())
    }

    fn capture_page_margin(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let page = self.current_page_def()?;
        page.margin_left = parse_attribute(element, b"Left")?.unwrap_or(page.margin_left);
        page.margin_right = parse_attribute(element, b"Right")?.unwrap_or(page.margin_right);
        page.margin_top = parse_attribute(element, b"Top")?.unwrap_or(page.margin_top);
        page.margin_bottom = parse_attribute(element, b"Bottom")?.unwrap_or(page.margin_bottom);
        page.margin_header = parse_attribute(element, b"Header")?.unwrap_or(page.margin_header);
        page.margin_footer = parse_attribute(element, b"Footer")?.unwrap_or(page.margin_footer);
        page.margin_gutter = parse_attribute(element, b"Gutter")?.unwrap_or(page.margin_gutter);
        Ok(())
    }

    fn current_page_def(&mut self) -> Result<&mut PageDef, HmlError> {
        let section = self
            .source
            .sections
            .last_mut()
            .ok_or_else(|| HmlError::InvalidXml("PAGEDEF outside SECTION".to_string()))?;
        Ok(section.page_def.get_or_insert_with(PageDef::a4_default))
    }

    fn start_paragraph(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let page_break = parse_bool_attribute(element, b"PageBreak")?;
        let column_break = parse_bool_attribute(element, b"ColumnBreak")?;
        let (column_type, raw_break_type) = if page_break {
            (ColumnBreakType::Page, 0x04)
        } else if column_break {
            (ColumnBreakType::Column, 0x08)
        } else {
            (ColumnBreakType::None, 0)
        };
        self.paragraphs.push(HmlParagraph {
            para_shape_id: parse_attribute(element, b"ParaShape")?.unwrap_or(0),
            style_id: parse_attribute(element, b"Style")?.unwrap_or(0),
            column_type,
            raw_break_type,
            ..Default::default()
        });
        Ok(())
    }

    fn start_text_run(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let char_shape_id = parse_attribute(element, b"CharShape")?.unwrap_or(0);
        let paragraph = self.current_paragraph()?;
        if paragraph
            .char_shapes
            .last()
            .is_none_or(|last| last.char_shape_id != char_shape_id)
        {
            paragraph.char_shapes.push(CharShapeRef {
                start_pos: paragraph.raw_pos,
                char_shape_id,
            });
        }
        Ok(())
    }

    fn start_equation(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        self.reserve_control_slot()?;
        let path = format!("/{}", self.stack.join("/"));
        for item in element.attributes() {
            let attr = item.map_err(|error| HmlError::InvalidXml(error.to_string()))?;
            let name = std::str::from_utf8(attr.key.as_ref())
                .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute name".to_string()))?;
            if !matches!(
                name,
                "BaseLine" | "BaseUnit" | "TextColor" | "Version" | "Font"
            ) {
                let raw = std::str::from_utf8(attr.value.as_ref())
                    .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute".to_string()))?;
                let value = quick_xml::escape::unescape(raw)
                    .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
                self.source
                    .warnings
                    .push(HmlWarning::unsupported_equation_semantics(
                        format!("{path}/@{name}"),
                        &bounded_equation_semantics(name, &value),
                    ));
            }
        }
        self.equations.push(HmlEquation {
            baseline: parse_attribute(element, b"BaseLine")?.unwrap_or(0),
            font_size: parse_attribute(element, b"BaseUnit")?.unwrap_or(1000),
            color: parse_attribute(element, b"TextColor")?.unwrap_or(0),
            version_info: attribute(element, b"Version")?.unwrap_or_default(),
            font_name: attribute(element, b"Font")?.unwrap_or_default(),
            ..Default::default()
        });
        Ok(())
    }

    fn start_equation_script(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if self.stack.iter().rev().nth(1).map(String::as_str) != Some("EQUATION") {
            return Ok(());
        }
        let equation = self
            .equations
            .last_mut()
            .ok_or_else(|| HmlError::InvalidXml("SCRIPT outside EQUATION".to_string()))?;
        equation.script_count += 1;
        if equation.script_count == 1 {
            self.accepted_script_depth = Some(self.stack.len());
        } else {
            self.source
                .warnings
                .push(HmlWarning::unsupported_equation_semantics(
                    format!("/{}[{}]", self.stack.join("/"), equation.script_count),
                    "SCRIPT",
                ));
        }
        let path = if equation.script_count == 1 {
            format!("/{}", self.stack.join("/"))
        } else {
            format!("/{}[{}]", self.stack.join("/"), equation.script_count)
        };
        for item in element.attributes() {
            let attr = item.map_err(|error| HmlError::InvalidXml(error.to_string()))?;
            let name = std::str::from_utf8(attr.key.as_ref())
                .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute name".to_string()))?;
            let raw = std::str::from_utf8(attr.value.as_ref())
                .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute".to_string()))?;
            let value = quick_xml::escape::unescape(raw)
                .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
            self.source
                .warnings
                .push(HmlWarning::unsupported_equation_semantics(
                    format!("{path}/@{name}"),
                    &bounded_equation_semantics(name, &value),
                ));
        }
        Ok(())
    }

    fn finish_equation_script(&mut self) {
        if self.accepted_script_depth == Some(self.stack.len()) {
            self.accepted_script_depth = None;
        }
    }

    fn start_rectangle(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        self.reserve_control_slot()?;
        let mut rectangle = HmlRectangle::default();
        for (index, key) in [b"X0", b"X1", b"X2", b"X3"].iter().enumerate() {
            rectangle.x_coords[index] = parse_attribute(element, *key)?.unwrap_or(0);
        }
        for (index, key) in [b"Y0", b"Y1", b"Y2", b"Y3"].iter().enumerate() {
            rectangle.y_coords[index] = parse_attribute(element, *key)?.unwrap_or(0);
        }
        self.rectangles.push(rectangle);
        Ok(())
    }

    fn start_table(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        self.reserve_control_slot()?;
        self.tables.push(HmlTable {
            row_count: parse_attribute(element, b"RowCount")?.unwrap_or(0),
            col_count: parse_attribute(element, b"ColCount")?.unwrap_or(0),
            cell_spacing: parse_attribute(element, b"CellSpacing")?.unwrap_or(0),
            border_fill_id: parse_attribute(element, b"BorderFill")?.unwrap_or(0),
            ..Default::default()
        });
        Ok(())
    }

    fn start_cell(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        self.cells.push(HmlCell {
            col: parse_attribute(element, b"ColAddr")?.unwrap_or(0),
            row: parse_attribute(element, b"RowAddr")?.unwrap_or(0),
            col_span: parse_attribute(element, b"ColSpan")?.unwrap_or(1),
            row_span: parse_attribute(element, b"RowSpan")?.unwrap_or(1),
            width: parse_attribute(element, b"Width")?.unwrap_or(0),
            height: parse_attribute(element, b"Height")?.unwrap_or(0),
            border_fill_id: parse_attribute(element, b"BorderFill")?.unwrap_or(0),
            ..Default::default()
        });
        Ok(())
    }

    fn capture_object_size(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if self.nearest_object_is_table() {
            if let Some(table) = self.tables.last_mut() {
                table.common.width = parse_attribute(element, b"Width")?.unwrap_or(0);
                table.common.height = parse_attribute(element, b"Height")?.unwrap_or(0);
            }
        } else if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.width = parse_attribute(element, b"Width")?.unwrap_or(0);
            rectangle.height = parse_attribute(element, b"Height")?.unwrap_or(0);
        }
        Ok(())
    }

    fn capture_object_position(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if self.nearest_object_is_table() {
            if let Some(table) = self.tables.last_mut() {
                table.common.horizontal_offset =
                    parse_attribute::<i32>(element, b"HorzOffset")?.unwrap_or(0) as u32;
                table.common.vertical_offset =
                    parse_attribute::<i32>(element, b"VertOffset")?.unwrap_or(0) as u32;
                table.common.treat_as_char = parse_bool_attribute(element, b"TreatAsChar")?;
                table.common.flow_with_text = parse_bool_attribute(element, b"FlowWithText")?;
                table.common.allow_overlap = parse_bool_attribute(element, b"AllowOverlap")?;
                table.common.horz_rel_to =
                    parse_horz_rel_to(attribute(element, b"HorzRelTo")?.as_deref());
                table.common.vert_rel_to =
                    parse_vert_rel_to(attribute(element, b"VertRelTo")?.as_deref());
                table.common.horz_align =
                    parse_horz_align(attribute(element, b"HorzAlign")?.as_deref());
                table.common.vert_align =
                    parse_vert_align(attribute(element, b"VertAlign")?.as_deref());
            }
        } else if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.horizontal_offset = parse_attribute(element, b"HorzOffset")?.unwrap_or(0);
            rectangle.vertical_offset = parse_attribute(element, b"VertOffset")?.unwrap_or(0);
            rectangle.treat_as_char = parse_bool_attribute(element, b"TreatAsChar")?;
            rectangle.flow_with_text = parse_bool_attribute(element, b"FlowWithText")?;
            rectangle.allow_overlap = parse_bool_attribute(element, b"AllowOverlap")?;
            rectangle.horz_rel_to = parse_horz_rel_to(attribute(element, b"HorzRelTo")?.as_deref());
            rectangle.vert_rel_to = parse_vert_rel_to(attribute(element, b"VertRelTo")?.as_deref());
            rectangle.horz_align = parse_horz_align(attribute(element, b"HorzAlign")?.as_deref());
            rectangle.vert_align = parse_vert_align(attribute(element, b"VertAlign")?.as_deref());
        }
        Ok(())
    }

    fn nearest_object_is_table(&self) -> bool {
        let rectangle = self.stack.iter().rposition(|name| name == "RECTANGLE");
        let table = self.stack.iter().rposition(|name| name == "TABLE");
        table.is_some_and(|table_index| rectangle.is_none_or(|rect_index| table_index > rect_index))
    }

    fn capture_shape_object(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.text_wrap = parse_text_wrap(attribute(element, b"TextWrap")?.as_deref());
        }
        Ok(())
    }

    fn capture_shape_component(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.shape_attr.offset_x = parse_attribute(element, b"XPos")?.unwrap_or(0);
            rectangle.shape_attr.offset_y = parse_attribute(element, b"YPos")?.unwrap_or(0);
            rectangle.shape_attr.original_width =
                parse_attribute(element, b"OriWidth")?.unwrap_or(0);
            rectangle.shape_attr.original_height =
                parse_attribute(element, b"OriHeight")?.unwrap_or(0);
            rectangle.shape_attr.current_width = parse_attribute(element, b"CurWidth")?
                .filter(|value| *value > 0)
                .unwrap_or(rectangle.shape_attr.original_width);
            rectangle.shape_attr.current_height = parse_attribute(element, b"CurHeight")?
                .filter(|value| *value > 0)
                .unwrap_or(rectangle.shape_attr.original_height);
            if rectangle.width == 0 {
                rectangle.width = rectangle.shape_attr.original_width;
            }
            if rectangle.height == 0 {
                rectangle.height = rectangle.shape_attr.original_height;
            }
        }
        Ok(())
    }

    fn capture_line_shape(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        let width = parse_attribute(element, b"Width")?.unwrap_or(0);
        let style = attribute(element, b"Style")?.unwrap_or_else(|| "Solid".to_string());
        let end_cap = attribute(element, b"EndCap")?.unwrap_or_else(|| "Flat".to_string());
        let alpha = parse_attribute::<u8>(element, b"Alpha")?.unwrap_or(0);
        let mut attr = 0u32;

        if style == "Solid" {
            attr |= 1;
        } else {
            self.source.warnings.push(HmlWarning::unsupported_attribute(
                format!("/{}", self.stack.join("/")),
                &format!("Style={style}"),
            ));
        }
        if end_cap == "Flat" {
            attr |= 1 << 6;
        } else {
            self.source.warnings.push(HmlWarning::unsupported_attribute(
                format!("/{}", self.stack.join("/")),
                &format!("EndCap={end_cap}"),
            ));
        }
        if alpha != 0 {
            self.source.warnings.push(HmlWarning::unsupported_attribute(
                format!("/{}", self.stack.join("/")),
                &format!("Alpha={alpha}"),
            ));
        }
        if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.border_line.width = width;
            rectangle.border_line.attr = attr;
        }
        Ok(())
    }

    fn capture_text_margin(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.text_margin = parse_padding(element)?;
        }
        Ok(())
    }

    fn capture_window_brush(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if self.rectangles.is_empty() {
            return Ok(());
        }
        let Some(background_color) = parse_attribute(element, b"FaceColor")? else {
            return Ok(());
        };
        let pattern_color = parse_attribute(element, b"HatchColor")?.unwrap_or(0);
        let alpha = parse_attribute(element, b"Alpha")?.unwrap_or(0);
        if let Some(hatch_style) = attribute(element, b"HatchStyle")? {
            self.source.warnings.push(HmlWarning::unsupported_attribute(
                format!("/{}", self.stack.join("/")),
                &format!("HatchStyle={hatch_style}"),
            ));
        }
        if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.fill = Fill {
                fill_type: FillType::Solid,
                solid: Some(SolidFill {
                    background_color,
                    pattern_color,
                    pattern_type: -1,
                }),
                alpha,
                ..Fill::default()
            };
        }
        Ok(())
    }

    fn capture_table_padding(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(table) = self.tables.last_mut() {
            table.padding = parse_padding(element)?;
        }
        Ok(())
    }

    fn capture_cell_padding(&mut self, element: &BytesStart<'_>) -> Result<(), HmlError> {
        if let Some(cell) = self.cells.last_mut() {
            cell.padding = parse_padding(element)?;
        }
        Ok(())
    }

    fn reserve_control_slot(&mut self) -> Result<(), HmlError> {
        self.current_paragraph()?.raw_pos += 8;
        Ok(())
    }

    fn finish_paragraph(&mut self) -> Result<(), HmlError> {
        let paragraph = self
            .paragraphs
            .pop()
            .ok_or_else(|| HmlError::InvalidXml("unexpected P end".to_string()))?;
        if let Some(cell) = self.cells.last_mut() {
            cell.paragraphs.push(paragraph);
        } else if let Some(rectangle) = self.rectangles.last_mut() {
            rectangle.text_box.push(paragraph);
        } else {
            self.source
                .sections
                .last_mut()
                .ok_or_else(|| HmlError::InvalidXml("P outside SECTION".to_string()))?
                .paragraphs
                .push(paragraph);
        }
        Ok(())
    }

    fn finish_rectangle(&mut self) -> Result<(), HmlError> {
        let rectangle = self
            .rectangles
            .pop()
            .ok_or_else(|| HmlError::InvalidXml("unexpected RECTANGLE end".to_string()))?;
        self.current_paragraph()?
            .controls
            .push(HmlControl::Rectangle(rectangle));
        Ok(())
    }

    fn finish_equation(&mut self) -> Result<(), HmlError> {
        let equation = self
            .equations
            .pop()
            .ok_or_else(|| HmlError::InvalidXml("unexpected EQUATION end".to_string()))?;
        self.current_paragraph()?
            .controls
            .push(HmlControl::Equation(equation));
        Ok(())
    }

    fn finish_cell(&mut self) -> Result<(), HmlError> {
        let cell = self
            .cells
            .pop()
            .ok_or_else(|| HmlError::InvalidXml("unexpected CELL end".to_string()))?;
        self.tables
            .last_mut()
            .ok_or_else(|| HmlError::InvalidXml("CELL outside TABLE".to_string()))?
            .cells
            .push(cell);
        Ok(())
    }

    fn finish_table(&mut self) -> Result<(), HmlError> {
        let table = self
            .tables
            .pop()
            .ok_or_else(|| HmlError::InvalidXml("unexpected TABLE end".to_string()))?;
        self.current_paragraph()?
            .controls
            .push(HmlControl::Table(table));
        Ok(())
    }

    fn append_text(&mut self, text: &str) -> Result<(), HmlError> {
        if self.pending_capture.is_some() {
            return Ok(());
        }
        if self.stack.last().map(String::as_str) == Some("SCRIPT")
            && self.accepted_script_depth == Some(self.stack.len())
        {
            self.equations
                .last_mut()
                .ok_or_else(|| HmlError::InvalidXml("SCRIPT outside EQUATION".to_string()))?
                .script
                .push_str(text);
            return Ok(());
        }
        let inside_equation = self.stack.iter().any(|item| item == "EQUATION");
        if inside_equation && !text.trim().is_empty() {
            self.append_unsupported_equation_text(text);
            return Ok(());
        }
        if self.stack.last().map(String::as_str) != Some("CHAR") {
            return Ok(());
        }
        let paragraph = self.current_paragraph()?;
        for character in text.chars() {
            paragraph.char_offsets.push(paragraph.raw_pos);
            paragraph.text.push(character);
            paragraph.raw_pos += character.len_utf16() as u32;
        }
        Ok(())
    }

    fn append_unsupported_equation_text(&mut self, text: &str) {
        const MESSAGE_PREFIX: &str = "보존할 수 없는 HML 수식 의미를 건너뛰었습니다: #text=";
        let path = format!("/{}/#text", self.stack.join("/"));
        if let Some(warning) = self.source.warnings.last_mut().filter(|warning| {
            warning.code == HmlWarningCode::UnsupportedEquationSemantics
                && warning.xml_path == path
                && warning.message.starts_with(MESSAGE_PREFIX)
        }) {
            let current = &warning.message[MESSAGE_PREFIX.len()..];
            if current.chars().count() + "#text=".chars().count() == MAX_EQUATION_DIAGNOSTIC_CHARS
                && current.ends_with('…')
            {
                return;
            }
            warning.message = HmlWarning::unsupported_equation_semantics(
                path,
                &bounded_equation_semantics("#text", &format!("{current}{text}")),
            )
            .message;
            return;
        }
        self.source
            .warnings
            .push(HmlWarning::unsupported_equation_semantics(
                path,
                &bounded_equation_semantics("#text", text),
            ));
    }

    fn current_paragraph(&mut self) -> Result<&mut HmlParagraph, HmlError> {
        self.paragraphs
            .last_mut()
            .ok_or_else(|| HmlError::InvalidXml("inline content outside P".to_string()))
    }

    /// 미지원 요소를 경고로 기록하고, 저장 시 원문 그대로 되돌릴 수 있는 대상이면
    /// `(부모 요소, xml_path)`를 반환한다.
    fn warn_if_unsupported(
        &mut self,
        name: &str,
        element: &BytesStart<'_>,
    ) -> Result<Option<(String, String)>, HmlError> {
        let parent = self.stack.last().map(String::as_str);
        let unknown_document_child = match parent {
            Some("HEAD") => !matches!(name, "DOCSUMMARY" | "DOCSETTING" | "MAPPINGTABLE"),
            Some("BODY") => name != "SECTION",
            Some("TAIL") => true,
            _ => false,
        };
        let unsupported_control = self.is_unsupported_inline(name);
        let unsupported_equation_child = self.stack.iter().any(|item| item == "EQUATION")
            && !(parent == Some("EQUATION") && name == "SCRIPT");
        let explicitly_unsupported = matches!(name, "PICTURE" | "BINDATA");
        if unknown_document_child
            || name == "SCRIPTCODE"
            || unsupported_control
            || unsupported_equation_child
            || explicitly_unsupported
        {
            let path = format!("/{}/{}", self.stack.join("/"), name);
            let preserved = unknown_document_child
                || (name == "SCRIPTCODE" && matches!(parent, Some("TAIL") | Some("HEAD")));
            let warning = if unsupported_equation_child {
                HmlWarning::unsupported_equation_semantics(path.clone(), name)
            } else {
                HmlWarning::unsupported_element(path.clone(), name, preserved)
            };
            self.source.warnings.push(warning);
            if unsupported_equation_child {
                for item in element.attributes() {
                    let attr = item.map_err(|error| HmlError::InvalidXml(error.to_string()))?;
                    let attr_name = std::str::from_utf8(attr.key.as_ref()).map_err(|_| {
                        HmlError::InvalidXml("non-UTF-8 attribute name".to_string())
                    })?;
                    let raw = std::str::from_utf8(attr.value.as_ref())
                        .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute".to_string()))?;
                    let value = quick_xml::escape::unescape(raw)
                        .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
                    self.source
                        .warnings
                        .push(HmlWarning::unsupported_equation_semantics(
                            format!("{path}/@{attr_name}"),
                            &bounded_equation_semantics(attr_name, &value),
                        ));
                }
            }
            if preserved {
                return Ok(Some((
                    parent
                        .expect("preserved implies parent present")
                        .to_string(),
                    path,
                )));
            }
        }
        Ok(None)
    }

    fn is_unsupported_inline(&self, name: &str) -> bool {
        self.stack.last().map(String::as_str) == Some("TEXT")
            && !matches!(
                name,
                "CHAR" | "SECDEF" | "COLDEF" | "EQUATION" | "RECTANGLE" | "TABLE"
            )
    }

    fn finish(mut self) -> Result<HmlSource, HmlError> {
        if !self.stack.is_empty() || self.source.version.is_empty() {
            return Err(HmlError::InvalidXml(
                "incomplete HWPML document".to_string(),
            ));
        }
        if !self.saw_head {
            return Err(HmlError::MissingHead);
        }
        if !self.saw_body {
            return Err(HmlError::MissingBody);
        }
        self.warn_invalid_references();
        Ok(self.source)
    }

    fn warn_invalid_references(&mut self) {
        let char_count = self.source.char_shapes.len();
        let para_count = self.source.para_shapes.len();
        let style_count = self.source.styles.len();
        for (section_index, section) in self.source.sections.iter().enumerate() {
            for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
                let path = format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]");
                if paragraph.para_shape_id as usize >= para_count && para_count != 0 {
                    self.source.warnings.push(HmlWarning::invalid_reference(
                        path.clone(),
                        format!("ParaShape {}", paragraph.para_shape_id),
                    ));
                }
                if paragraph.style_id as usize >= style_count && style_count != 0 {
                    self.source.warnings.push(HmlWarning::invalid_reference(
                        path.clone(),
                        format!("Style {}", paragraph.style_id),
                    ));
                }
                for reference in &paragraph.char_shapes {
                    if reference.char_shape_id as usize >= char_count && char_count != 0 {
                        self.source.warnings.push(HmlWarning::invalid_reference(
                            path.clone(),
                            format!("CharShape {}", reference.char_shape_id),
                        ));
                    }
                }
            }
        }
    }
}

pub(crate) fn has_hwpml_root(xml: &str) -> bool {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                return element.name().as_ref() == b"HWPML"
                    && attribute(&element, b"Version")
                        .ok()
                        .flatten()
                        .is_some_and(|version| !version.is_empty());
            }
            Ok(Event::Decl(_) | Event::Comment(_) | Event::PI(_)) => {}
            Ok(Event::Text(text)) if text.iter().all(|byte| byte.is_ascii_whitespace()) => {}
            Ok(Event::Eof) | Err(_) => return false,
            _ => return false,
        }
    }
}

pub(crate) fn read_hml(xml: &str, limits: &HmlLimits) -> Result<HmlSource, HmlError> {
    let mut reader = Reader::from_str(xml);
    let mut state = ReadState::new(xml);
    // 이전 이벤트가 소비를 마친 지점 = 다음 시작 태그의 첫 바이트 오프셋.
    // 보존 캡슐이 원문을 바이트 그대로 잘라내는 데 사용한다.
    let mut prev_pos: u64 = 0;
    loop {
        let start_pos = prev_pos;
        let event = reader
            .read_event()
            .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
        let end_pos = reader.buffer_position();
        match event {
            Event::Start(element) => {
                enforce_depth(state.stack.len(), limits.max_depth)?;
                state.start(&element, limits, start_pos)?;
            }
            Event::Empty(element) => {
                enforce_depth(state.stack.len(), limits.max_depth)?;
                state.empty(&element, limits, start_pos, end_pos)?;
            }
            Event::End(element) => state.end(element.name().as_ref(), end_pos)?,
            Event::Text(text) => append_decoded_text(&mut state, &text, limits)?,
            Event::CData(text) => append_cdata(&mut state, &text, limits)?,
            Event::GeneralRef(reference) => append_reference(&mut state, &reference)?,
            Event::DocType(_) => {
                return Err(HmlError::InvalidXml("DTD is not allowed".to_string()))
            }
            Event::Eof => break,
            _ => {}
        }
        prev_pos = end_pos;
    }
    state.finish()
}

fn append_cdata(
    state: &mut ReadState<'_>,
    text: &quick_xml::events::BytesCData<'_>,
    limits: &HmlLimits,
) -> Result<(), HmlError> {
    if text.len() > limits.max_text_node_bytes {
        return Err(HmlError::LimitExceeded("text node size".to_string()));
    }
    let decoded = text
        .decode()
        .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
    state.append_text(&decoded)
}

fn enforce_depth(current_depth: usize, max_depth: usize) -> Result<(), HmlError> {
    if current_depth >= max_depth {
        return Err(HmlError::LimitExceeded("XML depth".to_string()));
    }
    Ok(())
}

fn append_decoded_text(
    state: &mut ReadState<'_>,
    text: &quick_xml::events::BytesText<'_>,
    limits: &HmlLimits,
) -> Result<(), HmlError> {
    if text.len() > limits.max_text_node_bytes {
        return Err(HmlError::LimitExceeded("text node size".to_string()));
    }
    let decoded = text
        .decode()
        .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
    state.append_text(&decoded)
}

fn append_reference(
    state: &mut ReadState<'_>,
    reference: &quick_xml::events::BytesRef<'_>,
) -> Result<(), HmlError> {
    if let Some(character) = reference
        .resolve_char_ref()
        .map_err(|error| HmlError::InvalidXml(error.to_string()))?
    {
        return state.append_text(&character.to_string());
    }
    let name = reference
        .decode()
        .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
    let value = match name.as_ref() {
        "lt" => "<",
        "gt" => ">",
        "amp" => "&",
        "quot" => "\"",
        "apos" => "'",
        _ => {
            return Err(HmlError::InvalidXml(format!(
                "entity &{name}; is not allowed"
            )))
        }
    };
    state.append_text(value)
}

fn element_name(element: &BytesStart<'_>) -> Result<String, HmlError> {
    std::str::from_utf8(element.name().as_ref())
        .map(str::to_owned)
        .map_err(|_| HmlError::InvalidXml("non-UTF-8 element name".to_string()))
}

fn validate_attributes(element: &BytesStart<'_>, max: usize) -> Result<(), HmlError> {
    let mut count = 0usize;
    for item in element.attributes() {
        item.map_err(|error| HmlError::InvalidXml(error.to_string()))?;
        count += 1;
        if count > max {
            return Err(HmlError::LimitExceeded("attribute count".to_string()));
        }
    }
    Ok(())
}

fn attribute(element: &BytesStart<'_>, key: &[u8]) -> Result<Option<String>, HmlError> {
    for item in element.attributes() {
        let attr = item.map_err(|error| HmlError::InvalidXml(error.to_string()))?;
        if attr.key.as_ref() == key {
            let raw = std::str::from_utf8(attr.value.as_ref())
                .map_err(|_| HmlError::InvalidXml("non-UTF-8 attribute".to_string()))?;
            let value = quick_xml::escape::unescape(raw)
                .map_err(|error| HmlError::InvalidXml(error.to_string()))?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

fn parse_attribute<T>(element: &BytesStart<'_>, key: &[u8]) -> Result<Option<T>, HmlError>
where
    T: std::str::FromStr,
{
    attribute(element, key)?
        .map(|value| {
            value
                .parse()
                .map_err(|_| HmlError::InvalidXml(format!("invalid numeric attribute {value}")))
        })
        .transpose()
}

fn parse_border_width(element: &BytesStart<'_>) -> Result<u8, HmlError> {
    let Some(value) = attribute(element, b"Width")? else {
        return Ok(border_width_index(0.1));
    };
    let millimeters = value
        .strip_suffix("mm")
        .and_then(|number| number.trim().parse::<f64>().ok())
        .ok_or_else(|| HmlError::InvalidXml(format!("invalid border width {value}")))?;
    Ok(border_width_index(millimeters))
}

fn parse_bool_attribute(element: &BytesStart<'_>, key: &[u8]) -> Result<bool, HmlError> {
    Ok(matches!(
        attribute(element, key)?.as_deref(),
        Some("true" | "1")
    ))
}

fn parse_padding(element: &BytesStart<'_>) -> Result<Padding, HmlError> {
    Ok(Padding {
        left: parse_attribute(element, b"Left")?.unwrap_or(0),
        right: parse_attribute(element, b"Right")?.unwrap_or(0),
        top: parse_attribute(element, b"Top")?.unwrap_or(0),
        bottom: parse_attribute(element, b"Bottom")?.unwrap_or(0),
    })
}

fn language_array<T>(element: &BytesStart<'_>) -> Result<[T; 7], HmlError>
where
    T: std::str::FromStr + Copy + Default,
{
    let mut values = [T::default(); 7];
    for (index, name) in LANGUAGE_NAMES.iter().enumerate() {
        values[index] = parse_attribute(element, name.as_bytes())?.unwrap_or_default();
    }
    Ok(values)
}

fn parse_alignment(value: Option<&str>) -> Alignment {
    match value {
        Some("Left") => Alignment::Left,
        Some("Right") => Alignment::Right,
        Some("Center") => Alignment::Center,
        Some("Distribute") => Alignment::Distribute,
        Some("Split") => Alignment::Split,
        _ => Alignment::Justify,
    }
}

fn parse_vert_rel_to(value: Option<&str>) -> VertRelTo {
    match value {
        Some("Page") => VertRelTo::Page,
        Some("Para") => VertRelTo::Para,
        _ => VertRelTo::Paper,
    }
}

fn parse_vert_align(value: Option<&str>) -> VertAlign {
    match value {
        Some("Center") => VertAlign::Center,
        Some("Bottom") => VertAlign::Bottom,
        Some("Inside") => VertAlign::Inside,
        Some("Outside") => VertAlign::Outside,
        _ => VertAlign::Top,
    }
}

fn parse_horz_rel_to(value: Option<&str>) -> HorzRelTo {
    match value {
        Some("Page") => HorzRelTo::Page,
        Some("Column") => HorzRelTo::Column,
        Some("Para") => HorzRelTo::Para,
        _ => HorzRelTo::Paper,
    }
}

fn parse_horz_align(value: Option<&str>) -> HorzAlign {
    match value {
        Some("Center") => HorzAlign::Center,
        Some("Right") => HorzAlign::Right,
        Some("Inside") => HorzAlign::Inside,
        Some("Outside") => HorzAlign::Outside,
        _ => HorzAlign::Left,
    }
}

fn parse_text_wrap(value: Option<&str>) -> TextWrap {
    match value {
        Some("Tight") => TextWrap::Tight,
        Some("Through") => TextWrap::Through,
        Some("TopAndBottom") => TextWrap::TopAndBottom,
        Some("BehindText") => TextWrap::BehindText,
        Some("InFrontOfText") => TextWrap::InFrontOfText,
        _ => TextWrap::Square,
    }
}

fn set_indexed<T: Default>(values: &mut Vec<T>, index: usize, value: T) {
    if values.len() <= index {
        values.resize_with(index + 1, T::default);
    }
    values[index] = value;
}
