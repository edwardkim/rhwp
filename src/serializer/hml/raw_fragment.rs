use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::parser::hml::{HmlLimits, PreservedFragment};

pub(crate) fn validate(fragment: &PreservedFragment, body_sections: usize) -> Result<(), String> {
    validate_envelope(fragment, body_sections)?;
    let mut limits = HmlLimits::default();
    limits.max_depth = limits.max_depth.saturating_sub(2);
    let root = validate_subtree(&fragment.raw_xml, &limits)?;
    let expected = fragment
        .xml_path
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .split('[')
        .next()
        .unwrap_or_default();
    if root != expected {
        return Err(format!(
            "raw root {root} does not match envelope path root {expected}"
        ));
    }
    if is_modeled_child(&fragment.parent, &root) {
        return Err("raw fragment duplicates a modeled document child".to_string());
    }
    Ok(())
}

fn validate_envelope(fragment: &PreservedFragment, body_sections: usize) -> Result<(), String> {
    let max_anchor = match fragment.parent.as_str() {
        "HEAD" => 1,
        "BODY" => body_sections,
        "TAIL" => 0,
        other => return Err(format!("unsupported raw fragment parent {other}")),
    };
    let prefix = format!("/HWPML/{}/", fragment.parent);
    if !fragment.xml_path.starts_with(&prefix) || fragment.xml_path[prefix.len()..].contains('/') {
        return Err("raw fragment path is not a direct child of its parent".to_string());
    }
    if fragment.modeled_siblings_before > max_anchor {
        return Err(format!(
            "raw fragment anchor {} exceeds modeled child count {max_anchor}",
            fragment.modeled_siblings_before
        ));
    }
    Ok(())
}

fn is_modeled_child(parent: &str, root: &str) -> bool {
    match parent {
        "HEAD" => matches!(root, "DOCSUMMARY" | "DOCSETTING" | "MAPPINGTABLE"),
        "BODY" => root == "SECTION",
        _ => false,
    }
}

fn validate_subtree(xml: &str, limits: &HmlLimits) -> Result<String, String> {
    if xml.len() > limits.max_xml_bytes {
        return Err("raw fragment exceeds the XML byte limit".to_string());
    }
    let mut reader = Reader::from_str(xml);
    let mut state = SubtreeState::default();
    loop {
        let event = reader.read_event().map_err(|error| error.to_string())?;
        match event {
            Event::Start(element) => state.start(&element, &reader, limits)?,
            Event::Empty(element) => state.empty(&element, &reader, limits)?,
            Event::End(_) => state.end()?,
            Event::Text(text) => {
                let decoded = text.decode().map_err(|error| error.to_string())?;
                state.text(&decoded, text.len(), limits)?;
            }
            Event::CData(text) => {
                let decoded = text.decode().map_err(|error| error.to_string())?;
                state.text(&decoded, text.len(), limits)?;
            }
            Event::GeneralRef(reference) => state.reference(&reference)?,
            Event::Decl(_) | Event::DocType(_) => {
                return Err("XML declarations, DTDs, and entities are not allowed".to_string())
            }
            Event::Comment(_) | Event::PI(_) if state.depth > 0 => {}
            Event::Comment(_) | Event::PI(_) => {
                return Err("content outside the raw fragment root is not allowed".to_string())
            }
            Event::Eof => break,
        }
    }
    state.finish()
}

#[derive(Default)]
struct SubtreeState {
    depth: usize,
    roots: usize,
    root_name: Option<String>,
}

impl SubtreeState {
    fn start(
        &mut self,
        element: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
        limits: &HmlLimits,
    ) -> Result<(), String> {
        if self.depth >= limits.max_depth {
            return Err("raw fragment exceeds the XML depth limit".to_string());
        }
        if self.depth == 0 {
            self.note_root(element)?;
        }
        validate_attributes(element, reader, limits)?;
        self.depth += 1;
        Ok(())
    }

    fn empty(
        &mut self,
        element: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
        limits: &HmlLimits,
    ) -> Result<(), String> {
        if self.depth >= limits.max_depth {
            return Err("raw fragment exceeds the XML depth limit".to_string());
        }
        if self.depth == 0 {
            self.note_root(element)?;
        }
        validate_attributes(element, reader, limits)
    }

    fn note_root(&mut self, element: &BytesStart<'_>) -> Result<(), String> {
        self.roots += 1;
        if self.roots > 1 {
            return Err("raw fragment must contain exactly one root subtree".to_string());
        }
        self.root_name = Some(
            std::str::from_utf8(element.name().as_ref())
                .map_err(|_| "raw fragment root name is not UTF-8".to_string())?
                .to_string(),
        );
        Ok(())
    }

    fn end(&mut self) -> Result<(), String> {
        if self.depth == 0 {
            return Err("unexpected raw fragment closing element".to_string());
        }
        self.depth -= 1;
        Ok(())
    }

    fn text(&self, value: &str, bytes: usize, limits: &HmlLimits) -> Result<(), String> {
        if bytes > limits.max_text_node_bytes {
            return Err("raw fragment text exceeds the text-node limit".to_string());
        }
        if self.depth == 0 && !value.chars().all(char::is_whitespace) {
            return Err("non-whitespace text outside the raw fragment root".to_string());
        }
        validate_xml_chars(value)
    }

    fn reference(&self, reference: &quick_xml::events::BytesRef<'_>) -> Result<(), String> {
        if self.depth == 0 {
            return Err("entity reference outside the raw fragment root".to_string());
        }
        if let Some(character) = reference
            .resolve_char_ref()
            .map_err(|error| error.to_string())?
        {
            return validate_xml_chars(&character.to_string());
        }
        let name = reference.decode().map_err(|error| error.to_string())?;
        if matches!(name.as_ref(), "lt" | "gt" | "amp" | "quot" | "apos") {
            Ok(())
        } else {
            Err(format!("unsupported entity reference &{name};"))
        }
    }

    fn finish(self) -> Result<String, String> {
        if self.depth != 0 || self.roots != 1 {
            return Err("raw fragment must be one complete XML subtree".to_string());
        }
        self.root_name
            .ok_or_else(|| "raw fragment root is missing".to_string())
    }
}

fn validate_attributes(
    element: &BytesStart<'_>,
    reader: &Reader<&[u8]>,
    limits: &HmlLimits,
) -> Result<(), String> {
    let mut count = 0usize;
    for attribute in element.attributes().with_checks(true) {
        count += 1;
        if count > limits.max_attributes {
            return Err("raw fragment exceeds the attribute limit".to_string());
        }
        let attribute = attribute.map_err(|error| error.to_string())?;
        let value = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, reader.decoder())
            .map_err(|error| error.to_string())?;
        validate_xml_chars(&value)?;
    }
    Ok(())
}

pub(crate) fn validate_xml_chars(value: &str) -> Result<(), String> {
    if value.chars().all(is_xml_1_0_char) {
        Ok(())
    } else {
        Err("value contains a character forbidden by XML 1.0".to_string())
    }
}

fn is_xml_1_0_char(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{a}' | '\u{d}')
        || ('\u{20}'..='\u{d7ff}').contains(&character)
        || ('\u{e000}'..='\u{fffd}').contains(&character)
        || ('\u{10000}'..='\u{10ffff}').contains(&character)
}
