//! Bounded reader for the legacy VtChart 6 presentation archive.
//!
//! Follow object references, never search arbitrary payloads for type signatures.
//! Unsupported class versions fail closed; no native chart library is required.

use std::collections::BTreeMap;
use std::rc::Rc;

use encoding_rs::EUC_KR;

type Result<T> = std::result::Result<T, ()>;

#[derive(Clone, Copy)]
pub(super) struct Axis {
    pub min: f64,
    pub max: f64,
    pub divisions: u16,
}

#[derive(Clone)]
pub(super) struct Series {
    pub color: u32,
    pub fill: u32,
    pub width: f64,
    pub secondary: bool,
    pub index: usize,
    pub selectors: Vec<i16>,
    pub legend: Option<Text>,
}

/// 저장 글꼴 크기는 포인트 단위이며 차트의 캐시 사각형 크기와 구별한다.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Font {
    pub family: String,
    pub size: f64,
    pub style: u16,
    pub effects: u16,
    pub color: u32,
}

#[derive(Clone, Debug)]
pub(super) struct Text {
    pub value: String,
    pub font: Font,
    pub metrics: TextMetrics,
}

pub(super) struct Presentation {
    pub chart_type: usize,
    pub axes: [Axis; 2],
    pub series: Vec<Series>,
    pub extent: [f64; 4],
    pub plot: [f64; 4],
    pub title: Option<Text>,
    pub legend_font: Option<Font>,
    pub label_fonts: [Option<Font>; 4],
    /// 계열 막대 사이와 범주 사이의 간격을 막대 너비에 대한 비율로 저장한다.
    pub bar_gap: f64,
    pub x_gap: f64,
    pub backdrop: Option<Backdrop>,
    pub title_section: Option<Section>,
    pub legend_section: Option<Section>,
    pub plot_section: Section,
    pub axis_styles: [AxisStyle; 4],
    pub labels: [Option<Text>; 4],
}

/// 소유 텍스트의 저장 점유 정보. 글꼴 크기와 같은 값으로 추정하지 않는다.
#[derive(Clone, Debug)]
pub(super) struct TextMetrics {
    pub margin: i32,
    pub rect: [i32; 4],
    pub extent: [i32; 2],
    pub cache_valid: bool,
    pub horizontal: u16,
    pub vertical: u16,
    pub orientation: u16,
    pub paint_rect: [i32; 4],
}
#[derive(Clone, Debug)]
pub(super) struct Section {
    pub external: i16,
    pub internal: i16,
    pub visible: bool,
    pub rect: [f64; 4],
    pub location: u16,
    pub backdrop: Option<Backdrop>,
}
#[derive(Clone, Debug)]
pub(super) struct Backdrop {
    pub frame_color: u32,
    pub frame_width: f64,
    pub frame_enabled: bool,
    pub fill_kind: i16,
    pub fill_color: u32,
    pub shadow_enabled: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Pen {
    pub enabled: bool,
    pub color: u32,
    pub width: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AxisStyle {
    pub labels_visible: bool,
    pub pen: Pen,
    pub major_grid: Pen,
    pub minor_grid: Pen,
    pub tick_length: f64,
    pub tick_style: u16,
}
fn pen(bytes: &[u8]) -> Result<Pen> {
    if bytes.len() != 22 || bytes[..2] != [1, 0] || bytes[8..10] != [1, 0] {
        return Err(());
    }
    let width = f32::from_le_bytes(bytes[14..18].try_into().map_err(|_| ())?) as f64;
    if !width.is_finite() || width < 0.0 {
        return Err(());
    }
    Ok(Pen {
        enabled: u16::from_le_bytes(bytes[2..4].try_into().map_err(|_| ())?) != 0,
        color: u32::from_le_bytes(bytes[10..14].try_into().map_err(|_| ())?),
        width,
    })
}
fn axis_label(node: &Node) -> Option<Text> {
    match node {
        Node::Text(text) => Some(text.clone()),
        Node::Array(items) => items
            .iter()
            .filter_map(|item| axis_label(item))
            .max_by_key(|text| text.metrics.extent[0]),
        _ => None,
    }
}

enum Node {
    Empty,
    Count(usize),
    Array(Vec<Rc<Node>>),
    Axis(Option<Axis>, Option<Font>, AxisStyle, Option<Text>),
    Series(Series),
    Section(Section),
    Backdrop(Backdrop),
    Fill(i16, u32),
    Plot(
        Box<(
            [Option<Axis>; 4],
            [Option<Font>; 4],
            Section,
            [f64; 2],
            [AxisStyle; 4],
            [Option<Text>; 4],
        )>,
    ),
    String(String),
    Font(Font),
    Text(Text),
    Title(Option<Text>, Section),
    Legend(Option<Font>, Section),
}

fn axis_label_font(node: &Node) -> Option<Font> {
    match node {
        Node::Text(text) => Some(text.font.clone()),
        Node::Array(items) => {
            let mut font = None;
            for item in items {
                match item.as_ref() {
                    Node::Empty => {}
                    Node::Text(text) => {
                        if font.as_ref().is_some_and(|font| *font != text.font) {
                            // 다단 축의 서로 다른 레이블 글꼴을 하나로 덮어쓰지 않는다.
                            return None;
                        }
                        font = Some(text.font.clone());
                    }
                    _ => return None,
                }
            }
            font
        }
        _ => None,
    }
}

struct Reader<'a> {
    data: &'a [u8],
    at: usize,
    types: BTreeMap<i32, (String, u16)>,
    objects: BTreeMap<i32, Rc<Node>>,
    depth: usize,
    remaining: usize,
}

impl<'a> Reader<'a> {
    fn bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(len).ok_or(())?;
        let bytes = self.data.get(self.at..end).ok_or(())?;
        self.at = end;
        Ok(bytes)
    }

