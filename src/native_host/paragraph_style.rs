//! Named paragraph styles for an editable body or top-level table-cell paragraph.

use serde::Serialize;

use crate::document_core::native_interaction::NativeTextSelection;
use crate::model::{document::DocInfo, paragraph::Paragraph, style::Style};

use super::{editor::NativeEditOutcome, editor::NativeEditor, NativeHostError};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeParagraphStyle {
    pub id: u8,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeParagraphStyles {
    pub current_style_id: Option<u8>,
    pub styles: Vec<NativeParagraphStyle>,
}

impl NativeParagraphStyles {
    pub(super) fn retained_bytes(&self) -> usize {
        self.styles
            .iter()
            .map(|style| style.name.capacity())
            .sum::<usize>()
            + self.styles.capacity() * std::mem::size_of::<NativeParagraphStyle>()
    }
}

impl NativeEditor {
    pub fn paragraph_styles(
        &self,
        expected_revision: u64,
        selection: &NativeTextSelection,
    ) -> Result<NativeParagraphStyles, NativeHostError> {
        self.check_edit_ready(expected_revision)?;
        if selection != self.selection() {
            return Err(NativeHostError::InvalidArgument("stale selection".into()));
        }
        let paragraph = self.paragraph_style_paragraph()?;
        let doc_info = &self.session().document().document().doc_info;
        let styles = (0..=u8::MAX)
            .map_while(|id| doc_info.styles.get(id as usize).map(|style| (id, style)))
            .filter(|(_, style)| is_applicable_style(doc_info, style))
            .map(|(id, style)| {
                let name = if style.local_name.trim().is_empty() {
                    &style.english_name
                } else {
                    &style.local_name
                };
                NativeParagraphStyle {
                    id,
                    name: name.clone(),
                }
            })
            .collect();
        let current_style_id = doc_info
            .styles
            .get(paragraph.style_id as usize)
            .filter(|style| is_applicable_style(doc_info, style))
            .map(|_| paragraph.style_id);
        Ok(NativeParagraphStyles {
            current_style_id,
            styles,
        })
    }

    pub(super) fn apply_paragraph_style(
        &mut self,
        revision: u64,
        style_id: u8,
    ) -> Result<NativeEditOutcome, NativeHostError> {
        let paragraph = self.paragraph_style_paragraph()?;
        let doc_info = &self.session().document().document().doc_info;
        let style_id = style_id as usize;
        if !doc_info
            .styles
            .get(style_id)
            .is_some_and(|style| is_applicable_style(doc_info, style))
        {
            return Err(NativeHostError::InvalidArgument(
                "unknown or unavailable paragraph style".into(),
            ));
        }
        if paragraph.style_id as usize == style_id {
            return Ok(self.outcome(false, false));
        }

        let selection = self.selection().clone();
        self.edit_transaction(revision, |core| {
            let focus = &selection.focus;
            let section = focus.section as usize;
            let paragraph = focus.paragraph as usize;
            if let Some(cell) = focus.cell_path.first() {
                core.apply_cell_style_native(
                    section,
                    paragraph,
                    cell.control as usize,
                    cell.cell as usize,
                    cell.paragraph as usize,
                    style_id,
                )?;
            } else {
                core.apply_style_native(section, paragraph, style_id)?;
            }
            Ok(selection)
        })
    }

    fn paragraph_style_paragraph(&self) -> Result<&Paragraph, NativeHostError> {
        if self.selection().focus.cell_path.len() > 1 {
            return Err(NativeHostError::Unsupported(
                "paragraph styles require body text or a top-level table cell".into(),
            ));
        }
        self.formatting_paragraph()
    }
}

fn is_applicable_style(doc_info: &DocInfo, style: &Style) -> bool {
    style.style_type == 0
        && (style.para_shape_id as usize) < doc_info.para_shapes.len()
        && (style.char_shape_id as usize) < doc_info.char_shapes.len()
}
