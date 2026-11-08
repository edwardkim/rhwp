//! Bounded form action sources and addressed updates shared by Native/WASM hosts.
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::{control::Control, form_script::scripts, paragraph::Paragraph};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MAX_SCRIPT_BYTES: usize = 1_048_576;
fn invalid() -> HwpError {
    HwpError::InvalidField("FORM_ACTION_INVALID: invalid or excessive form action".into())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FormAddress {
    pub section: usize,
    pub para: usize,
    pub control: usize,
    #[serde(default)]
    pub cell_path: Vec<(usize, usize, usize)>,
}

fn collect(
    paragraph: &Paragraph,
    address: FormAddress,
    out: &mut Vec<Value>,
    visited: &mut usize,
    bytes_left: &mut usize,
) -> Result<(), HwpError> {
    *visited += 1;
    if *visited > 100_000 || address.cell_path.len() > 32 || out.len() > 4096 {
        return Err(invalid());
    }
    for (index, control) in paragraph.controls.iter().enumerate() {
        match control {
            Control::Form(form) => {
                let cost = form
                    .name
                    .len()
                    .checked_add(form.text.len())
                    .and_then(|v| v.checked_add(form.caption.len()))
                    .and_then(|v| {
                        v.checked_add(form.properties.get("_rhwp_items").map_or(0, String::len))
                    })
                    .ok_or_else(invalid)?;
                *bytes_left = bytes_left.checked_sub(cost).ok_or_else(invalid)?;
                if out.len() >= 4096 {
                    return Err(invalid());
                }
                let mut target = address.clone();
                target.control = index;
                out.push(json!({"address":target, "name":form.name, "formType":form.form_type,
                    "text":form.text,"caption":form.caption,"value":form.value,"enabled":form.enabled,
                    "items": form.properties.get("_rhwp_items").and_then(|s| serde_json::from_str::<Vec<String>>(s).ok()).unwrap_or_default()}));
            }
            Control::Table(table) => {
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    for (para_index, child) in cell.paragraphs.iter().enumerate() {
                        let mut target = address.clone();
                        target.cell_path.push((index, cell_index, para_index));
                        collect(child, target, out, visited, bytes_left)?;
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(textbox) = crate::document_core::helpers::get_textbox_from_shape(shape)
                {
                    for (para_index, child) in textbox.paragraphs.iter().enumerate() {
                        let mut target = address.clone();
                        target.cell_path.push((index, 0, para_index));
                        collect(child, target, out, visited, bytes_left)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Update {
    address: FormAddress,
    text: String,
    caption: String,
    value: i32,
    enabled: bool,
    items: Vec<String>,
}

impl DocumentCore {
    pub fn form_action_context_native(&self, address: &FormAddress) -> Result<String, HwpError> {
        let paragraph = if address.cell_path.is_empty() {
            self.document
                .sections
                .get(address.section)
                .and_then(|s| s.paragraphs.get(address.para))
                .ok_or_else(invalid)?
        } else {
            self.resolve_paragraph_by_path(address.section, address.para, &address.cell_path)?
        };
        let Some(Control::Form(button)) = paragraph.controls.get(address.control) else {
            return Err(invalid());
        };
        if button.form_type != crate::model::control::FormType::PushButton || !button.enabled {
            return Err(invalid());
        }
        let (header, source) = scripts(&self.document)?;
        let mut forms = Vec::new();
        let mut visited = 0;
        let mut bytes_left = MAX_SCRIPT_BYTES;
        for (section, data) in self.document.sections.iter().enumerate() {
            for (para, paragraph) in data.paragraphs.iter().enumerate() {
                collect(
                    paragraph,
                    FormAddress {
                        section,
                        para,
                        control: 0,
                        cell_path: Vec::new(),
                    },
                    &mut forms,
                    &mut visited,
                    &mut bytes_left,
                )?;
            }
        }
        let trigger = button
            .properties
            .get("OnClick")
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| format!("On{}_Click", button.name));
        let result = json!({"schemaVersion":1,"button":address,"trigger":trigger,"header":header,"source":source,"forms":forms}).to_string();
        if result.len() > 4 * MAX_SCRIPT_BYTES {
            return Err(invalid());
        }
        Ok(result)
    }

    /// All targets and values are checked before any change; paths never fall back to body.
    pub fn apply_form_action_updates_native(&mut self, updates_json: &str) -> Result<(), HwpError> {
        if updates_json.len() > MAX_SCRIPT_BYTES {
            return Err(invalid());
        }
        let updates: Vec<Update> = serde_json::from_str(updates_json).map_err(|_| invalid())?;
        if updates.len() > 512 {
            return Err(invalid());
        }
        let mut prepared = Vec::new();
        for update in updates {
            let address = &update.address;
            if address.cell_path.len() > 32
                || update.text.len() > 65536
                || update.caption.len() > 65536
                || update.items.len() > 512
                || update.items.iter().map(String::len).sum::<usize>() > 65536
            {
                return Err(invalid());
            }
            let paragraph = if address.cell_path.is_empty() {
                self.document
                    .sections
                    .get(address.section)
                    .and_then(|s| s.paragraphs.get(address.para))
                    .ok_or_else(invalid)?
            } else {
                self.resolve_paragraph_by_path(address.section, address.para, &address.cell_path)?
            };
            let Some(Control::Form(form)) = paragraph.controls.get(address.control) else {
                return Err(invalid());
            };
            let _ = form;
            prepared.push(update);
        }
        let mut sections = std::collections::BTreeSet::new();
        for update in prepared {
            let address = update.address;
            let paragraph = if address.cell_path.is_empty() {
                self.document
                    .sections
                    .get_mut(address.section)
                    .and_then(|s| s.paragraphs.get_mut(address.para))
                    .ok_or_else(invalid)?
            } else {
                self.get_cell_paragraph_mut_by_path(
                    address.section,
                    address.para,
                    &address.cell_path,
                )?
            };
            let Control::Form(form) = &mut paragraph.controls[address.control] else {
                return Err(invalid());
            };
            form.text = update.text;
            form.caption = update.caption;
            form.value = update.value;
            form.enabled = update.enabled;
            if form.form_type == crate::model::control::FormType::ComboBox {
                form.properties.insert(
                    "_rhwp_items".into(),
                    serde_json::to_string(&update.items).map_err(|_| invalid())?,
                );
            }
            sections.insert(address.section);
        }
        for section in sections {
            self.document.sections[section].raw_stream = None;
            self.recompose_section(section);
        }
        Ok(())
    }
}
