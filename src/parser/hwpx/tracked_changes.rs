//! Apply saved tracking visibility before section parsing, including owned lists.
//! The official OWPML model defines type/hide, Id/TcId, paraend and merged.
//! Hidden spans affect text and objects; paint and pagination consume one IR.

use super::{utils::local_name, HwpxError};
use quick_xml::{
    events::{BytesStart, Event},
    Reader, Writer,
};
use std::collections::{HashMap, HashSet};

pub(super) struct Visibility(HashMap<String, String>);
impl Visibility {
    pub(super) fn from_header(xml: &str) -> Result<Self, HwpxError> {
        let mut reader = Reader::from_str(xml);
        let mut hidden = HashMap::new();
        loop {
            match reader.read_event().map_err(xml_error)? {
                Event::Start(e) | Event::Empty(e)
                    if local_name(e.name().as_ref()) == b"trackChange" =>
                {
                    if matches!(attribute(&e, b"hide")?.as_deref(), Some("1" | "true")) {
                        let kind = attribute(&e, b"type")?.unwrap_or_default();
                        if matches!(kind.as_str(), "Insert" | "Delete") {
                            let id = attribute(&e, b"id")?
                                .ok_or_else(|| structure("missing change ID"))?;
                            hidden.insert(id, kind);
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(Self(hidden))
    }
    pub(super) fn project(&self, xml: &str) -> Result<Option<String>, HwpxError> {
        if self.0.is_empty() {
            return Ok(None);
        }
        // A header can retain unused history. Such a section must keep its
        // original parsing path and stored paragraph/line partition.
        let mut scan = Reader::from_str(xml);
        let mut used = false;
        loop {
            match scan.read_event().map_err(xml_error)? {
                Event::Start(e) | Event::Empty(e) => {
                    let name = e.name();
                    let local = local_name(name.as_ref());
                    let kind = match local {
                        b"insertBegin" => "Insert",
                        b"deleteBegin" => "Delete",
                        _ => continue,
                    };
                    if attribute(&e, b"TcId")?
                        .is_some_and(|id| self.0.get(&id).is_some_and(|k| k == kind))
                    {
                        used = true;
                        break;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        if !used {
            return Ok(None);
        }
        let mut reader = Reader::from_str(xml);
        let mut stack: Vec<Node> = Vec::new();
        let mut roots = Vec::new();
        let mut node_count = 0usize;
        loop {
            let event = reader.read_event().map_err(xml_error)?.into_owned();
            // Bound the additional tree allocation independently of ZIP byte
            // limits; tiny tags otherwise amplify memory in this input path.
            node_count += 1;
            if node_count > 512 * 1024 {
                return Err(structure("tracking XML node budget"));
            }
            match event {
                Event::Start(start) => {
                    if stack.len() >= 256 {
                        return Err(structure("tracking XML nesting limit"));
                    }
                    stack.push(Node {
                        start: Some(start),
                        children: Vec::new(),
                        event: None,
                    });
                }
                Event::End(_) => {
                    let node = stack.pop().ok_or_else(|| structure("unbalanced XML end"))?;
                    append(&mut stack, &mut roots, node);
                }
                Event::Eof => break,
                event => append(
                    &mut stack,
                    &mut roots,
                    Node {
                        start: None,
                        children: Vec::new(),
                        event: Some(event),
                    },
                ),
            }
        }
        if !stack.is_empty() {
            return Err(structure("unclosed XML element"));
        }
        project_lists(&mut roots, self)?;
        let mut writer = Writer::new(Vec::new());
        for node in &roots {
            node.write(&mut writer)?;
        }
        String::from_utf8(writer.into_inner())
            .map(Some)
            .map_err(|e| structure(&e.to_string()))
    }
}
fn xml_error(error: quick_xml::Error) -> HwpxError {
    HwpxError::XmlError(format!("tracking: {error}"))
}
fn structure(message: &str) -> HwpxError {
    HwpxError::XmlError(format!("tracking structure: {message}"))
}
fn attribute(e: &BytesStart<'_>, key: &[u8]) -> Result<Option<String>, HwpxError> {
    for a in e.attributes() {
        let a = a.map_err(|e| structure(&e.to_string()))?;
        if a.key.as_ref().as_bytes() == key {
            return a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .map(|v| Some(v.into_owned()))
                .map_err(xml_error);
        }
    }
    Ok(None)
}
struct Node {
    start: Option<BytesStart<'static>>,
    children: Vec<Node>,
    event: Option<Event<'static>>,
}
impl Node {
    fn element(&self) -> Option<&BytesStart<'_>> {
        self.start.as_ref().or(match &self.event {
            Some(Event::Empty(e)) => Some(e),
            _ => None,
        })
    }
    fn is(&self, name: &[u8]) -> bool {
        self.element()
            .is_some_and(|e| local_name(e.name().as_ref()) == name)
    }
    fn is_xml_spacing(&self) -> bool {
        matches!(&self.event, Some(Event::Text(text))
            if text.as_ref().bytes().all(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n')))
    }
    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<(), HwpxError> {
        if let Some(start) = &self.start {
            writer
                .write_event(Event::Start(start.clone()))
                .map_err(|e| structure(&e.to_string()))?;
            for child in &self.children {
                child.write(writer)?;
            }
            writer
                .write_event(Event::End(start.to_end()))
                .map_err(|e| structure(&e.to_string()))?;
        } else if let Some(event) = &self.event {
            writer
                .write_event(event.clone())
                .map_err(|e| structure(&e.to_string()))?;
        }
        Ok(())
    }
}
fn append(stack: &mut [Node], roots: &mut Vec<Node>, node: Node) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}
fn clear_lines(node: &mut Node) {
    node.children
        .retain(|n| !n.is(b"linesegarray") && !n.is(b"lineseg"));
}

/// One paragraph of a list after projection, on the stored text axis.
struct Piece {
    merged: bool,
    hidden_end: bool,
    /// Stored axis length without the paragraph mark, hidden content included.
    total: u32,
    /// Stored axis offset of the first visible unit, if any.
    first_visible: Option<u32>,
    /// Stored line starts (`lineseg textpos`) before projection.
    starts: Vec<u32>,
}
/// Stored `merged` paragraphs share the line array of the first one. Its
/// text positions run over the whole chain, one unit per paragraph mark.
struct Chain {
    starts: Vec<u32>,
    pos: u32,
}
fn stored_line_starts(node: &Node) -> Result<Vec<u32>, HwpxError> {
    let mut starts = Vec::new();
    for array in node.children.iter().filter(|n| n.is(b"linesegarray")) {
        for seg in array.children.iter().filter(|n| n.is(b"lineseg")) {
            if let Some(pos) = attribute(seg.element().unwrap(), b"textpos")? {
                if let Ok(pos) = pos.parse() {
                    starts.push(pos);
                }
            }
        }
    }
    Ok(starts)
}
/// Stored axis position of the first visible unit at or after `start`, where
/// `start` is the first unit of the merged paragraph `index`. A visible
/// paragraph mark counts as a unit; a hidden one continues into the next
/// merged paragraph.
fn first_visible_position(
    pieces: &[Option<Piece>],
    spacing: &[bool],
    index: usize,
    start: u32,
) -> u32 {
    let mut pos = start;
    for (offset, piece) in pieces[index..].iter().enumerate() {
        let Some(piece) = piece else {
            if spacing[index + offset] {
                continue;
            }
            break;
        };
        if offset > 0 && !piece.merged {
            break;
        }
        if let Some(visible) = piece.first_visible {
            return pos + visible;
        }
        if !piece.hidden_end {
            return pos + piece.total;
        }
        pos += piece.total + 1;
    }
    pos.saturating_sub(1)
}

fn project_lists(nodes: &mut Vec<Node>, visibility: &Visibility) -> Result<(), HwpxError> {
    for node in nodes.iter_mut() {
        project_lists(&mut node.children, visibility)?;
    }
    let mut pieces: Vec<Option<Piece>> = Vec::with_capacity(nodes.len());
    let mut spacing: Vec<bool> = Vec::with_capacity(nodes.len());
    for node in nodes.iter_mut() {
        spacing.push(node.is_xml_spacing());
        if !node.is(b"p") {
            pieces.push(None);
            continue;
        }
        let merged = matches!(
            attribute(node.element().unwrap(), b"merged")?.as_deref(),
            Some("1" | "true")
        );
        let starts = stored_line_starts(node)?;
        let mut ranges = Ranges::default();
        let mut axis = Axis::default();
        let (changed, hidden_end) = project_content(
            &mut node.children,
            visibility,
            &mut ranges,
            Some(&mut axis),
            false,
        )?;
        if !ranges.active.is_empty() {
            return Err(structure("unclosed hidden change range in paragraph"));
        }
        if changed {
            clear_lines(node);
        }
        pieces.push(Some(Piece {
            merged,
            hidden_end,
            total: axis.total,
            first_visible: axis.first_visible,
            starts,
        }));
    }
    let mut output: Vec<Node> = Vec::new();
    let mut hidden_end = false;
    let mut chain: Option<Chain> = None;
    for (index, mut node) in std::mem::take(nodes).into_iter().enumerate() {
        let Some(piece) = &pieces[index] else {
            output.push(node);
            if !spacing[index] {
                hidden_end = false;
                chain = None;
            }
            continue;
        };
        // Paragraphs join only inside one list. No joining across a cell,
        // caption, note or other owner. Formatting whitespace between sibling
        // elements is not a paragraph boundary. Keep it in XML, while
        // resolving the preceding list owner.
        let previous = output
            .iter()
            .rposition(|n| !n.is_xml_spacing())
            .filter(|&i| output[i].is(b"p"));
        let chained = piece.merged && previous.is_some();
        let start = match (&chain, chained) {
            (Some(chain), true) => Some((chain.pos, chain.starts.as_slice())),
            _ => None,
        };
        // A hidden paragraph mark joins the next paragraph into the same
        // line. A visible mark before a stored `merged` paragraph follows
        // the stored lines of the chain: Hancom Print keeps them, and they
        // start a line at the first visible unit after the mark only when
        // the mark breaks the line. Without stored lines the mark breaks.
        let join = previous.is_some()
            && (hidden_end
                || start.is_some_and(|(start, starts)| {
                    let line = starts.iter().copied().filter(|&s| s >= start).min();
                    match line {
                        Some(line) => {
                            line != first_visible_position(&pieces, &spacing, index, start)
                        }
                        None => !starts.is_empty(),
                    }
                }));
        chain = Some(match (chain.take(), chained) {
            (Some(chain), true) => Chain {
                pos: chain.pos + piece.total + 1,
                ..chain
            },
            _ => Chain {
                starts: piece.starts.clone(),
                pos: piece.total + 1,
            },
        });
        hidden_end = piece.hidden_end;
        if let Some(previous) = previous.filter(|_| join || chained) {
            // The stored lines of the chain no longer describe either part.
            clear_lines(&mut output[previous]);
            clear_lines(&mut node);
            if join {
                output[previous].children.extend(node.children);
                continue;
            }
        }
        output.push(node);
    }
    *nodes = output;
    Ok(())
}
/// Stored text axis of one paragraph: a control takes 8 units, a character
/// its UTF-16 length. Range marks and highlight marks take none.
#[derive(Default)]
struct Axis {
    total: u32,
    first_visible: Option<u32>,
}
impl Axis {
    fn add(&mut self, units: u32, visible: bool) {
        if visible && units > 0 {
            self.first_visible.get_or_insert(self.total);
        }
        self.total += units;
    }
}
fn text_units(text: &str) -> u32 {
    text.chars()
        .map(|c| if (c as u32) > 0xFFFF { 2 } else { 1 })
        .sum()
}
fn element_units(local: &[u8]) -> u32 {
    match local {
        b"markpenBegin" | b"markpenEnd" | b"linesegarray" | b"lineseg" | b"p" => 0,
        b"lineBreak" | b"columnBreak" | b"nbSpace" | b"fwSpace" | b"hypen" | b"hyphen"
        | b"softHyphen" => 1,
        _ => 8,
    }
}
/// Hidden ranges open in one paragraph. Only an open range hides content.
/// Hancom can store an end that closes no open range: a repeat at a later run
/// split, or an end whose begin is absent from the whole document. Such an
/// end encloses nothing, so it adds no content boundary (Hancom Print keeps
/// the text before it).
#[derive(Default)]
struct Ranges {
    active: HashSet<(String, String)>,
}
fn project_content(
    nodes: &mut Vec<Node>,
    visibility: &Visibility,
    ranges: &mut Ranges,
    mut axis: Option<&mut Axis>,
    in_text: bool,
) -> Result<(bool, bool), HwpxError> {
    let mut output = Vec::new();
    let mut changed = false;
    let mut hidden_end = false;
    for mut node in std::mem::take(nodes) {
        if let Some(element) = node.element() {
            let name = element.name();
            let local = local_name(name.as_ref());
            if matches!(
                local,
                b"insertBegin" | b"insertEnd" | b"deleteBegin" | b"deleteEnd"
            ) {
                let kind = if local.starts_with(b"insert") {
                    "Insert"
                } else {
                    "Delete"
                };
                let tc_id = attribute(element, b"TcId")?
                    .ok_or_else(|| structure("missing range change ID"))?;
                if visibility.0.get(&tc_id).is_some_and(|k| k == kind) {
                    let id =
                        attribute(element, b"Id")?.ok_or_else(|| structure("missing range ID"))?;
                    let key = (kind.to_owned(), id);
                    if local.ends_with(b"Begin") {
                        if !ranges.active.insert(key) {
                            return Err(structure("duplicate active change range"));
                        }
                    } else {
                        if !ranges.active.remove(&key) {
                            // Encloses no open range: no boundary, no paragraph-end merge.
                            continue;
                        }
                        hidden_end |= matches!(
                            attribute(element, b"paraend")?.as_deref(),
                            Some("1" | "true")
                        );
                    }
                }
                continue;
            }
            // Child paragraph lists were projected in their own ownership scope.
            if node.is(b"p") {
                output.push(node);
                continue;
            }
            let flow = matches!(local, b"run" | b"t");
            let hidden = !ranges.active.is_empty() && !flow;
            let child_in_text = local == b"t";
            if !flow {
                if let Some(axis) = axis.as_deref_mut() {
                    axis.add(element_units(local), !hidden);
                }
            }
            if hidden {
                changed = true;
                continue;
            }
            let child_axis = if flow { axis.as_deref_mut() } else { None };
            let (child_changed, child_end) = project_content(
                &mut node.children,
                visibility,
                ranges,
                child_axis,
                child_in_text,
            )?;
            changed |= child_changed;
            hidden_end |= child_end;
            output.push(node);
        } else {
            let content = match &node.event {
                Some(Event::Text(text)) => Some(text_units(text.as_ref())),
                Some(Event::CData(text)) => Some(text_units(text.as_ref())),
                Some(Event::GeneralRef(_)) => Some(1),
                _ => None,
            };
            let hidden = !ranges.active.is_empty() && content.is_some();
            if let (Some(axis), Some(units), true) = (axis.as_deref_mut(), content, in_text) {
                axis.add(units, !hidden);
            }
            if hidden {
                changed = true;
            } else {
                output.push(node);
            }
        }
    }
    *nodes = output;
    Ok((changed, hidden_end))
}
