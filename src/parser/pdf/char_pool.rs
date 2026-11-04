//! Share exactly equal generated character records across PDF pages.
//! The IR uses u32 references, but saved HWPX readers have a bounded resource
//! domain. Repeating identical records must not exhaust that domain.
use super::{PdfImportError, MAX_POOL_ENTRIES};
use crate::model::{control::Control, document::Document, paragraph::Paragraph};
use std::collections::BTreeMap;

pub(super) struct Pool(
    BTreeMap<Vec<u8>, u32>,
    std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
);

impl Pool {
    pub fn new(
        doc: &Document,
        budget: std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
    ) -> Result<Self, PdfImportError> {
        let mut pool = Self(BTreeMap::new(), budget);
        for (id, shape) in doc.doc_info.char_shapes.iter().enumerate() {
            // Include every field, including raw provenance: equality is not
            // inferred from a subset of font/size/spacing properties.
            let allocation = super::allocation::Allocation(&pool.1);
            let key = allocation.json(shape, pool.1.limits.stream_bytes)?;
            if let std::collections::btree_map::Entry::Vacant(entry) = pool.0.entry(key) {
                allocation.tree_entry::<Vec<u8>, u32>()?;
                entry.insert(id as u32);
            }
        }
        Ok(pool)
    }

    pub fn compact_page(
        &mut self,
        doc: &mut Document,
        first: usize,
        section: usize,
    ) -> Result<(), PdfImportError> {
        let allocation = super::allocation::Allocation(&self.1);
        let count = doc
            .doc_info
            .char_shapes
            .len()
            .checked_sub(first)
            .ok_or_else(|| PdfImportError::budget("character shape split range invalid"))?;
        // Move owned shapes into a fallibly reserved buffer. Their strings/
        // arrays move too; no payload clone and no infallible split_off allocation.
        let mut added = allocation.vec(count)?;
        added.extend(doc.doc_info.char_shapes.drain(first..));
        let mut remap = allocation.vec(added.len())?;
        for shape in added {
            let key = allocation.json(&shape, self.1.limits.stream_bytes)?;
            let id = if let Some(id) = self.0.get(&key) {
                *id
            } else {
                if doc.doc_info.char_shapes.len() >= MAX_POOL_ENTRIES {
                    return Err(PdfImportError::budget("character shape ID budget exceeded"));
                }
                let id = doc.doc_info.char_shapes.len() as u32;
                doc.doc_info.char_shapes.push(shape);
                allocation.tree_entry::<Vec<u8>, u32>()?;
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