    fn skip(&mut self, len: usize) -> Result<()> {
        self.bytes(len).map(|_| ())
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.bytes(2)?.try_into().map_err(|_| ())?,
        ))
    }

    fn i16(&mut self) -> Result<i16> {
        Ok(self.u16()? as i16)
    }

    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(
            self.bytes(4)?.try_into().map_err(|_| ())?,
        ))
    }

    fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(
            self.bytes(8)?.try_into().map_err(|_| ())?,
        ))
    }

    fn rect(&mut self) -> Result<[f64; 4]> {
        let mut result = [0.0; 4];
        for value in &mut result {
            *value = f32::from_le_bytes(self.bytes(4)?.try_into().map_err(|_| ())?) as f64;
            if !value.is_finite() {
                return Err(());
            }
        }
        Ok(result)
    }

    fn class(&mut self) -> Result<(String, u16)> {
        let id = self.i32()?;
        if let Some(class) = self.types.get(&id) {
            return Ok(class.clone());
        }
        if id < 0 || self.types.len() >= 64 {
            return Err(());
        }
        let len = self.u16()? as usize;
        if !(2..=64).contains(&len) {
            return Err(());
        }
        let bytes = self.bytes(len)?;
        if bytes.last() != Some(&0) {
            return Err(());
        }
        let name = std::str::from_utf8(&bytes[..len - 1])
            .map_err(|_| ())?
            .to_owned();
        let version = self.u16()?;
        self.types.insert(id, (name.clone(), version));
        Ok((name, version))
    }

    fn base(&mut self, expected: &str) -> Result<Node> {
        let (name, version) = self.class()?;
        if name != expected {
            return Err(());
        }
        self.body(&name, version)
    }

    fn object(&mut self) -> Result<Rc<Node>> {
        self.remaining = self.remaining.checked_sub(1).ok_or(())?;
        let id = self.i32()?;
        if id == -1 {
            return Ok(Rc::new(Node::Empty));
        }
        if let Some(object) = self.objects.get(&id) {
            return Ok(Rc::clone(object));
        }
        if id < 0 || self.depth >= 32 || self.objects.len() >= 32768 {
            return Err(());
        }
        let (name, version) = self.class()?;
        self.depth += 1;
        let result = self.body(&name, version);
        self.depth -= 1;
        let object = Rc::new(result?);
        self.objects.insert(id, Rc::clone(&object));
        Ok(object)
    }

    fn body(&mut self, name: &str, version: u16) -> Result<Node> {
        let expected_version = match name {
            "VtChartPlot" => 4,
            "VtAxis" => 3,
            "VtSeries" | "VtTextBlock" | "VtWindow" => 2,
            _ => 1,
        };
        if version != expected_version {
            return Err(());
        }
        match name {
            "VtObject" => {}
            "VtCollection" => {
                let count = self.u16()? as usize;
                self.base("VtObject")?;
                return Ok(Node::Count(count));
            }
            "VtList" => {
                let Node::Count(0) = self.base("VtCollection")? else {
                    return Err(());
                };
            }
            "VtArray" => {
                let capacity = self.u16()? as usize;
                let Node::Count(count) = self.base("VtCollection")? else {
                    return Err(());
                };
                if count > capacity || capacity > self.remaining {
                    return Err(());
                }
                let mut items = Vec::with_capacity(capacity);
                for _ in 0..capacity {
                    items.push(self.object()?);
                }
                return Ok(Node::Array(items));
            }
            "VtDataGrid" => {
                self.base("VtMatrix")?;
                self.skip(8)?;
            }
            "VtMatrix" => {
                self.base("VtCollection")?;
                let rows = self.u16()? as usize;
                let cols = self.u16()? as usize;
                let count = rows.checked_mul(cols).ok_or(())?;
                if count > self.remaining {
                    return Err(());
                }
                for _ in 0..count {
                    self.object()?;
                }
            }
            "VtString" => {
                let len = self.u16()? as usize;
                let text = if len > 0 {
                    // Legacy archives reserve one trailing byte, but newer
                    // writers do not always zero it. Keep the length/bounds
                    // check without treating that byte as a NUL contract.
                    let bytes = self.bytes(len + 1)?;
                    let end = bytes[..len]
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(len);
                    let (text, _, errors) = EUC_KR.decode(&bytes[..end]);
                    if errors {
                        return Err(());
                    }
                    text.into_owned()
                } else {
                    String::new()
                };
                self.base("VtValue")?;
                return Ok(Node::String(text));
            }
            "VtDouble" => {
                self.skip(10)?;
                self.base("VtValue")?;
            }
            "VtValue" => {
                self.base("VtObject")?;
            }
            "VtBackdrop" => {
                let bytes = self.bytes(50)?;
                let frame_width =
                    f32::from_le_bytes(bytes[44..48].try_into().map_err(|_| ())?) as f64;
                if !frame_width.is_finite() || frame_width < 0.0 {
                    return Err(());
                }
                let fill = self.object()?;
                let (fill_kind, fill_color) = match fill.as_ref() {
                    Node::Fill(kind, color) => (*kind, *color),
                    Node::Empty => (0, 0),
                    _ => return Err(()),
                };
                let backdrop = Backdrop {
                    frame_color: u32::from_le_bytes(bytes[34..38].try_into().map_err(|_| ())?),
                    frame_width,
                    frame_enabled: bytes[48..50] != [0, 0],
                    shadow_enabled: bytes[28..30] != [0, 0],
                    fill_kind,
                    fill_color,
                };
                self.base("VtObject")?;
                return Ok(Node::Backdrop(backdrop));
            }
            "VtFill" => {
                let bytes = self.bytes(34)?;
                let color = u32::from_le_bytes(bytes[6..10].try_into().map_err(|_| ())?);
                self.object()?;
                let kind = self.i16()?;
                self.base("VtObject")?;
                return Ok(Node::Fill(kind, color));
            }
            "VtPicture" => {
                self.skip(2)?;
                if self.i16()? != 0 {
                    // The embedded-picture branch still uses an object reference.
                    // A null reference is a valid empty backdrop/marker, not a
                    // bitmap payload. Nonempty VtPict objects remain unsupported.
                    if self.i32()? != -1 {
                        return Err(());
                    }
                } else {
                    self.object()?;
                }
                self.base("VtObject")?;
            }
            "VtFont" => {
                let object = self.object()?;
                let Node::String(family) = object.as_ref() else {
                    return Err(());
                };
                let bytes = self.bytes(14)?;
                let size = f32::from_le_bytes(bytes[..4].try_into().map_err(|_| ())?) as f64;
                if !size.is_finite() || size <= 0.0 || bytes[8..10] != [1, 0] {
                    return Err(());
                }
                let font = Font {
                    family: family.clone(),
                    size,
                    style: u16::from_le_bytes(bytes[4..6].try_into().map_err(|_| ())?),
                    effects: u16::from_le_bytes(bytes[6..8].try_into().map_err(|_| ())?),
                    color: u32::from_le_bytes(bytes[10..14].try_into().map_err(|_| ())?),
                };
                self.base("VtObject")?;
                return Ok(Node::Font(font));
            }
            "VtTextBlock" => {
                let layout = self.bytes(12)?;
                if layout[..2] != [1, 0] {
                    return Err(());
                }
                let horizontal = u16::from_le_bytes(layout[4..6].try_into().map_err(|_| ())?);
                let vertical = u16::from_le_bytes(layout[6..8].try_into().map_err(|_| ())?);
                let orientation = u16::from_le_bytes(layout[8..10].try_into().map_err(|_| ())?);
                self.object()?;
                let font = self.object()?;
                let Node::Font(font) = font.as_ref() else {
                    return Err(());
                };
                self.u16()?;
                let margin = self.i32()?;
                let mut rect = [0; 4];
                for value in &mut rect {
                    *value = self.i32()?;
                }
                let cache_valid = self.u16()? != 0;
                let text = self.object()?;
                let value = match text.as_ref() {
                    Node::String(value) => value.clone(),
                    Node::Empty => String::new(),
                    _ => return Err(()),
                };
                let extent = [self.i32()?, self.i32()?];
                let mut paint_rect = [0; 4];
                for value in &mut paint_rect {
                    *value = self.i32()?;
                }
                self.skip(2)?;
                self.base("VtObject")?;
                return Ok(Node::Text(Text {
                    value,
                    font: font.clone(),
                    metrics: TextMetrics {
                        margin,
                        rect,
                        extent,
                        cache_valid,
                        horizontal,
                        vertical,
                        orientation,
                        paint_rect,
                    },
                }));
            }
            "VtChartText" => {
                let text = self.object()?;
                let Node::Text(text) = text.as_ref() else {
                    return Err(());
                };
                let Node::Section(section) = self.base("VtChartSection")? else {
                    return Err(());
                };
                return Ok(Node::Title(section.visible.then(|| text.clone()), section));
            }
            "VtChartTitle" | "VtChartFootnote" => {
                return self.base("VtChartText");
            }
            "VtChartSection" => {
                let external = self.i16()?;
                let internal = self.i16()?;
                if self.u16()? != 1 || external < 0 || internal < 0 {
                    return Err(());
                }
                let visible = self.u16()? != 0;
                let rect = self.rect()?;
                let location = self.u16()?;
                let object = self.object()?;
                let backdrop = match object.as_ref() {
                    Node::Backdrop(b) => Some(b.clone()),
                    Node::Empty => None,
                    _ => return Err(()),
                };
                self.base("VtObject")?;
                return Ok(Node::Section(Section {
                    external,
                    internal,
                    visible,
                    rect,
                    location,
                    backdrop,
                }));
            }
            "VtChartLegend" => {
                let font = self.object()?;
                let Node::Font(font) = font.as_ref() else {
                    return Err(());
                };
                self.skip(10)?;
                let Node::Section(section) = self.base("VtChartSection")? else {
                    return Err(());
                };
                return Ok(Node::Legend(section.visible.then(|| font.clone()), section));
            }
            "VtLight3" => {
                self.object()?;
                self.skip(10)?;
                self.base("VtObject")?;
            }
            "VtInfLight3" => {
                self.skip(16)?;
                self.base("VtObject")?;
            }
            "VtSeriesLabel" => {
                return self.base("VtTextBlock");
            }
            "VtSeriesPoint" => {
                self.object()?;
                self.skip(12)?;
                self.object()?;
                self.object()?;
                self.base("VtObject")?;
            }
            "VtTextFormat" => {
                self.base("VtObject")?;
                self.skip(2)?;
                self.object()?;
            }
            "VtValueBlock" => {
                self.object()?;
                self.object()?;
                self.skip(2)?;
                self.object()?;
                self.skip(3)?;
                return self.base("VtTextBlock");
            }
            "VtAxisScaleBlock" => {
                self.object()?;
                self.skip(4)?;
                return self.base("VtValueBlock");
            }
            "VtAxis" => {
                self.u16()?;
                let labels_visible = self.u16()? != 0;
                self.u16()?;
                let manual = self.i16()? != 0;
                let props = self.bytes(74)?;
                let tick_length =
                    f32::from_le_bytes(props[66..70].try_into().map_err(|_| ())?) as f64;
                if !tick_length.is_finite() || tick_length < 0.0 {
                    return Err(());
                }
                let mut style = AxisStyle {
                    labels_visible,
                    tick_style: 0,
                    pen: pen(&props[..22])?,
                    major_grid: pen(&props[22..44])?,
                    minor_grid: pen(&props[44..66])?,
                    tick_length,
                };
                // 첫 참조는 축 제목, 둘째 참조는 눈금/범주 레이블이다.
                self.object()?;
                let label = self.object()?;
                let label_font = axis_label_font(label.as_ref());
                self.skip(34)?;
                let axis = if manual {
                    let min = self.f64()?;
                    let max = self.f64()?;
                    let divisions = self.u16()?;
                    self.skip(6)?;
                    if !min.is_finite()
                        || !max.is_finite()
                        || max <= min
                        || !(max - min).is_finite()
                        || !(1..=100).contains(&divisions)
                    {
                        return Err(());
                    }
                    Some(Axis {
                        min,
                        max,
                        divisions,
                    })
                } else {
                    None
                };
                let tail = self.bytes(16)?;
                // WriteOn에서 percent basis/unit 뒤에 저장하는 TickStyle.
                style.tick_style = u16::from_le_bytes(tail[4..6].try_into().map_err(|_| ())?);
                self.base("VtObject")?;
                return Ok(Node::Axis(
                    axis,
                    label_font,
                    style,
                    axis_label(label.as_ref()),
                ));
            }
            "VtCLineItem" => {
                self.skip(48)?;
                self.object()?;
                self.base("VtObject")?;
            }
            "VtSurfaceDesc" => {
                self.skip(46)?;
                self.object()?;
                self.object()?;
                self.skip(2)?;
                self.object()?;
                self.object()?;
                self.skip(14)?;
                self.base("VtObject")?;
            }
            "VtChartPlot" => {
                self.object()?;
                // VtChartPlot v4의 고정 prefix: 깊이/너비 비율 뒤의
                // BarGap·xGap를 읽고, 아직 지원하지 않는 나머지 표현은 건너뛴다.
                // 공식 Plot 속성과 ReadFrom/SetBarGap/SetXGap의 저장 순서를 대조했다.
                let prefix = self.bytes(136)?;
                let gaps = [8, 12].map(|at| {
                    f32::from_le_bytes(prefix[at..at + 4].try_into().expect("bounded plot prefix"))
                        as f64
                });
                if gaps.iter().any(|gap| !gap.is_finite() || *gap < 0.0) {
                    return Err(());
                }
                self.object()?;
                let mut axes = [None; 4];
                let mut fonts = std::array::from_fn(|_| None);
                let mut axis_styles = [AxisStyle::default(); 4];
                let mut labels = std::array::from_fn(|_| None);
                for (index, (axis, font)) in axes.iter_mut().zip(&mut fonts).enumerate() {
                    let object = self.object()?;
                    let Node::Axis(value, label_font, style, label) = object.as_ref() else {
                        return Err(());
                    };
                    *axis = *value;
                    *font = label_font.clone();
                    axis_styles[index] = *style;
                    labels[index] = label.clone();
                }
                self.skip(30)?;
                self.object()?;
                self.skip(16)?;
                let Node::Section(section) = self.base("VtChartSection")? else {
                    return Err(());
                };
                return Ok(Node::Plot(Box::new((
                    axes,
                    fonts,
                    section,
                    gaps,
                    axis_styles,
                    labels,
                ))));
            }
            "VtSeries" => return self.series(),
            "VtWindow" => {
                self.base("VtObject")?;
                self.skip(2)?;
            }
            _ => return Err(()),
        }
        Ok(Node::Empty)
    }

    fn series(&mut self) -> Result<Node> {
        // Two VtPen v1 values (22 bytes each), then VtBrush v1 (18 bytes).
        let prefix = self.bytes(62)?;
        for offset in [0, 22, 44] {
            if prefix[offset..offset + 2] != [1, 0] {
                return Err(());
            }
        }
        let color = u32::from_le_bytes(prefix[10..14].try_into().map_err(|_| ())?);
        let fill = u32::from_le_bytes(prefix[50..54].try_into().map_err(|_| ())?);
        let width = f32::from_le_bytes(prefix[14..18].try_into().map_err(|_| ())?) as f64;
        if !width.is_finite() || width < 0.0 {
            return Err(());
        }
        self.object()?;
        self.object()?;
        // Guides, outline pen, point options, label offsets and the v2 name.
        self.skip(12 + 22 + 16 + 16)?;
        self.object()?;
        self.object()?;
        let label = self.object()?;
        let legend = match label.as_ref() {
            Node::Text(text) => Some(text.clone()),
            Node::Empty => None,
            _ => return Err(()),
        };
        self.skip(2)?;
        self.object()?;
        self.object()?;
        let secondary = self.i16()? != 0;
        self.skip(2)?;
        if self.u16()? != 2 {
            // Inline VtMarker v2.
            return Err(());
        }
        self.skip(34)?;
        self.object()?;
        self.skip(2 + 8 + 12)?;
        let index = self.u16()? as usize;
        let count = self.u16()? as usize;
        if count != 27 {
            return Err(());
        }
        let mut selectors = Vec::with_capacity(count);
        for _ in 0..count {
            selectors.push(self.i16()?);
        }
        let extra = self.u16()? as usize;
        if extra > 27 {
            return Err(());
        }
        self.skip(extra * 2 + 10)?;
        self.base("VtObject")?;
        Ok(Node::Series(Series {
            color,
            fill,
            width,
            secondary,
            index,
            selectors,
            legend,
        }))
    }
}

