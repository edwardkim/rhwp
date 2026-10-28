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

fn project_lists(nodes: &mut Vec<Node>, visibility: &Visibility) -> Result<(), HwpxError> {
    for node in nodes.iter_mut() {
        project_lists(&mut node.children, visibility)?;
    }
    let mut output: Vec<Node> = Vec::new();
    let mut hidden_end = false;
    for mut node in std::mem::take(nodes) {
        if !node.is(b"p") {
            let is_spacing = node.is_xml_spacing();
            output.push(node);
            if !is_spacing {
                hidden_end = false;
            }
            continue;
        }
        let merged = matches!(
            attribute(node.element().unwrap(), b"merged")?.as_deref(),
            Some("1" | "true")
        );
        let mut active = HashSet::new();
        let (changed, end) = project_content(&mut node.children, visibility, &mut active)?;
        if !active.is_empty() {
            return Err(structure("unclosed hidden change range in paragraph"));
        }
        if changed {
            clear_lines(&mut node);
        }
        // Stored merged paragraphs belong to the previous paragraph in this
        // same list. No joining across a cell, caption, note or other owner.
        // Formatting whitespace between sibling elements is not a paragraph
        // boundary. Keep it in XML, while resolving the preceding list owner.
        let previous = output.iter().rposition(|n| !n.is_xml_spacing());
        if let Some(previous) = previous.filter(|&i| (merged || hidden_end) && output[i].is(b"p")) {
            let previous = &mut output[previous];
            clear_lines(previous);
            clear_lines(&mut node);
            previous.children.extend(node.children);
        } else {
            output.push(node);
        }
        hidden_end = end;
    }
    *nodes = output;
    Ok(())
}
fn project_content(
    nodes: &mut Vec<Node>,
    visibility: &Visibility,
    active: &mut HashSet<(String, String)>,
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
                        if !active.insert(key) {
                            return Err(structure("duplicate active change range"));
                        }
                    } else {
                        if !active.remove(&key) {
                            return Err(structure("hidden range end without begin"));
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
            if !active.is_empty() && !matches!(local, b"run" | b"t") {
                changed = true;
                continue;
            }
            let (child_changed, child_end) =
                project_content(&mut node.children, visibility, active)?;
            changed |= child_changed;
            hidden_end |= child_end;
            output.push(node);
        } else if !active.is_empty()
            && matches!(
                node.event,
                Some(Event::Text(_) | Event::CData(_) | Event::GeneralRef(_))
            )
        {
            changed = true;
        } else {
            output.push(node);
        }
    }
    *nodes = output;
    Ok((changed, hidden_end))
}
