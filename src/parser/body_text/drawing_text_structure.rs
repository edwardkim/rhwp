//! Validate unambiguous missing owned text structure before tolerant paragraph parsing.
use super::BodyTextError;
use crate::parser::{record::Record, tags};

struct Owner {
    level: u16,
    rectangle: bool,
    declared: Option<u32>,
    paragraphs: usize,
}

impl Owner {
    fn finish_list(&self) -> Result<(), BodyTextError> {
        if let Some(declared) = self.declared {
            // #6856: Hancom rejects the maintainer-checked zero-paragraph fixture.
            // No owned list is a rectangle; an owned list must declare a paragraph.
            if declared == 0 || u64::from(declared) > self.paragraphs as u64 {
                return Err(BodyTextError::DrawingTextStructure(format!(
                    "rectangle at level {} declares {} paragraphs, found {}",
                    self.level, declared, self.paragraphs
                )));
            }
        }
        Ok(())
    }
}

/// One bounded stack over already-decoded records; no recursive reparsing or
/// allocation based on the document's declared paragraph count.
pub(super) fn validate(records: &[Record]) -> Result<(), BodyTextError> {
    let mut owners: Vec<Owner> = Vec::new();
    for record in records {
        while owners
            .last()
            .is_some_and(|owner| owner.level >= record.level)
        {
            owners.pop().unwrap().finish_list()?;
        }
        if record.tag_id == tags::HWPTAG_SHAPE_COMPONENT {
            let rectangle =
                record.data.get(..4) == Some(tags::SHAPE_RECT_ID.to_le_bytes().as_slice());
            owners.push(Owner {
                level: record.level,
                rectangle,
                declared: None,
                paragraphs: 0,
            });
            continue;
        }
        let Some(owner) = owners.last_mut() else {
            continue;
        };
        if !owner.rectangle || record.level != owner.level + 1 {
            continue;
        }
        match record.tag_id {
            tags::HWPTAG_LIST_HEADER => {
                owner.finish_list()?;
                // HWP5 table 65: the shared list header needs at least 2+4
                // bytes. Do not require the 33-byte extension seen in our sample.
                if record.data.len() < 6 {
                    return Err(BodyTextError::DrawingTextStructure(format!(
                        "rectangle LIST_HEADER at level {} is truncated: {} bytes",
                        record.level,
                        record.data.len()
                    )));
                }
                // Match the drawing parser/writer's UINT32 count layout. Reading
                // only the low 16 bits would misclassify 65,536 as zero.
                owner.declared = Some(u32::from_le_bytes(
                    record.data[..4].try_into().expect("minimum header checked"),
                ));
                owner.paragraphs = 0;
            }
            tags::HWPTAG_PARA_HEADER if owner.declared.is_some() => owner.paragraphs += 1,
            _ => {}
        }
    }
    for owner in owners {
        owner.finish_list()?;
    }
    Ok(())
}