pub(super) fn parse(data: &[u8]) -> Option<Presentation> {
    parse_inner(data).ok()
}

fn parse_inner(data: &[u8]) -> Result<Presentation> {
    if data.len() > 16 * 1024 * 1024 {
        return Err(());
    }
    // Fixed legacy Contents header, not a fixture-specific object offset.
    let mut reader = Reader {
        data,
        at: 40,
        types: BTreeMap::new(),
        objects: BTreeMap::new(),
        depth: 0,
        remaining: 262144,
    };
    let (name, version) = reader.class()?;
    if name != "VtChart" || version != 6 {
        return Err(());
    }
    reader.object()?;
    reader.skip(2)?;
    let extent = reader.rect()?;
    let backdrop = reader.object()?;
    let backdrop = match backdrop.as_ref() {
        Node::Backdrop(b) => Some(b.clone()),
        Node::Empty => None,
        _ => return Err(()),
    };
    reader.object()?;
    let legend = reader.object()?;
    let plot = reader.object()?;
    let series = reader.object()?;
    let title = reader.object()?;
    reader.object()?;
    reader.skip(6)?;
    reader.object()?;
    reader.skip(10)?;
    let chart_type = reader.i16()?;
    reader.skip(4)?;
    reader.base("VtWindow")?;
    if !matches!(chart_type, 1 | 9) || reader.at != data.len() {
        return Err(());
    }
    let Node::Plot(plot) = plot.as_ref() else {
        return Err(());
    };
    let (axes, label_fonts, plot_section, gaps, axis_styles, labels) = plot.as_ref();
    let (title, title_section) = match title.as_ref() {
        Node::Title(text, section) => (text.clone(), Some(section.clone())),
        Node::Empty => (None, None),
        _ => return Err(()),
    };
    let (legend_font, legend_section) = match legend.as_ref() {
        Node::Legend(font, section) => (font.clone(), Some(section.clone())),
        Node::Empty => (None, None),
        _ => return Err(()),
    };
    let Node::Array(items) = series.as_ref() else {
        return Err(());
    };
    if items.is_empty() || items.len() > 64 {
        return Err(());
    }
    let mut styles = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let Node::Series(style) = item.as_ref() else {
            return Err(());
        };
        // 저장 차트 종류에 대응하는 painter를 사용한다.
        // 막대형(1)은 column만, 혼합형(9)은 column/line만 지원한다.
        let painter = style.selectors[chart_type as usize];
        if style.index != index || !(painter == 1 || chart_type == 9 && painter == 6) {
            return Err(());
        }
        styles.push(style.clone());
    }
    if chart_type == 9
        && (!styles.iter().any(|s| s.selectors[9] == 1)
            || !styles.iter().any(|s| s.selectors[9] == 6))
    {
        return Err(());
    }
    for rect in [extent, plot_section.rect] {
        if rect[2] <= rect[0] || rect[3] <= rect[1] {
            return Err(());
        }
    }
    Ok(Presentation {
        chart_type: chart_type as usize,
        axes: [
            axes[1].ok_or(())?,
            if styles.iter().any(|series| series.secondary) {
                axes[2].ok_or(())?
            } else {
                axes[1].ok_or(())?
            },
        ],
        series: styles,
        extent,
        plot: plot_section.rect,
        title,
        legend_font,
        label_fonts: label_fonts.clone(),
        bar_gap: gaps[0],
        x_gap: gaps[1],
        backdrop,
        title_section,
        legend_section,
        plot_section: plot_section.clone(),
        axis_styles: *axis_styles,
        labels: labels.clone(),
    })
}
