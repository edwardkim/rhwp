//! Share exactly equal generated character records across PDF pages.
//! The IR uses u32 references, but saved HWPX readers have a bounded resource
//! domain. Repeating identical records must not exhaust that domain.
use super::{PdfImportError, MAX_POOL_ENTRIES};
use crate::model::{control::Control, document::Document, paragraph::Paragraph};
use std::collections::BTreeMap;

pub(super) struct Pool(BTreeMap<Vec<u8>, u32>);

impl Pool {
    pub fn new(doc: &Document) -> Result<Self, PdfImportError> {
        let mut pool = Self(BTreeMap::new());
        for (id, shape) in doc.doc_info.char_shapes.iter().enumerate() {
            // Include every field, including raw provenance: equality is not
            // inferred from a subset of font/size/spacing properties.
            let key = serde_json::to_vec(shape)
                .map_err(|_| PdfImportError::budget("character shape key failed"))?;
            pool.0.entry(key).or_insert(id as u32);
        }
        Ok(pool)
    }

    pub fn compact_page(
        &mut self,
        doc: &mut Document,
        first: usize,
        section: usize,
    ) -> Result<(), PdfImportError> {
        let added = doc.doc_info.char_shapes.split_off(first);
        let mut remap = Vec::with_capacity(added.len());
        for shape in added {
            let key = serde_json::to_vec(&shape)
                .map_err(|_| PdfImportError::budget("character shape key failed"))?;
            let id = if let Some(id) = self.0.get(&key) {
                *id
            } else {
                if doc.doc_info.char_shapes.len() >= MAX_POOL_ENTRIES {
                    return Err(PdfImportError::budget("character shape ID budget exceeded"));
                }
                let id = doc.doc_info.char_shapes.len() as u32;
                doc.doc_info.char_shapes.push(shape);
                self.0.insert(key, id);
                id
            };
            remap.push(id);
        }
        fn paragraphs(paras: &mut [Paragraph], first: usize, ids: &[u32]) {
            for para in paras {
                for run in &mut para.char_shapes {
                    if run.char_shape_id as usize >= first {
                        run.char_shape_id = ids[run.char_shape_id as usize - first];
                    }
                }
                for control in &mut para.controls {
                    match control {
                        Control::Table(table) => {
                            for cell in &mut table.cells {
                                paragraphs(&mut cell.paragraphs, first, ids);
                            }
                        }
                        Control::Shape(shape) => {
                            if let Some(text) = shape
                                .drawing_mut()
                                .and_then(|drawing| drawing.text_box.as_mut())
                            {
                                paragraphs(&mut text.paragraphs, first, ids);
                            }
                        }
                        // PDF lowering emits pictures and the above editable
                        // owners; it does not create notes or numbered styles.
                        _ => {}
                    }
                }
            }
        }
        paragraphs(&mut doc.sections[section].paragraphs, first, &remap);
        Ok(())
    }
}
